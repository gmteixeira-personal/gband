use serde::{Deserialize, Serialize};

use crate::layout::{BandId, FloatingWindow, Proportion, WindowHeight, WindowId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutEvent {
    WindowOpened {
        window: WindowId,
        band: BandId,
    },
    WindowClosed {
        window: WindowId,
        band: BandId,
    },
    WindowMoved {
        window: WindowId,
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
    WindowHeightsChanged {
        band: BandId,
        column: usize,
        heights: Vec<WindowHeight>,
    },
    ColumnMoved {
        band: BandId,
        from: usize,
        to: usize,
    },
    WindowFloated {
        window: WindowId,
        band: BandId,
        record: FloatingWindow,
    },
    WindowTiled {
        window: WindowId,
        band: BandId,
        column: usize,
        width: Proportion,
        full_width: bool,
    },
    FloatingBoxChanged {
        window: WindowId,
        band: BandId,
        record: FloatingWindow,
    },
    BandAdded {
        band: BandId,
        index: usize,
    },
    BandRemoved {
        band: BandId,
    },
}
