use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::action::SessionCommand;
use crate::geometry::{Size, Span, WindowBox, boxes, column_spans, drawn_copy, loop_width, tiles};
use crate::layout::{Band, BandId, Direction, Layout, Location, Place, SessionAction, WindowId};

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
    CenterColumn,
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
    travel: i64,
    pinned: bool,
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
    loop_bands: bool,
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
            loop_bands: true,
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

    pub fn set_loop_bands(&mut self, loop_bands: bool) {
        self.loop_bands = loop_bands;
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

    pub fn travel(&self) -> i64 {
        self.bands.get(&self.band).map_or(0, |state| state.travel)
    }

    pub fn recent_in(&self, windows: &[WindowId]) -> Option<WindowId> {
        let recent = windows
            .iter()
            .filter_map(|window| self.recency.get(window).map(|&tick| (tick, *window)))
            .max()
            .map(|(_, window)| window);
        recent.or_else(|| windows.first().copied())
    }

    pub fn slide(&mut self, travel: i64, scene: Scene<'_>) {
        let strip = self.strip(scene);
        let state = self.bands.entry(self.band).or_default();
        state.travel = travel;
        state.camera = strip.map_or(travel, |strip| travel.rem_euclid(i64::from(strip)));
        state.pinned = true;
    }

    pub fn unpin(&mut self) {
        if let Some(state) = self.bands.get_mut(&self.band) {
            state.pinned = false;
        }
    }

    pub fn release_slide(&mut self, scene: Scene<'_>, press_travel: i64) {
        let Some(band) = scene.layout.band(self.band) else {
            return;
        };
        if band.columns.is_empty() {
            self.slide(press_travel, scene);
            self.unpin();
            return;
        }
        self.unpin();
        let spans = column_spans(band, scene.area);
        let mut middle = self.camera() + i64::from(scene.viewport.cols / 2);
        if let Some(strip) = self.looping(&spans, scene.viewport.cols) {
            middle = middle.rem_euclid(i64::from(strip));
        }
        let distance = |span: &Span| {
            let (start, end) = (i64::from(span.x), i64::from(span.end()));
            if middle < start {
                start - middle
            } else if middle >= end {
                middle - end + 1
            } else {
                0
            }
        };
        let Some(column) = (0..spans.len()).min_by_key(|&index| distance(&spans[index])) else {
            return;
        };
        let Some(window) = self.recent_in(&band.columns[column].windows) else {
            return;
        };
        self.focus(band, window);
        if let Some(location) = scene.layout.locate(window) {
            self.position = location;
        }
        self.aim(band.id, &spans, column, scene.viewport.cols, false, None);
    }

    pub fn strip(&self, scene: Scene<'_>) -> Option<u32> {
        let band = scene.layout.band(self.band)?;
        self.looping(&column_spans(band, scene.area), scene.viewport.cols)
    }

    fn looping(&self, spans: &[Span], viewport: u16) -> Option<u32> {
        loop_width(spans, viewport).filter(|_| self.loop_bands)
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

    pub fn centred_box(&self, scene: Scene<'_>) -> Option<SessionAction> {
        let window = self.focused()?;
        let band = scene.layout.band(self.band)?;
        let placed = boxes(band, scene.area)
            .into_iter()
            .find(|placed| placed.window == window)?;
        let col = (scene.area.cols - placed.width) / 2;
        let row = (scene.area.rows - placed.height) / 2;
        ((placed.x, placed.y) != (col, row)).then_some(SessionAction::SetPosition {
            window,
            col,
            row,
        })
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
        let camera = self.camera();
        let strip = self.strip(scene);
        let cols = scene.viewport.cols;
        let tiled = tiles(band, scene.area)
            .into_iter()
            .filter(|tile| {
                let left = i64::from(tile.x) - camera;
                let left = strip.map_or(left, |strip| drawn_copy(left, strip, cols));
                tile.width > 0
                    && tile.height > 0
                    && left < i64::from(cols)
                    && left + i64::from(tile.width) > 0
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
        let mut heading = None;
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
                    heading = match (self.layer(), action) {
                        (Layer::Tiled, ViewAction::FocusLeft) => Some(Direction::Left),
                        (Layer::Tiled, ViewAction::FocusRight) => Some(Direction::Right),
                        _ => None,
                    };
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
            ViewAction::CenterColumn => {
                let Some((column, _)) = self
                    .tiled_focus()
                    .filter(|_| self.layer() == Layer::Tiled)
                    .and_then(|window| band.locate(window))
                else {
                    return;
                };
                let spans = column_spans(band, scene.area);
                self.aim(band.id, &spans, column, scene.viewport.cols, true, None);
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
        self.settle(scene, previous, heading);
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
        self.settle(scene, previous, None);
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
        self.settle(scene, previous, None);
    }

    pub fn sync(&mut self, scene: Scene<'_>) {
        self.settle(scene, None, None);
    }

    fn settle(&mut self, scene: Scene<'_>, previous: Option<WindowId>, heading: Option<Direction>) {
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
        self.follow(band, scene, previous, heading);
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
        let last = band.columns.len() - 1;
        let target = match action {
            ViewAction::FocusLeft if column == 0 && self.loop_bands => last,
            ViewAction::FocusLeft => column.checked_sub(1)?,
            _ if column == last && self.loop_bands => 0,
            _ => column + 1,
        };
        if target == column {
            return None;
        }
        self.recent_in(&band.columns.get(target)?.windows)
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

    fn follow(
        &mut self,
        band: &Band,
        scene: Scene<'_>,
        previous: Option<WindowId>,
        heading: Option<Direction>,
    ) {
        if self.bands.get(&band.id).is_some_and(|state| state.pinned) {
            return;
        }
        let Some((column, _)) = self.tiled_focus().and_then(|window| band.locate(window)) else {
            return;
        };
        let spans = column_spans(band, scene.area);
        let viewport = i64::from(scene.viewport.cols);
        let from = previous
            .and_then(|window| band.locate(window))
            .map(|(from, _)| from)
            .filter(|&from| from != column);
        let centre = match self.policy {
            CenterFocusedColumn::Never => false,
            CenterFocusedColumn::Always => true,
            CenterFocusedColumn::OnOverflow => from.is_some_and(|from| {
                let looping = self.looping(&spans, scene.viewport.cols).is_some();
                let beside = match heading {
                    Some(_) if looping => from,
                    _ if from < column => column - 1,
                    _ => column + 1,
                };
                i64::from(spans[beside].width) + i64::from(spans[column].width) > viewport
            }),
        };
        let anchor = heading.zip(from);
        self.aim(band.id, &spans, column, scene.viewport.cols, centre, anchor);
    }

    fn aim(
        &mut self,
        band: BandId,
        spans: &[Span],
        column: usize,
        cols: u16,
        centre: bool,
        anchor: Option<(Direction, usize)>,
    ) {
        let viewport = i64::from(cols);
        let strip = self.looping(spans, cols);
        let state = self.bands.entry(band).or_default();
        let camera = state.camera;
        let width = i64::from(spans[column].width);
        let place = |x: i64| {
            if centre {
                centred(x, width, viewport)
            } else {
                revealed(x, width, viewport, camera)
            }
        };
        let x = i64::from(spans[column].x);
        let Some(strip) = strip else {
            state.camera = place(x);
            state.travel = state.camera;
            return;
        };
        let period = i64::from(strip);
        let beside = anchor.and_then(|(heading, from)| {
            let span = spans[from];
            let left = drawn_copy(i64::from(span.x) - camera, strip, cols);
            let from_width = i64::from(span.width);
            (left < viewport && left + from_width > 0).then(|| {
                let start = camera + left;
                match heading {
                    Direction::Right => x - (x - start - from_width).div_euclid(period) * period,
                    Direction::Left => x + (start - x - width).div_euclid(period) * period,
                }
            })
        });
        let copy = beside.unwrap_or_else(|| {
            let nearest = (camera - x).div_euclid(period);
            (nearest - 1..=nearest + 2)
                .map(|lap| x + lap * period)
                .min_by_key(|&copy| ((place(copy) - camera).abs(), copy))
                .unwrap_or(x)
        });
        let moved = place(copy);
        state.travel += moved - camera;
        state.camera = moved.rem_euclid(period);
    }
}

fn tiled_target(band: &Band, state: BandView) -> Option<WindowId> {
    state
        .tiled
        .filter(|&window| band.locate(window).is_some())
        .or_else(|| band.first_window())
}

fn centred(x: i64, width: i64, viewport: i64) -> i64 {
    if width >= viewport {
        x
    } else {
        x - (viewport - width) / 2
    }
}

fn revealed(x: i64, width: i64, viewport: i64, camera: i64) -> i64 {
    let end = x + width;
    if x >= camera && end <= camera + viewport {
        camera
    } else if width >= viewport || x < camera {
        x
    } else {
        end - viewport
    }
}
