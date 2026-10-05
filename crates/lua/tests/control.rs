mod common;

use std::sync::Arc;

use common::*;
use gband_core::action::Action;
use gband_core::geometry::Size;
use gband_core::input::{Key, KeyCode, Modifiers};
use gband_core::layout::{
    BandId, Direction, Layout, LayoutOptions, PaneContent, PaneHeight, PaneId, Program, Proportion,
    SessionAction, Step, Vertical, Weight,
};
use gband_core::view::ViewAction;
use gband_lua::{BandState, Config, Dispatch, Outcome, PaneInput, ViewState};
use mlua::Table;

const AREA: Size = Size::new(80, 24);

fn opened(layout: &mut Layout, band: usize, after: Option<PaneId>) -> PaneId {
    let pane = layout.allocate_pane();
    let id = layout.bands()[band].id;
    layout.open(pane, id, after, None, &LayoutOptions::default());
    pane
}

fn two_bands() -> Layout {
    let mut layout = Layout::new();
    let first = opened(&mut layout, 0, None);
    opened(&mut layout, 0, Some(first));
    opened(&mut layout, 1, None);
    layout
}

fn state(layout: Layout, band: u32, pane: Option<u32>, table: &str) -> ViewState {
    let count = layout.bands().len() as u32;
    ViewState {
        table: table.to_owned(),
        band: BandState {
            number: band,
            index: 1,
            count,
        },
        pane,
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
        layout.allocate_pane(),
        layout.allocate_pane(),
        layout.allocate_pane(),
    );
    layout.open(p1, band, None, None, &options);
    layout.open(p2, band, Some(p1), Some(Proportion::ONE_THIRD), &options);
    layout.open(p3, band, Some(p2), None, &options);
    for action in [
        SessionAction::ConsumeOrExpel {
            pane: p3,
            direction: Direction::Left,
        },
        SessionAction::ToggleFullWidth(p2),
        SessionAction::SetHeight {
            pane: p2,
            height: PaneHeight::Fixed(10),
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
            for _, pane in ipairs(column.panes) do
              parts[#parts + 1] = string.format("%d %s %s %s", pane.id, tostring(pane.rows), tostring(pane.weight), tostring(pane.window))
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
    assert_eq!(view.get::<Option<u32>>("pane").unwrap(), Some(2));
    assert_eq!(view.get::<Option<u32>>("window").unwrap(), None);
    assert_eq!(view.get::<String>("table").unwrap(), "prefix");
    assert_eq!(view.get::<u16>("cols").unwrap(), 80);
    assert_eq!(view.get::<u16>("rows").unwrap(), 23);
    assert!(!view.get::<bool>("floating").unwrap());
    assert_eq!(view.pairs::<String, mlua::Value>().count(), 6);
}

fn with_floating() -> Layout {
    let mut layout = two_bands();
    let options = LayoutOptions::default();
    let floating = layout.allocate_pane();
    layout.open_floating(floating, BandId(1), None, AREA, &options);
    let (pane, col, row) = (floating, 50, 3);
    let height = PaneHeight::Fixed(10);
    layout.apply(SessionAction::SetHeight { pane, height }, AREA, &options);
    layout.apply(
        SessionAction::SetPosition { pane, col, row },
        AREA,
        &options,
    );
    layout
}

#[test]
fn read_a_floating_pane() {
    let layout = with_floating();
    assert_eq!(layout.floating(PaneId(4)).unwrap().col, 40);
    let (_scratch, config) = loaded_with("layout-floating", "", state(layout, 1, Some(1), "root"));
    let summary: String = eval(
        &config,
        r#"
        local parts = {}
        for _, band in ipairs(gband.layout().bands) do
          for _, f in ipairs(band.floating) do
            parts[#parts + 1] = string.format("%d %.1f %s %d %d %d %s", f.id, f.width, tostring(f.full_width), f.rows, f.col, f.row, tostring(f.window))
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
    assert_eq!(view.get::<Option<u32>>("pane").unwrap(), Some(4));
    assert!(view.get::<bool>("floating").unwrap());
}

#[test]
fn open_a_floating_pane() {
    let floating = |band| {
        session(SessionAction::OpenPane {
            band: BandId(band),
            after: None,
            width: None,
            floating: true,
            focus: true,
            content: PaneContent::Program(None),
        })
    };
    assert_eq!(
        dispatched(
            "open-floating",
            state(two_bands(), 1, Some(1), "root"),
            "gband.action.open_pane({ floating = true })"
        ),
        [floating(1)]
    );
    assert_eq!(
        dispatched(
            "open-floating-band",
            state(two_bands(), 1, Some(1), "root"),
            "gband.action.open_pane({ band = 2, floating = true })"
        ),
        [floating(2)]
    );
    assert_eq!(
        dispatched(
            "open-tiled",
            state(two_bands(), 1, Some(1), "root"),
            "gband.action.open_pane({ after = 1, floating = false })"
        ),
        [session(SessionAction::open(
            BandId(1),
            Some(PaneId(1)),
            None
        ))]
    );
}

#[test]
fn floating_with_after() {
    failed(
        "floating-after",
        state(two_bands(), 1, Some(1), "root"),
        "gband.action.open_pane({ after = 1, floating = true })",
        "after",
    );
    failed(
        "floating-not-boolean",
        state(two_bands(), 1, Some(1), "root"),
        "gband.action.open_pane({ floating = 'yes' })",
        "floating",
    );
}

#[test]
fn open_after_a_floating_pane() {
    failed(
        "open-after-floating",
        state(with_floating(), 1, Some(1), "root"),
        "gband.action.open_pane({ after = 4 })",
        "4",
    );
}

#[test]
fn tile_a_named_pane_after_a_named_pane() {
    assert_eq!(
        dispatched(
            "tile-after",
            state(with_floating(), 1, Some(1), "root"),
            "gband.action.toggle_pane_floating({ pane = 4, after = 1 })"
        ),
        [session(SessionAction::ToggleFloating {
            pane: PaneId(4),
            after: Some(PaneId(1)),
        })]
    );
    assert_eq!(
        dispatched(
            "tile-default",
            state(with_floating(), 1, Some(1), "root"),
            "gband.action.toggle_pane_floating({ pane = 4 })"
        ),
        [session(SessionAction::ToggleFloating {
            pane: PaneId(4),
            after: None,
        })]
    );
}

#[test]
fn tile_after_a_pane_of_another_band() {
    failed(
        "tile-other-band",
        state(with_floating(), 1, Some(1), "root"),
        "gband.action.toggle_pane_floating({ pane = 4, after = 3 })",
        "3",
    );
    failed(
        "tile-without-pane",
        state(with_floating(), 1, Some(1), "root"),
        "gband.action.toggle_pane_floating({ after = 1 })",
        "pane",
    );
}

#[test]
fn place_a_floating_pane() {
    assert_eq!(
        dispatched(
            "set-position",
            state(with_floating(), 1, Some(1), "root"),
            "gband.pane.set_position(4, { col = 10, row = 2 })"
        ),
        [session(SessionAction::SetPosition {
            pane: PaneId(4),
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
            "gband.pane.set_position(1, { col = 0, row = 0 })",
            "pane 1",
        ),
        ("missing", "gband.pane.set_position(4, { col = 4 })", "row"),
        (
            "unknown",
            "gband.pane.set_position(4, { col = 4, row = 1, z = 2 })",
            "col",
        ),
        (
            "negative",
            "gband.pane.set_position(4, { col = -1, row = 1 })",
            "col",
        ),
        (
            "fraction",
            "gband.pane.set_position(4, { col = 1.5, row = 1 })",
            "col",
        ),
        (
            "absent",
            "gband.pane.set_position(9, { col = 1, row = 1 })",
            "9",
        ),
        ("not-a-table", "gband.pane.set_position(4, 3)", "col"),
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
    let pane: Option<u32> = eval(&config, "return gband.view().pane");
    assert_eq!(pane, None);
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
        "gband.action.focus_column_right()\nseen = gband.view().pane",
    );
    clean(&outcome);
    assert_eq!(global::<u32>(&config, "seen"), 1);
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusRight))]
    );
}

#[test]
fn close_a_named_pane() {
    assert_eq!(
        dispatched(
            "close-named",
            state(two_bands(), 1, Some(2), "root"),
            "gband.action.close_pane({ pane = 1 })"
        ),
        [session(SessionAction::ClosePane(PaneId(1)))]
    );
}

#[test]
fn target_overrides_the_view() {
    assert_eq!(
        dispatched(
            "target-overrides",
            state(two_bands(), 1, Some(2), "root"),
            "gband.action.cycle_column_width({ pane = 1 })"
        ),
        [session(SessionAction::CycleWidth(PaneId(1)))]
    );
}

#[test]
fn target_on_the_empty_band() {
    assert_eq!(
        dispatched(
            "target-empty",
            state(two_bands(), 3, None, "root"),
            "gband.action.close_pane({ pane = 1 })"
        ),
        [session(SessionAction::ClosePane(PaneId(1)))]
    );
}

#[test]
fn every_pane_action_takes_a_target() {
    let expected = [
        ("close_pane", SessionAction::ClosePane(PaneId(1))),
        (
            "consume_or_expel_left",
            SessionAction::ConsumeOrExpel {
                pane: PaneId(1),
                direction: Direction::Left,
            },
        ),
        (
            "consume_or_expel_right",
            SessionAction::ConsumeOrExpel {
                pane: PaneId(1),
                direction: Direction::Right,
            },
        ),
        ("cycle_column_width", SessionAction::CycleWidth(PaneId(1))),
        (
            "toggle_full_width",
            SessionAction::ToggleFullWidth(PaneId(1)),
        ),
        (
            "grow_column_width",
            SessionAction::StepWidth {
                pane: PaneId(1),
                step: Step::Grow,
            },
        ),
        (
            "shrink_column_width",
            SessionAction::StepWidth {
                pane: PaneId(1),
                step: Step::Shrink,
            },
        ),
        (
            "grow_pane_height",
            SessionAction::StepHeight {
                pane: PaneId(1),
                step: Step::Grow,
            },
        ),
        (
            "shrink_pane_height",
            SessionAction::StepHeight {
                pane: PaneId(1),
                step: Step::Shrink,
            },
        ),
        ("reset_pane_height", SessionAction::ResetHeight(PaneId(1))),
        (
            "move_column_left",
            SessionAction::MoveColumn {
                pane: PaneId(1),
                direction: Direction::Left,
            },
        ),
        (
            "move_column_right",
            SessionAction::MoveColumn {
                pane: PaneId(1),
                direction: Direction::Right,
            },
        ),
        (
            "move_pane_down",
            SessionAction::MovePane {
                pane: PaneId(1),
                direction: Vertical::Down,
            },
        ),
        (
            "move_pane_up",
            SessionAction::MovePane {
                pane: PaneId(1),
                direction: Vertical::Up,
            },
        ),
    ];
    let (_scratch, config) =
        loaded_with("pane-actions", JOB, state(two_bands(), 2, Some(3), "root"));
    for (name, action) in expected {
        let outcome = run_job(&config, &format!("gband.action.{name}({{ pane = 1 }})"));
        clean(&outcome);
        assert_eq!(outcome.dispatched, [session(action)], "{name}");
    }
}

#[test]
fn open_pane_targets() {
    let (_scratch, config) =
        loaded_with("open-targets", JOB, state(two_bands(), 1, Some(1), "root"));
    for (code, band, after) in [
        ("{ band = 2 }", 2, None),
        ("{ after = 3 }", 2, Some(3)),
        ("{ band = 1, after = 1 }", 1, Some(1)),
    ] {
        let outcome = run_job(&config, &format!("gband.action.open_pane({code})"));
        clean(&outcome);
        assert_eq!(
            outcome.dispatched,
            [session(SessionAction::open(
                BandId(band),
                after.map(PaneId),
                None
            ))],
            "{code}"
        );
    }
}

#[test]
fn send_prefix_to_a_named_pane() {
    assert_eq!(
        dispatched(
            "prefix-target",
            state(two_bands(), 1, Some(2), "root"),
            "gband.action.send_prefix({ pane = 1 })"
        ),
        [Dispatch::Input {
            pane: PaneId(1),
            input: PaneInput::Key(Key::new(KeyCode::Char(' '), Modifiers::CTRL)),
        }]
    );
}

#[test]
fn bad_targets_are_errors() {
    let view = || state(two_bands(), 1, Some(1), "root");
    for (index, (code, mentions)) in [
        ("gband.action.close_pane({ pane = 99 })", "99"),
        ("gband.action.close_pane(1)", "table"),
        ("gband.action.close_pane({ band = 1 })", "band"),
        ("gband.action.close_pane({})", "pane"),
        ("gband.action.open_pane({ band = 9 })", "9"),
        (
            "gband.action.open_pane({ band = 1, after = 3 })",
            "not in band 1",
        ),
        ("gband.action.open_pane({ pane = 1 })", "pane"),
        ("gband.action.open_pane({})", "band"),
        (
            "gband.action.focus_column_left({ pane = 1 })",
            "focus_column_left",
        ),
        ("gband.action.detach({})", "detach"),
    ]
    .into_iter()
    .enumerate()
    {
        failed(&format!("bad-target-{index}"), view(), code, mentions);
    }
}

#[test]
fn unknown_pane_is_an_error_at_the_line() {
    let scratch = Scratch::new("unknown-pane");
    let path = scratch.write(
        "\n\n\n\n\ngband.bind('alt+q', function() gband.action.close_pane({ pane = 99 }) end)\n",
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
    assert_error_at(error, &path, 6, "pane 99");
}

#[test]
fn focus_and_view_by_number() {
    assert_eq!(
        dispatched(
            "focus-view",
            state(two_bands(), 1, Some(1), "root"),
            "gband.pane.focus(3)\ngband.band.view(2)"
        ),
        [
            Dispatch::Action(Action::View(ViewAction::FocusPane(PaneId(3)))),
            Dispatch::Action(Action::View(ViewAction::ViewBand(BandId(2)))),
        ]
    );
    failed(
        "focus-unknown",
        state(two_bands(), 1, Some(1), "root"),
        "gband.pane.focus(9)",
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
            "gband.pane.set_width(1, 0.4)\n\
             gband.pane.set_height(1, { rows = 8 })\n\
             gband.pane.set_height(2, { weight = 1.5 })"
        ),
        [
            session(SessionAction::SetWidth {
                pane: PaneId(1),
                width: Proportion::new(2, 5),
            }),
            session(SessionAction::SetHeight {
                pane: PaneId(1),
                height: PaneHeight::Fixed(8),
            }),
            session(SessionAction::SetHeight {
                pane: PaneId(2),
                height: PaneHeight::Auto(Weight::new(3, 2)),
            }),
        ]
    );
}

#[test]
fn bad_sizes_are_errors() {
    for (index, code) in [
        "gband.pane.set_height(1, { rows = 8, weight = 1 })",
        "gband.pane.set_height(1, {})",
        "gband.pane.set_height(1, { rows = 0 })",
        "gband.pane.set_height(1, { rows = 1.5 })",
        "gband.pane.set_height(1, { weight = 0 })",
        "gband.pane.set_height(1, { lines = 3 })",
        "gband.pane.set_height(1, 8)",
        "gband.pane.set_width(1, 0)",
        "gband.pane.set_width(1, 10001)",
        "gband.pane.set_width(1, 'half')",
        "gband.pane.set_width(9, 0.5)",
    ]
    .into_iter()
    .enumerate()
    {
        failed(
            &format!("bad-size-{index}"),
            state(two_bands(), 1, Some(1), "root"),
            code,
            "gband.pane",
        );
    }
}

fn keys(pane: u32, keys: &[Key]) -> Vec<Dispatch> {
    keys.iter()
        .map(|&key| Dispatch::Input {
            pane: PaneId(pane),
            input: PaneInput::Key(key),
        })
        .collect()
}

#[test]
fn input_to_a_named_pane() {
    let text: Vec<Key> = "echo hi"
        .chars()
        .map(|c| Key::plain(KeyCode::Char(c)))
        .chain([Key::plain(KeyCode::Enter), Key::plain(KeyCode::Tab)])
        .collect();
    assert_eq!(
        dispatched(
            "send-text",
            state(two_bands(), 1, Some(2), "root"),
            "gband.pane.send_text(1, 'echo hi\\n\\t')"
        ),
        keys(1, &text)
    );
    assert_eq!(
        dispatched(
            "send-keys",
            state(two_bands(), 1, Some(2), "root"),
            "gband.pane.send_keys(1, 'ctrl+c')\ngband.pane.send_keys(2, { 'ctrl+space', 'up' })"
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
            "gband.pane.paste(1, 'a b')"
        ),
        [Dispatch::Input {
            pane: PaneId(1),
            input: PaneInput::Paste("a b".to_owned()),
        }]
    );
}

#[test]
fn bad_input_is_an_error() {
    for (index, (code, mentions)) in [
        ("gband.pane.send_keys(1, 'ctrl+shift+1')", "ctrl+shift+1"),
        ("gband.pane.send_keys(1, { 'a', 5 })", "key name"),
        ("gband.pane.send_text(1, 'a\\27b')", "U+001B"),
        ("gband.pane.send_text(9, 'a')", "9"),
        ("gband.pane.paste(1, 5)", "string"),
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
            "gband.pane.set_width(1, 1/3)\ngband.action.cycle_column_width({ pane = 1 })"
        ),
        [
            session(SessionAction::SetWidth {
                pane: PaneId(1),
                width: Proportion::ONE_THIRD,
            }),
            session(SessionAction::CycleWidth(PaneId(1))),
        ]
    );
}

#[test]
fn set_position_keeps_the_dispatch_order() {
    assert_eq!(
        dispatched(
            "order-position",
            state(with_floating(), 1, Some(1), "root"),
            "gband.action.move_column_right({ pane = 4 })\ngband.pane.set_position(4, { col = 1, row = 1 })\ngband.action.toggle_pane_floating({ pane = 4, after = 1 })"
        ),
        [
            session(SessionAction::MoveColumn {
                pane: PaneId(4),
                direction: Direction::Right,
            }),
            session(SessionAction::SetPosition {
                pane: PaneId(4),
                col: 1,
                row: 1,
            }),
            session(SessionAction::ToggleFloating {
                pane: PaneId(4),
                after: Some(PaneId(1)),
            }),
        ]
    );
}

#[test]
fn set_position_while_loading() {
    let scratch = Scratch::new("position-loading");
    let path = scratch.write("\ngband.pane.set_position(1, { col = 0, row = 0 })\n");
    let error = scratch.load().map(|_| ()).unwrap_err();
    assert_error_at(&error, &path, 2, "binding function");
}

#[test]
fn pane_control_while_loading() {
    let scratch = Scratch::new("pane-loading");
    let path = scratch.write("\n\ngband.pane.focus(1)\n");
    let error = scratch.load().map(|_| ()).unwrap_err();
    assert_error_at(&error, &path, 3, "binding function");
}

#[test]
fn spawn_after_a_named_pane() {
    assert_eq!(
        dispatched(
            "spawn-after",
            state(two_bands(), 1, Some(2), "root"),
            "gband.spawn({ cmd = 'fish', after = 1 })"
        ),
        [session(SessionAction::OpenPane {
            band: BandId(1),
            after: Some(PaneId(1)),
            width: None,
            floating: false,
            focus: true,
            content: PaneContent::Program(Some(Program::CommandLine("fish".to_owned()))),
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
