pub mod bindings;
mod connect;
pub mod input;
pub mod render;
mod requests;
mod transport;

use std::collections::HashMap;
use std::io::stdout;
use std::thread;

use anyhow::{Context, Result};
use crossterm::cursor::Show;
use crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event};
use crossterm::execute;
use gband_core::action::{Action, ClientAction};
use gband_core::geometry::Size;
use gband_core::input::Key;
use gband_core::layout::{Layout, PaneId};
use gband_core::view::{Scene, View, ViewAction};
use gband_emulator::{Emulator, Grid};
use gband_protocol::{ClientMessage, ExecutableId, ServerMessage, SessionName};
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;

use crate::bindings::{Command, Leader};
pub use crate::connect::{Connection, connect};
use crate::input::key_from_event;
use crate::render::{Ribbon, draw_frame};
pub use crate::requests::{kill_session, list_sessions};
pub use crate::transport::{Link, Transport, UnixTransport};

pub struct ClientConfig {
    pub session: SessionName,
    pub identity: ExecutableId,
    pub replace_mismatched: bool,
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

fn runtime() -> Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("cannot start the async runtime")
}

pub fn run(config: ClientConfig, transport: impl Transport) -> Result<Report> {
    let cwd = std::env::current_dir().context("cannot read the current directory")?;
    runtime()?.block_on(async {
        let mut connection = connect(&config, &transport).await?;
        connection
            .send(&ClientMessage::Attach {
                session: config.session.clone(),
                cwd,
            })
            .await?;
        tracing::info!(pid = connection.pid, session = %config.session, "attached");
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

pub struct Display {
    layout: Layout,
    area: Size,
    terminal: Size,
    grids: HashMap<PaneId, Grid>,
    view: Option<View>,
}

impl Display {
    pub fn new(terminal: Size) -> Self {
        Self {
            layout: Layout::new(),
            area: terminal,
            terminal,
            grids: HashMap::new(),
            view: None,
        }
    }

    pub fn focused(&self) -> Option<PaneId> {
        self.view.as_ref().and_then(View::focused)
    }

    pub fn apply(&mut self, message: ServerMessage) -> Option<Outcome> {
        match message {
            ServerMessage::Layout { cols, rows, layout } => {
                self.grids.retain(|&pane, _| layout.contains(pane));
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
                let mut grid = Grid::new(Size::new(cols, rows));
                grid.process(&contents);
                self.grids.insert(pane, grid);
            }
            ServerMessage::Update { pane, contents } => match self.grids.get_mut(&pane) {
                Some(grid) => grid.process(&contents),
                None => tracing::warn!(pane = %pane, "ignoring an update for an unknown pane"),
            },
            ServerMessage::Focus(pane) => {
                self.with_view(|view, scene| view.focus_pane(pane, scene))
            }
            ServerMessage::Exited => return Some(Outcome::Exited),
            ServerMessage::Info { .. } => tracing::warn!("ignoring a repeated server info"),
            ServerMessage::Sessions(_) | ServerMessage::Killed | ServerMessage::NoSuchSession => {
                tracing::warn!("ignoring an answer to a request this client did not send");
            }
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

    fn key_to_focused(&self, key: Key) -> Option<ClientMessage> {
        self.focused().map(|pane| ClientMessage::Key { pane, key })
    }
}

pub enum Step {
    Send(ClientMessage),
    Detach,
    Nothing,
}

fn press(display: &mut Display, leader: &mut Leader, key: Key) -> Step {
    match leader.handle(key) {
        Command::Send(key) => dispatch(display, Action::Client(ClientAction::SendKey(key))),
        Command::Run(action) => dispatch(display, action),
        Command::Discard => Step::Nothing,
    }
}

pub fn dispatch(display: &mut Display, action: Action) -> Step {
    let message = match action {
        Action::View(action) => {
            display.view_action(action);
            None
        }
        Action::Session(command) => display
            .view
            .as_ref()
            .and_then(|view| view.resolve(command))
            .map(ClientMessage::Action),
        Action::Client(ClientAction::Detach) => return Step::Detach,
        Action::Client(ClientAction::SendKey(key)) => display.key_to_focused(key),
    };
    message.map_or(Step::Nothing, Step::Send)
}

async fn attach(terminal: &mut DefaultTerminal, connection: &mut Connection) -> Result<Outcome> {
    let mut events = spawn_events();
    let size = terminal.size()?;
    let mut display = Display::new(Size::new(size.width, size.height));
    let mut leader = Leader::default();
    loop {
        if let Some(outcome) = apply_messages(connection, &mut display)? {
            draw(terminal, &display)?;
            return Ok(outcome);
        }
        draw(terminal, &display)?;
        tokio::select! {
            filled = connection.reader.fill() => match filled {
                Ok(true) => {}
                Ok(false) | Err(gband_protocol::IoError::Io(_)) => return Ok(Outcome::LostServer),
                Err(error) => return Err(error.into()),
            },
            event = events.recv() => match event {
                Some(Event::Key(key_event)) => {
                    let Some(key) = key_from_event(&key_event) else { continue };
                    match press(&mut display, &mut leader, key) {
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
    while let Some(message) = connection.reader.try_recv::<ServerMessage>()? {
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
            grids: &display.grids,
        };
        draw_frame(frame, &ribbon);
    })?;
    Ok(())
}
