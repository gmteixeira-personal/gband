use serde::{Deserialize, Serialize};

use crate::layout::{BandId, FloatingPane, PaneHeight, PaneId, Proportion};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutEvent {
    PaneOpened {
        pane: PaneId,
        band: BandId,
    },
    PaneClosed {
        pane: PaneId,
        band: BandId,
    },
    PaneMoved {
        pane: PaneId,
        band: BandId,
        column: usize,
        row: usize,
    },
    ColumnWidthChanged {
        band: BandId,
        column: usize,
        width: Proportion,
        full_width: bool,
    },
    PaneHeightsChanged {
        band: BandId,
        column: usize,
        heights: Vec<PaneHeight>,
    },
    ColumnMoved {
        band: BandId,
        from: usize,
        to: usize,
    },
    PaneFloated {
        pane: PaneId,
        band: BandId,
        record: FloatingPane,
    },
    PaneTiled {
        pane: PaneId,
        band: BandId,
        column: usize,
        width: Proportion,
        full_width: bool,
    },
    FloatingBoxChanged {
        pane: PaneId,
        band: BandId,
        record: FloatingPane,
    },
    BandAdded {
        band: BandId,
        index: usize,
    },
    BandRemoved {
        band: BandId,
    },
}
