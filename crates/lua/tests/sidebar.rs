mod common;

use common::*;
use gband_core::geometry::Size;
use gband_core::input::{Modifiers, MouseButton};
use gband_lua::{BarSide, Config, Event, Pointer, PointerTarget, ViewState};

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
    assert_eq!(resolved(&config, "SidebarBandActive"), ["bold=true"]);
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
            (4, "3".to_owned())
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
    let mut state = sized(80, 40);
    state.band.count = 36;
    let bar = presented(&config, state);
    let labels: String = rows(&bar)[2..38].concat();
    assert_eq!(labels, "123456789abcdefghijklmnopqrstuvwxyz");
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
            (0, "⊞".to_owned()),
            (2, "1".to_owned()),
            (3, "2".to_owned())
        ]
    );
}

#[test]
fn apps_character_after_the_leader() {
    let (_scratch, config) = floating("apps-leader");
    let mut state = drawn(80);
    state.table = "prefix".to_owned();
    let bar = presented(&config, state);
    assert_eq!(rows(&bar)[0], "⊞");
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
    assert_eq!(rows(&bar)[0], "⊞");
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
