use serde::{Deserialize, Serialize};

use crate::layout::{PaneHeight, PaneId, Proportion, WorkspaceId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutEvent {
    PaneOpened {
        pane: PaneId,
        workspace: WorkspaceId,
    },
    PaneClosed {
        pane: PaneId,
        workspace: WorkspaceId,
    },
    PaneMoved {
        pane: PaneId,
        workspace: WorkspaceId,
        column: usize,
        row: usize,
    },
    ColumnWidthChanged {
        workspace: WorkspaceId,
        column: usize,
        width: Proportion,
        full_width: bool,
    },
    PaneHeightsChanged {
        workspace: WorkspaceId,
        column: usize,
        heights: Vec<PaneHeight>,
    },
    WorkspaceAdded {
        workspace: WorkspaceId,
        index: usize,
    },
    WorkspaceRemoved {
        workspace: WorkspaceId,
    },
}
