use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::action::SessionCommand;
use crate::geometry::{Size, Span, WindowBox, boxes, column_spans, tiles};
use crate::layout::{Band, BandId, Layout, Location, Place, SessionAction, WindowId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewAction {
    FocusLeft,
    FocusRight,
    FocusDown,
    FocusUp,
    BandDown,
    BandUp,
    FocusWindow(WindowId),
    ViewBand(BandId),
    SwitchLayer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer {
    #[default]
    Tiled,
    Floating,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CenterFocusedColumn {
    #[default]
    Never,
    Always,
    OnOverflow,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BandView {
    layer: Layer,
    tiled: Option<WindowId>,
    floating: Option<WindowId>,
    camera: i64,
}

impl BandView {
    fn focused(&self) -> Option<WindowId> {
        match self.layer {
            Layer::Tiled => self.tiled,
            Layer::Floating => self.floating,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scene<'a> {
    pub layout: &'a Layout,
    pub area: Size,
    pub viewport: Size,
}

#[derive(Clone, Debug)]
pub struct View {
    band: BandId,
    bands: HashMap<BandId, BandView>,
    position: Location,
    recency: HashMap<WindowId, u64>,
    tick: u64,
    policy: CenterFocusedColumn,
}

impl View {
    pub fn new(scene: Scene<'_>) -> Self {
        Self::with_policy(scene, CenterFocusedColumn::default())
    }

    pub fn with_policy(scene: Scene<'_>, policy: CenterFocusedColumn) -> Self {
        let first = &scene.layout.bands()[0];
        let mut view = Self {
            band: first.id,
            bands: HashMap::new(),
            position: Location::default(),
            recency: HashMap::new(),
            tick: 0,
            policy,
        };
        if let Some(window) = first.first_window() {
            view.focus(first, window);
        }
        view.sync(scene);
        view
    }

    pub fn set_center_focused_column(&mut self, policy: CenterFocusedColumn) {
        self.policy = policy;
    }

    pub fn band(&self) -> BandId {
        self.band
    }

    pub fn focused(&self) -> Option<WindowId> {
        self.bands.get(&self.band).and_then(BandView::focused)
    }

    pub fn layer(&self) -> Layer {
        self.bands
            .get(&self.band)
            .map_or(Layer::Tiled, |state| state.layer)
    }

    fn tiled_focus(&self) -> Option<WindowId> {
        self.bands.get(&self.band).and_then(|state| state.tiled)
    }

    pub fn tiled_in(&self, band: &Band) -> Option<WindowId> {
        self.bands
            .get(&band.id)
            .and_then(|state| state.tiled)
            .filter(|&window| band.locate(window).is_some())
    }

    pub fn camera(&self) -> i64 {
        self.bands.get(&self.band).map_or(0, |state| state.camera)
    }

    pub fn resolve(&self, command: SessionCommand) -> Option<SessionAction> {
        let tiled = self.tiled_focus();
        match command {
            SessionCommand::OpenWindow => Some(SessionAction::open(self.band, tiled, None)),
            SessionCommand::ToggleFloating => Some(SessionAction::ToggleFloating {
                window: self.focused()?,
                after: tiled,
            }),
            command => command.on_window(self.focused()?),
        }
    }

    pub fn stacking(&self, scene: Scene<'_>) -> Vec<WindowId> {
        scene
            .layout
            .band(self.band)
            .map_or_else(Vec::new, |band| self.stacked(band))
    }

    fn stacked(&self, band: &Band) -> Vec<WindowId> {
        let mut order: Vec<(Option<u64>, usize, WindowId)> = band
            .floating
            .iter()
            .enumerate()
            .map(|(index, floating)| {
                (
                    self.recency.get(&floating.window).copied(),
                    index,
                    floating.window,
                )
            })
            .collect();
        order.sort_unstable();
        order.into_iter().map(|(_, _, window)| window).collect()
    }

    fn top_floating(&self, band: &Band) -> Option<WindowId> {
        self.stacked(band).last().copied()
    }

    pub fn shown(&self, scene: Scene<'_>) -> Vec<WindowId> {
        let Some(band) = scene.layout.band(self.band) else {
            return Vec::new();
        };
        let left = self.camera();
        let right = left + i64::from(scene.viewport.cols);
        let tiled = tiles(band, scene.area)
            .into_iter()
            .filter(|tile| {
                tile.width > 0
                    && tile.height > 0
                    && i64::from(tile.x) < right
                    && i64::from(tile.span().end()) > left
                    && tile.y < scene.viewport.rows
            })
            .map(|tile| tile.window);
        let floating = boxes(band, scene.area)
            .into_iter()
            .filter(|placed| {
                placed.width > 0
                    && placed.height > 0
                    && placed.x < scene.viewport.cols
                    && placed.y < scene.viewport.rows
            })
            .map(|placed| placed.window);
        tiled.chain(floating).collect()
    }

    pub fn apply(&mut self, action: ViewAction, scene: Scene<'_>) {
        match action {
            ViewAction::FocusWindow(window) => return self.focus_window(window, scene),
            ViewAction::ViewBand(band) => return self.view_band(band, scene),
            _ => {}
        }
        let previous = self.focused();
        let Some(index) = scene.layout.band_index(self.band) else {
            self.sync(scene);
            return;
        };
        let bands = scene.layout.bands();
        let band = &bands[index];
        match action {
            ViewAction::FocusLeft
            | ViewAction::FocusRight
            | ViewAction::FocusDown
            | ViewAction::FocusUp => {
                let target = match (self.layer(), action) {
                    (Layer::Floating, _) => self.neighbour_box(band, action, scene.area),
                    (Layer::Tiled, ViewAction::FocusLeft | ViewAction::FocusRight) => {
                        self.neighbour_column(band, action)
                    }
                    (Layer::Tiled, _) => self.neighbour_row(band, action),
                };
                if let Some(window) = target {
                    self.focus(band, window);
                }
            }
            ViewAction::SwitchLayer => {
                let state = self.bands.get(&band.id).copied().unwrap_or_default();
                let target = match state.layer {
                    Layer::Tiled => self.floating_target(band, state),
                    Layer::Floating => tiled_target(band, state),
                };
                if let Some(window) = target {
                    self.focus(band, window);
                }
            }
            ViewAction::BandDown | ViewAction::BandUp => {
                let target = match action {
                    ViewAction::BandDown => index + 1,
                    _ => match index.checked_sub(1) {
                        Some(target) => target,
                        None => return,
                    },
                };
                let Some(target) = bands.get(target) else {
                    return;
                };
                self.enter(target);
            }
            ViewAction::FocusWindow(_) | ViewAction::ViewBand(_) => {}
        }
        self.settle(scene, previous);
    }

    pub fn view_band(&mut self, band: BandId, scene: Scene<'_>) {
        if band == self.band {
            return;
        }
        let Some(target) = scene.layout.band(band) else {
            return;
        };
        let previous = self.focused();
        self.enter(target);
        self.settle(scene, previous);
    }

    pub fn focus_window(&mut self, window: WindowId, scene: Scene<'_>) {
        let index = match scene.layout.place(window) {
            Some(Place::Tiled(location)) => location.band,
            Some(Place::Floating { band, .. }) => band,
            None => return,
        };
        let previous = self.focused();
        let band = &scene.layout.bands()[index];
        self.band = band.id;
        self.focus(band, window);
        self.settle(scene, previous);
    }

    pub fn sync(&mut self, scene: Scene<'_>) {
        self.settle(scene, None);
    }

    fn settle(&mut self, scene: Scene<'_>, previous: Option<WindowId>) {
        let layout = scene.layout;
        self.bands.retain(|&id, _| layout.band_index(id).is_some());
        self.recency.retain(|&window, _| layout.contains(window));

        let index = match layout.band_index(self.band) {
            Some(index) => index,
            None => {
                let index = self.position.band.min(layout.bands().len() - 1);
                self.enter(&layout.bands()[index]);
                index
            }
        };
        let band = &layout.bands()[index];
        let state = *self.bands.entry(band.id).or_default();
        let target = match state.focused() {
            Some(window) if band.holds(window) => Some(window),
            Some(_) => match state.layer {
                Layer::Tiled => self.clamped(band).or_else(|| self.top_floating(band)),
                Layer::Floating => self
                    .top_floating(band)
                    .or_else(|| tiled_target(band, state)),
            },
            None => band.first_window().or_else(|| self.top_floating(band)),
        };
        match target {
            Some(window) if Some(window) == state.focused() => self.place_focus(band, window),
            Some(window) => self.focus(band, window),
            None => self.clear_focus(),
        }
        let state = self.bands.entry(band.id).or_default();
        state.tiled = state.tiled.filter(|&window| band.locate(window).is_some());
        state.floating = state
            .floating
            .filter(|&window| band.floating_index(window).is_some());
        self.position = match state.tiled.and_then(|window| layout.locate(window)) {
            Some(location) => location,
            None => Location {
                band: index,
                column: 0,
                row: 0,
            },
        };
        self.follow(band, scene, previous);
    }

    fn enter(&mut self, band: &Band) {
        self.band = band.id;
        let state = *self.bands.entry(band.id).or_default();
        let target = state
            .focused()
            .filter(|&window| band.holds(window))
            .or_else(|| band.first_window())
            .or_else(|| self.top_floating(band));
        if let Some(window) = target {
            self.focus(band, window);
        }
    }

    fn floating_target(&self, band: &Band, state: BandView) -> Option<WindowId> {
        state
            .floating
            .filter(|&window| band.floating_index(window).is_some())
            .or_else(|| self.top_floating(band))
    }

    fn clamped(&self, band: &Band) -> Option<WindowId> {
        let column = band
            .columns
            .get(self.position.column)
            .or(band.columns.last())?;
        let row = self.position.row.min(column.windows.len() - 1);
        Some(column.windows[row])
    }

    fn neighbour_column(&self, band: &Band, action: ViewAction) -> Option<WindowId> {
        let (column, _) = band.locate(self.focused()?)?;
        let target = match action {
            ViewAction::FocusLeft => column.checked_sub(1)?,
            _ => column + 1,
        };
        let windows = &band.columns.get(target)?.windows;
        let recent = windows
            .iter()
            .filter_map(|window| self.recency.get(window).map(|&tick| (tick, *window)))
            .max()
            .map(|(_, window)| window);
        Some(recent.unwrap_or(windows[0]))
    }

    fn neighbour_row(&self, band: &Band, action: ViewAction) -> Option<WindowId> {
        let (column, row) = band.locate(self.focused()?)?;
        let target = match action {
            ViewAction::FocusUp => row.checked_sub(1)?,
            _ => row + 1,
        };
        band.columns[column].windows.get(target).copied()
    }

    fn neighbour_box(&self, band: &Band, action: ViewAction, area: Size) -> Option<WindowId> {
        let focused = self.focused()?;
        let placed = boxes(band, area);
        let centre = |placed: &WindowBox| {
            (
                2 * i32::from(placed.x) + i32::from(placed.width),
                2 * i32::from(placed.y) + i32::from(placed.height),
            )
        };
        let (x, y) = centre(placed.iter().find(|placed| placed.window == focused)?);
        placed
            .iter()
            .enumerate()
            .filter(|(_, candidate)| candidate.window != focused)
            .filter_map(|(index, candidate)| {
                let (cx, cy) = centre(candidate);
                let (along, across) = match action {
                    ViewAction::FocusLeft => (x - cx, (cy - y).abs()),
                    ViewAction::FocusRight => (cx - x, (cy - y).abs()),
                    ViewAction::FocusUp => (y - cy, (cx - x).abs()),
                    _ => (cy - y, (cx - x).abs()),
                };
                (along > 0).then_some((along, across, index, candidate.window))
            })
            .min()
            .map(|(_, _, _, window)| window)
    }

    fn focus(&mut self, band: &Band, window: WindowId) {
        self.place_focus(band, window);
        self.touch(window);
    }

    fn place_focus(&mut self, band: &Band, window: WindowId) {
        let state = self.bands.entry(band.id).or_default();
        if band.floating_index(window).is_some() {
            state.layer = Layer::Floating;
            state.floating = Some(window);
        } else {
            state.layer = Layer::Tiled;
            state.tiled = Some(window);
        }
    }

    fn clear_focus(&mut self) {
        let state = self.bands.entry(self.band).or_default();
        state.layer = Layer::Tiled;
        state.tiled = None;
        state.floating = None;
    }

    fn touch(&mut self, window: WindowId) {
        self.tick += 1;
        self.recency.insert(window, self.tick);
    }

    fn follow(&mut self, band: &Band, scene: Scene<'_>, previous: Option<WindowId>) {
        let Some((column, _)) = self.tiled_focus().and_then(|window| band.locate(window)) else {
            return;
        };
        let spans = column_spans(band, scene.area);
        let span = spans[column];
        let viewport = i64::from(scene.viewport.cols);
        let state = self.bands.entry(band.id).or_default();
        let centre = match self.policy {
            CenterFocusedColumn::Never => false,
            CenterFocusedColumn::Always => true,
            CenterFocusedColumn::OnOverflow => previous
                .and_then(|window| band.locate(window))
                .filter(|&(from, _)| from != column)
                .is_some_and(|(from, _)| {
                    let beside = if from < column {
                        column - 1
                    } else {
                        column + 1
                    };
                    let (left, right) = (spans[column.min(beside)], spans[column.max(beside)]);
                    i64::from(right.end()) - i64::from(left.x) > viewport
                }),
        };
        state.camera = if centre {
            centred(span, viewport)
        } else {
            revealed(span, viewport, state.camera)
        };
    }
}

fn tiled_target(band: &Band, state: BandView) -> Option<WindowId> {
    state
        .tiled
        .filter(|&window| band.locate(window).is_some())
        .or_else(|| band.first_window())
}

fn centred(span: Span, viewport: i64) -> i64 {
    let width = i64::from(span.width);
    let x = i64::from(span.x);
    if width >= viewport {
        x
    } else {
        x - (viewport - width) / 2
    }
}

fn revealed(span: Span, viewport: i64, camera: i64) -> i64 {
    let (x, end, width) = (
        i64::from(span.x),
        i64::from(span.end()),
        i64::from(span.width),
    );
    if x >= camera && end <= camera + viewport {
        camera
    } else if width >= viewport || x < camera {
        x
    } else {
        end - viewport
    }
}
