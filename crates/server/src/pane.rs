use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{Context, Result};
use gband_core::input::{Key, Modes, encode_key, encode_paste};
use portable_pty::{
    ChildKiller, CommandBuilder, ExitStatus, MasterPty, PtySize, native_pty_system,
};
use tokio::sync::{mpsc, watch};

use crate::callbacks::LoggingCallbacks;

const INITIAL_COLS: u16 = 80;
const INITIAL_ROWS: u16 = 24;

pub enum Input {
    Key(Key),
    Paste(String),
}

pub struct Pane {
    terminal: Mutex<Terminal>,
    generation: watch::Sender<u64>,
}

struct Terminal {
    parser: vt100::Parser<LoggingCallbacks>,
    master: Box<dyn MasterPty + Send>,
}

pub struct Session {
    pub pane: Arc<Pane>,
    pub input: mpsc::UnboundedSender<Input>,
    pub exit: watch::Receiver<Option<ExitStatus>>,
    pub drained: watch::Receiver<bool>,
    pub killer: Box<dyn ChildKiller + Send + Sync>,
    pub pid: Option<u32>,
}

impl Pane {
    pub fn screen(&self) -> vt100::Screen {
        self.terminal.lock().unwrap().parser.screen().clone()
    }

    pub fn modes(&self) -> Modes {
        let terminal = self.terminal.lock().unwrap();
        let screen = terminal.parser.screen();
        Modes {
            application_cursor: screen.application_cursor(),
            bracketed_paste: screen.bracketed_paste(),
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.generation.subscribe()
    }

    pub fn foreground_group(&self) -> Option<i32> {
        self.terminal.lock().unwrap().master.process_group_leader()
    }

    pub fn resize(&self, cols: u16, rows: u16) {
        if cols == 0 || rows == 0 {
            return;
        }
        {
            let mut terminal = self.terminal.lock().unwrap();
            if terminal.parser.screen().size() == (rows, cols) {
                return;
            }
            if let Err(error) = terminal.master.resize(pty_size(cols, rows)) {
                tracing::warn!("cannot resize the PTY to {cols}x{rows}: {error:#}");
                return;
            }
            terminal.parser.screen_mut().set_size(rows, cols);
        }
        tracing::debug!("PTY resized to {cols}x{rows}");
        self.bump();
    }

    fn process(&self, bytes: &[u8]) {
        self.terminal.lock().unwrap().parser.process(bytes);
        self.bump();
    }

    fn bump(&self) {
        self.generation.send_modify(|generation| *generation += 1);
    }
}

pub fn spawn(program: &[OsString], cwd: &Path, socket: &Path) -> Result<Session> {
    let pair = native_pty_system()
        .openpty(pty_size(INITIAL_COLS, INITIAL_ROWS))
        .context("cannot open a PTY")?;

    let mut command = CommandBuilder::from_argv(program.to_vec());
    command.env("TERM", "xterm-256color");
    command.env("COLORTERM", "truecolor");
    command.env("GBAND", socket);
    command.cwd(cwd);
    let mut child = pair
        .slave
        .spawn_command(command)
        .with_context(|| format!("cannot start {}", display_argv(program)))?;
    drop(pair.slave);
    let pid = child.process_id();
    let killer = child.clone_killer();
    tracing::info!(pid, "started {}", display_argv(program));

    let reader = pair
        .master
        .try_clone_reader()
        .context("cannot read the PTY")?;
    let writer = pair.master.take_writer().context("cannot write the PTY")?;
    let pane = Arc::new(Pane {
        terminal: Mutex::new(Terminal {
            parser: vt100::Parser::new_with_callbacks(
                INITIAL_ROWS,
                INITIAL_COLS,
                0,
                LoggingCallbacks,
            ),
            master: pair.master,
        }),
        generation: watch::Sender::new(0),
    });

    let (drained_tx, drained) = watch::channel(false);
    let output_pane = Arc::clone(&pane);
    thread::spawn(move || read_output(reader, &output_pane, &drained_tx));

    let (exit_tx, exit) = watch::channel(None);
    thread::spawn(move || {
        let status = child.wait().unwrap_or_else(|error| {
            tracing::warn!("cannot wait for the program: {error:#}");
            ExitStatus::with_exit_code(1)
        });
        exit_tx.send_replace(Some(status));
    });

    let input = spawn_input(Arc::clone(&pane), writer);
    Ok(Session {
        pane,
        input,
        exit,
        drained,
        killer,
        pid,
    })
}

fn read_output(mut reader: Box<dyn Read + Send>, pane: &Pane, drained: &watch::Sender<bool>) {
    let mut buffer = vec![0; 64 * 1024];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(n) => pane.process(&buffer[..n]),
        }
    }
    drained.send_replace(true);
}

fn spawn_input(pane: Arc<Pane>, mut writer: Box<dyn Write + Send>) -> mpsc::UnboundedSender<Input> {
    let (sender, mut receiver) = mpsc::unbounded_channel();
    thread::spawn(move || {
        while let Some(input) = receiver.blocking_recv() {
            let modes = pane.modes();
            let bytes = match input {
                Input::Key(key) => encode_key(key, modes),
                Input::Paste(text) => encode_paste(&text, modes),
            };
            if let Err(error) = writer.write_all(&bytes).and_then(|()| writer.flush()) {
                tracing::warn!("cannot write to the PTY: {error:#}");
                break;
            }
        }
    });
    sender
}

fn pty_size(cols: u16, rows: u16) -> PtySize {
    PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn display_argv(program: &[OsString]) -> String {
    program
        .iter()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}
