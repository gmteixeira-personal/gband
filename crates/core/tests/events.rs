use gband_core::event::LayoutEvent;
use gband_core::geometry::Size;
use gband_core::layout::{
    BandId, Direction, FloatingPane, Layout, LayoutOptions, PaneHeight, PaneId, Proportion,
    SessionAction, Step, Vertical, Weight,
};

const AREA: Size = Size::new(80, 24);

fn open(layout: &mut Layout, band: usize, after: Option<PaneId>) -> (PaneId, Vec<LayoutEvent>) {
    let pane = layout.allocate_pane();
    let id = layout.bands()[band].id;
    let events = layout.open(pane, id, after, None, &LayoutOptions::default());
    (pane, events)
}

fn band(layout: &Layout, index: usize) -> BandId {
    layout.bands()[index].id
}

#[test]
fn open_in_the_empty_band_adds_one_below() {
    let mut layout = Layout::new();
    let empty = band(&layout, 0);
    let (pane, events) = open(&mut layout, 0, None);
    assert_eq!(
        events,
        [
            LayoutEvent::PaneOpened { pane, band: empty },
            LayoutEvent::BandAdded {
                band: band(&layout, 1),
                index: 1,
            },
        ]
    );
}

#[test]
fn open_beside_a_pane_adds_no_band() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (second, events) = open(&mut layout, 0, Some(first));
    assert_eq!(
        events,
        [LayoutEvent::PaneOpened {
            pane: second,
            band: band(&layout, 0),
        }]
    );
}

#[test]
fn last_pane_of_a_middle_band_closing_removes_it() {
    let mut layout = Layout::new();
    let (top, _) = open(&mut layout, 0, None);
    open(&mut layout, 1, None);
    let middle = band(&layout, 0);
    assert_eq!(
        layout.remove(top),
        [
            LayoutEvent::PaneClosed {
                pane: top,
                band: middle,
            },
            LayoutEvent::BandRemoved { band: middle },
        ]
    );
}

#[test]
fn closing_the_only_pane_removes_its_band() {
    let mut layout = Layout::new();
    let (pane, _) = open(&mut layout, 0, None);
    let filled = band(&layout, 0);
    assert_eq!(
        layout.remove(pane),
        [
            LayoutEvent::PaneClosed { pane, band: filled },
            LayoutEvent::BandRemoved { band: filled },
        ]
    );
}

#[test]
fn consume_into_the_left_neighbour_moves_one_pane() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (second, _) = open(&mut layout, 0, Some(first));
    let events = layout.apply(
        SessionAction::ConsumeOrExpel {
            pane: second,
            direction: Direction::Left,
        },
        AREA,
        &LayoutOptions::default(),
    );
    assert_eq!(
        events,
        [LayoutEvent::PaneMoved {
            pane: second,
            band: band(&layout, 0),
            column: 0,
            row: 1,
        }]
    );
}

#[test]
fn expel_to_the_right_moves_one_pane() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (second, _) = open(&mut layout, 0, Some(first));
    layout.apply(
        SessionAction::ConsumeOrExpel {
            pane: second,
            direction: Direction::Left,
        },
        AREA,
        &LayoutOptions::default(),
    );
    let events = layout.apply(
        SessionAction::ConsumeOrExpel {
            pane: second,
            direction: Direction::Right,
        },
        AREA,
        &LayoutOptions::default(),
    );
    assert_eq!(
        events,
        [LayoutEvent::PaneMoved {
            pane: second,
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
    for (pane, direction) in [(first, Direction::Left), (second, Direction::Right)] {
        assert!(
            layout
                .apply(
                    SessionAction::ConsumeOrExpel { pane, direction },
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
fn actions_on_a_missing_pane_produce_nothing() {
    let mut layout = Layout::new();
    open(&mut layout, 0, None);
    let missing = PaneId(99);
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
        pane: first,
        step: Step::Grow,
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
        pane: first,
        step: Step::Shrink,
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

fn second_column_stack() -> (Layout, PaneId, PaneId) {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (top, _) = open(&mut layout, 0, Some(first));
    let (bottom, _) = open(&mut layout, 0, Some(top));
    layout.apply(
        SessionAction::ConsumeOrExpel {
            pane: bottom,
            direction: Direction::Left,
        },
        AREA,
        &LayoutOptions::default(),
    );
    (layout, top, bottom)
}

#[test]
fn growing_a_pane_height_reports_the_column() {
    let (mut layout, top, _) = second_column_stack();
    assert_eq!(
        layout.apply(
            SessionAction::StepHeight {
                pane: top,
                step: Step::Grow,
            },
            AREA,
            &LayoutOptions::default()
        ),
        [LayoutEvent::PaneHeightsChanged {
            band: band(&layout, 0),
            column: 1,
            heights: vec![PaneHeight::Fixed(14), PaneHeight::Auto(Weight::ONE)],
        }]
    );
}

#[test]
fn height_steps_at_the_limits_produce_nothing() {
    for (step, presses) in [(Step::Grow, 5), (Step::Shrink, 5)] {
        let (mut layout, top, _) = second_column_stack();
        let action = SessionAction::StepHeight { pane: top, step };
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

fn row_of_three() -> (Layout, [PaneId; 3]) {
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
            SessionAction::MoveColumn { pane: a, direction }
        ),
        [LayoutEvent::ColumnMoved {
            band: band(&layout, 0),
            from: 0,
            to: 1,
        }]
    );
}

#[test]
fn swapping_two_panes_reports_each() {
    let (mut layout, [_, _, c]) = row_of_three();
    let (p2, _) = open(&mut layout, 0, Some(c));
    let direction = Direction::Left;
    apply(
        &mut layout,
        SessionAction::ConsumeOrExpel {
            pane: p2,
            direction,
        },
    );
    assert_eq!(layout.locate(c).unwrap().column, 2);
    let id = band(&layout, 0);
    let direction = Vertical::Down;
    assert_eq!(
        apply(&mut layout, SessionAction::MovePane { pane: c, direction }),
        [
            LayoutEvent::PaneMoved {
                pane: c,
                band: id,
                column: 2,
                row: 1,
            },
            LayoutEvent::PaneMoved {
                pane: p2,
                band: id,
                column: 2,
                row: 0,
            },
        ]
    );
}

#[test]
fn floating_a_pane_reports_its_box() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (p2, _) = open(&mut layout, 0, Some(first));
    let id = band(&layout, 0);
    assert_eq!(
        apply(
            &mut layout,
            SessionAction::ToggleFloating {
                pane: p2,
                after: None,
            }
        ),
        [LayoutEvent::PaneFloated {
            pane: p2,
            band: id,
            record: FloatingPane {
                pane: p2,
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
fn moving_a_floating_pane_reports_its_new_box() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (pane, _) = open(&mut layout, 0, Some(first));
    apply(
        &mut layout,
        SessionAction::ToggleFloating { pane, after: None },
    );
    let mut moved = *layout.floating(pane).unwrap();
    moved.col = 28;
    let direction = Direction::Right;
    assert_eq!(
        apply(&mut layout, SessionAction::MoveColumn { pane, direction }),
        [LayoutEvent::FloatingBoxChanged {
            pane,
            band: band(&layout, 0),
            record: moved,
        }]
    );
}

#[test]
fn tiling_a_pane_reports_its_column() {
    let (mut layout, [a, b, _]) = row_of_three();
    apply(
        &mut layout,
        SessionAction::ToggleFloating {
            pane: b,
            after: None,
        },
    );
    let width = Proportion::ONE_THIRD;
    apply(&mut layout, SessionAction::SetWidth { pane: b, width });
    assert_eq!(
        apply(
            &mut layout,
            SessionAction::ToggleFloating {
                pane: b,
                after: Some(a)
            }
        ),
        [LayoutEvent::PaneTiled {
            pane: b,
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
    let pane = layout.allocate_pane();
    let empty = band(&layout, 0);
    let events = layout.open_floating(pane, empty, None, AREA, &LayoutOptions::default());
    let record = *layout.floating(pane).unwrap();
    assert_eq!(
        events,
        [
            LayoutEvent::PaneOpened { pane, band: empty },
            LayoutEvent::PaneFloated {
                pane,
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
            SessionAction::MoveColumn { pane: a, direction }
        )
        .is_empty()
    );
    let (pane, col, row) = (a, 3, 3);
    assert!(apply(&mut layout, SessionAction::SetPosition { pane, col, row }).is_empty());
}
