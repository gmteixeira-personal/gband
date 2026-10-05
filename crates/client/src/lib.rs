pub mod animation;
pub mod bindings;
mod channel;
pub mod color;
mod connect;
pub mod input;
pub mod placement;
pub mod render;
mod requests;
mod transport;
pub mod windows;

use std::collections::{BTreeMap, HashMap};
use std::io::{Write, stdout};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use anyhow::{Context, Result};
use crossterm::cursor::Show;
use crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event as TerminalEvent};
use crossterm::execute;
use gband_core::action::{Action, ClientAction, SessionCommand};
use gband_core::geometry::Size;
use gband_core::input::Key;
use gband_core::layout::{BandId, Layout, PaneId, Program, Proportion, SessionAction};
use gband_core::view::{CenterFocusedColumn, Scene, View, ViewAction};
use gband_emulator::{Emulator, Grid};
use gband_lua::{
    BandState, Binding, ColumnState, Config, ConfigError, Dispatch, Event, Options, Outcome as Ran,
    PaneInput, PaneStates, PluginManifest, Requirement as Needed, Runtime, StatusLine, Version,
    ViewState, WindowRequest,
};
use gband_protocol::{ClientMessage, ExecutableId, Requirement, ServerMessage, SessionName, Value};
use ratatui::layout::Rect;
use ratatui::{DefaultTerminal, Frame};
use tokio::sync::mpsc;

use crate::animation::{
    ANIMATIONS_VARIABLE, Animations, Drawn, FRAME, Presentation, Targets, parse_animations,
};
use crate::bindings::{Command, Keymap, Leader, ROOT};
use crate::channel::{Channel, Served};
pub use crate::channel::{Loader, TestChannel};
use crate::color::ColorSupport;
pub use crate::connect::{Connection, connect, connect_reporting};
use crate::input::key_from_event;
use crate::placement::Placement;
use crate::render::{Ribbon, StatusArea, draw_frame};
pub use crate::requests::{kill_session, list_sessions};
pub use crate::transport::{Link, Transport, UnixTransport};
use crate::windows::{OpenRequest, Opened, Windows};

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
    pub channel: Option<TestChannel>,
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
    let placement = Placement::new(&configuration.config.options.statusline);
    runtime()?.block_on(async {
        let mut connection = connect_reporting(&config, &transport, placement).await?;
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
    layout: Arc<Layout>,
    area: Size,
    terminal: Size,
    placement: Placement,
    ribbon: Rect,
    status: Option<Rect>,
    grids: HashMap<PaneId, Grid>,
    view: Option<View>,
    shown: Option<Vec<PaneId>>,
    presentation: Presentation,
    prefix: Option<Key>,
    policy: CenterFocusedColumn,
    banner: Option<String>,
    colors: ColorSupport,
    line: Option<StatusLine>,
    windows: Windows,
    states: Arc<PaneStates>,
    ready: bool,
}

impl Display {
    pub fn new(terminal: Size, animations: Animations) -> Self {
        let (ribbon, status) = Placement::OFF.split(terminal);
        Self {
            layout: Arc::new(Layout::new()),
            area: terminal,
            terminal,
            placement: Placement::OFF,
            ribbon,
            status,
            grids: HashMap::new(),
            view: None,
            shown: None,
            presentation: Presentation::new(animations),
            prefix: None,
            policy: CenterFocusedColumn::default(),
            banner: None,
            colors: ColorSupport::default(),
            line: None,
            windows: Windows::new(),
            states: Arc::new(PaneStates::new()),
            ready: false,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn pane_state(&self, pane: PaneId) -> Option<&BTreeMap<String, Value>> {
        self.states.get(&pane)
    }

    fn set_state(&mut self, pane: PaneId, key: String, value: Option<Value>) -> Option<Value> {
        let states = Arc::make_mut(&mut self.states);
        let state = states.entry(pane).or_default();
        let previous = match value {
            Some(value) => state.insert(key, value),
            None => state.remove(&key),
        };
        if state.is_empty() {
            states.remove(&pane);
        }
        previous
    }

    pub fn windows(&self) -> &Windows {
        &self.windows
    }

    pub fn focused_window(&self) -> Option<u32> {
        self.windows
            .focused_float()
            .or_else(|| self.focused().and_then(|pane| self.windows.window_of(pane)))
    }

    pub fn grid_size(&self, pane: PaneId) -> Option<Size> {
        self.grids.get(&pane).map(Grid::size)
    }

    pub fn configure(&mut self, options: &Options) {
        self.prefix = Some(options.prefix);
        self.policy = options.center_focused_column;
        if let Some(view) = &mut self.view {
            view.set_center_focused_column(self.policy);
        }
        self.set_placement(Placement::new(&options.statusline));
    }

    pub fn set_colors(&mut self, colors: ColorSupport) {
        self.colors = colors;
    }

    pub fn colors(&self) -> ColorSupport {
        self.colors
    }

    pub fn reported_size(&self) -> Size {
        Size::new(self.ribbon.width, self.ribbon.height)
    }

    pub fn ribbon_area(&self) -> Rect {
        self.ribbon
    }

    pub fn status_area(&self) -> Option<Rect> {
        self.status
    }

    pub fn status_line(&self) -> Option<&StatusLine> {
        self.line.as_ref()
    }

    pub fn set_status_line(&mut self, line: StatusLine) {
        self.line = Some(line);
    }

    pub fn set_size(&mut self, terminal: Size, placement: Placement) -> Option<Size> {
        let before = self.reported_size();
        self.terminal = terminal;
        self.placement = placement;
        (self.ribbon, self.status) = placement.split(terminal);
        let after = self.reported_size();
        if after == before {
            return None;
        }
        self.presentation.snap();
        self.with_view(View::sync);
        Some(after)
    }

    pub fn set_placement(&mut self, placement: Placement) -> Option<Size> {
        self.set_size(self.terminal, placement)
    }

    pub fn view_state(&self, table: &str) -> ViewState {
        let bands = self.layout.bands();
        let viewed = match &self.view {
            Some(view) => Some(view.band()),
            None => bands.first().map(|band| band.id),
        };
        let index = viewed.and_then(|id| bands.iter().position(|band| band.id == id));
        let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        let column = index.and_then(|index| {
            let band = &bands[index];
            let (column, _) = band.locate(self.focused()?)?;
            Some(ColumnState {
                index: count(column + 1),
                count: count(band.columns.len()),
            })
        });
        ViewState {
            table: table.to_owned(),
            band: BandState {
                number: viewed.map_or(0, |id| id.0),
                index: index.map_or(0, |index| count(index + 1)),
                count: count(bands.len()),
            },
            column,
            pane: self.focused().map(|pane| pane.0),
            width: self.terminal.cols,
            drawn: self.status.is_some(),
            error: self.banner.clone(),
            layout: Arc::clone(&self.layout),
            area: self.area,
            ribbon: self.reported_size(),
            states: Arc::clone(&self.states),
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
            shape: self
                .layout
                .bands()
                .iter()
                .map(|band| {
                    let columns = band
                        .columns
                        .iter()
                        .map(|column| (column.panes.clone(), column.width, column.full_width))
                        .collect();
                    (band.id, columns)
                })
                .collect(),
        })
    }

    pub fn apply(&mut self, message: ServerMessage) -> Option<Outcome> {
        match message {
            ServerMessage::Layout { cols, rows, layout } => {
                self.grids.retain(|&pane, _| layout.contains(pane));
                let previous = Arc::clone(&self.layout);
                if self
                    .states
                    .keys()
                    .any(|&pane| previous.contains(pane) && !layout.contains(pane))
                {
                    Arc::make_mut(&mut self.states)
                        .retain(|&pane, _| layout.contains(pane) || !previous.contains(pane));
                }
                self.layout = Arc::new(layout);
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
            ServerMessage::Opened { .. }
            | ServerMessage::Event { .. }
            | ServerMessage::PaneState { .. }
            | ServerMessage::Result { .. }
            | ServerMessage::Requirements(_)
            | ServerMessage::ServerError(_) => {
                tracing::warn!("ignoring a bridge message outside the controls")
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

    pub fn present(&mut self, now: Instant) -> Option<Drawn> {
        let view = self.view.as_ref()?;
        let targets = Targets::new(&self.layout, self.area, view, self.reported_size());
        self.presentation.update(now, &targets);
        Some(self.presentation.drawn(now))
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>, now: Instant) {
        let drawn = self.present(now);
        let (Some(view), Some(drawn)) = (&self.view, &drawn) else {
            return;
        };
        let ribbon = Ribbon {
            layout: &self.layout,
            area: self.area,
            view,
            grids: &self.grids,
            drawn,
            region: self.ribbon,
            floats: self.windows.floats(),
            float_focused: self.windows.focused_float().is_some(),
            colors: self.colors,
            banner: self.banner.as_deref(),
            status: self.status.map(|area| StatusArea {
                area,
                line: self.line.as_ref(),
                colors: self.colors,
            }),
        };
        draw_frame(frame, &ribbon);
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        self.presentation.is_animating(now)
    }

    fn scene(&self) -> Scene<'_> {
        Scene {
            layout: &self.layout,
            area: self.area,
            viewport: self.reported_size(),
        }
    }

    fn with_view(&mut self, change: impl FnOnce(&mut View, Scene<'_>)) {
        let scene = Scene {
            layout: &self.layout,
            area: self.area,
            viewport: Size::new(self.ribbon.width, self.ribbon.height),
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
    Write(Vec<u8>),
    Detach,
    Nothing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observed {
    focused: Option<PaneId>,
    band: BandId,
    panes: BTreeMap<PaneId, BandId>,
    size: Size,
    shape: Vec<(BandId, Vec<ColumnShape>)>,
}

type ColumnShape = (Vec<PaneId>, Proportion, bool);

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
    if after.shape != before.shape {
        events.push(Event::LayoutChanged);
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
    refresh_pending: bool,
    plugins: Vec<PluginManifest>,
    pending: Vec<Event>,
    awaiting: Option<String>,
}

fn unmet(plugins: &[PluginManifest], required: &Requirement) -> Option<ConfigError> {
    let Requirement {
        plugin,
        requirement,
    } = required;
    let needed: Needed = match requirement.parse() {
        Ok(needed) => needed,
        Err(reason) => {
            return Some(ConfigError::new(format!(
                "the server requires the plugin `{plugin}` {requirement} in the client, which is not a requirement: {reason}"
            )));
        }
    };
    let held = plugins
        .iter()
        .find(|manifest| manifest.name == *plugin)
        .and_then(|manifest| manifest.version.clone());
    match held {
        None => Some(ConfigError::new(format!(
            "the server requires the plugin `{plugin}` {requirement} in the client, which does not have it"
        ))),
        Some(version) => {
            let met = version
                .parse::<Version>()
                .is_ok_and(|parsed| needed.is_met_by(parsed));
            (!met).then(|| {
                ConfigError::new(format!(
                    "the server requires the plugin `{plugin}` {requirement} in the client, which has version {version}"
                ))
            })
        }
    }
}

impl Controls {
    pub fn new(config: Config, display: &mut Display) -> Self {
        config.runtime.set_window_counter(display.windows.counter());
        display.configure(&config.options);
        display.set_banner(config.errors.last().map(ToString::to_string));
        Self {
            keymap: Keymap::new(config.options.prefix, config.keymap),
            runtime: config.runtime,
            leader: Leader::default(),
            refresh_pending: false,
            plugins: config.plugins,
            pending: Vec::new(),
            awaiting: None,
        }
    }

    pub fn attach_when_ready(&mut self, session: &str) {
        self.awaiting = Some(session.to_owned());
    }

    pub fn is_attached(&self) -> bool {
        self.awaiting.is_none()
    }

    fn report(&mut self, display: &mut Display, error: String) {
        tracing::warn!("configuration error: {error}");
        display.set_banner(Some(error));
    }

    fn bridge(&mut self, display: &mut Display, message: ServerMessage, steps: &mut Vec<Step>) {
        match message {
            ServerMessage::PaneState { pane, key, value } => {
                let previous = display.set_state(pane, key.clone(), value.clone());
                if display.ready && previous != value {
                    self.pending.push(Event::PaneStateChanged {
                        pane,
                        key,
                        value,
                        previous,
                    });
                }
            }
            ServerMessage::Event {
                name,
                data,
                queued,
                time,
            } => self.pending.push(Event::ServerEvent {
                name,
                data,
                queued,
                time,
            }),
            ServerMessage::Result { call, result } => {
                let outcome = self.runtime.answer(call, result);
                self.apply(display, outcome, steps);
            }
            ServerMessage::Requirements(required) => {
                display.ready = true;
                let errors: Vec<ConfigError> = required
                    .iter()
                    .filter_map(|required| unmet(&self.plugins, required))
                    .collect();
                for error in errors {
                    self.report(display, error.to_string());
                }
                if let Some(session) = self.awaiting.take() {
                    self.refresh_pending = true;
                    self.pending.push(Event::Attached { session });
                }
            }
            ServerMessage::ServerError(error) => self.report(display, format!("server: {error}")),
            _ => {}
        }
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn active_table(&self) -> &str {
        self.leader.active()
    }

    pub fn attached(&mut self, display: &mut Display, session: &str) -> Vec<Step> {
        self.refresh_pending = true;
        self.react(
            display,
            vec![Event::Attached {
                session: session.to_owned(),
            }],
            |_, _, _| {},
        )
    }

    pub fn refresh(&mut self, display: &mut Display) -> Vec<Step> {
        self.react(display, Vec::new(), |controls, display, steps| {
            controls.refresh_now(display, steps);
        })
    }

    pub fn next_timer(&self) -> Option<Instant> {
        self.runtime.next_timer()
    }

    pub fn fire_timers(&mut self, display: &mut Display, now: Instant) -> Vec<Step> {
        self.react(display, Vec::new(), |controls, display, steps| {
            controls.push_state(display, steps);
            let outcome = controls.runtime.fire_timers(now);
            controls.apply(display, outcome, steps);
        })
    }

    pub fn receive(
        &mut self,
        display: &mut Display,
        messages: impl IntoIterator<Item = ServerMessage>,
    ) -> Received {
        let mut outcome = None;
        let steps = self.react(display, Vec::new(), |controls, display, steps| {
            for message in messages {
                if let ServerMessage::Opened { request, pane } = message {
                    controls.opened(display, request, pane, steps);
                    continue;
                }
                if matches!(
                    message,
                    ServerMessage::PaneState { .. }
                        | ServerMessage::Event { .. }
                        | ServerMessage::Result { .. }
                        | ServerMessage::Requirements(_)
                        | ServerMessage::ServerError(_)
                ) {
                    controls.bridge(display, message, steps);
                    continue;
                }
                outcome = display.apply(message);
                if outcome.is_some() {
                    return;
                }
            }
            controls.track_panes(display, steps);
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
        self.react(display, Vec::new(), |_, display, steps| {
            let placement = display.placement;
            if let Some(size) = display.set_size(terminal, placement) {
                steps.push(resize_step(size));
            }
        })
    }

    pub fn press(&mut self, display: &mut Display, key: Key) -> Vec<Step> {
        self.react(display, Vec::new(), |controls, display, steps| {
            let released = controls.runtime.release_windows();
            controls.apply(display, released, steps);
            match controls.leader.handle(&controls.keymap, key) {
                Command::Send(key) => match display.focused_window() {
                    Some(window) => {
                        let outcome = controls.runtime.window_key(window, key);
                        controls.apply(display, outcome, steps);
                    }
                    None => steps.push(dispatch(
                        display,
                        Action::Client(ClientAction::SendKey(key)),
                    )),
                },
                Command::Run(Binding::Action(action)) => steps.push(dispatch(display, action)),
                Command::Run(Binding::Callback(callback)) => {
                    let outcome = controls.runtime.call(callback);
                    controls.apply(display, outcome, steps);
                }
                Command::Discard => {}
            }
        })
    }

    pub fn eval(
        &mut self,
        display: &mut Display,
        source: &str,
        args: &[Value],
    ) -> (Result<Vec<Value>, String>, Vec<Step>) {
        let mut answer = Err("the chunk did not run".to_owned());
        let steps = self.react(display, Vec::new(), |controls, display, steps| {
            let (result, outcome) = controls.runtime.eval(source, args);
            answer = result;
            controls.apply(display, outcome, steps);
        });
        (answer, steps)
    }

    pub fn paste(&mut self, display: &mut Display, text: String) -> Vec<Step> {
        if display.focused_window().is_some() {
            return Vec::new();
        }
        display
            .focused()
            .map(|pane| Step::Send(ClientMessage::Paste { pane, text }))
            .into_iter()
            .collect()
    }

    fn opened(
        &mut self,
        display: &mut Display,
        request: u32,
        pane: Option<PaneId>,
        steps: &mut Vec<Step>,
    ) {
        match display.windows.opened(request, pane) {
            Opened::Close(pane) => {
                if let Some(pane) = pane {
                    steps.push(Step::Send(ClientMessage::Action(SessionAction::ClosePane(
                        pane,
                    ))));
                }
            }
            Opened::Window(window) => {
                let outcome = self.runtime.window_opened(window, pane);
                self.apply(display, outcome, steps);
            }
        }
    }

    fn track_panes(&mut self, display: &mut Display, steps: &mut Vec<Step>) {
        for (pane, window) in display.windows.panes() {
            if !display.layout.contains(pane) {
                display.windows.forget(pane);
                let outcome = self.runtime.pane_closed(window);
                self.apply(display, outcome, steps);
                continue;
            }
            let Some(size) = display.grid_size(pane) else {
                continue;
            };
            if display.windows.resized(pane, size) {
                let outcome = self.runtime.pane_resized(window, size);
                self.apply(display, outcome, steps);
            }
        }
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
                let closed: Vec<Step> = display
                    .windows
                    .close_all()
                    .into_iter()
                    .map(Step::Send)
                    .collect();
                let resized = display.set_placement(Placement::new(&config.options.statusline));
                let awaiting = self.awaiting.take();
                let pending = std::mem::take(&mut self.pending);
                *self = Self::new(config, display);
                self.awaiting = awaiting;
                self.pending = pending;
                let mut events = vec![Event::ConfigReloaded];
                if previous != ROOT {
                    events.push(Event::KeyTableChanged {
                        table: ROOT.to_owned(),
                        previous,
                    });
                }
                self.refresh_pending = true;
                let mut steps = closed;
                steps.extend(resized.map(resize_step));
                steps.extend(self.react(display, events, |_, _, _| {}));
                steps
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
        let drawn = display.status_area().is_some();
        self.push_state(display, &mut steps);
        change(self, display, &mut steps);
        for depth in 0.. {
            self.table_changed(&mut table, &mut events);
            let after = display.observe();
            events.extend(changes(before, after.as_ref()));
            events.append(&mut self.pending);
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
                self.push_state(display, &mut steps);
                let outcome = self.runtime.emit(&event);
                self.apply(display, outcome, &mut steps);
            }
        }
        if !drawn && display.status_area().is_some() {
            self.refresh_pending = true;
        }
        if self.refresh_pending && display.view.is_some() {
            self.refresh_pending = false;
            self.refresh_now(display, &mut steps);
        }
        self.push_state(display, &mut steps);
        if let Some(line) = self.runtime.take_line() {
            display.set_status_line(line);
        }
        let frames = self.runtime.take_frames();
        steps.extend(display.windows.present(frames).into_iter().map(Step::Send));
        steps
    }

    fn refresh_now(&mut self, display: &mut Display, steps: &mut Vec<Step>) {
        self.push_state(display, steps);
        let outcome = self.runtime.refresh_statusline();
        self.apply(display, outcome, steps);
    }

    fn push_state(&mut self, display: &mut Display, steps: &mut Vec<Step>) {
        let outcome = self
            .runtime
            .set_state(display.view_state(self.leader.active()));
        self.apply(display, outcome, steps);
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
                Dispatch::Session(action) => {
                    steps.push(Step::Send(ClientMessage::Action(action)));
                }
                Dispatch::Input { pane, input } => steps.push(Step::Send(match input {
                    PaneInput::Key(key) => ClientMessage::Key { pane, key },
                    PaneInput::Paste(text) => ClientMessage::Paste { pane, text },
                })),
                Dispatch::Window(request) => steps.push(window_step(display, request)),
                Dispatch::Write(bytes) => steps.push(Step::Write(bytes)),
                Dispatch::Call { call, name, args } => {
                    steps.push(Step::Send(ClientMessage::Command { call, name, args }));
                }
                Dispatch::Targeted { .. } => {
                    tracing::debug!("ignoring a server action dispatched in the client")
                }
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

fn resize_step(size: Size) -> Step {
    Step::Send(ClientMessage::Resize {
        cols: size.cols,
        rows: size.rows,
    })
}

fn window_step(display: &mut Display, request: WindowRequest) -> Step {
    match request {
        WindowRequest::Open {
            window,
            target,
            width,
            focus,
        } => {
            let target = target.or_else(|| {
                let view = display.view.as_ref()?;
                Some((view.band(), view.focused()))
            });
            let Some((band, after)) = target else {
                return Step::Nothing;
            };
            Step::Send(display.windows.open(OpenRequest {
                window,
                band,
                after,
                width,
                focus,
            }))
        }
        WindowRequest::Close { window } => display
            .windows
            .close(window)
            .map_or(Step::Nothing, Step::Send),
    }
}

fn spawn(display: &mut Display, program: Option<Program>) -> Step {
    let open = display
        .view
        .as_ref()
        .and_then(|view| view.resolve(SessionCommand::OpenPane));
    match open {
        Some(SessionAction::OpenPane { band, after, .. }) => Step::Send(ClientMessage::Action(
            SessionAction::open(band, after, program),
        )),
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

pub(crate) async fn perform(
    connection: &mut Connection,
    steps: Vec<Step>,
) -> Result<Option<Outcome>> {
    for step in steps {
        match step {
            Step::Send(message) => connection.send(&message).await?,
            Step::Write(bytes) => {
                let mut out = stdout();
                if let Err(error) = out.write_all(&bytes).and_then(|()| out.flush()) {
                    tracing::warn!("cannot write to the terminal: {error}");
                }
            }
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
    display.set_colors(ColorSupport::detect());
    let mut controls = Controls::new(configuration.config, &mut display);
    if let Some(error) = configuration.error {
        display.set_banner(Some(error.to_string()));
    }
    let mut reloads = configuration.reloads;
    let mut channel = configuration.channel.map(Channel::new).transpose()?;
    controls.attach_when_ready(session.as_str());
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
        if let Some(channel) = &mut channel
            && !display.is_animating(now)
        {
            channel.drawn(connection.sent).await?;
        }
        let frame = display
            .is_animating(now)
            .then(|| tokio::time::Instant::from_std(now + FRAME));
        let timer = controls.next_timer().map(tokio::time::Instant::from_std);
        tokio::select! {
            () = tokio::time::sleep_until(frame.unwrap_or_else(tokio::time::Instant::now)),
                if frame.is_some() => {}
            () = tokio::time::sleep_until(timer.unwrap_or_else(tokio::time::Instant::now)),
                if timer.is_some() => {
                let steps = controls.fire_timers(&mut display, Instant::now());
                if let Some(outcome) = perform(connection, steps).await? {
                    return Ok(outcome);
                }
            }
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
            filled = async { channel.as_mut().expect("guarded by is_some").fill().await },
                if channel.is_some() => {
                let Some(open) = channel.as_mut().filter(|_| filled) else {
                    let _ = connection.send(&ClientMessage::Detach).await;
                    return Ok(Outcome::Detached);
                };
                let served = open.serve(&mut controls, &mut display, connection).await?;
                if let Served::Finished(outcome) = served {
                    return Ok(outcome);
                }
            }
            event = events.recv(), if controls.is_attached() => match event {
                Some(TerminalEvent::Key(key_event)) => {
                    let Some(key) = key_from_event(&key_event) else { continue };
                    let steps = controls.press(&mut display, key);
                    if let Some(outcome) = perform(connection, steps).await? {
                        return Ok(outcome);
                    }
                }
                Some(TerminalEvent::Paste(text)) => {
                    let steps = controls.paste(&mut display, text);
                    if let Some(outcome) = perform(connection, steps).await? {
                        return Ok(outcome);
                    }
                }
                Some(TerminalEvent::FocusGained) => {
                    if let Some(channel) = &mut channel {
                        channel.marker(connection.sent).await?;
                    }
                }
                Some(TerminalEvent::Resize(cols, rows)) => {
                    terminal.autoresize()?;
                    let steps = controls.resize(&mut display, Size::new(cols, rows));
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
    terminal.draw(|frame| display.draw(frame, now))?;
    Ok(())
}
