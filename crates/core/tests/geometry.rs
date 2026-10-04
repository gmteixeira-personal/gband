use gband_core::geometry::{Size, Tile, column_spans, tiles};
use gband_core::layout::{Direction, Layout, PaneId, SessionAction};

fn single() -> (Layout, PaneId) {
    let mut layout = Layout::new();
    let pane = layout.allocate_pane();
    let workspace = layout.workspaces()[0].id;
    layout.open(pane, workspace, None);
    (layout, pane)
}

fn open_after(layout: &mut Layout, after: PaneId) -> PaneId {
    let pane = layout.allocate_pane();
    let workspace = layout.workspaces()[0].id;
    layout.open(pane, workspace, Some(after));
    pane
}

fn first_tiles(layout: &Layout, area: Size) -> Vec<Tile> {
    tiles(&layout.workspaces()[0], area)
}

#[test]
fn default_column_on_an_80_by_24_area() {
    let (layout, pane) = single();
    let tiles = first_tiles(&layout, Size::new(80, 24));
    assert_eq!(
        tiles,
        vec![Tile {
            pane,
            column: 0,
            row: 0,
            x: 0,
            y: 0,
            width: 40,
            height: 24,
        }]
    );
    assert_eq!(tiles[0].terminal_size(), Size::new(38, 22));
}

#[test]
fn columns_side_by_side() {
    let (mut layout, first) = single();
    let second = open_after(&mut layout, first);
    layout.apply(SessionAction::CycleWidth(first));
    layout.apply(SessionAction::CycleWidth(first));
    layout.apply(SessionAction::CycleWidth(second));
    let spans = column_spans(&layout.workspaces()[0], Size::new(90, 30));
    assert_eq!((spans[0].x, spans[0].end()), (0, 30));
    assert_eq!((spans[1].x, spans[1].end()), (30, 90));
    let tiles = first_tiles(&layout, Size::new(90, 30));
    assert_eq!((tiles[1].x, tiles[1].width), (30, 60));
}

#[test]
fn stack_with_a_leftover_row() {
    let (mut layout, first) = single();
    let second = open_after(&mut layout, first);
    layout.apply(SessionAction::ConsumeOrExpel {
        pane: second,
        direction: Direction::Left,
    });
    let tiles = first_tiles(&layout, Size::new(80, 25));
    assert_eq!((tiles[0].pane, tiles[0].y, tiles[0].height), (first, 0, 13));
    assert_eq!(
        (tiles[1].pane, tiles[1].y, tiles[1].height),
        (second, 13, 12)
    );
    assert_eq!(tiles[1].terminal_size(), Size::new(38, 10));
}

#[test]
fn full_width_fills_the_area() {
    let (mut layout, first) = single();
    layout.apply(SessionAction::ToggleFullWidth(first));
    let tiles = first_tiles(&layout, Size::new(100, 30));
    assert_eq!(tiles[0].width, 100);
}

#[test]
fn columns_are_at_least_three_cells_wide() {
    let (layout, _) = single();
    let tiles = first_tiles(&layout, Size::new(4, 24));
    assert_eq!(tiles[0].width, 3);
    assert_eq!(tiles[0].terminal_size(), Size::new(1, 22));
}

#[test]
fn terminal_size_is_at_least_one_cell() {
    let (layout, _) = single();
    let tiles = first_tiles(&layout, Size::new(2, 1));
    assert_eq!(tiles[0].terminal_size(), Size::new(1, 1));
}

#[test]
fn empty_workspace_has_no_tiles() {
    let (layout, _) = single();
    assert!(tiles(&layout.workspaces()[1], Size::new(80, 24)).is_empty());
}
