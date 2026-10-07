use gband_core::geometry::Size;
use gband_core::input::{MouseEncoding, MouseTracking};
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

fn mouse_modes<E: Emulator>()
where
    E::Screen: Screen,
{
    let mut emulator = E::new(SIZE);
    assert_eq!(emulator.modes().mouse_tracking, MouseTracking::None);
    assert_eq!(emulator.modes().mouse_encoding, MouseEncoding::Default);
    emulator.process(b"\x1b[?1002h\x1b[?1006h");
    assert_eq!(emulator.modes().mouse_tracking, MouseTracking::ButtonMotion);
    assert_eq!(emulator.modes().mouse_encoding, MouseEncoding::Sgr);
    emulator.process(b"\x1b[?1003h");
    assert_eq!(emulator.modes().mouse_tracking, MouseTracking::AnyMotion);
    let copy = reproduced(&emulator);
    assert_eq!(copy.modes().mouse_tracking, MouseTracking::AnyMotion);
    assert_eq!(copy.modes().mouse_encoding, MouseEncoding::Sgr);
    assert_same(&emulator, &copy);
    emulator.process(b"\x1b[?1003l\x1b[?1006l");
    assert_eq!(emulator.modes().mouse_tracking, MouseTracking::None);
    assert_eq!(emulator.modes().mouse_encoding, MouseEncoding::Default);
    emulator.process(b"\x1b[?1000h\x1b[?1000l");
    assert_eq!(emulator.modes().mouse_tracking, MouseTracking::None);
    emulator.process(b"\x1b[?9h\x1b[?1005h");
    assert_eq!(emulator.modes().mouse_tracking, MouseTracking::Press);
    assert_eq!(emulator.modes().mouse_encoding, MouseEncoding::Utf8);
    emulator.process(b"\x1bc");
    assert_eq!(emulator.modes().mouse_tracking, MouseTracking::None);
    assert_eq!(emulator.modes().mouse_encoding, MouseEncoding::Default);
}

fn mouse_modes_after_a_checkpoint<E: Emulator>()
where
    E::Screen: Screen,
{
    let mut emulator = E::new(SIZE);
    let checkpoint = emulator.checkpoint();
    let mut copy = reproduced(&emulator);
    emulator.process(b"\x1b[?1000h\x1b[?1006h");
    copy.process(&emulator.diff(&checkpoint));
    assert_eq!(copy.modes().mouse_tracking, MouseTracking::PressRelease);
    assert_same(&emulator, &copy);
}

fn text_between_cells<E: Emulator>() {
    let mut emulator = E::new(Size::new(10, 5));
    emulator.process(b"hello world\r\nsecond  \r\nabcdef");
    assert_eq!(emulator.text_between((6, 0), (5, 2)), "world\nsecond");
    assert_eq!(emulator.text_between((0, 2), (9, 3)), "second\nabcdef");
    assert_eq!(emulator.text_between((1, 3), (4, 3)), "bcde");
    assert_eq!(emulator.text_between((0, 2), (9, 2)), "second");
    assert_eq!(emulator.text_between((2, 0), (6, 0)), "llo w");
}

#[test]
fn grid_reports_mouse_modes() {
    mouse_modes::<Grid>();
}

#[test]
fn grid_diff_reproduces_mouse_modes() {
    mouse_modes_after_a_checkpoint::<Grid>();
}

#[test]
fn grid_reads_text_between_cells() {
    text_between_cells::<Grid>();
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

fn titled(output: &[u8]) -> Grid {
    let mut grid = Grid::new(SIZE);
    grid.process(output);
    grid
}

#[test]
fn no_title_at_first() {
    let mut grid = titled(b"hello");
    assert_eq!(grid.title(), None);
    assert!(!grid.take_title_changed());
}

#[test]
fn osc_0_and_2_set_the_title() {
    let mut grid = titled(b"\x1b]2;build\x07");
    assert_eq!(grid.title(), Some("build"));
    assert!(grid.take_title_changed());
    assert!(!grid.take_title_changed());
    grid.process(b"\x1b]0;test\x1b\\");
    assert_eq!(grid.title(), Some("test"));
}

#[test]
fn title_holding_a_semicolon() {
    assert_eq!(
        titled(b"\x1b]0;make; make test\x07").title(),
        Some("make; make test")
    );
    assert_eq!(titled(b"\x1b]2;a;b;c\x07").title(), Some("a;b;c"));
}

#[test]
fn osc_1_leaves_the_title() {
    assert_eq!(
        titled(b"\x1b]2;build\x07\x1b]1;icon\x07").title(),
        Some("build")
    );
    assert_eq!(titled(b"\x1b]1;icon;x\x07").title(), None);
}

#[test]
fn empty_title_clears_it() {
    let mut grid = titled(b"\x1b]2;build\x07");
    grid.take_title_changed();
    grid.process(b"\x1b]2;\x07");
    assert_eq!(grid.title(), None);
    assert!(grid.take_title_changed());
    assert_eq!(titled(b"\x1b]2;build\x07\x1b]2;  \x07").title(), None);
}

#[test]
fn control_characters_removed() {
    assert_eq!(titled(b"\x1b]2; a\tb \x07").title(), Some("ab"));
}

#[test]
fn same_title_is_no_change() {
    let mut grid = titled(b"\x1b]2;build\x07");
    grid.take_title_changed();
    grid.process(b"\x1b]2;build\x07");
    assert!(!grid.take_title_changed());
}

#[test]
fn pushed_title_is_restored() {
    for (push, pop) in [("22;0", "23;0"), ("22;2", "23;2"), ("22", "23")] {
        let output = format!("\x1b]2;user@host:~\x07\x1b[{push}t\x1b]2;VIM - notes\x07");
        let mut grid = titled(output.as_bytes());
        assert_eq!(grid.title(), Some("VIM - notes"));
        grid.process(format!("\x1b[{pop}t").as_bytes());
        assert_eq!(grid.title(), Some("user@host:~"));
    }
}

#[test]
fn pushed_absence_clears_the_title() {
    let grid = titled(b"\x1b[22;2t\x1b]2;VIM - notes\x07\x1b[23;2t");
    assert_eq!(grid.title(), None);
}

#[test]
fn pop_from_an_empty_stack_keeps_the_title() {
    assert_eq!(titled(b"\x1b]2;build\x07\x1b[23t").title(), Some("build"));
}

#[test]
fn icon_stack_is_ignored() {
    let grid = titled(b"\x1b]2;a\x07\x1b[22;1t\x1b]2;b\x07\x1b[23;1t");
    assert_eq!(grid.title(), Some("b"));
    let grid = titled(b"\x1b]2;a\x07\x1b[22;0t\x1b]2;b\x07\x1b[22;1t\x1b[23;0t");
    assert_eq!(grid.title(), Some("a"));
}

#[test]
fn title_stack_drops_the_oldest_entry() {
    let mut grid = titled(b"");
    for index in 0..11 {
        grid.process(format!("\x1b]2;t{index}\x07\x1b[22t").as_bytes());
    }
    grid.process(b"\x1b]2;top\x07");
    for index in (1..11).rev() {
        grid.process(b"\x1b[23t");
        assert_eq!(grid.title(), Some(format!("t{index}").as_str()));
    }
    grid.process(b"\x1b[23t");
    assert_eq!(grid.title(), Some("t1"));
}

#[test]
fn title_kept_after_reset() {
    assert_eq!(titled(b"\x1b]2;build\x07\x1bc").title(), Some("build"));
}
