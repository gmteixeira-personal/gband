use gband_core::geometry::Size;
use gband_core::layout::{Direction, Layout, PaneId, SessionAction};
use gband_core::view::{Scene, View, ViewAction};

const AREA: Size = Size::new(80, 24);

fn scene(layout: &Layout) -> Scene<'_> {
    Scene {
        layout,
        area: AREA,
        viewport_cols: 80,
    }
}

fn open(layout: &mut Layout, workspace: usize, after: Option<PaneId>) -> PaneId {
    let pane = layout.allocate_pane();
    let id = layout.workspaces()[workspace].id;
    assert!(layout.open(pane, id, after));
    pane
}

fn row_of_columns(count: usize) -> (Layout, Vec<PaneId>) {
    let mut layout = Layout::new();
    let mut panes = vec![open(&mut layout, 0, None)];
    for _ in 1..count {
        let last = *panes.last().unwrap();
        panes.push(open(&mut layout, 0, Some(last)));
    }
    (layout, panes)
}

fn stack_into_left(layout: &mut Layout, pane: PaneId) {
    layout.apply(SessionAction::ConsumeOrExpel {
        pane,
        direction: Direction::Left,
    });
}

fn act(view: &mut View, layout: &Layout, actions: &[ViewAction]) {
    for &action in actions {
        view.apply(action, scene(layout));
    }
}

#[test]
fn initial_view_focuses_the_first_pane() {
    let (layout, panes) = row_of_columns(2);
    let view = View::new(scene(&layout));
    assert_eq!(view.workspace(), layout.workspaces()[0].id);
    assert_eq!(view.focused(), Some(panes[0]));
    assert_eq!(view.camera(), 0);
}

#[test]
fn initial_view_of_an_empty_session() {
    let layout = Layout::new();
    let view = View::new(scene(&layout));
    assert_eq!(view.focused(), None);
}

#[test]
fn focus_moves_across_columns_and_stops_at_the_edges() {
    let (layout, panes) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.focused(), Some(panes[0]));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    assert_eq!(view.focused(), Some(panes[2]));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(panes[2]));
}

#[test]
fn focus_returns_to_the_remembered_pane() {
    let (mut layout, panes) = row_of_columns(3);
    stack_into_left(&mut layout, panes[1]);
    let (p2, b) = (panes[1], panes[2]);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusDown]);
    assert_eq!(view.focused(), Some(p2));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(b));
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.focused(), Some(p2));
}

#[test]
fn unvisited_column_focuses_its_top_pane() {
    let (mut layout, panes) = row_of_columns(3);
    stack_into_left(&mut layout, panes[2]);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(panes[1]));
}

#[test]
fn focus_moves_within_a_stack() {
    let (mut layout, panes) = row_of_columns(3);
    stack_into_left(&mut layout, panes[1]);
    stack_into_left(&mut layout, panes[2]);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusUp]);
    assert_eq!(view.focused(), Some(panes[0]));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusDown, ViewAction::FocusDown],
    );
    assert_eq!(view.focused(), Some(panes[2]));
    act(&mut view, &layout, &[ViewAction::FocusDown]);
    assert_eq!(view.focused(), Some(panes[2]));
}

#[test]
fn switching_down_to_the_empty_workspace_and_back() {
    let (layout, panes) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    act(&mut view, &layout, &[ViewAction::WorkspaceDown]);
    assert_eq!(view.workspace(), layout.workspaces()[1].id);
    assert_eq!(view.focused(), None);
    act(&mut view, &layout, &[ViewAction::WorkspaceDown]);
    assert_eq!(view.workspace(), layout.workspaces()[1].id);
    act(&mut view, &layout, &[ViewAction::WorkspaceUp]);
    assert_eq!(view.workspace(), layout.workspaces()[0].id);
    assert_eq!(view.focused(), Some(panes[1]));
    act(&mut view, &layout, &[ViewAction::WorkspaceUp]);
    assert_eq!(view.workspace(), layout.workspaces()[0].id);
    assert_eq!(view.focused(), Some(panes[1]));
}

#[test]
fn views_of_one_layout_are_independent() {
    let (layout, panes) = row_of_columns(2);
    let mut first = View::new(scene(&layout));
    let mut second = View::new(scene(&layout));
    act(&mut first, &layout, &[ViewAction::FocusRight]);
    act(&mut second, &layout, &[ViewAction::FocusRight]);
    act(&mut first, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(first.focused(), Some(panes[0]));
    assert_eq!(second.focused(), Some(panes[1]));
}

#[test]
fn closing_the_focused_column_focuses_the_one_to_its_right() {
    let (mut layout, panes) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    layout.remove(panes[1]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(panes[2]));
}

#[test]
fn closing_the_last_column_focuses_the_one_to_its_left() {
    let (mut layout, panes) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    layout.remove(panes[2]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(panes[1]));
}

#[test]
fn closing_a_focused_stacked_pane_focuses_the_one_below() {
    let (mut layout, panes) = row_of_columns(3);
    stack_into_left(&mut layout, panes[1]);
    stack_into_left(&mut layout, panes[2]);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusDown]);
    layout.remove(panes[1]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(panes[2]));
    layout.remove(panes[2]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(panes[0]));
}

#[test]
fn focused_pane_expelled_keeps_focus() {
    let (mut layout, panes) = row_of_columns(2);
    stack_into_left(&mut layout, panes[1]);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusDown]);
    layout.apply(SessionAction::ConsumeOrExpel {
        pane: panes[1],
        direction: Direction::Right,
    });
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(panes[1]));
    assert_eq!(layout.locate(panes[1]).unwrap().column, 1);
}

#[test]
fn viewed_workspace_removed_moves_to_the_one_in_its_place() {
    let mut layout = Layout::new();
    let first = open(&mut layout, 0, None);
    let other = open(&mut layout, 1, None);
    let w2 = layout.workspaces()[1].id;
    let mut view = View::new(scene(&layout));
    layout.remove(first);
    view.sync(scene(&layout));
    assert_eq!(view.workspace(), w2);
    assert_eq!(view.focused(), Some(other));
}

#[test]
fn last_workspace_removed_moves_to_the_new_last() {
    let mut layout = Layout::new();
    open(&mut layout, 0, None);
    let other = open(&mut layout, 1, None);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::WorkspaceDown]);
    assert_eq!(view.focused(), Some(other));
    layout.remove(other);
    view.sync(scene(&layout));
    assert_eq!(view.workspace(), layout.workspaces()[1].id);
    assert_eq!(view.focused(), None);
}

#[test]
fn pane_opened_in_the_viewed_empty_workspace_gains_focus() {
    let mut layout = Layout::new();
    let mut view = View::new(scene(&layout));
    let pane = open(&mut layout, 0, None);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(pane));
}

#[test]
fn focus_pane_switches_to_its_workspace() {
    let (mut layout, _) = row_of_columns(1);
    let mut view = View::new(scene(&layout));
    let pane = open(&mut layout, 1, None);
    view.focus_pane(pane, scene(&layout));
    assert_eq!(view.workspace(), layout.workspaces()[1].id);
    assert_eq!(view.focused(), Some(pane));
}

#[test]
fn camera_scrolls_just_enough_and_back() {
    let (layout, panes) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 0);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(panes[2]));
    assert_eq!(view.camera(), 40);
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.camera(), 40);
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.camera(), 0);
}

#[test]
fn camera_aligns_a_wide_column_to_its_start() {
    let (mut layout, panes) = row_of_columns(2);
    layout.apply(SessionAction::ToggleFullWidth(panes[1]));
    let mut view = View::new(Scene {
        layout: &layout,
        area: Size::new(100, 24),
        viewport_cols: 80,
    });
    view.apply(
        ViewAction::FocusRight,
        Scene {
            layout: &layout,
            area: Size::new(100, 24),
            viewport_cols: 80,
        },
    );
    assert_eq!(view.camera(), 50);
}

#[test]
fn camera_follows_a_narrower_terminal() {
    let (layout, _) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 0);
    view.sync(Scene {
        layout: &layout,
        area: AREA,
        viewport_cols: 60,
    });
    assert_eq!(view.camera(), 20);
}

#[test]
fn each_workspace_keeps_its_camera() {
    let mut layout = Layout::new();
    let mut last = open(&mut layout, 0, None);
    for _ in 0..2 {
        last = open(&mut layout, 0, Some(last));
    }
    open(&mut layout, 1, None);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    assert_eq!(view.camera(), 40);
    act(&mut view, &layout, &[ViewAction::WorkspaceDown]);
    assert_eq!(view.camera(), 0);
    act(&mut view, &layout, &[ViewAction::WorkspaceUp]);
    assert_eq!(view.camera(), 40);
    assert_eq!(view.focused(), Some(last));
}
