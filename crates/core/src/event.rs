use serde::{Deserialize, Serialize};

use crate::layout::{BandId, PaneHeight, PaneId, Proportion};

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
    BandAdded {
        band: BandId,
        index: usize,
    },
    BandRemoved {
        band: BandId,
    },
}
