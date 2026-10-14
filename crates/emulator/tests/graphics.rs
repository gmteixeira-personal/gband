use gband_core::geometry::Size;
use gband_emulator::{Emulator, GRAPHICS_LIMIT, Grid, query_reply};

const SIZE: Size = Size::new(20, 4);

const TRANSMIT: &[u8] = b"\x1b_Gf=24,s=1,v=1,a=T;AAAA\x1b\\";

#[test]
fn transmit_sequence_is_collected() {
    let mut grid = Grid::new(SIZE);
    grid.process(TRANSMIT);
    assert_eq!(grid.take_graphics(), vec![TRANSMIT.to_vec()]);
    assert_eq!(grid.contents(), "");
    assert_eq!(grid.cursor(), Some((0, 0)));
}

#[test]
fn sequence_split_across_calls() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"\x1b_Gf=24,s=1,v=1,");
    grid.process(b"a=T;AAAA\x1b");
    assert!(grid.take_graphics().is_empty());
    grid.process(b"\\");
    assert_eq!(grid.take_graphics(), vec![TRANSMIT.to_vec()]);
}

#[test]
fn introducer_split_across_calls() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"x\x1b");
    grid.process(b"_");
    grid.process(b"Ga=T;AAAA\x1b\\");
    assert_eq!(grid.take_graphics(), vec![b"\x1b_Ga=T;AAAA\x1b\\".to_vec()]);
    assert_eq!(grid.contents(), "x");
}

#[test]
fn collected_sequences_are_cleared_when_read() {
    let mut grid = Grid::new(SIZE);
    grid.process(TRANSMIT);
    assert_eq!(grid.take_graphics().len(), 1);
    assert!(grid.take_graphics().is_empty());
}

#[test]
fn sequences_are_reported_in_order() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"\x1b_Ga=T;A\x1b\\text\x1b_Ga=T;B\x1b\\");
    assert_eq!(
        grid.take_graphics(),
        vec![b"\x1b_Ga=T;A\x1b\\".to_vec(), b"\x1b_Ga=T;B\x1b\\".to_vec()]
    );
    assert_eq!(grid.contents(), "text");
}

#[test]
fn non_graphics_apc_string_is_ignored() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"\x1b_qMODE\x1b\\");
    assert!(grid.take_graphics().is_empty());
}

#[test]
fn sequence_past_the_limit_is_discarded() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"\x1b_Ga=T;");
    grid.process(&vec![b'A'; GRAPHICS_LIMIT]);
    grid.process(b"\x1b\\");
    assert!(grid.take_graphics().is_empty());
    grid.process(TRANSMIT);
    assert_eq!(grid.take_graphics(), vec![TRANSMIT.to_vec()]);
}

#[test]
fn snapshot_holds_no_graphics() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"hi");
    grid.process(TRANSMIT);
    assert!(!grid.snapshot().windows(3).any(|bytes| bytes == b"\x1b_G"));
}

#[test]
fn query_reply_echoes_the_id() {
    assert_eq!(
        query_reply(b"\x1b_Gi=31,s=1,v=1,a=q;AAAA\x1b\\"),
        Some(b"\x1b_Gi=31;OK\x1b\\".to_vec())
    );
}

#[test]
fn query_reply_without_an_id() {
    assert_eq!(
        query_reply(b"\x1b_Ga=q,s=1,v=1;AAAA\x1b\\"),
        Some(b"\x1b_Gi=0;OK\x1b\\".to_vec())
    );
}

#[test]
fn query_answer_precedes_a_later_device_attributes_reply() {
    let mut silent = Grid::new(SIZE);
    silent.process(b"\x1b[c");
    let attributes = silent.take_write_back();
    assert!(!attributes.is_empty());

    let mut grid = Grid::new(SIZE);
    grid.answer_graphics_queries(true);
    grid.process(b"\x1b_Gi=31,a=q;AAAA\x1b\\\x1b[c");
    let mut expected = b"\x1b_Gi=31;OK\x1b\\".to_vec();
    expected.extend_from_slice(&attributes);
    assert_eq!(grid.take_write_back(), expected);
}

#[test]
fn query_is_not_answered_by_default() {
    let mut grid = Grid::new(SIZE);
    grid.process(b"\x1b_Gi=31,a=q;AAAA\x1b\\");
    assert!(grid.take_write_back().is_empty());
    assert_eq!(grid.take_graphics().len(), 1);
}

#[test]
fn query_reply_ignores_other_actions() {
    assert_eq!(query_reply(TRANSMIT), None);
}
