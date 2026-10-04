use std::collections::HashMap;

use gband_core::geometry::{BORDER, Size, Tile, tiles};
use gband_core::layout::{Layout, PaneId};
use gband_core::view::View;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Widget};
use tui_term::widget::{Cursor, PseudoTerminal};

pub const FOCUSED_BORDER: Style = Style::new().add_modifier(Modifier::BOLD);
pub const UNFOCUSED_BORDER: Style = Style::new().add_modifier(Modifier::DIM);

pub struct Ribbon<'a> {
    pub layout: &'a Layout,
    pub area: Size,
    pub view: &'a View,
    pub parsers: &'a HashMap<PaneId, vt100::Parser>,
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
        let screen = ribbon.parsers.get(&tile.pane).map(vt100::Parser::screen);
        let scratch = draw_tile(&tile, screen, is_focused);
        for row in 0..tile.height {
            let y = tile.y + row;
            if y >= target.height {
                break;
            }
            for column in 0..tile.width {
                let x = left + i64::from(column);
                if x < 0 || x >= i64::from(target.width) {
                    continue;
                }
                buffer[(target.x + x as u16, target.y + y)] = scratch[(column, row)].clone();
            }
        }
        if is_focused {
            cursor = screen.and_then(|screen| cursor_position(&tile, left, screen, target));
        }
    }
    cursor
}

fn draw_tile(tile: &Tile, screen: Option<&vt100::Screen>, focused: bool) -> Buffer {
    let area = Rect::new(0, 0, tile.width, tile.height);
    let mut scratch = Buffer::empty(area);
    let style = if focused {
        FOCUSED_BORDER
    } else {
        UNFOCUSED_BORDER
    };
    let block = Block::bordered().border_style(style);
    let inner = block.inner(area);
    block.render(area, &mut scratch);
    if let Some(screen) = screen {
        PseudoTerminal::new(screen)
            .cursor(Cursor::default().visibility(false))
            .render(inner, &mut scratch);
    }
    scratch
}

fn cursor_position(
    tile: &Tile,
    left: i64,
    screen: &vt100::Screen,
    target: Rect,
) -> Option<Position> {
    if screen.hide_cursor() {
        return None;
    }
    let (row, column) = screen.cursor_position();
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
