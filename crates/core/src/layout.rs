use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::event::LayoutEvent;
use crate::geometry::{
    MIN_TILE_HEIGHT, PaneBox, Size, fixed_height_limit, height_step, pane_heights, placed,
    width_step,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PaneId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BandId(pub u32);

impl fmt::Display for PaneId {
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

    pub const MAX: u32 = 10000;
    const STEP_TENTHS: u64 = 10;

    pub const fn new(num: u32, den: u32) -> Self {
        Self { num, den }
    }

    pub fn of(self, cells: u16) -> u16 {
        let cells = u64::from(cells) * u64::from(self.num) / u64::from(self.den);
        cells.min(u64::from(u16::MAX)) as u16
    }

    pub fn step(self, step: Step) -> Self {
        let den = u64::from(self.den) * Self::STEP_TENTHS;
        let num = u64::from(self.num) * Self::STEP_TENTHS;
        let num = match step {
            Step::Grow => num + u64::from(self.den),
            Step::Shrink => num.saturating_sub(u64::from(self.den)),
        };
        let num = num.min(u64::from(Self::MAX) * den);
        let divisor = gcd(num, den);
        Self::new((num / divisor) as u32, (den / divisor) as u32)
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

pub fn step_width(current: (Proportion, bool), step: Step) -> (Proportion, bool) {
    (effective_width(current).step(step), false)
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
pub enum PaneHeight {
    Auto(Weight),
    Fixed(u16),
}

impl PaneHeight {
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
    pub panes: Vec<PaneId>,
    pub heights: Vec<PaneHeight>,
    pub width: Proportion,
    pub full_width: bool,
}

impl Column {
    pub fn new(pane: PaneId, width: Proportion) -> Self {
        Self {
            panes: vec![pane],
            heights: vec![PaneHeight::DEFAULT],
            width,
            full_width: false,
        }
    }

    fn push(&mut self, pane: PaneId) {
        self.panes.push(pane);
        self.heights.push(PaneHeight::DEFAULT);
    }

    fn remove(&mut self, row: usize) {
        self.panes.remove(row);
        self.heights.remove(row);
        if let [last @ PaneHeight::Auto(_)] = self.heights.as_mut_slice() {
            *last = PaneHeight::DEFAULT;
        }
    }

    fn width(&self) -> (Proportion, bool) {
        (self.width, self.full_width)
    }

    pub fn step_width(&mut self, step: Step) {
        (self.width, self.full_width) = step_width(self.width(), step);
    }

    pub fn set_width(&mut self, width: Proportion) {
        (self.width, self.full_width) = set_width(width);
    }

    fn step_height(&mut self, row: usize, step: Step, area: Size) {
        let rows = pane_heights(self, area.rows);
        let step_rows = height_step(area);
        let target = match step {
            Step::Grow => rows[row].saturating_add(step_rows),
            Step::Shrink => rows[row].saturating_sub(step_rows),
        };
        self.fix_height(row, target, &rows, area);
    }

    pub fn set_height(&mut self, row: usize, height: PaneHeight, area: Size) {
        match height {
            PaneHeight::Fixed(target) => {
                let rows = pane_heights(self, area.rows);
                self.fix_height(row, target, &rows, area);
            }
            PaneHeight::Auto(weight) => self.heights[row] = PaneHeight::Auto(weight),
        }
    }

    fn fix_height(&mut self, row: usize, target: u16, rows: &[u16], area: Size) {
        if matches!(self.heights[row], PaneHeight::Auto(_)) {
            let mut sorted = rows.to_vec();
            sorted.sort_unstable();
            let median = sorted[sorted.len() / 2].max(1);
            for (height, &tile) in self.heights.iter_mut().zip(rows) {
                *height = PaneHeight::Auto(Weight::new(u32::from(tile.max(1)), u32::from(median)));
            }
        }
        let ceiling = fixed_height_limit(self.panes.len(), area.rows);
        self.heights[row] = PaneHeight::Fixed(target.min(ceiling).max(MIN_TILE_HEIGHT));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloatingPane {
    pub pane: PaneId,
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

impl FloatingPane {
    fn opened(pane: PaneId, width: Proportion, area: Size) -> Self {
        let mut record = Self {
            pane,
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
    pub floating: Vec<FloatingPane>,
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

    pub fn panes(&self) -> impl Iterator<Item = PaneId> + '_ {
        self.columns
            .iter()
            .flat_map(|column| column.panes.iter().copied())
            .chain(self.floating.iter().map(|floating| floating.pane))
    }

    pub fn floating_index(&self, pane: PaneId) -> Option<usize> {
        self.floating
            .iter()
            .position(|floating| floating.pane == pane)
    }

    pub fn holds(&self, pane: PaneId) -> bool {
        self.locate(pane).is_some() || self.floating_index(pane).is_some()
    }

    pub fn locate(&self, pane: PaneId) -> Option<(usize, usize)> {
        self.columns.iter().enumerate().find_map(|(index, column)| {
            column
                .panes
                .iter()
                .position(|&candidate| candidate == pane)
                .map(|row| (index, row))
        })
    }

    pub fn first_pane(&self) -> Option<PaneId> {
        self.columns.first().map(|column| column.panes[0])
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaneContent {
    Program(Option<Program>),
    Plugin { request: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionAction {
    OpenPane {
        band: BandId,
        after: Option<PaneId>,
        width: Option<Proportion>,
        floating: bool,
        focus: bool,
        content: PaneContent,
    },
    ClosePane(PaneId),
    ConsumeOrExpel {
        pane: PaneId,
        direction: Direction,
    },
    CycleWidth(PaneId),
    ToggleFullWidth(PaneId),
    StepWidth {
        pane: PaneId,
        step: Step,
    },
    StepHeight {
        pane: PaneId,
        step: Step,
    },
    ResetHeight(PaneId),
    SetWidth {
        pane: PaneId,
        width: Proportion,
    },
    SetHeight {
        pane: PaneId,
        height: PaneHeight,
    },
    ToggleFloating {
        pane: PaneId,
        after: Option<PaneId>,
    },
    MoveColumn {
        pane: PaneId,
        direction: Direction,
    },
    MovePane {
        pane: PaneId,
        direction: Vertical,
    },
    SetPosition {
        pane: PaneId,
        col: u16,
        row: u16,
    },
}

impl SessionAction {
    pub fn open(band: BandId, after: Option<PaneId>, program: Option<Program>) -> Self {
        Self::OpenPane {
            band,
            after,
            width: None,
            floating: false,
            focus: true,
            content: PaneContent::Program(program),
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
    boxes: BTreeMap<PaneId, FloatingPane>,
    next_pane: u32,
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
            next_pane: 1,
            next_band: 1,
        };
        layout.normalize();
        layout
    }

    pub fn allocate_pane(&mut self) -> PaneId {
        let id = PaneId(self.next_pane);
        self.next_pane += 1;
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

    pub fn panes(&self) -> impl Iterator<Item = PaneId> + '_ {
        self.bands.iter().flat_map(Band::panes)
    }

    pub fn is_empty(&self) -> bool {
        self.panes().next().is_none()
    }

    pub fn contains(&self, pane: PaneId) -> bool {
        self.place(pane).is_some()
    }

    pub fn place(&self, pane: PaneId) -> Option<Place> {
        self.bands
            .iter()
            .enumerate()
            .find_map(|(band, candidate)| match candidate.locate(pane) {
                Some((column, row)) => Some(Place::Tiled(Location { band, column, row })),
                None => candidate
                    .floating_index(pane)
                    .map(|index| Place::Floating { band, index }),
            })
    }

    pub fn floating(&self, pane: PaneId) -> Option<&FloatingPane> {
        match self.place(pane)? {
            Place::Floating { band, index } => Some(&self.bands[band].floating[index]),
            Place::Tiled(_) => None,
        }
    }

    pub fn locate(&self, pane: PaneId) -> Option<Location> {
        self.bands.iter().enumerate().find_map(|(band, candidate)| {
            candidate
                .locate(pane)
                .map(|(column, row)| Location { band, column, row })
        })
    }

    pub fn can_open(&self, band: BandId, after: Option<PaneId>) -> bool {
        match (self.band(band), after) {
            (None, _) => false,
            (Some(_), None) => true,
            (Some(target), Some(pane)) => target.locate(pane).is_some(),
        }
    }

    pub fn open(
        &mut self,
        pane: PaneId,
        band: BandId,
        after: Option<PaneId>,
        width: Option<Proportion>,
        options: &LayoutOptions,
    ) -> Vec<LayoutEvent> {
        if !self.can_open(band, after) || self.contains(pane) {
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
                pane,
                width.map_or(options.default_width, Proportion::lowest),
            ),
        );
        let mut events = vec![LayoutEvent::PaneOpened { pane, band }];
        events.extend(self.normalize());
        events
    }

    pub fn open_floating(
        &mut self,
        pane: PaneId,
        band: BandId,
        width: Option<Proportion>,
        area: Size,
        options: &LayoutOptions,
    ) -> Vec<LayoutEvent> {
        if self.contains(pane) {
            return Vec::new();
        }
        let Some(target) = self.bands.iter_mut().find(|candidate| candidate.id == band) else {
            return Vec::new();
        };
        let width = width.map_or(options.default_width, Proportion::lowest);
        let record = FloatingPane::opened(pane, width, area);
        target.floating.push(record);
        let mut events = vec![
            LayoutEvent::PaneOpened { pane, band },
            LayoutEvent::PaneFloated { pane, band, record },
        ];
        events.extend(self.normalize());
        events
    }

    pub fn remove(&mut self, pane: PaneId) -> Vec<LayoutEvent> {
        let Some(place) = self.place(pane) else {
            return Vec::new();
        };
        self.boxes.remove(&pane);
        let band = match place {
            Place::Tiled(location) => {
                let band = &mut self.bands[location.band];
                let column = &mut band.columns[location.column];
                column.remove(location.row);
                if column.panes.is_empty() {
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
        let mut events = vec![LayoutEvent::PaneClosed {
            pane,
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
            SessionAction::OpenPane { .. } | SessionAction::ClosePane(_) => Vec::new(),
            SessionAction::ConsumeOrExpel { pane, direction } => {
                self.consume_or_expel(pane, direction, options)
            }
            SessionAction::CycleWidth(pane) => {
                self.with_width(pane, area, |width| cycle_width(width, &options.presets))
            }
            SessionAction::ToggleFullWidth(pane) => {
                self.with_width(pane, area, |(width, full_width)| (width, !full_width))
            }
            SessionAction::StepWidth { pane, step } => {
                self.with_width(pane, area, |width| step_width(width, step))
            }
            SessionAction::SetWidth { pane, width } => {
                self.with_width(pane, area, |_| set_width(width))
            }
            SessionAction::StepHeight { pane, step } => self.with_height(
                pane,
                area,
                |column, row| column.step_height(row, step, area),
                |height| {
                    Some(match step {
                        Step::Grow => height.saturating_add(height_step(area)),
                        Step::Shrink => height.saturating_sub(height_step(area)),
                    })
                },
            ),
            SessionAction::ResetHeight(pane) => self.with_height(
                pane,
                area,
                |column, row| column.heights[row] = PaneHeight::DEFAULT,
                |_| Some(opening_rows(area)),
            ),
            SessionAction::SetHeight { pane, height } => self.with_height(
                pane,
                area,
                |column, row| column.set_height(row, height, area),
                |_| match height {
                    PaneHeight::Fixed(rows) => Some(rows),
                    PaneHeight::Auto(_) => None,
                },
            ),
            SessionAction::ToggleFloating { pane, after } => match self.place(pane) {
                Some(Place::Tiled(location)) => self.float(pane, location, area, options),
                Some(Place::Floating { band, index }) => self.tile(band, index, after),
                None => Vec::new(),
            },
            SessionAction::MoveColumn { pane, direction } => match self.place(pane) {
                Some(Place::Tiled(location)) => self.move_column(location, direction),
                Some(Place::Floating { .. }) => self.with_box(pane, area, |record, placed| {
                    let limit = area.cols - placed.width;
                    record.col = match direction {
                        Direction::Left => placed.x.saturating_sub(width_step(area)),
                        Direction::Right => placed.x.saturating_add(width_step(area)).min(limit),
                    };
                    record.row = placed.y;
                }),
                None => Vec::new(),
            },
            SessionAction::MovePane { pane, direction } => match self.place(pane) {
                Some(Place::Tiled(location)) => self.move_pane(pane, location, direction),
                Some(Place::Floating { .. }) => self.with_box(pane, area, |record, placed| {
                    let limit = area.rows - placed.height;
                    record.row = match direction {
                        Vertical::Up => placed.y.saturating_sub(height_step(area)),
                        Vertical::Down => placed.y.saturating_add(height_step(area)).min(limit),
                    };
                    record.col = placed.x;
                }),
                None => Vec::new(),
            },
            SessionAction::SetPosition { pane, col, row } => {
                self.with_box(pane, area, |record, placed| {
                    record.col = col.min(area.cols - placed.width);
                    record.row = row.min(area.rows - placed.height);
                })
            }
        }
    }

    fn float(
        &mut self,
        pane: PaneId,
        location: Location,
        area: Size,
        options: &LayoutOptions,
    ) -> Vec<LayoutEvent> {
        let band = &mut self.bands[location.band];
        let column = &mut band.columns[location.column];
        let record = self
            .boxes
            .remove(&pane)
            .unwrap_or_else(|| FloatingPane::opened(pane, options.default_width, area));

        column.remove(location.row);
        if column.panes.is_empty() {
            band.columns.remove(location.column);
        }
        band.floating.push(record);
        let mut events = vec![LayoutEvent::PaneFloated {
            pane,
            band: band.id,
            record,
        }];
        events.extend(self.normalize());
        events
    }

    fn tile(&mut self, band: usize, index: usize, after: Option<PaneId>) -> Vec<LayoutEvent> {
        let band = &mut self.bands[band];
        let record = band.floating.remove(index);
        let column = after
            .and_then(|after| band.locate(after))
            .map_or(0, |(column, _)| column + 1);
        let mut tiled = Column::new(record.pane, record.width);
        tiled.full_width = record.full_width;
        band.columns.insert(column, tiled);
        self.boxes.insert(record.pane, record);
        vec![LayoutEvent::PaneTiled {
            pane: record.pane,
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

    fn move_pane(
        &mut self,
        pane: PaneId,
        location: Location,
        direction: Vertical,
    ) -> Vec<LayoutEvent> {
        let band = &mut self.bands[location.band];
        let column = &mut band.columns[location.column];
        let to = match direction {
            Vertical::Up => location.row.checked_sub(1),
            Vertical::Down => Some(location.row + 1).filter(|&to| to < column.panes.len()),
        };
        let Some(to) = to else {
            return Vec::new();
        };
        column.panes.swap(location.row, to);
        column.heights.swap(location.row, to);
        vec![
            LayoutEvent::PaneMoved {
                pane,
                band: band.id,
                column: location.column,
                row: to,
            },
            LayoutEvent::PaneMoved {
                pane: column.panes[location.row],
                band: band.id,
                column: location.column,
                row: location.row,
            },
        ]
    }

    fn with_width(
        &mut self,
        pane: PaneId,
        area: Size,
        change: impl FnOnce((Proportion, bool)) -> (Proportion, bool),
    ) -> Vec<LayoutEvent> {
        match self.place(pane) {
            Some(Place::Tiled(_)) => self.with_column(pane, |column| {
                (column.width, column.full_width) = change(column.width());
            }),
            Some(Place::Floating { .. }) => self.with_box(pane, area, |record, _| {
                (record.width, record.full_width) = change(record.width());
            }),
            None => Vec::new(),
        }
    }

    fn with_height(
        &mut self,
        pane: PaneId,
        area: Size,
        tiled: impl FnOnce(&mut Column, usize),
        floating: impl FnOnce(u16) -> Option<u16>,
    ) -> Vec<LayoutEvent> {
        match self.place(pane) {
            Some(Place::Tiled(_)) => self.with_heights(pane, tiled),
            Some(Place::Floating { .. }) => self.with_box(pane, area, |record, placed| {
                if let Some(rows) = floating(placed.height) {
                    record.rows = rows.min(area.rows).max(MIN_TILE_HEIGHT);
                }
            }),
            None => Vec::new(),
        }
    }

    fn with_box(
        &mut self,
        pane: PaneId,
        area: Size,
        change: impl FnOnce(&mut FloatingPane, PaneBox),
    ) -> Vec<LayoutEvent> {
        let Some(Place::Floating { band, index }) = self.place(pane) else {
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
            pane,
            band: band.id,
            record: *record,
        }]
    }

    fn with_heights(
        &mut self,
        pane: PaneId,
        change: impl FnOnce(&mut Column, usize),
    ) -> Vec<LayoutEvent> {
        let Some(location) = self.locate(pane) else {
            return Vec::new();
        };
        let band = &mut self.bands[location.band];
        let column = &mut band.columns[location.column];
        let before = column.heights.clone();
        change(column, location.row);
        if column.heights == before {
            return Vec::new();
        }
        vec![LayoutEvent::PaneHeightsChanged {
            band: band.id,
            column: location.column,
            heights: column.heights.clone(),
        }]
    }

    fn with_column(&mut self, pane: PaneId, change: impl FnOnce(&mut Column)) -> Vec<LayoutEvent> {
        let Some(location) = self.locate(pane) else {
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
        pane: PaneId,
        direction: Direction,
        options: &LayoutOptions,
    ) -> Vec<LayoutEvent> {
        let Some(location) = self.locate(pane) else {
            return Vec::new();
        };
        let band = &mut self.bands[location.band];
        let columns = &mut band.columns;
        let (column, row) = if columns[location.column].panes.len() > 1 {
            columns[location.column].remove(location.row);
            let index = match direction {
                Direction::Left => location.column,
                Direction::Right => location.column + 1,
            };
            columns.insert(index, Column::new(pane, options.default_width));
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
            columns[target].push(pane);
            (target, columns[target].panes.len() - 1)
        };
        vec![LayoutEvent::PaneMoved {
            pane,
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
