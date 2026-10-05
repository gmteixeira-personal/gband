use gband_core::geometry::{Size, Span, Tile, column_spans, column_width, tiles};
use gband_core::layout::{Column, Direction, Layout, PaneId, Proportion, SessionAction};

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
fn column_width_follows_the_proportion_and_the_minimum() {
    let mut column = Column::new(PaneId(1));
    let area = Size::new(90, 24);
    assert_eq!(column_width(&column, area), 45);
    column.width = Proportion::ONE_THIRD;
    assert_eq!(column_width(&column, area), 30);
    column.full_width = true;
    assert_eq!(column_width(&column, area), 90);
    assert_eq!(column_width(&column, Size::new(2, 24)), 3);
    column.full_width = false;
    assert_eq!(column_width(&column, Size::new(5, 24)), 3);
}

#[test]
fn panes_beyond_the_area_rows_get_no_height() {
    let (mut layout, first) = single();
    let second = open_after(&mut layout, first);
    let third = open_after(&mut layout, second);
    for pane in [second, third] {
        layout.apply(SessionAction::ConsumeOrExpel {
            pane,
            direction: Direction::Left,
        });
    }
    let tiles = first_tiles(&layout, Size::new(80, 2));
    let rows: Vec<_> = tiles
        .iter()
        .map(|tile| (tile.pane, tile.y, tile.height))
        .collect();
    assert_eq!(rows, vec![(first, 0, 1), (second, 1, 1), (third, 2, 0)]);
    assert_eq!(tiles[2].terminal_size(), Size::new(38, 1));
}

#[test]
fn tile_span_is_its_column_span() {
    let (mut layout, first) = single();
    let second = open_after(&mut layout, first);
    let area = Size::new(80, 24);
    let tiles = first_tiles(&layout, area);
    assert_eq!(tiles[1].pane, second);
    assert_eq!(tiles[1].span(), Span { x: 40, width: 40 });
    assert_eq!(
        tiles[1].span(),
        column_spans(&layout.workspaces()[0], area)[1]
    );
}

#[test]
fn empty_workspace_has_no_tiles() {
    let (layout, _) = single();
    assert!(tiles(&layout.workspaces()[1], Size::new(80, 24)).is_empty());
}
