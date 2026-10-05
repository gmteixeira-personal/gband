use gband_core::action::SessionCommand;
use gband_core::geometry::Size;
use gband_core::layout::{
    Direction, Layout, LayoutOptions, PaneId, Proportion, SessionAction, Step,
};
use gband_core::view::{CenterFocusedColumn, Scene, View, ViewAction};

const AREA: Size = Size::new(80, 24);

fn scene(layout: &Layout) -> Scene<'_> {
    Scene {
        layout,
        area: AREA,
        viewport: Size::new(80, 24),
    }
}

fn open(layout: &mut Layout, band: usize, after: Option<PaneId>) -> PaneId {
    let pane = layout.allocate_pane();
    let id = layout.bands()[band].id;
    assert!(
        !layout
            .open(pane, id, after, None, &LayoutOptions::default())
            .is_empty()
    );
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
    layout.apply(
        SessionAction::ConsumeOrExpel {
            pane,
            direction: Direction::Left,
        },
        AREA,
        &LayoutOptions::default(),
    );
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
    assert_eq!(view.band(), layout.bands()[0].id);
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
fn switching_down_to_the_empty_band_and_back() {
    let (layout, panes) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    act(&mut view, &layout, &[ViewAction::BandDown]);
    assert_eq!(view.band(), layout.bands()[1].id);
    assert_eq!(view.focused(), None);
    act(&mut view, &layout, &[ViewAction::BandDown]);
    assert_eq!(view.band(), layout.bands()[1].id);
    act(&mut view, &layout, &[ViewAction::BandUp]);
    assert_eq!(view.band(), layout.bands()[0].id);
    assert_eq!(view.focused(), Some(panes[1]));
    act(&mut view, &layout, &[ViewAction::BandUp]);
    assert_eq!(view.band(), layout.bands()[0].id);
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
    layout.apply(
        SessionAction::ConsumeOrExpel {
            pane: panes[1],
            direction: Direction::Right,
        },
        AREA,
        &LayoutOptions::default(),
    );
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(panes[1]));
    assert_eq!(layout.locate(panes[1]).unwrap().column, 1);
}

#[test]
fn viewed_band_removed_moves_to_the_one_in_its_place() {
    let mut layout = Layout::new();
    let first = open(&mut layout, 0, None);
    let other = open(&mut layout, 1, None);
    let w2 = layout.bands()[1].id;
    let mut view = View::new(scene(&layout));
    layout.remove(first);
    view.sync(scene(&layout));
    assert_eq!(view.band(), w2);
    assert_eq!(view.focused(), Some(other));
}

#[test]
fn view_action_after_the_viewed_band_is_removed_only_moves_the_view() {
    let mut layout = Layout::new();
    let first = open(&mut layout, 0, None);
    let other = open(&mut layout, 1, None);
    open(&mut layout, 1, Some(other));
    let w2 = layout.bands()[1].id;
    let mut view = View::new(scene(&layout));
    layout.remove(first);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.band(), w2);
    assert_eq!(view.focused(), Some(other));
}

#[test]
fn last_band_removed_moves_to_the_new_last() {
    let mut layout = Layout::new();
    open(&mut layout, 0, None);
    let other = open(&mut layout, 1, None);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::BandDown]);
    assert_eq!(view.focused(), Some(other));
    layout.remove(other);
    view.sync(scene(&layout));
    assert_eq!(view.band(), layout.bands()[1].id);
    assert_eq!(view.focused(), None);
}

#[test]
fn pane_opened_in_the_viewed_empty_band_gains_focus() {
    let mut layout = Layout::new();
    let mut view = View::new(scene(&layout));
    let pane = open(&mut layout, 0, None);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(pane));
}

#[test]
fn focus_pane_switches_to_its_band() {
    let (mut layout, _) = row_of_columns(1);
    let mut view = View::new(scene(&layout));
    let pane = open(&mut layout, 1, None);
    view.focus_pane(pane, scene(&layout));
    assert_eq!(view.band(), layout.bands()[1].id);
    assert_eq!(view.focused(), Some(pane));
}

#[test]
fn focus_pane_ignores_a_pane_the_layout_does_not_hold() {
    let (layout, panes) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    view.focus_pane(PaneId(99), scene(&layout));
    assert_eq!(view.band(), layout.bands()[0].id);
    assert_eq!(view.focused(), Some(panes[1]));
}

#[test]
fn focus_moves_on_an_empty_band_change_nothing() {
    let layout = Layout::new();
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[
            ViewAction::FocusLeft,
            ViewAction::FocusRight,
            ViewAction::FocusUp,
            ViewAction::FocusDown,
        ],
    );
    assert_eq!(view.band(), layout.bands()[0].id);
    assert_eq!(view.focused(), None);
}

#[test]
fn focus_move_after_an_unsynced_close_lands_where_a_sync_would() {
    for action in [
        ViewAction::FocusLeft,
        ViewAction::FocusRight,
        ViewAction::FocusUp,
        ViewAction::FocusDown,
    ] {
        let (mut layout, panes) = row_of_columns(3);
        let mut view = View::new(scene(&layout));
        act(&mut view, &layout, &[ViewAction::FocusRight]);
        layout.remove(panes[1]);
        act(&mut view, &layout, &[action]);
        assert_eq!(view.focused(), Some(panes[2]), "{action:?}");
    }
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
    layout.apply(
        SessionAction::ToggleFullWidth(panes[1]),
        AREA,
        &LayoutOptions::default(),
    );
    let mut view = View::new(Scene {
        layout: &layout,
        area: Size::new(100, 24),
        viewport: Size::new(80, 24),
    });
    view.apply(
        ViewAction::FocusRight,
        Scene {
            layout: &layout,
            area: Size::new(100, 24),
            viewport: Size::new(80, 24),
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
        viewport: Size::new(60, 24),
    });
    assert_eq!(view.camera(), 20);
}

fn view_with(layout: &Layout, policy: CenterFocusedColumn) -> View {
    View::with_policy(scene(layout), policy)
}

fn columns_of(width: Proportion, count: usize) -> (Layout, Vec<PaneId>) {
    let options = LayoutOptions {
        default_width: width,
        ..LayoutOptions::default()
    };
    let mut layout = Layout::new();
    let mut panes = Vec::new();
    for _ in 0..count {
        let pane = layout.allocate_pane();
        let id = layout.bands()[0].id;
        layout.open(pane, id, panes.last().copied(), None, &options);
        panes.push(pane);
    }
    (layout, panes)
}

#[test]
fn always_centres_the_focused_column() {
    let (layout, _) = row_of_columns(3);
    let mut view = view_with(&layout, CenterFocusedColumn::Always);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 20);
}

#[test]
fn always_centres_the_first_column_left_of_the_strip() {
    let (layout, _) = row_of_columns(1);
    let view = view_with(&layout, CenterFocusedColumn::Always);
    assert_eq!(view.camera(), -20);
}

#[test]
fn always_aligns_a_column_wider_than_the_terminal() {
    let (layout, _) = columns_of(Proportion::new(3, 2), 2);
    let mut view = view_with(&layout, CenterFocusedColumn::Always);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 120);
}

#[test]
fn on_overflow_scrolls_just_enough_when_the_pair_fits() {
    let (layout, _) = row_of_columns(3);
    let mut view = view_with(&layout, CenterFocusedColumn::OnOverflow);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 0);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 40);
}

#[test]
fn on_overflow_centres_when_the_pair_does_not_fit() {
    let (layout, _) = columns_of(Proportion::TWO_THIRDS, 3);
    let mut view = view_with(&layout, CenterFocusedColumn::OnOverflow);
    assert_eq!(view.camera(), 0);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 40);
}

#[test]
fn on_overflow_follows_a_width_change_as_never() {
    let (mut layout, panes) = row_of_columns(2);
    let mut view = view_with(&layout, CenterFocusedColumn::OnOverflow);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 0);
    layout.apply(
        SessionAction::StepWidth {
            pane: panes[1],
            step: Step::Grow,
        },
        AREA,
        &LayoutOptions::default(),
    );
    view.sync(scene(&layout));
    assert_eq!(view.camera(), 8);
}

#[test]
fn on_overflow_centres_a_new_column_that_does_not_fit_beside_the_focus() {
    let (mut layout, panes) = columns_of(Proportion::TWO_THIRDS, 1);
    let mut view = view_with(&layout, CenterFocusedColumn::OnOverflow);
    let opened = layout.allocate_pane();
    let options = LayoutOptions {
        default_width: Proportion::TWO_THIRDS,
        ..LayoutOptions::default()
    };
    layout.open(opened, layout.bands()[0].id, Some(panes[0]), None, &options);
    view.sync(scene(&layout));
    view.focus_pane(opened, scene(&layout));
    assert_eq!(view.camera(), 40);
}

#[test]
fn policy_change_applies_from_the_next_move() {
    let (layout, _) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    view.set_center_focused_column(CenterFocusedColumn::Always);
    assert_eq!(view.camera(), 0);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 20);
}

#[test]
fn each_band_keeps_its_camera() {
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
    act(&mut view, &layout, &[ViewAction::BandDown]);
    assert_eq!(view.camera(), 0);
    act(&mut view, &layout, &[ViewAction::BandUp]);
    assert_eq!(view.camera(), 40);
    assert_eq!(view.focused(), Some(last));
}

#[test]
fn session_commands_resolve_to_the_focused_pane() {
    let (layout, panes) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    assert_eq!(view.focused(), Some(panes[2]));
    let band = layout.bands()[0].id;
    let cases = [
        (
            SessionCommand::CycleWidth,
            SessionAction::CycleWidth(panes[2]),
        ),
        (
            SessionCommand::ToggleFullWidth,
            SessionAction::ToggleFullWidth(panes[2]),
        ),
        (
            SessionCommand::ClosePane,
            SessionAction::ClosePane(panes[2]),
        ),
        (
            SessionCommand::ConsumeOrExpel(Direction::Left),
            SessionAction::ConsumeOrExpel {
                pane: panes[2],
                direction: Direction::Left,
            },
        ),
        (
            SessionCommand::OpenPane,
            SessionAction::open(band, Some(panes[2]), None),
        ),
        (
            SessionCommand::StepWidth(Step::Grow),
            SessionAction::StepWidth {
                pane: panes[2],
                step: Step::Grow,
            },
        ),
        (
            SessionCommand::StepWidth(Step::Shrink),
            SessionAction::StepWidth {
                pane: panes[2],
                step: Step::Shrink,
            },
        ),
        (
            SessionCommand::StepHeight(Step::Grow),
            SessionAction::StepHeight {
                pane: panes[2],
                step: Step::Grow,
            },
        ),
        (
            SessionCommand::StepHeight(Step::Shrink),
            SessionAction::StepHeight {
                pane: panes[2],
                step: Step::Shrink,
            },
        ),
        (
            SessionCommand::ResetHeight,
            SessionAction::ResetHeight(panes[2]),
        ),
    ];
    for (command, expected) in cases {
        assert_eq!(view.resolve(command), Some(expected), "{command:?}");
    }
}

#[test]
fn commands_on_a_pane_resolve_to_nothing_without_focus() {
    let layout = Layout::new();
    let view = View::new(scene(&layout));
    assert_eq!(view.focused(), None);
    for command in [
        SessionCommand::ClosePane,
        SessionCommand::ConsumeOrExpel(Direction::Left),
        SessionCommand::ConsumeOrExpel(Direction::Right),
        SessionCommand::CycleWidth,
        SessionCommand::ToggleFullWidth,
        SessionCommand::StepWidth(Step::Grow),
        SessionCommand::StepHeight(Step::Shrink),
        SessionCommand::ResetHeight,
    ] {
        assert_eq!(view.resolve(command), None, "{command:?}");
    }
}

#[test]
fn open_pane_on_the_empty_band_names_it_and_no_pane() {
    let (layout, _) = row_of_columns(1);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::BandDown]);
    let empty = layout.bands()[1].id;
    assert_eq!(view.band(), empty);
    assert_eq!(
        view.resolve(SessionCommand::OpenPane),
        Some(SessionAction::open(empty, None, None))
    );
}

#[test]
fn column_beyond_the_right_edge_is_not_shown() {
    let (layout, panes) = row_of_columns(3);
    let view = View::new(scene(&layout));
    assert_eq!(view.shown(scene(&layout)), [panes[0], panes[1]]);
}

#[test]
fn partly_visible_column_is_shown() {
    let (mut layout, panes) = row_of_columns(2);
    for &pane in &panes {
        layout.apply(
            SessionAction::CycleWidth(pane),
            AREA,
            &LayoutOptions::default(),
        );
    }
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 26);
    assert_eq!(view.shown(scene(&layout)), [panes[0], panes[1]]);
}

#[test]
fn pane_below_the_bottom_edge_is_not_shown() {
    let (mut layout, panes) = row_of_columns(2);
    stack_into_left(&mut layout, panes[1]);
    let scene = Scene {
        layout: &layout,
        area: Size::new(120, 60),
        viewport: Size::new(100, 30),
    };
    let view = View::new(scene);
    assert_eq!(view.shown(scene), [panes[0]]);
}

#[test]
fn panes_of_other_bands_are_not_shown() {
    let mut layout = Layout::new();
    open(&mut layout, 0, None);
    let other = open(&mut layout, 1, None);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::BandDown]);
    assert_eq!(view.shown(scene(&layout)), [other]);
    act(&mut view, &layout, &[ViewAction::BandDown]);
    assert_eq!(view.shown(scene(&layout)), []);
}

#[test]
fn focusing_a_named_pane_in_the_viewed_band() {
    let (layout, panes) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusPane(panes[2])]);
    assert_eq!(view.focused(), Some(panes[2]));
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.focused(), Some(panes[1]));
}

#[test]
fn focusing_a_named_pane_in_another_band() {
    let (mut layout, _) = row_of_columns(1);
    let mut view = View::new(scene(&layout));
    let pane = open(&mut layout, 1, None);
    act(&mut view, &layout, &[ViewAction::FocusPane(pane)]);
    assert_eq!(view.band(), layout.bands()[1].id);
    assert_eq!(view.focused(), Some(pane));
}

#[test]
fn viewing_a_named_band_two_bands_down() {
    let (mut layout, _) = row_of_columns(1);
    open(&mut layout, 1, None);
    let mut view = View::new(scene(&layout));
    let empty = layout.bands()[2].id;
    act(&mut view, &layout, &[ViewAction::ViewBand(empty)]);
    assert_eq!(view.band(), empty);
    assert_eq!(view.focused(), None);
}

#[test]
fn viewing_a_named_band_returns_to_the_remembered_focus() {
    let (mut layout, _) = row_of_columns(1);
    let first = open(&mut layout, 1, None);
    let second = open(&mut layout, 1, Some(first));
    let mut view = View::new(scene(&layout));
    let (b1, b2) = (layout.bands()[0].id, layout.bands()[1].id);
    act(
        &mut view,
        &layout,
        &[
            ViewAction::BandDown,
            ViewAction::FocusRight,
            ViewAction::ViewBand(b1),
            ViewAction::ViewBand(b2),
        ],
    );
    assert_eq!(view.band(), b2);
    assert_eq!(view.focused(), Some(second));
}

#[test]
fn viewing_the_viewed_or_an_unknown_band_changes_nothing() {
    let (layout, panes) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    let band = view.band();
    act(
        &mut view,
        &layout,
        &[
            ViewAction::ViewBand(band),
            ViewAction::ViewBand(gband_core::layout::BandId(99)),
        ],
    );
    assert_eq!(view.band(), band);
    assert_eq!(view.focused(), Some(panes[1]));
}
