use gband_core::geometry::{CellSize, Size};
use gband_emulator::{Emulator, GRAPHICS_LIMIT, Grid};

const SIZE: Size = Size::new(20, 6);

const SIXEL: &[u8] = b"\x1bPq#0;2;0;0;0#0~~\x1b\\";

#[test]
fn sixel_is_collected_with_its_position() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"\x1b[3;5H");
    grid.process(SIXEL);
    assert_eq!(grid.take_sixel(), vec![(3, 5, SIXEL.to_vec())]);
    assert_eq!(grid.contents(), "");
    assert_eq!(grid.cursor(), Some((2, 4)));
}

#[test]
fn position_is_taken_where_the_sequence_begins() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"ab\x1bPq#0~~\x1b\\cd");
    assert_eq!(
        grid.take_sixel(),
        vec![(1, 3, b"\x1bPq#0~~\x1b\\".to_vec())]
    );
    assert_eq!(grid.contents(), "abcd");
}

#[test]
fn sequence_split_across_calls() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"\x1bPq#0;2;0;0;0");
    assert!(grid.take_sixel().is_empty());
    grid.process(b"#0~~\x1b\\");
    assert_eq!(grid.take_sixel(), vec![(1, 1, SIXEL.to_vec())]);
}

#[test]
fn introducer_split_across_calls() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"x\x1b");
    grid.process(b"P0;1");
    grid.process(b"q#0~~\x1b\\");
    assert_eq!(
        grid.take_sixel(),
        vec![(1, 2, b"\x1bP0;1q#0~~\x1b\\".to_vec())]
    );
    assert_eq!(grid.contents(), "x");
}

#[test]
fn collected_sequences_are_cleared_when_read() {
    let mut grid = Grid::new(SIZE);
    grid.process(SIXEL);
    assert_eq!(grid.take_sixel().len(), 1);
    assert!(grid.take_sixel().is_empty());
}

#[test]
fn sequences_are_reported_in_order() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"\x1bPqA\x1b\\\r\n\x1bPqB\x1b\\");
    assert_eq!(
        grid.take_sixel(),
        vec![
            (1, 1, b"\x1bPqA\x1b\\".to_vec()),
            (2, 1, b"\x1bPqB\x1b\\".to_vec())
        ]
    );
}

#[test]
fn non_sixel_device_control_string_is_ignored() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"\x1bP$qm\x1b\\");
    assert!(grid.take_sixel().is_empty());
    grid.process(b"\x1bP>|x\x1b\\");
    assert!(grid.take_sixel().is_empty());
}

#[test]
fn sixel_and_graphics_are_collected_apart() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"\x1b_Ga=T;AAAA\x1b\\");
    grid.process(SIXEL);
    assert_eq!(grid.take_graphics(), vec![b"\x1b_Ga=T;AAAA\x1b\\".to_vec()]);
    assert_eq!(grid.take_sixel(), vec![(1, 1, SIXEL.to_vec())]);
}

#[test]
fn sequence_past_the_limit_is_discarded() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"\x1bPq");
    grid.process(&vec![b'~'; GRAPHICS_LIMIT]);
    grid.process(b"\x1b\\");
    assert!(grid.take_sixel().is_empty());
    grid.process(SIXEL);
    assert_eq!(grid.take_sixel(), vec![(1, 1, SIXEL.to_vec())]);
}

#[test]
fn snapshot_holds_no_sixel() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"hi");
    grid.process(SIXEL);
    assert!(!grid.snapshot().windows(3).any(|bytes| bytes == b"\x1bPq"));
}

#[test]
fn device_attributes_without_sixel() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"\x1b[c");
    assert_eq!(grid.take_write_back(), b"\x1b[?62;22c");
}

#[test]
fn device_attributes_with_sixel() {
    let mut grid = Grid::new(SIZE);
    grid.set_sixel_supported(true);
    grid.process(b"\x1b[c");
    assert_eq!(grid.take_write_back(), b"\x1b[?62;4;22c");
    grid.set_sixel_supported(false);
    grid.process(b"\x1b[0c");
    assert_eq!(grid.take_write_back(), b"\x1b[?62;22c");
}

#[test]
fn cell_size_replies() {
    let mut grid = Grid::new(Size::new(80, 24));
    grid.set_cell_size(Some(CellSize::new(10, 20)));
    grid.process(b"\x1b[16t\x1b[14t");
    assert_eq!(grid.take_write_back(), b"\x1b[6;20;10t\x1b[4;480;800t");
    assert_eq!(grid.contents(), "");
}

#[test]
fn cell_size_unset_answers_nothing() {
    let mut grid = Grid::new(Size::new(80, 24));
    grid.process(b"\x1b[16t\x1b[14t");
    assert!(grid.take_write_back().is_empty());
}
