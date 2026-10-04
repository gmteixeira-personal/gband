use serde::{Deserialize, Serialize};

use crate::layout::{Column, PaneId, Workspace};

pub const MIN_COLUMN_WIDTH: u16 = 3;
pub const BORDER: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub cols: u16,
    pub rows: u16,
}

impl Size {
    pub const fn new(cols: u16, rows: u16) -> Self {
        Self { cols, rows }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub x: u32,
    pub width: u16,
}

impl Span {
    pub fn end(&self) -> u32 {
        self.x + u32::from(self.width)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    pub pane: PaneId,
    pub column: usize,
    pub row: usize,
    pub x: u32,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Tile {
    pub fn span(&self) -> Span {
        Span {
            x: self.x,
            width: self.width,
        }
    }

    pub fn terminal_size(&self) -> Size {
        Size {
            cols: self.width.saturating_sub(2 * BORDER).max(1),
            rows: self.height.saturating_sub(2 * BORDER).max(1),
        }
    }
}

pub fn column_width(column: &Column, area: Size) -> u16 {
    let width = if column.full_width {
        area.cols
    } else {
        column.width.of(area.cols)
    };
    width.max(MIN_COLUMN_WIDTH)
}

pub fn column_spans(workspace: &Workspace, area: Size) -> Vec<Span> {
    let mut x = 0;
    workspace
        .columns
        .iter()
        .map(|column| {
            let span = Span {
                x,
                width: column_width(column, area),
            };
            x = span.end();
            span
        })
        .collect()
}

pub fn tiles(workspace: &Workspace, area: Size) -> Vec<Tile> {
    workspace
        .columns
        .iter()
        .zip(column_spans(workspace, area))
        .enumerate()
        .flat_map(|(column_index, (column, span))| {
            let count = column.panes.len() as u16;
            let base = area.rows / count;
            let extra = area.rows % count;
            let mut y = 0;
            column
                .panes
                .iter()
                .enumerate()
                .map(move |(row, &pane)| {
                    let height = base + u16::from((row as u16) < extra);
                    let tile = Tile {
                        pane,
                        column: column_index,
                        row,
                        x: span.x,
                        y,
                        width: span.width,
                        height,
                    };
                    y += height;
                    tile
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
