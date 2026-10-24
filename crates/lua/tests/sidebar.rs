mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use common::*;
use gband_core::action::Action;
use gband_core::geometry::Size;
use gband_core::input::{Modifiers, MouseButton};
use gband_core::layout::{Layout, LayoutOptions, WindowId};
use gband_core::view::ViewAction;
use gband_lua::{
    BandState, Bar, BarSide, Color, Config, Dispatch, Event, Outcome, Pointer, PointerTarget,
    ViewState, WindowStates,
};
use gband_protocol::{Key, Value};

fn loaded(name: &str, source: &str) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    scratch.write(source);
    let config = scratch.loaded();
    (scratch, config)
}

fn plugin_error(config: &Config) -> &gband_lua::ConfigError {
    config
        .errors
        .first()
        .unwrap_or_else(|| panic!("no error was reported"))
}

fn resolved(config: &Config, group: &str) -> Vec<String> {
    let mut fields: Vec<String> = eval(
        config,
        &format!(
            "local s = gband.hl.get('{group}', {{ resolve = true }})\nlocal out = {{}} for k, v in pairs(s) do out[#out + 1] = k .. '=' .. tostring(v) end\nreturn out"
        ),
    );
    fields.sort();
    fields
}

#[test]
fn default_sidebar() {
    let (_scratch, config) = loaded("default", "gband.plugin('gband.sidebar')");
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    let bar = presented(&config, drawn(80));
    assert_eq!(bar.slot.side, BarSide::Left);
    assert_eq!(bar.slot.size, 1);
    let placed: Vec<i64> = eval(
        &config,
        "local info = gband.bar.info('sidebar') return { info.col, info.width, info.height }",
    );
    assert_eq!(placed, [0, 1, 24]);
    let side: String = eval(&config, "return gband.bar.info('sidebar').side");
    assert_eq!(side, "left");
}

#[test]
fn sidebar_on_the_right() {
    let (_scratch, config) = loaded("right", "gband.plugin('gband.sidebar', { side = 'right' })");
    let bar = presented(&config, drawn(80));
    assert_eq!(bar.slot.side, BarSide::Right);
    let col: i64 = eval(&config, "return gband.bar.info('sidebar').col");
    assert_eq!(col, 79);
}

#[test]
fn unknown_option() {
    let (_scratch, config) = loaded(
        "unknown-option",
        "ok = gband.plugin('gband.sidebar', { width = 3 })",
    );
    assert!(!global::<bool>(&config, "ok"));
    assert_eq!(plugin_error(&config).plugin.as_deref(), Some("sidebar"));
    assert!(plugin_error(&config).message.contains("width"));
    let count: i64 = eval(&config, "return #gband.bar.list()");
    assert_eq!(count, 0);
}

#[test]
fn wrong_side() {
    for (opts, mentions) in [
        ("{ side = 'top' }", "side"),
        ("{ order = 'first' }", "order"),
        ("'left'", "sidebar"),
    ] {
        let (_scratch, config) = loaded(
            "wrong-side",
            &format!("ok = gband.plugin('gband.sidebar', {opts})"),
        );
        assert!(!global::<bool>(&config, "ok"), "{opts}");
        assert!(plugin_error(&config).message.contains(mentions), "{opts}");
        let count: i64 = eval(&config, "return #gband.bar.list()");
        assert_eq!(count, 0, "{opts}");
    }
}

#[test]
fn defaults_without_a_colorscheme() {
    let scratch = Scratch::new("defaults");
    scratch.user_file("colors/plain.lua", "");
    scratch.write("gband.plugin('gband.sidebar')\ngband.colorscheme('plain')");
    let config = scratch.loaded();
    assert_eq!(resolved(&config, "SidebarMode"), ["bold=true"]);
    assert_eq!(resolved(&config, "SidebarBand"), ["dim=true"]);
    assert_eq!(
        resolved(&config, "SidebarBandActive"),
        ["dim=true", "underline=true"]
    );
    assert_eq!(resolved(&config, "SidebarError"), ["bold=true", "fg=1"]);
}

#[test]
fn rows_follow_the_pushed_state() {
    let (_scratch, config) = loaded("rows", "gband.plugin('gband.sidebar')");
    let mut state = drawn(80);
    state.band.count = 3;
    state.band.index = 2;
    let bar = presented(&config, state.clone());
    assert_eq!(
        shown_rows(&bar),
        [
            (0, "I".to_owned()),
            (2, "1".to_owned()),
            (3, "2".to_owned()),
            (4, "3".to_owned()),
            (12, "─".to_owned())
        ]
    );
    state.error = Some("broken".to_owned());
    let bar = presented(&config, state);
    assert_eq!(shown_rows(&bar).last(), Some(&(23, "!".to_owned())));
    assert!(config.runtime.error_marker_shown());
}

#[test]
fn band_labels_past_nine() {
    let (_scratch, config) = loaded("labels", "gband.plugin('gband.sidebar')");
    let mut state = sized(80, 80);
    state.band.count = 36;
    let bar = presented(&config, state);
    let labels: String = rows(&bar)[2..38].concat();
    assert_eq!(labels, "123456789abcdefghijklmnopqrstuvwxyz");
}

#[test]
fn bands_cut_at_the_divider() {
    let (_scratch, config) = loaded("labels-cut", "gband.plugin('gband.sidebar')");
    let mut state = drawn(80);
    state.band.count = 12;
    let bar = presented(&config, state);
    let labels: String = rows(&bar)[2..12].concat();
    assert_eq!(labels, "123456789a");
    assert_eq!(rows(&bar)[12], "─");
    assert!(rows(&bar)[13..].iter().all(String::is_empty));
}

#[test]
fn no_divider_on_a_short_sidebar() {
    let (_scratch, config) = loaded("short", "gband.plugin('gband.sidebar')");
    let mut state = sized(80, 5);
    state.band.count = 2;
    let bar = presented(&config, state);
    assert_eq!(rows(&bar), ["I", "", "1", "2", ""]);
}

#[test]
fn one_row() {
    let (_scratch, config) = loaded("one-row", "gband.plugin('gband.sidebar')");
    let mut state = sized(80, 1);
    let bar = presented(&config, state.clone());
    assert_eq!(rows(&bar), ["I"]);
    state.error = Some("broken".to_owned());
    let bar = presented(&config, state);
    assert_eq!(rows(&bar), ["!"]);
}

#[test]
fn multi_mode_letter() {
    let (_scratch, config) = loaded("multi", "gband.plugin('gband.sidebar')");
    let mut state = drawn(80);
    state.multi = true;
    assert_eq!(rows(&presented(&config, state.clone()))[0], "M");
    state.table = "prefix".to_owned();
    assert_ne!(rows(&presented(&config, state.clone()))[0], "M");
    state.table = "root".to_owned();
    state.multi = false;
    assert_eq!(rows(&presented(&config, state))[0], "I");
}

#[test]
fn multi_mode_with_the_floating_style() {
    let (_scratch, config) = floating("apps-multi");
    let mut state = drawn(80);
    state.multi = true;
    assert_eq!(rows(&presented(&config, state))[0], "∷");
}

fn floating(name: &str) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    scratch.user_file("keystyle.lua", "return \"floating\"\n");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    (scratch, config)
}

fn with_ribbon(mut state: ViewState) -> ViewState {
    state.ribbon = Size::new(state.width - 1, state.height);
    state
}

fn press_row_zero(config: &Config) -> gband_lua::Outcome {
    config.runtime.emit(&Event::MousePressed {
        button: MouseButton::Left,
        pointer: Pointer {
            col: 0,
            row: 0,
            modifiers: Modifiers::NONE,
            target: PointerTarget::Outside,
            window: None,
            plugin_window: None,
            content: None,
            boxed: None,
            table: "root".to_owned(),
        },
    })
}

fn plugin_windows(config: &Config) -> i64 {
    eval(config, "return #gband.win.list()")
}

#[test]
fn apps_character_with_the_floating_style() {
    let (_scratch, config) = floating("apps-floating");
    let mut state = drawn(80);
    state.band.count = 2;
    let bar = presented(&config, state);
    assert_eq!(
        shown_rows(&bar),
        [
            (0, "∷".to_owned()),
            (2, "1".to_owned()),
            (3, "2".to_owned()),
            (12, "─".to_owned())
        ]
    );
}

#[test]
fn apps_character_after_the_leader() {
    let (_scratch, config) = floating("apps-leader");
    let mut state = drawn(80);
    state.table = "prefix".to_owned();
    let bar = presented(&config, state);
    assert_eq!(rows(&bar)[0], "∷");
}

#[test]
fn open_the_window_list_from_the_sidebar() {
    let (_scratch, config) = floating("apps-click");
    presented(&config, with_ribbon(drawn(80)));
    clean(&press_row_zero(&config));
    assert_eq!(plugin_windows(&config), 1);
    let focused: Option<u32> = eval(&config, "return gband.view().plugin_window");
    assert!(focused.is_some());
}

#[test]
fn apps_character_without_the_desktop_plugin() {
    let (_scratch, config) = loaded(
        "apps-no-desktop",
        "gband.keystyle.use('floating')\ngband.plugin('gband.sidebar')\ngband.action['desktop.list'] = nil",
    );
    let bar = presented(&config, with_ribbon(drawn(80)));
    assert_eq!(rows(&bar)[0], "∷");
    clean(&press_row_zero(&config));
    assert_eq!(plugin_windows(&config), 0);
}

#[test]
fn floating_preset_required_by_name() {
    let (_scratch, config) = loaded(
        "apps-required",
        "require('gband.keystyle.floating')\ngband.plugin('gband.sidebar')",
    );
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    let bar = presented(&config, drawn(80));
    assert_eq!(rows(&bar)[0], "I");
}

#[test]
fn click_the_mode_letters_row_with_the_modal_style() {
    let (_scratch, config) = loaded(
        "mode-row-modal",
        "gband.keystyle.use('modal')\ngband.plugin('gband.sidebar')",
    );
    presented(&config, with_ribbon(drawn(80)));
    let outcome = press_row_zero(&config);
    clean(&outcome);
    assert!(outcome.dispatched.is_empty(), "{:?}", outcome.dispatched);
    assert_eq!(plugin_windows(&config), 0);
}

const GROUPS: [&str; 13] = [
    "SidebarBand",
    "SidebarDivider",
    "SidebarBandActive",
    "SidebarAgentBlocked",
    "SidebarAgentBlockedActive",
    "SidebarAgentDone",
    "SidebarAgentDoneActive",
    "SidebarAgentWorking",
    "SidebarAgentWorkingActive",
    "SidebarAgentIdle",
    "SidebarAgentIdleActive",
    "SidebarAgentUnknown",
    "SidebarAgentUnknownActive",
];

const FIRST_GROUP_COLOR: u8 = 20;

fn marked() -> String {
    let marks: String = GROUPS
        .iter()
        .zip(FIRST_GROUP_COLOR..)
        .map(|(group, color)| format!("gband.hl.set('{group}', {{ fg = {color} }})\n"))
        .collect();
    format!("gband.plugin('gband.sidebar')\n{marks}")
}

fn group_at(bar: &Bar, row: usize) -> &'static str {
    let [run] = bar.lines[row].as_slice() else {
        panic!("row {row}: {:?}", bar.lines[row]);
    };
    match run.style.fg {
        Some(Color::Index(color)) if color >= FIRST_GROUP_COLOR => {
            GROUPS[usize::from(color - FIRST_GROUP_COLOR)]
        }
        other => panic!("row {row}: {other:?}"),
    }
}

fn opened(layout: &mut Layout, band: usize, after: Option<WindowId>) -> WindowId {
    let window = layout.allocate_window();
    let id = layout.bands()[band].id;
    layout.open(window, id, after, None, &LayoutOptions::default());
    window
}

fn agent_state(state: &str) -> Value {
    Value::Table(vec![
        (Key::string("agent"), Value::string("claude")),
        (Key::string("state"), Value::string(state)),
    ])
}

fn agents_view(
    layout: Layout,
    agents: &[(WindowId, &str)],
    focused: Option<WindowId>,
    band: u32,
    height: u16,
) -> ViewState {
    let states: WindowStates = agents
        .iter()
        .map(|&(window, state)| {
            (
                window,
                BTreeMap::from([("agent_state".to_owned(), agent_state(state))]),
            )
        })
        .collect();
    let count = layout.bands().len() as u32;
    ViewState {
        band: BandState {
            number: band,
            index: band,
            count,
        },
        window: focused.map(|window| window.0),
        layout: Arc::new(layout),
        area: Size::new(80, height),
        ribbon: Size::new(79, height),
        states: Arc::new(states),
        ..sized(80, height)
    }
}

fn one_band(windows: usize) -> (Layout, Vec<WindowId>) {
    let mut layout = Layout::new();
    let mut opened_windows: Vec<WindowId> = Vec::new();
    for _ in 0..windows {
        let after = opened_windows.last().copied();
        opened_windows.push(opened(&mut layout, 0, after));
    }
    (layout, opened_windows)
}

fn changed(config: &Config, window: WindowId, previous: &str, state: &str) -> Outcome {
    config.runtime.emit(&Event::WindowStateChanged {
        window,
        key: "agent_state".to_owned(),
        value: Some(agent_state(state)),
        previous: Some(agent_state(previous)),
    })
}

fn press(config: &Config, row: u16, table: &str) -> Outcome {
    config.runtime.emit(&Event::MousePressed {
        button: MouseButton::Left,
        pointer: Pointer {
            col: 0,
            row,
            modifiers: Modifiers::NONE,
            target: PointerTarget::Outside,
            window: None,
            plugin_window: None,
            content: None,
            boxed: None,
            table: table.to_owned(),
        },
    })
}

#[test]
fn rows_with_agents() {
    let (_scratch, config) = loaded("agent-rows", "gband.plugin('gband.sidebar')");
    let (layout, windows) = one_band(2);
    let agents = [(windows[0], "idle"), (windows[1], "working")];
    let bar = presented(&config, agents_view(layout, &agents, None, 1, 24));
    assert_eq!(
        shown_rows(&bar),
        [
            (0, "I".to_owned()),
            (2, "1".to_owned()),
            (3, "2".to_owned()),
            (12, "─".to_owned()),
            (13, "1".to_owned()),
            (14, "2".to_owned()),
        ]
    );
}

#[test]
fn divider_in_its_group() {
    let (_scratch, config) = loaded("agent-divider", &marked());
    let (layout, windows) = one_band(1);
    let bar = presented(
        &config,
        agents_view(layout, &[(windows[0], "idle")], None, 1, 24),
    );
    assert_eq!(group_at(&bar, 12), "SidebarDivider");
}

#[test]
fn bands_and_agents_cut_at_the_divider() {
    let (_scratch, config) = loaded("agent-bands-cut", "gband.plugin('gband.sidebar')");
    let mut layout = Layout::new();
    let agent = opened(&mut layout, 0, None);
    for band in 1..4 {
        opened(&mut layout, band, None);
    }
    assert_eq!(layout.bands().len(), 5);
    let bar = presented(
        &config,
        agents_view(layout, &[(agent, "working")], None, 1, 8),
    );
    assert_eq!(
        shown_rows(&bar),
        [
            (0, "I".to_owned()),
            (2, "1".to_owned()),
            (3, "2".to_owned()),
            (4, "─".to_owned()),
            (5, "1".to_owned()),
        ]
    );
}

#[test]
fn no_room_for_agents() {
    let (_scratch, config) = loaded("agent-no-room", "gband.plugin('gband.sidebar')");
    let (layout, windows) = one_band(1);
    let bar = presented(
        &config,
        agents_view(layout, &[(windows[0], "working")], None, 1, 5),
    );
    assert_eq!(rows(&bar), ["I", "", "1", "2", ""]);
}

#[test]
fn agents_cut_at_the_last_row() {
    let (_scratch, config) = loaded("agent-cut", "gband.plugin('gband.sidebar')");
    let (layout, windows) = one_band(4);
    let four: Vec<(WindowId, &str)> = windows.iter().map(|&w| (w, "idle")).collect();
    let bar = presented(&config, agents_view(layout, &four, None, 1, 10));
    let agent_rows: Vec<(usize, String)> = shown_rows(&bar).into_iter().skip(4).collect();
    let expected: Vec<(usize, String)> = (6..9).map(|row| (row, (row - 5).to_string())).collect();
    assert_eq!(agent_rows, expected);
    assert_eq!(rows(&bar)[9], "");
}

#[test]
fn band_of_a_working_agent() {
    let (_scratch, config) = loaded("agent-band", &marked());
    let mut layout = Layout::new();
    opened(&mut layout, 0, None);
    let agent = opened(&mut layout, 1, None);
    let bar = presented(
        &config,
        agents_view(layout, &[(agent, "working")], None, 1, 24),
    );
    assert_eq!(group_at(&bar, 2), "SidebarBandActive");
    assert_eq!(group_at(&bar, 3), "SidebarAgentWorking");
    assert_eq!(group_at(&bar, 4), "SidebarBand");
}

#[test]
fn most_urgent_agent_wins() {
    let (_scratch, config) = loaded("agent-urgent", &marked());
    let (layout, windows) = one_band(3);
    let agents = [
        (windows[0], "idle"),
        (windows[1], "blocked"),
        (windows[2], "working"),
    ];
    let bar = presented(&config, agents_view(layout, &agents, None, 1, 24));
    assert_eq!(group_at(&bar, 2), "SidebarAgentBlockedActive");
}

#[test]
fn band_follows_its_agent() {
    let (_scratch, config) = loaded("agent-band-follows", &marked());
    let mut layout = Layout::new();
    let plain = opened(&mut layout, 0, None);
    let agent = opened(&mut layout, 1, None);
    let view = |state| agents_view(layout.clone(), &[(agent, state)], Some(plain), 1, 24);
    presented(&config, view("working"));
    clean(&config.runtime.set_state(view("done")));
    clean(&changed(&config, agent, "working", "done"));
    let bar = sidebar(&config).expect("the sidebar is redrawn");
    assert_eq!(group_at(&bar, 3), "SidebarAgentDone");
    assert_eq!(group_at(&bar, 13), "SidebarAgentDone");
}

#[test]
fn blocked_agent() {
    let (_scratch, config) = loaded("agent-blocked", &marked());
    let (layout, windows) = one_band(2);
    let agents = [(windows[1], "blocked")];
    let bar = presented(
        &config,
        agents_view(layout, &agents, Some(windows[0]), 1, 24),
    );
    assert_eq!(rows(&bar)[13], "1");
    assert_eq!(group_at(&bar, 13), "SidebarAgentBlocked");
}

#[test]
fn focused_agent_takes_its_active_group() {
    let (_scratch, config) = loaded("agent-focused", &marked());
    let (layout, windows) = one_band(2);
    let agents = [(windows[0], "idle"), (windows[1], "working")];
    let bar = presented(
        &config,
        agents_view(layout, &agents, Some(windows[1]), 1, 24),
    );
    assert_eq!(group_at(&bar, 13), "SidebarAgentIdle");
    assert_eq!(group_at(&bar, 14), "SidebarAgentWorkingActive");
}

#[test]
fn every_state_has_its_group() {
    let (_scratch, config) = loaded("agent-states", &marked());
    let (layout, windows) = one_band(5);
    let states = ["blocked", "done", "working", "idle", "unknown"];
    let agents: Vec<(WindowId, &str)> = windows.iter().copied().zip(states).collect();
    let bar = presented(&config, agents_view(layout, &agents, None, 1, 24));
    let groups: Vec<&str> = (13..18).map(|row| group_at(&bar, row)).collect();
    assert_eq!(
        groups,
        [
            "SidebarAgentBlocked",
            "SidebarAgentDone",
            "SidebarAgentWorking",
            "SidebarAgentIdle",
            "SidebarAgentUnknown"
        ]
    );
}

#[test]
fn state_change_recolors() {
    let (_scratch, config) = loaded("agent-recolor", &marked());
    let (layout, windows) = one_band(2);
    let view = |state| {
        agents_view(
            layout.clone(),
            &[(windows[1], state)],
            Some(windows[0]),
            1,
            24,
        )
    };
    presented(&config, view("working"));
    clean(&config.runtime.set_state(view("blocked")));
    clean(&changed(&config, windows[1], "working", "blocked"));
    let bar = sidebar(&config).expect("the sidebar is redrawn");
    assert_eq!(rows(&bar)[13], "1");
    assert_eq!(group_at(&bar, 13), "SidebarAgentBlocked");
}

#[test]
fn state_colors_follow_the_palette() {
    let (_scratch, config) = loaded("agent-palette", "gband.plugin('gband.sidebar')");
    let (layout, windows) = one_band(3);
    let agents = [(windows[1], "blocked"), (windows[2], "blocked")];
    let bar = presented(
        &config,
        agents_view(layout, &agents, Some(windows[2]), 1, 24),
    );
    let style = |row: usize| bar.lines[row][0].style;
    assert_eq!(style(13).fg, Some(Color::Index(1)));
    assert!(!style(13).bold);
    assert!(!style(13).underline);
    assert_eq!(style(14).fg, Some(Color::Index(1)));
    assert!(!style(14).bold);
    assert!(style(14).underline);
}

#[test]
fn viewed_agent_band_underlined_in_its_state_colour() {
    let (_scratch, config) = loaded("agent-band-underline", "gband.plugin('gband.sidebar')");
    let (layout, windows) = one_band(1);
    let bar = presented(
        &config,
        agents_view(layout, &[(windows[0], "working")], None, 1, 24),
    );
    let style = bar.lines[2][0].style;
    assert_eq!(style.fg, Some(Color::Index(3)));
    assert!(!style.bold);
    assert!(style.underline);
}

#[test]
fn agent_group_defaults() {
    let scratch = Scratch::new("agent-defaults");
    scratch.user_file("colors/plain.lua", "");
    scratch.write("gband.plugin('gband.sidebar')\ngband.colorscheme('plain')");
    let config = scratch.loaded();
    assert_eq!(resolved(&config, "SidebarAgentWorking"), ["fg=yellow"]);
    assert_eq!(
        resolved(&config, "SidebarAgentWorkingActive"),
        ["fg=yellow", "underline=true"]
    );
    assert_eq!(resolved(&config, "SidebarAgentBlocked"), ["fg=red"]);
    assert_eq!(resolved(&config, "SidebarAgentDone"), ["fg=green"]);
    assert_eq!(resolved(&config, "SidebarAgentIdle"), ["fg=blue"]);
    assert_eq!(resolved(&config, "SidebarAgentUnknown"), ["dim=true"]);
    assert_eq!(
        resolved(&config, "SidebarAgentUnknownActive"),
        ["dim=true", "underline=true"]
    );
    assert_eq!(resolved(&config, "SidebarDivider"), ["dim=true"]);
}

#[test]
fn active_defaults_add_an_underline() {
    let scratch = Scratch::new("agent-active-defaults");
    scratch.user_file("colors/plain.lua", "");
    scratch.write("gband.plugin('gband.sidebar')\ngband.colorscheme('plain')");
    let config = scratch.loaded();
    for (group, expected) in [
        ("SidebarBandActive", ["dim=true", "underline=true"]),
        ("SidebarAgentUnknownActive", ["dim=true", "underline=true"]),
        ("SidebarAgentWorkingActive", ["fg=yellow", "underline=true"]),
        ("SidebarAgentBlockedActive", ["fg=red", "underline=true"]),
        ("SidebarAgentDoneActive", ["fg=green", "underline=true"]),
        ("SidebarAgentIdleActive", ["fg=blue", "underline=true"]),
    ] {
        assert_eq!(resolved(&config, group), expected, "{group}");
    }
}

fn three_bands_with_agents() -> (Layout, WindowId, WindowId) {
    let mut layout = Layout::new();
    let first = opened(&mut layout, 0, None);
    let second = opened(&mut layout, 1, None);
    opened(&mut layout, 2, None);
    let fourth = opened(&mut layout, 1, Some(second));
    assert_eq!(layout.bands().len(), 4);
    (layout, first, fourth)
}

#[test]
fn click_an_agent_in_another_band() {
    let (_scratch, config) = loaded("agent-click", "gband.plugin('gband.sidebar')");
    let (layout, first, fourth) = three_bands_with_agents();
    let agents = [(first, "idle"), (fourth, "working")];
    presented(&config, agents_view(layout, &agents, Some(first), 1, 24));
    let outcome = press(&config, 14, "root");
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusWindow(
            fourth
        )))]
    );
}

#[test]
fn click_in_navigation_mode() {
    let (_scratch, config) = loaded(
        "agent-click-navigation",
        "gband.keystyle.use('modal')\ngband.plugin('gband.sidebar')",
    );
    let (layout, first, fourth) = three_bands_with_agents();
    let agents = [(first, "idle"), (fourth, "working")];
    let mut view = agents_view(layout, &agents, Some(fourth), 1, 24);
    view.table = "prefix".to_owned();
    presented(&config, view);
    let outcome = press(&config, 13, "prefix");
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusWindow(
            first
        )))]
    );
}

#[test]
fn click_beside_the_agent_labels() {
    let (_scratch, config) = loaded("agent-click-beside", "gband.plugin('gband.sidebar')");
    let (layout, first, fourth) = three_bands_with_agents();
    let agents = [(first, "idle"), (fourth, "working")];
    presented(&config, agents_view(layout, &agents, Some(first), 1, 24));
    for row in [1, 6, 12, 15, 22] {
        let outcome = press(&config, row, "root");
        clean(&outcome);
        assert!(
            outcome.dispatched.is_empty(),
            "row {row}: {:?}",
            outcome.dispatched
        );
    }
}
