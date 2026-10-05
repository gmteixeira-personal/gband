use std::collections::HashSet;

use gband_core::event::LayoutEvent;
use gband_core::geometry::{Size, pane_heights};
use gband_core::layout::Step::{Grow, Shrink};
use gband_core::layout::{
    Column, Direction, Layout, Location, PaneHeight, PaneId, Proportion, SessionAction, Weight,
    WorkspaceId,
};

const AREA: Size = Size::new(80, 24);

fn columns(layout: &Layout, workspace: usize) -> Vec<Vec<PaneId>> {
    layout.workspaces()[workspace]
        .columns
        .iter()
        .map(|column| column.panes.clone())
        .collect()
}

fn workspace_id(layout: &Layout, index: usize) -> WorkspaceId {
    layout.workspaces()[index].id
}

fn open(layout: &mut Layout, workspace: usize, after: Option<PaneId>) -> PaneId {
    let pane = layout.allocate_pane();
    let id = workspace_id(layout, workspace);
    assert!(!layout.open(pane, id, after).is_empty());
    pane
}

fn started() -> (Layout, PaneId) {
    let mut layout = Layout::new();
    let first = open(&mut layout, 0, None);
    (layout, first)
}

fn row_of_columns(count: usize) -> (Layout, Vec<PaneId>) {
    let (mut layout, first) = started();
    let mut panes = vec![first];
    for _ in 1..count {
        let last = *panes.last().unwrap();
        panes.push(open(&mut layout, 0, Some(last)));
    }
    (layout, panes)
}

fn width_of(layout: &Layout, pane: PaneId) -> (Proportion, bool) {
    let location = layout.locate(pane).unwrap();
    let column = &layout.workspaces()[location.workspace].columns[location.column];
    (column.width, column.full_width)
}

fn assert_invariants(layout: &Layout) {
    let workspaces = layout.workspaces();
    assert!(!workspaces.is_empty());
    let (last, rest) = workspaces.split_last().unwrap();
    assert!(last.is_empty(), "the last workspace must be empty");
    assert!(rest.iter().all(|workspace| !workspace.is_empty()));
    for workspace in workspaces {
        assert!(
            workspace
                .columns
                .iter()
                .all(|column| !column.panes.is_empty())
        );
        assert!(
            workspace
                .columns
                .iter()
                .all(|column| column.panes.len() == column.heights.len()),
            "a column's heights differ in length from its panes"
        );
    }
    let panes: Vec<_> = layout.panes().collect();
    let unique: HashSet<_> = panes.iter().collect();
    assert_eq!(panes.len(), unique.len(), "a pane appears twice");
}

fn shift(layout: &mut Layout, pane: PaneId, direction: Direction) -> Vec<LayoutEvent> {
    let events = layout.apply(SessionAction::ConsumeOrExpel { pane, direction }, AREA);
    assert_invariants(layout);
    events
}

#[test]
fn new_layout_holds_one_empty_workspace() {
    let layout = Layout::new();
    assert_eq!(layout.workspaces().len(), 1);
    assert!(layout.is_empty());
}

#[test]
fn initial_layout_has_the_first_pane_and_an_empty_workspace() {
    let (layout, first) = started();
    assert_eq!(layout.workspaces().len(), 2);
    assert_eq!(columns(&layout, 0), vec![vec![first]]);
    assert!(layout.workspaces()[1].is_empty());
    assert_eq!(width_of(&layout, first), (Proportion::ONE_HALF, false));
}

#[test]
fn identifiers_are_not_reused() {
    let (mut layout, panes) = row_of_columns(2);
    let first_workspace = workspace_id(&layout, 0);
    assert!(!layout.remove(panes[1]).is_empty());
    let next = open(&mut layout, 0, Some(panes[0]));
    assert_ne!(next, panes[1]);

    assert!(!layout.remove(panes[0]).is_empty());
    assert!(!layout.remove(next).is_empty());
    open(&mut layout, 0, None);
    assert_ne!(workspace_id(&layout, 0), first_workspace);
}

#[test]
fn opens_right_of_the_named_column() {
    let (mut layout, panes) = row_of_columns(2);
    let a = panes[0];
    let b = panes[1];
    assert!(!layout.apply(SessionAction::CycleWidth(b), AREA).is_empty());
    let new = open(&mut layout, 0, Some(a));
    assert_eq!(columns(&layout, 0), vec![vec![a], vec![new], vec![b]]);
    assert_eq!(width_of(&layout, a), (Proportion::ONE_HALF, false));
    assert_eq!(width_of(&layout, b), (Proportion::TWO_THIRDS, false));
}

#[test]
fn opens_as_the_first_column_without_a_named_pane() {
    let (mut layout, first) = started();
    let new = open(&mut layout, 0, None);
    assert_eq!(columns(&layout, 0), vec![vec![new], vec![first]]);
}

#[test]
fn opening_in_the_empty_workspace_adds_another() {
    let (mut layout, first) = started();
    let w1 = workspace_id(&layout, 0);
    let w2 = workspace_id(&layout, 1);
    let new = open(&mut layout, 1, None);
    assert_eq!(layout.workspaces().len(), 3);
    assert_eq!(workspace_id(&layout, 0), w1);
    assert_eq!(workspace_id(&layout, 1), w2);
    assert_eq!(columns(&layout, 0), vec![vec![first]]);
    assert_eq!(columns(&layout, 1), vec![vec![new]]);
    assert!(layout.workspaces()[2].is_empty());
}

#[test]
fn open_refuses_unknown_targets() {
    let (mut layout, first) = started();
    let pane = layout.allocate_pane();
    assert!(layout.open(pane, WorkspaceId(99), None).is_empty());
    let second = workspace_id(&layout, 1);
    assert!(layout.open(pane, second, Some(first)).is_empty());
    assert!(layout.open(pane, second, Some(PaneId(99))).is_empty());
    assert_eq!(columns(&layout, 0), vec![vec![first]]);
}

#[test]
fn column_is_removed_with_its_last_pane() {
    let (mut layout, panes) = row_of_columns(3);
    assert!(!layout.remove(panes[1]).is_empty());
    assert_eq!(columns(&layout, 0), vec![vec![panes[0]], vec![panes[2]]]);
}

#[test]
fn pane_leaves_a_stack_and_the_column_keeps_its_width() {
    let (mut layout, panes) = row_of_columns(3);
    let (p1, p2, p3) = (panes[0], panes[1], panes[2]);
    shift(&mut layout, p2, Direction::Left);
    shift(&mut layout, p3, Direction::Left);
    layout.apply(SessionAction::CycleWidth(p1), AREA);
    assert_eq!(columns(&layout, 0), vec![vec![p1, p2, p3]]);
    assert!(!layout.remove(p2).is_empty());
    assert_eq!(columns(&layout, 0), vec![vec![p1, p3]]);
    assert_eq!(width_of(&layout, p1), (Proportion::TWO_THIRDS, false));
}

#[test]
fn emptied_middle_workspace_is_removed() {
    let (mut layout, first) = started();
    let w2 = workspace_id(&layout, 1);
    let other = open(&mut layout, 1, None);
    let w3 = workspace_id(&layout, 2);
    assert!(!layout.remove(first).is_empty());
    assert_eq!(layout.workspaces().len(), 2);
    assert_eq!(workspace_id(&layout, 0), w2);
    assert_eq!(workspace_id(&layout, 1), w3);
    assert_eq!(columns(&layout, 0), vec![vec![other]]);
}

#[test]
fn removing_the_last_pane_leaves_one_empty_workspace() {
    let (mut layout, first) = started();
    assert!(!layout.remove(first).is_empty());
    assert_eq!(layout.workspaces().len(), 1);
    assert!(layout.is_empty());
    assert!(layout.remove(first).is_empty());
}

#[test]
fn expel_to_the_left() {
    let (mut layout, panes) = row_of_columns(3);
    let (a, p1, p2) = (panes[0], panes[1], panes[2]);
    shift(&mut layout, p2, Direction::Left);
    assert_eq!(columns(&layout, 0), vec![vec![a], vec![p1, p2]]);
    assert!(!shift(&mut layout, p2, Direction::Left).is_empty());
    assert_eq!(columns(&layout, 0), vec![vec![a], vec![p2], vec![p1]]);
    assert_eq!(width_of(&layout, p2), (Proportion::ONE_HALF, false));
}

#[test]
fn expel_to_the_right() {
    let (mut layout, panes) = row_of_columns(2);
    let (p1, p2) = (panes[0], panes[1]);
    shift(&mut layout, p2, Direction::Left);
    layout.apply(SessionAction::CycleWidth(p1), AREA);
    assert!(!shift(&mut layout, p1, Direction::Right).is_empty());
    assert_eq!(columns(&layout, 0), vec![vec![p2], vec![p1]]);
    assert_eq!(width_of(&layout, p1), (Proportion::ONE_HALF, false));
    assert_eq!(width_of(&layout, p2), (Proportion::TWO_THIRDS, false));
}

#[test]
fn consume_to_the_right_keeps_the_target_width() {
    let (mut layout, panes) = row_of_columns(3);
    let (p1, p2, p3) = (panes[0], panes[1], panes[2]);
    shift(&mut layout, p3, Direction::Left);
    layout.apply(SessionAction::CycleWidth(p2), AREA);
    assert!(!shift(&mut layout, p1, Direction::Right).is_empty());
    assert_eq!(columns(&layout, 0), vec![vec![p2, p3, p1]]);
    assert_eq!(width_of(&layout, p1), (Proportion::TWO_THIRDS, false));
}

#[test]
fn consume_to_the_left() {
    let (mut layout, panes) = row_of_columns(3);
    let (a, b, c) = (panes[0], panes[1], panes[2]);
    assert!(!shift(&mut layout, c, Direction::Left).is_empty());
    assert_eq!(columns(&layout, 0), vec![vec![a], vec![b, c]]);
}

#[test]
fn consume_at_the_edge_changes_nothing() {
    let (mut layout, panes) = row_of_columns(2);
    let before = layout.clone();
    assert!(shift(&mut layout, panes[0], Direction::Left).is_empty());
    assert!(shift(&mut layout, panes[1], Direction::Right).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn width_cycles_through_the_presets() {
    let (mut layout, first) = started();
    let mut seen = Vec::new();
    for _ in 0..3 {
        layout.apply(SessionAction::CycleWidth(first), AREA);
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

#[test]
fn cycling_from_full_width_wraps_to_one_third() {
    let (mut layout, first) = started();
    layout.apply(SessionAction::CycleWidth(first), AREA);
    layout.apply(SessionAction::ToggleFullWidth(first), AREA);
    assert_eq!(width_of(&layout, first), (Proportion::TWO_THIRDS, true));
    layout.apply(SessionAction::CycleWidth(first), AREA);
    assert_eq!(width_of(&layout, first), (Proportion::ONE_THIRD, false));
}

#[test]
fn toggling_full_width_twice_keeps_the_proportion() {
    let (mut layout, first) = started();
    layout.apply(SessionAction::CycleWidth(first), AREA);
    layout.apply(SessionAction::CycleWidth(first), AREA);
    layout.apply(SessionAction::ToggleFullWidth(first), AREA);
    layout.apply(SessionAction::ToggleFullWidth(first), AREA);
    assert_eq!(width_of(&layout, first), (Proportion::ONE_THIRD, false));
}

#[test]
fn actions_on_a_missing_pane_change_nothing() {
    let (mut layout, _) = started();
    let before = layout.clone();
    let missing = PaneId(99);
    assert!(
        layout
            .apply(SessionAction::CycleWidth(missing), AREA)
            .is_empty()
    );
    assert!(step_width(&mut layout, missing, Grow).is_empty());
    assert!(step_height(&mut layout, missing, Grow, AREA).is_empty());
    assert!(
        layout
            .apply(SessionAction::ResetHeight(missing), AREA)
            .is_empty()
    );
    assert!(
        layout
            .apply(SessionAction::ToggleFullWidth(missing), AREA)
            .is_empty()
    );
    assert!(shift(&mut layout, missing, Direction::Left).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn opening_a_pane_the_layout_already_holds_changes_nothing() {
    let (mut layout, first) = started();
    let before = layout.clone();
    let (current, empty) = (workspace_id(&layout, 0), workspace_id(&layout, 1));
    assert!(layout.open(first, current, None).is_empty());
    assert!(layout.open(first, empty, None).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn opening_and_closing_are_left_to_the_server() {
    let (mut layout, first) = started();
    let before = layout.clone();
    let workspace = workspace_id(&layout, 0);
    assert!(
        layout
            .apply(
                SessionAction::OpenPane {
                    workspace,
                    after: Some(first),
                },
                AREA
            )
            .is_empty()
    );
    assert!(
        layout
            .apply(SessionAction::ClosePane(first), AREA)
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
fn first_pane_is_the_top_of_the_leftmost_column() {
    let (mut layout, panes) = row_of_columns(2);
    let leftmost = open(&mut layout, 0, None);
    shift(&mut layout, panes[0], Direction::Left);
    assert_eq!(
        columns(&layout, 0),
        vec![vec![leftmost, panes[0]], vec![panes[1]]]
    );
    assert_eq!(layout.workspaces()[0].first_pane(), Some(leftmost));
    assert_eq!(layout.workspaces()[1].first_pane(), None);
}

#[test]
fn workspace_index_follows_removals() {
    let (mut layout, first) = started();
    let other = open(&mut layout, 1, None);
    let ids = [0, 1, 2].map(|index| workspace_id(&layout, index));
    assert_eq!(
        ids.map(|id| layout.workspace_index(id)),
        [0, 1, 2].map(Some)
    );
    layout.remove(first);
    assert_eq!(
        ids.map(|id| layout.workspace_index(id)),
        [None, Some(0), Some(1)]
    );
    assert_eq!(columns(&layout, 0), vec![vec![other]]);
    assert_eq!(layout.workspace_index(WorkspaceId(99)), None);
}

fn stack_of_three() -> (Layout, [PaneId; 3]) {
    let (mut layout, panes) = row_of_columns(3);
    for &pane in &panes[1..] {
        shift(&mut layout, pane, Direction::Left);
    }
    assert_eq!(columns(&layout, 0), vec![panes.clone()]);
    (layout, [panes[0], panes[1], panes[2]])
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
    assert_eq!(PaneId(7).to_string(), "7");
    assert_eq!(WorkspaceId(12).to_string(), "12");
}

#[test]
fn default_layout_is_a_new_layout() {
    assert_eq!(Layout::default(), Layout::new());
}

#[derive(Clone, Copy)]
enum Step {
    Open { workspace: usize, after: usize },
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
        Open {
            workspace: 0,
            after: 0,
        },
        Open {
            workspace: 0,
            after: 1,
        },
        Shift(2, Left),
        Open {
            workspace: 1,
            after: 0,
        },
        Open {
            workspace: 1,
            after: 3,
        },
        Shift(1, Right),
        Cycle(4),
        Full(3),
        Shift(3, Left),
        Open {
            workspace: 2,
            after: 9,
        },
        Remove(0),
        Shift(4, Right),
        Remove(2),
        Shift(5, Left),
        Shift(5, Left),
        Remove(3),
        Open {
            workspace: 0,
            after: 4,
        },
        Remove(1),
        Remove(4),
        Open {
            workspace: 1,
            after: 5,
        },
        Remove(5),
        Remove(6),
    ];
    let (mut layout, first) = started();
    let mut panes = vec![first];
    assert_invariants(&layout);
    for step in steps {
        let pick = |index: usize, panes: &[PaneId]| panes[index % panes.len()];
        match step {
            Open { workspace, after } => {
                let count = layout.workspaces().len();
                let target = layout.workspaces()[workspace % count].id;
                let named = layout.workspaces()[workspace % count]
                    .panes()
                    .nth(after % 4)
                    .or_else(|| layout.workspaces()[workspace % count].panes().next());
                let pane = layout.allocate_pane();
                assert!(!panes.contains(&pane), "{pane} was reused");
                let events = layout.open(pane, target, named);
                assert_eq!(
                    events[0],
                    LayoutEvent::PaneOpened {
                        pane,
                        workspace: target
                    }
                );
                panes.push(pane);
            }
            Remove(index) => {
                let pane = pick(index, &panes);
                let events = layout.remove(pane);
                assert!(
                    matches!(events[0], LayoutEvent::PaneClosed { pane: closed, .. } if closed == pane)
                );
                panes.retain(|&candidate| candidate != pane);
            }
            Shift(index, direction) => {
                let pane = pick(index, &panes);
                let events = shift(&mut layout, pane, direction);
                for event in events {
                    let LayoutEvent::PaneMoved {
                        pane: moved,
                        column,
                        row,
                        ..
                    } = event
                    else {
                        panic!("a shift reported {event:?}");
                    };
                    assert_eq!(moved, pane);
                    let location = layout.locate(pane).unwrap();
                    assert_eq!((location.column, location.row), (column, row));
                }
            }
            Cycle(index) => {
                layout.apply(SessionAction::CycleWidth(pick(index, &panes)), AREA);
            }
            Full(index) => {
                layout.apply(SessionAction::ToggleFullWidth(pick(index, &panes)), AREA);
            }
        }
        assert_invariants(&layout);
        let mut present: Vec<_> = layout.panes().collect();
        let mut expected = panes.clone();
        present.sort();
        expected.sort();
        assert_eq!(present, expected, "every pane keeps its identifier");
        if panes.is_empty() {
            break;
        }
    }
}

#[test]
fn identifier_survives_moves() {
    let (mut layout, panes) = row_of_columns(2);
    let (first, second) = (panes[0], panes[1]);
    let at = |column, row| Location {
        workspace: 0,
        column,
        row,
    };
    let steps = [
        (
            SessionAction::ConsumeOrExpel {
                pane: second,
                direction: Direction::Left,
            },
            at(0, 1),
        ),
        (
            SessionAction::ConsumeOrExpel {
                pane: second,
                direction: Direction::Right,
            },
            at(1, 0),
        ),
        (SessionAction::CycleWidth(second), at(1, 0)),
    ];
    for (action, location) in steps {
        assert!(!layout.apply(action, AREA).is_empty(), "{action:?}");
        let mut present: Vec<_> = layout.panes().collect();
        present.sort();
        assert_eq!(present, [first, second], "{action:?}");
        assert_eq!(layout.locate(second), Some(location), "{action:?}");
    }
}

type Resize = gband_core::layout::Step;

fn step_width(layout: &mut Layout, pane: PaneId, step: Resize) -> Vec<LayoutEvent> {
    layout.apply(SessionAction::StepWidth { pane, step }, AREA)
}

fn widths_while(layout: &mut Layout, pane: PaneId, step: Resize, times: usize) -> Vec<Proportion> {
    (0..times)
        .map(|_| {
            step_width(layout, pane, step);
            let (width, full_width) = width_of(layout, pane);
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
    layout.apply(SessionAction::CycleWidth(first), AREA);
    layout.apply(SessionAction::CycleWidth(first), AREA);
    assert_eq!(width_of(&layout, first), (Proportion::ONE_THIRD, false));
    assert_eq!(
        widths_while(&mut layout, first, Grow, 1),
        [Proportion::new(13, 30)]
    );
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
    let mut column = Column::new(PaneId(1));
    column.width = Proportion::new(Proportion::MAX, 1);
    let before = column.clone();
    column.step_width(Grow);
    assert_eq!(column, before);
    assert_eq!(
        Proportion::new(99999, 10).step(Grow),
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
        layout.apply(SessionAction::CycleWidth(first), AREA);
        layout.apply(SessionAction::CycleWidth(first), AREA);
        layout.apply(SessionAction::ToggleFullWidth(first), AREA);
        assert_eq!(width_of(&layout, first), (Proportion::ONE_THIRD, true));
        assert!(!step_width(&mut layout, first, step).is_empty());
        assert_eq!(width_of(&layout, first), (expected, false));
    }
}

#[test]
fn proportion_steps_stay_in_lowest_terms() {
    assert_eq!(Proportion::new(2, 4).step(Grow), Proportion::new(3, 5));
    assert_eq!(Proportion::new(1, 10).step(Shrink), Proportion::new(0, 1));
    assert_eq!(Proportion::new(0, 1).step(Shrink), Proportion::new(0, 1));
    assert_eq!(Proportion::new(9, 10).step(Grow), Proportion::WHOLE);
}

#[test]
fn proportion_of_saturates_at_the_cell_limit() {
    assert_eq!(Proportion::new(11, 10).of(80), 88);
    assert_eq!(Proportion::new(10000, 1).of(80), u16::MAX);
    assert_eq!(Proportion::new(0, 1).of(80), 0);
}

fn stacked(count: usize) -> (Layout, Vec<PaneId>) {
    let (mut layout, panes) = row_of_columns(count);
    for &pane in &panes[1..] {
        shift(&mut layout, pane, Direction::Left);
    }
    assert_eq!(columns(&layout, 0), vec![panes.clone()]);
    (layout, panes)
}

fn column_of(layout: &Layout, pane: PaneId) -> &Column {
    let location = layout.locate(pane).unwrap();
    &layout.workspaces()[location.workspace].columns[location.column]
}

fn heights_of(layout: &Layout, pane: PaneId) -> Vec<PaneHeight> {
    column_of(layout, pane).heights.clone()
}

fn rows_of(layout: &Layout, pane: PaneId, area: Size) -> Vec<u16> {
    pane_heights(column_of(layout, pane), area.rows)
}

fn step_height(layout: &mut Layout, pane: PaneId, step: Resize, area: Size) -> Vec<LayoutEvent> {
    layout.apply(SessionAction::StepHeight { pane, step }, area)
}

fn auto(num: u16, den: u16) -> PaneHeight {
    PaneHeight::Auto(Weight::new(num, den))
}

#[test]
fn each_height_step_adds_the_same_rows() {
    let (mut layout, panes) = stacked(2);
    let mut seen = Vec::new();
    for _ in 0..3 {
        assert!(!step_height(&mut layout, panes[0], Grow, AREA).is_empty());
        seen.push(rows_of(&layout, panes[0], AREA));
    }
    assert_eq!(seen, [[14, 10], [16, 8], [18, 6]]);
    assert_eq!(
        heights_of(&layout, panes[0]),
        [PaneHeight::Fixed(18), auto(1, 1)]
    );
}

#[test]
fn height_shrinks_in_a_stack_of_two() {
    let (mut layout, panes) = stacked(2);
    step_height(&mut layout, panes[0], Shrink, AREA);
    assert_eq!(
        heights_of(&layout, panes[0]),
        [PaneHeight::Fixed(10), auto(1, 1)]
    );
    assert_eq!(rows_of(&layout, panes[0], AREA), [10, 14]);
}

#[test]
fn height_step_rounds_half_up_on_an_odd_area() {
    let (mut layout, panes) = stacked(2);
    let area = Size::new(80, 25);
    step_height(&mut layout, panes[0], Grow, area);
    assert_eq!(heights_of(&layout, panes[0])[0], PaneHeight::Fixed(16));
    assert_eq!(rows_of(&layout, panes[0], area), [16, 9]);
}

#[test]
fn resizing_another_pane_keeps_the_earlier_one_larger() {
    let (mut layout, panes) = stacked(3);
    step_height(&mut layout, panes[0], Grow, AREA);
    assert_eq!(rows_of(&layout, panes[0], AREA), [10, 7, 7]);
    step_height(&mut layout, panes[2], Grow, AREA);
    assert_eq!(
        heights_of(&layout, panes[0]),
        [auto(10, 7), auto(1, 1), PaneHeight::Fixed(9)]
    );
    assert_eq!(rows_of(&layout, panes[0], AREA), [9, 6, 9]);
}

#[test]
fn height_stops_at_the_upper_limit() {
    let (mut layout, panes) = stacked(3);
    for _ in 0..5 {
        step_height(&mut layout, panes[0], Grow, AREA);
    }
    assert_eq!(heights_of(&layout, panes[0])[0], PaneHeight::Fixed(18));
    let before = layout.clone();
    assert!(step_height(&mut layout, panes[0], Grow, AREA).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn height_stops_at_the_lower_limit() {
    let (mut layout, panes) = stacked(2);
    for _ in 0..5 {
        step_height(&mut layout, panes[0], Shrink, AREA);
    }
    assert_eq!(heights_of(&layout, panes[0])[0], PaneHeight::Fixed(3));
    let before = layout.clone();
    assert!(step_height(&mut layout, panes[0], Shrink, AREA).is_empty());
    assert_eq!(layout, before);
}

#[test]
fn lone_pane_shrinks_below_the_area() {
    let (mut layout, first) = started();
    step_height(&mut layout, first, Shrink, AREA);
    assert_eq!(heights_of(&layout, first), [PaneHeight::Fixed(22)]);
    assert_eq!(rows_of(&layout, first, AREA), [22]);
}

#[test]
fn reset_height_returns_to_weight_one() {
    let (mut layout, panes) = stacked(2);
    step_height(&mut layout, panes[0], Grow, AREA);
    step_height(&mut layout, panes[0], Grow, AREA);
    assert_eq!(heights_of(&layout, panes[0])[0], PaneHeight::Fixed(16));
    assert!(
        !layout
            .apply(SessionAction::ResetHeight(panes[0]), AREA)
            .is_empty()
    );
    assert_eq!(heights_of(&layout, panes[0]), [auto(1, 1), auto(1, 1)]);
    assert_eq!(rows_of(&layout, panes[0], AREA), [12, 12]);
    assert!(
        layout
            .apply(SessionAction::ResetHeight(panes[0]), AREA)
            .is_empty()
    );
}

#[test]
fn consume_next_to_a_fixed_height() {
    let (mut layout, panes) = row_of_columns(3);
    let (p1, p2, p3) = (panes[0], panes[1], panes[2]);
    shift(&mut layout, p3, Direction::Left);
    step_height(&mut layout, p2, Grow, AREA);
    step_height(&mut layout, p2, Grow, AREA);
    shift(&mut layout, p1, Direction::Right);
    assert_eq!(columns(&layout, 0), vec![vec![p2, p3, p1]]);
    assert_eq!(
        heights_of(&layout, p2),
        [PaneHeight::Fixed(16), auto(1, 1), auto(1, 1)]
    );
    assert_eq!(rows_of(&layout, p2, AREA), [16, 4, 4]);
}

#[test]
fn last_pane_in_a_column_takes_weight_one() {
    let (mut layout, panes) = row_of_columns(4);
    let (p1, middle, p2, p3) = (panes[0], panes[1], panes[2], panes[3]);
    shift(&mut layout, middle, Direction::Left);
    shift(&mut layout, p2, Direction::Left);
    step_height(&mut layout, p1, Grow, AREA);
    step_height(&mut layout, p2, Grow, AREA);
    layout.remove(middle);
    assert_invariants(&layout);
    assert_eq!(heights_of(&layout, p1), [auto(10, 7), PaneHeight::Fixed(9)]);
    layout.remove(p2);
    assert_invariants(&layout);
    assert_eq!(heights_of(&layout, p1), [auto(1, 1)]);
    shift(&mut layout, p3, Direction::Left);
    assert_eq!(columns(&layout, 0), vec![vec![p1, p3]]);
    assert_eq!(heights_of(&layout, p1), [auto(1, 1), auto(1, 1)]);
    assert_eq!(rows_of(&layout, p1, AREA), [12, 12]);
}

#[test]
fn expelled_pane_takes_weight_one_and_the_rest_keep_theirs() {
    let (mut layout, panes) = stacked(3);
    step_height(&mut layout, panes[0], Grow, AREA);
    step_height(&mut layout, panes[2], Grow, AREA);
    shift(&mut layout, panes[0], Direction::Right);
    assert_eq!(
        heights_of(&layout, panes[1]),
        [auto(1, 1), PaneHeight::Fixed(9)]
    );
    assert_eq!(heights_of(&layout, panes[0]), [auto(1, 1)]);
}

#[test]
fn weights_are_held_in_lowest_terms() {
    assert_eq!(Weight::new(14, 7), Weight::new(2, 1));
    let weight = Weight::new(20, 14);
    assert_eq!((weight.num(), weight.den()), (10, 7));
    assert_eq!(Weight::ONE, Weight::new(5, 5));
}
