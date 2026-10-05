use serde::{Deserialize, Serialize};

use crate::layout::{Column, PaneHeight, PaneId, Weight, Workspace, gcd};

pub const MIN_COLUMN_WIDTH: u16 = 3;
pub const MIN_TILE_HEIGHT: u16 = 3;
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

fn share(rows: u16, weights: &[u64]) -> Vec<u16> {
    let total: u64 = weights.iter().sum();
    if total == 0 {
        return vec![0; weights.len()];
    }
    let mut shares: Vec<u16> = weights
        .iter()
        .map(|&weight| (u64::from(rows) * weight / total) as u16)
        .collect();
    let leftover = rows - shares.iter().sum::<u16>();
    for share in shares.iter_mut().take(usize::from(leftover)) {
        *share += 1;
    }
    shares
}

fn scaled(weights: &[Weight]) -> Vec<u64> {
    let common = weights.iter().fold(1u64, |lcm, weight| {
        let den = u64::from(weight.den());
        lcm / gcd(lcm, den) * den
    });
    weights
        .iter()
        .map(|weight| u64::from(weight.num()) * (common / u64::from(weight.den())))
        .collect()
}

fn share_automatic(rows: u16, weights: &[Weight]) -> Vec<u16> {
    let weights = scaled(weights);
    let count = weights.len() as u32;
    let mut raised = vec![false; weights.len()];
    loop {
        let open: Vec<usize> = (0..weights.len()).filter(|&index| !raised[index]).collect();
        let raised_rows = (weights.len() - open.len()) as u16 * MIN_TILE_HEIGHT;
        let open_weights: Vec<u64> = open.iter().map(|&index| weights[index]).collect();
        let shares = share(rows - raised_rows, &open_weights);
        let short = open
            .iter()
            .zip(&shares)
            .find(|&(_, &share)| share < MIN_TILE_HEIGHT);
        match short {
            Some((&index, _)) if u32::from(rows) >= count * u32::from(MIN_TILE_HEIGHT) => {
                raised[index] = true;
            }
            _ => {
                let mut result = vec![MIN_TILE_HEIGHT; weights.len()];
                for (&index, share) in open.iter().zip(shares) {
                    result[index] = share;
                }
                return result;
            }
        }
    }
}

pub fn fixed_height_limit(panes: usize, rows: u16) -> u16 {
    let others = panes.saturating_sub(1) as u16;
    rows.saturating_sub(MIN_TILE_HEIGHT.saturating_mul(others))
}

pub fn pane_heights(column: &Column, rows: u16) -> Vec<u16> {
    let ceiling = fixed_height_limit(column.heights.len(), rows);
    let mut heights: Vec<u16> = column
        .heights
        .iter()
        .map(|height| match *height {
            PaneHeight::Fixed(fixed) => fixed.min(ceiling),
            PaneHeight::Auto(_) => 0,
        })
        .collect();
    let remaining = rows.saturating_sub(heights.iter().sum());
    let automatic: Vec<(usize, Weight)> = column
        .heights
        .iter()
        .enumerate()
        .filter_map(|(index, height)| match *height {
            PaneHeight::Auto(weight) => Some((index, weight)),
            PaneHeight::Fixed(_) => None,
        })
        .collect();
    let weights: Vec<Weight> = automatic.iter().map(|&(_, weight)| weight).collect();
    for (&(index, _), share) in automatic.iter().zip(share_automatic(remaining, &weights)) {
        heights[index] = share;
    }
    heights
}

pub fn tiles(workspace: &Workspace, area: Size) -> Vec<Tile> {
    workspace
        .columns
        .iter()
        .zip(column_spans(workspace, area))
        .enumerate()
        .flat_map(|(column_index, (column, span))| {
            let mut y = 0;
            column
                .panes
                .iter()
                .zip(pane_heights(column, area.rows))
                .enumerate()
                .map(move |(row, (&pane, height))| {
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
