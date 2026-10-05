use std::collections::HashMap;

use gband_core::geometry::{BORDER, PaneBox, Size, Tile, placed, tiles};
use gband_core::layout::{Layout, PaneId};
use gband_core::view::{Scene, View};
use gband_emulator::{Emulator, Grid};
use gband_lua::StatusLine;
use gband_lua::windows::FloatFrame;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Clear, Widget};
use tui_term::widget::{Cursor, PseudoTerminal, Screen};

use crate::animation::{Drawn, DrawnTile};
use crate::color::ColorSupport;

pub const FOCUSED_BORDER: Style = Style::new().add_modifier(Modifier::BOLD);
pub const UNFOCUSED_BORDER: Style = Style::new().add_modifier(Modifier::DIM);
pub const BANNER: Style = Style::new().fg(Color::Red).add_modifier(Modifier::REVERSED);

pub struct StatusArea<'a> {
    pub area: Rect,
    pub line: Option<&'a StatusLine>,
    pub colors: ColorSupport,
}

pub struct Ribbon<'a> {
    pub layout: &'a Layout,
    pub area: Size,
    pub view: &'a View,
    pub grids: &'a HashMap<PaneId, Grid>,
    pub drawn: &'a Drawn,
    pub region: Rect,
    pub floats: Vec<&'a FloatFrame>,
    pub float_focused: bool,
    pub colors: ColorSupport,
    pub banner: Option<&'a str>,
    pub status: Option<StatusArea<'a>>,
}

pub fn draw_frame(frame: &mut Frame<'_>, ribbon: &Ribbon<'_>) {
    let cursor = render(ribbon, frame.buffer_mut());
    for float in &ribbon.floats {
        draw_float(frame.buffer_mut(), ribbon.region, float, ribbon.colors);
    }
    match (&ribbon.status, ribbon.banner) {
        (Some(status), _) => draw_status(frame.buffer_mut(), status),
        (None, Some(banner)) => draw_banner(frame.buffer_mut(), ribbon.region, banner),
        (None, None) => {}
    }
    if let Some(cursor) = cursor.filter(|_| !ribbon.float_focused) {
        frame.set_cursor_position(cursor);
    }
}

fn draw_float(buffer: &mut Buffer, region: Rect, float: &FloatFrame, colors: ColorSupport) {
    let placed = Rect::new(
        region.x.saturating_add(float.col),
        region.y.saturating_add(float.row),
        float.width,
        float.height,
    );
    let area = placed.intersection(region).intersection(buffer.area);
    if area.is_empty() {
        return;
    }
    Clear.render(area, buffer);
    buffer.set_style(area, colors.style(&float.base));
    let inner = if float.border {
        Block::bordered()
            .border_style(colors.style(&float.border_style))
            .render(area, buffer);
        if let Some(title) = &float.title {
            let room = area.width.saturating_sub(2);
            let cells = (gband_lua::ui::width(title) as u16).min(room);
            let x = area.x + 1;
            buffer.set_style(Rect::new(x, area.y, cells, 1), Style::reset());
            buffer.set_stringn(
                x,
                area.y,
                title,
                usize::from(cells),
                colors.style(&float.title_style),
            );
        }
        Block::bordered().inner(area)
    } else {
        area
    };
    for (row, runs) in float.lines.iter().enumerate() {
        let Some(y) = u16::try_from(row)
            .ok()
            .map(|row| inner.y + row)
            .filter(|&y| y < inner.bottom())
        else {
            break;
        };
        let mut x = inner.x;
        for run in runs {
            let room = inner.right().saturating_sub(x);
            if room == 0 {
                break;
            }
            let cells = (gband_lua::ui::width(&run.text) as u16).min(room);
            buffer.set_style(Rect::new(x, y, cells, 1), Style::reset());
            buffer.set_stringn(
                x,
                y,
                &run.text,
                usize::from(cells),
                colors.style(&run.style),
            );
            x += cells;
        }
    }
}

fn draw_status(buffer: &mut Buffer, status: &StatusArea<'_>) {
    let area = status.area.intersection(buffer.area);
    if area.is_empty() {
        return;
    }
    Clear.render(area, buffer);
    let Some(line) = status.line else {
        return;
    };
    buffer.set_style(area, status.colors.style(&line.base));
    for span in &line.spans {
        let Some(room) = area.width.checked_sub(span.col).filter(|room| *room > 0) else {
            continue;
        };
        let cells = gband_lua::ui::width(&span.text).min(usize::from(room)) as u16;
        let x = area.x + span.col;
        buffer.set_style(Rect::new(x, area.y, cells, 1), Style::reset());
        buffer.set_stringn(
            x,
            area.y,
            &span.text,
            usize::from(cells),
            status.colors.style(&span.style),
        );
    }
}

fn draw_banner(buffer: &mut Buffer, region: Rect, text: &str) {
    let area = region.intersection(buffer.area);
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
    let target = ribbon.region.intersection(buffer.area);
    let focused = ribbon.view.focused();
    let mut cursor = None;
    for drawn in &ribbon.drawn.bands {
        let Some(band) = ribbon.layout.band(drawn.band) else {
            continue;
        };
        let mut placed: Vec<(PaneId, DrawnTile)> = tiles(band, ribbon.area)
            .iter()
            .map(|tile| {
                let moving = ribbon.drawn.tiles.get(&tile.pane).copied();
                (tile.pane, moving.unwrap_or_else(|| DrawnTile::from(tile)))
            })
            .collect();
        placed.sort_by_key(|&(pane, _)| focused == Some(pane));
        for (pane, tile) in placed {
            let Some(placement) = Placement::new(&tile, drawn.camera, drawn.top, target) else {
                continue;
            };
            let is_focused = focused == Some(pane);
            let grid = ribbon.grids.get(&pane);
            paint(buffer, target, &tile, &placement, grid, is_focused);
            if is_focused && ribbon.drawn.settled {
                cursor = grid.and_then(|grid| cursor_position(&tile, &placement, grid, target));
            }
        }
    }
    let scene = Scene {
        layout: ribbon.layout,
        area: ribbon.area,
        viewport: Size::new(target.width, target.height),
    };
    for pane in ribbon.view.stacking(scene) {
        let Some(floating) = ribbon.layout.floating(pane) else {
            continue;
        };
        let placed = placed(floating, ribbon.area);
        let tile = DrawnTile::from(&placed);
        let Some(placement) = Placement::new(&tile, 0, 0, target) else {
            continue;
        };
        if cursor.is_some_and(|cursor| covers(&placed, target, cursor)) {
            cursor = None;
        }
        let is_focused = focused == Some(pane);
        let grid = ribbon.grids.get(&pane);
        paint(buffer, target, &tile, &placement, grid, is_focused);
        if is_focused && ribbon.drawn.settled {
            cursor = grid.and_then(|grid| cursor_position(&tile, &placement, grid, target));
        }
    }
    cursor
}

fn covers(placed: &PaneBox, target: Rect, cursor: Position) -> bool {
    let x = cursor.x.checked_sub(target.x);
    let y = cursor.y.checked_sub(target.y);
    x.zip(y).is_some_and(|(x, y)| placed.contains(x, y))
}

fn paint(
    buffer: &mut Buffer,
    target: Rect,
    tile: &DrawnTile,
    placement: &Placement,
    grid: Option<&Grid>,
    focused: bool,
) {
    let (scratch, shift) = draw_tile(tile, grid, focused, &placement.window);
    let window = &placement.window;
    for row in window.top..window.bottom {
        for column in window.start..window.end {
            let x = (placement.left + i64::from(column)) as u16;
            let y = (placement.top + i64::from(row)) as u16;
            buffer[(target.x + x, target.y + y)] =
                scratch[(column - shift.cols, row - shift.rows)].clone();
        }
    }
}

impl From<&PaneBox> for DrawnTile {
    fn from(placed: &PaneBox) -> Self {
        Self {
            x: i64::from(placed.x),
            y: i64::from(placed.y),
            width: placed.width,
            height: placed.height,
        }
    }
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
    fn new(tile: &DrawnTile, camera: i64, band_top: i64, target: Rect) -> Option<Self> {
        let height = i64::from(target.height);
        let left = tile.x - camera;
        let top = band_top + tile.y;
        let clip_top = band_top.max(0);
        let clip_bottom = (band_top + height).min(height);

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
