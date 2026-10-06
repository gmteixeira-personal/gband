use gband_core::action::SessionCommand;
use gband_core::geometry::{Size, boxes, drawn_copy, tiles};
use gband_core::layout::{
    Direction, Layout, LayoutOptions, Proportion, SessionAction, Step, WindowHeight, WindowId,
};
use gband_core::view::{CenterFocusedColumn, Layer, Scene, View, ViewAction};

const AREA: Size = Size::new(80, 24);

fn scene(layout: &Layout) -> Scene<'_> {
    Scene {
        layout,
        area: AREA,
        viewport: Size::new(80, 24),
    }
}

fn open(layout: &mut Layout, band: usize, after: Option<WindowId>) -> WindowId {
    let window = layout.allocate_window();
    let id = layout.bands()[band].id;
    assert!(
        !layout
            .open(window, id, after, None, &LayoutOptions::default())
            .is_empty()
    );
    window
}

fn row_of_columns(count: usize) -> (Layout, Vec<WindowId>) {
    let mut layout = Layout::new();
    let mut windows = vec![open(&mut layout, 0, None)];
    for _ in 1..count {
        let last = *windows.last().unwrap();
        windows.push(open(&mut layout, 0, Some(last)));
    }
    (layout, windows)
}

fn stack_into_left(layout: &mut Layout, window: WindowId) {
    layout.apply(
        SessionAction::ConsumeOrExpel {
            window,
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
fn initial_view_focuses_the_first_window() {
    let (layout, windows) = row_of_columns(2);
    let view = View::new(scene(&layout));
    assert_eq!(view.band(), layout.bands()[0].id);
    assert_eq!(view.focused(), Some(windows[0]));
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
    let (layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    view.set_loop_bands(false);
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.focused(), Some(windows[0]));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    assert_eq!(view.focused(), Some(windows[2]));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(windows[2]));
}

#[test]
fn focus_returns_to_the_remembered_window() {
    let (mut layout, windows) = row_of_columns(3);
    stack_into_left(&mut layout, windows[1]);
    let (p2, b) = (windows[1], windows[2]);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusDown]);
    assert_eq!(view.focused(), Some(p2));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(b));
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.focused(), Some(p2));
}

#[test]
fn unvisited_column_focuses_its_top_window() {
    let (mut layout, windows) = row_of_columns(3);
    stack_into_left(&mut layout, windows[2]);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(windows[1]));
}

#[test]
fn focus_moves_within_a_stack() {
    let (mut layout, windows) = row_of_columns(3);
    stack_into_left(&mut layout, windows[1]);
    stack_into_left(&mut layout, windows[2]);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusUp]);
    assert_eq!(view.focused(), Some(windows[0]));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusDown, ViewAction::FocusDown],
    );
    assert_eq!(view.focused(), Some(windows[2]));
    act(&mut view, &layout, &[ViewAction::FocusDown]);
    assert_eq!(view.focused(), Some(windows[2]));
}

#[test]
fn switching_down_to_the_empty_band_and_back() {
    let (layout, windows) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    act(&mut view, &layout, &[ViewAction::BandDown]);
    assert_eq!(view.band(), layout.bands()[1].id);
    assert_eq!(view.focused(), None);
    act(&mut view, &layout, &[ViewAction::BandDown]);
    assert_eq!(view.band(), layout.bands()[1].id);
    act(&mut view, &layout, &[ViewAction::BandUp]);
    assert_eq!(view.band(), layout.bands()[0].id);
    assert_eq!(view.focused(), Some(windows[1]));
    act(&mut view, &layout, &[ViewAction::BandUp]);
    assert_eq!(view.band(), layout.bands()[0].id);
    assert_eq!(view.focused(), Some(windows[1]));
}

#[test]
fn views_of_one_layout_are_independent() {
    let (layout, windows) = row_of_columns(2);
    let mut first = View::new(scene(&layout));
    let mut second = View::new(scene(&layout));
    act(&mut first, &layout, &[ViewAction::FocusRight]);
    act(&mut second, &layout, &[ViewAction::FocusRight]);
    act(&mut first, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(first.focused(), Some(windows[0]));
    assert_eq!(second.focused(), Some(windows[1]));
}

#[test]
fn closing_the_focused_column_focuses_the_one_to_its_right() {
    let (mut layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    layout.remove(windows[1]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(windows[2]));
}

#[test]
fn closing_the_last_column_focuses_the_one_to_its_left() {
    let (mut layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    layout.remove(windows[2]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(windows[1]));
}

#[test]
fn closing_a_focused_stacked_window_focuses_the_one_below() {
    let (mut layout, windows) = row_of_columns(3);
    stack_into_left(&mut layout, windows[1]);
    stack_into_left(&mut layout, windows[2]);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusDown]);
    layout.remove(windows[1]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(windows[2]));
    layout.remove(windows[2]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(windows[0]));
}

#[test]
fn focused_window_expelled_keeps_focus() {
    let (mut layout, windows) = row_of_columns(2);
    stack_into_left(&mut layout, windows[1]);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusDown]);
    layout.apply(
        SessionAction::ConsumeOrExpel {
            window: windows[1],
            direction: Direction::Right,
        },
        AREA,
        &LayoutOptions::default(),
    );
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(windows[1]));
    assert_eq!(layout.locate(windows[1]).unwrap().column, 1);
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
fn window_opened_in_the_viewed_empty_band_gains_focus() {
    let mut layout = Layout::new();
    let mut view = View::new(scene(&layout));
    let window = open(&mut layout, 0, None);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(window));
}

#[test]
fn focus_window_switches_to_its_band() {
    let (mut layout, _) = row_of_columns(1);
    let mut view = View::new(scene(&layout));
    let window = open(&mut layout, 1, None);
    view.focus_window(window, scene(&layout));
    assert_eq!(view.band(), layout.bands()[1].id);
    assert_eq!(view.focused(), Some(window));
}

#[test]
fn focus_window_ignores_a_window_the_layout_does_not_hold() {
    let (layout, windows) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    view.focus_window(WindowId(99), scene(&layout));
    assert_eq!(view.band(), layout.bands()[0].id);
    assert_eq!(view.focused(), Some(windows[1]));
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
        let (mut layout, windows) = row_of_columns(3);
        let mut view = View::new(scene(&layout));
        act(&mut view, &layout, &[ViewAction::FocusRight]);
        layout.remove(windows[1]);
        act(&mut view, &layout, &[action]);
        assert_eq!(view.focused(), Some(windows[2]), "{action:?}");
    }
}

#[test]
fn camera_scrolls_just_enough_and_back() {
    let (layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 0);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(windows[2]));
    assert_eq!(view.camera(), 40);
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.camera(), 40);
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.camera(), 0);
}

#[test]
fn camera_aligns_a_wide_column_to_its_start() {
    let (mut layout, windows) = row_of_columns(2);
    layout.apply(
        SessionAction::ToggleFullWidth(windows[1]),
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

fn columns_of(width: Proportion, count: usize) -> (Layout, Vec<WindowId>) {
    let options = LayoutOptions {
        default_width: width,
        ..LayoutOptions::default()
    };
    let mut layout = Layout::new();
    let mut windows = Vec::new();
    for _ in 0..count {
        let window = layout.allocate_window();
        let id = layout.bands()[0].id;
        layout.open(window, id, windows.last().copied(), None, &options);
        windows.push(window);
    }
    (layout, windows)
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
    let (mut layout, windows) = row_of_columns(2);
    let mut view = view_with(&layout, CenterFocusedColumn::OnOverflow);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 0);
    layout.apply(
        SessionAction::StepWidth {
            window: windows[1],
            step: Step::Grow,
            by: Proportion::TENTH,
        },
        AREA,
        &LayoutOptions::default(),
    );
    view.sync(scene(&layout));
    assert_eq!(view.camera(), 8);
}

#[test]
fn on_overflow_centres_a_new_column_that_does_not_fit_beside_the_focus() {
    let (mut layout, windows) = columns_of(Proportion::TWO_THIRDS, 1);
    let mut view = view_with(&layout, CenterFocusedColumn::OnOverflow);
    let opened = layout.allocate_window();
    let options = LayoutOptions {
        default_width: Proportion::TWO_THIRDS,
        ..LayoutOptions::default()
    };
    layout.open(
        opened,
        layout.bands()[0].id,
        Some(windows[0]),
        None,
        &options,
    );
    view.sync(scene(&layout));
    view.focus_window(opened, scene(&layout));
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
fn session_commands_resolve_to_the_focused_window() {
    let (layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    assert_eq!(view.focused(), Some(windows[2]));
    let band = layout.bands()[0].id;
    let cases = [
        (
            SessionCommand::CycleWidth,
            SessionAction::CycleWidth(windows[2]),
        ),
        (
            SessionCommand::ToggleFullWidth,
            SessionAction::ToggleFullWidth(windows[2]),
        ),
        (
            SessionCommand::CloseWindow,
            SessionAction::CloseWindow(windows[2]),
        ),
        (
            SessionCommand::ConsumeOrExpel(Direction::Left),
            SessionAction::ConsumeOrExpel {
                window: windows[2],
                direction: Direction::Left,
            },
        ),
        (
            SessionCommand::OpenWindow,
            SessionAction::open(band, Some(windows[2]), None),
        ),
        (
            SessionCommand::StepWidth {
                step: Step::Grow,
                by: Proportion::TENTH,
            },
            SessionAction::StepWidth {
                window: windows[2],
                step: Step::Grow,
                by: Proportion::TENTH,
            },
        ),
        (
            SessionCommand::StepWidth {
                step: Step::Shrink,
                by: Proportion::TENTH,
            },
            SessionAction::StepWidth {
                window: windows[2],
                step: Step::Shrink,
                by: Proportion::TENTH,
            },
        ),
        (
            SessionCommand::StepHeight {
                step: Step::Grow,
                by: Proportion::TENTH,
            },
            SessionAction::StepHeight {
                window: windows[2],
                step: Step::Grow,
                by: Proportion::TENTH,
            },
        ),
        (
            SessionCommand::StepHeight {
                step: Step::Shrink,
                by: Proportion::TENTH,
            },
            SessionAction::StepHeight {
                window: windows[2],
                step: Step::Shrink,
                by: Proportion::TENTH,
            },
        ),
        (
            SessionCommand::ResetHeight,
            SessionAction::ResetHeight(windows[2]),
        ),
    ];
    for (command, expected) in cases {
        assert_eq!(view.resolve(command), Some(expected), "{command:?}");
    }
}

#[test]
fn commands_on_a_window_resolve_to_nothing_without_focus() {
    let layout = Layout::new();
    let view = View::new(scene(&layout));
    assert_eq!(view.focused(), None);
    for command in [
        SessionCommand::CloseWindow,
        SessionCommand::ConsumeOrExpel(Direction::Left),
        SessionCommand::ConsumeOrExpel(Direction::Right),
        SessionCommand::CycleWidth,
        SessionCommand::ToggleFullWidth,
        SessionCommand::StepWidth {
            step: Step::Grow,
            by: Proportion::TENTH,
        },
        SessionCommand::StepHeight {
            step: Step::Shrink,
            by: Proportion::TENTH,
        },
        SessionCommand::ResetHeight,
    ] {
        assert_eq!(view.resolve(command), None, "{command:?}");
    }
}

#[test]
fn open_window_on_the_empty_band_names_it_and_no_window() {
    let (layout, _) = row_of_columns(1);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::BandDown]);
    let empty = layout.bands()[1].id;
    assert_eq!(view.band(), empty);
    assert_eq!(
        view.resolve(SessionCommand::OpenWindow),
        Some(SessionAction::open(empty, None, None))
    );
}

#[test]
fn column_beyond_the_right_edge_is_not_shown() {
    let (layout, windows) = row_of_columns(3);
    let view = View::new(scene(&layout));
    assert_eq!(view.shown(scene(&layout)), [windows[0], windows[1]]);
}

#[test]
fn partly_visible_column_is_shown() {
    let (mut layout, windows) = row_of_columns(2);
    for &window in &windows {
        layout.apply(
            SessionAction::CycleWidth(window),
            AREA,
            &LayoutOptions::default(),
        );
    }
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 26);
    assert_eq!(view.shown(scene(&layout)), [windows[0], windows[1]]);
}

#[test]
fn window_below_the_bottom_edge_is_not_shown() {
    let (mut layout, windows) = row_of_columns(2);
    stack_into_left(&mut layout, windows[1]);
    let scene = Scene {
        layout: &layout,
        area: Size::new(120, 60),
        viewport: Size::new(100, 30),
    };
    let view = View::new(scene);
    assert_eq!(view.shown(scene), [windows[0]]);
}

#[test]
fn windows_of_other_bands_are_not_shown() {
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
fn focusing_a_named_window_in_the_viewed_band() {
    let (layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusWindow(windows[2])]);
    assert_eq!(view.focused(), Some(windows[2]));
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.focused(), Some(windows[1]));
}

#[test]
fn focusing_a_named_window_in_another_band() {
    let (mut layout, _) = row_of_columns(1);
    let mut view = View::new(scene(&layout));
    let window = open(&mut layout, 1, None);
    act(&mut view, &layout, &[ViewAction::FocusWindow(window)]);
    assert_eq!(view.band(), layout.bands()[1].id);
    assert_eq!(view.focused(), Some(window));
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
    let (layout, windows) = row_of_columns(2);
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
    assert_eq!(view.focused(), Some(windows[1]));
}

fn apply(layout: &mut Layout, action: SessionAction) {
    layout.apply(action, AREA, &LayoutOptions::default());
}

fn float(layout: &mut Layout, window: WindowId) {
    apply(
        layout,
        SessionAction::ToggleFloating {
            window,
            after: None,
        },
    );
    assert!(layout.floating(window).is_some());
}

fn place(layout: &mut Layout, window: WindowId, col: u16, row: u16, width: Proportion) {
    let height = WindowHeight::Fixed(4);
    apply(layout, SessionAction::SetWidth { window, width });
    apply(layout, SessionAction::SetHeight { window, height });
    apply(layout, SessionAction::SetPosition { window, col, row });
}

fn with_floating(columns: usize, floating: usize) -> (Layout, Vec<WindowId>, Vec<WindowId>) {
    let (mut layout, mut windows) = row_of_columns(columns + floating);
    let floated = windows.split_off(columns);
    for &window in &floated {
        float(&mut layout, window);
    }
    (layout, windows, floated)
}

#[test]
fn view_enters_a_band_in_the_tiled_layer() {
    let (layout, windows, _) = with_floating(1, 1);
    let view = View::new(scene(&layout));
    assert_eq!(view.focused(), Some(windows[0]));
    assert_eq!(view.layer(), Layer::Tiled);
}

#[test]
fn remembered_layer() {
    let (mut layout, _, floated) = with_floating(1, 1);
    open(&mut layout, 1, None);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[
            ViewAction::SwitchLayer,
            ViewAction::BandDown,
            ViewAction::BandUp,
        ],
    );
    assert_eq!(view.layer(), Layer::Floating);
    assert_eq!(view.focused(), Some(floated[0]));
}

#[test]
fn last_tiled_window_closes() {
    let (mut layout, windows, floated) = with_floating(1, 1);
    let mut view = View::new(scene(&layout));
    layout.remove(windows[0]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(floated[0]));
    assert_eq!(view.layer(), Layer::Floating);
}

#[test]
fn focus_by_number_activates_the_layer() {
    let (layout, windows, floated) = with_floating(1, 1);
    let mut view = View::new(scene(&layout));
    assert_eq!(view.focused(), Some(windows[0]));
    act(&mut view, &layout, &[ViewAction::FocusWindow(floated[0])]);
    assert_eq!(view.layer(), Layer::Floating);
    assert_eq!(view.focused(), Some(floated[0]));
    act(&mut view, &layout, &[ViewAction::FocusWindow(windows[0])]);
    assert_eq!(view.layer(), Layer::Tiled);
}

#[test]
fn band_with_only_floating_windows() {
    let (layout, _, floated) = with_floating(0, 1);
    let view = View::new(scene(&layout));
    assert_eq!(view.layer(), Layer::Floating);
    assert_eq!(view.focused(), Some(floated[0]));
}

#[test]
fn switch_into_the_floating_layer_and_back() {
    let (layout, windows, floated) = with_floating(1, 2);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[
            ViewAction::FocusWindow(floated[1]),
            ViewAction::FocusWindow(floated[0]),
            ViewAction::FocusWindow(floated[1]),
            ViewAction::FocusWindow(windows[0]),
            ViewAction::SwitchLayer,
        ],
    );
    assert_eq!(view.focused(), Some(floated[1]));
    act(&mut view, &layout, &[ViewAction::SwitchLayer]);
    assert_eq!(view.focused(), Some(windows[0]));
    assert_eq!(view.layer(), Layer::Tiled);
}

#[test]
fn switch_layers_without_a_floating_window() {
    let (layout, windows) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::SwitchLayer]);
    assert_eq!(view.focused(), Some(windows[0]));
    assert_eq!(view.layer(), Layer::Tiled);
}

#[test]
fn switch_into_a_never_focused_layer_takes_the_top_window() {
    let (layout, _, floated) = with_floating(1, 2);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::SwitchLayer]);
    assert_eq!(view.focused(), Some(floated[1]));
}

#[test]
fn switch_to_a_tiled_layer_whose_window_left_takes_the_first_column() {
    let (mut layout, windows, floated) = with_floating(2, 1);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusWindow(floated[0])],
    );
    layout.remove(windows[1]);
    view.sync(scene(&layout));
    act(&mut view, &layout, &[ViewAction::SwitchLayer]);
    assert_eq!(view.focused(), Some(windows[0]));
}

#[test]
fn two_clients_in_different_layers() {
    let (layout, windows, floated) = with_floating(1, 1);
    let mut first = View::new(scene(&layout));
    let second = View::new(scene(&layout));
    act(&mut first, &layout, &[ViewAction::SwitchLayer]);
    assert_eq!(first.focused(), Some(floated[0]));
    assert_eq!(second.focused(), Some(windows[0]));
}

fn boxes_in_a_row() -> (Layout, Vec<WindowId>) {
    let (mut layout, _, floated) = with_floating(1, 3);
    for (&window, col) in floated.iter().zip([0, 30, 60]) {
        place(&mut layout, window, col, 2, Proportion::new(1, 4));
    }
    (layout, floated)
}

#[test]
fn focus_right_between_floating_windows() {
    let (layout, floated) = boxes_in_a_row();
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusWindow(floated[0]), ViewAction::FocusRight],
    );
    assert_eq!(view.focused(), Some(floated[1]));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(floated[2]));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusLeft, ViewAction::FocusLeft],
    );
    assert_eq!(view.focused(), Some(floated[0]));
}

#[test]
fn no_floating_window_in_that_direction() {
    let (layout, floated) = boxes_in_a_row();
    let mut view = View::new(scene(&layout));
    for action in [
        ViewAction::FocusLeft,
        ViewAction::FocusUp,
        ViewAction::FocusDown,
    ] {
        act(
            &mut view,
            &layout,
            &[ViewAction::FocusWindow(floated[0]), action],
        );
        assert_eq!(view.focused(), Some(floated[0]), "{action:?}");
    }
}

#[test]
fn floating_focus_ties_go_to_the_nearer_row_then_the_list() {
    let (mut layout, _, floated) = with_floating(1, 4);
    let width = Proportion::new(1, 8);
    place(&mut layout, floated[0], 0, 8, width);
    place(&mut layout, floated[1], 40, 0, width);
    place(&mut layout, floated[2], 40, 10, width);
    place(&mut layout, floated[3], 40, 7, width);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusWindow(floated[0]), ViewAction::FocusRight],
    );
    assert_eq!(view.focused(), Some(floated[3]));
    place(&mut layout, floated[3], 40, 6, width);
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusWindow(floated[0]), ViewAction::FocusRight],
    );
    assert_eq!(view.focused(), Some(floated[2]));
}

#[test]
fn floating_layer_stays_in_its_layer() {
    let (layout, _, floated) = with_floating(2, 1);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::SwitchLayer]);
    for action in [ViewAction::FocusLeft, ViewAction::FocusRight] {
        act(&mut view, &layout, &[action]);
        assert_eq!(view.focused(), Some(floated[0]));
        assert_eq!(view.layer(), Layer::Floating);
    }
}

#[test]
fn float_the_focused_window() {
    let (mut layout, windows) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    float(&mut layout, windows[1]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(windows[1]));
    assert_eq!(view.layer(), Layer::Floating);
}

#[test]
fn tile_the_focused_window() {
    let (mut layout, windows, floated) = with_floating(1, 1);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::SwitchLayer]);
    let action = view.resolve(SessionCommand::ToggleFloating).unwrap();
    apply(&mut layout, action);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(floated[0]));
    assert_eq!(view.layer(), Layer::Tiled);
    assert_eq!(layout.locate(floated[0]).unwrap().column, 1);
    assert_eq!(layout.locate(windows[0]).unwrap().column, 0);
}

#[test]
fn last_floating_window_closes() {
    let (mut layout, windows, floated) = with_floating(1, 1);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::SwitchLayer]);
    layout.remove(floated[0]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(windows[0]));
    assert_eq!(view.layer(), Layer::Tiled);
}

#[test]
fn closed_floating_focus_goes_to_the_most_recent_floating_window() {
    let (mut layout, _, floated) = with_floating(1, 3);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[
            ViewAction::FocusWindow(floated[0]),
            ViewAction::FocusWindow(floated[2]),
        ],
    );
    layout.remove(floated[2]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(floated[0]));
}

#[test]
fn closed_floating_focus_without_history_goes_to_the_last_in_the_list() {
    let (mut layout, _, floated) = with_floating(0, 3);
    let mut view = View::new(scene(&layout));
    assert_eq!(view.focused(), Some(floated[2]));
    layout.remove(floated[2]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(floated[1]));
}

#[test]
fn back_up_restores_a_floating_focus() {
    let (mut layout, _, floated) = with_floating(1, 1);
    open(&mut layout, 1, None);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[
            ViewAction::FocusWindow(floated[0]),
            ViewAction::BandDown,
            ViewAction::BandUp,
        ],
    );
    assert_eq!(view.focused(), Some(floated[0]));
    assert_eq!(view.layer(), Layer::Floating);
}

#[test]
fn focused_column_moved() {
    let (mut layout, windows) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    apply(
        &mut layout,
        SessionAction::MoveColumn {
            window: windows[0],
            direction: Direction::Right,
        },
    );
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(windows[0]));
    assert_eq!(layout.locate(windows[0]).unwrap().column, 1);
}

#[test]
fn floating_focus_keeps_the_camera() {
    let (mut layout, windows) = row_of_columns(4);
    float(&mut layout, windows[3]);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    assert_eq!(view.camera(), 40);
    act(&mut view, &layout, &[ViewAction::SwitchLayer]);
    assert_eq!(view.focused(), Some(windows[3]));
    assert_eq!(view.camera(), 40);
}

#[test]
fn open_window_from_the_floating_layer() {
    let (layout, windows, floated) = with_floating(1, 1);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusWindow(floated[0])]);
    assert_eq!(
        view.resolve(SessionCommand::OpenWindow),
        Some(SessionAction::open(
            layout.bands()[0].id,
            Some(windows[0]),
            None
        ))
    );
}

#[test]
fn tile_the_focused_floating_window_after_the_tiled_focus() {
    let (layout, windows, floated) = with_floating(1, 1);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusWindow(floated[0])]);
    assert_eq!(
        view.resolve(SessionCommand::ToggleFloating),
        Some(SessionAction::ToggleFloating {
            window: floated[0],
            after: Some(windows[0]),
        })
    );
}

#[test]
fn toggle_floating_resolves_to_nothing_without_focus() {
    let layout = Layout::new();
    let view = View::new(scene(&layout));
    assert_eq!(view.resolve(SessionCommand::ToggleFloating), None);
}

#[test]
fn floating_window_shown() {
    let (mut layout, windows) = row_of_columns(4);
    float(&mut layout, windows[3]);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    assert_eq!(view.camera(), 40);
    assert_eq!(
        view.shown(scene(&layout)),
        [windows[1], windows[2], windows[3]]
    );
}

#[test]
fn box_below_a_small_terminal() {
    let (mut layout, _, floated) = with_floating(1, 1);
    let area = Size::new(120, 60);
    layout.apply(
        SessionAction::SetPosition {
            window: floated[0],
            col: 0,
            row: 40,
        },
        area,
        &LayoutOptions::default(),
    );
    let small = Scene {
        layout: &layout,
        area,
        viewport: Size::new(100, 30),
    };
    let view = View::new(small);
    assert!(!view.shown(small).contains(&floated[0]));
}

#[test]
fn focus_raises() {
    let (mut layout, _, floated) = with_floating(1, 2);
    place(&mut layout, floated[0], 10, 2, Proportion::ONE_HALF);
    place(&mut layout, floated[1], 20, 4, Proportion::ONE_HALF);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[
            ViewAction::FocusWindow(floated[1]),
            ViewAction::FocusWindow(floated[0]),
        ],
    );
    assert_eq!(view.stacking(scene(&layout)), [floated[1], floated[0]]);
}

#[test]
fn never_focused_follow_the_list() {
    let (layout, _, floated) = with_floating(1, 2);
    let view = View::new(scene(&layout));
    assert_eq!(view.stacking(scene(&layout)), [floated[0], floated[1]]);
}

#[test]
fn other_clients_keep_their_order() {
    let (layout, _, floated) = with_floating(1, 2);
    let mut first = View::new(scene(&layout));
    let mut second = View::new(scene(&layout));
    act(&mut first, &layout, &[ViewAction::FocusWindow(floated[0])]);
    act(&mut second, &layout, &[ViewAction::FocusWindow(floated[1])]);
    assert_eq!(first.stacking(scene(&layout)).last(), Some(&floated[0]));
    assert_eq!(second.stacking(scene(&layout)).last(), Some(&floated[1]));
}

fn centre_third_of_three(policy: CenterFocusedColumn) {
    let (layout, windows) = row_of_columns(3);
    let mut view = view_with(&layout, policy);
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    assert_eq!(view.camera(), 40);
    act(&mut view, &layout, &[ViewAction::CenterColumn]);
    assert_eq!(view.camera(), 60);
    assert_eq!(view.focused(), Some(windows[2]));
}

#[test]
fn center_a_column_at_the_right_edge() {
    centre_third_of_three(CenterFocusedColumn::Never);
}

#[test]
fn center_a_column_at_the_right_edge_on_overflow() {
    centre_third_of_three(CenterFocusedColumn::OnOverflow);
}

#[test]
fn center_the_first_column() {
    let (layout, windows) = row_of_columns(1);
    let mut view = View::new(scene(&layout));
    assert_eq!(view.camera(), 0);
    act(&mut view, &layout, &[ViewAction::CenterColumn]);
    assert_eq!(view.camera(), -20);
    assert_eq!(view.focused(), Some(windows[0]));
}

#[test]
fn center_a_column_wider_than_the_terminal() {
    let (mut layout, windows) = row_of_columns(2);
    apply(
        &mut layout,
        SessionAction::SetWidth {
            window: windows[1],
            width: Proportion::new(5, 4),
        },
    );
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::CenterColumn],
    );
    assert_eq!(view.camera(), 40);
}

#[test]
fn centred_column_keeps_the_camera() {
    let (layout, _) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::CenterColumn],
    );
    assert_eq!(view.camera(), 20);
    view.sync(Scene {
        layout: &layout,
        area: AREA,
        viewport: Size::new(78, 24),
    });
    assert_eq!(view.camera(), 20);
}

#[test]
fn policy_resumes_on_the_next_focus_change() {
    let (layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    view.set_loop_bands(false);
    act(
        &mut view,
        &layout,
        &[
            ViewAction::CenterColumn,
            ViewAction::FocusWindow(windows[2]),
        ],
    );
    assert_eq!(view.camera(), 40);
}

#[test]
fn center_column_on_the_floating_layer_keeps_the_camera() {
    let (layout, windows, floated) = with_floating(3, 1);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    assert_eq!(view.focused(), Some(windows[2]));
    assert_eq!(view.camera(), 40);
    act(
        &mut view,
        &layout,
        &[ViewAction::SwitchLayer, ViewAction::CenterColumn],
    );
    assert_eq!(view.camera(), 40);
    assert_eq!(view.focused(), Some(floated[0]));
}

#[test]
fn centred_box_centres_the_focused_floating_window() {
    let (mut layout, _, floated) = with_floating(1, 1);
    place(&mut layout, floated[0], 0, 0, Proportion::ONE_HALF);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusWindow(floated[0])]);
    assert_eq!(
        view.centred_box(scene(&layout)),
        Some(SessionAction::SetPosition {
            window: floated[0],
            col: 20,
            row: 10,
        })
    );
}

#[test]
fn centred_box_of_a_centred_floating_window_is_nothing() {
    let (layout, _, floated) = with_floating(1, 1);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusWindow(floated[0])]);
    assert_eq!(view.centred_box(scene(&layout)), None);
}

#[test]
fn centred_box_of_a_tiled_focus_is_nothing() {
    let (layout, windows, _) = with_floating(1, 1);
    let view = View::new(scene(&layout));
    assert_eq!(view.focused(), Some(windows[0]));
    assert_eq!(view.centred_box(scene(&layout)), None);
}

#[test]
fn center_column_on_an_empty_band_changes_nothing() {
    let layout = Layout::new();
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::CenterColumn]);
    assert_eq!(view.camera(), 0);
    assert_eq!(view.focused(), None);
}

fn drawn(view: &View, layout: &Layout) -> Vec<(WindowId, i64)> {
    drawn_in(view, scene(layout))
}

fn drawn_in(view: &View, scene: Scene<'_>) -> Vec<(WindowId, i64)> {
    let shown = view.shown(scene);
    let strip = view.strip(scene);
    let mut drawn: Vec<(WindowId, i64)> = tiles(&scene.layout.bands()[0], scene.area)
        .into_iter()
        .filter(|tile| shown.contains(&tile.window))
        .map(|tile| {
            let left = i64::from(tile.x) - view.camera();
            let left = strip.map_or(left, |strip| drawn_copy(left, strip, scene.viewport.cols));
            (tile.window, left)
        })
        .collect();
    drawn.sort_by_key(|&(_, left)| left);
    drawn
}

#[test]
fn right_of_the_last_column_goes_round() {
    let (layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusWindow(windows[2])]);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(windows[0]));
}

#[test]
fn left_of_the_first_column_goes_round_to_the_remembered_window() {
    let (mut layout, windows) = row_of_columns(5);
    stack_into_left(&mut layout, windows[1]);
    stack_into_left(&mut layout, windows[4]);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[
            ViewAction::FocusWindow(windows[4]),
            ViewAction::FocusWindow(windows[0]),
            ViewAction::FocusLeft,
        ],
    );
    assert_eq!(view.focused(), Some(windows[4]));
}

#[test]
fn short_band_still_goes_round() {
    let (layout, windows) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    assert_eq!(view.focused(), Some(windows[0]));
    assert_eq!(view.strip(scene(&layout)), None);
}

#[test]
fn one_column_does_not_go_round() {
    let (mut layout, windows) = row_of_columns(2);
    stack_into_left(&mut layout, windows[1]);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusWindow(windows[0])]);
    for action in [ViewAction::FocusRight, ViewAction::FocusLeft] {
        act(&mut view, &layout, &[action]);
        assert_eq!(view.focused(), Some(windows[0]));
        assert_eq!(view.camera(), 0);
    }
}

fn third_of_three(policy: CenterFocusedColumn) -> (Layout, Vec<WindowId>, View) {
    let (layout, windows) = row_of_columns(3);
    let mut view = view_with(&layout, policy);
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    (layout, windows, view)
}

#[test]
fn scroll_right_across_the_seam() {
    let (layout, windows, mut view) = third_of_three(CenterFocusedColumn::Never);
    assert_eq!(view.camera(), 40);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(windows[0]));
    assert_eq!(view.camera(), 80);
    assert_eq!(drawn(&view, &layout), [(windows[2], 0), (windows[0], 40)]);
}

#[test]
fn keep_going_right_holds_the_camera_and_keeps_the_travel() {
    let (layout, windows, mut view) = third_of_three(CenterFocusedColumn::Never);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!((view.camera(), view.travel()), (80, 80));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(windows[1]));
    assert_eq!((view.camera(), view.travel()), (0, 120));
    assert_eq!(drawn(&view, &layout), [(windows[0], 0), (windows[1], 40)]);
}

#[test]
fn scroll_left_across_the_seam() {
    let (layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.focused(), Some(windows[2]));
    assert_eq!((view.camera(), view.travel()), (80, -40));
    assert_eq!(drawn(&view, &layout), [(windows[2], 0), (windows[0], 40)]);
}

#[test]
fn always_centre_across_the_seam() {
    let (layout, windows, mut view) = third_of_three(CenterFocusedColumn::Always);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 100);
    assert_eq!(
        drawn(&view, &layout),
        [(windows[2], -20), (windows[0], 20), (windows[1], 60)]
    );
}

#[test]
fn on_overflow_centres_across_the_seam() {
    let (layout, windows) = columns_of(Proportion::TWO_THIRDS, 3);
    let mut view = view_with(&layout, CenterFocusedColumn::OnOverflow);
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.focused(), Some(windows[2]));
    assert_eq!((view.camera(), view.travel()), (93, -66));
}

#[test]
fn on_overflow_scrolls_just_enough_across_the_seam() {
    let (layout, windows, mut view) = third_of_three(CenterFocusedColumn::OnOverflow);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.focused(), Some(windows[0]));
    assert_eq!(view.camera(), 80);
}

#[test]
fn strip_one_cell_short_of_the_terminal_loops() {
    let (layout, windows) = row_of_columns(3);
    let narrow = Scene {
        layout: &layout,
        area: Size::new(79, 24),
        viewport: Size::new(79, 24),
    };
    let mut view = View::with_policy(narrow, CenterFocusedColumn::Never);
    for _ in 0..2 {
        view.apply(ViewAction::FocusRight, narrow);
    }
    assert_eq!((view.focused(), view.camera()), (Some(windows[2]), 38));
    view.apply(ViewAction::FocusRight, narrow);
    assert_eq!((view.focused(), view.camera()), (Some(windows[0]), 77));
    let strip = view.strip(narrow).expect("the strip loops");
    let shown = view.shown(narrow);
    let drawn: Vec<(WindowId, i64)> = tiles(&layout.bands()[0], narrow.area)
        .into_iter()
        .filter(|tile| shown.contains(&tile.window))
        .map(|tile| {
            let left = i64::from(tile.x) - view.camera();
            (tile.window, drawn_copy(left, strip, 79))
        })
        .collect();
    assert_eq!(
        drawn,
        [(windows[0], 40), (windows[1], -38), (windows[2], 1)]
    );
}

#[test]
fn short_strip_does_not_loop() {
    let (layout, windows) = row_of_columns(2);
    let mut view = view_with(&layout, CenterFocusedColumn::Always);
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    assert_eq!(view.focused(), Some(windows[0]));
    assert_eq!(view.camera(), -20);
    assert_eq!(drawn(&view, &layout), [(windows[0], 20), (windows[1], 60)]);
}

#[test]
fn looping_off_draws_the_strip_once() {
    let (layout, windows) = row_of_columns(3);
    let mut view = view_with(&layout, CenterFocusedColumn::Always);
    view.set_loop_bands(false);
    view.sync(scene(&layout));
    assert_eq!(view.camera(), -20);
    assert_eq!(view.strip(scene(&layout)), None);
    assert_eq!(drawn(&view, &layout), [(windows[0], 20), (windows[1], 60)]);
}

#[test]
fn looping_off_stops_focus_at_the_first_column() {
    let (layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    view.set_loop_bands(false);
    act(&mut view, &layout, &[ViewAction::FocusLeft]);
    assert_eq!(view.focused(), Some(windows[0]));
    assert_eq!(view.camera(), 0);
}

#[test]
fn center_the_first_column_of_a_looping_strip() {
    let (layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::CenterColumn]);
    assert_eq!((view.camera(), view.travel()), (100, -20));
    assert_eq!(
        drawn(&view, &layout),
        [(windows[2], -20), (windows[0], 20), (windows[1], 60)]
    );
}

#[test]
fn focus_jump_takes_the_copy_needing_the_smallest_move() {
    let (layout, windows) = row_of_columns(4);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusWindow(windows[3])]);
    assert_eq!((view.camera(), view.travel()), (120, -40));
    assert_eq!(drawn(&view, &layout), [(windows[3], 0), (windows[0], 40)]);
}

#[test]
fn focus_jump_ties_take_the_copy_with_the_smaller_start() {
    let (layout, windows, mut view) = third_of_three(CenterFocusedColumn::Never);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 80);
    act(&mut view, &layout, &[ViewAction::FocusWindow(windows[1])]);
    assert_eq!((view.camera(), view.travel()), (40, 40));
}

#[test]
fn first_column_shown_after_the_last() {
    let (layout, windows, mut view) = third_of_three(CenterFocusedColumn::Never);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.shown(scene(&layout)), [windows[0], windows[2]]);
    assert_eq!(drawn(&view, &layout), [(windows[2], 0), (windows[0], 40)]);
}

#[test]
fn floating_window_stays_in_place_across_the_seam() {
    let (mut layout, windows) = row_of_columns(4);
    float(&mut layout, windows[3]);
    place(&mut layout, windows[3], 10, 2, Proportion::ONE_THIRD);
    let mut view = View::new(scene(&layout));
    act(
        &mut view,
        &layout,
        &[ViewAction::FocusRight, ViewAction::FocusRight],
    );
    assert_eq!(view.camera(), 40);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 80);
    assert!(view.shown(scene(&layout)).contains(&windows[3]));
    let placed = boxes(&layout.bands()[0], AREA);
    assert_eq!(placed[0].x, 10);
}

#[test]
fn band_stops_looping_and_resets_the_travel() {
    let (mut layout, windows, mut view) = third_of_three(CenterFocusedColumn::Never);
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!((view.camera(), view.travel()), (80, 80));
    layout.remove(windows[2]);
    view.sync(scene(&layout));
    assert_eq!(view.strip(scene(&layout)), None);
    assert_eq!(view.camera(), view.travel());
}

#[test]
fn opening_across_the_seam() {
    let (mut layout, windows) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    act(&mut view, &layout, &[ViewAction::FocusRight]);
    assert_eq!(view.camera(), 0);
    let opened = open(&mut layout, 0, Some(windows[1]));
    view.sync(scene(&layout));
    view.focus_window(opened, scene(&layout));
    assert_eq!(view.camera(), 40);
    assert_eq!(drawn(&view, &layout), [(windows[1], 0), (opened, 40)]);
}

#[test]
fn opening_left_across_the_seam() {
    let (mut layout, windows) = row_of_columns(2);
    let mut view = View::new(scene(&layout));
    let opened = open(&mut layout, 0, None);
    view.sync(scene(&layout));
    view.focus_window(opened, scene(&layout));
    assert_eq!(drawn(&view, &layout), [(opened, 0), (windows[0], 40)]);
}

#[test]
fn closing_the_last_column_pulls_the_camera_back() {
    let (mut layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    view.set_loop_bands(false);
    act(&mut view, &layout, &[ViewAction::FocusWindow(windows[2])]);
    assert_eq!(view.camera(), 40);
    layout.remove(windows[2]);
    view.sync(scene(&layout));
    assert_eq!(view.focused(), Some(windows[1]));
    assert_eq!(view.camera(), 0);
    assert_eq!(drawn(&view, &layout), [(windows[0], 0), (windows[1], 40)]);
}

#[test]
fn shrinking_the_last_column_pulls_the_camera_back() {
    let (mut layout, windows) = row_of_columns(3);
    let mut view = View::new(scene(&layout));
    view.set_loop_bands(false);
    act(&mut view, &layout, &[ViewAction::FocusWindow(windows[2])]);
    assert_eq!(view.camera(), 40);
    apply(
        &mut layout,
        SessionAction::SetWidth {
            window: windows[2],
            width: Proportion::new(1, 4),
        },
    );
    view.sync(scene(&layout));
    assert_eq!(view.camera(), 20);
    assert_eq!(drawn(&view, &layout).last(), Some(&(windows[2], 60)));
}

#[test]
fn narrower_area_pulls_the_camera_back() {
    let (layout, windows) = row_of_columns(2);
    let wide = Scene {
        layout: &layout,
        area: AREA,
        viewport: Size::new(60, 24),
    };
    let mut view = View::new(wide);
    view.apply(ViewAction::FocusRight, wide);
    assert_eq!(view.camera(), 20);
    let narrow = Scene {
        area: Size::new(60, 24),
        ..wide
    };
    view.sync(narrow);
    assert_eq!(view.camera(), 0);
    assert_eq!(drawn_in(&view, narrow), [(windows[0], 0), (windows[1], 30)]);
}

#[test]
fn pulled_back_no_further_than_0() {
    let (layout, windows) = row_of_columns(2);
    let wide = Scene {
        layout: &layout,
        area: Size::new(70, 24),
        viewport: Size::new(80, 24),
    };
    let mut view = View::new(wide);
    view.apply(ViewAction::FocusRight, wide);
    view.slide(10, wide);
    view.unpin();
    let narrow = Scene {
        area: Size::new(60, 24),
        ..wide
    };
    view.sync(narrow);
    assert_eq!(view.camera(), 0);
    assert_eq!(drawn_in(&view, narrow), [(windows[0], 0), (windows[1], 30)]);
}

#[test]
fn centred_camera_keeps_its_blank_strip() {
    let (layout, windows) = row_of_columns(3);
    let mut view = view_with(&layout, CenterFocusedColumn::Always);
    view.set_loop_bands(false);
    act(&mut view, &layout, &[ViewAction::FocusWindow(windows[2])]);
    let narrow = Scene {
        layout: &layout,
        area: Size::new(60, 24),
        viewport: Size::new(80, 24),
    };
    view.sync(narrow);
    assert_eq!(view.camera(), 35);
    assert_eq!(drawn_in(&view, narrow).last(), Some(&(windows[2], 25)));
}
