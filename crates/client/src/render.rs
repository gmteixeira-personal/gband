use std::collections::HashMap;

use gband_core::geometry::{BORDER, Size, Tile, WindowBox, drawn_copy, placed, tiles};
use gband_core::layout::{Layout, WindowId};
use gband_core::view::{Scene, View};
use gband_emulator::{Emulator, Grid};
use gband_lua::plugin_windows::{FloatingFrame, Run};
use gband_lua::{Bar, Border};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Clear, Widget};
use tui_term::widget::{Cursor, PseudoTerminal, Screen};

use crate::animation::{Drawn, DrawnTile};
use crate::color::ColorSupport;

pub const FOCUSED_BORDER: Style = Style::new().add_modifier(Modifier::BOLD);
pub const UNFOCUSED_BORDER: Style = Style::new().add_modifier(Modifier::DIM);
pub const BANNER: Style = Style::new().fg(Color::Red).add_modifier(Modifier::REVERSED);

pub struct Shown<'a> {
    pub bar: &'a Bar,
    pub area: Rect,
}

pub struct Ribbon<'a> {
    pub layout: &'a Layout,
    pub area: Size,
    pub view: &'a View,
    pub grids: &'a HashMap<WindowId, Grid>,
    pub drawn: &'a Drawn,
    pub region: Rect,
    pub tile_border: &'a Border,
    pub floating_border: &'a Border,
    pub floats: Vec<&'a FloatingFrame>,
    pub float_focused: bool,
    pub colors: ColorSupport,
    pub banner: Option<&'a str>,
    pub bars: Vec<Shown<'a>>,
}

pub fn draw_frame(frame: &mut Frame<'_>, ribbon: &Ribbon<'_>) {
    let cursor = render(ribbon, frame.buffer_mut());
    for float in &ribbon.floats {
        draw_float(frame.buffer_mut(), ribbon.region, float, ribbon.colors);
    }
    for shown in &ribbon.bars {
        draw_bar(frame.buffer_mut(), shown, ribbon.colors);
    }
    if let Some(banner) = ribbon.banner {
        draw_banner(frame.buffer_mut(), ribbon.region, banner);
    }
    if let Some(cursor) = cursor.filter(|_| !ribbon.float_focused) {
        frame.set_cursor_position(cursor);
    }
}

fn draw_float(buffer: &mut Buffer, region: Rect, float: &FloatingFrame, colors: ColorSupport) {
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
    let inner = if let Some(border) = &float.border {
        draw_border(buffer, area, border, colors.style(&float.border_style));
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
        interior(area)
    } else {
        area
    };
    draw_lines(buffer, inner, &float.lines, colors);
}

fn draw_lines(buffer: &mut Buffer, inner: Rect, lines: &[Vec<Run>], colors: ColorSupport) {
    for (row, runs) in lines.iter().enumerate() {
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

fn draw_bar(buffer: &mut Buffer, shown: &Shown<'_>, colors: ColorSupport) {
    let area = shown.area.intersection(buffer.area);
    if area.is_empty() {
        return;
    }
    Clear.render(area, buffer);
    buffer.set_style(area, colors.style(&shown.bar.base));
    draw_lines(buffer, area, &shown.bar.lines, colors);
}

pub fn interior(area: Rect) -> Rect {
    Rect::new(
        area.x.saturating_add(BORDER),
        area.y.saturating_add(BORDER),
        area.width.saturating_sub(2 * BORDER),
        area.height.saturating_sub(2 * BORDER),
    )
}

pub fn draw_border(buffer: &mut Buffer, area: Rect, border: &Border, style: Style) {
    if area.is_empty() {
        return;
    }
    let [
        top_left,
        top,
        top_right,
        right,
        bottom_right,
        bottom,
        bottom_left,
        left,
    ] = border.chars.glyphs();
    let sides = border.sides;
    let (first_x, last_x) = (area.left(), area.right() - 1);
    let (first_y, last_y) = (area.top(), area.bottom() - 1);
    let mut put = |x: u16, y: u16, symbol: &str| {
        if let Some(cell) = buffer.cell_mut((x, y)) {
            cell.set_symbol(symbol).set_style(style);
        }
    };
    let side = |drawn: bool, symbol| if drawn { symbol } else { " " };
    for x in first_x..=last_x {
        put(x, first_y, side(sides.top, top));
        put(x, last_y, side(sides.bottom, bottom));
    }
    for y in first_y..=last_y {
        put(first_x, y, side(sides.left, left));
        put(last_x, y, side(sides.right, right));
    }
    let corner = |vertical: bool, horizontal: bool, both, vertical_side, horizontal_side| match (
        vertical, horizontal,
    ) {
        (true, true) => both,
        (true, false) => vertical_side,
        (false, true) => horizontal_side,
        (false, false) => " ",
    };
    put(
        first_x,
        first_y,
        corner(sides.top, sides.left, top_left, top, left),
    );
    put(
        last_x,
        first_y,
        corner(sides.top, sides.right, top_right, top, right),
    );
    put(
        last_x,
        last_y,
        corner(sides.bottom, sides.right, bottom_right, bottom, right),
    );
    put(
        first_x,
        last_y,
        corner(sides.bottom, sides.left, bottom_left, bottom, left),
    );
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
        let mut placed: Vec<(WindowId, DrawnTile)> = tiles(band, ribbon.area)
            .iter()
            .map(|tile| {
                let moving = ribbon.drawn.tiles.get(&tile.window).copied();
                (tile.window, moving.unwrap_or_else(|| DrawnTile::from(tile)))
            })
            .collect();
        placed.sort_by_key(|&(window, _)| focused == Some(window));
        for (window, tile) in placed {
            let Some(placement) =
                Placement::new(&tile, drawn.camera, drawn.strip, drawn.top, target)
            else {
                continue;
            };
            let is_focused = focused == Some(window);
            let grid = ribbon.grids.get(&window);
            let border = (ribbon.tile_border, is_focused);
            paint(buffer, target, &tile, &placement, grid, border);
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
    for window in ribbon.view.stacking(scene) {
        let Some(floating) = ribbon.layout.floating(window) else {
            continue;
        };
        let placed = placed(floating, ribbon.area);
        let tile = DrawnTile::from(&placed);
        let Some(placement) = Placement::new(&tile, 0, None, 0, target) else {
            continue;
        };
        if cursor.is_some_and(|cursor| covers(&placed, target, cursor)) {
            cursor = None;
        }
        let is_focused = focused == Some(window);
        let grid = ribbon.grids.get(&window);
        let border = (ribbon.floating_border, is_focused);
        paint(buffer, target, &tile, &placement, grid, border);
        if is_focused && ribbon.drawn.settled {
            cursor = grid.and_then(|grid| cursor_position(&tile, &placement, grid, target));
        }
    }
    cursor
}

fn covers(placed: &WindowBox, target: Rect, cursor: Position) -> bool {
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
    border: (&Border, bool),
) {
    let (scratch, shift) = draw_tile(tile, grid, border, &placement.clip);
    let clip = &placement.clip;
    for row in clip.top..clip.bottom {
        for column in clip.start..clip.end {
            let x = (placement.left + i64::from(column)) as u16;
            let y = (placement.top + i64::from(row)) as u16;
            buffer[(target.x + x, target.y + y)] =
                scratch[(column - shift.cols, row - shift.rows)].clone();
        }
    }
}

impl From<&WindowBox> for DrawnTile {
    fn from(placed: &WindowBox) -> Self {
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
    clip: Clip,
}

impl Placement {
    fn new(
        tile: &DrawnTile,
        camera: i64,
        strip: Option<u32>,
        band_top: i64,
        target: Rect,
    ) -> Option<Self> {
        let height = i64::from(target.height);
        let left = tile.x - camera;
        let left = strip.map_or(left, |strip| drawn_copy(left, strip, target.width));
        let top = band_top + tile.y;
        let clip_top = band_top.max(0);
        let clip_bottom = (band_top + height).min(height);

        let width = i64::from(tile.width);
        let rows = i64::from(tile.height);
        let clip = Clip {
            start: (-left).clamp(0, width) as u16,
            end: (i64::from(target.width) - left).clamp(0, width) as u16,
            top: (clip_top - top).clamp(0, rows) as u16,
            bottom: (clip_bottom - top).clamp(0, rows) as u16,
        };
        (clip.start < clip.end && clip.top < clip.bottom).then_some(Self { left, top, clip })
    }
}

struct Clip {
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
    (border, focused): (&Border, bool),
    clip: &Clip,
) -> (Buffer, Shift) {
    let cut_left = u16::from(clip.start > 0);
    let cut_right = u16::from(clip.end < tile.width);
    let cut_top = u16::from(clip.top > 0);
    let cut_bottom = u16::from(clip.bottom < tile.height);
    let shift = Shift {
        cols: clip.start - cut_left,
        rows: clip.top - cut_top,
    };
    let area = Rect::new(
        0,
        0,
        clip.end - clip.start + cut_left + cut_right,
        clip.bottom - clip.top + cut_top + cut_bottom,
    );
    let mut scratch = Buffer::empty(area);
    let style = if focused {
        FOCUSED_BORDER
    } else {
        UNFOCUSED_BORDER
    };
    draw_border(&mut scratch, area, border, style);
    let inner = interior(area);
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
