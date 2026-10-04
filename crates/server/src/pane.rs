use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc as std_mpsc};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use gband_core::geometry::Size;
use gband_core::input::{Key, Modes, encode_key, encode_paste};
use gband_core::layout::PaneId;
use gband_protocol::SessionName;
use portable_pty::{
    ChildKiller, CommandBuilder, ExitStatus, MasterPty, PtySize, native_pty_system,
};
use tokio::sync::{mpsc, watch};
use tracing::Span;

use crate::callbacks::PaneCallbacks;

const DRAIN_TIMEOUT: Duration = Duration::from_millis(500);

pub enum Input {
    Key(Key),
    Paste(String),
    Reply(Vec<u8>),
}

pub struct Pane {
    terminal: Mutex<Terminal>,
    generation: AtomicU64,
    changed: Arc<watch::Sender<u64>>,
    replies: mpsc::WeakUnboundedSender<Input>,
}

struct Terminal {
    parser: vt100::Parser<PaneCallbacks>,
    master: Box<dyn MasterPty + Send>,
}

#[derive(Clone)]
pub struct PaneEntry {
    pub pane: Arc<Pane>,
    pub input: mpsc::UnboundedSender<Input>,
}

pub struct Spawned {
    pub entry: PaneEntry,
    pub killer: Box<dyn ChildKiller + Send + Sync>,
    pub pid: Option<u32>,
    pub exit: watch::Receiver<Option<ExitStatus>>,
}

pub struct PaneExit {
    pub pane: PaneId,
    pub status: ExitStatus,
}

pub struct SpawnRequest<'a> {
    pub id: PaneId,
    pub program: &'a [OsString],
    pub cwd: &'a Path,
    pub socket: &'a Path,
    pub session: &'a SessionName,
    pub size: Size,
}

impl Pane {
    pub fn screen(&self) -> (u64, vt100::Screen) {
        let terminal = self.terminal.lock().unwrap();
        (
            self.generation.load(Ordering::Acquire),
            terminal.parser.screen().clone(),
        )
    }

    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub fn modes(&self) -> Modes {
        let terminal = self.terminal.lock().unwrap();
        let screen = terminal.parser.screen();
        Modes {
            application_cursor: screen.application_cursor(),
            bracketed_paste: screen.bracketed_paste(),
        }
    }

    pub fn foreground_group(&self) -> Option<i32> {
        self.terminal.lock().unwrap().master.process_group_leader()
    }

    pub fn resize(&self, size: Size) {
        let Size { cols, rows } = size;
        if cols == 0 || rows == 0 {
            return;
        }
        {
            let mut terminal = self.terminal.lock().unwrap();
            if terminal.parser.screen().size() == (rows, cols) {
                return;
            }
            if let Err(error) = terminal.master.resize(pty_size(size)) {
                tracing::warn!("cannot resize the PTY to {cols}x{rows}: {error:#}");
                return;
            }
            terminal.parser.screen_mut().set_size(rows, cols);
            self.generation.fetch_add(1, Ordering::AcqRel);
        }
        tracing::debug!("PTY resized to {cols}x{rows}");
        self.notify();
    }

    fn process(&self, bytes: &[u8]) {
        {
            let mut terminal = self.terminal.lock().unwrap();
            terminal.parser.process(bytes);
            self.generation.fetch_add(1, Ordering::AcqRel);
            let replies = std::mem::take(&mut terminal.parser.callbacks_mut().replies);
            if !replies.is_empty()
                && let Some(sender) = self.replies.upgrade()
            {
                let _ = sender.send(Input::Reply(replies));
            }
        }
        self.notify();
    }

    fn notify(&self) {
        self.changed.send_modify(|generation| *generation += 1);
    }
}

pub fn spawn(
    request: SpawnRequest<'_>,
    changed: &Arc<watch::Sender<u64>>,
    exits: mpsc::UnboundedSender<PaneExit>,
) -> Result<Spawned> {
    let SpawnRequest {
        id,
        program,
        cwd,
        socket,
        session,
        size,
    } = request;
    let pair = native_pty_system()
        .openpty(pty_size(size))
        .context("cannot open a PTY")?;

    let mut command = CommandBuilder::from_argv(program.to_vec());
    command.env("TERM", "xterm-256color");
    command.env("COLORTERM", "truecolor");
    command.env("GBAND", socket);
    command.env("GBAND_SESSION", session.as_str());
    command.env("GBAND_PANE", id.to_string());
    command.cwd(cwd);
    let mut child = pair
        .slave
        .spawn_command(command)
        .with_context(|| format!("cannot start {}", display_argv(program)))?;
    drop(pair.slave);
    let pid = child.process_id();
    let killer = child.clone_killer();
    tracing::info!(pane = %id, pid, "started {}", display_argv(program));

    let reader = pair
        .master
        .try_clone_reader()
        .context("cannot read the PTY")?;
    let writer = pair.master.take_writer().context("cannot write the PTY")?;
    let (input, receiver) = mpsc::unbounded_channel();
    let pane = Arc::new(Pane {
        terminal: Mutex::new(Terminal {
            parser: vt100::Parser::new_with_callbacks(
                size.rows,
                size.cols,
                0,
                PaneCallbacks::default(),
            ),
            master: pair.master,
        }),
        generation: AtomicU64::new(0),
        changed: Arc::clone(changed),
        replies: input.downgrade(),
    });

    let (drained_tx, drained) = std_mpsc::channel::<()>();
    let output_pane = Arc::clone(&pane);
    thread::spawn(move || {
        read_output(reader, &output_pane);
        drop(drained_tx);
    });

    let (exit_tx, exit) = watch::channel(None);
    let span = Span::current();
    thread::spawn(move || {
        let _span = span.enter();
        let status = child.wait().unwrap_or_else(|error| {
            tracing::warn!("cannot wait for the program: {error:#}");
            ExitStatus::with_exit_code(1)
        });
        exit_tx.send_replace(Some(status.clone()));
        let _ = drained.recv_timeout(DRAIN_TIMEOUT);
        let _ = exits.send(PaneExit { pane: id, status });
    });

    spawn_input(Arc::clone(&pane), receiver, writer);
    Ok(Spawned {
        entry: PaneEntry { pane, input },
        killer,
        pid,
        exit,
    })
}

fn read_output(mut reader: Box<dyn Read + Send>, pane: &Pane) {
    let mut buffer = vec![0; 64 * 1024];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(n) => pane.process(&buffer[..n]),
        }
    }
}

fn spawn_input(
    pane: Arc<Pane>,
    mut receiver: mpsc::UnboundedReceiver<Input>,
    mut writer: Box<dyn Write + Send>,
) {
    let span = Span::current();
    thread::spawn(move || {
        let _span = span.enter();
        while let Some(input) = receiver.blocking_recv() {
            let bytes = match input {
                Input::Key(key) => encode_key(key, pane.modes()),
                Input::Paste(text) => encode_paste(&text, pane.modes()),
                Input::Reply(bytes) => bytes,
            };
            if let Err(error) = writer.write_all(&bytes).and_then(|()| writer.flush()) {
                tracing::warn!("cannot write to the PTY: {error:#}");
                break;
            }
        }
    });
}

fn pty_size(size: Size) -> PtySize {
    PtySize {
        rows: size.rows,
        cols: size.cols,
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
