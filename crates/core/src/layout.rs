use std::fmt;

use serde::{Deserialize, Serialize};

use crate::event::LayoutEvent;
use crate::geometry::{MIN_TILE_HEIGHT, Size, fixed_height_limit, pane_heights};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PaneId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WorkspaceId(pub u32);

impl fmt::Display for PaneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for WorkspaceId {
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

    fn exceeds(self, other: Self) -> bool {
        u64::from(self.num) * u64::from(other.den) > u64::from(other.num) * u64::from(self.den)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weight {
    num: u16,
    den: u16,
}

impl Weight {
    pub const ONE: Self = Self { num: 1, den: 1 };

    pub fn new(num: u16, den: u16) -> Self {
        let divisor = gcd(u64::from(num), u64::from(den)) as u16;
        Self {
            num: num / divisor,
            den: den / divisor,
        }
    }

    pub fn num(self) -> u16 {
        self.num
    }

    pub fn den(self) -> u16 {
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

    fn step_height(&mut self, row: usize, step: Step, area: Size) {
        let rows = pane_heights(self, area.rows);
        if matches!(self.heights[row], PaneHeight::Auto(_)) {
            let mut sorted = rows.clone();
            sorted.sort_unstable();
            let median = sorted[sorted.len() / 2].max(1);
            for (height, &tile) in self.heights.iter_mut().zip(&rows) {
                *height = PaneHeight::Auto(Weight::new(tile.max(1), median));
            }
        }
        let ceiling = fixed_height_limit(self.panes.len(), area.rows);
        let step_rows = ((u32::from(area.rows) + 5) / 10).max(1) as u16;
        let target = match step {
            Step::Grow => rows[row].saturating_add(step_rows),
            Step::Shrink => rows[row].saturating_sub(step_rows),
        };
        self.heights[row] = PaneHeight::Fixed(target.min(ceiling).max(MIN_TILE_HEIGHT));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub columns: Vec<Column>,
}

impl Workspace {
    fn new(id: WorkspaceId) -> Self {
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
pub enum SessionAction {
    OpenPane {
        workspace: WorkspaceId,
        after: Option<PaneId>,
        program: Option<Program>,
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
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Location {
    pub workspace: usize,
    pub column: usize,
    pub row: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    workspaces: Vec<Workspace>,
    next_pane: u32,
    next_workspace: u32,
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

impl Layout {
    pub fn new() -> Self {
        let mut layout = Self {
            workspaces: Vec::new(),
            next_pane: 1,
            next_workspace: 1,
        };
        layout.normalize();
        layout
    }

    pub fn allocate_pane(&mut self) -> PaneId {
        let id = PaneId(self.next_pane);
        self.next_pane += 1;
        id
    }

    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }

    pub fn workspace(&self, id: WorkspaceId) -> Option<&Workspace> {
        self.workspaces.iter().find(|workspace| workspace.id == id)
    }

    pub fn workspace_index(&self, id: WorkspaceId) -> Option<usize> {
        self.workspaces
            .iter()
            .position(|workspace| workspace.id == id)
    }

    pub fn panes(&self) -> impl Iterator<Item = PaneId> + '_ {
        self.workspaces.iter().flat_map(Workspace::panes)
    }

    pub fn is_empty(&self) -> bool {
        self.panes().next().is_none()
    }

    pub fn contains(&self, pane: PaneId) -> bool {
        self.locate(pane).is_some()
    }

    pub fn locate(&self, pane: PaneId) -> Option<Location> {
        self.workspaces
            .iter()
            .enumerate()
            .find_map(|(workspace, candidate)| {
                candidate.locate(pane).map(|(column, row)| Location {
                    workspace,
                    column,
                    row,
                })
            })
    }

    pub fn can_open(&self, workspace: WorkspaceId, after: Option<PaneId>) -> bool {
        match (self.workspace(workspace), after) {
            (None, _) => false,
            (Some(_), None) => true,
            (Some(target), Some(pane)) => target.locate(pane).is_some(),
        }
    }

    pub fn open(
        &mut self,
        pane: PaneId,
        workspace: WorkspaceId,
        after: Option<PaneId>,
        options: &LayoutOptions,
    ) -> Vec<LayoutEvent> {
        if !self.can_open(workspace, after) || self.contains(pane) {
            return Vec::new();
        }
        let target = self
            .workspaces
            .iter_mut()
            .find(|candidate| candidate.id == workspace)
            .expect("checked by can_open");
        let index = match after.and_then(|after| target.locate(after)) {
            Some((column, _)) => column + 1,
            None => 0,
        };
        target
            .columns
            .insert(index, Column::new(pane, options.default_width));
        let mut events = vec![LayoutEvent::PaneOpened { pane, workspace }];
        events.extend(self.normalize());
        events
    }

    pub fn remove(&mut self, pane: PaneId) -> Vec<LayoutEvent> {
        let Some(location) = self.locate(pane) else {
            return Vec::new();
        };
        let workspace = &mut self.workspaces[location.workspace];
        let column = &mut workspace.columns[location.column];
        column.remove(location.row);
        if column.panes.is_empty() {
            workspace.columns.remove(location.column);
        }
        let mut events = vec![LayoutEvent::PaneClosed {
            pane,
            workspace: workspace.id,
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
        let workspace = &mut self.workspaces[location.workspace];
        let column = &mut workspace.columns[location.column];
        let before = column.heights.clone();
        change(column, location.row);
        if column.heights == before {
            return Vec::new();
        }
        vec![LayoutEvent::PaneHeightsChanged {
            workspace: workspace.id,
            column: location.column,
            heights: column.heights.clone(),
        }]
    }

    fn with_column(&mut self, pane: PaneId, change: impl FnOnce(&mut Column)) -> Vec<LayoutEvent> {
        let Some(location) = self.locate(pane) else {
            return Vec::new();
        };
        let workspace = &mut self.workspaces[location.workspace];
        let column = &mut workspace.columns[location.column];
        let before = (column.width, column.full_width);
        change(column);
        if (column.width, column.full_width) == before {
            return Vec::new();
        }
        vec![LayoutEvent::ColumnWidthChanged {
            workspace: workspace.id,
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
        let workspace = &mut self.workspaces[location.workspace];
        let columns = &mut workspace.columns;
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
            workspace: workspace.id,
            column,
            row,
        }]
    }

    fn normalize(&mut self) -> Vec<LayoutEvent> {
        let last = self.workspaces.len().saturating_sub(1);
        let mut events = Vec::new();
        let mut index = 0;
        self.workspaces.retain(|workspace| {
            let keep = index == last || !workspace.is_empty();
            index += 1;
            if !keep {
                events.push(LayoutEvent::WorkspaceRemoved {
                    workspace: workspace.id,
                });
            }
            keep
        });
        if self.workspaces.last().is_none_or(|last| !last.is_empty()) {
            let id = WorkspaceId(self.next_workspace);
            self.next_workspace += 1;
            events.push(LayoutEvent::WorkspaceAdded {
                workspace: id,
                index: self.workspaces.len(),
            });
            self.workspaces.push(Workspace::new(id));
        }
        events
    }
}
