mod connect;
pub mod input;
pub mod prefix;

use std::io::stdout;
use std::path::PathBuf;
use std::thread;

use anyhow::{Context, Result};
use crossterm::cursor::Show;
use crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event};
use crossterm::execute;
use gband_protocol::{ClientMessage, ExecutableId, ServerMessage};
use ratatui::DefaultTerminal;
use ratatui::layout::Position;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tui_term::widget::{Cursor, PseudoTerminal};

pub use crate::connect::{Connection, connect};
use crate::input::key_from_event;
use crate::prefix::{Action, PrefixState};

const READ_BUFFER_LEN: usize = 64 * 1024;

pub struct ClientConfig {
    pub runtime_dir: PathBuf,
    pub executable_path: PathBuf,
    pub identity: ExecutableId,
    pub replace_mismatched: bool,
    pub log_dir: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Detached,
    Exited,
    LostServer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Report {
    pub outcome: Outcome,
    pub stale_server: bool,
}

pub fn run(config: ClientConfig) -> Result<Report> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("cannot start the async runtime")?;
    runtime.block_on(async {
        let mut connection = connect(&config).await?;
        tracing::info!(pid = connection.pid, "attached");
        let stale_server = connection.stale_server;
        let outcome = {
            let _restore = TerminalGuard::enter()?;
            let mut terminal = ratatui::init();
            attach(&mut terminal, &mut connection).await?
        };
        tracing::info!("client finished: {outcome:?}");
        Ok(Report {
            outcome,
            stale_server,
        })
    })
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        execute!(stdout(), EnableBracketedPaste).context("cannot enable bracketed paste")?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), DisableBracketedPaste);
        ratatui::restore();
        let _ = execute!(stdout(), Show);
    }
}

async fn attach(terminal: &mut DefaultTerminal, connection: &mut Connection) -> Result<Outcome> {
    let mut events = spawn_events();
    let mut parser = vt100::Parser::new(24, 80, 0);
    let mut prefix = PrefixState::default();
    let mut buffer = vec![0; READ_BUFFER_LEN];
    loop {
        if let Some(outcome) = apply_messages(connection, &mut parser)? {
            draw(terminal, parser.screen())?;
            return Ok(outcome);
        }
        draw(terminal, parser.screen())?;
        tokio::select! {
            read = connection.stream.read(&mut buffer) => match read {
                Ok(0) | Err(_) => return Ok(Outcome::LostServer),
                Ok(n) => connection.decoder.feed(&buffer[..n])?,
            },
            event = events.recv() => match event {
                Some(Event::Key(key_event)) => {
                    let Some(key) = key_from_event(&key_event) else { continue };
                    match prefix.handle(key) {
                        Action::Send(key) => connection.send(&ClientMessage::Key(key)).await?,
                        Action::Detach => {
                            let _ = connection.send(&ClientMessage::Detach).await;
                            return Ok(Outcome::Detached);
                        }
                        Action::Discard => {}
                    }
                }
                Some(Event::Paste(text)) => connection.send(&ClientMessage::Paste(text)).await?,
                Some(Event::Resize(cols, rows)) => {
                    terminal.autoresize()?;
                    connection.send(&ClientMessage::Resize { cols, rows }).await?;
                }
                Some(_) => {}
                None => return Ok(Outcome::LostServer),
            },
        }
    }
}

fn apply_messages(
    connection: &mut Connection,
    parser: &mut vt100::Parser,
) -> Result<Option<Outcome>> {
    while let Some(message) = connection.decoder.next_message::<ServerMessage>()? {
        match message {
            ServerMessage::Snapshot {
                cols,
                rows,
                contents,
            } => {
                *parser = vt100::Parser::new(rows, cols, 0);
                parser.process(&contents);
            }
            ServerMessage::Update(contents) => parser.process(&contents),
            ServerMessage::Exited => return Ok(Some(Outcome::Exited)),
            ServerMessage::Info { .. } => tracing::warn!("ignoring a repeated server info"),
        }
    }
    Ok(None)
}

fn spawn_events() -> mpsc::UnboundedReceiver<Event> {
    let (sender, receiver) = mpsc::unbounded_channel();
    thread::spawn(move || {
        while let Ok(event) = event::read() {
            if sender.send(event).is_err() {
                break;
            }
        }
    });
    receiver
}

fn draw(terminal: &mut DefaultTerminal, screen: &vt100::Screen) -> Result<()> {
    terminal.draw(|frame| {
        let area = frame.area();
        frame.render_widget(
            PseudoTerminal::new(screen).cursor(Cursor::default().visibility(false)),
            area,
        );
        let (row, col) = screen.cursor_position();
        if !screen.hide_cursor() && row < area.height && col < area.width {
            frame.set_cursor_position(Position::new(col, row));
        }
    })?;
    Ok(())
}
