pub mod bindings;
mod connect;
pub mod input;
pub mod render;

use std::collections::HashMap;
use std::io::stdout;
use std::path::PathBuf;
use std::thread;

use anyhow::{Context, Result};
use crossterm::cursor::Show;
use crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event};
use crossterm::execute;
use gband_core::geometry::Size;
use gband_core::input::Key;
use gband_core::layout::{Layout, PaneId, SessionAction};
use gband_core::view::{Scene, View, ViewAction};
use gband_protocol::{ClientMessage, ExecutableId, ServerMessage};
use ratatui::DefaultTerminal;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

use crate::bindings::{Binding, Command, Leader, PREFIX, SessionCommand};
pub use crate::connect::{Connection, connect};
use crate::input::key_from_event;
use crate::render::{Ribbon, render};

const READ_BUFFER_LEN: usize = 64 * 1024;

pub struct ClientConfig {
    pub socket: PathBuf,
    pub executable_path: PathBuf,
    pub identity: ExecutableId,
    pub replace_mismatched: bool,
    pub log_dir: PathBuf,
    pub kill_command: String,
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

struct Display {
    layout: Layout,
    area: Size,
    terminal: Size,
    parsers: HashMap<PaneId, vt100::Parser>,
    view: Option<View>,
}

impl Display {
    fn new(terminal: Size) -> Self {
        Self {
            layout: Layout::new(),
            area: terminal,
            terminal,
            parsers: HashMap::new(),
            view: None,
        }
    }

    fn focused(&self) -> Option<PaneId> {
        self.view.as_ref().and_then(View::focused)
    }

    fn apply(&mut self, message: ServerMessage) -> Option<Outcome> {
        match message {
            ServerMessage::Layout { cols, rows, layout } => {
                self.parsers.retain(|&pane, _| layout.contains(pane));
                self.layout = layout;
                self.area = Size::new(cols, rows);
                self.sync();
            }
            ServerMessage::Snapshot {
                pane,
                cols,
                rows,
                contents,
            } => {
                let mut parser = vt100::Parser::new(rows, cols, 0);
                parser.process(&contents);
                self.parsers.insert(pane, parser);
            }
            ServerMessage::Update { pane, contents } => match self.parsers.get_mut(&pane) {
                Some(parser) => parser.process(&contents),
                None => tracing::warn!(pane = %pane, "ignoring an update for an unknown pane"),
            },
            ServerMessage::Focus(pane) => {
                self.with_view(|view, scene| view.focus_pane(pane, scene))
            }
            ServerMessage::Exited => return Some(Outcome::Exited),
            ServerMessage::Info { .. } => tracing::warn!("ignoring a repeated server info"),
        }
        None
    }

    fn sync(&mut self) {
        if self.view.is_none() {
            self.view = Some(View::new(self.scene()));
        } else {
            self.with_view(View::sync);
        }
    }

    fn view_action(&mut self, action: ViewAction) {
        self.with_view(|view, scene| view.apply(action, scene));
    }

    fn resize(&mut self, terminal: Size) {
        self.terminal = terminal;
        self.with_view(View::sync);
    }

    fn scene(&self) -> Scene<'_> {
        Scene {
            layout: &self.layout,
            area: self.area,
            viewport_cols: self.terminal.cols,
        }
    }

    fn with_view(&mut self, change: impl FnOnce(&mut View, Scene<'_>)) {
        let scene = Scene {
            layout: &self.layout,
            area: self.area,
            viewport_cols: self.terminal.cols,
        };
        if let Some(view) = &mut self.view {
            change(view, scene);
        }
    }

    fn resolve(&self, command: SessionCommand) -> Option<SessionAction> {
        let view = self.view.as_ref()?;
        let focused = view.focused();
        Some(match command {
            SessionCommand::OpenPane => SessionAction::OpenPane {
                workspace: view.workspace(),
                after: focused,
            },
            SessionCommand::ClosePane => SessionAction::ClosePane(focused?),
            SessionCommand::ConsumeOrExpel(direction) => SessionAction::ConsumeOrExpel {
                pane: focused?,
                direction,
            },
            SessionCommand::CycleWidth => SessionAction::CycleWidth(focused?),
            SessionCommand::ToggleFullWidth => SessionAction::ToggleFullWidth(focused?),
        })
    }

    fn key_to_focused(&self, key: Key) -> Option<ClientMessage> {
        self.focused().map(|pane| ClientMessage::Key { pane, key })
    }
}

enum Step {
    Send(ClientMessage),
    Detach,
    Nothing,
}

fn dispatch(display: &mut Display, leader: &mut Leader, key: Key) -> Step {
    let message = match leader.handle(key) {
        Command::Send(key) => display.key_to_focused(key),
        Command::Discard => None,
        Command::Run(Binding::Detach) => return Step::Detach,
        Command::Run(Binding::SendPrefix) => display.key_to_focused(PREFIX),
        Command::Run(Binding::View(action)) => {
            display.view_action(action);
            None
        }
        Command::Run(Binding::Session(command)) => {
            display.resolve(command).map(ClientMessage::Action)
        }
    };
    message.map_or(Step::Nothing, Step::Send)
}

async fn attach(terminal: &mut DefaultTerminal, connection: &mut Connection) -> Result<Outcome> {
    let mut events = spawn_events();
    let size = terminal.size()?;
    let mut display = Display::new(Size::new(size.width, size.height));
    let mut leader = Leader::default();
    let mut buffer = vec![0; READ_BUFFER_LEN];
    loop {
        if let Some(outcome) = apply_messages(connection, &mut display)? {
            draw(terminal, &display)?;
            return Ok(outcome);
        }
        draw(terminal, &display)?;
        tokio::select! {
            read = connection.stream.read(&mut buffer) => match read {
                Ok(0) | Err(_) => return Ok(Outcome::LostServer),
                Ok(n) => connection.decoder.feed(&buffer[..n])?,
            },
            event = events.recv() => match event {
                Some(Event::Key(key_event)) => {
                    let Some(key) = key_from_event(&key_event) else { continue };
                    match dispatch(&mut display, &mut leader, key) {
                        Step::Send(message) => connection.send(&message).await?,
                        Step::Detach => {
                            let _ = connection.send(&ClientMessage::Detach).await;
                            return Ok(Outcome::Detached);
                        }
                        Step::Nothing => {}
                    }
                }
                Some(Event::Paste(text)) => {
                    if let Some(pane) = display.focused() {
                        connection.send(&ClientMessage::Paste { pane, text }).await?;
                    }
                }
                Some(Event::Resize(cols, rows)) => {
                    terminal.autoresize()?;
                    display.resize(Size::new(cols, rows));
                    connection.send(&ClientMessage::Resize { cols, rows }).await?;
                }
                Some(_) => {}
                None => return Ok(Outcome::LostServer),
            },
        }
    }
}

fn apply_messages(connection: &mut Connection, display: &mut Display) -> Result<Option<Outcome>> {
    while let Some(message) = connection.decoder.next_message::<ServerMessage>()? {
        if let Some(outcome) = display.apply(message) {
            return Ok(Some(outcome));
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

fn draw(terminal: &mut DefaultTerminal, display: &Display) -> Result<()> {
    terminal.draw(|frame| {
        let Some(view) = &display.view else {
            return;
        };
        let ribbon = Ribbon {
            layout: &display.layout,
            area: display.area,
            view,
            parsers: &display.parsers,
        };
        if let Some(cursor) = render(&ribbon, frame.buffer_mut()) {
            frame.set_cursor_position(cursor);
        }
    })?;
    Ok(())
}
