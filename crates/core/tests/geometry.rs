use gband_core::geometry::{Size, Span, Tile, column_spans, column_width, pane_heights, tiles};
use gband_core::layout::{
    Band, BandId, Column, Direction, Layout, LayoutOptions, PaneHeight, PaneId, Proportion,
    SessionAction, Weight,
};

const AREA: Size = Size::new(80, 24);

fn single() -> (Layout, PaneId) {
    let mut layout = Layout::new();
    let pane = layout.allocate_pane();
    let band = layout.bands()[0].id;
    layout.open(pane, band, None, &LayoutOptions::default());
    (layout, pane)
}

fn open_after(layout: &mut Layout, after: PaneId) -> PaneId {
    let pane = layout.allocate_pane();
    let band = layout.bands()[0].id;
    layout.open(pane, band, Some(after), &LayoutOptions::default());
    pane
}

fn first_tiles(layout: &Layout, area: Size) -> Vec<Tile> {
    tiles(&layout.bands()[0], area)
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
    layout.apply(
        SessionAction::CycleWidth(first),
        AREA,
        &LayoutOptions::default(),
    );
    layout.apply(
        SessionAction::CycleWidth(first),
        AREA,
        &LayoutOptions::default(),
    );
    layout.apply(
        SessionAction::CycleWidth(second),
        AREA,
        &LayoutOptions::default(),
    );
    let spans = column_spans(&layout.bands()[0], Size::new(90, 30));
    assert_eq!((spans[0].x, spans[0].end()), (0, 30));
    assert_eq!((spans[1].x, spans[1].end()), (30, 90));
    let tiles = first_tiles(&layout, Size::new(90, 30));
    assert_eq!((tiles[1].x, tiles[1].width), (30, 60));
}

#[test]
fn stack_with_a_leftover_row() {
    let (mut layout, first) = single();
    let second = open_after(&mut layout, first);
    layout.apply(
        SessionAction::ConsumeOrExpel {
            pane: second,
            direction: Direction::Left,
        },
        AREA,
        &LayoutOptions::default(),
    );
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
    layout.apply(
        SessionAction::ToggleFullWidth(first),
        AREA,
        &LayoutOptions::default(),
    );
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
    let mut column = Column::new(PaneId(1), Proportion::ONE_HALF);
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
        layout.apply(
            SessionAction::ConsumeOrExpel {
                pane,
                direction: Direction::Left,
            },
            AREA,
            &LayoutOptions::default(),
        );
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
    assert_eq!(tiles[1].span(), column_spans(&layout.bands()[0], area)[1]);
}

#[test]
fn empty_band_has_no_tiles() {
    let (layout, _) = single();
    assert!(tiles(&layout.bands()[1], Size::new(80, 24)).is_empty());
}

fn column(heights: &[PaneHeight]) -> Column {
    let mut column = Column::new(PaneId(1), Proportion::ONE_HALF);
    column.panes = (1..=heights.len() as u32).map(PaneId).collect();
    column.heights = heights.to_vec();
    column
}

fn band_of(columns: Vec<Column>) -> Band {
    Band {
        id: BandId(1),
        columns,
    }
}

fn rows(heights: &[PaneHeight]) -> Vec<(u16, u16)> {
    tiles(&band_of(vec![column(heights)]), AREA)
        .iter()
        .map(|tile| (tile.y, tile.height))
        .collect()
}

fn auto(num: u16, den: u16) -> PaneHeight {
    PaneHeight::Auto(Weight::new(num, den))
}

#[test]
fn fixed_pane_above_an_automatic_pane() {
    assert_eq!(
        rows(&[PaneHeight::Fixed(16), auto(1, 1)]),
        [(0, 16), (16, 8)]
    );
}

#[test]
fn automatic_panes_share_by_weight() {
    assert_eq!(
        rows(&[auto(10, 7), auto(1, 1), PaneHeight::Fixed(9)]),
        [(0, 9), (9, 6), (15, 9)]
    );
}

#[test]
fn fixed_height_leaves_room_for_the_others() {
    assert_eq!(
        rows(&[PaneHeight::Fixed(30), auto(1, 1), auto(1, 1)]),
        [(0, 18), (18, 3), (21, 3)]
    );
}

#[test]
fn automatic_pane_raised_to_three_rows() {
    assert_eq!(rows(&[auto(1, 20), auto(1, 1)]), [(0, 3), (3, 21)]);
}

#[test]
fn lone_fixed_pane_leaves_rows_uncovered() {
    assert_eq!(rows(&[PaneHeight::Fixed(20)]), [(0, 20)]);
    assert_eq!(rows(&[PaneHeight::Fixed(30)]), [(0, 24)]);
}

#[test]
fn weight_one_heights_split_as_before() {
    let heights = pane_heights(&column(&[auto(1, 1); 3]), 25);
    assert_eq!(heights, [9, 8, 8]);
}

fn column_of_width(width: Proportion) -> Column {
    let mut column = Column::new(PaneId(1), Proportion::ONE_HALF);
    column.width = width;
    column
}

#[test]
fn column_wider_than_the_area() {
    let column = column_of_width(Proportion::new(11, 10));
    assert_eq!(column_width(&column, AREA), 88);
    let tiles = tiles(&band_of(vec![column]), AREA);
    assert_eq!(tiles[0].width, 88);
}

#[test]
fn column_of_width_zero_is_three_cells() {
    assert_eq!(
        column_width(&column_of_width(Proportion::new(0, 1)), AREA),
        3
    );
}

#[test]
fn column_width_stops_at_the_cell_limit() {
    let column = column_of_width(Proportion::new(Proportion::MAX, 1));
    assert_eq!(column_width(&column, AREA), u16::MAX);
    let spans = column_spans(&band_of(vec![column.clone(), column]), AREA);
    assert_eq!((spans[1].x, spans[1].end()), (65535, 131070));
}
