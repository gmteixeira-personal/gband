mod common;

use common::*;
use gband_core::layout::WindowId;
use gband_lua::decorations::StylePatch;
use gband_lua::{Color, Config, ConfigError, DecorationInfo, DecorationSpan};

const INFO: DecorationInfo = DecorationInfo {
    window: WindowId(3),
    floating: true,
    focused: false,
    width: 40,
};

fn provider(name: &str, function: &str) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    scratch.write(&format!(
        "{JOB}gband.core.provide('decorations', {function})"
    ));
    let config = scratch.load_with_budget(100_000).unwrap();
    (scratch, config)
}

fn spans(name: &str, function: &str) -> Vec<DecorationSpan> {
    let (_scratch, config) = provider(name, function);
    config
        .runtime
        .decorations(&INFO)
        .unwrap_or_else(|error| panic!("{error}"))
}

fn failure(name: &str, function: &str) -> ConfigError {
    let (_scratch, config) = provider(name, function);
    config.runtime.decorations(&INFO).unwrap_err()
}

#[test]
fn info_table() {
    let (_scratch, config) = provider(
        "info",
        "function(info) seen = { info.window, info.floating, info.focused, info.width } end",
    );
    config.runtime.decorations(&INFO).unwrap();
    let window: i64 = eval(&config, "return seen[1]");
    let floating: bool = eval(&config, "return seen[2]");
    let focused: bool = eval(&config, "return seen[3]");
    let width: i64 = eval(&config, "return seen[4]");
    assert_eq!((window, floating, focused, width), (3, true, false, 40));
}

#[test]
fn string_and_table_spans() {
    assert_eq!(
        spans("spans", "function() return { '[_]', { text = '[X]' } } end"),
        [DecorationSpan::plain("[_]"), DecorationSpan::plain("[X]")]
    );
}

#[test]
fn style_patch() {
    let read = spans(
        "patch",
        "function() return { { text = 'a', style = { fg = 1 } }, \
         { text = 'b', style = { bg = '#102030', bold = false, dim = true } } } end",
    );
    assert_eq!(
        read[0].style,
        StylePatch {
            fg: Some(Color::Index(1)),
            ..StylePatch::default()
        }
    );
    assert_eq!(
        read[1].style,
        StylePatch {
            bg: Some(Color::Rgb(0x10, 0x20, 0x30)),
            bold: Some(false),
            dim: Some(true),
            ..StylePatch::default()
        }
    );
}

#[test]
fn control_characters_removed() {
    assert_eq!(
        spans("controls", "function() return { 'a\\tb' } end"),
        [DecorationSpan::plain("ab")]
    );
}

#[test]
fn nil_and_empty_give_no_spans() {
    assert!(spans("nil", "function() return nil end").is_empty());
    assert!(spans("empty", "function() return {} end").is_empty());
}

#[test]
fn wrong_span() {
    let error = failure("wrong-span", "function() return { '[_]', 7 } end");
    assert!(error.to_string().starts_with("decorations: "), "{error}");
    assert!(error.message.contains("span 2"), "{error}");
}

#[test]
fn wrong_return_value() {
    let error = failure("wrong-value", "function() return 5 end");
    assert!(error.to_string().starts_with("decorations: "), "{error}");
}

#[test]
fn error_names_the_line() {
    let scratch = Scratch::new("error-line");
    let path = scratch.write(
        "gband.core.provide('decorations', function()\n\
         local a = 1\n\
         error('boom')\n\
         end)",
    );
    let config = scratch.loaded();
    let error = config.runtime.decorations(&INFO).unwrap_err();
    assert_eq!(
        error.to_string(),
        format!("decorations: {}:3: boom", path.display())
    );
}

#[test]
fn endless_loop_marks_no_plugin_failed() {
    let scratch = Scratch::new("endless");
    scratch.client_plugin(
        "buttons",
        "gband.core.provide('decorations', function() while true do end end)",
    );
    let config = scratch.load_with_budget(100_000).unwrap();
    let error = config.runtime.decorations(&INFO).unwrap_err();
    assert_eq!(error.plugin.as_deref(), Some("decorations"));
    assert!(error.message.contains("instruction limit"), "{error}");
    assert!(!eval::<bool>(
        &config,
        "return gband.core.failed('buttons')"
    ));
}

#[test]
fn not_a_callback() {
    let (_scratch, config) = provider(
        "dispatching",
        "function() dispatching = gband.core.dispatching() end",
    );
    config.runtime.decorations(&INFO).unwrap();
    assert!(!global::<bool>(&config, "dispatching"));
}

#[test]
fn action_fails() {
    let (_scratch, config) = provider("action", "function() gband.action.focus_column_left() end");
    let error = config.runtime.decorations(&INFO).unwrap_err();
    assert!(error.to_string().starts_with("decorations: "), "{error}");
}

#[test]
fn stopped_until_registered_again() {
    let (_scratch, config) = provider(
        "stopped",
        "function() calls = (calls or 0) + 1; error('boom') end",
    );
    assert!(config.runtime.decorations(&INFO).is_err());
    assert_eq!(config.runtime.decorations(&INFO), Ok(Vec::new()));
    assert_eq!(global::<i64>(&config, "calls"), 1);
    clean(&run_job(
        &config,
        "gband.core.provide('decorations', function() return { '[X]' } end)",
    ));
    assert_eq!(
        config.runtime.decorations(&INFO),
        Ok(vec![DecorationSpan::plain("[X]")])
    );
}

#[test]
fn lua_runs_count_callbacks_only() {
    let (_scratch, config) = provider("runs", "function() return { '[X]' } end");
    let start = config.runtime.lua_runs();
    config.runtime.decorations(&INFO).unwrap();
    assert_eq!(config.runtime.lua_runs(), start);
    clean(&run_job(&config, "x = 1"));
    assert_eq!(config.runtime.lua_runs(), start + 1);
}
