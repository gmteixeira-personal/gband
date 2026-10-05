use std::collections::HashMap;

use gband_core::geometry::{BORDER, Size, Tile, tiles};
use gband_core::layout::{Layout, PaneId};
use gband_core::view::View;
use gband_emulator::{Emulator, Grid};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Clear, Widget};
use tui_term::widget::{Cursor, PseudoTerminal, Screen};

use crate::animation::{Band, Drawn, DrawnTile};

pub const FOCUSED_BORDER: Style = Style::new().add_modifier(Modifier::BOLD);
pub const UNFOCUSED_BORDER: Style = Style::new().add_modifier(Modifier::DIM);
pub const BANNER: Style = Style::new().fg(Color::Red).add_modifier(Modifier::REVERSED);

pub struct Ribbon<'a> {
    pub layout: &'a Layout,
    pub area: Size,
    pub view: &'a View,
    pub grids: &'a HashMap<PaneId, Grid>,
    pub drawn: &'a Drawn,
    pub banner: Option<&'a str>,
}

pub fn draw_frame(frame: &mut Frame<'_>, ribbon: &Ribbon<'_>) {
    let cursor = render(ribbon, frame.buffer_mut());
    if let Some(banner) = ribbon.banner {
        draw_banner(frame.buffer_mut(), banner);
    }
    if let Some(cursor) = cursor {
        frame.set_cursor_position(cursor);
    }
}

fn draw_banner(buffer: &mut Buffer, text: &str) {
    let area = buffer.area;
    let Some(row) = area.bottom().checked_sub(1).filter(|_| area.height > 0) else {
        return;
    };
    let line = Rect::new(area.x, row, area.width, 1);
    Clear.render(line, buffer);
    buffer.set_style(line, BANNER);
    let first_line = text.lines().next().unwrap_or_default();
    buffer.set_stringn(area.x, row, first_line, usize::from(area.width), BANNER);
}

pub fn render(ribbon: &Ribbon<'_>, buffer: &mut Buffer) -> Option<Position> {
    let target = buffer.area;
    let focused = ribbon.view.focused();
    let mut cursor = None;
    for band in &ribbon.drawn.bands {
        let Some(workspace) = ribbon.layout.workspace(band.workspace) else {
            continue;
        };
        let mut placed: Vec<(PaneId, DrawnTile)> = tiles(workspace, ribbon.area)
            .iter()
            .map(|tile| {
                let drawn = ribbon.drawn.tiles.get(&tile.pane).copied();
                (tile.pane, drawn.unwrap_or_else(|| DrawnTile::from(tile)))
            })
            .collect();
        placed.sort_by_key(|&(pane, _)| focused == Some(pane));
        for (pane, tile) in placed {
            let Some(placement) = Placement::new(&tile, band, target) else {
                continue;
            };
            let is_focused = focused == Some(pane);
            let grid = ribbon.grids.get(&pane);
            let (scratch, shift) = draw_tile(&tile, grid, is_focused, &placement.window);
            let window = &placement.window;
            for row in window.top..window.bottom {
                for column in window.start..window.end {
                    let x = (placement.left + i64::from(column)) as u16;
                    let y = (placement.top + i64::from(row)) as u16;
                    buffer[(target.x + x, target.y + y)] =
                        scratch[(column - shift.cols, row - shift.rows)].clone();
                }
            }
            if is_focused && ribbon.drawn.settled {
                cursor = grid.and_then(|grid| cursor_position(&tile, &placement, grid, target));
            }
        }
    }
    cursor
}

impl From<&Tile> for DrawnTile {
    fn from(tile: &Tile) -> Self {
        Self {
            x: i64::from(tile.x),
            y: i64::from(tile.y),
            width: tile.width,
            height: tile.height,
        }
    }
}

struct Placement {
    left: i64,
    top: i64,
    window: Window,
}

impl Placement {
    fn new(tile: &DrawnTile, band: &Band, target: Rect) -> Option<Self> {
        let height = i64::from(target.height);
        let left = tile.x - band.camera;
        let top = band.top + tile.y;
        let clip_top = band.top.max(0);
        let clip_bottom = (band.top + height).min(height);
        let width = i64::from(tile.width);
        let rows = i64::from(tile.height);
        let window = Window {
            start: (-left).clamp(0, width) as u16,
            end: (i64::from(target.width) - left).clamp(0, width) as u16,
            top: (clip_top - top).clamp(0, rows) as u16,
            bottom: (clip_bottom - top).clamp(0, rows) as u16,
        };
        (window.start < window.end && window.top < window.bottom).then_some(Self {
            left,
            top,
            window,
        })
    }
}

struct Window {
    start: u16,
    end: u16,
    top: u16,
    bottom: u16,
}

#[derive(Clone, Copy)]
struct Shift {
    cols: u16,
    rows: u16,
}

struct Shifted<'a, S> {
    screen: &'a S,
    shift: Shift,
}

impl<S: Screen> Screen for Shifted<'_, S> {
    type C = S::C;

    fn cell(&self, row: u16, col: u16) -> Option<&Self::C> {
        self.screen.cell(
            row.checked_add(self.shift.rows)?,
            col.checked_add(self.shift.cols)?,
        )
    }

    fn hide_cursor(&self) -> bool {
        true
    }

    fn cursor_position(&self) -> (u16, u16) {
        self.screen.cursor_position()
    }
}

fn draw_tile(
    tile: &DrawnTile,
    grid: Option<&Grid>,
    focused: bool,
    window: &Window,
) -> (Buffer, Shift) {
    let cut_left = u16::from(window.start > 0);
    let cut_right = u16::from(window.end < tile.width);
    let cut_top = u16::from(window.top > 0);
    let cut_bottom = u16::from(window.bottom < tile.height);
    let shift = Shift {
        cols: window.start - cut_left,
        rows: window.top - cut_top,
    };
    let area = Rect::new(
        0,
        0,
        window.end - window.start + cut_left + cut_right,
        window.bottom - window.top + cut_top + cut_bottom,
    );
    let mut scratch = Buffer::empty(area);
    let style = if focused {
        FOCUSED_BORDER
    } else {
        UNFOCUSED_BORDER
    };
    let block = Block::bordered().border_style(style);
    let inner = block.inner(area);
    block.render(area, &mut scratch);
    if let Some(grid) = grid {
        let screen = Shifted {
            screen: grid.screen(),
            shift,
        };
        PseudoTerminal::new(&screen)
            .cursor(Cursor::default().visibility(false))
            .render(inner, &mut scratch);
    }
    (scratch, shift)
}

fn cursor_position(
    tile: &DrawnTile,
    placement: &Placement,
    grid: &Grid,
    target: Rect,
) -> Option<Position> {
    let (row, column) = grid.cursor()?;
    let interior = Size::new(
        tile.width.saturating_sub(2 * BORDER).max(1),
        tile.height.saturating_sub(2 * BORDER).max(1),
    );
    if row >= interior.rows || column >= interior.cols {
        return None;
    }
    let x = placement.left + i64::from(BORDER + column);
    let y = placement.top + i64::from(BORDER + row);
    if x < 0 || x >= i64::from(target.width) || y < 0 || y >= i64::from(target.height) {
        return None;
    }
    Some(Position::new(target.x + x as u16, target.y + y as u16))
}
