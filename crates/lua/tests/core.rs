mod common;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use common::*;
use gband_core::action::Action;
use gband_core::layout::WindowId;
use gband_core::view::ViewAction;
use gband_lua::{BandState, DecorationInfo, Dispatch, ViewState};

const WIN: &str = include_str!("../src/runtime/gband/win.lua");
const BAR: &str = include_str!("../src/runtime/gband/bar.lua");

fn load_error(source: &str) -> (std::path::PathBuf, gband_lua::ConfigError) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let scratch = Scratch::new(&format!(
        "core-error-{}",
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let path = scratch.write(source);
    let error = match scratch.load() {
        Ok(_) => panic!("the configuration loaded: {source}"),
        Err(error) => error,
    };
    (path, error)
}

fn rejected(line: &str, mentions: &[&str]) {
    let (path, error) = load_error(&format!("local a = 1\n{line}"));
    assert_eq!(error.location, Some((path, 2)), "{line}: {error}");
    for mention in mentions {
        assert!(error.message.contains(mention), "{line}: {error}");
    }
}

#[test]
fn every_primitive_is_public() {
    let scratch = Scratch::new("core-public");
    scratch.write(
        "kinds = { type(gband.core.owner), type(gband.core.provide), \
         type(gband.core.present_window) }",
    );
    let config = scratch.loaded();
    let kinds: Vec<String> = global(&config, "kinds");
    assert_eq!(kinds, ["function", "function", "function"]);
}

#[test]
fn owner_inside_a_plugin() {
    let scratch = Scratch::new("core-owner");
    scratch.plugin_file(
        "hello",
        "lua/hello/init.lua",
        "return { setup = function() owner = gband.core.owner() end }",
    );
    scratch.write("gband.plugin('hello')\noutside = gband.core.owner()");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "owner"), "hello");
    assert_eq!(global::<Option<String>>(&config, "outside"), None);
}

#[test]
fn isolated_call() {
    let scratch = Scratch::new("core-call");
    scratch.write(JOB);
    let config = scratch.load_with_budget(100_000).unwrap();
    let outcome = run_job(
        &config,
        "ok, message = gband.core.call(nil, nil, function() while true do end end)\n\
         gband.action.focus_column_left()",
    );
    assert!(!global::<bool>(&config, "ok"));
    assert!(global::<String>(&config, "message").contains("instruction limit"));
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusLeft))]
    );
}

#[test]
fn call_returns_the_values() {
    let scratch = Scratch::new("core-call-values");
    scratch.write(
        "ok, a, b = gband.core.call('hello', nil, function(x) return x, gband.core.owner() end, 7)",
    );
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "ok"));
    assert_eq!(global::<i64>(&config, "a"), 7);
    assert_eq!(global::<String>(&config, "b"), "hello");
}

#[test]
fn wrong_argument() {
    let (path, error) =
        load_error("local a = 1\nlocal b = 2\nlocal c = 3\ngband.core.timer('soon', print)");
    assert_error_at(&error, &path, 4, "gband.core.timer");
}

#[test]
fn timer() {
    let scratch = Scratch::new("core-timer");
    scratch.write(JOB);
    let config = scratch.loaded();
    clean(&run_job(
        &config,
        "count = 0\n\
         id = gband.core.timer(100, function()\n\
           count = count + 1\n\
           if count == 2 then gband.core.cancel(id) end\n\
         end)",
    ));
    let start = Instant::now();
    for step in 1..=5 {
        clean(
            &config
                .runtime
                .fire_timers(start + Duration::from_millis(100 * step)),
        );
    }
    assert_eq!(global::<i64>(&config, "count"), 2);
    assert_eq!(config.runtime.next_timer(), None);
}

fn band(index: u32, count: u32) -> ViewState {
    ViewState {
        band: BandState {
            number: index,
            index,
            count,
        },
        ..drawn(80)
    }
}

#[test]
fn several_state_functions() {
    let scratch = Scratch::new("core-on-state");
    for name in ["one", "two"] {
        scratch.client_plugin(
            name,
            &format!("gband.core.on_state(function() seen[#seen + 1] = '{name}' end)"),
        );
    }
    scratch.write("seen = {}");
    let config = scratch.loaded();
    clean(&config.runtime.set_state(band(1, 2)));
    config.runtime.lua().load("seen = {}").exec().unwrap();
    clean(&config.runtime.set_state(band(2, 2)));
    let seen: Vec<String> = global(&config, "seen");
    assert_eq!(seen, ["one", "two"]);
}

#[test]
fn state_follows_multi_mode() {
    let scratch = Scratch::new("core-multi");
    scratch.client_plugin(
        "watch",
        "gband.core.on_state(function() seen = gband.core.state().multi end)",
    );
    let config = scratch.loaded();
    clean(&config.runtime.set_state(band(1, 2)));
    config.runtime.lua().load("seen = nil").exec().unwrap();
    let mut state = band(1, 2);
    state.multi = true;
    clean(&config.runtime.set_state(state));
    let seen: Option<bool> = global(&config, "seen");
    assert_eq!(seen, Some(true));
}

#[test]
fn argument_errors_name_the_primitive_and_the_field() {
    rejected(
        "gband.core.present_window(1, { kind = 'tiled', cols = 'wide', rows = 1, base = {}, lines = {} })",
        &["gband.core.present_window", "`cols`"],
    );
    rejected(
        "gband.core.present_window(1, { kind = 'tiled', cols = 1, rows = 1, base = {}, lines = { { { text = 1, style = {} } } } })",
        &["gband.core.present_window", "`text`"],
    );
    rejected(
        "gband.core.present_bars({ { id = 'x', side = 'up', size = 1, order = 0, seq = 1, base = {}, lines = {} } })",
        &["gband.core.present_bars", "`side`"],
    );
    rejected(
        "gband.core.place_bars({ { side = 'left', size = 'big', order = 0, seq = 1 } }, 80)",
        &["gband.core.place_bars", "`size`"],
    );
    rejected(
        "gband.core.request({ id = 1, op = 'move' })",
        &["gband.core.request", "`op`"],
    );
    rejected("gband.core.parse_key(5)", &["gband.core.parse_key"]);
    rejected("gband.core.border('round')", &["gband.core.border"]);
    rejected(
        "gband.core.error_marker('yes')",
        &["gband.core.error_marker"],
    );
    rejected("gband.core.removed()", &["gband.core.removed"]);
    rejected(
        "gband.core.forget_window('one')",
        &["gband.core.forget_window"],
    );
    rejected(
        "gband.core.focus_window('one')",
        &["gband.core.focus_window"],
    );
    rejected(
        "gband.core.palette.set({ fg = 'red' })",
        &["gband.core.palette.set", "`fg`"],
    );
    rejected(
        "gband.core.emit('Nothing', {})",
        &["gband.core.emit", "Nothing"],
    );
    rejected("gband.core.call(nil, nil, 3)", &["gband.core.call"]);
    rejected("gband.core.report(nil, 3)", &["gband.core.report"]);
    rejected("gband.core.on_state('draw')", &["gband.core.on_state"]);
    rejected(
        "gband.core.reopen_settings('top')",
        &["gband.core.reopen_settings"],
    );
}

#[test]
fn unknown_kind() {
    rejected(
        "gband.core.provide('menus', {})",
        &["gband.core.provide", "menus", "decorations"],
    );
    rejected(
        "gband.core.provide('decorations', {})",
        &["gband.core.provide", "function"],
    );
    rejected(
        "gband.core.provide('styles', {})",
        &["gband.core.provide", "function"],
    );
    rejected(
        "gband.core.provide('bars', print)",
        &["gband.core.provide", "table"],
    );
}

#[test]
fn no_provider_leaves_the_feature_absent() {
    let scratch = Scratch::new("core-no-provider");
    scratch.user_file("lua/gband/prelude.lua", "");
    scratch.write(JOB);
    let config = scratch.loaded();
    clean(&config.runtime.set_state(drawn(80)));
    clean(&config.runtime.refresh_plugins());
    assert_eq!(config.runtime.take_bars(), None);
    assert!(config.runtime.take_frames().is_empty());
    assert_eq!(config.runtime.take_client_styles(), None);
    let info = DecorationInfo {
        window: WindowId(1),
        floating: false,
        focused: true,
        width: 40,
    };
    assert_eq!(config.runtime.decorations(&info), Ok(Vec::new()));
    clean(&config.runtime.open_settings(1));
    let (closed, outcome) = config.runtime.close_focused_plugin_window();
    clean(&outcome);
    assert!(!closed);
}

#[test]
fn defaults_through_the_prelude() {
    let scratch = Scratch::new("core-prelude");
    scratch.write(
        "kinds = {}\n\
         for _, name in ipairs({ 'win', 'bar', 'hl', 'colorscheme', 'palette', 'settings', 'keystyle' }) do\n\
           kinds[#kinds + 1] = type(gband[name])\n\
         end",
    );
    let config = scratch.loaded();
    let kinds: Vec<String> = global(&config, "kinds");
    assert_eq!(
        kinds,
        [
            "table", "table", "table", "function", "table", "table", "table"
        ]
    );
}

#[test]
fn api_module_replaced() {
    let scratch = Scratch::new("core-replaced-win");
    let copy = WIN.replacen(
        "function api.open(opts)\n",
        "function api.open(opts)\n  gband.notify(\"opened\")\n",
        1,
    );
    assert_ne!(copy, WIN);
    scratch.user_file("lua/gband/win.lua", &copy);
    scratch.write(JOB);
    let config = scratch.loaded();
    clean(&config.runtime.set_state(drawn(80)));
    let outcome = run_job(&config, "gband.win.open({ lines = { 'hi' } })");
    clean(&outcome);
    assert!(
        outcome.dispatched.iter().any(|dispatch| matches!(
            dispatch,
            Dispatch::Write(bytes) if bytes.windows(6).any(|window| window == b"opened")
        )),
        "{:?}",
        outcome.dispatched
    );
    assert_eq!(config.runtime.take_frames().len(), 1);
}

#[test]
fn broken_prelude_override() {
    let scratch = Scratch::new("core-broken-prelude");
    scratch.write(JOB);
    scratch.loaded();
    let path = scratch.user_file("lua/gband/prelude.lua", "error('boom')");
    let error = match scratch.load() {
        Ok(_) => panic!("the configuration loaded"),
        Err(error) => error,
    };
    assert_error_at(&error, &path, 1, "boom");
}

#[test]
fn replaced_bar_implementation() {
    let scratch = Scratch::new("core-replaced-bar");
    let copy = BAR.replacen(
        "function hooks.flush()\n",
        "function hooks.flush()\n  flushes = (flushes or 0) + 1\n",
        1,
    );
    assert_ne!(copy, BAR);
    scratch.user_file("lua/gband/bar.lua", &copy);
    let config = scratch.loaded();
    let bar = presented(&config, drawn(80));
    let bundled = Scratch::new("core-bundled-bar");
    let expected = presented(&bundled.loaded(), drawn(80));
    assert_eq!(bar, expected);
    let before: i64 = global(&config, "flushes");
    clean(&config.runtime.refresh_plugins());
    assert!(global::<i64>(&config, "flushes") > before);
}
