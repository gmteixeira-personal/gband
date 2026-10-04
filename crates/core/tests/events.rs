use gband_core::event::LayoutEvent;
use gband_core::layout::{Direction, Layout, PaneId, Proportion, SessionAction, WorkspaceId};

fn open(
    layout: &mut Layout,
    workspace: usize,
    after: Option<PaneId>,
) -> (PaneId, Vec<LayoutEvent>) {
    let pane = layout.allocate_pane();
    let id = layout.workspaces()[workspace].id;
    let events = layout.open(pane, id, after);
    (pane, events)
}

fn workspace(layout: &Layout, index: usize) -> WorkspaceId {
    layout.workspaces()[index].id
}

#[test]
fn open_in_the_empty_workspace_adds_one_below() {
    let mut layout = Layout::new();
    let empty = workspace(&layout, 0);
    let (pane, events) = open(&mut layout, 0, None);
    assert_eq!(
        events,
        [
            LayoutEvent::PaneOpened {
                pane,
                workspace: empty,
            },
            LayoutEvent::WorkspaceAdded {
                workspace: workspace(&layout, 1),
                index: 1,
            },
        ]
    );
}

#[test]
fn open_beside_a_pane_adds_no_workspace() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (second, events) = open(&mut layout, 0, Some(first));
    assert_eq!(
        events,
        [LayoutEvent::PaneOpened {
            pane: second,
            workspace: workspace(&layout, 0),
        }]
    );
}

#[test]
fn last_pane_of_a_middle_workspace_closing_removes_it() {
    let mut layout = Layout::new();
    let (top, _) = open(&mut layout, 0, None);
    open(&mut layout, 1, None);
    let middle = workspace(&layout, 0);
    assert_eq!(
        layout.remove(top),
        [
            LayoutEvent::PaneClosed {
                pane: top,
                workspace: middle,
            },
            LayoutEvent::WorkspaceRemoved { workspace: middle },
        ]
    );
}

#[test]
fn closing_the_only_pane_removes_its_workspace() {
    let mut layout = Layout::new();
    let (pane, _) = open(&mut layout, 0, None);
    let filled = workspace(&layout, 0);
    assert_eq!(
        layout.remove(pane),
        [
            LayoutEvent::PaneClosed {
                pane,
                workspace: filled,
            },
            LayoutEvent::WorkspaceRemoved { workspace: filled },
        ]
    );
}

#[test]
fn consume_into_the_left_neighbour_moves_one_pane() {
    let mut layout = Layout::new();
    let (first, _) = open(&mut layout, 0, None);
    let (second, _) = open(&mut layout, 0, Some(first));
    let events = layout.apply(SessionAction::ConsumeOrExpel {
        pane: second,
        direction: Direction::Left,
    });
    assert_eq!(
        events,
        [LayoutEvent::PaneMoved {
            pane: second,
            workspace: workspace(&layout, 0),
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
    layout.apply(SessionAction::ConsumeOrExpel {
        pane: second,
        direction: Direction::Left,
    });
    let events = layout.apply(SessionAction::ConsumeOrExpel {
        pane: second,
        direction: Direction::Right,
    });
    assert_eq!(
        events,
        [LayoutEvent::PaneMoved {
            pane: second,
            workspace: workspace(&layout, 0),
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
                .apply(SessionAction::ConsumeOrExpel { pane, direction })
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
    let ws = workspace(&layout, 0);
    let changed = |width, full_width| {
        vec![LayoutEvent::ColumnWidthChanged {
            workspace: ws,
            column: 1,
            width,
            full_width,
        }]
    };
    assert_eq!(
        layout.apply(SessionAction::CycleWidth(second)),
        changed(Proportion::TWO_THIRDS, false)
    );
    assert_eq!(
        layout.apply(SessionAction::CycleWidth(second)),
        changed(Proportion::ONE_THIRD, false)
    );
    assert_eq!(
        layout.apply(SessionAction::ToggleFullWidth(second)),
        changed(Proportion::ONE_THIRD, true)
    );
    assert_eq!(
        layout.apply(SessionAction::ToggleFullWidth(second)),
        changed(Proportion::ONE_THIRD, false)
    );
}

#[test]
fn actions_on_a_missing_pane_produce_nothing() {
    let mut layout = Layout::new();
    open(&mut layout, 0, None);
    let missing = PaneId(99);
    assert!(layout.remove(missing).is_empty());
    assert!(layout.apply(SessionAction::CycleWidth(missing)).is_empty());
    assert!(
        layout
            .apply(SessionAction::ToggleFullWidth(missing))
            .is_empty()
    );
}
