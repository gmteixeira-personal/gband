pub mod animation;
pub mod bindings;
mod channel;
pub mod color;
mod connect;
pub mod input;
pub mod mouse;
pub mod plugin_windows;
pub mod render;
mod requests;
mod transport;

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::io::{Write, stdout};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use anyhow::{Context, Result};
use crossterm::cursor::Show;
use crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event as TerminalEvent};
use crossterm::execute;
use crossterm::style::Print;
use gband_core::action::{Action, ClientAction, SessionCommand};
use gband_core::geometry::{Size, WindowBox, placed};
use gband_core::input::Key;
use gband_core::layout::{
    BandId, FloatingWindow, Layout, LayoutOptions, Program, Proportion, SessionAction, WindowId,
};
use gband_core::view::{CenterFocusedColumn, Layer, Scene, View, ViewAction};
use gband_emulator::{Emulator, Grid};
use gband_lua::{
    BandState, Bar, Binding, Border, Config, ConfigError, Dispatch, Event, Options, Outcome as Ran,
    PluginManifest, PluginWindowRequest, Requirement as Needed, Runtime, Slot, Version, ViewState,
    WindowInput, WindowStates,
};
use gband_protocol::{ClientMessage, ExecutableId, Requirement, ServerMessage, SessionName, Value};
use ratatui::layout::Rect;
use ratatui::{DefaultTerminal, Frame};
use tokio::sync::mpsc;

use crate::animation::{
    ANIMATIONS_VARIABLE, Animations, Drawn, FRAME, HeldBands, Hold, Presentation, Targets,
    parse_animations,
};
use crate::bindings::{Command, Keymap, Leader, ROOT};
use crate::channel::{Channel, Served};
pub use crate::channel::{Loader, TestChannel};
use crate::color::ColorSupport;
pub use crate::connect::{Connection, connect};
use crate::input::{key_from_event, mouse_from_event};
use crate::mouse::{
    Axis, DropPlace, Edges, Geometry, Gesture, Held, Hit, Motion, Pointer, Pressing, Resized,
    Sends, drop_columns, drop_place,
};
use crate::plugin_windows::{OpenRequest, Opened, PluginWindows};
use crate::render::{Lifted, Overlay, Region, RegionKind, Ribbon, Shown, draw_frame, regions};
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

const MOUSE_ON: &str = "\x1b[?1000h\x1b[?1002h\x1b[?1003h\x1b[?1006h";
const MOUSE_OFF: &str = "\x1b[?1006l\x1b[?1003l\x1b[?1002l\x1b[?1000l";

impl TerminalGuard {
    fn enter() -> Result<Self> {
        execute!(stdout(), EnableBracketedPaste).context("cannot enable bracketed paste")?;
        execute!(stdout(), Print(MOUSE_ON)).context("cannot enable mouse reporting")?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), Print(MOUSE_OFF));
        let _ = execute!(stdout(), DisableBracketedPaste);
        ratatui::restore();
        let _ = execute!(stdout(), Show);
    }
}

pub struct Display {
    layout: Arc<Layout>,
    area: Size,
    terminal: Size,
    ribbon: Rect,
    bars: Vec<Bar>,
    placed: Vec<Option<Rect>>,
    error_item: bool,
    grids: HashMap<WindowId, Grid>,
    view: Option<View>,
    shown: Option<Vec<WindowId>>,
    presentation: Presentation,
    prefix: Option<Key>,
    policy: CenterFocusedColumn,
    loop_bands: bool,
    banner: Option<String>,
    errors: Vec<String>,
    colors: ColorSupport,
    plugin_windows: PluginWindows,
    states: Arc<WindowStates>,
    ready: bool,
    tile_border: Border,
    floating_border: Border,
    pointer: Pointer,
    regions: Vec<Region>,
    framed: bool,
    drawn: Option<Drawn>,
}

impl Display {
    pub fn new(terminal: Size, animations: Animations) -> Self {
        Self {
            layout: Arc::new(Layout::new()),
            area: terminal,
            terminal,
            ribbon: Rect::new(0, 0, terminal.cols, terminal.rows),
            bars: Vec::new(),
            placed: Vec::new(),
            error_item: false,
            grids: HashMap::new(),
            view: None,
            shown: None,
            presentation: Presentation::new(animations),
            prefix: None,
            policy: CenterFocusedColumn::default(),
            loop_bands: true,
            banner: None,
            errors: Vec::new(),
            colors: ColorSupport::default(),
            plugin_windows: PluginWindows::new(),
            states: Arc::new(WindowStates::new()),
            ready: false,
            tile_border: Border::default(),
            floating_border: Border::default(),
            pointer: Pointer::default(),
            regions: Vec::new(),
            framed: false,
            drawn: None,
        }
    }

    pub fn copy_buffer(&self) -> &str {
        &self.pointer.copy_buffer
    }

    pub fn selection(&self) -> Option<crate::render::Selected> {
        self.pointer
            .selection
            .filter(|selection| selection.anchor != selection.head)
            .map(|selection| selection.selected())
    }

    pub fn regions(&self) -> &[Region] {
        &self.regions
    }

    pub fn hit(&mut self, col: u16, row: u16, now: Instant) -> Hit {
        if !self.framed {
            self.regions = self.current_regions(now);
        }
        let band = self.view.as_ref().map_or(BandId(0), View::band);
        mouse::hit(
            &self.regions,
            self.ribbon,
            band,
            |window| self.plugin_windows.plugin_window_of(window),
            col,
            row,
        )
    }

    fn region_of(&self, window: WindowId) -> Option<Region> {
        self.regions
            .iter()
            .rev()
            .find(|region| {
                region.window == Some(window) && !matches!(region.kind, RegionKind::PluginFloat(_))
            })
            .copied()
    }

    fn pend(&mut self, change: impl FnOnce(&mut Geometry)) {
        if let Some(sends) = &mut self.pointer.sends {
            change(&mut sends.pending);
        }
    }

    fn floating_box(&self, window: WindowId) -> Option<WindowBox> {
        let floating = self.layout.floating(window)?;
        Some(
            self.pointer
                .floating
                .filter(|moved| moved.window == window)
                .unwrap_or_else(|| placed(floating, self.area)),
        )
    }

    fn window_motion(
        &self,
        kind: ClientAction,
        window: WindowId,
        region: Region,
        edges: Edges,
        travel: i64,
    ) -> Option<Motion> {
        match (kind, region.kind) {
            (ClientAction::DragWindow, RegionKind::Floating) => Some(Motion::MoveFloating {
                window,
                origin: self.floating_box(window)?,
            }),
            (ClientAction::DragWindow, RegionKind::Tile { .. } | RegionKind::Lifted) => {
                Some(Motion::Lift {
                    window,
                    region,
                    drawn: self
                        .drawn
                        .as_ref()
                        .and_then(|drawn| drawn.tiles.get(&window).copied()),
                    columns: drop_columns(&self.regions, self.view.as_ref()?.band()),
                    lifted: false,
                })
            }
            (ClientAction::DragResize, RegionKind::Floating) => Some(Motion::Resize {
                target: Resized::Floating {
                    window,
                    origin: self.floating_box(window)?,
                },
                edges,
            }),
            (ClientAction::DragResize, RegionKind::Tile { .. } | RegionKind::Lifted) => {
                Some(Motion::Resize {
                    target: Resized::Tile {
                        window,
                        width: region.width,
                        height: region.height,
                        travel,
                    },
                    edges,
                })
            }
            _ => None,
        }
    }

    fn initial_sends(&self, gesture: &Gesture) -> Option<Sends> {
        let (window, sent) = match &gesture.motion {
            Motion::MoveFloating { window, origin } => (
                *window,
                Geometry {
                    position: Some((origin.x, origin.y)),
                    ..Geometry::default()
                },
            ),
            Motion::Resize {
                target: Resized::Floating { window, origin },
                ..
            } => (
                *window,
                Geometry {
                    position: Some((origin.x, origin.y)),
                    width: Some(origin.width),
                    height: Some(origin.height),
                },
            ),
            Motion::Resize {
                target:
                    Resized::Tile {
                        window,
                        width,
                        height,
                        ..
                    },
                ..
            } => (
                *window,
                Geometry {
                    position: None,
                    width: Some(*width),
                    height: Some(*height),
                },
            ),
            _ => return None,
        };
        Some(Sends {
            window,
            pending: Geometry::default(),
            sent,
        })
    }

    fn drop_action(&self, window: WindowId, place: DropPlace) -> Option<SessionAction> {
        let view = self.view.as_ref()?;
        let band = self.layout.band(view.band())?;
        let column = band.columns.get(place.column)?;
        let reference = match place.window {
            Some(reference) => reference,
            None => {
                let others: Vec<WindowId> = column
                    .windows
                    .iter()
                    .copied()
                    .filter(|&other| other != window)
                    .collect();
                view.recent_in(&others)?
            }
        };
        let action = SessionAction::MoveToPlace {
            window,
            reference,
            place: place.place,
        };
        let mut moved = (*self.layout).clone();
        let changed = !moved
            .apply(action.clone(), self.area, &LayoutOptions::default())
            .is_empty();
        changed.then_some(action)
    }

    fn overlay(&self) -> Overlay {
        let lifted = match self.pointer.gesture() {
            Some(
                gesture @ Gesture {
                    motion:
                        Motion::Lift {
                            window,
                            region,
                            columns,
                            lifted: true,
                            ..
                        },
                    ..
                },
            ) => {
                let cell = self.pointer.last_cell.unwrap_or(gesture.press);
                let (dx, dy) = gesture.moved(cell);
                let outline = drop_place(
                    columns,
                    *window,
                    self.ribbon,
                    i64::from(cell.0),
                    i64::from(cell.1),
                )
                .map(|place| place.outline);
                Some(Lifted {
                    window: *window,
                    x: region.x + dx,
                    y: region.y + dy,
                    width: region.width,
                    height: region.height,
                    outline,
                })
            }
            _ => None,
        };
        Overlay {
            selection: self.selection(),
            lifted,
            floating: self.pointer.floating,
        }
    }

    fn held_top(&self, band: BandId, dy: i64) -> Option<(i64, i64)> {
        let index = self.layout.band_index(band)? as i64;
        let height = i64::from(self.ribbon.height);
        let last = self.layout.bands().len() as i64 - 1;
        Some((
            index * height,
            (index * height - dy).clamp(0, last * height),
        ))
    }

    fn hold_bands(&mut self, band: BandId, dy: i64) {
        let (Some(view), Some((viewed, top))) = (&self.view, self.held_top(band, dy)) else {
            return;
        };
        let toward = match top.cmp(&viewed) {
            Ordering::Greater => Some(ViewAction::BandDown),
            Ordering::Less => Some(ViewAction::BandUp),
            Ordering::Equal => None,
        };
        let peek = toward.map(|action| {
            let mut neighbour = view.clone();
            neighbour.apply(action, self.scene());
            (
                neighbour.band(),
                neighbour.travel(),
                neighbour.strip(self.scene()),
            )
        });
        self.presentation.hold(Hold {
            vertical: Some(HeldBands { top, peek }),
            ..Hold::default()
        });
    }

    fn release_bands(&mut self, band: BandId, dy: i64, now: Instant) {
        let Some((viewed, top)) = self.held_top(band, dy) else {
            return;
        };
        let height = i64::from(self.ribbon.height).max(1);
        let middle = (top + height / 2) / height * height;
        let action = match middle.cmp(&viewed) {
            Ordering::Greater => ViewAction::BandDown,
            Ordering::Less => ViewAction::BandUp,
            Ordering::Equal => return self.presentation.release_bands(now),
        };
        self.presentation.hold(Hold::default());
        self.view_action(action);
    }

    fn end_gesture_for_layout(&mut self) {
        if let Some(gesture) = self.pointer.gesture().cloned()
            && let Motion::Slide {
                axis: Some(Axis::Vertical { band }),
                ..
            } = gesture.motion
        {
            if self.layout.band_index(band).is_some() {
                let (_, dy) = gesture.moved(self.pointer.last_cell.unwrap_or(gesture.press));
                self.hold_bands(band, dy);
            } else {
                self.pointer.held = Held::Free;
                self.presentation.hold(Hold::default());
            }
        }
        if let Some(window) = self.pointer.gesture().and_then(Gesture::window)
            && !self.layout.contains(window)
        {
            self.pointer.held = Held::Free;
            self.pointer.sends = None;
            self.pointer.floating = None;
            self.presentation.hold(Hold::default());
            if let Some(view) = &mut self.view {
                view.unpin();
            }
        }
        if self.pointer.gesture().is_none() && self.pointer.sends.is_none() {
            self.pointer.floating = None;
        }
        if let Some(selection) = self.pointer.selection
            && !self.layout.contains(selection.window)
        {
            self.pointer.selection = None;
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn window_state(&self, window: WindowId) -> Option<&BTreeMap<String, Value>> {
        self.states.get(&window)
    }

    fn set_state(&mut self, window: WindowId, key: String, value: Option<Value>) -> Option<Value> {
        let states = Arc::make_mut(&mut self.states);
        let state = states.entry(window).or_default();
        let previous = match value {
            Some(value) => state.insert(key, value),
            None => state.remove(&key),
        };
        if state.is_empty() {
            states.remove(&window);
        }
        previous
    }

    pub fn plugin_windows(&self) -> &PluginWindows {
        &self.plugin_windows
    }

    pub fn focused_plugin_window(&self) -> Option<u32> {
        self.plugin_windows.focused_float().or_else(|| {
            self.focused()
                .and_then(|window| self.plugin_windows.plugin_window_of(window))
        })
    }

    pub fn grid_size(&self, window: WindowId) -> Option<Size> {
        self.grids.get(&window).map(Grid::size)
    }

    pub fn configure(&mut self, options: &Options) {
        self.prefix = Some(options.prefix);
        self.policy = options.center_focused_column;
        self.tile_border = options.tile_border.clone();
        self.floating_border = options.floating_border.clone();
        let looping = std::mem::replace(&mut self.loop_bands, options.loop_bands);
        if let Some(view) = &mut self.view {
            view.set_center_focused_column(self.policy);
            view.set_loop_bands(self.loop_bands);
        }
        if looping != self.loop_bands {
            self.with_view(View::sync);
        }
    }

    pub fn set_colors(&mut self, colors: ColorSupport) {
        self.colors = colors;
    }

    pub fn colors(&self) -> ColorSupport {
        self.colors
    }

    pub fn reported_size(&self) -> Size {
        self.terminal
    }

    pub fn ribbon_area(&self) -> Rect {
        self.ribbon
    }

    fn ribbon_size(&self) -> Size {
        Size::new(self.ribbon.width, self.ribbon.height)
    }

    pub fn bars(&self) -> impl Iterator<Item = (&Bar, Rect)> {
        self.bars
            .iter()
            .zip(&self.placed)
            .filter_map(|(bar, placed)| placed.map(|placed| (bar, placed)))
    }

    pub fn set_error_item(&mut self, shown: bool) {
        self.error_item = shown;
    }

    fn place_bars(&mut self) -> bool {
        let slots: Vec<Slot> = self.bars.iter().map(|bar| bar.slot).collect();
        let (placed, ribbon) = gband_lua::bars::place(&slots, self.terminal.cols);
        let rows = self.terminal.rows;
        self.placed = placed
            .into_iter()
            .map(|columns| columns.map(|columns| Rect::new(columns.col, 0, columns.width, rows)))
            .collect();
        let ribbon = Rect::new(ribbon.col, 0, ribbon.width, rows);
        std::mem::replace(&mut self.ribbon, ribbon) != ribbon
    }

    pub fn set_size(&mut self, terminal: Size) -> Option<Size> {
        let resized = std::mem::replace(&mut self.terminal, terminal) != terminal;
        let moved = self.place_bars();
        if !resized && !moved {
            return None;
        }
        self.presentation.snap();
        self.with_view(View::sync);
        resized.then_some(terminal)
    }

    pub fn set_bars(&mut self, bars: Vec<Bar>) -> bool {
        self.bars = bars;
        let moved = self.place_bars();
        if moved {
            self.presentation.snap();
            self.with_view(View::sync);
        }
        moved
    }

    fn tiled_beside(&self, window: WindowId) -> Option<WindowId> {
        let band = self.layout.bands().iter().find(|band| band.holds(window))?;
        self.view.as_ref()?.tiled_in(band)
    }

    pub fn view_state(&self, table: &str) -> ViewState {
        let bands = self.layout.bands();
        let viewed = match &self.view {
            Some(view) => Some(view.band()),
            None => bands.first().map(|band| band.id),
        };
        let index = viewed.and_then(|id| bands.iter().position(|band| band.id == id));
        let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        ViewState {
            table: table.to_owned(),
            band: BandState {
                number: viewed.map_or(0, |id| id.0),
                index: index.map_or(0, |index| count(index + 1)),
                count: count(bands.len()),
            },
            window: self.focused().map(|window| window.0),
            width: self.terminal.cols,
            height: self.terminal.rows,
            error: self.banner.clone(),
            errors: self.errors.clone(),
            layout: Arc::clone(&self.layout),
            area: self.area,
            ribbon: self.ribbon_size(),
            states: Arc::clone(&self.states),
        }
    }

    pub fn banner(&self) -> Option<&str> {
        self.banner.as_deref()
    }

    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    pub fn report_error(&mut self, error: String) {
        self.errors.push(error.clone());
        self.banner = Some(error);
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
        self.banner = None;
    }

    pub fn report_shown(&mut self) -> Option<ClientMessage> {
        let shown = self.view.as_ref()?.shown(self.scene());
        if self.shown.as_ref() == Some(&shown) {
            return None;
        }
        self.shown = Some(shown.clone());
        Some(ClientMessage::Shown(shown))
    }

    pub fn focused(&self) -> Option<WindowId> {
        self.view.as_ref().and_then(View::focused)
    }

    pub fn camera(&self) -> Option<i64> {
        self.view.as_ref().map(View::camera)
    }

    pub fn observe(&self) -> Option<Observed> {
        let view = self.view.as_ref()?;
        Some(Observed {
            focused: view.focused(),
            band: view.band(),
            windows: self
                .layout
                .bands()
                .iter()
                .flat_map(|band| band.windows().map(move |window| (window, band.id)))
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
                        .map(|column| (column.windows.clone(), column.width, column.full_width))
                        .collect();
                    (band.id, columns, band.floating.clone())
                })
                .collect(),
        })
    }

    pub fn apply(&mut self, message: ServerMessage) -> Option<Outcome> {
        match message {
            ServerMessage::Layout { cols, rows, layout } => {
                self.grids.retain(|&window, _| layout.contains(window));
                let previous = Arc::clone(&self.layout);
                if self
                    .states
                    .keys()
                    .any(|&window| previous.contains(window) && !layout.contains(window))
                {
                    Arc::make_mut(&mut self.states)
                        .retain(|&window, _| layout.contains(window) || !previous.contains(window));
                }
                self.layout = Arc::new(layout);
                let area = Size::new(cols, rows);
                if area != self.area {
                    self.presentation.snap();
                }
                self.area = area;
                self.end_gesture_for_layout();
                self.sync();
            }
            ServerMessage::Snapshot {
                window,
                cols,
                rows,
                contents,
            } => {
                let mut grid = Grid::new(Size::new(cols, rows));
                grid.process(&contents);
                self.grids.insert(window, grid);
            }
            ServerMessage::Update { window, contents } => match self.grids.get_mut(&window) {
                Some(grid) => grid.process(&contents),
                None => {
                    tracing::warn!(window = %window, "ignoring an update for an unknown window")
                }
            },
            ServerMessage::Focus(window) => {
                self.with_view(|view, scene| view.focus_window(window, scene))
            }
            ServerMessage::Opened { .. }
            | ServerMessage::Event { .. }
            | ServerMessage::WindowState { .. }
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
            let mut view = View::with_policy(self.scene(), self.policy);
            view.set_loop_bands(self.loop_bands);
            view.sync(self.scene());
            self.view = Some(view);
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
        let targets = Targets::new(&self.layout, self.area, view, self.ribbon_size());
        self.presentation.update(now, &targets);
        Some(self.presentation.drawn(now))
    }

    fn ribbon<'a>(&'a self, view: &'a View, drawn: &'a Drawn) -> Ribbon<'a> {
        Ribbon {
            layout: &self.layout,
            area: self.area,
            view,
            grids: &self.grids,
            drawn,
            region: self.ribbon,
            tile_border: &self.tile_border,
            floating_border: &self.floating_border,
            floats: self.plugin_windows.floats(),
            float_focused: self.plugin_windows.focused_float().is_some(),
            colors: self.colors,
            banner: self.banner.as_deref().filter(|_| !self.error_item),
            bars: self.bars().map(|(bar, area)| Shown { bar, area }).collect(),
            overlay: self.overlay(),
        }
    }

    fn current_regions(&mut self, now: Instant) -> Vec<Region> {
        let drawn = self.present(now);
        let (Some(view), Some(drawn)) = (&self.view, &drawn) else {
            return Vec::new();
        };
        let area = Rect::new(0, 0, self.terminal.cols, self.terminal.rows);
        regions(&self.ribbon(view, drawn), area)
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>, now: Instant) {
        let drawn = self.present(now);
        let (Some(view), Some(shown)) = (&self.view, &drawn) else {
            return;
        };
        let ribbon = self.ribbon(view, shown);
        draw_frame(frame, &ribbon);
        let drawn_regions = regions(&ribbon, frame.area());
        self.regions = drawn_regions;
        self.framed = true;
        self.drawn = drawn;
        if let Some(selection) = self.pointer.selection
            && self.region_of(selection.window).is_none()
        {
            self.pointer.selection = None;
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        self.presentation.is_animating(now)
    }

    fn scene(&self) -> Scene<'_> {
        Scene {
            layout: &self.layout,
            area: self.area,
            viewport: self.ribbon_size(),
        }
    }

    fn with_view(&mut self, change: impl FnOnce(&mut View, Scene<'_>)) {
        let scene = Scene {
            layout: &self.layout,
            area: self.area,
            viewport: self.ribbon_size(),
        };
        if let Some(view) = &mut self.view {
            change(view, scene);
        }
    }

    fn key_to_focused(&self, key: Key) -> Option<ClientMessage> {
        self.focused()
            .map(|window| ClientMessage::Key { window, key })
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
    focused: Option<WindowId>,
    band: BandId,
    windows: BTreeMap<WindowId, BandId>,
    size: Size,
    shape: Vec<(BandId, Vec<ColumnShape>, Vec<FloatingWindow>)>,
}

type ColumnShape = (Vec<WindowId>, Proportion, bool);

fn changes(before: Option<Observed>, after: Option<&Observed>) -> Vec<Event> {
    let (Some(before), Some(after)) = (before, after) else {
        return Vec::new();
    };
    let mut events = Vec::new();
    for (&window, &band) in &before.windows {
        if !after.windows.contains_key(&window) {
            events.push(Event::WindowClosed { window, band });
        }
    }
    for (&window, &band) in &after.windows {
        if !before.windows.contains_key(&window) {
            events.push(Event::WindowOpened { window, band });
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
            window: after.focused,
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
    pressing: Option<Pressing>,
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
        config
            .runtime
            .set_plugin_window_counter(display.plugin_windows.counter());
        display.configure(&config.options);
        if config.errors.is_empty() {
            display.clear_errors();
        }
        for error in &config.errors {
            display.report_error(error.to_string());
        }
        Self {
            keymap: Keymap::new(&config.options, config.keymap, config.modes),
            runtime: config.runtime,
            leader: Leader::default(),
            refresh_pending: false,
            plugins: config.plugins,
            pending: Vec::new(),
            awaiting: None,
            pressing: None,
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
        display.report_error(error);
    }

    fn bridge(&mut self, display: &mut Display, message: ServerMessage, steps: &mut Vec<Step>) {
        match message {
            ServerMessage::WindowState { window, key, value } => {
                let previous = display.set_state(window, key.clone(), value.clone());
                if display.ready && previous != value {
                    self.pending.push(Event::WindowStateChanged {
                        window,
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
                if let ServerMessage::Opened { request, window } = message {
                    controls.opened(display, request, window, steps);
                    continue;
                }
                if matches!(
                    message,
                    ServerMessage::WindowState { .. }
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
            controls.track_windows(display, steps);
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
            if let Some(size) = display.set_size(terminal) {
                steps.push(resize_step(size));
            }
        })
    }

    pub fn press(&mut self, display: &mut Display, key: Key) -> Vec<Step> {
        self.react(display, Vec::new(), |controls, display, steps| {
            let released = controls.runtime.release_plugin_windows();
            controls.apply(display, released, steps);
            match controls.leader.handle(&controls.keymap, key) {
                Command::Send(key) => match display.focused_plugin_window() {
                    Some(plugin_window) => {
                        let outcome = controls.runtime.plugin_window_key(plugin_window, key);
                        controls.apply(display, outcome, steps);
                    }
                    None => steps.push(dispatch(
                        display,
                        Action::Client(ClientAction::SendKey(key)),
                    )),
                },
                Command::Run(Binding::Action(action)) => {
                    controls.run_action(display, action, steps)
                }
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
        if let Some(plugin_window) = display.focused_plugin_window() {
            return self.react(display, Vec::new(), |controls, display, steps| {
                let released = controls.runtime.release_plugin_windows();
                controls.apply(display, released, steps);
                let outcome = controls.runtime.plugin_window_paste(plugin_window, &text);
                controls.apply(display, outcome, steps);
            });
        }
        display.pointer.selection = None;
        display
            .focused()
            .map(|window| Step::Send(ClientMessage::Paste { window, text }))
            .into_iter()
            .collect()
    }

    fn opened(
        &mut self,
        display: &mut Display,
        request: u32,
        window: Option<WindowId>,
        steps: &mut Vec<Step>,
    ) {
        match display.plugin_windows.opened(request, window) {
            Opened::Close(window) => {
                if let Some(window) = window {
                    steps.push(Step::Send(ClientMessage::Action(
                        SessionAction::CloseWindow(window),
                    )));
                }
            }
            Opened::PluginWindow(plugin_window) => {
                let outcome = self.runtime.plugin_window_opened(plugin_window, window);
                self.apply(display, outcome, steps);
            }
        }
    }

    fn track_windows(&mut self, display: &mut Display, steps: &mut Vec<Step>) {
        for (window, plugin_window) in display.plugin_windows.windows() {
            if !display.layout.contains(window) {
                display.plugin_windows.forget(window);
                let outcome = self.runtime.window_closed(plugin_window);
                self.apply(display, outcome, steps);
                continue;
            }
            let Some(size) = display.grid_size(window) else {
                continue;
            };
            if display.plugin_windows.resized(window, size) {
                let outcome = self.runtime.window_resized(plugin_window, size);
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
                    .plugin_windows
                    .close_all()
                    .into_iter()
                    .map(Step::Send)
                    .collect();
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
                steps.extend(self.react(display, events, |_, _, _| {}));
                steps
            }
            Err(error) => {
                tracing::warn!("configuration error: {error}");
                display.report_error(error.to_string());
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
        self.push_state(display, &mut steps);
        change(self, display, &mut steps);
        if steps.iter().any(|step| {
            matches!(
                step,
                Step::Send(ClientMessage::Key { .. } | ClientMessage::Paste { .. })
            )
        }) {
            display.pointer.selection = None;
        }
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
        if self.refresh_pending && display.view.is_some() {
            self.refresh_pending = false;
            self.refresh_now(display, &mut steps);
        }
        self.push_state(display, &mut steps);
        if let Some(bars) = self.runtime.take_bars()
            && display.set_bars(bars)
        {
            self.push_state(display, &mut steps);
            if let Some(bars) = self.runtime.take_bars() {
                display.set_bars(bars);
            }
        }
        display.set_error_item(self.runtime.error_item_shown());
        let frames = self.runtime.take_frames();
        steps.extend(
            display
                .plugin_windows
                .present(frames)
                .into_iter()
                .map(Step::Send),
        );
        steps
    }

    fn refresh_now(&mut self, display: &mut Display, steps: &mut Vec<Step>) {
        self.push_state(display, steps);
        let outcome = self.runtime.refresh_plugins();
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

    fn run_action(&mut self, display: &mut Display, action: Action, steps: &mut Vec<Step>) {
        if let Action::Client(
            kind @ (ClientAction::DragWindow | ClientAction::DragResize | ClientAction::DragBand),
        ) = action
        {
            if let Some(pressing) = self.pressing.take() {
                self.start_gesture(display, pressing, kind, steps);
            }
            return;
        }
        if matches!(action, Action::Session(SessionCommand::CloseWindow)) {
            let (closed, outcome) = self.runtime.close_focused_plugin_window();
            self.apply(display, outcome, steps);
            if closed {
                return;
            }
        }
        steps.push(dispatch(display, action));
    }

    fn apply(&mut self, display: &mut Display, outcome: Ran, steps: &mut Vec<Step>) {
        for entry in outcome.dispatched {
            match entry {
                Dispatch::Action(action) => self.run_action(display, action, steps),
                Dispatch::Spawn(program) => steps.push(spawn(display, program)),
                Dispatch::Enter(table) => self.leader.enter(table),
                Dispatch::Session(SessionAction::ToggleFloating {
                    window,
                    after: None,
                }) => {
                    let after = display.tiled_beside(window);
                    let action = SessionAction::ToggleFloating { window, after };
                    steps.push(Step::Send(ClientMessage::Action(action)));
                }
                Dispatch::Session(action) => {
                    steps.push(Step::Send(ClientMessage::Action(action)));
                }
                Dispatch::Input { window, input } => steps.push(Step::Send(match input {
                    WindowInput::Key(key) => ClientMessage::Key { window, key },
                    WindowInput::Paste(text) => ClientMessage::Paste { window, text },
                })),
                Dispatch::PluginWindow(request) => steps.push(plugin_window_step(display, request)),
                Dispatch::Write(bytes) => steps.push(Step::Write(bytes)),
                Dispatch::Call { call, name, args } => {
                    steps.push(Step::Send(ClientMessage::Command { call, name, args }));
                }
                Dispatch::Targeted { .. } => {
                    tracing::debug!("ignoring a server action dispatched in the client")
                }
                Dispatch::ClearErrors => display.clear_errors(),
            }
        }
        for error in &outcome.errors {
            tracing::warn!("configuration error: {error}");
        }
        for error in &outcome.errors {
            display.report_error(error.to_string());
        }
    }
}

fn resize_step(size: Size) -> Step {
    Step::Send(ClientMessage::Resize {
        cols: size.cols,
        rows: size.rows,
    })
}

fn plugin_window_step(display: &mut Display, request: PluginWindowRequest) -> Step {
    match request {
        PluginWindowRequest::Open {
            plugin_window,
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
            Step::Send(display.plugin_windows.open(OpenRequest {
                plugin_window,
                band,
                after,
                width,
                focus,
            }))
        }
        PluginWindowRequest::Close { plugin_window } => display
            .plugin_windows
            .close(plugin_window)
            .map_or(Step::Nothing, Step::Send),
    }
}

fn spawn(display: &mut Display, program: Option<Program>) -> Step {
    let open = display
        .view
        .as_ref()
        .and_then(|view| view.resolve(SessionCommand::OpenWindow));
    match open {
        Some(SessionAction::OpenWindow { band, after, .. }) => Step::Send(ClientMessage::Action(
            SessionAction::open(band, after, program),
        )),
        _ => Step::Nothing,
    }
}

pub fn dispatch(display: &mut Display, action: Action) -> Step {
    let message = match action {
        Action::View(ViewAction::CenterColumn)
            if display
                .view
                .as_ref()
                .is_some_and(|view| view.layer() == Layer::Floating) =>
        {
            display
                .view
                .as_ref()
                .and_then(|view| view.centred_box(display.scene()))
                .map(ClientMessage::Action)
        }
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
        Action::Client(
            ClientAction::DragWindow | ClientAction::DragResize | ClientAction::DragBand,
        ) => None,
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
        display.report_error(error.to_string());
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
        let flush = controls
            .next_flush(&display)
            .map(tokio::time::Instant::from_std);
        tokio::select! {
            () = tokio::time::sleep_until(flush.unwrap_or_else(tokio::time::Instant::now)),
                if flush.is_some() => {
                let steps = controls.flush(&mut display, Instant::now());
                if let Some(outcome) = perform(connection, steps).await? {
                    return Ok(outcome);
                }
            }
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
                Some(TerminalEvent::Mouse(mouse_event)) => {
                    let Some(event) = mouse_from_event(&mouse_event) else { continue };
                    let steps = controls.mouse(&mut display, event, Instant::now());
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
