use gband_core::geometry::{
    Size, Span, Tile, WindowBox, boxes, column_spans, column_width, drawn_copy, height_step,
    loop_width, placed, tiles, width_step, window_heights,
};
use gband_core::layout::{
    Band, BandId, Column, Direction, FloatingWindow, Layout, LayoutOptions, Proportion,
    SessionAction, Weight, WindowHeight, WindowId,
};

const AREA: Size = Size::new(80, 24);

fn single() -> (Layout, WindowId) {
    let mut layout = Layout::new();
    let window = layout.allocate_window();
    let band = layout.bands()[0].id;
    layout.open(window, band, None, None, &LayoutOptions::default());
    (layout, window)
}

fn open_after(layout: &mut Layout, after: WindowId) -> WindowId {
    let window = layout.allocate_window();
    let band = layout.bands()[0].id;
    layout.open(window, band, Some(after), None, &LayoutOptions::default());
    window
}

fn first_tiles(layout: &Layout, area: Size) -> Vec<Tile> {
    tiles(&layout.bands()[0], area)
}

#[test]
fn default_column_on_an_80_by_24_area() {
    let (layout, window) = single();
    let tiles = first_tiles(&layout, Size::new(80, 24));
    assert_eq!(
        tiles,
        vec![Tile {
            window,
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
            window: second,
            direction: Direction::Left,
        },
        AREA,
        &LayoutOptions::default(),
    );
    let tiles = first_tiles(&layout, Size::new(80, 25));
    assert_eq!(
        (tiles[0].window, tiles[0].y, tiles[0].height),
        (first, 0, 13)
    );
    assert_eq!(
        (tiles[1].window, tiles[1].y, tiles[1].height),
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
    let mut column = Column::new(WindowId(1), Proportion::ONE_HALF);
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
fn windows_beyond_the_area_rows_get_no_height() {
    let (mut layout, first) = single();
    let second = open_after(&mut layout, first);
    let third = open_after(&mut layout, second);
    for window in [second, third] {
        layout.apply(
            SessionAction::ConsumeOrExpel {
                window,
                direction: Direction::Left,
            },
            AREA,
            &LayoutOptions::default(),
        );
    }
    let tiles = first_tiles(&layout, Size::new(80, 2));
    let rows: Vec<_> = tiles
        .iter()
        .map(|tile| (tile.window, tile.y, tile.height))
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
    assert_eq!(tiles[1].window, second);
    assert_eq!(tiles[1].span(), Span { x: 40, width: 40 });
    assert_eq!(tiles[1].span(), column_spans(&layout.bands()[0], area)[1]);
}

#[test]
fn empty_band_has_no_tiles() {
    let (layout, _) = single();
    assert!(tiles(&layout.bands()[1], Size::new(80, 24)).is_empty());
}

fn column(heights: &[WindowHeight]) -> Column {
    let mut column = Column::new(WindowId(1), Proportion::ONE_HALF);
    column.windows = (1..=heights.len() as u32).map(WindowId).collect();
    column.heights = heights.to_vec();
    column
}

fn band_of(columns: Vec<Column>) -> Band {
    Band {
        id: BandId(1),
        columns,
        floating: Vec::new(),
    }
}

fn rows(heights: &[WindowHeight]) -> Vec<(u16, u16)> {
    tiles(&band_of(vec![column(heights)]), AREA)
        .iter()
        .map(|tile| (tile.y, tile.height))
        .collect()
}

fn auto(num: u32, den: u32) -> WindowHeight {
    WindowHeight::Auto(Weight::new(num, den))
}

#[test]
fn fixed_window_above_an_automatic_window() {
    assert_eq!(
        rows(&[WindowHeight::Fixed(16), auto(1, 1)]),
        [(0, 16), (16, 8)]
    );
}

#[test]
fn automatic_windows_share_by_weight() {
    assert_eq!(
        rows(&[auto(10, 7), auto(1, 1), WindowHeight::Fixed(9)]),
        [(0, 9), (9, 6), (15, 9)]
    );
}

#[test]
fn fixed_height_leaves_room_for_the_others() {
    assert_eq!(
        rows(&[WindowHeight::Fixed(30), auto(1, 1), auto(1, 1)]),
        [(0, 18), (18, 3), (21, 3)]
    );
}

#[test]
fn automatic_window_raised_to_three_rows() {
    assert_eq!(rows(&[auto(1, 20), auto(1, 1)]), [(0, 3), (3, 21)]);
}

#[test]
fn lone_fixed_window_leaves_rows_uncovered() {
    assert_eq!(rows(&[WindowHeight::Fixed(20)]), [(0, 20)]);
    assert_eq!(rows(&[WindowHeight::Fixed(30)]), [(0, 24)]);
}

#[test]
fn weight_one_heights_split_as_before() {
    let heights = window_heights(&column(&[auto(1, 1); 3]), 25);
    assert_eq!(heights, [9, 8, 8]);
}

fn column_of_width(width: Proportion) -> Column {
    let mut column = Column::new(WindowId(1), Proportion::ONE_HALF);
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

fn floating(col: u16, row: u16, width: Proportion, rows: u16) -> FloatingWindow {
    FloatingWindow {
        window: WindowId(1),
        col,
        row,
        width,
        full_width: false,
        rows,
    }
}

fn cells(placed: WindowBox) -> ((u16, u16), (u16, u16)) {
    (
        (placed.x, placed.x + placed.width - 1),
        (placed.y, placed.y + placed.height - 1),
    )
}

#[test]
fn box_in_an_80_by_24_area() {
    let placed = placed(&floating(10, 4, Proportion::ONE_HALF, 12), AREA);
    assert_eq!(cells(placed), ((10, 49), (4, 15)));
    assert_eq!(placed.terminal_size(), Size::new(38, 10));
}

#[test]
fn box_pushed_back_inside_a_smaller_area() {
    let placed = placed(&floating(50, 4, Proportion::ONE_HALF, 12), AREA);
    assert_eq!(cells(placed).0, (40, 79));
}

#[test]
fn box_taller_than_the_area() {
    let placed = placed(&floating(0, 5, Proportion::ONE_HALF, 30), AREA);
    assert_eq!(cells(placed).1, (0, 23));
}

#[test]
fn box_returns_when_the_area_grows_back() {
    let record = floating(50, 0, Proportion::ONE_HALF, 10);
    assert_eq!(placed(&record, Size::new(120, 40)).x, 50);
    assert_eq!(placed(&record, AREA).x, 40);
    assert_eq!(placed(&record, Size::new(120, 40)).x, 50);
}

#[test]
fn full_width_box_fills_the_area() {
    let mut record = floating(10, 0, Proportion::ONE_THIRD, 10);
    record.full_width = true;
    assert_eq!(cells(placed(&record, AREA)).0, (0, 79));
}

#[test]
fn box_is_at_least_three_cells_wide() {
    let placed = placed(&floating(0, 0, Proportion::new(0, 1), 3), AREA);
    assert_eq!(placed.width, 3);
    assert_eq!(placed.terminal_size(), Size::new(1, 1));
}

#[test]
fn steps_are_a_tenth_rounded_half_up() {
    assert_eq!(height_step(Size::new(80, 24)), 2);
    assert_eq!(height_step(Size::new(80, 25)), 3);
    assert_eq!(height_step(Size::new(80, 4)), 1);
    assert_eq!(width_step(Size::new(80, 24)), 8);
    assert_eq!(width_step(Size::new(85, 24)), 9);
}

#[test]
fn band_boxes_follow_the_floating_list() {
    let mut band = band_of(Vec::new());
    band.floating = vec![
        floating(0, 0, Proportion::ONE_HALF, 5),
        FloatingWindow {
            window: WindowId(2),
            ..floating(30, 2, Proportion::ONE_THIRD, 5)
        },
    ];
    let placed: Vec<_> = boxes(&band, AREA)
        .iter()
        .map(|placed| placed.window)
        .collect();
    assert_eq!(placed, [WindowId(1), WindowId(2)]);
}

fn spans_of(widths: &[u16]) -> Vec<Span> {
    let mut x = 0;
    widths
        .iter()
        .map(|&width| {
            let span = Span { x, width };
            x = span.end();
            span
        })
        .collect()
}

#[test]
fn strip_loops_when_it_outlasts_the_terminal_by_its_widest_column() {
    assert_eq!(loop_width(&spans_of(&[40, 40, 40]), 80), Some(120));
    assert_eq!(loop_width(&spans_of(&[39, 39, 39]), 79), Some(117));
    assert_eq!(loop_width(&spans_of(&[39, 39, 39]), 80), None);
    assert_eq!(loop_width(&spans_of(&[40, 40]), 80), None);
    assert_eq!(loop_width(&spans_of(&[40, 30, 50]), 80), None);
    assert_eq!(loop_width(&[], 80), None);
}

#[test]
fn drawn_copy_is_the_one_inside_the_terminal() {
    assert_eq!(drawn_copy(-80, 120, 80), 40);
    assert_eq!(drawn_copy(40, 120, 80), 40);
    assert_eq!(drawn_copy(80, 120, 80), -40);
    assert_eq!(drawn_copy(-20, 120, 80), -20);
}
