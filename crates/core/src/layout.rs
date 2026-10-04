use std::fmt;

use serde::{Deserialize, Serialize};

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
pub struct Proportion {
    pub num: u8,
    pub den: u8,
}

impl Proportion {
    pub const ONE_THIRD: Self = Self::new(1, 3);
    pub const ONE_HALF: Self = Self::new(1, 2);
    pub const TWO_THIRDS: Self = Self::new(2, 3);
    pub const WHOLE: Self = Self::new(1, 1);

    pub const fn new(num: u8, den: u8) -> Self {
        Self { num, den }
    }

    pub fn of(self, cells: u16) -> u16 {
        (u32::from(cells) * u32::from(self.num) / u32::from(self.den)) as u16
    }

    fn exceeds(self, other: Self) -> bool {
        u16::from(self.num) * u16::from(other.den) > u16::from(other.num) * u16::from(self.den)
    }
}

pub const WIDTH_PRESETS: [Proportion; 3] = [
    Proportion::ONE_THIRD,
    Proportion::ONE_HALF,
    Proportion::TWO_THIRDS,
];
pub const DEFAULT_WIDTH: Proportion = Proportion::ONE_HALF;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Column {
    pub panes: Vec<PaneId>,
    pub width: Proportion,
    pub full_width: bool,
}

impl Column {
    pub fn new(pane: PaneId) -> Self {
        Self {
            panes: vec![pane],
            width: DEFAULT_WIDTH,
            full_width: false,
        }
    }

    fn cycle_width(&mut self) {
        let current = if self.full_width {
            Proportion::WHOLE
        } else {
            self.width
        };
        self.full_width = false;
        self.width = WIDTH_PRESETS
            .into_iter()
            .find(|preset| preset.exceeds(current))
            .unwrap_or(WIDTH_PRESETS[0]);
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionAction {
    OpenPane {
        workspace: WorkspaceId,
        after: Option<PaneId>,
    },
    ClosePane(PaneId),
    ConsumeOrExpel {
        pane: PaneId,
        direction: Direction,
    },
    CycleWidth(PaneId),
    ToggleFullWidth(PaneId),
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

    pub fn open(&mut self, pane: PaneId, workspace: WorkspaceId, after: Option<PaneId>) -> bool {
        if !self.can_open(workspace, after) || self.contains(pane) {
            return false;
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
        target.columns.insert(index, Column::new(pane));
        self.normalize();
        true
    }

    pub fn remove(&mut self, pane: PaneId) -> bool {
        let Some(location) = self.locate(pane) else {
            return false;
        };
        let workspace = &mut self.workspaces[location.workspace];
        let column = &mut workspace.columns[location.column];
        column.panes.remove(location.row);
        if column.panes.is_empty() {
            workspace.columns.remove(location.column);
        }
        self.normalize();
        true
    }

    pub fn apply(&mut self, action: SessionAction) -> bool {
        match action {
            SessionAction::OpenPane { .. } | SessionAction::ClosePane(_) => false,
            SessionAction::ConsumeOrExpel { pane, direction } => {
                self.consume_or_expel(pane, direction)
            }
            SessionAction::CycleWidth(pane) => self.with_column(pane, Column::cycle_width),
            SessionAction::ToggleFullWidth(pane) => {
                self.with_column(pane, |column| column.full_width = !column.full_width)
            }
        }
    }

    fn with_column(&mut self, pane: PaneId, change: impl FnOnce(&mut Column)) -> bool {
        let Some(location) = self.locate(pane) else {
            return false;
        };
        change(&mut self.workspaces[location.workspace].columns[location.column]);
        true
    }

    fn consume_or_expel(&mut self, pane: PaneId, direction: Direction) -> bool {
        let Some(location) = self.locate(pane) else {
            return false;
        };
        let workspace = &mut self.workspaces[location.workspace];
        let columns = &mut workspace.columns;
        if columns[location.column].panes.len() > 1 {
            columns[location.column].panes.remove(location.row);
            let index = match direction {
                Direction::Left => location.column,
                Direction::Right => location.column + 1,
            };
            columns.insert(index, Column::new(pane));
            return true;
        }
        let target = match direction {
            Direction::Left => location.column.checked_sub(1),
            Direction::Right => Some(location.column + 1).filter(|&index| index < columns.len()),
        };
        let Some(target) = target else {
            return false;
        };
        columns.remove(location.column);
        let target = if target > location.column {
            target - 1
        } else {
            target
        };
        columns[target].panes.push(pane);
        true
    }

    fn normalize(&mut self) {
        let last = self.workspaces.len().saturating_sub(1);
        let mut index = 0;
        self.workspaces.retain(|workspace| {
            let keep = index == last || !workspace.is_empty();
            index += 1;
            keep
        });
        if self.workspaces.last().is_none_or(|last| !last.is_empty()) {
            let id = WorkspaceId(self.next_workspace);
            self.next_workspace += 1;
            self.workspaces.push(Workspace::new(id));
        }
    }
}
