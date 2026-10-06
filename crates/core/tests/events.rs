use gband_core::event::LayoutEvent;
use gband_core::geometry::Size;
use gband_core::layout::{
    BandId, Direction, FloatingWindow, Layout, LayoutOptions, Proportion, SessionAction, Step,
    Vertical, Weight, WindowHeight, WindowId,
};

const AREA: Size = Size::new(80, 24);

fn open(layout: &mut Layout, band: usize, after: Option<WindowId>) -> (WindowId, Vec<LayoutEvent>) {
    let window = layout.allocate_window();
    let id = layout.bands()[band].id;
    let events = layout.open(window, id, after, None, &LayoutOptions::default());
    (window, events)
}

fn band(layout: &Layout, index: usize) -> BandId {
    layout.bands()[index].id
}

#[test]
fn open_in_the_empty_band_adds_one_below() {
    let mut layout = Layout::new();
    let empty = band(&layout, 0);
    let (window, events) = open(&mut layout, 0, None);
    assert_eq!(
        events,
        [
            LayoutEvent::WindowOpened {
                window,
                band: empty
            },
            LayoutEvent::BandAdded {
                band: band(&layout, 1),
                index: 1,
            },
        ]
    );
}

#[test]
fn open_beside_a_window_adds_no_band() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (second, events) = open(&mut layout, 0, Some(first));
    assert_eq!(
        events,
        [LayoutEvent::WindowOpened {
            window: second,
            band: band(&layout, 0),
        }]
    );
}

#[test]
fn last_window_of_a_middle_band_closing_removes_it() {
    let mut layout = Layout::new();
    let (top, _) = open(&mut layout, 0, None);
    open(&mut layout, 1, None);
    let middle = band(&layout, 0);
    assert_eq!(
        layout.remove(top),
        [
            LayoutEvent::WindowClosed {
                window: top,
                band: middle,
            },
            LayoutEvent::BandRemoved { band: middle },
        ]
    );
}

#[test]
fn closing_the_only_window_removes_its_band() {
    let mut layout = Layout::new();
    let (window, _) = open(&mut layout, 0, None);
    let filled = band(&layout, 0);
    assert_eq!(
        layout.remove(window),
        [
            LayoutEvent::WindowClosed {
                window,
                band: filled
            },
            LayoutEvent::BandRemoved { band: filled },
        ]
    );
}

#[test]
fn consume_into_the_left_neighbour_moves_one_window() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (second, _) = open(&mut layout, 0, Some(first));
    let events = layout.apply(
        SessionAction::ConsumeOrExpel {
            window: second,
            direction: Direction::Left,
        },
        AREA,
        &LayoutOptions::default(),
    );
    assert_eq!(
        events,
        [LayoutEvent::WindowMoved {
            window: second,
            band: band(&layout, 0),
            column: 0,
            row: 1,
        }]
    );
}

#[test]
fn expel_to_the_right_moves_one_window() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (second, _) = open(&mut layout, 0, Some(first));
    layout.apply(
        SessionAction::ConsumeOrExpel {
            window: second,
            direction: Direction::Left,
        },
        AREA,
        &LayoutOptions::default(),
    );
    let events = layout.apply(
        SessionAction::ConsumeOrExpel {
            window: second,
            direction: Direction::Right,
        },
        AREA,
        &LayoutOptions::default(),
    );
    assert_eq!(
        events,
        [LayoutEvent::WindowMoved {
            window: second,
            band: band(&layout, 0),
            column: 1,
            row: 0,
        }]
    );
}

#[test]
fn consume_at_the_edge_produces_nothing() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (second, _) = open(&mut layout, 0, Some(first));
    let before = layout.clone();
    for (window, direction) in [(first, Direction::Left), (second, Direction::Right)] {
        assert!(
            layout
                .apply(
                    SessionAction::ConsumeOrExpel { window, direction },
                    AREA,
                    &LayoutOptions::default()
                )
                .is_empty()
        );
    }
    assert_eq!(layout, before);
}

#[test]
fn cycling_and_toggling_widths_report_the_column() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (second, _) = open(&mut layout, 0, Some(first));
    let ws = band(&layout, 0);
    let changed = |width, full_width| {
        vec![LayoutEvent::ColumnWidthChanged {
            band: ws,
            column: 1,
            width,
            full_width,
        }]
    };
    assert_eq!(
        layout.apply(
            SessionAction::CycleWidth(second),
            AREA,
            &LayoutOptions::default()
        ),
        changed(Proportion::TWO_THIRDS, false)
    );
    assert_eq!(
        layout.apply(
            SessionAction::CycleWidth(second),
            AREA,
            &LayoutOptions::default()
        ),
        changed(Proportion::ONE_THIRD, false)
    );
    assert_eq!(
        layout.apply(
            SessionAction::ToggleFullWidth(second),
            AREA,
            &LayoutOptions::default()
        ),
        changed(Proportion::ONE_THIRD, true)
    );
    assert_eq!(
        layout.apply(
            SessionAction::ToggleFullWidth(second),
            AREA,
            &LayoutOptions::default()
        ),
        changed(Proportion::ONE_THIRD, false)
    );
}

#[test]
fn actions_on_a_missing_window_produce_nothing() {
    let mut layout = Layout::new();
    open(&mut layout, 0, None);
    let missing = WindowId(99);
    assert!(layout.remove(missing).is_empty());
    assert!(
        layout
            .apply(
                SessionAction::CycleWidth(missing),
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
}

#[test]
fn growing_a_column_width_reports_it_once() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let grow = SessionAction::StepWidth {
        window: first,
        step: Step::Grow,
        by: Proportion::TENTH,
    };
    assert_eq!(
        layout.apply(grow, AREA, &LayoutOptions::default()),
        [LayoutEvent::ColumnWidthChanged {
            band: band(&layout, 0),
            column: 0,
            width: Proportion::new(3, 5),
            full_width: false,
        }]
    );
    let shrink = SessionAction::StepWidth {
        window: first,
        step: Step::Shrink,
        by: Proportion::TENTH,
    };
    for _ in 0..6 {
        layout.apply(shrink.clone(), AREA, &LayoutOptions::default());
    }
    assert!(
        layout
            .apply(shrink.clone(), AREA, &LayoutOptions::default())
            .is_empty()
    );
}

fn second_column_stack() -> (Layout, WindowId, WindowId) {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (top, _) = open(&mut layout, 0, Some(first));
    let (bottom, _) = open(&mut layout, 0, Some(top));
    layout.apply(
        SessionAction::ConsumeOrExpel {
            window: bottom,
            direction: Direction::Left,
        },
        AREA,
        &LayoutOptions::default(),
    );
    (layout, top, bottom)
}

#[test]
fn growing_a_window_height_reports_the_column() {
    let (mut layout, top, _) = second_column_stack();
    assert_eq!(
        layout.apply(
            SessionAction::StepHeight {
                window: top,
                step: Step::Grow,
                by: Proportion::TENTH,
            },
            AREA,
            &LayoutOptions::default()
        ),
        [LayoutEvent::WindowHeightsChanged {
            band: band(&layout, 0),
            column: 1,
            heights: vec![WindowHeight::Fixed(14), WindowHeight::Auto(Weight::ONE)],
        }]
    );
}

#[test]
fn height_steps_at_the_limits_produce_nothing() {
    for (step, presses) in [(Step::Grow, 5), (Step::Shrink, 5)] {
        let (mut layout, top, _) = second_column_stack();
        let action = SessionAction::StepHeight {
            window: top,
            step,
            by: Proportion::TENTH,
        };
        for _ in 0..presses {
            assert!(
                !layout
                    .apply(action.clone(), AREA, &LayoutOptions::default())
                    .is_empty()
            );
        }
        let before = layout.clone();
        assert!(
            layout
                .apply(action.clone(), AREA, &LayoutOptions::default())
                .is_empty()
        );
        assert_eq!(layout, before);
    }
}

fn apply(layout: &mut Layout, action: SessionAction) -> Vec<LayoutEvent> {
    layout.apply(action, AREA, &LayoutOptions::default())
}

fn row_of_three() -> (Layout, [WindowId; 3]) {
    let mut layout = Layout::new();
    let (a, _) = open(&mut layout, 0, None);
    let (b, _) = open(&mut layout, 0, Some(a));
    let (c, _) = open(&mut layout, 0, Some(b));
    (layout, [a, b, c])
}

#[test]
fn moving_a_column_reports_both_positions() {
    let (mut layout, [a, _, _]) = row_of_three();
    let direction = Direction::Right;
    assert_eq!(
        apply(
            &mut layout,
            SessionAction::MoveColumn {
                window: a,
                direction
            }
        ),
        [LayoutEvent::ColumnMoved {
            band: band(&layout, 0),
            from: 0,
            to: 1,
        }]
    );
}

#[test]
fn swapping_two_windows_reports_each() {
    let (mut layout, [_, _, c]) = row_of_three();
    let (p2, _) = open(&mut layout, 0, Some(c));
    let direction = Direction::Left;
    apply(
        &mut layout,
        SessionAction::ConsumeOrExpel {
            window: p2,
            direction,
        },
    );
    assert_eq!(layout.locate(c).unwrap().column, 2);
    let id = band(&layout, 0);
    let direction = Vertical::Down;
    assert_eq!(
        apply(
            &mut layout,
            SessionAction::MoveWindow {
                window: c,
                direction
            }
        ),
        [
            LayoutEvent::WindowMoved {
                window: c,
                band: id,
                column: 2,
                row: 1,
            },
            LayoutEvent::WindowMoved {
                window: p2,
                band: id,
                column: 2,
                row: 0,
            },
        ]
    );
}

#[test]
fn floating_a_window_reports_its_box() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (p2, _) = open(&mut layout, 0, Some(first));
    let id = band(&layout, 0);
    assert_eq!(
        apply(
            &mut layout,
            SessionAction::ToggleFloating {
                window: p2,
                after: None,
            }
        ),
        [LayoutEvent::WindowFloated {
            window: p2,
            band: id,
            record: FloatingWindow {
                window: p2,
                col: 20,
                row: 2,
                width: Proportion::ONE_HALF,
                full_width: false,
                rows: 20,
            },
        }]
    );
}

#[test]
fn moving_a_floating_window_reports_its_new_box() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (window, _) = open(&mut layout, 0, Some(first));
    apply(
        &mut layout,
        SessionAction::ToggleFloating {
            window,
            after: None,
        },
    );
    let mut moved = *layout.floating(window).unwrap();
    moved.col = 28;
    let direction = Direction::Right;
    assert_eq!(
        apply(&mut layout, SessionAction::MoveColumn { window, direction }),
        [LayoutEvent::FloatingBoxChanged {
            window,
            band: band(&layout, 0),
            record: moved,
        }]
    );
}

#[test]
fn tiling_a_window_reports_its_column() {
    let (mut layout, [a, b, _]) = row_of_three();
    apply(
        &mut layout,
        SessionAction::ToggleFloating {
            window: b,
            after: None,
        },
    );
    let width = Proportion::ONE_THIRD;
    apply(&mut layout, SessionAction::SetWidth { window: b, width });
    assert_eq!(
        apply(
            &mut layout,
            SessionAction::ToggleFloating {
                window: b,
                after: Some(a)
            }
        ),
        [LayoutEvent::WindowTiled {
            window: b,
            band: band(&layout, 0),
            column: 1,
            width,
            full_width: false,
        }]
    );
}

#[test]
fn opening_floating_in_the_empty_band() {
    let mut layout = Layout::new();
    let window = layout.allocate_window();
    let empty = band(&layout, 0);
    let events = layout.open_floating(window, empty, None, AREA, &LayoutOptions::default());
    let record = *layout.floating(window).unwrap();
    assert_eq!(
        events,
        [
            LayoutEvent::WindowOpened {
                window,
                band: empty
            },
            LayoutEvent::WindowFloated {
                window,
                band: empty,
                record,
            },
            LayoutEvent::BandAdded {
                band: band(&layout, 1),
                index: 1,
            },
        ]
    );
}

#[test]
fn floating_ops_without_change_produce_nothing() {
    let (mut layout, [a, _, _]) = row_of_three();
    let direction = Direction::Left;
    assert!(
        apply(
            &mut layout,
            SessionAction::MoveColumn {
                window: a,
                direction
            }
        )
        .is_empty()
    );
    let (window, col, row) = (a, 3, 3);
    assert!(apply(&mut layout, SessionAction::SetPosition { window, col, row }).is_empty());
}
