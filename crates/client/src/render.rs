use std::collections::HashMap;

use gband_core::geometry::{BORDER, Size, Tile, WindowBox, drawn_copy, placed, tiles};
use gband_core::layout::{BandId, Layout, WindowId};
use gband_core::view::{Scene, View};
use gband_emulator::{Emulator, Grid};
use gband_lua::plugin_windows::{FloatingFrame, Run};
use gband_lua::{Bar, Border, BorderChars, ClientStyles, Palette, Sides};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Clear, Widget};
use tui_term::widget::{Cursor, PseudoTerminal, Screen};

use crate::animation::{Drawn, DrawnTile};
use crate::color::ColorSupport;

pub struct Shown<'a> {
    pub bar: &'a Bar,
    pub area: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionKind {
    Tile { column: usize, row: usize },
    Lifted,
    Floating,
    PluginFloat(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub kind: RegionKind,
    pub window: Option<WindowId>,
    pub band: Option<BandId>,
    pub x: i64,
    pub y: i64,
    pub width: u16,
    pub height: u16,
    pub visible: Rect,
    pub border: bool,
}

impl Region {
    pub fn contains(&self, col: u16, row: u16) -> bool {
        self.visible.contains(Position::new(col, row))
    }

    fn inset(&self) -> u16 {
        if self.border { BORDER } else { 0 }
    }

    pub fn content_size(&self) -> (u16, u16) {
        let inset = 2 * self.inset();
        (
            self.width.saturating_sub(inset).max(1),
            self.height.saturating_sub(inset).max(1),
        )
    }

    pub fn content_cell(&self, col: u16, row: u16) -> Option<(u16, u16)> {
        let inset = i64::from(self.inset());
        let (cols, rows) = self.content_size();
        let x = i64::from(col) - self.x - inset;
        let y = i64::from(row) - self.y - inset;
        ((0..i64::from(cols)).contains(&x) && (0..i64::from(rows)).contains(&y))
            .then_some((x as u16, y as u16))
    }

    pub fn nearest_content(&self, col: i64, row: i64) -> (u16, u16) {
        let inset = i64::from(self.inset());
        let (cols, rows) = self.content_size();
        let x = (col - self.x - inset).clamp(0, i64::from(cols) - 1);
        let y = (row - self.y - inset).clamp(0, i64::from(rows) - 1);
        (x as u16, y as u16)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selected {
    pub window: WindowId,
    pub start: (u16, u16),
    pub end: (u16, u16),
}

impl Selected {
    fn holds(&self, (col, row): (u16, u16)) -> bool {
        let at = (row, col);
        at >= (self.start.1, self.start.0) && at <= (self.end.1, self.end.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lifted {
    pub window: WindowId,
    pub x: i64,
    pub y: i64,
    pub width: u16,
    pub height: u16,
    pub outline: Option<Rect>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Overlay {
    pub selection: Option<Selected>,
    pub lifted: Option<Lifted>,
    pub floating: Option<WindowBox>,
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
    pub focused_tile_chars: &'a BorderChars,
    pub focused_floating_chars: &'a BorderChars,
    pub styles: ClientStyles,
    pub palette: Palette,
    pub floats: Vec<(u32, &'a FloatingFrame)>,
    pub float_focused: bool,
    pub colors: ColorSupport,
    pub banner: Option<&'a str>,
    pub bars: Vec<Shown<'a>>,
    pub overlay: Overlay,
}

pub fn draw_frame(frame: &mut Frame<'_>, ribbon: &Ribbon<'_>) {
    let cursor = render(ribbon, frame.buffer_mut());
    for (_, float) in &ribbon.floats {
        draw_float(frame.buffer_mut(), ribbon.region, float, ribbon.colors);
    }
    for shown in &ribbon.bars {
        draw_bar(frame.buffer_mut(), shown, ribbon.colors);
    }
    if let Some(banner) = ribbon.banner {
        draw_banner(
            frame.buffer_mut(),
            ribbon.region,
            banner,
            ribbon.colors.style(&ribbon.styles.banner),
        );
    }
    apply_palette(frame.buffer_mut(), &ribbon.palette, ribbon.colors);
    if let Some(cursor) = cursor.filter(|_| !ribbon.float_focused) {
        frame.set_cursor_position(cursor);
    }
}

fn palette_index(color: Color) -> Option<usize> {
    Some(match color {
        Color::Indexed(index) if index < 16 => usize::from(index),
        Color::Black => 0,
        Color::Red => 1,
        Color::Green => 2,
        Color::Yellow => 3,
        Color::Blue => 4,
        Color::Magenta => 5,
        Color::Cyan => 6,
        Color::Gray => 7,
        Color::DarkGray => 8,
        Color::LightRed => 9,
        Color::LightGreen => 10,
        Color::LightYellow => 11,
        Color::LightBlue => 12,
        Color::LightMagenta => 13,
        Color::LightCyan => 14,
        Color::White => 15,
        _ => return None,
    })
}

fn mapped(
    color: Color,
    default: Option<gband_lua::Rgb>,
    palette: &Palette,
) -> Option<gband_lua::Rgb> {
    match color {
        Color::Reset => default,
        color => palette_index(color).and_then(|index| palette.colors[index]),
    }
}

pub fn apply_palette(buffer: &mut Buffer, palette: &Palette, colors: ColorSupport) {
    if palette.is_empty() {
        return;
    }
    let drawn = |(r, g, b)| colors.color(gband_lua::Color::Rgb(r, g, b));
    for cell in &mut buffer.content {
        if let Some(rgb) = mapped(cell.fg, palette.fg, palette) {
            cell.fg = drawn(rgb);
        }
        if let Some(rgb) = mapped(cell.bg, palette.bg, palette) {
            cell.bg = drawn(rgb);
        }
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

fn draw_banner(buffer: &mut Buffer, region: Rect, text: &str, style: Style) {
    let area = region.intersection(buffer.area);
    let Some(row) = area.bottom().checked_sub(1).filter(|_| area.height > 0) else {
        return;
    };
    let line = Rect::new(area.x, row, area.width, 1);
    Clear.render(line, buffer);
    buffer.set_style(line, style);
    let first_line = text.lines().next().unwrap_or_default();
    buffer.set_stringn(area.x, row, first_line, usize::from(area.width), style);
}

struct Layer {
    region: Region,
    tile: DrawnTile,
    placement: Placement,
    border: Border,
    focused: bool,
}

fn layers(ribbon: &Ribbon<'_>, target: Rect) -> Vec<Layer> {
    let focused = ribbon.view.focused();
    let lifted = ribbon.overlay.lifted;
    let mut layers = Vec::new();
    let mut layer = |kind,
                     window,
                     band,
                     tile: DrawnTile,
                     placement: Placement,
                     (border, focused_chars): (&Border, &BorderChars)| {
        let focused = focused == Some(window);
        let clip = &placement.clip;
        let x = i64::from(target.x) + placement.left;
        let y = i64::from(target.y) + placement.top;
        let visible = Rect::new(
            (x + i64::from(clip.start)) as u16,
            (y + i64::from(clip.top)) as u16,
            clip.end - clip.start,
            clip.bottom - clip.top,
        );
        layers.push(Layer {
            region: Region {
                kind,
                window: Some(window),
                band,
                x,
                y,
                width: tile.width,
                height: tile.height,
                visible,
                border: true,
            },
            tile,
            placement,
            border: Border {
                sides: border.sides,
                chars: if focused {
                    focused_chars.clone()
                } else {
                    border.chars.clone()
                },
            },
            focused,
        });
    };
    for drawn in &ribbon.drawn.bands {
        let Some(band) = ribbon.layout.band(drawn.band) else {
            continue;
        };
        let band_tiles = tiles(band, ribbon.area);
        let mut placed: Vec<(&Tile, DrawnTile)> = band_tiles
            .iter()
            .filter(|tile| lifted.is_none_or(|lifted| lifted.window != tile.window))
            .map(|tile| {
                let moving = ribbon.drawn.tiles.get(&tile.window).copied();
                (tile, moving.unwrap_or_else(|| DrawnTile::from(tile)))
            })
            .collect();
        placed.sort_by_key(|&(tile, _)| focused == Some(tile.window));
        for (tile, drawn_tile) in placed {
            let Some(placement) =
                Placement::new(&drawn_tile, drawn.camera, drawn.strip, drawn.top, target)
            else {
                continue;
            };
            let kind = RegionKind::Tile {
                column: tile.column,
                row: tile.row,
            };
            layer(
                kind,
                tile.window,
                Some(drawn.band),
                drawn_tile,
                placement,
                (ribbon.tile_border, ribbon.focused_tile_chars),
            );
        }
    }
    if let Some(lifted) = lifted {
        let tile = DrawnTile {
            x: lifted.x - i64::from(target.x),
            y: lifted.y - i64::from(target.y),
            width: lifted.width,
            height: lifted.height,
        };
        if let Some(placement) = Placement::new(&tile, 0, None, 0, target) {
            layer(
                RegionKind::Lifted,
                lifted.window,
                Some(ribbon.view.band()),
                tile,
                placement,
                (ribbon.tile_border, ribbon.focused_tile_chars),
            );
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
        let placed = ribbon
            .overlay
            .floating
            .filter(|moved| moved.window == window)
            .unwrap_or_else(|| placed(floating, ribbon.area));
        let tile = DrawnTile::from(&placed);
        let Some(placement) = Placement::new(&tile, 0, None, 0, target) else {
            continue;
        };
        layer(
            RegionKind::Floating,
            window,
            Some(ribbon.view.band()),
            tile,
            placement,
            (ribbon.floating_border, ribbon.focused_floating_chars),
        );
    }
    layers
}

pub fn regions(ribbon: &Ribbon<'_>, area: Rect) -> Vec<Region> {
    let target = ribbon.region.intersection(area);
    let mut regions: Vec<Region> = layers(ribbon, target)
        .into_iter()
        .map(|layer| layer.region)
        .collect();
    for &(id, float) in &ribbon.floats {
        let placed = Rect::new(
            ribbon.region.x.saturating_add(float.col),
            ribbon.region.y.saturating_add(float.row),
            float.width,
            float.height,
        );
        let visible = placed.intersection(ribbon.region).intersection(area);
        if visible.is_empty() {
            continue;
        }
        regions.push(Region {
            kind: RegionKind::PluginFloat(id),
            window: None,
            band: None,
            x: i64::from(placed.x),
            y: i64::from(placed.y),
            width: placed.width,
            height: placed.height,
            visible,
            border: float.border.is_some(),
        });
    }
    regions
}

pub fn render(ribbon: &Ribbon<'_>, buffer: &mut Buffer) -> Option<Position> {
    let target = ribbon.region.intersection(buffer.area);
    let mut cursor = None;
    let mut outlined = false;
    for layer in layers(ribbon, target) {
        let Layer {
            region,
            tile,
            placement,
            border,
            focused,
        } = layer;
        let Some(window) = region.window else {
            continue;
        };
        if region.kind == RegionKind::Floating && !outlined {
            outlined = true;
            draw_outline(ribbon, buffer, target);
        }
        if region.kind == RegionKind::Floating
            && cursor.is_some_and(|cursor| covers(&tile, &placement, target, cursor))
        {
            cursor = None;
        }
        let grid = ribbon.grids.get(&window);
        let selection = ribbon
            .overlay
            .selection
            .filter(|selected| selected.window == window);
        let style = if focused {
            &ribbon.styles.border_focused
        } else {
            &ribbon.styles.border
        };
        paint(
            buffer,
            target,
            &tile,
            &placement,
            grid,
            (&border, ribbon.colors.style(style)),
            selection,
        );
        if focused && region.kind != RegionKind::Lifted && ribbon.drawn.settled {
            cursor = grid.and_then(|grid| cursor_position(&tile, &placement, grid, target));
        }
    }
    if !outlined {
        draw_outline(ribbon, buffer, target);
    }
    cursor
}

fn draw_outline(ribbon: &Ribbon<'_>, buffer: &mut Buffer, target: Rect) {
    let Some(outline) = ribbon.overlay.lifted.and_then(|lifted| lifted.outline) else {
        return;
    };
    let sides = Border {
        sides: Sides::ALL,
        chars: ribbon.focused_tile_chars.clone(),
    };
    draw_border(
        buffer,
        outline.intersection(target),
        &sides,
        Style::reset().patch(ribbon.colors.style(&ribbon.styles.border_focused)),
    );
}

fn covers(tile: &DrawnTile, placement: &Placement, target: Rect, cursor: Position) -> bool {
    let x = i64::from(cursor.x) - i64::from(target.x) - placement.left;
    let y = i64::from(cursor.y) - i64::from(target.y) - placement.top;
    (0..i64::from(tile.width)).contains(&x) && (0..i64::from(tile.height)).contains(&y)
}

fn paint(
    buffer: &mut Buffer,
    target: Rect,
    tile: &DrawnTile,
    placement: &Placement,
    grid: Option<&Grid>,
    border: (&Border, Style),
    selection: Option<Selected>,
) {
    let (scratch, shift) = draw_tile(tile, grid, border, &placement.clip);
    let clip = &placement.clip;
    for row in clip.top..clip.bottom {
        for column in clip.start..clip.end {
            let x = (placement.left + i64::from(column)) as u16;
            let y = (placement.top + i64::from(row)) as u16;
            let cell = &mut buffer[(target.x + x, target.y + y)];
            *cell = scratch[(column - shift.cols, row - shift.rows)].clone();
            let interior = (BORDER..tile.width.saturating_sub(BORDER)).contains(&column)
                && (BORDER..tile.height.saturating_sub(BORDER)).contains(&row);
            if interior
                && selection.is_some_and(|selected| selected.holds((column - BORDER, row - BORDER)))
            {
                cell.modifier.toggle(Modifier::REVERSED);
            }
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
    (border, style): (&Border, Style),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn gruvbox() -> Palette {
        let mut palette = Palette {
            fg: Some((0xeb, 0xdb, 0xb2)),
            bg: Some((0x28, 0x28, 0x28)),
            ..Palette::default()
        };
        palette.colors[1] = Some((0xcc, 0x24, 0x1d));
        palette
    }

    fn cell(buffer: &mut Buffer, fg: Color, bg: Color) {
        buffer[(0, 0)].set_symbol("x").set_fg(fg).set_bg(bg);
    }

    fn colors(buffer: &Buffer, x: u16) -> (Color, Color) {
        let cell = &buffer[(x, 0)];
        (cell.fg, cell.bg)
    }

    #[test]
    fn program_color_mapped() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 2, 1));
        cell(&mut buffer, Color::Indexed(1), Color::Reset);
        apply_palette(&mut buffer, &gruvbox(), ColorSupport::TrueColor);
        assert_eq!(
            colors(&buffer, 0),
            (Color::Rgb(0xcc, 0x24, 0x1d), Color::Rgb(0x28, 0x28, 0x28))
        );
    }

    #[test]
    fn named_ratatui_color_mapped() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        cell(&mut buffer, Color::Red, Color::Reset);
        apply_palette(&mut buffer, &gruvbox(), ColorSupport::TrueColor);
        assert_eq!(colors(&buffer, 0).0, Color::Rgb(0xcc, 0x24, 0x1d));
    }

    #[test]
    fn empty_ribbon_takes_the_background() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 2, 1));
        apply_palette(&mut buffer, &gruvbox(), ColorSupport::TrueColor);
        assert_eq!(
            colors(&buffer, 1),
            (Color::Rgb(0xeb, 0xdb, 0xb2), Color::Rgb(0x28, 0x28, 0x28))
        );
    }

    #[test]
    fn mapped_color_without_24_bit_color() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        cell(&mut buffer, Color::Indexed(1), Color::Reset);
        apply_palette(&mut buffer, &gruvbox(), ColorSupport::Indexed);
        assert_eq!(colors(&buffer, 0).0, Color::Indexed(160));
    }

    #[test]
    fn colors_beyond_the_sixteen_unchanged() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        cell(&mut buffer, Color::Indexed(208), Color::Rgb(1, 2, 3));
        apply_palette(&mut buffer, &gruvbox(), ColorSupport::TrueColor);
        assert_eq!(
            colors(&buffer, 0),
            (Color::Indexed(208), Color::Rgb(1, 2, 3))
        );
    }

    #[test]
    fn unset_fields_unchanged() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        cell(&mut buffer, Color::Indexed(2), Color::Reset);
        let only_bg = Palette {
            bg: Some((0x10, 0x10, 0x10)),
            ..Palette::default()
        };
        apply_palette(&mut buffer, &only_bg, ColorSupport::TrueColor);
        assert_eq!(
            colors(&buffer, 0),
            (Color::Indexed(2), Color::Rgb(0x10, 0x10, 0x10))
        );
    }

    #[test]
    fn empty_palette_leaves_the_buffer_equal() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 2, 1));
        cell(&mut buffer, Color::Indexed(1), Color::Reset);
        let before = buffer.clone();
        apply_palette(&mut buffer, &Palette::default(), ColorSupport::TrueColor);
        assert_eq!(buffer, before);
    }
}
