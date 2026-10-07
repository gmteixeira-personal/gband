mod common;

use std::sync::Arc;

use common::*;
use gband_core::action::{Action, ClientAction, Edges, SessionCommand};
use gband_core::geometry::Size;
use gband_core::input::{Key, KeyCode, Modifiers};
use gband_core::layout::{
    BandId, Direction, Layout, LayoutOptions, Program, Proportion, SessionAction, Step, Vertical,
    Weight, WindowContent, WindowHeight, WindowId,
};
use gband_core::view::ViewAction;
use gband_lua::{
    BandState, Config, Dispatch, Outcome, PluginWindowRequest, ViewState, WindowInput, WindowName,
    WindowNames,
};
use mlua::Table;

const AREA: Size = Size::new(80, 24);

fn opened(layout: &mut Layout, band: usize, after: Option<WindowId>) -> WindowId {
    let window = layout.allocate_window();
    let id = layout.bands()[band].id;
    layout.open(window, id, after, None, &LayoutOptions::default());
    window
}

fn two_bands() -> Layout {
    let mut layout = Layout::new();
    let first = opened(&mut layout, 0, None);
    opened(&mut layout, 0, Some(first));
    opened(&mut layout, 1, None);
    layout
}

fn state(layout: Layout, band: u32, window: Option<u32>, table: &str) -> ViewState {
    let count = layout.bands().len() as u32;
    ViewState {
        table: table.to_owned(),
        band: BandState {
            number: band,
            index: 1,
            count,
        },
        window,
        width: 80,
        layout: Arc::new(layout),
        area: AREA,
        ribbon: Size::new(80, 23),
        ..ViewState::default()
    }
}

fn loaded_with(name: &str, source: &str, view: ViewState) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    scratch.write(source);
    let config = scratch.loaded();
    clean(&config.runtime.set_state(view));
    (scratch, config)
}

fn job(name: &str, view: ViewState, code: &str) -> Outcome {
    let (_scratch, config) = loaded_with(name, JOB, view);
    run_job(&config, code)
}

fn dispatched(name: &str, view: ViewState, code: &str) -> Vec<Dispatch> {
    let outcome = job(name, view, code);
    clean(&outcome);
    outcome.dispatched
}

fn failed(name: &str, view: ViewState, code: &str, mentions: &str) {
    let outcome = job(name, view, code);
    assert!(outcome.dispatched.is_empty(), "{:?}", outcome.dispatched);
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert!(error.message.contains(mentions), "{error}");
}

fn session(action: SessionAction) -> Dispatch {
    Dispatch::Session(action)
}

#[test]
fn read_the_layout() {
    let mut layout = Layout::new();
    let options = LayoutOptions::default();
    let band = layout.bands()[0].id;
    let (p1, p2, p3) = (
        layout.allocate_window(),
        layout.allocate_window(),
        layout.allocate_window(),
    );
    layout.open(p1, band, None, None, &options);
    layout.open(p2, band, Some(p1), Some(Proportion::ONE_THIRD), &options);
    layout.open(p3, band, Some(p2), None, &options);
    for action in [
        SessionAction::ConsumeOrExpel {
            window: p3,
            direction: Direction::Left,
        },
        SessionAction::ToggleFullWidth(p2),
        SessionAction::SetHeight {
            window: p2,
            height: WindowHeight::Fixed(10),
        },
    ] {
        layout.apply(action, AREA, &options);
    }
    let (_scratch, config) = loaded_with("layout", "", state(layout, 1, Some(1), "root"));
    let described: Table = eval(&config, "return gband.layout()");
    assert_eq!(described.get::<u16>("cols").unwrap(), 80);
    assert_eq!(described.get::<u16>("rows").unwrap(), 24);
    let summary: String = eval(
        &config,
        r#"
        local layout = gband.layout()
        local parts = {}
        for _, band in ipairs(layout.bands) do
          parts[#parts + 1] = "band " .. band.id
          for _, column in ipairs(band.columns) do
            parts[#parts + 1] = string.format("%.4f %s", column.width, tostring(column.full_width))
            for _, window in ipairs(column.windows) do
              parts[#parts + 1] = string.format("%d %s %s %s", window.id, tostring(window.rows), tostring(window.weight), tostring(window.plugin_window))
            end
          end
        end
        return table.concat(parts, "; ")
        "#,
    );
    assert_eq!(
        summary,
        "band 1; 0.5000 false; 1 nil 1.0 nil; 0.3333 true; 2 10 nil nil; 3 nil 1.0 nil; band 2"
    );
}

#[test]
fn layout_is_a_copy() {
    let (_scratch, config) = loaded_with("layout-copy", "", state(two_bands(), 1, Some(1), "root"));
    let still: i64 = eval(
        &config,
        "local layout = gband.layout()\nlayout.bands[1].columns = {}\nreturn #gband.layout().bands[1].columns",
    );
    assert_eq!(still, 2);
}

#[test]
fn layout_while_loading() {
    let scratch = Scratch::new("layout-loading");
    let path = scratch.write("\n\n\nlocal layout = gband.layout()\n");
    let error = scratch.load().map(|_| ()).unwrap_err();
    assert_error_at(&error, &path, 4, "gband.layout");
}

#[test]
fn read_the_view() {
    let (_scratch, config) = loaded_with("view", "", state(two_bands(), 1, Some(2), "prefix"));
    let view: Table = eval(&config, "return gband.view()");
    assert_eq!(view.get::<u32>("band").unwrap(), 1);
    assert_eq!(view.get::<Option<u32>>("window").unwrap(), Some(2));
    assert_eq!(view.get::<Option<u32>>("plugin_window").unwrap(), None);
    assert_eq!(view.get::<String>("table").unwrap(), "prefix");
    assert_eq!(view.get::<u16>("cols").unwrap(), 80);
    assert_eq!(view.get::<u16>("rows").unwrap(), 23);
    assert!(!view.get::<bool>("floating").unwrap());
    assert_eq!(view.pairs::<String, mlua::Value>().count(), 6);
}

fn with_floating() -> Layout {
    let mut layout = two_bands();
    let options = LayoutOptions::default();
    let floating = layout.allocate_window();
    layout.open_floating(floating, BandId(1), None, AREA, &options);
    let (window, col, row) = (floating, 50, 3);
    let height = WindowHeight::Fixed(10);
    layout.apply(SessionAction::SetHeight { window, height }, AREA, &options);
    layout.apply(
        SessionAction::SetPosition { window, col, row },
        AREA,
        &options,
    );
    layout
}

#[test]
fn read_a_floating_window() {
    let layout = with_floating();
    assert_eq!(layout.floating(WindowId(4)).unwrap().col, 40);
    let (_scratch, config) = loaded_with("layout-floating", "", state(layout, 1, Some(1), "root"));
    let summary: String = eval(
        &config,
        r#"
        local parts = {}
        for _, band in ipairs(gband.layout().bands) do
          for _, f in ipairs(band.floating) do
            parts[#parts + 1] = string.format("%d %.1f %s %d %d %d %s", f.id, f.width, tostring(f.full_width), f.rows, f.col, f.row, tostring(f.plugin_window))
          end
          parts[#parts + 1] = "|"
        end
        return table.concat(parts, " ")
        "#,
    );
    assert_eq!(summary, "4 0.5 false 10 40 3 nil | | |");
}

#[test]
fn floating_focus() {
    let (_scratch, config) = loaded_with(
        "view-floating",
        "",
        state(with_floating(), 1, Some(4), "root"),
    );
    let view: Table = eval(&config, "return gband.view()");
    assert_eq!(view.get::<Option<u32>>("window").unwrap(), Some(4));
    assert!(view.get::<bool>("floating").unwrap());
}

#[test]
fn open_a_floating_window() {
    let floating = |band| {
        session(SessionAction::OpenWindow {
            band: BandId(band),
            after: None,
            width: None,
            floating: true,
            focus: true,
            content: WindowContent::Program(None),
        })
    };
    assert_eq!(
        dispatched(
            "open-floating",
            state(two_bands(), 1, Some(1), "root"),
            "gband.action.open_window({ floating = true })"
        ),
        [floating(1)]
    );
    assert_eq!(
        dispatched(
            "open-floating-band",
            state(two_bands(), 1, Some(1), "root"),
            "gband.action.open_window({ band = 2, floating = true })"
        ),
        [floating(2)]
    );
    assert_eq!(
        dispatched(
            "open-tiled",
            state(two_bands(), 1, Some(1), "root"),
            "gband.action.open_window({ after = 1, floating = false })"
        ),
        [session(SessionAction::open(
            BandId(1),
            Some(WindowId(1)),
            None
        ))]
    );
}

#[test]
fn floating_with_after() {
    failed(
        "floating-after",
        state(two_bands(), 1, Some(1), "root"),
        "gband.action.open_window({ after = 1, floating = true })",
        "after",
    );
    failed(
        "floating-not-boolean",
        state(two_bands(), 1, Some(1), "root"),
        "gband.action.open_window({ floating = 'yes' })",
        "floating",
    );
}

#[test]
fn open_after_a_floating_window() {
    failed(
        "open-after-floating",
        state(with_floating(), 1, Some(1), "root"),
        "gband.action.open_window({ after = 4 })",
        "4",
    );
}

#[test]
fn tile_a_named_window_after_a_named_window() {
    assert_eq!(
        dispatched(
            "tile-after",
            state(with_floating(), 1, Some(1), "root"),
            "gband.action.toggle_window_floating({ window = 4, after = 1 })"
        ),
        [session(SessionAction::ToggleFloating {
            window: WindowId(4),
            after: Some(WindowId(1)),
            floating: None,
        })]
    );
    assert_eq!(
        dispatched(
            "tile-default",
            state(with_floating(), 1, Some(1), "root"),
            "gband.action.toggle_window_floating({ window = 4 })"
        ),
        [session(SessionAction::ToggleFloating {
            window: WindowId(4),
            after: None,
            floating: None,
        })]
    );
}

#[test]
fn toggle_floating_names_a_layer() {
    assert_eq!(
        dispatched(
            "float-named",
            state(with_floating(), 1, Some(1), "root"),
            "gband.action.toggle_window_floating({ window = 4, floating = true })"
        ),
        [session(SessionAction::ToggleFloating {
            window: WindowId(4),
            after: None,
            floating: Some(true),
        })]
    );
    assert_eq!(
        dispatched(
            "tile-named",
            state(with_floating(), 1, Some(1), "root"),
            "gband.action.toggle_window_floating({ window = 4, after = 1, floating = false })"
        ),
        [session(SessionAction::ToggleFloating {
            window: WindowId(4),
            after: Some(WindowId(1)),
            floating: Some(false),
        })]
    );
}

#[test]
fn float_twice_dispatches_twice() {
    let float = SessionAction::ToggleFloating {
        window: WindowId(2),
        after: None,
        floating: Some(true),
    };
    assert_eq!(
        dispatched(
            "float-twice",
            state(two_bands(), 1, Some(1), "root"),
            "gband.action.toggle_window_floating({ window = 2, floating = true })\ngband.action.toggle_window_floating({ window = 2, floating = true })"
        ),
        [session(float.clone()), session(float)]
    );
}

#[test]
fn tile_after_a_window_of_another_band() {
    failed(
        "tile-other-band",
        state(with_floating(), 1, Some(1), "root"),
        "gband.action.toggle_window_floating({ window = 4, after = 3 })",
        "3",
    );
    failed(
        "tile-without-window",
        state(with_floating(), 1, Some(1), "root"),
        "gband.action.toggle_window_floating({ after = 1 })",
        "window",
    );
    failed(
        "floating-yes",
        state(with_floating(), 1, Some(1), "root"),
        "gband.action.toggle_window_floating({ window = 4, floating = 'yes' })",
        "`floating`",
    );
    failed(
        "float-with-after",
        state(with_floating(), 1, Some(1), "root"),
        "gband.action.toggle_window_floating({ window = 4, after = 1, floating = true })",
        "`after`",
    );
    failed(
        "float-without-window",
        state(with_floating(), 1, Some(1), "root"),
        "gband.action.toggle_window_floating({ floating = true })",
        "`window`",
    );
}

#[test]
fn place_a_floating_window() {
    assert_eq!(
        dispatched(
            "set-position",
            state(with_floating(), 1, Some(1), "root"),
            "gband.window.set_position(4, { col = 10, row = 2 })"
        ),
        [session(SessionAction::SetPosition {
            window: WindowId(4),
            col: 10,
            row: 2,
        })]
    );
}

#[test]
fn set_position_errors() {
    for (name, code, mentions) in [
        (
            "tiled",
            "gband.window.set_position(1, { col = 0, row = 0 })",
            "window 1",
        ),
        (
            "missing",
            "gband.window.set_position(4, { col = 4 })",
            "row",
        ),
        (
            "unknown",
            "gband.window.set_position(4, { col = 4, row = 1, z = 2 })",
            "col",
        ),
        (
            "negative",
            "gband.window.set_position(4, { col = -1, row = 1 })",
            "col",
        ),
        (
            "fraction",
            "gband.window.set_position(4, { col = 1.5, row = 1 })",
            "col",
        ),
        (
            "absent",
            "gband.window.set_position(9, { col = 1, row = 1 })",
            "9",
        ),
        ("not-a-table", "gband.window.set_position(4, 3)", "col"),
    ] {
        failed(
            &format!("set-position-{name}"),
            state(with_floating(), 1, Some(1), "root"),
            code,
            mentions,
        );
    }
}

#[test]
fn view_of_the_empty_band() {
    let (_scratch, config) = loaded_with("view-empty", "", state(two_bands(), 3, None, "root"));
    let window: Option<u32> = eval(&config, "return gband.view().window");
    assert_eq!(window, None);
}

#[test]
fn view_while_loading() {
    let scratch = Scratch::new("view-loading");
    let path = scratch.write("\ngband.view()\n");
    let error = scratch.load().map(|_| ()).unwrap_err();
    assert_error_at(&error, &path, 2, "gband.view");
}

#[test]
fn state_as_of_the_call() {
    let (_scratch, config) = loaded_with("as-of", JOB, state(two_bands(), 1, Some(1), "root"));
    let outcome = run_job(
        &config,
        "gband.action.focus_column_right()\nseen = gband.view().window",
    );
    clean(&outcome);
    assert_eq!(global::<u32>(&config, "seen"), 1);
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusRight))]
    );
}

#[test]
fn close_a_named_window() {
    assert_eq!(
        dispatched(
            "close-named",
            state(two_bands(), 1, Some(2), "root"),
            "gband.action.close_window({ window = 1 })"
        ),
        [session(SessionAction::CloseWindow(WindowId(1)))]
    );
}

#[test]
fn target_overrides_the_view() {
    assert_eq!(
        dispatched(
            "target-overrides",
            state(two_bands(), 1, Some(2), "root"),
            "gband.action.cycle_column_width({ window = 1 })"
        ),
        [session(SessionAction::CycleWidth(WindowId(1)))]
    );
}

#[test]
fn target_on_the_empty_band() {
    assert_eq!(
        dispatched(
            "target-empty",
            state(two_bands(), 3, None, "root"),
            "gband.action.close_window({ window = 1 })"
        ),
        [session(SessionAction::CloseWindow(WindowId(1)))]
    );
}

#[test]
fn every_window_action_takes_a_target() {
    let expected = [
        ("close_window", SessionAction::CloseWindow(WindowId(1))),
        (
            "consume_or_expel_left",
            SessionAction::ConsumeOrExpel {
                window: WindowId(1),
                direction: Direction::Left,
            },
        ),
        (
            "consume_or_expel_right",
            SessionAction::ConsumeOrExpel {
                window: WindowId(1),
                direction: Direction::Right,
            },
        ),
        ("cycle_column_width", SessionAction::CycleWidth(WindowId(1))),
        (
            "toggle_full_width",
            SessionAction::ToggleFullWidth(WindowId(1)),
        ),
        (
            "grow_column_width",
            SessionAction::StepWidth {
                window: WindowId(1),
                step: Step::Grow,
                by: Proportion::TENTH,
            },
        ),
        (
            "shrink_column_width",
            SessionAction::StepWidth {
                window: WindowId(1),
                step: Step::Shrink,
                by: Proportion::TENTH,
            },
        ),
        (
            "grow_window_height",
            SessionAction::StepHeight {
                window: WindowId(1),
                step: Step::Grow,
                by: Proportion::TENTH,
            },
        ),
        (
            "shrink_window_height",
            SessionAction::StepHeight {
                window: WindowId(1),
                step: Step::Shrink,
                by: Proportion::TENTH,
            },
        ),
        (
            "reset_window_height",
            SessionAction::ResetHeight(WindowId(1)),
        ),
        (
            "move_column_left",
            SessionAction::MoveColumn {
                window: WindowId(1),
                direction: Direction::Left,
            },
        ),
        (
            "move_column_right",
            SessionAction::MoveColumn {
                window: WindowId(1),
                direction: Direction::Right,
            },
        ),
        (
            "move_window_down",
            SessionAction::MoveWindow {
                window: WindowId(1),
                direction: Vertical::Down,
            },
        ),
        (
            "move_window_up",
            SessionAction::MoveWindow {
                window: WindowId(1),
                direction: Vertical::Up,
            },
        ),
    ];
    let (_scratch, config) = loaded_with(
        "window-actions",
        JOB,
        state(two_bands(), 2, Some(3), "root"),
    );
    for (name, action) in expected {
        let outcome = run_job(&config, &format!("gband.action.{name}({{ window = 1 }})"));
        clean(&outcome);
        assert_eq!(outcome.dispatched, [session(action)], "{name}");
    }
}

#[test]
fn open_window_targets() {
    let (_scratch, config) =
        loaded_with("open-targets", JOB, state(two_bands(), 1, Some(1), "root"));
    for (code, band, after) in [
        ("{ band = 2 }", 2, None),
        ("{ after = 3 }", 2, Some(3)),
        ("{ band = 1, after = 1 }", 1, Some(1)),
    ] {
        let outcome = run_job(&config, &format!("gband.action.open_window({code})"));
        clean(&outcome);
        assert_eq!(
            outcome.dispatched,
            [session(SessionAction::open(
                BandId(band),
                after.map(WindowId),
                None
            ))],
            "{code}"
        );
    }
}

#[test]
fn send_prefix_to_a_named_window() {
    assert_eq!(
        dispatched(
            "prefix-target",
            state(two_bands(), 1, Some(2), "root"),
            "gband.action.send_prefix({ window = 1 })"
        ),
        [Dispatch::Input {
            window: WindowId(1),
            input: WindowInput::Key(Key::new(KeyCode::Char(' '), Modifiers::CTRL)),
        }]
    );
}

#[test]
fn resize_by_named_edges() {
    let view = || state(two_bands(), 1, Some(1), "root");
    let resize = |edges| {
        [Dispatch::Action(Action::Client(ClientAction::DragResize(
            Some(edges),
        )))]
    };
    assert_eq!(
        dispatched(
            "edges-bottom",
            view(),
            "gband.action.drag_resize_window({ edges = { 'bottom' } })"
        ),
        resize(Edges {
            bottom: true,
            ..Edges::default()
        })
    );
    let corner = resize(Edges {
        left: true,
        top: true,
        ..Edges::default()
    });
    for (index, code) in [
        "gband.action.drag_resize_window({ edges = { 'top', 'left' } })",
        "gband.action.drag_resize_window({ edges = { 'left', 'top' } })",
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            dispatched(&format!("edges-corner-{index}"), view(), code),
            corner
        );
    }
}

#[test]
fn grow_by_a_given_step() {
    let view = || state(two_bands(), 1, Some(1), "root");
    assert_eq!(
        dispatched(
            "step-given",
            view(),
            "gband.action.grow_column_width({ step = 1/4 })"
        ),
        [Dispatch::Action(Action::Session(
            SessionCommand::StepWidth {
                step: Step::Grow,
                by: Proportion::new(1, 4),
            }
        ))]
    );
    assert_eq!(
        dispatched(
            "step-from-a-target",
            view(),
            "gband.action.shrink_window_height({ window = 2, step = 1/4 })"
        ),
        [session(SessionAction::StepHeight {
            window: WindowId(2),
            step: Step::Shrink,
            by: Proportion::new(1, 4),
        })]
    );
}

#[test]
fn steps_from_the_client_options() {
    let source = format!("gband.opt.width_step = 1/20\ngband.opt.height_step = 1/5\n{JOB}");
    let (_scratch, config) = loaded_with(
        "step-options",
        &source,
        state(two_bands(), 1, Some(1), "root"),
    );
    let outcome = run_job(
        &config,
        "gband.action.grow_column_width()\ngband.action.grow_window_height({ window = 2 })",
    );
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [
            Dispatch::Action(Action::Session(SessionCommand::StepWidth {
                step: Step::Grow,
                by: Proportion::new(1, 20),
            })),
            session(SessionAction::StepHeight {
                window: WindowId(2),
                step: Step::Grow,
                by: Proportion::new(1, 5),
            }),
        ]
    );
}

#[test]
fn step_out_of_range() {
    let view = || state(two_bands(), 1, Some(1), "root");
    for (index, code) in [
        "gband.action.grow_window_height({ step = 2 })",
        "gband.action.shrink_column_width({ window = 1, step = 0 })",
        "gband.action.grow_column_width({ step = 10001 })",
        "gband.action.grow_column_width({ step = 'big' })",
        "gband.action.close_window({ window = 1, step = 1/4 })",
    ]
    .into_iter()
    .enumerate()
    {
        failed(&format!("step-out-of-range-{index}"), view(), code, "step");
    }
    let scratch = Scratch::new("step-out-of-range-line");
    let path = scratch.write(
        "\n\n\ngband.bind('alt+s', function() gband.action.grow_window_height({ step = 2 }) end)\n",
    );
    let config = scratch.loaded();
    clean(
        &config
            .runtime
            .set_state(state(two_bands(), 1, Some(1), "root")),
    );
    let chord = gband_lua::Chord::Key(key("alt+s"));
    let Some((_, gband_lua::Binding::Callback(callback))) = config.keymap["root"]
        .iter()
        .find(|(bound, _)| *bound == chord)
    else {
        panic!("alt+s is not bound to a function");
    };
    let outcome = config.runtime.call(*callback);
    assert!(outcome.dispatched.is_empty());
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_error_at(error, &path, 4, "step");
}

#[test]
fn bad_targets_are_errors() {
    let view = || state(two_bands(), 1, Some(1), "root");
    for (index, (code, mentions)) in [
        ("gband.action.close_window({ window = 99 })", "99"),
        ("gband.action.close_window(1)", "table"),
        ("gband.action.close_window({ band = 1 })", "band"),
        ("gband.action.close_window({})", "window"),
        ("gband.action.open_window({ band = 9 })", "9"),
        (
            "gband.action.open_window({ band = 1, after = 3 })",
            "not in band 1",
        ),
        ("gband.action.open_window({ window = 1 })", "window"),
        ("gband.action.open_window({})", "band"),
        (
            "gband.action.focus_column_left({ window = 1 })",
            "focus_column_left",
        ),
        ("gband.action.detach({})", "detach"),
        ("gband.action.drag_resize_window({ edges = {} })", "`edges`"),
        (
            "gband.action.drag_resize_window({ edges = { 'middle' } })",
            "middle",
        ),
        (
            "gband.action.drag_resize_window({ edges = { 'top', 'top' } })",
            "`top` twice",
        ),
        (
            "gband.action.drag_resize_window({ edges = { 'left', 'right' } })",
            "`left` and `right`",
        ),
        (
            "gband.action.drag_resize_window({ edges = { 'top', 'bottom' } })",
            "`top` and `bottom`",
        ),
        (
            "gband.action.drag_resize_window({ edges = 'left' })",
            "list of edge names",
        ),
        (
            "gband.action.drag_resize_window({ edges = { left = true } })",
            "list of edge names",
        ),
        ("gband.action.drag_resize_window({})", "`edges`"),
        (
            "gband.action.drag_resize_window({ edges = { 'top' }, window = 1 })",
            "window",
        ),
        ("gband.action.drag_resize_window(1)", "table"),
        (
            "gband.action.drag_window({ edges = { 'left' } })",
            "drag_window",
        ),
        ("gband.action.drag_band({ edges = { 'top' } })", "drag_band"),
    ]
    .into_iter()
    .enumerate()
    {
        failed(&format!("bad-target-{index}"), view(), code, mentions);
    }
}

#[test]
fn unknown_window_is_an_error_at_the_line() {
    let scratch = Scratch::new("unknown-window");
    let path = scratch.write(
        "\n\n\n\n\ngband.bind('alt+q', function() gband.action.close_window({ window = 99 }) end)\n",
    );
    let config = scratch.loaded();
    clean(
        &config
            .runtime
            .set_state(state(two_bands(), 1, Some(1), "root")),
    );
    let chord = gband_lua::Chord::Key(key("alt+q"));
    let Some((_, gband_lua::Binding::Callback(callback))) = config.keymap["root"]
        .iter()
        .find(|(bound, _)| *bound == chord)
    else {
        panic!("alt+q is not bound to a function");
    };
    let outcome = config.runtime.call(*callback);
    assert!(outcome.dispatched.is_empty());
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_error_at(error, &path, 6, "window 99");
}

#[test]
fn floating_not_a_boolean_is_an_error_at_the_line() {
    let scratch = Scratch::new("floating-not-boolean");
    let path = scratch.write(
        "\n\n\n\n\n\ngband.bind('alt+f', function() gband.action.toggle_window_floating({ window = 3, floating = 'yes' }) end)\n",
    );
    let config = scratch.loaded();
    clean(
        &config
            .runtime
            .set_state(state(two_bands(), 1, Some(1), "root")),
    );
    let chord = gband_lua::Chord::Key(key("alt+f"));
    let Some((_, gband_lua::Binding::Callback(callback))) = config.keymap["root"]
        .iter()
        .find(|(bound, _)| *bound == chord)
    else {
        panic!("alt+f is not bound to a function");
    };
    let outcome = config.runtime.call(*callback);
    assert!(outcome.dispatched.is_empty());
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_error_at(error, &path, 7, "floating");
}

#[test]
fn opposite_edges_are_an_error_at_the_line() {
    let scratch = Scratch::new("opposite-edges");
    let path = scratch.write(
        "\n\n\n\ngband.bind('alt+r', function() gband.action.drag_resize_window({ edges = { 'left', 'right' } }) end)\n",
    );
    let config = scratch.loaded();
    clean(
        &config
            .runtime
            .set_state(state(two_bands(), 1, Some(1), "root")),
    );
    let chord = gband_lua::Chord::Key(key("alt+r"));
    let Some((_, gband_lua::Binding::Callback(callback))) = config.keymap["root"]
        .iter()
        .find(|(bound, _)| *bound == chord)
    else {
        panic!("alt+r is not bound to a function");
    };
    let outcome = config.runtime.call(*callback);
    assert!(outcome.dispatched.is_empty());
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_error_at(error, &path, 5, "`left` and `right`");
}

#[test]
fn focus_and_view_by_number() {
    assert_eq!(
        dispatched(
            "focus-view",
            state(two_bands(), 1, Some(1), "root"),
            "gband.window.focus(3)\ngband.band.view(2)"
        ),
        [
            Dispatch::Action(Action::View(ViewAction::FocusWindow(WindowId(3)))),
            Dispatch::Action(Action::View(ViewAction::ViewBand(BandId(2)))),
        ]
    );
    failed(
        "focus-unknown",
        state(two_bands(), 1, Some(1), "root"),
        "gband.window.focus(9)",
        "9",
    );
    failed(
        "view-unknown",
        state(two_bands(), 1, Some(1), "root"),
        "gband.band.view(9)",
        "9",
    );
}

#[test]
fn exact_sizes() {
    assert_eq!(
        dispatched(
            "sizes",
            state(two_bands(), 1, Some(1), "root"),
            "gband.window.set_width(1, 0.4)\n\
             gband.window.set_height(1, { rows = 8 })\n\
             gband.window.set_height(2, { weight = 1.5 })"
        ),
        [
            session(SessionAction::SetWidth {
                window: WindowId(1),
                width: Proportion::new(2, 5),
            }),
            session(SessionAction::SetHeight {
                window: WindowId(1),
                height: WindowHeight::Fixed(8),
            }),
            session(SessionAction::SetHeight {
                window: WindowId(2),
                height: WindowHeight::Auto(Weight::new(3, 2)),
            }),
        ]
    );
}

#[test]
fn bad_sizes_are_errors() {
    for (index, code) in [
        "gband.window.set_height(1, { rows = 8, weight = 1 })",
        "gband.window.set_height(1, {})",
        "gband.window.set_height(1, { rows = 0 })",
        "gband.window.set_height(1, { rows = 1.5 })",
        "gband.window.set_height(1, { weight = 0 })",
        "gband.window.set_height(1, { lines = 3 })",
        "gband.window.set_height(1, 8)",
        "gband.window.set_width(1, 0)",
        "gband.window.set_width(1, 10001)",
        "gband.window.set_width(1, 'half')",
        "gband.window.set_width(9, 0.5)",
    ]
    .into_iter()
    .enumerate()
    {
        failed(
            &format!("bad-size-{index}"),
            state(two_bands(), 1, Some(1), "root"),
            code,
            "gband.window",
        );
    }
}

fn keys(window: u32, keys: &[Key]) -> Vec<Dispatch> {
    keys.iter()
        .map(|&key| Dispatch::Input {
            window: WindowId(window),
            input: WindowInput::Key(key),
        })
        .collect()
}

#[test]
fn input_to_a_named_window() {
    let text: Vec<Key> = "echo hi"
        .chars()
        .map(|c| Key::plain(KeyCode::Char(c)))
        .chain([Key::plain(KeyCode::Enter), Key::plain(KeyCode::Tab)])
        .collect();
    assert_eq!(
        dispatched(
            "send-text",
            state(two_bands(), 1, Some(2), "root"),
            "gband.window.send_text(1, 'echo hi\\n\\t')"
        ),
        keys(1, &text)
    );
    assert_eq!(
        dispatched(
            "send-keys",
            state(two_bands(), 1, Some(2), "root"),
            "gband.window.send_keys(1, 'ctrl+c')\ngband.window.send_keys(2, { 'ctrl+space', 'up' })"
        ),
        [
            keys(1, &[key("ctrl+c")]),
            keys(2, &[key("ctrl+space"), key("up")]),
        ]
        .concat()
    );
    assert_eq!(
        dispatched(
            "paste",
            state(two_bands(), 1, Some(2), "root"),
            "gband.window.paste(1, 'a b')"
        ),
        [Dispatch::Input {
            window: WindowId(1),
            input: WindowInput::Paste("a b".to_owned()),
        }]
    );
}

#[test]
fn bad_input_is_an_error() {
    for (index, (code, mentions)) in [
        ("gband.window.send_keys(1, 'ctrl+shift+1')", "ctrl+shift+1"),
        ("gband.window.send_keys(1, { 'a', 5 })", "key name"),
        ("gband.window.send_text(1, 'a\\27b')", "U+001B"),
        ("gband.window.send_text(9, 'a')", "9"),
        ("gband.window.paste(1, 5)", "string"),
    ]
    .into_iter()
    .enumerate()
    {
        failed(
            &format!("bad-input-{index}"),
            state(two_bands(), 1, Some(1), "root"),
            code,
            mentions,
        );
    }
}

#[test]
fn dispatch_order_follows_the_calls() {
    assert_eq!(
        dispatched(
            "order",
            state(two_bands(), 1, Some(1), "root"),
            "gband.window.set_width(1, 1/3)\ngband.action.cycle_column_width({ window = 1 })"
        ),
        [
            session(SessionAction::SetWidth {
                window: WindowId(1),
                width: Proportion::ONE_THIRD,
            }),
            session(SessionAction::CycleWidth(WindowId(1))),
        ]
    );
}

#[test]
fn set_position_keeps_the_dispatch_order() {
    assert_eq!(
        dispatched(
            "order-position",
            state(with_floating(), 1, Some(1), "root"),
            "gband.action.move_column_right({ window = 4 })\ngband.window.set_position(4, { col = 1, row = 1 })\ngband.action.toggle_window_floating({ window = 4, after = 1 })"
        ),
        [
            session(SessionAction::MoveColumn {
                window: WindowId(4),
                direction: Direction::Right,
            }),
            session(SessionAction::SetPosition {
                window: WindowId(4),
                col: 1,
                row: 1,
            }),
            session(SessionAction::ToggleFloating {
                window: WindowId(4),
                after: Some(WindowId(1)),
                floating: None,
            }),
        ]
    );
}

#[test]
fn set_position_while_loading() {
    let scratch = Scratch::new("position-loading");
    let path = scratch.write("\ngband.window.set_position(1, { col = 0, row = 0 })\n");
    let error = scratch.load().map(|_| ()).unwrap_err();
    assert_error_at(&error, &path, 2, "binding function");
}

#[test]
fn window_control_while_loading() {
    let scratch = Scratch::new("window-loading");
    let path = scratch.write("\n\ngband.window.focus(1)\n");
    let error = scratch.load().map(|_| ()).unwrap_err();
    assert_error_at(&error, &path, 3, "binding function");
}

#[test]
fn spawn_after_a_named_window() {
    assert_eq!(
        dispatched(
            "spawn-after",
            state(two_bands(), 1, Some(2), "root"),
            "gband.spawn({ cmd = 'fish', after = 1 })"
        ),
        [session(SessionAction::OpenWindow {
            band: BandId(1),
            after: Some(WindowId(1)),
            width: None,
            floating: false,
            focus: true,
            content: WindowContent::Program(Some(Program::CommandLine("fish".to_owned()))),
        })]
    );
}

#[test]
fn spawn_rejects_an_unknown_field() {
    failed(
        "spawn-width",
        state(two_bands(), 1, Some(2), "root"),
        "gband.spawn({ cmd = 'fish', width = 1/2 })",
        "width",
    );
    failed(
        "spawn-bad-after",
        state(two_bands(), 1, Some(2), "root"),
        "gband.spawn({ after = 9 })",
        "9",
    );
}

fn named(layout: Layout, names: &[(u32, &str, Option<&str>)]) -> ViewState {
    let names: WindowNames = names
        .iter()
        .map(|&(window, shown, manual)| {
            (
                WindowId(window),
                WindowName {
                    shown: shown.to_owned(),
                    manual: manual.map(str::to_owned),
                },
            )
        })
        .collect();
    ViewState {
        names: Arc::new(names),
        ..state(layout, 1, Some(1), "root")
    }
}

fn three_windows() -> Layout {
    let mut layout = Layout::new();
    let first = opened(&mut layout, 0, None);
    let second = opened(&mut layout, 0, Some(first));
    opened(&mut layout, 0, Some(second));
    layout
}

#[test]
fn window_names() {
    let view = named(
        three_windows(),
        &[
            (1, "bash #1", None),
            (2, "bash #2", None),
            (3, "notes", Some("notes")),
        ],
    );
    let (_scratch, config) = loaded_with("names", "", view);
    let summary: String = eval(
        &config,
        r#"
        local parts = {}
        for _, column in ipairs(gband.layout().bands[1].columns) do
          for _, window in ipairs(column.windows) do
            parts[#parts + 1] = string.format("%d %s %s", window.id, tostring(window.name), tostring(window.manual_name))
          end
        end
        return table.concat(parts, "; ")
        "#,
    );
    assert_eq!(summary, "1 bash #1 nil; 2 bash #2 nil; 3 notes notes");
}

#[test]
fn floating_window_names() {
    let view = named(with_floating(), &[(4, "vim", None)]);
    let (_scratch, config) = loaded_with("names-floating", "", view);
    let name: String = eval(&config, "return gband.layout().bands[1].floating[1].name");
    assert_eq!(name, "vim");
}

fn drawn_window() -> (Scratch, Config) {
    let (scratch, config) = loaded_with("names-drawn", JOB, state(two_bands(), 1, Some(1), "root"));
    let outcome = run_job(&config, "gband.win.open({ kind = 'tiled' })");
    let [Dispatch::PluginWindow(PluginWindowRequest::Open { plugin_window, .. })] =
        outcome.dispatched.as_slice()
    else {
        panic!("{:?}", outcome.dispatched);
    };
    clean(
        &config
            .runtime
            .plugin_window_opened(*plugin_window, Some(WindowId(2))),
    );
    (scratch, config)
}

#[test]
fn drawn_window_has_no_name() {
    let (_scratch, config) = drawn_window();
    let summary: String = eval(
        &config,
        r#"
        local window = gband.layout().bands[1].columns[2].windows[1]
        return string.format("%s %s", tostring(window.plugin_window), tostring(window.name))
        "#,
    );
    assert_eq!(summary, "1 nil");
}

#[test]
fn rename_a_drawn_window() {
    let (_scratch, config) = drawn_window();
    let outcome = run_job(&config, "gband.window.rename(2, 'x')");
    assert!(outcome.dispatched.is_empty(), "{:?}", outcome.dispatched);
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert!(error.message.contains("drawn window"), "{error}");
}

#[test]
fn rename_dispatched_in_order() {
    assert_eq!(
        dispatched(
            "rename",
            state(two_bands(), 1, Some(1), "root"),
            "gband.window.focus(2)\ngband.window.rename(1, '  logs ')\ngband.window.rename(2, '')\ngband.window.rename(2)"
        ),
        [
            Dispatch::Action(Action::View(ViewAction::FocusWindow(WindowId(2)))),
            Dispatch::Rename {
                window: WindowId(1),
                name: Some("logs".to_owned()),
            },
            Dispatch::Rename {
                window: WindowId(2),
                name: None,
            },
            Dispatch::Rename {
                window: WindowId(2),
                name: None,
            },
        ]
    );
}

#[test]
fn rename_errors() {
    let view = || state(two_bands(), 1, Some(1), "root");
    failed(
        "rename-unknown",
        view(),
        "gband.window.rename(9, 'x')",
        "window 9",
    );
    failed(
        "rename-control",
        view(),
        "gband.window.rename(1, 'a\\nb')",
        "control character",
    );
    failed(
        "rename-type",
        view(),
        "gband.window.rename(1, 3)",
        "string or nil",
    );
}

#[test]
fn rename_during_loading() {
    let scratch = Scratch::new("rename-loading");
    let path = scratch.write("\n\ngband.window.rename(1, 'logs')\n");
    let error = scratch.load().map(|_| ()).unwrap_err();
    assert_error_at(&error, &path, 3, "binding function");
}
