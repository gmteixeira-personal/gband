use std::collections::HashMap;

use gband_core::geometry::{BORDER, Size, Tile, tiles};
use gband_core::layout::{Layout, PaneId};
use gband_core::view::View;
use gband_emulator::{Emulator, Grid};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Widget};
use tui_term::widget::{Cursor, PseudoTerminal, Screen};

pub const FOCUSED_BORDER: Style = Style::new().add_modifier(Modifier::BOLD);
pub const UNFOCUSED_BORDER: Style = Style::new().add_modifier(Modifier::DIM);

pub struct Ribbon<'a> {
    pub layout: &'a Layout,
    pub area: Size,
    pub view: &'a View,
    pub grids: &'a HashMap<PaneId, Grid>,
}

pub fn draw_frame(frame: &mut Frame<'_>, ribbon: &Ribbon<'_>) {
    if let Some(cursor) = render(ribbon, frame.buffer_mut()) {
        frame.set_cursor_position(cursor);
    }
}

pub fn render(ribbon: &Ribbon<'_>, buffer: &mut Buffer) -> Option<Position> {
    let target = buffer.area;
    let workspace = ribbon.layout.workspace(ribbon.view.workspace())?;
    let camera = i64::from(ribbon.view.camera());
    let focused = ribbon.view.focused();
    let mut cursor = None;
    for tile in tiles(workspace, ribbon.area) {
        let left = i64::from(tile.x) - camera;
        let right = left + i64::from(tile.width);
        if right <= 0 || left >= i64::from(target.width) || tile.y >= target.height {
            continue;
        }
        let is_focused = focused == Some(tile.pane);
        let grid = ribbon.grids.get(&tile.pane);
        let window = Window {
            start: (-left).max(0) as u16,
            end: (i64::from(target.width) - left).min(i64::from(tile.width)) as u16,
            rows: tile.height.min(target.height - tile.y),
        };
        let (scratch, shift) = draw_tile(&tile, grid, is_focused, &window);
        for row in 0..window.rows {
            for column in window.start..window.end {
                let x = (left + i64::from(column)) as u16;
                buffer[(target.x + x, target.y + tile.y + row)] =
                    scratch[(column - shift, row)].clone();
            }
        }
        if is_focused {
            cursor = grid.and_then(|grid| cursor_position(&tile, left, grid, target));
        }
    }
    cursor
}

struct Window {
    start: u16,
    end: u16,
    rows: u16,
}

struct Shifted<'a, S> {
    screen: &'a S,
    cols: u16,
}

impl<S: Screen> Screen for Shifted<'_, S> {
    type C = S::C;

    fn cell(&self, row: u16, col: u16) -> Option<&Self::C> {
        self.screen.cell(row, col.checked_add(self.cols)?)
    }

    fn hide_cursor(&self) -> bool {
        true
    }

    fn cursor_position(&self) -> (u16, u16) {
        self.screen.cursor_position()
    }
}

fn draw_tile(tile: &Tile, grid: Option<&Grid>, focused: bool, window: &Window) -> (Buffer, u16) {
    let cut_left = u16::from(window.start > 0);
    let cut_right = u16::from(window.end < tile.width);
    let cut_bottom = u16::from(window.rows < tile.height);
    let shift = window.start - cut_left;
    let area = Rect::new(
        0,
        0,
        window.end - window.start + cut_left + cut_right,
        window.rows + cut_bottom,
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
            cols: shift,
        };
        PseudoTerminal::new(&screen)
            .cursor(Cursor::default().visibility(false))
            .render(inner, &mut scratch);
    }
    (scratch, shift)
}

fn cursor_position(tile: &Tile, left: i64, grid: &Grid, target: Rect) -> Option<Position> {
    let (row, column) = grid.cursor()?;
    let inner = tile.terminal_size();
    if row >= inner.rows || column >= inner.cols {
        return None;
    }
    let x = left + i64::from(BORDER + column);
    let y = tile.y + BORDER + row;
    if x < 0 || x >= i64::from(target.width) || y >= target.height {
        return None;
    }
    Some(Position::new(target.x + x as u16, target.y + y))
}
