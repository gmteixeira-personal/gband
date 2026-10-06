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
use gband_core::layout::WindowId;
use gband_emulator::{Emulator, Grid};
use gband_protocol::SessionName;
use gband_protocol::test::SOCKET_VARIABLE;
use portable_pty::{
    ChildKiller, CommandBuilder, ExitStatus, MasterPty, PtySize, native_pty_system,
};
use tokio::sync::{mpsc, watch};
use tracing::Span;

use crate::scripting::Taps;

const DRAIN_TIMEOUT: Duration = Duration::from_millis(500);

pub type Checkpoint = <Grid as Emulator>::Checkpoint;

pub enum Input {
    Key(Key),
    Paste(String),
    WriteBack(Vec<u8>),
}

pub struct Window {
    terminal: Mutex<Terminal>,
    generation: AtomicU64,
    changed: Arc<watch::Sender<u64>>,
    write_back: mpsc::WeakUnboundedSender<Input>,
}

struct Terminal {
    grid: Grid,
    master: Option<Box<dyn MasterPty + Send>>,
}

pub struct Seen {
    size: Size,
    checkpoint: Checkpoint,
}

pub enum Contents {
    Snapshot(Size, Vec<u8>),
    Update(Vec<u8>),
}

#[derive(Clone)]
pub struct WindowEntry {
    pub window: Arc<Window>,
    pub input: mpsc::UnboundedSender<Input>,
}

pub struct Spawned {
    pub entry: WindowEntry,
    pub killer: Box<dyn ChildKiller + Send + Sync>,
    pub pid: Option<u32>,
    pub exit: watch::Receiver<Option<ExitStatus>>,
}

pub struct WindowExit {
    pub window: WindowId,
    pub status: ExitStatus,
}

pub struct SpawnRequest<'a> {
    pub id: WindowId,
    pub program: &'a [OsString],
    pub cwd: &'a Path,
    pub socket: &'a Path,
    pub session: &'a SessionName,
    pub size: Size,
    pub taps: &'a Arc<Taps>,
}

impl Window {
    pub fn blank(size: Size, changed: &Arc<watch::Sender<u64>>) -> WindowEntry {
        let (input, mut discarded) = mpsc::unbounded_channel();
        tokio::spawn(async move { while discarded.recv().await.is_some() {} });
        let window = Arc::new(Window {
            terminal: Mutex::new(Terminal {
                grid: Grid::new(size),
                master: None,
            }),
            generation: AtomicU64::new(0),
            changed: Arc::clone(changed),
            write_back: input.downgrade(),
        });
        WindowEntry { window, input }
    }

    pub fn replace(&self, output: &[u8]) {
        {
            let mut terminal = self.terminal.lock().unwrap();
            let mut grid = Grid::new(terminal.grid.size());
            grid.process(output);
            grid.take_write_back();
            terminal.grid = grid;
            self.generation.fetch_add(1, Ordering::AcqRel);
        }
        self.notify();
    }

    pub fn catch_up(&self, seen: Option<&Seen>) -> (u64, Seen, Contents) {
        let terminal = self.terminal.lock().unwrap();
        let grid = &terminal.grid;
        let size = grid.size();
        let contents = match seen {
            Some(seen) if seen.size == size => Contents::Update(grid.diff(&seen.checkpoint)),
            _ => Contents::Snapshot(size, grid.snapshot()),
        };
        let seen = Seen {
            size,
            checkpoint: grid.checkpoint(),
        };
        (self.generation.load(Ordering::Acquire), seen, contents)
    }

    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub fn modes(&self) -> Modes {
        self.terminal.lock().unwrap().grid.modes()
    }

    pub fn foreground_group(&self) -> Option<i32> {
        self.terminal
            .lock()
            .unwrap()
            .master
            .as_ref()?
            .process_group_leader()
    }

    pub fn resize(&self, size: Size) {
        let Size { cols, rows } = size;
        if cols == 0 || rows == 0 {
            return;
        }
        {
            let mut terminal = self.terminal.lock().unwrap();
            if terminal.grid.size() == size {
                return;
            }
            if let Some(master) = &terminal.master
                && let Err(error) = master.resize(pty_size(size))
            {
                tracing::warn!("cannot resize the PTY to {cols}x{rows}: {error:#}");
                return;
            }
            terminal.grid.resize(size);
            self.generation.fetch_add(1, Ordering::AcqRel);
        }
        tracing::debug!("PTY resized to {cols}x{rows}");
        self.notify();
    }

    fn process(&self, bytes: &[u8]) {
        {
            let mut terminal = self.terminal.lock().unwrap();
            terminal.grid.process(bytes);
            self.generation.fetch_add(1, Ordering::AcqRel);
            let write_back = terminal.grid.take_write_back();
            if !write_back.is_empty()
                && let Some(sender) = self.write_back.upgrade()
            {
                let _ = sender.send(Input::WriteBack(write_back));
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
    exits: mpsc::UnboundedSender<WindowExit>,
) -> Result<Spawned> {
    let SpawnRequest {
        id,
        program,
        cwd,
        socket,
        session,
        size,
        taps,
    } = request;
    let pair = native_pty_system()
        .openpty(pty_size(size))
        .context("cannot open a PTY")?;

    let mut command = CommandBuilder::from_argv(program.to_vec());
    command.env("TERM", "xterm-256color");
    command.env("COLORTERM", "truecolor");
    command.env("GBAND", socket);
    command.env("GBAND_SESSION", session.as_str());
    command.env("GBAND_WINDOW", id.to_string());
    command.env_remove(SOCKET_VARIABLE);
    command.cwd(cwd);
    let mut child = pair
        .slave
        .spawn_command(command)
        .with_context(|| format!("cannot start {}", display_argv(program)))?;
    drop(pair.slave);
    let pid = child.process_id();
    let killer = child.clone_killer();
    tracing::info!(window = %id, pid, "started {}", display_argv(program));

    let reader = pair
        .master
        .try_clone_reader()
        .context("cannot read the PTY")?;
    let writer = pair.master.take_writer().context("cannot write the PTY")?;
    let (input, receiver) = mpsc::unbounded_channel();
    let window = Arc::new(Window {
        terminal: Mutex::new(Terminal {
            grid: Grid::new(size),
            master: Some(pair.master),
        }),
        generation: AtomicU64::new(0),
        changed: Arc::clone(changed),
        write_back: input.downgrade(),
    });

    let (drained_tx, drained) = std_mpsc::channel::<()>();
    let output_window = Arc::clone(&window);
    let tap = Tap {
        taps: Arc::clone(taps),
        session: session.clone(),
        window: id,
    };
    thread::spawn(move || {
        read_output(reader, &output_window, &tap);
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
        let _ = exits.send(WindowExit { window: id, status });
    });

    spawn_input(Arc::clone(&window), receiver, writer);
    Ok(Spawned {
        entry: WindowEntry { window, input },
        killer,
        pid,
        exit,
    })
}

struct Tap {
    taps: Arc<Taps>,
    session: SessionName,
    window: WindowId,
}

fn read_output(mut reader: Box<dyn Read + Send>, window: &Window, tap: &Tap) {
    let mut buffer = vec![0; 64 * 1024];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                tap.taps.output(&tap.session, tap.window, &buffer[..n]);
                window.process(&buffer[..n]);
            }
        }
    }
}

fn spawn_input(
    window: Arc<Window>,
    mut receiver: mpsc::UnboundedReceiver<Input>,
    mut writer: Box<dyn Write + Send>,
) {
    let span = Span::current();
    thread::spawn(move || {
        let _span = span.enter();
        while let Some(input) = receiver.blocking_recv() {
            let bytes = match input {
                Input::Key(key) => encode_key(key, window.modes()),
                Input::Paste(text) => encode_paste(&text, window.modes()),
                Input::WriteBack(bytes) => bytes,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn blank(size: Size) -> WindowEntry {
        Window::blank(size, &Arc::new(watch::Sender::new(0)))
    }

    fn feed(grid: &mut Grid, contents: Contents) {
        match contents {
            Contents::Snapshot(size, bytes) => {
                *grid = Grid::new(size);
                grid.process(&bytes);
            }
            Contents::Update(bytes) => grid.process(&bytes),
        }
    }

    #[tokio::test]
    async fn replaced_screen_reaches_a_client_as_an_update() {
        let entry = blank(Size::new(20, 5));
        let mut client = Grid::new(Size::new(20, 5));
        entry.window.replace(b"one");
        let (_, seen, contents) = entry.window.catch_up(None);
        feed(&mut client, contents);
        entry.window.replace(b"\x1b[2;1Htwo");
        let (_, _, contents) = entry.window.catch_up(Some(&seen));
        assert!(matches!(contents, Contents::Update(_)));
        feed(&mut client, contents);
        let rows: Vec<String> = client.contents().lines().map(str::to_owned).collect();
        assert_eq!(rows, ["", "two"]);
    }

    #[tokio::test]
    async fn resize_keeps_the_cells_that_fit() {
        let entry = blank(Size::new(20, 5));
        entry.window.replace(b"hello\r\nworld");
        let before = entry.window.generation();
        entry.window.resize(Size::new(3, 1));
        assert!(entry.window.generation() > before);
        let (_, _, contents) = entry.window.catch_up(None);
        let mut client = Grid::new(Size::new(1, 1));
        feed(&mut client, contents);
        assert_eq!(client.size(), Size::new(3, 1));
        assert_eq!(client.contents(), "hel");
    }

    #[tokio::test]
    async fn input_to_a_blank_window_is_discarded() {
        let entry = blank(Size::new(20, 5));
        assert!(entry.input.send(Input::Paste("x".into())).is_ok());
        assert!(entry.window.foreground_group().is_none());
    }
}
