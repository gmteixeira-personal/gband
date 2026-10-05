use std::fmt;

use serde::{Deserialize, Serialize};

use crate::event::LayoutEvent;
use crate::geometry::{MIN_TILE_HEIGHT, Size, fixed_height_limit, pane_heights};

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

    fn effective_width(&self) -> Proportion {
        if self.full_width {
            Proportion::WHOLE
        } else {
            self.width
        }
    }

    fn cycle_width(&mut self, presets: &[Proportion]) {
        let current = self.effective_width();
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
            .filter(|preset| preset.exceeds(current))
            .reduce(|nearest, preset| {
                if nearest.exceeds(preset) {
                    preset
                } else {
                    nearest
                }
            });
        let Some(width) = larger.or(smallest) else {
            return;
        };
        self.full_width = false;
        self.width = width;
    }

    pub fn step_width(&mut self, step: Step) {
        let current = self.effective_width();
        self.full_width = false;
        self.width = current.step(step);
    }

    pub fn set_width(&mut self, width: Proportion) {
        self.full_width = false;
        self.width = width.lowest();
    }

    fn step_height(&mut self, row: usize, step: Step, area: Size) {
        let rows = pane_heights(self, area.rows);
        let step_rows = ((u32::from(area.rows) + 5) / 10).max(1) as u16;
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Band {
    pub id: BandId,
    pub columns: Vec<Column>,
}

impl Band {
    fn new(id: BandId) -> Self {
        Self {
            id,
            columns: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    pub fn panes(&self) -> impl Iterator<Item = PaneId> + '_ {
        self.columns
            .iter()
            .flat_map(|column| column.panes.iter().copied())
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
}

impl SessionAction {
    pub fn open(band: BandId, after: Option<PaneId>, program: Option<Program>) -> Self {
        Self::OpenPane {
            band,
            after,
            width: None,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    bands: Vec<Band>,
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
        self.locate(pane).is_some()
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

    pub fn remove(&mut self, pane: PaneId) -> Vec<LayoutEvent> {
        let Some(location) = self.locate(pane) else {
            return Vec::new();
        };
        let band = &mut self.bands[location.band];
        let column = &mut band.columns[location.column];
        column.remove(location.row);
        if column.panes.is_empty() {
            band.columns.remove(location.column);
        }
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
                self.with_column(pane, |column| column.cycle_width(&options.presets))
            }
            SessionAction::ToggleFullWidth(pane) => {
                self.with_column(pane, |column| column.full_width = !column.full_width)
            }
            SessionAction::StepWidth { pane, step } => {
                self.with_column(pane, |column| column.step_width(step))
            }
            SessionAction::StepHeight { pane, step } => {
                self.with_heights(pane, |column, row| column.step_height(row, step, area))
            }
            SessionAction::ResetHeight(pane) => self.with_heights(pane, |column, row| {
                column.heights[row] = PaneHeight::DEFAULT;
            }),
            SessionAction::SetWidth { pane, width } => {
                self.with_column(pane, |column| column.set_width(width))
            }
            SessionAction::SetHeight { pane, height } => {
                self.with_heights(pane, |column, row| column.set_height(row, height, area))
            }
        }
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
