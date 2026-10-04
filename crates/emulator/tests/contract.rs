use gband_core::geometry::Size;
use gband_emulator::{Emulator, Grid};
use ratatui::buffer::Cell;
use ratatui::style::{Color, Modifier};
use tui_term::widget::{Cell as _, Screen};

const SIZE: Size = Size::new(80, 24);

fn cell<E: Emulator>(emulator: &E, row: u16, col: u16) -> Cell
where
    E::Screen: Screen,
{
    let mut cell = Cell::default();
    if let Some(source) = emulator.screen().cell(row, col) {
        source.apply(&mut cell);
    }
    cell
}

fn cells<E: Emulator>(emulator: &E) -> Vec<Cell>
where
    E::Screen: Screen,
{
    let size = emulator.size();
    (0..size.rows)
        .flat_map(|row| (0..size.cols).map(move |col| (row, col)))
        .map(|(row, col)| cell(emulator, row, col))
        .collect()
}

fn reproduced<E: Emulator>(emulator: &E) -> E {
    let mut copy = E::new(emulator.size());
    copy.process(&emulator.snapshot());
    copy
}

fn assert_same<E: Emulator>(a: &E, b: &E)
where
    E::Screen: Screen,
{
    assert_eq!(a.size(), b.size());
    assert_eq!(cells(a), cells(b));
    assert_eq!(a.cursor(), b.cursor());
    assert_eq!(a.modes(), b.modes());
}

fn coloured_text<E: Emulator>()
where
    E::Screen: Screen,
{
    let mut emulator = E::new(SIZE);
    emulator.process(b"\x1b[1;31mred\x1b[0m plain\r\n");
    let copy = reproduced(&emulator);
    for grid in [&emulator, &copy] {
        for col in 0..3 {
            let cell = cell(grid, 0, col);
            assert_eq!(
                cell.symbol(),
                &"red"[usize::from(col)..usize::from(col) + 1]
            );
            assert_eq!(cell.fg, Color::Indexed(1));
            assert!(cell.modifier.contains(Modifier::BOLD));
        }
        for (col, expected) in (3..9).zip(" plain".chars()) {
            let cell = cell(grid, 0, col);
            assert_eq!(cell.symbol(), expected.to_string());
            assert_eq!(cell.fg, Color::Reset);
            assert!(cell.modifier.is_empty());
        }
        assert_eq!(grid.cursor(), Some((1, 0)));
    }
    assert_same(&emulator, &copy);
}

fn wide_characters<E: Emulator>()
where
    E::Screen: Screen,
{
    let mut emulator = E::new(SIZE);
    emulator.process("漢字x".as_bytes());
    let copy = reproduced(&emulator);
    for grid in [&emulator, &copy] {
        assert_eq!(cell(grid, 0, 0).symbol(), "漢");
        assert_eq!(cell(grid, 0, 2).symbol(), "字");
        assert_eq!(cell(grid, 0, 4).symbol(), "x");
        assert!(grid.contents().starts_with("漢字x"));
    }
    assert_same(&emulator, &copy);
}

fn alternate_screen_and_modes<E: Emulator>()
where
    E::Screen: Screen,
{
    let mut emulator = E::new(SIZE);
    emulator.process(b"main\x1b[?1049h\x1b[?1h\x1b[?2004halternate");
    assert!(emulator.alternate_screen());
    let copy = reproduced(&emulator);
    for grid in [&emulator, &copy] {
        assert!(grid.contents().contains("alternate"));
        assert!(!grid.contents().contains("main"));
        assert!(grid.modes().application_cursor);
        assert!(grid.modes().bracketed_paste);
    }
    assert_same(&emulator, &copy);
}

fn cursor_movement_after_a_checkpoint<E: Emulator>()
where
    E::Screen: Screen,
{
    let mut emulator = E::new(SIZE);
    emulator.process(b"hello\r\nworld\r\n");
    let checkpoint = emulator.checkpoint();
    let mut copy = reproduced(&emulator);
    emulator.process(b"abc\x1b[2;5Hxy\x1b[1K");
    copy.process(&emulator.diff(&checkpoint));
    assert_same(&emulator, &copy);
}

fn nothing_changed<E: Emulator>() {
    let mut emulator = E::new(SIZE);
    emulator.process(b"\x1b[1;32msome text\x1b[0m\r\n");
    let checkpoint = emulator.checkpoint();
    assert!(emulator.diff(&checkpoint).is_empty());
}

fn shrink<E: Emulator>() {
    let mut emulator = E::new(SIZE);
    emulator.process(b"hello");
    emulator.resize(Size::new(40, 10));
    assert_eq!(emulator.size(), Size::new(40, 10));
    assert_eq!(emulator.contents().lines().next(), Some("hello"));
}

fn plain_output_asks_nothing<E: Emulator>() {
    let mut emulator = E::new(SIZE);
    emulator.process(b"hello\r\n");
    assert!(emulator.take_write_back().is_empty());
}

#[test]
fn grid_reproduces_coloured_text() {
    coloured_text::<Grid>();
}

#[test]
fn grid_reproduces_wide_characters() {
    wide_characters::<Grid>();
}

#[test]
fn grid_reproduces_the_alternate_screen_and_modes() {
    alternate_screen_and_modes::<Grid>();
}

#[test]
fn grid_diff_reproduces_cursor_movement() {
    cursor_movement_after_a_checkpoint::<Grid>();
}

#[test]
fn grid_diff_of_nothing_is_empty() {
    nothing_changed::<Grid>();
}

#[test]
fn grid_keeps_cells_when_shrunk() {
    shrink::<Grid>();
}

#[test]
fn grid_writes_nothing_back_for_plain_output() {
    plain_output_asks_nothing::<Grid>();
}
