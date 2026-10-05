pub mod animation;
pub mod bindings;
mod connect;
pub mod input;
pub mod render;
mod requests;
mod transport;

use std::collections::{BTreeMap, HashMap};
use std::io::stdout;
use std::thread;
use std::time::Instant;

use anyhow::{Context, Result};
use crossterm::cursor::Show;
use crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event as TerminalEvent};
use crossterm::execute;
use gband_core::action::{Action, ClientAction, SessionCommand};
use gband_core::geometry::Size;
use gband_core::input::Key;
use gband_core::layout::{BandId, Layout, PaneId, Program, SessionAction};
use gband_core::view::{CenterFocusedColumn, Scene, View, ViewAction};
use gband_emulator::{Emulator, Grid};
use gband_lua::{Binding, Config, ConfigError, Dispatch, Event, Options, Outcome as Ran, Runtime};
use gband_protocol::{ClientMessage, ExecutableId, ServerMessage, SessionName};
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;

use crate::animation::{
    ANIMATIONS_VARIABLE, Animations, Drawn, FRAME, Presentation, Targets, parse_animations,
};
use crate::bindings::{Command, Keymap, Leader, ROOT};
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

pub struct Configuration {
    pub config: Config,
    pub error: Option<ConfigError>,
    pub reloads: mpsc::UnboundedReceiver<Result<Config, ConfigError>>,
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

pub fn run(
    config: ClientConfig,
    transport: impl Transport,
    configuration: Configuration,
) -> Result<Report> {
    let cwd = std::env::current_dir().context("cannot read the current directory")?;
    let animations = parse_animations(std::env::var(ANIMATIONS_VARIABLE).ok().as_deref());
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
            attach(
                &mut terminal,
                &mut connection,
                animations,
                configuration,
                &config.session,
            )
            .await?
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
    shown: Option<Vec<PaneId>>,
    presentation: Presentation,
    prefix: Option<Key>,
    policy: CenterFocusedColumn,
    banner: Option<String>,
}

impl Display {
    pub fn new(terminal: Size, animations: Animations) -> Self {
        Self {
            layout: Layout::new(),
            area: terminal,
            terminal,
            grids: HashMap::new(),
            view: None,
            shown: None,
            presentation: Presentation::new(animations),
            prefix: None,
            policy: CenterFocusedColumn::default(),
            banner: None,
        }
    }

    pub fn configure(&mut self, options: &Options) {
        self.prefix = Some(options.prefix);
        self.policy = options.center_focused_column;
        if let Some(view) = &mut self.view {
            view.set_center_focused_column(self.policy);
        }
    }

    pub fn banner(&self) -> Option<&str> {
        self.banner.as_deref()
    }

    pub fn set_banner(&mut self, banner: Option<String>) {
        self.banner = banner;
    }

    pub fn report_shown(&mut self) -> Option<ClientMessage> {
        let shown = self.view.as_ref()?.shown(self.scene());
        if self.shown.as_ref() == Some(&shown) {
            return None;
        }
        self.shown = Some(shown.clone());
        Some(ClientMessage::Shown(shown))
    }

    pub fn focused(&self) -> Option<PaneId> {
        self.view.as_ref().and_then(View::focused)
    }

    pub fn observe(&self) -> Option<Observed> {
        let view = self.view.as_ref()?;
        Some(Observed {
            focused: view.focused(),
            band: view.band(),
            panes: self
                .layout
                .bands()
                .iter()
                .flat_map(|band| band.panes().map(move |pane| (pane, band.id)))
                .collect(),
            size: self.terminal,
        })
    }

    pub fn apply(&mut self, message: ServerMessage) -> Option<Outcome> {
        match message {
            ServerMessage::Layout { cols, rows, layout } => {
                self.grids.retain(|&pane, _| layout.contains(pane));
                self.layout = layout;
                let area = Size::new(cols, rows);
                if area != self.area {
                    self.presentation.snap();
                }
                self.area = area;
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
            self.view = Some(View::with_policy(self.scene(), self.policy));
            self.presentation.snap();
        } else {
            self.with_view(View::sync);
        }
    }

    fn view_action(&mut self, action: ViewAction) {
        self.with_view(|view, scene| view.apply(action, scene));
    }

    fn resize(&mut self, terminal: Size) {
        self.terminal = terminal;
        self.presentation.snap();
        self.with_view(View::sync);
    }

    pub fn present(&mut self, now: Instant) -> Option<Drawn> {
        let view = self.view.as_ref()?;
        let targets = Targets::new(&self.layout, self.area, view, self.terminal);
        self.presentation.update(now, &targets);
        Some(self.presentation.drawn(now))
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        self.presentation.is_animating(now)
    }

    fn scene(&self) -> Scene<'_> {
        Scene {
            layout: &self.layout,
            area: self.area,
            viewport: self.terminal,
        }
    }

    fn with_view(&mut self, change: impl FnOnce(&mut View, Scene<'_>)) {
        let scene = Scene {
            layout: &self.layout,
            area: self.area,
            viewport: self.terminal,
        };
        if let Some(view) = &mut self.view {
            change(view, scene);
        }
    }

    fn key_to_focused(&self, key: Key) -> Option<ClientMessage> {
        self.focused().map(|pane| ClientMessage::Key { pane, key })
    }
}

#[derive(Debug, PartialEq)]
pub enum Step {
    Send(ClientMessage),
    Detach,
    Nothing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observed {
    focused: Option<PaneId>,
    band: BandId,
    panes: BTreeMap<PaneId, BandId>,
    size: Size,
}

fn changes(before: Option<Observed>, after: Option<&Observed>) -> Vec<Event> {
    let (Some(before), Some(after)) = (before, after) else {
        return Vec::new();
    };
    let mut events = Vec::new();
    for (&pane, &band) in &before.panes {
        if !after.panes.contains_key(&pane) {
            events.push(Event::PaneClosed { pane, band });
        }
    }
    for (&pane, &band) in &after.panes {
        if !before.panes.contains_key(&pane) {
            events.push(Event::PaneOpened { pane, band });
        }
    }
    if after.band != before.band {
        events.push(Event::BandChanged {
            band: after.band,
            previous: before.band,
        });
    }
    if after.focused != before.focused {
        events.push(Event::FocusChanged {
            pane: after.focused,
            previous: before.focused,
        });
    }
    if after.size != before.size {
        events.push(Event::TerminalResized {
            cols: after.size.cols,
            rows: after.size.rows,
        });
    }
    events
}

const EVENT_DEPTH: usize = 10;

#[derive(Debug, Default, PartialEq)]
pub struct Received {
    pub steps: Vec<Step>,
    pub outcome: Option<Outcome>,
}

pub struct Controls {
    keymap: Keymap,
    runtime: Runtime,
    leader: Leader,
}

impl Controls {
    pub fn new(config: Config, display: &mut Display) -> Self {
        display.configure(&config.options);
        display.set_banner(config.errors.last().map(ToString::to_string));
        Self {
            keymap: Keymap::new(config.options.prefix, config.keymap),
            runtime: config.runtime,
            leader: Leader::default(),
        }
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn active_table(&self) -> &str {
        self.leader.active()
    }

    pub fn attached(&mut self, display: &mut Display, session: &str) -> Vec<Step> {
        self.react(
            display,
            vec![Event::Attached {
                session: session.to_owned(),
            }],
            |_, _, _| {},
        )
    }

    pub fn receive(
        &mut self,
        display: &mut Display,
        messages: impl IntoIterator<Item = ServerMessage>,
    ) -> Received {
        let mut outcome = None;
        let steps = self.react(display, Vec::new(), |_, display, _| {
            for message in messages {
                outcome = display.apply(message);
                if outcome.is_some() {
                    return;
                }
            }
        });
        match outcome {
            Some(outcome) => Received {
                steps: Vec::new(),
                outcome: Some(outcome),
            },
            None => Received {
                steps,
                outcome: None,
            },
        }
    }

    pub fn resize(&mut self, display: &mut Display, terminal: Size) -> Vec<Step> {
        self.react(display, Vec::new(), |_, display, _| {
            display.resize(terminal)
        })
    }

    pub fn press(&mut self, display: &mut Display, key: Key) -> Vec<Step> {
        self.react(
            display,
            Vec::new(),
            |controls, display, steps| match controls.leader.handle(&controls.keymap, key) {
                Command::Send(key) => steps.push(dispatch(
                    display,
                    Action::Client(ClientAction::SendKey(key)),
                )),
                Command::Run(Binding::Action(action)) => steps.push(dispatch(display, action)),
                Command::Run(Binding::Callback(callback)) => {
                    let outcome = controls.runtime.call(callback);
                    controls.apply(display, outcome, steps);
                }
                Command::Discard => {}
            },
        )
    }

    pub fn reload(
        &mut self,
        display: &mut Display,
        result: Result<Config, ConfigError>,
    ) -> Vec<Step> {
        match result {
            Ok(config) => {
                tracing::info!("configuration reloaded");
                for error in &config.errors {
                    tracing::warn!("configuration error: {error}");
                }
                let previous = self.leader.active().to_owned();
                *self = Self::new(config, display);
                let mut events = vec![Event::ConfigReloaded];
                if previous != ROOT {
                    events.push(Event::KeyTableChanged {
                        table: ROOT.to_owned(),
                        previous,
                    });
                }
                self.react(display, events, |_, _, _| {})
            }
            Err(error) => {
                tracing::warn!("configuration error: {error}");
                display.set_banner(Some(error.to_string()));
                self.react(display, Vec::new(), |controls, _, _| {
                    controls.leader.reset()
                })
            }
        }
    }

    fn react(
        &mut self,
        display: &mut Display,
        mut events: Vec<Event>,
        change: impl FnOnce(&mut Self, &mut Display, &mut Vec<Step>),
    ) -> Vec<Step> {
        let mut steps = Vec::new();
        let mut before = display.observe();
        let mut table = self.leader.active().to_owned();
        change(self, display, &mut steps);
        for depth in 0.. {
            self.table_changed(&mut table, &mut events);
            let after = display.observe();
            events.extend(changes(before, after.as_ref()));
            before = after;
            if events.is_empty() {
                break;
            }
            if depth == EVENT_DEPTH {
                tracing::warn!(
                    "dropping {} events caused by event handlers {EVENT_DEPTH} times in a row",
                    events.len()
                );
                break;
            }
            for event in std::mem::take(&mut events) {
                let outcome = self.runtime.emit(&event);
                self.apply(display, outcome, &mut steps);
            }
        }
        steps
    }

    fn table_changed(&mut self, table: &mut String, events: &mut Vec<Event>) {
        let active = self.leader.active();
        if active != table {
            self.runtime.set_active_table(active);
            events.push(Event::KeyTableChanged {
                table: active.to_owned(),
                previous: std::mem::replace(table, active.to_owned()),
            });
        }
    }

    fn apply(&mut self, display: &mut Display, outcome: Ran, steps: &mut Vec<Step>) {
        for entry in outcome.dispatched {
            match entry {
                Dispatch::Action(action) => steps.push(dispatch(display, action)),
                Dispatch::Spawn(program) => steps.push(spawn(display, program)),
                Dispatch::Enter(table) => self.leader.enter(table),
            }
        }
        for error in &outcome.errors {
            tracing::warn!("configuration error: {error}");
        }
        if let Some(error) = outcome.errors.last() {
            display.set_banner(Some(error.to_string()));
        }
    }
}

fn spawn(display: &mut Display, program: Option<Program>) -> Step {
    let open = display
        .view
        .as_ref()
        .and_then(|view| view.resolve(SessionCommand::OpenPane));
    match open {
        Some(SessionAction::OpenPane { band, after, .. }) => {
            Step::Send(ClientMessage::Action(SessionAction::OpenPane {
                band,
                after,
                program,
            }))
        }
        _ => Step::Nothing,
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
        Action::Client(ClientAction::SendPrefix) => display
            .prefix
            .and_then(|prefix| display.key_to_focused(prefix)),
        Action::Client(ClientAction::SendKey(key)) => display.key_to_focused(key),
    };
    message.map_or(Step::Nothing, Step::Send)
}

async fn perform(connection: &mut Connection, steps: Vec<Step>) -> Result<Option<Outcome>> {
    for step in steps {
        match step {
            Step::Send(message) => connection.send(&message).await?,
            Step::Detach => {
                let _ = connection.send(&ClientMessage::Detach).await;
                return Ok(Some(Outcome::Detached));
            }
            Step::Nothing => {}
        }
    }
    Ok(None)
}

async fn attach(
    terminal: &mut DefaultTerminal,
    connection: &mut Connection,
    animations: Animations,
    configuration: Configuration,
    session: &SessionName,
) -> Result<Outcome> {
    let mut events = spawn_events();
    let size = terminal.size()?;
    let mut display = Display::new(Size::new(size.width, size.height), animations);
    let mut controls = Controls::new(configuration.config, &mut display);
    if let Some(error) = configuration.error {
        display.set_banner(Some(error.to_string()));
    }
    let mut reloads = configuration.reloads;
    let steps = controls.attached(&mut display, &session.to_string());
    if let Some(outcome) = perform(connection, steps).await? {
        return Ok(outcome);
    }
    loop {
        let mut messages = Vec::new();
        while let Some(message) = connection.reader.try_recv::<ServerMessage>()? {
            messages.push(message);
        }
        if !messages.is_empty() {
            let received = controls.receive(&mut display, messages);
            if let Some(outcome) = received.outcome {
                draw(terminal, &mut display, Instant::now())?;
                return Ok(outcome);
            }
            if let Some(outcome) = perform(connection, received.steps).await? {
                return Ok(outcome);
            }
        }
        if let Some(message) = display.report_shown() {
            let _ = connection.send(&message).await;
        }
        let now = Instant::now();
        draw(terminal, &mut display, now)?;
        let frame = display
            .is_animating(now)
            .then(|| tokio::time::Instant::from_std(now + FRAME));
        tokio::select! {
            () = tokio::time::sleep_until(frame.unwrap_or_else(tokio::time::Instant::now)),
                if frame.is_some() => {}
            Some(result) = reloads.recv() => {
                let steps = controls.reload(&mut display, result);
                if let Some(outcome) = perform(connection, steps).await? {
                    return Ok(outcome);
                }
            }
            filled = connection.reader.fill() => match filled {
                Ok(true) => {}
                Ok(false) | Err(gband_protocol::IoError::Io(_)) => return Ok(Outcome::LostServer),
                Err(error) => return Err(error.into()),
            },
            event = events.recv() => match event {
                Some(TerminalEvent::Key(key_event)) => {
                    let Some(key) = key_from_event(&key_event) else { continue };
                    let steps = controls.press(&mut display, key);
                    if let Some(outcome) = perform(connection, steps).await? {
                        return Ok(outcome);
                    }
                }
                Some(TerminalEvent::Paste(text)) => {
                    if let Some(pane) = display.focused() {
                        connection.send(&ClientMessage::Paste { pane, text }).await?;
                    }
                }
                Some(TerminalEvent::Resize(cols, rows)) => {
                    terminal.autoresize()?;
                    let steps = controls.resize(&mut display, Size::new(cols, rows));
                    connection.send(&ClientMessage::Resize { cols, rows }).await?;
                    if let Some(outcome) = perform(connection, steps).await? {
                        return Ok(outcome);
                    }
                }
                Some(_) => {}
                None => return Ok(Outcome::LostServer),
            },
        }
    }
}

fn spawn_events() -> mpsc::UnboundedReceiver<TerminalEvent> {
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

fn draw(terminal: &mut DefaultTerminal, display: &mut Display, now: Instant) -> Result<()> {
    let drawn = display.present(now);
    terminal.draw(|frame| {
        let (Some(view), Some(drawn)) = (&display.view, &drawn) else {
            return;
        };
        let ribbon = Ribbon {
            layout: &display.layout,
            area: display.area,
            view,
            grids: &display.grids,
            drawn,
            banner: display.banner.as_deref(),
        };
        draw_frame(frame, &ribbon);
    })?;
    Ok(())
}
