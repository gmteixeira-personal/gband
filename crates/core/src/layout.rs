use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::event::LayoutEvent;
use crate::geometry::{
    MIN_TILE_HEIGHT, Size, WindowBox, fixed_height_limit, height_step, placed, width_step,
    window_heights,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WindowId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BandId(pub u32);

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for BandId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    Grow,
    Shrink,
}

pub(crate) fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proportion {
    pub num: u32,
    pub den: u32,
}

impl Proportion {
    pub const ONE_THIRD: Self = Self::new(1, 3);
    pub const ONE_HALF: Self = Self::new(1, 2);
    pub const TWO_THIRDS: Self = Self::new(2, 3);
    pub const WHOLE: Self = Self::new(1, 1);
    pub const TENTH: Self = Self::new(1, 10);

    pub const MAX: u32 = 10000;

    pub const fn new(num: u32, den: u32) -> Self {
        Self { num, den }
    }

    pub fn of(self, cells: u16) -> u16 {
        let cells = u64::from(cells) * u64::from(self.num) / u64::from(self.den);
        cells.min(u64::from(u16::MAX)) as u16
    }

    pub fn step(self, step: Step, by: Self) -> Self {
        let den = u64::from(self.den) * u64::from(by.den);
        let num = u64::from(self.num) * u64::from(by.den);
        let delta = u64::from(by.num) * u64::from(self.den);
        let num = match step {
            Step::Grow => num + delta,
            Step::Shrink => num.saturating_sub(delta),
        };
        let num = num.min(u64::from(Self::MAX) * den);
        let divisor = gcd(num, den).max(1);
        let (mut num, mut den) = (num / divisor, den / divisor);
        while den > u64::from(u32::MAX) {
            num /= 2;
            den /= 2;
            let divisor = gcd(num, den).max(1);
            (num, den) = (num / divisor, den / divisor);
        }
        Self::new(num as u32, den as u32)
    }

    pub fn rows_of(self, rows: u16) -> u16 {
        let scaled = u64::from(rows) * u64::from(self.num) * 2 + u64::from(self.den);
        let rows = scaled / (2 * u64::from(self.den));
        rows.clamp(1, u64::from(u16::MAX)) as u16
    }

    pub fn lowest(self) -> Self {
        let divisor = gcd(u64::from(self.num), u64::from(self.den)).max(1) as u32;
        Self::new(self.num / divisor, self.den / divisor)
    }

    fn exceeds(self, other: Self) -> bool {
        u64::from(self.num) * u64::from(other.den) > u64::from(other.num) * u64::from(self.den)
    }
}

fn effective_width((width, full_width): (Proportion, bool)) -> Proportion {
    if full_width { Proportion::WHOLE } else { width }
}

pub fn cycle_width(current: (Proportion, bool), presets: &[Proportion]) -> (Proportion, bool) {
    let effective = effective_width(current);
    let smallest = presets.iter().copied().reduce(|smallest, preset| {
        if smallest.exceeds(preset) {
            preset
        } else {
            smallest
        }
    });
    let larger = presets
        .iter()
        .copied()
        .filter(|preset| preset.exceeds(effective))
        .reduce(|nearest, preset| {
            if nearest.exceeds(preset) {
                preset
            } else {
                nearest
            }
        });
    match larger.or(smallest) {
        Some(width) => (width, false),
        None => current,
    }
}

pub fn step_width(current: (Proportion, bool), step: Step, by: Proportion) -> (Proportion, bool) {
    (effective_width(current).step(step, by), false)
}

pub fn set_width(width: Proportion) -> (Proportion, bool) {
    (width.lowest(), false)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weight {
    num: u32,
    den: u32,
}

impl Weight {
    pub const ONE: Self = Self { num: 1, den: 1 };

    pub fn new(num: u32, den: u32) -> Self {
        let divisor = gcd(u64::from(num), u64::from(den)).max(1) as u32;
        Self {
            num: num / divisor,
            den: den / divisor,
        }
    }

    pub fn num(self) -> u32 {
        self.num
    }

    pub fn den(self) -> u32 {
        self.den
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowHeight {
    Auto(Weight),
    Fixed(u16),
}

impl WindowHeight {
    pub const DEFAULT: Self = Self::Auto(Weight::ONE);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutOptions {
    pub default_width: Proportion,
    pub presets: Vec<Proportion>,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            default_width: Proportion::ONE_HALF,
            presets: vec![
                Proportion::ONE_THIRD,
                Proportion::ONE_HALF,
                Proportion::TWO_THIRDS,
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Program {
    CommandLine(String),
    Argv(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Column {
    pub windows: Vec<WindowId>,
    pub heights: Vec<WindowHeight>,
    pub width: Proportion,
    pub full_width: bool,
}

impl Column {
    pub fn new(window: WindowId, width: Proportion) -> Self {
        Self {
            windows: vec![window],
            heights: vec![WindowHeight::DEFAULT],
            width,
            full_width: false,
        }
    }

    fn push(&mut self, window: WindowId) {
        self.windows.push(window);
        self.heights.push(WindowHeight::DEFAULT);
    }

    fn remove(&mut self, row: usize) {
        self.windows.remove(row);
        self.heights.remove(row);
        if let [last @ WindowHeight::Auto(_)] = self.heights.as_mut_slice() {
            *last = WindowHeight::DEFAULT;
        }
    }

    fn width(&self) -> (Proportion, bool) {
        (self.width, self.full_width)
    }

    pub fn step_width(&mut self, step: Step, by: Proportion) {
        (self.width, self.full_width) = step_width(self.width(), step, by);
    }

    pub fn set_width(&mut self, width: Proportion) {
        (self.width, self.full_width) = set_width(width);
    }

    fn step_height(&mut self, row: usize, step: Step, by: Proportion, area: Size) {
        let rows = window_heights(self, area.rows);
        let step_rows = by.rows_of(area.rows);
        let target = match step {
            Step::Grow => rows[row].saturating_add(step_rows),
            Step::Shrink => rows[row].saturating_sub(step_rows),
        };
        self.fix_height(row, target, &rows, area);
    }

    pub fn set_height(&mut self, row: usize, height: WindowHeight, area: Size) {
        match height {
            WindowHeight::Fixed(target) => {
                let rows = window_heights(self, area.rows);
                self.fix_height(row, target, &rows, area);
            }
            WindowHeight::Auto(weight) => self.heights[row] = WindowHeight::Auto(weight),
        }
    }

    fn fix_height(&mut self, row: usize, target: u16, rows: &[u16], area: Size) {
        if matches!(self.heights[row], WindowHeight::Auto(_)) {
            let mut sorted = rows.to_vec();
            sorted.sort_unstable();
            let median = sorted[sorted.len() / 2].max(1);
            for (height, &tile) in self.heights.iter_mut().zip(rows) {
                *height =
                    WindowHeight::Auto(Weight::new(u32::from(tile.max(1)), u32::from(median)));
            }
        }
        let ceiling = fixed_height_limit(self.windows.len(), area.rows);
        self.heights[row] = WindowHeight::Fixed(target.min(ceiling).max(MIN_TILE_HEIGHT));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloatingWindow {
    pub window: WindowId,
    pub col: u16,
    pub row: u16,
    pub width: Proportion,
    pub full_width: bool,
    pub rows: u16,
}

fn opening_rows(area: Size) -> u16 {
    area.rows
        .saturating_sub(2 * height_step(area))
        .max(MIN_TILE_HEIGHT)
}

impl FloatingWindow {
    fn opened(window: WindowId, width: Proportion, area: Size) -> Self {
        let mut record = Self {
            window,
            col: 0,
            row: 0,
            width,
            full_width: false,
            rows: opening_rows(area),
        };
        let placed = placed(&record, area);
        record.col = area.cols.saturating_sub(placed.width) / 2;
        record.row = area.rows.saturating_sub(placed.height) / 2;
        record
    }

    fn width(&self) -> (Proportion, bool) {
        (self.width, self.full_width)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Band {
    pub id: BandId,
    pub columns: Vec<Column>,
    pub floating: Vec<FloatingWindow>,
}

impl Band {
    fn new(id: BandId) -> Self {
        Self {
            id,
            columns: Vec::new(),
            floating: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty() && self.floating.is_empty()
    }

    pub fn windows(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.columns
            .iter()
            .flat_map(|column| column.windows.iter().copied())
            .chain(self.floating.iter().map(|floating| floating.window))
    }

    pub fn floating_index(&self, window: WindowId) -> Option<usize> {
        self.floating
            .iter()
            .position(|floating| floating.window == window)
    }

    pub fn holds(&self, window: WindowId) -> bool {
        self.locate(window).is_some() || self.floating_index(window).is_some()
    }

    pub fn locate(&self, window: WindowId) -> Option<(usize, usize)> {
        self.columns.iter().enumerate().find_map(|(index, column)| {
            column
                .windows
                .iter()
                .position(|&candidate| candidate == window)
                .map(|row| (index, row))
        })
    }

    pub fn first_window(&self) -> Option<WindowId> {
        self.columns.first().map(|column| column.windows[0])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vertical {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetPlace {
    ColumnLeft,
    ColumnRight,
    Above,
    Below,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowContent {
    Program(Option<Program>),
    Plugin { request: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionAction {
    OpenWindow {
        band: BandId,
        after: Option<WindowId>,
        width: Option<Proportion>,
        floating: bool,
        focus: bool,
        content: WindowContent,
    },
    CloseWindow(WindowId),
    ConsumeOrExpel {
        window: WindowId,
        direction: Direction,
    },
    CycleWidth(WindowId),
    ToggleFullWidth(WindowId),
    StepWidth {
        window: WindowId,
        step: Step,
        by: Proportion,
    },
    StepHeight {
        window: WindowId,
        step: Step,
        by: Proportion,
    },
    ResetHeight(WindowId),
    SetWidth {
        window: WindowId,
        width: Proportion,
    },
    SetHeight {
        window: WindowId,
        height: WindowHeight,
    },
    ToggleFloating {
        window: WindowId,
        after: Option<WindowId>,
        floating: Option<bool>,
    },
    MoveColumn {
        window: WindowId,
        direction: Direction,
    },
    MoveWindow {
        window: WindowId,
        direction: Vertical,
    },
    SetPosition {
        window: WindowId,
        col: u16,
        row: u16,
    },
    MoveToPlace {
        window: WindowId,
        reference: WindowId,
        place: TargetPlace,
    },
}

impl SessionAction {
    pub fn open(band: BandId, after: Option<WindowId>, program: Option<Program>) -> Self {
        Self::OpenWindow {
            band,
            after,
            width: None,
            floating: false,
            focus: true,
            content: WindowContent::Program(program),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Location {
    pub band: usize,
    pub column: usize,
    pub row: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Tiled(Location),
    Floating { band: usize, index: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    bands: Vec<Band>,
    boxes: BTreeMap<WindowId, FloatingWindow>,
    next_window: u32,
    next_band: u32,
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

impl Layout {
    pub fn new() -> Self {
        let mut layout = Self {
            bands: Vec::new(),
            boxes: BTreeMap::new(),
            next_window: 1,
            next_band: 1,
        };
        layout.normalize();
        layout
    }

    pub fn allocate_window(&mut self) -> WindowId {
        let id = WindowId(self.next_window);
        self.next_window += 1;
        id
    }

    pub fn bands(&self) -> &[Band] {
        &self.bands
    }

    pub fn band(&self, id: BandId) -> Option<&Band> {
        self.bands.iter().find(|band| band.id == id)
    }

    pub fn band_index(&self, id: BandId) -> Option<usize> {
        self.bands.iter().position(|band| band.id == id)
    }

    pub fn windows(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.bands.iter().flat_map(Band::windows)
    }

    pub fn is_empty(&self) -> bool {
        self.windows().next().is_none()
    }

    pub fn contains(&self, window: WindowId) -> bool {
        self.place(window).is_some()
    }

    pub fn place(&self, window: WindowId) -> Option<Place> {
        self.bands
            .iter()
            .enumerate()
            .find_map(|(band, candidate)| match candidate.locate(window) {
                Some((column, row)) => Some(Place::Tiled(Location { band, column, row })),
                None => candidate
                    .floating_index(window)
                    .map(|index| Place::Floating { band, index }),
            })
    }

    pub fn floating(&self, window: WindowId) -> Option<&FloatingWindow> {
        match self.place(window)? {
            Place::Floating { band, index } => Some(&self.bands[band].floating[index]),
            Place::Tiled(_) => None,
        }
    }

    pub fn locate(&self, window: WindowId) -> Option<Location> {
        self.bands.iter().enumerate().find_map(|(band, candidate)| {
            candidate
                .locate(window)
                .map(|(column, row)| Location { band, column, row })
        })
    }

    pub fn can_open(&self, band: BandId, after: Option<WindowId>) -> bool {
        match (self.band(band), after) {
            (None, _) => false,
            (Some(_), None) => true,
            (Some(target), Some(window)) => target.locate(window).is_some(),
        }
    }

    pub fn open(
        &mut self,
        window: WindowId,
        band: BandId,
        after: Option<WindowId>,
        width: Option<Proportion>,
        options: &LayoutOptions,
    ) -> Vec<LayoutEvent> {
        if !self.can_open(band, after) || self.contains(window) {
            return Vec::new();
        }
        let target = self
            .bands
            .iter_mut()
            .find(|candidate| candidate.id == band)
            .expect("checked by can_open");
        let index = match after.and_then(|after| target.locate(after)) {
            Some((column, _)) => column + 1,
            None => 0,
        };
        target.columns.insert(
            index,
            Column::new(
                window,
                width.map_or(options.default_width, Proportion::lowest),
            ),
        );
        let mut events = vec![LayoutEvent::WindowOpened { window, band }];
        events.extend(self.normalize());
        events
    }

    pub fn open_floating(
        &mut self,
        window: WindowId,
        band: BandId,
        width: Option<Proportion>,
        area: Size,
        options: &LayoutOptions,
    ) -> Vec<LayoutEvent> {
        if self.contains(window) {
            return Vec::new();
        }
        let Some(target) = self.bands.iter_mut().find(|candidate| candidate.id == band) else {
            return Vec::new();
        };
        let width = width.map_or(options.default_width, Proportion::lowest);
        let record = FloatingWindow::opened(window, width, area);
        target.floating.push(record);
        let mut events = vec![
            LayoutEvent::WindowOpened { window, band },
            LayoutEvent::WindowFloated {
                window,
                band,
                record,
            },
        ];
        events.extend(self.normalize());
        events
    }

    pub fn remove(&mut self, window: WindowId) -> Vec<LayoutEvent> {
        let Some(place) = self.place(window) else {
            return Vec::new();
        };
        self.boxes.remove(&window);
        let band = match place {
            Place::Tiled(location) => {
                let band = &mut self.bands[location.band];
                let column = &mut band.columns[location.column];
                column.remove(location.row);
                if column.windows.is_empty() {
                    band.columns.remove(location.column);
                }
                band
            }
            Place::Floating { band, index } => {
                let band = &mut self.bands[band];
                band.floating.remove(index);
                band
            }
        };
        let mut events = vec![LayoutEvent::WindowClosed {
            window,
            band: band.id,
        }];
        events.extend(self.normalize());
        events
    }

    pub fn apply(
        &mut self,
        action: SessionAction,
        area: Size,
        options: &LayoutOptions,
    ) -> Vec<LayoutEvent> {
        match action {
            SessionAction::OpenWindow { .. } | SessionAction::CloseWindow(_) => Vec::new(),
            SessionAction::ConsumeOrExpel { window, direction } => {
                self.consume_or_expel(window, direction, options)
            }
            SessionAction::CycleWidth(window) => {
                self.with_width(window, area, |width| cycle_width(width, &options.presets))
            }
            SessionAction::ToggleFullWidth(window) => {
                self.with_width(window, area, |(width, full_width)| (width, !full_width))
            }
            SessionAction::StepWidth { window, step, by } => {
                self.with_width(window, area, |width| step_width(width, step, by))
            }
            SessionAction::SetWidth { window, width } => {
                self.with_width(window, area, |_| set_width(width))
            }
            SessionAction::StepHeight { window, step, by } => self.with_height(
                window,
                area,
                |column, row| column.step_height(row, step, by, area),
                |height| {
                    Some(match step {
                        Step::Grow => height.saturating_add(by.rows_of(area.rows)),
                        Step::Shrink => height.saturating_sub(by.rows_of(area.rows)),
                    })
                },
            ),
            SessionAction::ResetHeight(window) => self.with_height(
                window,
                area,
                |column, row| column.heights[row] = WindowHeight::DEFAULT,
                |_| Some(opening_rows(area)),
            ),
            SessionAction::SetHeight { window, height } => self.with_height(
                window,
                area,
                |column, row| column.set_height(row, height, area),
                |_| match height {
                    WindowHeight::Fixed(rows) => Some(rows),
                    WindowHeight::Auto(_) => None,
                },
            ),
            SessionAction::ToggleFloating {
                window,
                after,
                floating,
            } => match (self.place(window), floating) {
                (Some(Place::Tiled(location)), None | Some(true)) => {
                    self.float(window, location, area, options)
                }
                (Some(Place::Floating { band, index }), None | Some(false)) => {
                    self.tile(band, index, after)
                }
                _ => Vec::new(),
            },
            SessionAction::MoveColumn { window, direction } => match self.place(window) {
                Some(Place::Tiled(location)) => self.move_column(location, direction),
                Some(Place::Floating { .. }) => self.with_box(window, area, |record, placed| {
                    let limit = area.cols - placed.width;
                    record.col = match direction {
                        Direction::Left => placed.x.saturating_sub(width_step(area)),
                        Direction::Right => placed.x.saturating_add(width_step(area)).min(limit),
                    };
                    record.row = placed.y;
                }),
                None => Vec::new(),
            },
            SessionAction::MoveWindow { window, direction } => match self.place(window) {
                Some(Place::Tiled(location)) => self.move_window(window, location, direction),
                Some(Place::Floating { .. }) => self.with_box(window, area, |record, placed| {
                    let limit = area.rows - placed.height;
                    record.row = match direction {
                        Vertical::Up => placed.y.saturating_sub(height_step(area)),
                        Vertical::Down => placed.y.saturating_add(height_step(area)).min(limit),
                    };
                    record.col = placed.x;
                }),
                None => Vec::new(),
            },
            SessionAction::SetPosition { window, col, row } => {
                self.with_box(window, area, |record, placed| {
                    record.col = col.min(area.cols - placed.width);
                    record.row = row.min(area.rows - placed.height);
                })
            }
            SessionAction::MoveToPlace {
                window,
                reference,
                place,
            } => self.move_to_place(window, reference, place),
        }
    }

    fn move_to_place(
        &mut self,
        window: WindowId,
        reference: WindowId,
        place: TargetPlace,
    ) -> Vec<LayoutEvent> {
        let (Some(from), Some(to)) = (self.locate(window), self.locate(reference)) else {
            return Vec::new();
        };
        if window == reference || from.band != to.band {
            return Vec::new();
        }
        let band = &mut self.bands[from.band];
        let before = band.columns.clone();
        let source = &mut band.columns[from.column];
        let (width, full_width) = source.width();
        let height = source.heights[from.row];
        source.remove(from.row);
        if source.windows.is_empty() {
            band.columns.remove(from.column);
        }
        let (column, row) = band.locate(reference).expect("reference stays in its band");
        let (column, row) = match place {
            TargetPlace::ColumnLeft | TargetPlace::ColumnRight => {
                let index = column + usize::from(place == TargetPlace::ColumnRight);
                let mut moved = Column::new(window, width);
                moved.full_width = full_width;
                band.columns.insert(index, moved);
                (index, 0)
            }
            TargetPlace::Above | TargetPlace::Below => {
                let index = row + usize::from(place == TargetPlace::Below);
                let height = if from.column == to.column {
                    height
                } else {
                    WindowHeight::DEFAULT
                };
                let target = &mut band.columns[column];
                target.windows.insert(index, window);
                target.heights.insert(index, height);
                (column, index)
            }
        };
        if band.columns == before {
            return Vec::new();
        }
        vec![LayoutEvent::WindowMoved {
            window,
            band: band.id,
            column,
            row,
        }]
    }

    fn float(
        &mut self,
        window: WindowId,
        location: Location,
        area: Size,
        options: &LayoutOptions,
    ) -> Vec<LayoutEvent> {
        let band = &mut self.bands[location.band];
        let column = &mut band.columns[location.column];
        let record = self
            .boxes
            .remove(&window)
            .unwrap_or_else(|| FloatingWindow::opened(window, options.default_width, area));

        column.remove(location.row);
        if column.windows.is_empty() {
            band.columns.remove(location.column);
        }
        band.floating.push(record);
        let mut events = vec![LayoutEvent::WindowFloated {
            window,
            band: band.id,
            record,
        }];
        events.extend(self.normalize());
        events
    }

    fn tile(&mut self, band: usize, index: usize, after: Option<WindowId>) -> Vec<LayoutEvent> {
        let band = &mut self.bands[band];
        let record = band.floating.remove(index);
        let column = after
            .and_then(|after| band.locate(after))
            .map_or(0, |(column, _)| column + 1);
        let mut tiled = Column::new(record.window, record.width);
        tiled.full_width = record.full_width;
        band.columns.insert(column, tiled);
        self.boxes.insert(record.window, record);
        vec![LayoutEvent::WindowTiled {
            window: record.window,
            band: band.id,
            column,
            width: record.width,
            full_width: record.full_width,
        }]
    }

    fn move_column(&mut self, location: Location, direction: Direction) -> Vec<LayoutEvent> {
        let band = &mut self.bands[location.band];
        let to = match direction {
            Direction::Left => location.column.checked_sub(1),
            Direction::Right => Some(location.column + 1).filter(|&to| to < band.columns.len()),
        };
        let Some(to) = to else {
            return Vec::new();
        };
        band.columns.swap(location.column, to);
        vec![LayoutEvent::ColumnMoved {
            band: band.id,
            from: location.column,
            to,
        }]
    }

    fn move_window(
        &mut self,
        window: WindowId,
        location: Location,
        direction: Vertical,
    ) -> Vec<LayoutEvent> {
        let band = &mut self.bands[location.band];
        let column = &mut band.columns[location.column];
        let to = match direction {
            Vertical::Up => location.row.checked_sub(1),
            Vertical::Down => Some(location.row + 1).filter(|&to| to < column.windows.len()),
        };
        let Some(to) = to else {
            return Vec::new();
        };
        column.windows.swap(location.row, to);
        column.heights.swap(location.row, to);
        vec![
            LayoutEvent::WindowMoved {
                window,
                band: band.id,
                column: location.column,
                row: to,
            },
            LayoutEvent::WindowMoved {
                window: column.windows[location.row],
                band: band.id,
                column: location.column,
                row: location.row,
            },
        ]
    }

    fn with_width(
        &mut self,
        window: WindowId,
        area: Size,
        change: impl FnOnce((Proportion, bool)) -> (Proportion, bool),
    ) -> Vec<LayoutEvent> {
        match self.place(window) {
            Some(Place::Tiled(_)) => self.with_column(window, |column| {
                (column.width, column.full_width) = change(column.width());
            }),
            Some(Place::Floating { .. }) => self.with_box(window, area, |record, _| {
                (record.width, record.full_width) = change(record.width());
            }),
            None => Vec::new(),
        }
    }

    fn with_height(
        &mut self,
        window: WindowId,
        area: Size,
        tiled: impl FnOnce(&mut Column, usize),
        floating: impl FnOnce(u16) -> Option<u16>,
    ) -> Vec<LayoutEvent> {
        match self.place(window) {
            Some(Place::Tiled(_)) => self.with_heights(window, tiled),
            Some(Place::Floating { .. }) => self.with_box(window, area, |record, placed| {
                if let Some(rows) = floating(placed.height) {
                    record.rows = rows.min(area.rows).max(MIN_TILE_HEIGHT);
                }
            }),
            None => Vec::new(),
        }
    }

    fn with_box(
        &mut self,
        window: WindowId,
        area: Size,
        change: impl FnOnce(&mut FloatingWindow, WindowBox),
    ) -> Vec<LayoutEvent> {
        let Some(Place::Floating { band, index }) = self.place(window) else {
            return Vec::new();
        };
        let band = &mut self.bands[band];
        let record = &mut band.floating[index];
        let before = *record;
        change(record, placed(&before, area));
        if *record == before {
            return Vec::new();
        }
        vec![LayoutEvent::FloatingBoxChanged {
            window,
            band: band.id,
            record: *record,
        }]
    }

    fn with_heights(
        &mut self,
        window: WindowId,
        change: impl FnOnce(&mut Column, usize),
    ) -> Vec<LayoutEvent> {
        let Some(location) = self.locate(window) else {
            return Vec::new();
        };
        let band = &mut self.bands[location.band];
        let column = &mut band.columns[location.column];
        let before = column.heights.clone();
        change(column, location.row);
        if column.heights == before {
            return Vec::new();
        }
        vec![LayoutEvent::WindowHeightsChanged {
            band: band.id,
            column: location.column,
            heights: column.heights.clone(),
        }]
    }

    fn with_column(
        &mut self,
        window: WindowId,
        change: impl FnOnce(&mut Column),
    ) -> Vec<LayoutEvent> {
        let Some(location) = self.locate(window) else {
            return Vec::new();
        };
        let band = &mut self.bands[location.band];
        let column = &mut band.columns[location.column];
        let before = (column.width, column.full_width);
        change(column);
        if (column.width, column.full_width) == before {
            return Vec::new();
        }
        vec![LayoutEvent::ColumnWidthChanged {
            band: band.id,
            column: location.column,
            width: column.width,
            full_width: column.full_width,
        }]
    }

    fn consume_or_expel(
        &mut self,
        window: WindowId,
        direction: Direction,
        options: &LayoutOptions,
    ) -> Vec<LayoutEvent> {
        let Some(location) = self.locate(window) else {
            return Vec::new();
        };
        let band = &mut self.bands[location.band];
        let columns = &mut band.columns;
        let (column, row) = if columns[location.column].windows.len() > 1 {
            columns[location.column].remove(location.row);
            let index = match direction {
                Direction::Left => location.column,
                Direction::Right => location.column + 1,
            };
            columns.insert(index, Column::new(window, options.default_width));
            (index, 0)
        } else {
            let target = match direction {
                Direction::Left => location.column.checked_sub(1),
                Direction::Right => {
                    Some(location.column + 1).filter(|&index| index < columns.len())
                }
            };
            let Some(target) = target else {
                return Vec::new();
            };
            columns.remove(location.column);
            let target = if target > location.column {
                target - 1
            } else {
                target
            };
            columns[target].push(window);
            (target, columns[target].windows.len() - 1)
        };
        vec![LayoutEvent::WindowMoved {
            window,
            band: band.id,
            column,
            row,
        }]
    }

    fn normalize(&mut self) -> Vec<LayoutEvent> {
        let last = self.bands.len().saturating_sub(1);
        let mut events = Vec::new();
        let mut index = 0;
        self.bands.retain(|band| {
            let keep = index == last || !band.is_empty();
            index += 1;
            if !keep {
                events.push(LayoutEvent::BandRemoved { band: band.id });
            }
            keep
        });
        if self.bands.last().is_none_or(|last| !last.is_empty()) {
            let id = BandId(self.next_band);
            self.next_band += 1;
            events.push(LayoutEvent::BandAdded {
                band: id,
                index: self.bands.len(),
            });
            self.bands.push(Band::new(id));
        }
        events
    }
}
