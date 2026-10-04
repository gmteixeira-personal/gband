use std::collections::HashSet;

use gband_core::layout::{Direction, Layout, PaneId, Proportion, SessionAction, WorkspaceId};

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
    assert!(layout.open(pane, id, after));
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
    }
    let panes: Vec<_> = layout.panes().collect();
    let unique: HashSet<_> = panes.iter().collect();
    assert_eq!(panes.len(), unique.len(), "a pane appears twice");
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
    assert!(layout.remove(panes[1]));
    let next = open(&mut layout, 0, Some(panes[0]));
    assert_ne!(next, panes[1]);

    assert!(layout.remove(panes[0]));
    assert!(layout.remove(next));
    open(&mut layout, 0, None);
    assert_ne!(workspace_id(&layout, 0), first_workspace);
}

#[test]
fn opens_right_of_the_named_column() {
    let (mut layout, panes) = row_of_columns(2);
    let a = panes[0];
    let b = panes[1];
    assert!(layout.apply(SessionAction::CycleWidth(b)));
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
    assert!(!layout.open(pane, WorkspaceId(99), None));
    let second = workspace_id(&layout, 1);
    assert!(!layout.open(pane, second, Some(first)));
    assert!(!layout.open(pane, second, Some(PaneId(99))));
    assert_eq!(columns(&layout, 0), vec![vec![first]]);
}

#[test]
fn column_is_removed_with_its_last_pane() {
    let (mut layout, panes) = row_of_columns(3);
    assert!(layout.remove(panes[1]));
    assert_eq!(columns(&layout, 0), vec![vec![panes[0]], vec![panes[2]]]);
}

#[test]
fn pane_leaves_a_stack_and_the_column_keeps_its_width() {
    let (mut layout, panes) = row_of_columns(3);
    let (p1, p2, p3) = (panes[0], panes[1], panes[2]);
    layout.apply(SessionAction::ConsumeOrExpel {
        pane: p2,
        direction: Direction::Left,
    });
    layout.apply(SessionAction::ConsumeOrExpel {
        pane: p3,
        direction: Direction::Left,
    });
    layout.apply(SessionAction::CycleWidth(p1));
    assert_eq!(columns(&layout, 0), vec![vec![p1, p2, p3]]);
    assert!(layout.remove(p2));
    assert_eq!(columns(&layout, 0), vec![vec![p1, p3]]);
    assert_eq!(width_of(&layout, p1), (Proportion::TWO_THIRDS, false));
}

#[test]
fn emptied_middle_workspace_is_removed() {
    let (mut layout, first) = started();
    let w2 = workspace_id(&layout, 1);
    let other = open(&mut layout, 1, None);
    let w3 = workspace_id(&layout, 2);
    assert!(layout.remove(first));
    assert_eq!(layout.workspaces().len(), 2);
    assert_eq!(workspace_id(&layout, 0), w2);
    assert_eq!(workspace_id(&layout, 1), w3);
    assert_eq!(columns(&layout, 0), vec![vec![other]]);
}

#[test]
fn removing_the_last_pane_leaves_one_empty_workspace() {
    let (mut layout, first) = started();
    assert!(layout.remove(first));
    assert_eq!(layout.workspaces().len(), 1);
    assert!(layout.is_empty());
    assert!(!layout.remove(first));
}

#[test]
fn expel_to_the_left() {
    let (mut layout, panes) = row_of_columns(3);
    let (a, p1, p2) = (panes[0], panes[1], panes[2]);
    layout.apply(SessionAction::ConsumeOrExpel {
        pane: p2,
        direction: Direction::Left,
    });
    assert_eq!(columns(&layout, 0), vec![vec![a], vec![p1, p2]]);
    assert!(layout.apply(SessionAction::ConsumeOrExpel {
        pane: p2,
        direction: Direction::Left,
    }));
    assert_eq!(columns(&layout, 0), vec![vec![a], vec![p2], vec![p1]]);
    assert_eq!(width_of(&layout, p2), (Proportion::ONE_HALF, false));
}

#[test]
fn expel_to_the_right() {
    let (mut layout, panes) = row_of_columns(2);
    let (p1, p2) = (panes[0], panes[1]);
    layout.apply(SessionAction::ConsumeOrExpel {
        pane: p2,
        direction: Direction::Left,
    });
    layout.apply(SessionAction::CycleWidth(p1));
    assert!(layout.apply(SessionAction::ConsumeOrExpel {
        pane: p1,
        direction: Direction::Right,
    }));
    assert_eq!(columns(&layout, 0), vec![vec![p2], vec![p1]]);
    assert_eq!(width_of(&layout, p1), (Proportion::ONE_HALF, false));
    assert_eq!(width_of(&layout, p2), (Proportion::TWO_THIRDS, false));
}

#[test]
fn consume_to_the_right_keeps_the_target_width() {
    let (mut layout, panes) = row_of_columns(3);
    let (p1, p2, p3) = (panes[0], panes[1], panes[2]);
    layout.apply(SessionAction::ConsumeOrExpel {
        pane: p3,
        direction: Direction::Left,
    });
    layout.apply(SessionAction::CycleWidth(p2));
    assert!(layout.apply(SessionAction::ConsumeOrExpel {
        pane: p1,
        direction: Direction::Right,
    }));
    assert_eq!(columns(&layout, 0), vec![vec![p2, p3, p1]]);
    assert_eq!(width_of(&layout, p1), (Proportion::TWO_THIRDS, false));
}

#[test]
fn consume_to_the_left() {
    let (mut layout, panes) = row_of_columns(3);
    let (a, b, c) = (panes[0], panes[1], panes[2]);
    assert!(layout.apply(SessionAction::ConsumeOrExpel {
        pane: c,
        direction: Direction::Left,
    }));
    assert_eq!(columns(&layout, 0), vec![vec![a], vec![b, c]]);
}

#[test]
fn consume_at_the_edge_changes_nothing() {
    let (mut layout, panes) = row_of_columns(2);
    let before = layout.clone();
    assert!(!layout.apply(SessionAction::ConsumeOrExpel {
        pane: panes[0],
        direction: Direction::Left,
    }));
    assert!(!layout.apply(SessionAction::ConsumeOrExpel {
        pane: panes[1],
        direction: Direction::Right,
    }));
    assert_eq!(layout, before);
}

#[test]
fn width_cycles_through_the_presets() {
    let (mut layout, first) = started();
    let mut seen = Vec::new();
    for _ in 0..3 {
        layout.apply(SessionAction::CycleWidth(first));
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
    layout.apply(SessionAction::CycleWidth(first));
    layout.apply(SessionAction::ToggleFullWidth(first));
    assert_eq!(width_of(&layout, first), (Proportion::TWO_THIRDS, true));
    layout.apply(SessionAction::CycleWidth(first));
    assert_eq!(width_of(&layout, first), (Proportion::ONE_THIRD, false));
}

#[test]
fn toggling_full_width_twice_keeps_the_proportion() {
    let (mut layout, first) = started();
    layout.apply(SessionAction::CycleWidth(first));
    layout.apply(SessionAction::CycleWidth(first));
    layout.apply(SessionAction::ToggleFullWidth(first));
    layout.apply(SessionAction::ToggleFullWidth(first));
    assert_eq!(width_of(&layout, first), (Proportion::ONE_THIRD, false));
}

#[test]
fn actions_on_a_missing_pane_change_nothing() {
    let (mut layout, _) = started();
    let before = layout.clone();
    let missing = PaneId(99);
    assert!(!layout.apply(SessionAction::CycleWidth(missing)));
    assert!(!layout.apply(SessionAction::ToggleFullWidth(missing)));
    assert!(!layout.apply(SessionAction::ConsumeOrExpel {
        pane: missing,
        direction: Direction::Left,
    }));
    assert_eq!(layout, before);
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
                assert!(layout.open(pane, target, named));
                panes.push(pane);
            }
            Remove(index) => {
                let pane = pick(index, &panes);
                assert!(layout.remove(pane));
                panes.retain(|&candidate| candidate != pane);
            }
            Shift(index, direction) => {
                layout.apply(SessionAction::ConsumeOrExpel {
                    pane: pick(index, &panes),
                    direction,
                });
            }
            Cycle(index) => {
                layout.apply(SessionAction::CycleWidth(pick(index, &panes)));
            }
            Full(index) => {
                layout.apply(SessionAction::ToggleFullWidth(pick(index, &panes)));
            }
        }
        assert_invariants(&layout);
        let mut present: Vec<_> = layout.panes().collect();
        let mut expected = panes.clone();
        present.sort();
        expected.sort();
        assert_eq!(present, expected);
        if panes.is_empty() {
            break;
        }
    }
}
