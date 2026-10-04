use serde::{Deserialize, Serialize};

use crate::layout::{PaneId, Proportion, WorkspaceId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    WorkspaceAdded {
        workspace: WorkspaceId,
        index: usize,
    },
    WorkspaceRemoved {
        workspace: WorkspaceId,
    },
}
