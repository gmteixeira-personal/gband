use std::collections::HashSet;

use gband_core::event::LayoutEvent;
use gband_core::geometry::{Size, placed, window_heights};
use gband_core::layout::Step::{Grow, Shrink};
use gband_core::layout::{
    BandId, Column, Direction, FloatingWindow, Layout, LayoutOptions, Location, Place, Proportion,
    SessionAction, TargetPlace, Vertical, Weight, WindowHeight, WindowId,
};

const AREA: Size = Size::new(80, 24);

fn columns(layout: &Layout, band: usize) -> Vec<Vec<WindowId>> {
    layout.bands()[band]
        .columns
        .iter()
        .map(|column| column.windows.clone())
        .collect()
}

fn band_id(layout: &Layout, index: usize) -> BandId {
    layout.bands()[index].id
}

fn open(layout: &mut Layout, band: usize, after: Option<WindowId>) -> WindowId {
    let window = layout.allocate_window();
    let id = band_id(layout, band);
    assert!(
        !layout
            .open(window, id, after, None, &LayoutOptions::default())
            .is_empty()
    );
    window
}

fn started() -> (Layout, WindowId) {
    let mut layout = Layout::new();
    let first = open(&mut layout, 0, None);
    (layout, first)
}

fn row_of_columns(count: usize) -> (Layout, Vec<WindowId>) {
    let (mut layout, first) = started();
    let mut windows = vec![first];
    for _ in 1..count {
        let last = *windows.last().unwrap();
        windows.push(open(&mut layout, 0, Some(last)));
    }
    (layout, windows)
}

fn width_of(layout: &Layout, window: WindowId) -> (Proportion, bool) {
    let location = layout.locate(window).unwrap();
    let column = &layout.bands()[location.band].columns[location.column];
    (column.width, column.full_width)
}

fn assert_invariants(layout: &Layout) {
    let bands = layout.bands();
    assert!(!bands.is_empty());
    let (last, rest) = bands.split_last().unwrap();
    assert!(last.is_empty(), "the last band must be empty");
    assert!(rest.iter().all(|band| !band.is_empty()));
    for band in bands {
        assert!(band.columns.iter().all(|column| !column.windows.is_empty()));
        assert!(
            band.columns
                .iter()
                .all(|column| column.windows.len() == column.heights.len()),
            "a column's heights differ in length from its windows"
        );
    }
    let windows: Vec<_> = layout.windows().collect();
    let unique: HashSet<_> = windows.iter().collect();
    assert_eq!(windows.len(), unique.len(), "a window appears twice");
}

fn shift(layout: &mut Layout, window: WindowId, direction: Direction) -> Vec<LayoutEvent> {
    let events = layout.apply(
        SessionAction::ConsumeOrExpel { window, direction },
        AREA,
        &LayoutOptions::default(),
    );
    assert_invariants(layout);
    events
}

#[test]
fn new_layout_holds_one_empty_band() {
    let layout = Layout::new();
    assert_eq!(layout.bands().len(), 1);
    assert!(layout.is_empty());
}

#[test]
fn initial_layout_has_the_first_window_and_an_empty_band() {
    let (layout, first) = started();
    assert_eq!(layout.bands().len(), 2);
    assert_eq!(columns(&layout, 0), vec![vec![first]]);
    assert!(layout.bands()[1].is_empty());
    assert_eq!(width_of(&layout, first), (Proportion::ONE_HALF, false));
}

#[test]
fn identifiers_are_not_reused() {
    let (mut layout, windows) = row_of_columns(2);
    let first_band = band_id(&layout, 0);
    assert!(!layout.remove(windows[1]).is_empty());
    let next = open(&mut layout, 0, Some(windows[0]));
    assert_ne!(next, windows[1]);

    assert!(!layout.remove(windows[0]).is_empty());
    assert!(!layout.remove(next).is_empty());
    open(&mut layout, 0, None);
    assert_ne!(band_id(&layout, 0), first_band);
}

#[test]
fn opens_right_of_the_named_column() {
    let (mut layout, windows) = row_of_columns(2);
    let a = windows[0];
    let b = windows[1];
    assert!(
        !layout
            .apply(
                SessionAction::CycleWidth(b),
                AREA,
                &LayoutOptions::default()
            )
            .is_empty()
    );
    let new = open(&mut layout, 0, Some(a));
    assert_eq!(columns(&layout, 0), vec![vec![a], vec![new], vec![b]]);
    assert_eq!(width_of(&layout, a), (Proportion::ONE_HALF, false));
    assert_eq!(width_of(&layout, b), (Proportion::TWO_THIRDS, false));
}

#[test]
fn opens_as_the_first_column_without_a_named_window() {
    let (mut layout, first) = started();
    let new = open(&mut layout, 0, None);
    assert_eq!(columns(&layout, 0), vec![vec![new], vec![first]]);
}

#[test]
fn opening_in_the_empty_band_adds_another() {
    let (mut layout, first) = started();
    let w1 = band_id(&layout, 0);
    let w2 = band_id(&layout, 1);
    let new = open(&mut layout, 1, None);
    assert_eq!(layout.bands().len(), 3);
    assert_eq!(band_id(&layout, 0), w1);
    assert_eq!(band_id(&layout, 1), w2);
    assert_eq!(columns(&layout, 0), vec![vec![first]]);
    assert_eq!(columns(&layout, 1), vec![vec![new]]);
    assert!(layout.bands()[2].is_empty());
}

#[test]
fn open_refuses_unknown_targets() {
    let (mut layout, first) = started();
    let window = layout.allocate_window();
    assert!(
        layout
            .open(window, BandId(99), None, None, &LayoutOptions::default())
            .is_empty()
    );
    let second = band_id(&layout, 1);
    assert!(
        layout
            .open(window, second, Some(first), None, &LayoutOptions::default())
            .is_empty()
    );
    assert!(
        layout
            .open(
                window,
                second,
                Some(WindowId(99)),
                None,
                &LayoutOptions::default()
            )
            .is_empty()
    );
    assert_eq!(columns(&layout, 0), vec![vec![first]]);
}

#[test]
fn column_is_removed_with_its_last_window() {
    let (mut layout, windows) = row_of_columns(3);
    assert!(!layout.remove(windows[1]).is_empty());
    assert_eq!(
        columns(&layout, 0),
        vec![vec![windows[0]], vec![windows[2]]]
    );
}

#[test]
fn window_leaves_a_stack_and_the_column_keeps_its_width() {
    let (mut layout, windows) = row_of_columns(3);
    let (p1, p2, p3) = (windows[0], windows[1], windows[2]);
    shift(&mut layout, p2, Direction::Left);
    shift(&mut layout, p3, Direction::Left);
    layout.apply(
        SessionAction::CycleWidth(p1),
        AREA,
        &LayoutOptions::default(),
    );
    assert_eq!(columns(&layout, 0), vec![vec![p1, p2, p3]]);
    assert!(!layout.remove(p2).is_empty());
    assert_eq!(columns(&layout, 0), vec![vec![p1, p3]]);
    assert_eq!(width_of(&layout, p1), (Proportion::TWO_THIRDS, false));
}

#[test]
fn emptied_middle_band_is_removed() {
    let (mut layout, first) = started();
    let w2 = band_id(&layout, 1);
    let other = open(&mut layout, 1, None);
    let w3 = band_id(&layout, 2);
    assert!(!layout.remove(first).is_empty());
    assert_eq!(layout.bands().len(), 2);
    assert_eq!(band_id(&layout, 0), w2);
    assert_eq!(band_id(&layout, 1), w3);
    assert_eq!(columns(&layout, 0), vec![vec![other]]);
}

#[test]
fn removing_the_last_window_leaves_one_empty_band() {
    let (mut layout, first) = started();
    assert!(!layout.remove(first).is_empty());
    assert_eq!(layout.bands().len(), 1);
    assert!(layout.is_empty());
    assert!(layout.remove(first).is_empty());
}

#[test]
fn expel_to_the_left() {
    let (mut layout, windows) = row_of_columns(3);
    let (a, p1, p2) = (windows[0], windows[1], windows[2]);
    shift(&mut layout, p2, Direction::Left);
    assert_eq!(columns(&layout, 0), vec![vec![a], vec![p1, p2]]);
    assert!(!shift(&mut layout, p2, Direction::Left).is_empty());
    assert_eq!(columns(&layout, 0), vec![vec![a], vec![p2], vec![p1]]);
    assert_eq!(width_of(&layout, p2), (Proportion::ONE_HALF, false));
}

#[test]
fn expel_to_the_right() {
    let (mut layout, windows) = row_of_columns(2);
    let (p1, p2) = (windows[0], windows[1]);
    shift(&mut layout, p2, Direction::Left);
    layout.apply(
        SessionAction::CycleWidth(p1),
        AREA,
        &LayoutOptions::default(),
    );
    assert!(!shift(&mut layout, p1, Direction::Right).is_empty());
    assert_eq!(columns(&layout, 0), vec![vec![p2], vec![p1]]);
    assert_eq!(width_of(&layout, p1), (Proportion::ONE_HALF, false));
    assert_eq!(width_of(&layout, p2), (Proportion::TWO_THIRDS, false));
}

#[test]
fn consume_to_the_right_keeps_the_target_width() {
    let (mut layout, windows) = row_of_columns(3);
    let (p1, p2, p3) = (windows[0], windows[1], windows[2]);
    shift(&mut layout, p3, Direction::Left);
    layout.apply(
        SessionAction::CycleWidth(p2),
        AREA,
        &LayoutOptions::default(),
    );
    assert!(!shift(&mut layout, p1, Direction::Right).is_empty());
    assert_eq!(columns(&layout, 0), vec![vec![p2, p3, p1]]);
    assert_eq!(width_of(&layout, p1), (Proportion::TWO_THIRDS, false));
}

#[test]
fn consume_to_the_left() {
    let (mut layout, windows) = row_of_columns(3);
    let (a, b, c) = (windows[0], windows[1], windows[2]);
    assert!(!shift(&mut layout, c, Direction::Left).is_empty());
    assert_eq!(columns(&layout, 0), vec![vec![a], vec![b, c]]);
}

#[test]
fn consume_at_the_edge_changes_nothing() {
    let (mut layout, windows) = row_of_columns(2);
    let before = layout.clone();
    assert!(shift(&mut layout, windows[0], Direction::Left).is_empty());
    assert!(shift(&mut layout, windows[1], Direction::Right).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn width_cycles_through_the_presets() {
    let (mut layout, first) = started();
    let mut seen = Vec::new();
    for _ in 0..3 {
        layout.apply(
            SessionAction::CycleWidth(first),
            AREA,
            &LayoutOptions::default(),
        );
        seen.push(width_of(&layout, first).0);
    }
    assert_eq!(
        seen,
        vec![
            Proportion::TWO_THIRDS,
            Proportion::ONE_THIRD,
            Proportion::ONE_HALF
        ]
    );
}

fn presets(presets: &[Proportion]) -> LayoutOptions {
    LayoutOptions {
        presets: presets.to_vec(),
        ..LayoutOptions::default()
    }
}

#[test]
fn width_cycles_through_configured_presets() {
    let options = presets(&[
        Proportion::new(1, 4),
        Proportion::ONE_HALF,
        Proportion::new(3, 4),
    ]);
    let (mut layout, first) = started();
    let mut seen = Vec::new();
    for _ in 0..2 {
        layout.apply(SessionAction::CycleWidth(first), AREA, &options);
        seen.push(width_of(&layout, first).0);
    }
    assert_eq!(seen, [Proportion::new(3, 4), Proportion::new(1, 4)]);
}

#[test]
fn width_cycles_from_between_presets() {
    let options = presets(&[Proportion::new(1, 4), Proportion::new(3, 4)]);
    let (mut layout, first) = started();
    layout.apply(SessionAction::CycleWidth(first), AREA, &options);
    assert_eq!(width_of(&layout, first), (Proportion::new(3, 4), false));
}

#[test]
fn new_column_takes_the_configured_default_width() {
    let options = LayoutOptions {
        default_width: Proportion::ONE_THIRD,
        ..LayoutOptions::default()
    };
    let mut layout = Layout::new();
    let window = layout.allocate_window();
    let band = band_id(&layout, 0);
    assert!(!layout.open(window, band, None, None, &options).is_empty());
    assert_eq!(width_of(&layout, window), (Proportion::ONE_THIRD, false));
}

#[test]
fn expelled_column_takes_the_configured_default_width() {
    let (mut layout, first) = started();
    let second = open(&mut layout, 0, Some(first));
    shift(&mut layout, second, Direction::Left);
    let options = LayoutOptions {
        default_width: Proportion::ONE_THIRD,
        ..LayoutOptions::default()
    };
    layout.apply(
        SessionAction::ConsumeOrExpel {
            window: second,
            direction: Direction::Right,
        },
        AREA,
        &options,
    );
    assert_eq!(width_of(&layout, second), (Proportion::ONE_THIRD, false));
}

#[test]
fn cycling_from_full_width_wraps_to_one_third() {
    let (mut layout, first) = started();
    layout.apply(
        SessionAction::CycleWidth(first),
        AREA,
        &LayoutOptions::default(),
    );
    layout.apply(
        SessionAction::ToggleFullWidth(first),
        AREA,
        &LayoutOptions::default(),
    );
    assert_eq!(width_of(&layout, first), (Proportion::TWO_THIRDS, true));
    layout.apply(
        SessionAction::CycleWidth(first),
        AREA,
        &LayoutOptions::default(),
    );
    assert_eq!(width_of(&layout, first), (Proportion::ONE_THIRD, false));
}

#[test]
fn toggling_full_width_twice_keeps_the_proportion() {
    let (mut layout, first) = started();
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
        SessionAction::ToggleFullWidth(first),
        AREA,
        &LayoutOptions::default(),
    );
    layout.apply(
        SessionAction::ToggleFullWidth(first),
        AREA,
        &LayoutOptions::default(),
    );
    assert_eq!(width_of(&layout, first), (Proportion::ONE_THIRD, false));
}

#[test]
fn actions_on_a_missing_window_change_nothing() {
    let (mut layout, _) = started();
    let before = layout.clone();
    let missing = WindowId(99);
    assert!(
        layout
            .apply(
                SessionAction::CycleWidth(missing),
                AREA,
                &LayoutOptions::default()
            )
            .is_empty()
    );
    assert!(step_width(&mut layout, missing, Grow).is_empty());
    assert!(step_height(&mut layout, missing, Grow, AREA).is_empty());
    assert!(
        layout
            .apply(
                SessionAction::ResetHeight(missing),
                AREA,
                &LayoutOptions::default()
            )
            .is_empty()
    );
    assert!(
        layout
            .apply(
                SessionAction::ToggleFullWidth(missing),
                AREA,
                &LayoutOptions::default()
            )
            .is_empty()
    );
    assert!(shift(&mut layout, missing, Direction::Left).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn opening_a_window_the_layout_already_holds_changes_nothing() {
    let (mut layout, first) = started();
    let before = layout.clone();
    let (current, empty) = (band_id(&layout, 0), band_id(&layout, 1));
    assert!(
        layout
            .open(first, current, None, None, &LayoutOptions::default())
            .is_empty()
    );
    assert!(
        layout
            .open(first, empty, None, None, &LayoutOptions::default())
            .is_empty()
    );
    assert_eq!(layout, before);
}

#[test]
fn opening_and_closing_are_left_to_the_server() {
    let (mut layout, first) = started();
    let before = layout.clone();
    let band = band_id(&layout, 0);
    assert!(
        layout
            .apply(
                SessionAction::open(band, Some(first), None),
                AREA,
                &LayoutOptions::default()
            )
            .is_empty()
    );
    assert!(
        layout
            .apply(
                SessionAction::CloseWindow(first),
                AREA,
                &LayoutOptions::default()
            )
            .is_empty()
    );
    assert_eq!(layout, before);
}

#[test]
fn proportion_of_rounds_down() {
    assert_eq!(Proportion::new(1, 3), Proportion::ONE_THIRD);
    assert_eq!(Proportion::ONE_THIRD.of(80), 26);
    assert_eq!(Proportion::TWO_THIRDS.of(80), 53);
    assert_eq!(Proportion::TWO_THIRDS.of(1), 0);
    assert_eq!(Proportion::TWO_THIRDS.of(u16::MAX), 43690);
    assert_eq!(Proportion::WHOLE.of(u16::MAX), u16::MAX);
}

#[test]
fn first_window_is_the_top_of_the_leftmost_column() {
    let (mut layout, windows) = row_of_columns(2);
    let leftmost = open(&mut layout, 0, None);
    shift(&mut layout, windows[0], Direction::Left);
    assert_eq!(
        columns(&layout, 0),
        vec![vec![leftmost, windows[0]], vec![windows[1]]]
    );
    assert_eq!(layout.bands()[0].first_window(), Some(leftmost));
    assert_eq!(layout.bands()[1].first_window(), None);
}

#[test]
fn band_index_follows_removals() {
    let (mut layout, first) = started();
    let other = open(&mut layout, 1, None);
    let ids = [0, 1, 2].map(|index| band_id(&layout, index));
    assert_eq!(ids.map(|id| layout.band_index(id)), [0, 1, 2].map(Some));
    layout.remove(first);
    assert_eq!(
        ids.map(|id| layout.band_index(id)),
        [None, Some(0), Some(1)]
    );
    assert_eq!(columns(&layout, 0), vec![vec![other]]);
    assert_eq!(layout.band_index(BandId(99)), None);
}

fn stack_of_three() -> (Layout, [WindowId; 3]) {
    let (mut layout, windows) = row_of_columns(3);
    for &window in &windows[1..] {
        shift(&mut layout, window, Direction::Left);
    }
    assert_eq!(columns(&layout, 0), vec![windows.clone()]);
    (layout, [windows[0], windows[1], windows[2]])
}

#[test]
fn expelling_the_middle_of_a_stack_keeps_the_others_stacked_in_order() {
    let (mut layout, [a, b, c]) = stack_of_three();
    shift(&mut layout, b, Direction::Left);
    assert_eq!(columns(&layout, 0), vec![vec![b], vec![a, c]]);

    let (mut layout, [a, b, c]) = stack_of_three();
    shift(&mut layout, b, Direction::Right);
    assert_eq!(columns(&layout, 0), vec![vec![a, c], vec![b]]);
}

#[test]
fn identifiers_display_as_their_number() {
    assert_eq!(WindowId(7).to_string(), "7");
    assert_eq!(BandId(12).to_string(), "12");
}

#[test]
fn default_layout_is_a_new_layout() {
    assert_eq!(Layout::default(), Layout::new());
}

#[derive(Clone, Copy)]
enum Step {
    Open { band: usize, after: usize },
    Remove(usize),
    Shift(usize, Direction),
    Cycle(usize),
    Full(usize),
}

#[test]
fn invariants_hold_through_mixed_operations() {
    use Direction::{Left, Right};
    use Step::*;
    let steps = [
        Open { band: 0, after: 0 },
        Open { band: 0, after: 1 },
        Shift(2, Left),
        Open { band: 1, after: 0 },
        Open { band: 1, after: 3 },
        Shift(1, Right),
        Cycle(4),
        Full(3),
        Shift(3, Left),
        Open { band: 2, after: 9 },
        Remove(0),
        Shift(4, Right),
        Remove(2),
        Shift(5, Left),
        Shift(5, Left),
        Remove(3),
        Open { band: 0, after: 4 },
        Remove(1),
        Remove(4),
        Open { band: 1, after: 5 },
        Remove(5),
        Remove(6),
    ];
    let (mut layout, first) = started();
    let mut windows = vec![first];
    assert_invariants(&layout);
    for step in steps {
        let pick = |index: usize, windows: &[WindowId]| windows[index % windows.len()];
        match step {
            Open { band, after } => {
                let count = layout.bands().len();
                let target = layout.bands()[band % count].id;
                let named = layout.bands()[band % count]
                    .windows()
                    .nth(after % 4)
                    .or_else(|| layout.bands()[band % count].windows().next());
                let window = layout.allocate_window();
                assert!(!windows.contains(&window), "{window} was reused");
                let events = layout.open(window, target, named, None, &LayoutOptions::default());
                assert_eq!(
                    events[0],
                    LayoutEvent::WindowOpened {
                        window,
                        band: target
                    }
                );
                windows.push(window);
            }
            Remove(index) => {
                let window = pick(index, &windows);
                let events = layout.remove(window);
                assert!(
                    matches!(events[0], LayoutEvent::WindowClosed { window: closed, .. } if closed == window)
                );
                windows.retain(|&candidate| candidate != window);
            }
            Shift(index, direction) => {
                let window = pick(index, &windows);
                let events = shift(&mut layout, window, direction);
                for event in events {
                    let LayoutEvent::WindowMoved {
                        window: moved,
                        column,
                        row,
                        ..
                    } = event
                    else {
                        panic!("a shift reported {event:?}");
                    };
                    assert_eq!(moved, window);
                    let location = layout.locate(window).unwrap();
                    assert_eq!((location.column, location.row), (column, row));
                }
            }
            Cycle(index) => {
                layout.apply(
                    SessionAction::CycleWidth(pick(index, &windows)),
                    AREA,
                    &LayoutOptions::default(),
                );
            }
            Full(index) => {
                layout.apply(
                    SessionAction::ToggleFullWidth(pick(index, &windows)),
                    AREA,
                    &LayoutOptions::default(),
                );
            }
        }
        assert_invariants(&layout);
        let mut present: Vec<_> = layout.windows().collect();
        let mut expected = windows.clone();
        present.sort();
        expected.sort();
        assert_eq!(present, expected, "every window keeps its identifier");
        if windows.is_empty() {
            break;
        }
    }
}

#[test]
fn identifier_survives_moves() {
    let (mut layout, windows) = row_of_columns(2);
    let (first, second) = (windows[0], windows[1]);
    let at = |column, row| Location {
        band: 0,
        column,
        row,
    };
    let steps = [
        (
            SessionAction::ConsumeOrExpel {
                window: second,
                direction: Direction::Left,
            },
            at(0, 1),
        ),
        (
            SessionAction::ConsumeOrExpel {
                window: second,
                direction: Direction::Right,
            },
            at(1, 0),
        ),
        (SessionAction::CycleWidth(second), at(1, 0)),
    ];
    for (action, location) in steps {
        assert!(
            !layout
                .apply(action.clone(), AREA, &LayoutOptions::default())
                .is_empty(),
            "{action:?}"
        );
        let mut present: Vec<_> = layout.windows().collect();
        present.sort();
        assert_eq!(present, [first, second], "{action:?}");
        assert_eq!(layout.locate(second), Some(location), "{action:?}");
    }
}

type Resize = gband_core::layout::Step;

fn step_width(layout: &mut Layout, window: WindowId, step: Resize) -> Vec<LayoutEvent> {
    layout.apply(
        SessionAction::StepWidth {
            window,
            step,
            by: Proportion::TENTH,
        },
        AREA,
        &LayoutOptions::default(),
    )
}

fn widths_while(
    layout: &mut Layout,
    window: WindowId,
    step: Resize,
    times: usize,
) -> Vec<Proportion> {
    (0..times)
        .map(|_| {
            step_width(layout, window, step);
            let (width, full_width) = width_of(layout, window);
            assert!(!full_width);
            width
        })
        .collect()
}

#[test]
fn width_grows_from_the_default() {
    let (mut layout, first) = started();
    assert_eq!(
        widths_while(&mut layout, first, Grow, 2),
        [Proportion::new(3, 5), Proportion::new(7, 10)]
    );
}

#[test]
fn width_grows_from_a_preset_of_thirds() {
    let (mut layout, first) = started();
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
    assert_eq!(width_of(&layout, first), (Proportion::ONE_THIRD, false));
    assert_eq!(
        widths_while(&mut layout, first, Grow, 1),
        [Proportion::new(13, 30)]
    );
}

#[test]
fn width_grows_by_another_step() {
    let (mut layout, first) = started();
    layout.apply(
        SessionAction::StepWidth {
            window: first,
            step: Grow,
            by: Proportion::new(1, 4),
        },
        AREA,
        &LayoutOptions::default(),
    );
    assert_eq!(width_of(&layout, first), (Proportion::new(3, 4), false));
}

#[test]
fn proportion_steps_keep_large_denominators_in_range() {
    let width = Proportion::new(1, 97).step(Grow, Proportion::new(1, 89));
    assert_eq!(width, Proportion::new(186, 8633));
    let mut width = Proportion::new(1, 2);
    for den in [97, 89, 83, 79, 73, 71, 67] {
        width = width.step(Grow, Proportion::new(1, den));
    }
    assert!(width.den > 0);
    assert!(width.num < width.den);
}

#[test]
fn width_shrinks_to_zero_and_stops() {
    let (mut layout, first) = started();
    assert_eq!(
        widths_while(&mut layout, first, Shrink, 5),
        [
            Proportion::new(2, 5),
            Proportion::new(3, 10),
            Proportion::new(1, 5),
            Proportion::new(1, 10),
            Proportion::new(0, 1),
        ]
    );
    let before = layout.clone();
    assert!(step_width(&mut layout, first, Shrink).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn width_grows_past_the_screen_width() {
    let (mut layout, first) = started();
    let widths = widths_while(&mut layout, first, Grow, 6);
    assert_eq!(widths[4], Proportion::WHOLE);
    assert_eq!(widths[5], Proportion::new(11, 10));
}

#[test]
fn width_stops_growing_at_the_limit() {
    let mut column = Column::new(WindowId(1), Proportion::ONE_HALF);
    column.width = Proportion::new(Proportion::MAX, 1);
    let before = column.clone();
    column.step_width(Grow, Proportion::TENTH);
    assert_eq!(column, before);
    assert_eq!(
        Proportion::new(99999, 10).step(Grow, Proportion::TENTH),
        Proportion::new(10000, 1)
    );
}

#[test]
fn width_steps_from_full_width() {
    for (step, expected) in [
        (Grow, Proportion::new(11, 10)),
        (Shrink, Proportion::new(9, 10)),
    ] {
        let (mut layout, first) = started();
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
            SessionAction::ToggleFullWidth(first),
            AREA,
            &LayoutOptions::default(),
        );
        assert_eq!(width_of(&layout, first), (Proportion::ONE_THIRD, true));
        assert!(!step_width(&mut layout, first, step).is_empty());
        assert_eq!(width_of(&layout, first), (expected, false));
    }
}

#[test]
fn proportion_steps_stay_in_lowest_terms() {
    assert_eq!(
        Proportion::new(2, 4).step(Grow, Proportion::TENTH),
        Proportion::new(3, 5)
    );
    assert_eq!(
        Proportion::new(1, 10).step(Shrink, Proportion::TENTH),
        Proportion::new(0, 1)
    );
    assert_eq!(
        Proportion::new(0, 1).step(Shrink, Proportion::TENTH),
        Proportion::new(0, 1)
    );
    assert_eq!(
        Proportion::new(9, 10).step(Grow, Proportion::TENTH),
        Proportion::WHOLE
    );
}

#[test]
fn proportion_of_saturates_at_the_cell_limit() {
    assert_eq!(Proportion::new(11, 10).of(80), 88);
    assert_eq!(Proportion::new(10000, 1).of(80), u16::MAX);
    assert_eq!(Proportion::new(0, 1).of(80), 0);
}

fn stacked(count: usize) -> (Layout, Vec<WindowId>) {
    let (mut layout, windows) = row_of_columns(count);
    for &window in &windows[1..] {
        shift(&mut layout, window, Direction::Left);
    }
    assert_eq!(columns(&layout, 0), vec![windows.clone()]);
    (layout, windows)
}

fn column_of(layout: &Layout, window: WindowId) -> &Column {
    let location = layout.locate(window).unwrap();
    &layout.bands()[location.band].columns[location.column]
}

fn heights_of(layout: &Layout, window: WindowId) -> Vec<WindowHeight> {
    column_of(layout, window).heights.clone()
}

fn rows_of(layout: &Layout, window: WindowId, area: Size) -> Vec<u16> {
    window_heights(column_of(layout, window), area.rows)
}

fn step_height(
    layout: &mut Layout,
    window: WindowId,
    step: Resize,
    area: Size,
) -> Vec<LayoutEvent> {
    layout.apply(
        SessionAction::StepHeight {
            window,
            step,
            by: Proportion::TENTH,
        },
        area,
        &LayoutOptions::default(),
    )
}

fn auto(num: u32, den: u32) -> WindowHeight {
    WindowHeight::Auto(Weight::new(num, den))
}

#[test]
fn each_height_step_adds_the_same_rows() {
    let (mut layout, windows) = stacked(2);
    let mut seen = Vec::new();
    for _ in 0..3 {
        assert!(!step_height(&mut layout, windows[0], Grow, AREA).is_empty());
        seen.push(rows_of(&layout, windows[0], AREA));
    }
    assert_eq!(seen, [[14, 10], [16, 8], [18, 6]]);
    assert_eq!(
        heights_of(&layout, windows[0]),
        [WindowHeight::Fixed(18), auto(1, 1)]
    );
}

#[test]
fn height_step_from_the_request() {
    let (mut layout, windows) = stacked(2);
    layout.apply(
        SessionAction::StepHeight {
            window: windows[0],
            step: Grow,
            by: Proportion::new(1, 4),
        },
        AREA,
        &LayoutOptions::default(),
    );
    assert_eq!(
        heights_of(&layout, windows[0]),
        [WindowHeight::Fixed(18), auto(1, 1)]
    );
    assert_eq!(rows_of(&layout, windows[0], AREA), [18, 6]);
}

#[test]
fn height_shrinks_in_a_stack_of_two() {
    let (mut layout, windows) = stacked(2);
    step_height(&mut layout, windows[0], Shrink, AREA);
    assert_eq!(
        heights_of(&layout, windows[0]),
        [WindowHeight::Fixed(10), auto(1, 1)]
    );
    assert_eq!(rows_of(&layout, windows[0], AREA), [10, 14]);
}

#[test]
fn height_step_rounds_half_up_on_an_odd_area() {
    let (mut layout, windows) = stacked(2);
    let area = Size::new(80, 25);
    step_height(&mut layout, windows[0], Grow, area);
    assert_eq!(heights_of(&layout, windows[0])[0], WindowHeight::Fixed(16));
    assert_eq!(rows_of(&layout, windows[0], area), [16, 9]);
}

#[test]
fn resizing_another_window_keeps_the_earlier_one_larger() {
    let (mut layout, windows) = stacked(3);
    step_height(&mut layout, windows[0], Grow, AREA);
    assert_eq!(rows_of(&layout, windows[0], AREA), [10, 7, 7]);
    step_height(&mut layout, windows[2], Grow, AREA);
    assert_eq!(
        heights_of(&layout, windows[0]),
        [auto(10, 7), auto(1, 1), WindowHeight::Fixed(9)]
    );
    assert_eq!(rows_of(&layout, windows[0], AREA), [9, 6, 9]);
}

#[test]
fn height_stops_at_the_upper_limit() {
    let (mut layout, windows) = stacked(3);
    for _ in 0..5 {
        step_height(&mut layout, windows[0], Grow, AREA);
    }
    assert_eq!(heights_of(&layout, windows[0])[0], WindowHeight::Fixed(18));
    let before = layout.clone();
    assert!(step_height(&mut layout, windows[0], Grow, AREA).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn height_stops_at_the_lower_limit() {
    let (mut layout, windows) = stacked(2);
    for _ in 0..5 {
        step_height(&mut layout, windows[0], Shrink, AREA);
    }
    assert_eq!(heights_of(&layout, windows[0])[0], WindowHeight::Fixed(3));
    let before = layout.clone();
    assert!(step_height(&mut layout, windows[0], Shrink, AREA).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn lone_window_shrinks_below_the_area() {
    let (mut layout, first) = started();
    step_height(&mut layout, first, Shrink, AREA);
    assert_eq!(heights_of(&layout, first), [WindowHeight::Fixed(22)]);
    assert_eq!(rows_of(&layout, first, AREA), [22]);
}

#[test]
fn reset_height_returns_to_weight_one() {
    let (mut layout, windows) = stacked(2);
    step_height(&mut layout, windows[0], Grow, AREA);
    step_height(&mut layout, windows[0], Grow, AREA);
    assert_eq!(heights_of(&layout, windows[0])[0], WindowHeight::Fixed(16));
    assert!(
        !layout
            .apply(
                SessionAction::ResetHeight(windows[0]),
                AREA,
                &LayoutOptions::default()
            )
            .is_empty()
    );
    assert_eq!(heights_of(&layout, windows[0]), [auto(1, 1), auto(1, 1)]);
    assert_eq!(rows_of(&layout, windows[0], AREA), [12, 12]);
    assert!(
        layout
            .apply(
                SessionAction::ResetHeight(windows[0]),
                AREA,
                &LayoutOptions::default()
            )
            .is_empty()
    );
}

#[test]
fn consume_next_to_a_fixed_height() {
    let (mut layout, windows) = row_of_columns(3);
    let (p1, p2, p3) = (windows[0], windows[1], windows[2]);
    shift(&mut layout, p3, Direction::Left);
    step_height(&mut layout, p2, Grow, AREA);
    step_height(&mut layout, p2, Grow, AREA);
    shift(&mut layout, p1, Direction::Right);
    assert_eq!(columns(&layout, 0), vec![vec![p2, p3, p1]]);
    assert_eq!(
        heights_of(&layout, p2),
        [WindowHeight::Fixed(16), auto(1, 1), auto(1, 1)]
    );
    assert_eq!(rows_of(&layout, p2, AREA), [16, 4, 4]);
}

#[test]
fn last_window_in_a_column_takes_weight_one() {
    let (mut layout, windows) = row_of_columns(4);
    let (p1, middle, p2, p3) = (windows[0], windows[1], windows[2], windows[3]);
    shift(&mut layout, middle, Direction::Left);
    shift(&mut layout, p2, Direction::Left);
    step_height(&mut layout, p1, Grow, AREA);
    step_height(&mut layout, p2, Grow, AREA);
    layout.remove(middle);
    assert_invariants(&layout);
    assert_eq!(
        heights_of(&layout, p1),
        [auto(10, 7), WindowHeight::Fixed(9)]
    );
    layout.remove(p2);
    assert_invariants(&layout);
    assert_eq!(heights_of(&layout, p1), [auto(1, 1)]);
    shift(&mut layout, p3, Direction::Left);
    assert_eq!(columns(&layout, 0), vec![vec![p1, p3]]);
    assert_eq!(heights_of(&layout, p1), [auto(1, 1), auto(1, 1)]);
    assert_eq!(rows_of(&layout, p1, AREA), [12, 12]);
}

#[test]
fn expelled_window_takes_weight_one_and_the_rest_keep_theirs() {
    let (mut layout, windows) = stacked(3);
    step_height(&mut layout, windows[0], Grow, AREA);
    step_height(&mut layout, windows[2], Grow, AREA);
    shift(&mut layout, windows[0], Direction::Right);
    assert_eq!(
        heights_of(&layout, windows[1]),
        [auto(1, 1), WindowHeight::Fixed(9)]
    );
    assert_eq!(heights_of(&layout, windows[0]), [auto(1, 1)]);
}

#[test]
fn weights_are_held_in_lowest_terms() {
    assert_eq!(Weight::new(14, 7), Weight::new(2, 1));
    let weight = Weight::new(20, 14);
    assert_eq!((weight.num(), weight.den()), (10, 7));
    assert_eq!(Weight::ONE, Weight::new(5, 5));
}

fn set_width(layout: &mut Layout, window: WindowId, width: Proportion) -> Vec<LayoutEvent> {
    layout.apply(
        SessionAction::SetWidth { window, width },
        AREA,
        &LayoutOptions::default(),
    )
}

fn set_height(layout: &mut Layout, window: WindowId, height: WindowHeight) -> Vec<LayoutEvent> {
    layout.apply(
        SessionAction::SetHeight { window, height },
        AREA,
        &LayoutOptions::default(),
    )
}

#[test]
fn set_width_turns_full_width_off() {
    let (mut layout, first) = started();
    layout.apply(
        SessionAction::ToggleFullWidth(first),
        AREA,
        &LayoutOptions::default(),
    );
    assert!(!set_width(&mut layout, first, Proportion::new(7, 20)).is_empty());
    assert_eq!(width_of(&layout, first), (Proportion::new(7, 20), false));
}

#[test]
fn set_width_is_held_in_lowest_terms() {
    let (mut layout, first) = started();
    set_width(&mut layout, first, Proportion::new(4, 10));
    assert_eq!(width_of(&layout, first), (Proportion::new(2, 5), false));
}

#[test]
fn setting_the_same_width_changes_nothing() {
    let (mut layout, first) = started();
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
    assert_eq!(width_of(&layout, first), (Proportion::ONE_THIRD, false));
    let before = layout.clone();
    assert!(set_width(&mut layout, first, Proportion::ONE_THIRD).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn width_named_on_open() {
    let (mut layout, first) = started();
    let window = layout.allocate_window();
    let band = band_id(&layout, 0);
    layout.open(
        window,
        band,
        Some(first),
        Some(Proportion::new(1, 4)),
        &LayoutOptions::default(),
    );
    assert_eq!(width_of(&layout, window), (Proportion::new(1, 4), false));
    assert_eq!(width_of(&layout, first), (Proportion::ONE_HALF, false));
}

#[test]
fn fixed_rows_in_a_stack_of_two() {
    let (mut layout, windows) = stacked(2);
    assert!(!set_height(&mut layout, windows[0], WindowHeight::Fixed(8)).is_empty());
    assert_eq!(
        heights_of(&layout, windows[0]),
        [WindowHeight::Fixed(8), auto(1, 1)]
    );
    assert_eq!(rows_of(&layout, windows[0], AREA), [8, 16]);
}

#[test]
fn rows_are_kept_within_the_limits() {
    let (mut layout, windows) = stacked(2);
    set_height(&mut layout, windows[0], WindowHeight::Fixed(30));
    assert_eq!(heights_of(&layout, windows[0])[0], WindowHeight::Fixed(21));
    set_height(&mut layout, windows[0], WindowHeight::Fixed(1));
    assert_eq!(heights_of(&layout, windows[0])[0], WindowHeight::Fixed(3));
    let before = layout.clone();
    assert!(set_height(&mut layout, windows[0], WindowHeight::Fixed(2)).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn fixed_height_moves_to_another_window() {
    let (mut layout, windows) = stacked(2);
    set_height(&mut layout, windows[0], WindowHeight::Fixed(14));
    set_height(&mut layout, windows[1], WindowHeight::Fixed(6));
    assert_eq!(
        heights_of(&layout, windows[0]),
        [auto(1, 1), WindowHeight::Fixed(6)]
    );
}

#[test]
fn set_a_weight_keeps_the_others() {
    let (mut layout, windows) = stacked(2);
    set_height(&mut layout, windows[0], WindowHeight::Fixed(14));
    assert!(!set_height(&mut layout, windows[0], auto(2, 1)).is_empty());
    assert_eq!(heights_of(&layout, windows[0]), [auto(2, 1), auto(1, 1)]);
    assert!(set_height(&mut layout, windows[0], auto(4, 2)).is_empty());
}

fn act(layout: &mut Layout, action: SessionAction, area: Size) -> Vec<LayoutEvent> {
    let events = layout.apply(action, area, &LayoutOptions::default());
    assert_invariants(layout);
    events
}

fn toggle(
    layout: &mut Layout,
    window: WindowId,
    after: Option<WindowId>,
    floating: Option<bool>,
) -> Vec<LayoutEvent> {
    act(
        layout,
        SessionAction::ToggleFloating {
            window,
            after,
            floating,
        },
        AREA,
    )
}

fn record(layout: &Layout, window: WindowId) -> FloatingWindow {
    *layout.floating(window).expect("a floating window")
}

fn floating_list(layout: &Layout, band: usize) -> Vec<WindowId> {
    layout.bands()[band]
        .floating
        .iter()
        .map(|floating| floating.window)
        .collect()
}

fn box_at(layout: &mut Layout, window: WindowId, col: u16, row: u16, width: Proportion, rows: u16) {
    let large = Size::new(1000, 1000);
    act(layout, SessionAction::SetWidth { window, width }, large);
    let height = WindowHeight::Fixed(rows);
    act(layout, SessionAction::SetHeight { window, height }, large);
    act(
        layout,
        SessionAction::SetPosition { window, col, row },
        large,
    );
    let placed = record(layout, window);
    assert_eq!(
        (placed.col, placed.row, placed.width, placed.rows),
        (col, row, width, rows)
    );
}

fn floated(columns: usize) -> (Layout, Vec<WindowId>) {
    let (mut layout, windows) = row_of_columns(columns);
    let last = *windows.last().unwrap();
    toggle(&mut layout, last, None, None);
    (layout, windows)
}

#[test]
fn float_the_only_window_of_a_band() {
    let (mut layout, first) = started();
    toggle(&mut layout, first, None, None);
    assert!(layout.bands()[0].columns.is_empty());
    assert_eq!(floating_list(&layout, 0), [first]);
    assert_eq!(layout.bands().len(), 2);
}

#[test]
fn window_in_one_place_only() {
    let (mut layout, windows) = row_of_columns(2);
    toggle(&mut layout, windows[1], None, None);
    assert_eq!(
        layout
            .windows()
            .filter(|&window| window == windows[1])
            .count(),
        1
    );
    assert_eq!(
        layout.place(windows[1]),
        Some(Place::Floating { band: 0, index: 0 })
    );
    toggle(&mut layout, windows[1], Some(windows[0]), None);
    assert_eq!(
        layout
            .windows()
            .filter(|&window| window == windows[1])
            .count(),
        1
    );
    assert!(matches!(layout.place(windows[1]), Some(Place::Tiled(_))));
}

#[test]
fn column_removed_when_its_last_window_floats() {
    let (mut layout, windows) = row_of_columns(3);
    toggle(&mut layout, windows[1], None, None);
    assert_eq!(
        columns(&layout, 0),
        vec![vec![windows[0]], vec![windows[2]]]
    );
    assert_eq!(floating_list(&layout, 0), [windows[1]]);
}

#[test]
fn floating_window_keeps_its_band() {
    let (mut layout, first) = started();
    open(&mut layout, 1, None);
    toggle(&mut layout, first, None, None);
    assert_eq!(layout.bands().len(), 3);
    assert!(!layout.bands()[0].is_empty());
    let kept = band_id(&layout, 1);
    layout.remove(first);
    assert_invariants(&layout);
    assert_eq!(layout.bands().len(), 2);
    assert_eq!(band_id(&layout, 0), kept);
    assert!(!layout.contains(first));
}

#[test]
fn floating_window_opened_in_the_empty_band() {
    let (mut layout, _) = started();
    let window = layout.allocate_window();
    let empty = band_id(&layout, 1);
    let events = layout.open_floating(window, empty, None, AREA, &LayoutOptions::default());
    assert_invariants(&layout);
    assert!(!events.is_empty());
    assert_eq!(layout.bands().len(), 3);
    assert_eq!(floating_list(&layout, 1), [window]);
    assert!(layout.bands()[2].is_empty());
}

#[test]
fn open_a_floating_window() {
    let (mut layout, windows) = row_of_columns(2);
    let window = layout.allocate_window();
    let band = band_id(&layout, 0);
    layout.open_floating(window, band, None, AREA, &LayoutOptions::default());
    assert_eq!(
        columns(&layout, 0),
        vec![vec![windows[0]], vec![windows[1]]]
    );
    assert_eq!(floating_list(&layout, 0).last(), Some(&window));
    assert_eq!(
        record(&layout, window),
        FloatingWindow {
            window,
            col: 20,
            row: 2,
            width: Proportion::ONE_HALF,
            full_width: false,
            rows: 20,
        }
    );
}

#[test]
fn open_a_floating_window_with_a_width() {
    let (mut layout, _) = started();
    let window = layout.allocate_window();
    let band = band_id(&layout, 0);
    let width = Some(Proportion::new(1, 4));
    layout.open_floating(window, band, width, AREA, &LayoutOptions::default());
    let floating = record(&layout, window);
    assert_eq!(
        (floating.width, floating.col, floating.rows),
        (Proportion::new(1, 4), 30, 20)
    );
}

#[test]
fn open_floating_in_a_missing_band() {
    let (mut layout, _) = started();
    let window = layout.allocate_window();
    let before = layout.clone();
    let events = layout.open_floating(window, BandId(99), None, AREA, &LayoutOptions::default());
    assert!(events.is_empty());
    assert_eq!(layout, before);
}

#[test]
fn floating_window_exits() {
    let (mut layout, windows) = row_of_columns(3);
    toggle(&mut layout, windows[1], None, None);
    toggle(&mut layout, windows[2], None, None);
    layout.remove(windows[1]);
    assert_invariants(&layout);
    assert_eq!(floating_list(&layout, 0), [windows[2]]);
}

#[test]
fn first_float_opens_a_new_box() {
    let (mut layout, windows) = row_of_columns(2);
    toggle(&mut layout, windows[1], None, None);
    assert_eq!(
        record(&layout, windows[1]),
        FloatingWindow {
            window: windows[1],
            col: 20,
            row: 2,
            width: Proportion::ONE_HALF,
            full_width: false,
            rows: 20,
        }
    );
    let placed = placed(&record(&layout, windows[1]), AREA);
    assert_eq!(
        (placed.x, placed.y, placed.width, placed.height),
        (20, 2, 40, 20)
    );
}

#[test]
fn first_float_takes_the_default_width() {
    let (mut layout, windows) = row_of_columns(2);
    act(
        &mut layout,
        SessionAction::ToggleFullWidth(windows[1]),
        AREA,
    );
    let options = LayoutOptions {
        default_width: Proportion::ONE_THIRD,
        ..LayoutOptions::default()
    };
    let (window, after) = (windows[1], None);
    layout.apply(
        SessionAction::ToggleFloating {
            window,
            after,
            floating: None,
        },
        AREA,
        &options,
    );
    let floating = record(&layout, windows[1]);
    assert_eq!(
        (
            floating.width,
            floating.full_width,
            floating.col,
            floating.row
        ),
        (Proportion::ONE_THIRD, false, 27, 2)
    );
}

#[test]
fn float_from_a_stack() {
    let (mut layout, windows) = stacked(2);
    set_width(&mut layout, windows[0], Proportion::ONE_THIRD);
    toggle(&mut layout, windows[1], None, None);
    assert_eq!(columns(&layout, 0), vec![vec![windows[0]]]);
    assert_eq!(heights_of(&layout, windows[0]), [auto(1, 1)]);
    let floating = record(&layout, windows[1]);
    assert_eq!(
        (floating.width, floating.rows, floating.col, floating.row),
        (Proportion::ONE_HALF, 20, 20, 2)
    );
}

#[test]
fn new_box_in_a_tall_area() {
    let (mut layout, windows) = row_of_columns(2);
    let area = Size::new(120, 67);
    let (window, after) = (windows[1], None);
    layout.apply(
        SessionAction::ToggleFloating {
            window,
            after,
            floating: None,
        },
        area,
        &LayoutOptions::default(),
    );
    let floating = record(&layout, windows[1]);
    assert_eq!((floating.rows, floating.row, floating.col), (53, 7, 30));
}

#[test]
fn float_again_returns_to_the_last_box() {
    let (mut layout, windows) = floated(2);
    box_at(&mut layout, windows[1], 5, 3, Proportion::ONE_THIRD, 10);
    let kept = record(&layout, windows[1]);
    toggle(&mut layout, windows[1], Some(windows[0]), None);
    toggle(&mut layout, windows[1], None, None);
    assert_eq!(record(&layout, windows[1]), kept);
}

#[test]
fn tile_right_of_the_named_window() {
    let (mut layout, windows) = row_of_columns(3);
    toggle(&mut layout, windows[2], None, None);
    set_width(&mut layout, windows[2], Proportion::ONE_THIRD);
    toggle(&mut layout, windows[2], Some(windows[0]), None);
    assert_eq!(
        columns(&layout, 0),
        vec![vec![windows[0]], vec![windows[2]], vec![windows[1]]]
    );
    assert_eq!(
        width_of(&layout, windows[2]),
        (Proportion::ONE_THIRD, false)
    );
    assert!(layout.bands()[0].floating.is_empty());
}

#[test]
fn tile_with_no_window_named() {
    let (mut layout, windows) = floated(2);
    toggle(&mut layout, windows[1], None, None);
    assert_eq!(
        columns(&layout, 0),
        vec![vec![windows[1]], vec![windows[0]]]
    );
    assert_eq!(heights_of(&layout, windows[1]), [WindowHeight::DEFAULT]);
}

#[test]
fn tile_after_a_window_of_another_band_goes_first() {
    let (mut layout, windows) = floated(2);
    let other = open(&mut layout, 1, None);
    toggle(&mut layout, windows[1], Some(other), None);
    assert_eq!(
        columns(&layout, 0),
        vec![vec![windows[1]], vec![windows[0]]]
    );
}

#[test]
fn tiled_full_width_is_kept() {
    let (mut layout, windows) = floated(2);
    act(
        &mut layout,
        SessionAction::ToggleFullWidth(windows[1]),
        AREA,
    );
    assert!(record(&layout, windows[1]).full_width);
    toggle(&mut layout, windows[1], Some(windows[0]), None);
    assert_eq!(width_of(&layout, windows[1]), (Proportion::ONE_HALF, true));
}

#[test]
fn request_to_float_a_tiled_window() {
    let (mut layout, windows) = row_of_columns(2);
    assert!(!toggle(&mut layout, windows[1], None, Some(true)).is_empty());
    assert_eq!(
        record(&layout, windows[1]),
        FloatingWindow {
            window: windows[1],
            col: 20,
            row: 2,
            width: Proportion::ONE_HALF,
            full_width: false,
            rows: 20,
        }
    );
}

#[test]
fn request_to_float_a_floating_window() {
    let (mut layout, windows) = row_of_columns(3);
    toggle(&mut layout, windows[1], None, None);
    toggle(&mut layout, windows[2], None, None);
    box_at(&mut layout, windows[1], 5, 3, Proportion::ONE_THIRD, 10);
    let before = layout.clone();
    assert!(toggle(&mut layout, windows[1], None, Some(true)).is_empty());
    assert_eq!(layout, before);
    assert_eq!(floating_list(&layout, 0), [windows[1], windows[2]]);
}

#[test]
fn two_requests_to_float_one_window() {
    let (mut layout, windows) = row_of_columns(2);
    toggle(&mut layout, windows[1], None, Some(true));
    toggle(&mut layout, windows[1], None, Some(true));
    assert_eq!(floating_list(&layout, 0), [windows[1]]);
    let floating = record(&layout, windows[1]);
    assert_eq!(
        (floating.width, floating.rows, floating.col, floating.row),
        (Proportion::ONE_HALF, 20, 20, 2)
    );
    assert_eq!(columns(&layout, 0), vec![vec![windows[0]]]);
}

#[test]
fn request_to_tile_a_floating_window() {
    let (mut layout, windows) = row_of_columns(3);
    toggle(&mut layout, windows[2], None, None);
    set_width(&mut layout, windows[2], Proportion::ONE_THIRD);
    toggle(&mut layout, windows[2], Some(windows[0]), Some(false));
    assert_eq!(
        columns(&layout, 0),
        vec![vec![windows[0]], vec![windows[2]], vec![windows[1]]]
    );
    assert_eq!(
        width_of(&layout, windows[2]),
        (Proportion::ONE_THIRD, false)
    );
    assert!(layout.bands()[0].floating.is_empty());
}

#[test]
fn request_to_tile_a_tiled_window() {
    let (mut layout, windows) = row_of_columns(2);
    let before = layout.clone();
    assert!(toggle(&mut layout, windows[1], Some(windows[0]), Some(false)).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn cycle_a_floating_width() {
    let (mut layout, windows) = floated(2);
    act(&mut layout, SessionAction::CycleWidth(windows[1]), AREA);
    assert_eq!(record(&layout, windows[1]).width, Proportion::TWO_THIRDS);
}

#[test]
fn floating_width_follows_the_column_rules() {
    let (mut layout, windows) = floated(2);
    let before = record(&layout, windows[1]);
    act(
        &mut layout,
        SessionAction::ToggleFullWidth(windows[1]),
        AREA,
    );
    assert!(record(&layout, windows[1]).full_width);
    let step = Shrink;
    act(
        &mut layout,
        SessionAction::StepWidth {
            window: windows[1],
            step,
            by: Proportion::TENTH,
        },
        AREA,
    );
    let floating = record(&layout, windows[1]);
    assert_eq!(
        (floating.width, floating.full_width),
        (Proportion::new(9, 10), false)
    );
    set_width(&mut layout, windows[1], Proportion::new(4, 10));
    let floating = record(&layout, windows[1]);
    assert_eq!(
        (floating.width, floating.full_width),
        (Proportion::new(2, 5), false)
    );
    assert_eq!((floating.col, floating.row), (before.col, before.row));
}

#[test]
fn grow_a_floating_width_by_another_step() {
    let (mut layout, windows) = floated(2);
    box_at(&mut layout, windows[1], 0, 0, Proportion::ONE_HALF, 12);
    act(
        &mut layout,
        SessionAction::StepWidth {
            window: windows[1],
            step: Grow,
            by: Proportion::new(1, 5),
        },
        AREA,
    );
    assert_eq!(record(&layout, windows[1]).width, Proportion::new(7, 10));
}

#[test]
fn grow_a_floating_height() {
    let (mut layout, windows) = floated(2);
    box_at(&mut layout, windows[1], 0, 0, Proportion::ONE_HALF, 12);
    step_height(&mut layout, windows[1], Grow, AREA);
    assert_eq!(record(&layout, windows[1]).rows, 14);
    step_height(&mut layout, windows[1], Shrink, AREA);
    assert_eq!(record(&layout, windows[1]).rows, 12);
}

#[test]
fn floating_height_limit() {
    let (mut layout, windows) = floated(2);
    set_height(&mut layout, windows[1], WindowHeight::Fixed(24));
    assert_eq!(record(&layout, windows[1]).rows, 24);
    let before = layout.clone();

    assert!(step_height(&mut layout, windows[1], Grow, AREA).is_empty());
    assert_eq!(layout, before);
    set_height(&mut layout, windows[1], WindowHeight::Fixed(1));
    assert_eq!(record(&layout, windows[1]).rows, 3);
}

#[test]
fn reset_a_floating_height() {
    let (mut layout, windows) = floated(2);
    set_height(&mut layout, windows[1], WindowHeight::Fixed(6));
    act(
        &mut layout,
        SessionAction::ResetHeight(windows[1]),
        Size::new(80, 25),
    );
    assert_eq!(record(&layout, windows[1]).rows, 19);
}

#[test]
fn weight_leaves_a_floating_window_unchanged() {
    let (mut layout, windows) = floated(2);
    assert!(set_height(&mut layout, windows[1], auto(2, 1)).is_empty());
}

#[test]
fn consume_or_expel_a_floating_window() {
    let (mut layout, windows) = floated(2);
    let before = layout.clone();
    assert!(shift(&mut layout, windows[1], Direction::Left).is_empty());
    assert!(shift(&mut layout, windows[1], Direction::Right).is_empty());
    assert_eq!(layout, before);
}

fn move_column(
    layout: &mut Layout,
    window: WindowId,
    direction: Direction,
    area: Size,
) -> Vec<LayoutEvent> {
    act(
        layout,
        SessionAction::MoveColumn { window, direction },
        area,
    )
}

fn move_window(
    layout: &mut Layout,
    window: WindowId,
    direction: Vertical,
    area: Size,
) -> Vec<LayoutEvent> {
    act(
        layout,
        SessionAction::MoveWindow { window, direction },
        area,
    )
}

#[test]
fn move_a_column_right() {
    let (mut layout, windows) = row_of_columns(3);
    set_width(&mut layout, windows[0], Proportion::ONE_THIRD);
    move_column(&mut layout, windows[0], Direction::Right, AREA);
    assert_eq!(
        columns(&layout, 0),
        vec![vec![windows[1]], vec![windows[0]], vec![windows[2]]]
    );
    assert_eq!(
        width_of(&layout, windows[0]),
        (Proportion::ONE_THIRD, false)
    );
    assert_eq!(width_of(&layout, windows[1]), (Proportion::ONE_HALF, false));
}

#[test]
fn move_a_column_at_the_edge() {
    let (mut layout, windows) = row_of_columns(2);
    let before = layout.clone();
    assert!(move_column(&mut layout, windows[0], Direction::Left, AREA).is_empty());
    assert!(move_column(&mut layout, windows[1], Direction::Right, AREA).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn move_a_window_down() {
    let (mut layout, windows) = stacked(2);
    set_height(&mut layout, windows[0], WindowHeight::Fixed(16));
    let below = heights_of(&layout, windows[0])[1];
    move_window(&mut layout, windows[0], Vertical::Down, AREA);
    assert_eq!(columns(&layout, 0), vec![vec![windows[1], windows[0]]]);
    assert_eq!(
        heights_of(&layout, windows[0]),
        [below, WindowHeight::Fixed(16)]
    );
}

#[test]
fn move_a_window_at_the_bottom() {
    let (mut layout, windows) = stacked(2);
    let before = layout.clone();
    assert!(move_window(&mut layout, windows[1], Vertical::Down, AREA).is_empty());
    assert!(move_window(&mut layout, windows[0], Vertical::Up, AREA).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn move_a_lone_window_up() {
    let (mut layout, windows) = row_of_columns(2);
    let before = layout.clone();
    assert!(move_window(&mut layout, windows[0], Vertical::Up, AREA).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn move_a_floating_window_right() {
    let (mut layout, windows) = floated(2);
    assert_eq!(record(&layout, windows[1]).col, 20);
    move_column(&mut layout, windows[1], Direction::Right, AREA);
    assert_eq!(record(&layout, windows[1]).col, 28);
    move_column(&mut layout, windows[1], Direction::Left, AREA);
    assert_eq!(record(&layout, windows[1]).col, 20);
}

#[test]
fn move_a_floating_window_against_the_edge() {
    let (mut layout, windows) = floated(2);
    box_at(&mut layout, windows[1], 36, 0, Proportion::ONE_HALF, 12);
    move_column(&mut layout, windows[1], Direction::Right, AREA);
    assert_eq!(record(&layout, windows[1]).col, 40);
    let before = layout.clone();
    assert!(move_column(&mut layout, windows[1], Direction::Right, AREA).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn move_a_floating_window_down() {
    let (mut layout, windows) = floated(2);
    box_at(&mut layout, windows[1], 20, 2, Proportion::ONE_HALF, 12);
    move_window(&mut layout, windows[1], Vertical::Down, Size::new(80, 25));
    assert_eq!(record(&layout, windows[1]).row, 5);
    move_window(&mut layout, windows[1], Vertical::Up, Size::new(80, 25));
    assert_eq!(record(&layout, windows[1]).row, 2);
}

#[test]
fn moving_starts_from_the_placed_box() {
    let (mut layout, windows) = floated(2);
    box_at(&mut layout, windows[1], 70, 20, Proportion::ONE_HALF, 12);
    move_window(&mut layout, windows[1], Vertical::Up, AREA);
    let floating = record(&layout, windows[1]);
    assert_eq!((floating.col, floating.row), (40, 10));
}

#[test]
fn place_exactly() {
    let (mut layout, windows) = floated(2);
    box_at(&mut layout, windows[1], 0, 0, Proportion::ONE_HALF, 12);
    let (window, col, row) = (windows[1], 70, 3);
    act(
        &mut layout,
        SessionAction::SetPosition { window, col, row },
        AREA,
    );
    let floating = record(&layout, windows[1]);
    assert_eq!((floating.col, floating.row), (40, 3));
}

#[test]
fn set_position_on_a_tiled_window() {
    let (mut layout, windows) = row_of_columns(2);
    let before = layout.clone();
    let (window, col, row) = (windows[0], 4, 4);
    assert!(
        act(
            &mut layout,
            SessionAction::SetPosition { window, col, row },
            AREA
        )
        .is_empty()
    );
    assert_eq!(layout, before);
}

#[test]
fn identifier_survives_floating() {
    let (mut layout, windows) = row_of_columns(2);
    toggle(&mut layout, windows[1], None, None);
    assert!(layout.contains(windows[1]));
    move_column(&mut layout, windows[1], Direction::Right, AREA);
    assert!(layout.contains(windows[1]));
    toggle(&mut layout, windows[1], Some(windows[0]), None);
    assert_eq!(layout.windows().collect::<Vec<_>>(), windows);
}

fn move_to_place(
    layout: &mut Layout,
    window: WindowId,
    reference: WindowId,
    place: TargetPlace,
) -> Vec<LayoutEvent> {
    let events = layout.apply(
        SessionAction::MoveToPlace {
            window,
            reference,
            place,
        },
        AREA,
        &LayoutOptions::default(),
    );
    assert_invariants(layout);
    events
}

fn apply(layout: &mut Layout, action: SessionAction) {
    layout.apply(action, AREA, &LayoutOptions::default());
}

fn heights(layout: &Layout, column: usize) -> Vec<WindowHeight> {
    layout.bands()[0].columns[column].heights.clone()
}

#[test]
fn move_into_a_new_column_right_of_another() {
    let (mut layout, windows) = row_of_columns(3);
    let [a, b, c] = windows[..] else { panic!() };
    apply(
        &mut layout,
        SessionAction::SetWidth {
            window: a,
            width: Proportion::new(1, 3),
        },
    );
    let events = move_to_place(&mut layout, a, b, TargetPlace::ColumnRight);
    assert_eq!(columns(&layout, 0), [vec![b], vec![a], vec![c]]);
    assert_eq!(width_of(&layout, a), (Proportion::new(1, 3), false));
    assert_eq!(
        events,
        [LayoutEvent::WindowMoved {
            window: a,
            band: band_id(&layout, 0),
            column: 1,
            row: 0,
        }]
    );
}

#[test]
fn move_from_a_stack_into_a_new_column() {
    let (mut layout, windows) = row_of_columns(3);
    let [p1, p2, p3] = windows[..] else { panic!() };
    shift(&mut layout, p2, Direction::Left);
    apply(
        &mut layout,
        SessionAction::SetHeight {
            window: p1,
            height: WindowHeight::Fixed(16),
        },
    );
    let width = width_of(&layout, p1);
    move_to_place(&mut layout, p1, p3, TargetPlace::ColumnLeft);
    assert_eq!(columns(&layout, 0), [vec![p2], vec![p1], vec![p3]]);
    assert_eq!(heights(&layout, 0), [WindowHeight::DEFAULT]);
    assert_eq!(heights(&layout, 1), [WindowHeight::DEFAULT]);
    assert_eq!(width_of(&layout, p1), width);
}

#[test]
fn move_into_another_column() {
    let (mut layout, windows) = row_of_columns(3);
    let [p1, p2, p3] = windows[..] else { panic!() };
    shift(&mut layout, p3, Direction::Left);
    apply(
        &mut layout,
        SessionAction::SetHeight {
            window: p2,
            height: WindowHeight::Fixed(8),
        },
    );
    move_to_place(&mut layout, p1, p2, TargetPlace::Below);
    assert_eq!(columns(&layout, 0), [vec![p2, p1, p3]]);
    assert_eq!(heights(&layout, 0)[0], WindowHeight::Fixed(8));
    assert_eq!(heights(&layout, 0)[1], WindowHeight::DEFAULT);
}

#[test]
fn move_within_its_column_keeps_its_height() {
    let (mut layout, windows) = row_of_columns(2);
    let [p1, p2] = windows[..] else { panic!() };
    shift(&mut layout, p2, Direction::Left);
    apply(
        &mut layout,
        SessionAction::SetHeight {
            window: p1,
            height: WindowHeight::Fixed(16),
        },
    );
    move_to_place(&mut layout, p1, p2, TargetPlace::Below);
    assert_eq!(columns(&layout, 0), [vec![p2, p1]]);
    assert_eq!(heights(&layout, 0)[1], WindowHeight::Fixed(16));
}

#[test]
fn move_back_to_its_own_place_changes_nothing() {
    let (mut layout, windows) = row_of_columns(2);
    let [a, b] = windows[..] else { panic!() };
    let before = layout.clone();
    assert!(move_to_place(&mut layout, a, b, TargetPlace::ColumnLeft).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn move_to_place_ignores_floating_and_invalid_references() {
    let (mut layout, windows) = row_of_columns(3);
    let [a, b, c] = windows[..] else { panic!() };
    apply(
        &mut layout,
        SessionAction::ToggleFloating {
            window: c,
            after: None,
            floating: None,
        },
    );
    let before = layout.clone();
    assert!(move_to_place(&mut layout, c, a, TargetPlace::Above).is_empty());
    assert!(move_to_place(&mut layout, a, c, TargetPlace::Above).is_empty());
    assert!(move_to_place(&mut layout, a, a, TargetPlace::Below).is_empty());
    assert!(move_to_place(&mut layout, a, WindowId(99), TargetPlace::Below).is_empty());
    assert_eq!(layout, before);
    let other = open(&mut layout, 1, None);
    let before = layout.clone();
    assert!(move_to_place(&mut layout, b, other, TargetPlace::Above).is_empty());
    assert_eq!(layout, before);
}
