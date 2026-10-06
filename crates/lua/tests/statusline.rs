mod common;

use std::time::{Duration, Instant};

use common::*;
use gband_core::layout::{BandId, WindowId};
use gband_lua::{Bar, BarSide, Color, ColumnState, Config, Event, Style, ViewState};

const COUNT: &str = "renders = {}\nlocal function counted(id, output)\n  return function(ctx)\n    renders[id] = (renders[id] or 0) + 1\n    heights = heights or {}\n    heights[id] = ctx.total_height\n    return output\n  end\nend\n";

const SETUP: &str = "gband.plugin('gband.statusline')\n";

fn loaded_with(name: &str, setup: &str, source: &str) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    scratch.write(&format!("{JOB}{COUNT}{setup}{source}"));
    let config = scratch.loaded();
    (scratch, config)
}

fn loaded(name: &str, source: &str) -> (Scratch, Config) {
    loaded_with(name, SETUP, source)
}

fn sized_by(min_width: u16, max_width: u16) -> String {
    format!(
        "gband.plugin('gband.statusline', {{ min_width = {min_width}, max_width = {max_width} }})\n"
    )
}

fn renders(config: &Config, id: &str) -> i64 {
    eval::<Option<i64>>(config, &format!("return renders['{id}']")).unwrap_or(0)
}

fn shown(config: &Config, state: ViewState) -> Vec<(usize, String)> {
    shown_rows(&presented(config, state))
}

fn latest(config: &Config) -> Bar {
    status_bar(config).expect("the status line is presented again")
}

fn row(bar: &Bar, row: usize) -> String {
    rows(bar).get(row).cloned().unwrap_or_default()
}

fn ids(config: &Config) -> Vec<String> {
    eval(
        config,
        "local ids = {} for _, c in ipairs(gband.ui.statusline.list()) do ids[#ids + 1] = c.id end return ids",
    )
}

fn plugin_error(config: &Config) -> &gband_lua::ConfigError {
    config
        .errors
        .first()
        .unwrap_or_else(|| panic!("no error was reported"))
}

fn line(row: usize, text: &str) -> (usize, String) {
    (row, text.to_owned())
}

#[test]
fn plugin_component_without_an_id() {
    let scratch = Scratch::new("plugin-no-id");
    scratch.client_plugin(
        "window",
        "added = gband.ui.statusline.add({ render = function() return 'x' end })",
    );
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "added"), "window");
}

#[test]
fn plugin_component_with_an_id() {
    let scratch = Scratch::new("plugin-id");
    scratch.client_plugin("window",
        "added = gband.ui.statusline.add({ id = 'count', render = function() end })\nfull = gband.ui.statusline.add({ id = 'window.other', render = function() end })\nok = pcall(gband.ui.statusline.add, { id = 'else.where', render = function() end })",
    );
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "added"), "window.count");
    assert_eq!(global::<String>(&config, "full"), "window.other");
    assert!(!global::<bool>(&config, "ok"));
}

#[test]
fn user_component() {
    let (_scratch, config) = loaded(
        "user",
        "added = gband.ui.statusline.add({ id = 'host', render = function() end })",
    );
    assert_eq!(global::<String>(&config, "added"), "host");
    let plugin: Option<String> = eval(&config, "return gband.ui.statusline.list()[1].plugin");
    assert_eq!(plugin, None);
}

#[test]
fn components_without_the_plugin() {
    let (_scratch, config) = loaded_with(
        "without-plugin",
        "",
        "added = gband.ui.statusline.add({ id = 'host', render = counted('host', 'x') })",
    );
    assert_eq!(global::<String>(&config, "added"), "host");
    clean(&config.runtime.set_state(drawn(80)));
    clean(&config.runtime.refresh_statusline());
    assert_eq!(renders(&config, "host"), 0);
    assert!(status_bar(&config).is_none());
    assert_eq!(config.runtime.next_timer(), None);
    assert!(!config.runtime.error_item_shown());
}

#[test]
fn duplicate_id() {
    let scratch = Scratch::new("duplicate");
    let path = scratch.write(
        "local r = function() end\ngband.ui.statusline.add({ id = 'host', render = r })\n\n\ngband.ui.statusline.add({ id = 'host', render = r })",
    );
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &path, 5, "host");
}

#[test]
fn unknown_event() {
    let scratch = Scratch::new("unknown-event");
    let module = scratch.plugin_file(
        "window",
        "lua/window/init.lua",
        "return { setup = function()\n\n  gband.ui.statusline.add({ redraw_on = { 'FocusChange' }, render = function() end })\nend }",
    );
    scratch.write("gband.plugin('window')");
    let config = scratch.loaded();
    let error = plugin_error(&config);
    assert_eq!(error.plugin.as_deref(), Some("window"));
    assert_error_at(error, &module, 3, "FocusChange");
}

#[test]
fn invalid_specs_add_nothing() {
    for (spec, mentions) in [
        ("{ id = 'a' }", "render"),
        ("{ id = 'a', render = 1 }", "render"),
        ("{ render = function() end }", "id"),
        ("{ id = '', render = function() end }", "id"),
        (
            "{ id = 'a', align = 'middle', render = function() end }",
            "align",
        ),
        (
            "{ id = 'a', align = 'right', render = function() end }",
            "align",
        ),
        (
            "{ id = 'a', priority = 'high', render = function() end }",
            "priority",
        ),
        ("{ id = 'a', order = {}, render = function() end }", "order"),
        ("{ id = 'a', hl = '1x', render = function() end }", "hl"),
        (
            "{ id = 'a', redraw_interval = 50, render = function() end }",
            "redraw_interval",
        ),
        (
            "{ id = 'a', redraw_interval = 150.5, render = function() end }",
            "redraw_interval",
        ),
        (
            "{ id = 'a', redraw_on = 'User', render = function() end }",
            "redraw_on",
        ),
        (
            "{ id = 'a', colour = 1, render = function() end }",
            "colour",
        ),
    ] {
        let scratch = Scratch::new("invalid-spec");
        let path = scratch.write(&format!(
            "ok = pcall(function() end)\ngband.ui.statusline.add({spec})"
        ));
        let error = scratch.load().err().unwrap();
        assert_error_at(&error, &path, 2, mentions);
    }
}

#[test]
fn user_events_and_built_in_events_are_accepted() {
    let (_scratch, config) = loaded(
        "events",
        "gband.ui.statusline.add({ id = 'a', redraw_on = { 'User', 'LayoutChanged', 'HighlightChanged' }, redraw_interval = 100, render = function() end })",
    );
    assert_eq!(ids(&config), ["a"]);
}

#[test]
fn list_entries() {
    let scratch = Scratch::new("list");
    scratch.client_plugin("window",
        "gband.ui.statusline.add({ align = 'bottom', priority = 3, order = 4, hl = 'WindowSegment', render = function() end })",
    );
    scratch.write("gband.ui.statusline.add({ id = 'zeta', render = function() end })\ngband.ui.statusline.add({ id = 'alpha', render = function() end })");
    let config = scratch.loaded();
    assert_eq!(ids(&config), ["alpha", "window", "zeta"]);
    let entry: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[2]\nreturn { e.id, e.align, tostring(e.priority), tostring(e.order), e.hl, e.plugin, tostring(e.enabled) }",
    );
    assert_eq!(
        entry,
        [
            "window",
            "bottom",
            "3",
            "4",
            "WindowSegment",
            "window",
            "true"
        ]
    );
    let defaults: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[1]\nreturn { e.align, tostring(e.priority), tostring(e.order), e.hl }",
    );
    assert_eq!(defaults, ["top", "0", "0", "StatusLineSegment"]);
    let copy: String = eval(
        &config,
        "gband.ui.statusline.list()[1].align = 'bottom'\nreturn gband.ui.statusline.list()[1].align",
    );
    assert_eq!(copy, "top");
}

fn default_config(name: &str) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    let config = gband_lua::load_defaults(
        &scratch.locations(),
        gband_lua::Side::Client,
        &gband_lua::LoadOptions::default(),
    )
    .unwrap();
    (scratch, config)
}

fn default_state() -> ViewState {
    let mut state = drawn(80);
    state.column = Some(ColumnState { index: 2, count: 3 });
    state.window = Some(2);
    state
}

#[test]
fn default_placement() {
    let (_scratch, config) = default_config("default-placement");
    let bar = presented(&config, default_state());
    assert_eq!(bar.slot.side, BarSide::Left);
    assert_eq!(bar.slot.size, 20);
    assert_eq!(rows(&bar).len(), 24);
    let placed: Vec<i64> = eval(
        &config,
        "local info = gband.bar.info('statusline') return { info.col, info.width, info.height }",
    );
    assert_eq!(placed, [0, 20, 24]);
}

#[test]
fn right_side() {
    let (_scratch, config) = loaded_with(
        "right-side",
        "gband.plugin('gband.statusline', { side = 'right' })\n",
        "",
    );
    let bar = presented(&config, drawn(80));
    assert_eq!(bar.slot.side, BarSide::Right);
    let col: i64 = eval(&config, "return gband.bar.info('statusline').col");
    assert_eq!(col, 60);
}

#[test]
fn top_placement() {
    let (_scratch, config) = loaded_with(
        "top-placement",
        "ok = gband.plugin('gband.statusline', { position = 'top' })\n",
        "",
    );
    assert!(!global::<bool>(&config, "ok"));
    let error = plugin_error(&config);
    assert_eq!(error.plugin.as_deref(), Some("statusline"));
    assert!(error.message.contains("position"), "{error}");
}

#[test]
fn invalid_setup_options() {
    for (opts, mentions) in [
        ("{ side = 'top' }", "side"),
        ("{ min_width = 0 }", "min_width"),
        ("{ min_width = 2.5 }", "min_width"),
        ("{ min_width = 30, max_width = 20 }", "max_width"),
        ("{ order = 'first' }", "order"),
        ("'left'", "statusline"),
    ] {
        let scratch = Scratch::new("invalid-setup");
        scratch.write(&format!("ok = gband.plugin('gband.statusline', {opts})"));
        let config = scratch.loaded();
        assert!(!global::<bool>(&config, "ok"), "{opts}");
        assert!(plugin_error(&config).message.contains(mentions), "{opts}");
        let count: i64 = eval(&config, "return #gband.bar.list()");
        assert_eq!(count, 0, "{opts}");
    }
}

#[test]
fn off() {
    let (_scratch, config) = loaded_with("off", "", "");
    clean(&config.runtime.set_state(drawn(80)));
    clean(&config.runtime.refresh_statusline());
    let count: i64 = eval(&config, "return #gband.bar.list()");
    assert_eq!(count, 0);
}

#[test]
fn terminal_too_short() {
    let (_scratch, config) = default_config("too-short");
    let mut state = default_state();
    state.width = 20;
    presented(&config, state);
    let placed: bool = eval(&config, "return gband.bar.info('statusline').shown");
    assert!(!placed);
}

#[test]
fn grows_with_its_content() {
    let (_scratch, config) = loaded_with(
        "grows",
        &sized_by(10, 30),
        "gband.ui.statusline.add({ id = 'a', render = function() return string.rep('a', 25) end })",
    );
    let bar = presented(&config, drawn(80));
    assert_eq!(bar.slot.size, 25);
    assert_eq!(row(&bar, 0), "a".repeat(25));
}

#[test]
fn capped_width() {
    let (_scratch, config) = loaded_with(
        "capped",
        &sized_by(10, 30),
        "gband.ui.statusline.add({ id = 'a', render = function() return string.rep('a', 50) end })",
    );
    let bar = presented(&config, drawn(80));
    assert_eq!(bar.slot.size, 30);
    assert_eq!(row(&bar, 0), format!("{}…", "a".repeat(29)));
}

#[test]
fn remove_a_bundled_segment() {
    let (_scratch, config) = default_config("remove");
    let mut state = drawn(80);
    state.table = "prefix".to_owned();
    assert!(
        shown(&config, state.clone())
            .iter()
            .any(|(_, text)| text == "navigation")
    );
    clean(&config.runtime.set_state(state.clone()));
    let lua = config.runtime.lua();
    lua.load("gband.bind = nil").exec().unwrap();
    let removed: bool = lua
        .load("return gband.ui.statusline.remove('mode')")
        .eval()
        .unwrap();
    assert!(removed);
    clean(&config.runtime.set_state(state));
    let bar = latest(&config);
    assert!(
        !rows(&bar).iter().any(|text| text == "navigation"),
        "{:?}",
        rows(&bar)
    );
    let again: bool = lua
        .load("return gband.ui.statusline.remove('mode')")
        .eval()
        .unwrap();
    assert!(!again);
}

#[test]
fn defaults_without_a_colorscheme_setting() {
    let scratch = Scratch::new("defaults");
    scratch.user_file("colors/plain.lua", "");
    scratch.write(&format!("{SETUP}gband.colorscheme('plain')"));
    let config = scratch.loaded();
    let accent: Vec<String> = eval(
        &config,
        "local s = gband.hl.get('StatusLineAccent', { resolve = true })\nlocal out = {} for k, v in pairs(s) do out[#out + 1] = k .. '=' .. tostring(v) end\nreturn out",
    );
    assert_eq!(accent, ["bold=true"]);
    let bar = presented(&config, drawn(30));
    assert!(bar.base.reverse);
    assert_eq!(bar.base.fg, None);
}

#[test]
fn plain_string() {
    let (_scratch, config) = loaded(
        "plain",
        "gband.colorscheme('none-such')\ngband.hl.set('StatusLine', { fg = 7, bg = 236 })\ngband.hl.set('StatusLineMuted', { fg = 3, bold = true })\ngband.ui.statusline.add({ id = 'a', hl = 'StatusLineMuted', render = function() return 'hi' end })",
    );
    let bar = presented(&config, drawn(30));
    assert_eq!(row(&bar, 0), "hi");
    let style = bar.lines[0][0].style;
    assert_eq!(style.fg, Some(Color::Index(3)));
    assert_eq!(style.bg, Some(Color::Index(236)));
    assert!(style.bold);
    assert_eq!(bar.base.fg, Some(Color::Index(7)));
    assert_eq!(bar.base.bg, Some(Color::Index(236)));
}

#[test]
fn escape_sequence_removed() {
    let (_scratch, config) = loaded(
        "escape",
        "gband.ui.statusline.add({ id = 'a', render = function() return { { text = 'a\\27[31mb\\194\\133' } } end })",
    );
    assert_eq!(shown(&config, drawn(30)), [line(0, "a[31mb")]);
}

#[test]
fn spans_with_their_own_groups() {
    let (_scratch, config) = loaded(
        "spans",
        "gband.hl.set('One', { fg = 1 })\ngband.ui.statusline.add({ id = 'a', render = function() return { 'x', { text = 'y', hl = 'One' } } end })",
    );
    let bar = presented(&config, drawn(30));
    assert_eq!(row(&bar, 0), "xy");
    assert_eq!(bar.lines[0][1].style.fg, Some(Color::Index(1)));
}

#[test]
fn hidden_component() {
    let (_scratch, config) = loaded(
        "hidden",
        "for _, pair in ipairs({ { 'a', 'A' }, { 'b', '' }, { 'c', 'C' }, { 'd', { { text = '' } } }, { 'e', nil }, { 'f', { lines = {} } }, { 'g', { lines = { '', {} } } } }) do\n  gband.ui.statusline.add({ id = pair[1], render = counted(pair[1], pair[2]) })\nend",
    );
    assert_eq!(shown(&config, drawn(30)), [line(0, "A"), line(1, "C")]);
}

#[test]
fn several_lines() {
    let (_scratch, config) = loaded(
        "several",
        "gband.ui.statusline.add({ id = 'a', render = function() return { lines = { 'one', { 'tw', { text = 'o', hl = 'One' } } } } end })\ngband.ui.statusline.add({ id = 'b', render = function() return 'three' end })",
    );
    assert_eq!(
        shown(&config, drawn(30)),
        [line(0, "one"), line(1, "two"), line(2, "three")]
    );
}

#[test]
fn invalid_lines_disable() {
    let (_scratch, config) = loaded(
        "invalid-lines",
        "gband.ui.statusline.add({ id = 'a', render = counted('a', { lines = 'one' }) })\ngband.ui.statusline.add({ id = 'b', render = counted('b', { lines = { 3 } }) })",
    );
    clean(&config.runtime.set_state(drawn(30)));
    let outcome = config.runtime.refresh_statusline();
    assert_eq!(outcome.errors.len(), 2, "{:?}", outcome.errors);
    let enabled: Vec<bool> = eval(
        &config,
        "local l = gband.ui.statusline.list() return { l[1].enabled, l[2].enabled }",
    );
    assert_eq!(enabled, [false, false]);
}

#[test]
fn context_values() {
    let (_scratch, config) = loaded(
        "context",
        "gband.ui.statusline.add({ id = 'a', render = function(ctx) seen = ctx end })",
    );
    let state = ViewState {
        table: "prefix".to_owned(),
        band: gband_lua::BandState {
            number: 5,
            index: 2,
            count: 3,
        },
        column: Some(ColumnState { index: 3, count: 5 }),
        window: Some(7),
        width: 100,
        height: 30,
        error: None,
        ..ViewState::default()
    };
    presented(&config, state);
    let seen: Vec<String> = eval(
        &config,
        "return { seen.id, seen.side, tostring(seen.total_width), tostring(seen.total_height), seen.table, tostring(seen.band.number), tostring(seen.band.index), tostring(seen.band.count), tostring(seen.column.index), tostring(seen.column.count), tostring(seen.window), tostring(seen.width), tostring(seen.height) }",
    );
    assert_eq!(
        seen,
        [
            "a", "client", "40", "30", "prefix", "5", "2", "3", "3", "5", "7", "40", "30"
        ]
    );
}

#[test]
fn context_without_a_column() {
    let (_scratch, config) = loaded(
        "context-empty",
        "gband.ui.statusline.add({ id = 'a', render = function(ctx) column, window = ctx.column, ctx.window end })",
    );
    presented(&config, drawn(30));
    let column: Option<mlua::Table> = global(&config, "column");
    assert!(column.is_none());
    let window: Option<i64> = global(&config, "window");
    assert!(window.is_none());
}

#[test]
fn available_width() {
    let (_scratch, config) = loaded_with(
        "available",
        &sized_by(10, 40),
        "gband.ui.statusline.add({ id = 'a', order = 1, render = function() return string.rep('a', 14) end })\ngband.ui.statusline.add({ id = 'b', order = 2, fill = true, render = function(ctx) width = ctx.width return string.rep('b', 30) end })",
    );
    let bar = presented(&config, drawn(80));
    assert_eq!(global::<i64>(&config, "width"), 14);
    assert_eq!(bar.slot.size, 14);
    assert_eq!(row(&bar, 1), format!("{}…", "b".repeat(13)));
}

#[test]
fn available_height() {
    let (_scratch, config) = loaded(
        "available-height",
        "gband.ui.statusline.add({ id = 'a', order = 1, render = function() return 'a' end })\ngband.ui.statusline.add({ id = 'f', order = 2, fill = true, render = function(ctx) height = ctx.height end })\ngband.ui.statusline.add({ id = 'z', align = 'bottom', render = function() return 'z' end })",
    );
    presented(&config, drawn(80));
    assert_eq!(global::<i64>(&config, "height"), 21);
}

#[test]
fn fill_of_the_wrong_type() {
    let scratch = Scratch::new("fill-type");
    let path = scratch.write(
        "local r = function() end\n\n\ngband.ui.statusline.add({ id = 'a', fill = 'yes', render = r })",
    );
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &path, 4, "fill");
}

#[test]
fn list_entries_hold_fill() {
    let (_scratch, config) = loaded(
        "fill-list",
        "gband.ui.statusline.add({ id = 'a', fill = true, render = function() end })\ngband.ui.statusline.add({ id = 'b', render = function() end })",
    );
    let fills: Vec<bool> = eval(
        &config,
        "local l = gband.ui.statusline.list()\nreturn { l[1].fill, l[2].fill }",
    );
    assert_eq!(fills, [true, false]);
}

fn column_state(index: u32, count: u32) -> ViewState {
    let mut state = drawn(80);
    state.column = Some(ColumnState { index, count });
    state
}

fn focus(config: &Config, state: ViewState) {
    clean(&config.runtime.set_state(state));
    clean(&config.runtime.emit(&Event::FocusChanged {
        window: Some(WindowId(1)),
        previous: Some(WindowId(2)),
    }));
}

const POSITION: &str = "gband.ui.statusline.add({ id = 'p', align = 'bottom', redraw_on = { 'FocusChanged' }, render = function(ctx) return ctx.column.index .. '/' .. ctx.column.count end })\n";

#[test]
fn fill_renders_after_the_others() {
    let (_scratch, config) = loaded(
        "fill-after",
        "order = {}\ngband.ui.statusline.add({ id = 'f', fill = true, order = 2, redraw_on = { 'KeyTableChanged' }, render = function(ctx) order[#order + 1] = 'f' fill_height = ctx.height return 'f' end })\ngband.ui.statusline.add({ id = 'm', order = 1, redraw_on = { 'KeyTableChanged' }, render = function(ctx) order[#order + 1] = 'm' if ctx.table ~= 'root' then return 'navigation' end end })",
    );
    presented(&config, drawn(80));
    assert_eq!(global::<i64>(&config, "fill_height"), 24);
    clean(&run_job(&config, "order = {}"));
    let mut state = drawn(80);
    state.table = "prefix".to_owned();
    clean(&config.runtime.set_state(state));
    clean(&config.runtime.emit(&Event::KeyTableChanged {
        table: "prefix".to_owned(),
        previous: "root".to_owned(),
    }));
    let order: Vec<String> = global(&config, "order");
    assert_eq!(order, ["m", "f"]);
    assert_eq!(global::<i64>(&config, "fill_height"), 23);
}

#[test]
fn fill_follows_another_components_width() {
    let (_scratch, config) = loaded_with(
        "fill-follows",
        &sized_by(10, 40),
        "gband.ui.statusline.add({ id = 'p', redraw_on = { 'FocusChanged' }, render = function(ctx) return string.rep('p', 18 + ctx.column.index) end })\ngband.ui.statusline.add({ id = 'f', fill = true, render = function(ctx) renders.f = (renders.f or 0) + 1 fill_width = ctx.width return 'f' end })",
    );
    presented(&config, column_state(2, 3));
    assert_eq!(renders(&config, "f"), 1);
    assert_eq!(global::<i64>(&config, "fill_width"), 20);
    focus(&config, column_state(4, 12));
    assert_eq!(renders(&config, "f"), 2);
    assert_eq!(global::<i64>(&config, "fill_width"), 22);
}

#[test]
fn fill_width_unchanged() {
    let (_scratch, config) = loaded(
        "fill-unchanged",
        &format!(
            "{POSITION}gband.ui.statusline.add({{ id = 'f', fill = true, render = counted('f', 'f') }})"
        ),
    );
    presented(&config, column_state(2, 3));
    focus(&config, column_state(3, 3));
    assert_eq!(renders(&config, "f"), 1);
}

#[test]
fn fill_pass_renders_once_more_at_most() {
    let (_scratch, config) = loaded(
        "fill-bounded",
        &format!(
            "{POSITION}for _, pair in ipairs({{ {{ 'f1', 2 }}, {{ 'f2', 1 }} }}) do\n  local id = pair[1]\n  gband.ui.statusline.add({{ id = id, fill = true, priority = pair[2], render = function(ctx) renders[id] = (renders[id] or 0) + 1 return string.rep('x', ctx.width // 2) end }})\nend"
        ),
    );
    presented(&config, column_state(2, 3));
    assert_eq!(renders(&config, "f1"), 2);
    assert_eq!(renders(&config, "f2"), 1);
    focus(&config, column_state(10, 12));
    assert_eq!(renders(&config, "f1"), 2);
    assert_eq!(renders(&config, "f2"), 1);
}

#[test]
fn default_segments() {
    let (_scratch, config) = default_config("default-segments");
    assert_eq!(
        shown(&config, default_state()),
        [
            line(0, "band 1"),
            line(1, "C-space navigation"),
            line(23, "2/3")
        ]
    );
}

#[test]
fn mode_shown() {
    let (_scratch, config) = default_config("mode-shown");
    presented(&config, default_state());
    let mut state = default_state();
    state.table = "prefix".to_owned();
    clean(&config.runtime.set_state(state));
    clean(&config.runtime.emit(&Event::KeyTableChanged {
        table: "prefix".to_owned(),
        previous: "root".to_owned(),
    }));
    let bar = latest(&config);
    assert_eq!(row(&bar, 0), "band 1");
    assert_eq!(row(&bar, 1), "navigation");
    assert!(row(&bar, 2).starts_with("h left"), "{:?}", rows(&bar));
    assert!(bar.lines[1][0].style.bold);
}

#[test]
fn label_of_a_mode() {
    let (_scratch, config) = loaded(
        "mode-label",
        "gband.keymap.mode('resize', { label = 'RESIZE' })
gband.keymap.set('resize', '=', gband.action.grow_column_width)
gband.plugin('gband.statusline.mode')",
    );
    let mut state = default_state();
    state.table = "resize".to_owned();
    assert_eq!(shown(&config, state), [line(0, "RESIZE")]);
}

#[test]
fn reorder_a_segment() {
    let (_scratch, config) = loaded(
        "reorder",
        "gband.plugin('gband.statusline.mode', { align = 'bottom', order = 1 })\ngband.plugin('gband.statusline.position')",
    );
    let mut state = default_state();
    state.table = "prefix".to_owned();
    assert_eq!(shown(&config, state), [line(22, "prefix"), line(23, "2/3")]);
}

#[test]
fn lowest_priority_dropped_first() {
    let (_scratch, config) = loaded(
        "dropped",
        "gband.ui.statusline.add({ id = 'a', priority = 1, render = function() return 'aaa' end })\ngband.ui.statusline.add({ id = 'b', priority = 2, render = function() return 'bbb' end })",
    );
    assert_eq!(shown(&config, sized(40, 1)), [line(0, "bbb")]);
    assert_eq!(
        shown(&config, sized(40, 2)),
        [line(0, "aaa"), line(1, "bbb")]
    );
    let enabled: bool = eval(&config, "return gband.ui.statusline.list()[1].enabled");
    assert!(enabled);
}

#[test]
fn equal_priority_drops_the_last_added() {
    let (_scratch, config) = loaded(
        "dropped-tie",
        "gband.ui.statusline.add({ id = 'a', render = function() return 'aaaa' end })\ngband.ui.statusline.add({ id = 'b', align = 'bottom', render = function() return 'bbbb' end })",
    );
    assert_eq!(shown(&config, sized(40, 2)), [line(0, "aaaa")]);
    assert_eq!(
        shown(&config, sized(40, 3)),
        [line(0, "aaaa"), line(2, "bbbb")]
    );
}

#[test]
fn last_component_lines_cut_at_the_last_row() {
    let (_scratch, config) = loaded(
        "rows-cut",
        "gband.ui.statusline.add({ id = 'a', render = function() return { lines = { 'one', 'two', 'three' } } end })",
    );
    assert_eq!(
        shown(&config, sized(40, 2)),
        [line(0, "one"), line(1, "two")]
    );
}

#[test]
fn last_component_cut() {
    let (_scratch, config) = loaded_with(
        "cut",
        &sized_by(1, 6),
        "gband.ui.statusline.add({ id = 'a', render = function() return 'abcdefghij' end })",
    );
    let bar = presented(&config, drawn(30));
    assert_eq!(bar.slot.size, 6);
    assert_eq!(row(&bar, 0), "abcde…");
}

#[test]
fn spans_cut_across_groups() {
    for (max_width, expected) in [(6, "abcde…"), (4, "abc…")] {
        let (_scratch, config) = loaded_with(
            "cut-spans",
            &sized_by(1, max_width),
            "gband.ui.statusline.add({ id = 'a', render = function() return { 'abc', 'defgh' } end })",
        );
        assert_eq!(row(&presented(&config, drawn(30)), 0), expected);
    }
}

#[test]
fn wide_characters() {
    let (_scratch, config) = loaded_with(
        "wide",
        &sized_by(1, 5),
        "gband.ui.statusline.add({ id = 'a', render = function() return '日本語' end })",
    );
    let bar = presented(&config, drawn(30));
    assert_eq!(row(&bar, 0), "日本…");
    assert_eq!(gband_lua::ui::width(&row(&bar, 0)), 5);
}

#[test]
fn center_region() {
    let (_scratch, config) = loaded(
        "center",
        "gband.ui.statusline.add({ id = 'a', align = 'center', render = function() return 'mid' end })",
    );
    assert_eq!(shown(&config, drawn(40)), [line(11, "mid")]);
}

#[test]
fn center_region_keeps_clear_of_the_others() {
    let (_scratch, config) = loaded(
        "center-clear",
        "gband.ui.statusline.add({ id = 't', render = function() local l = {} for i = 1, 12 do l[i] = 't' end return { lines = l } end })\ngband.ui.statusline.add({ id = 'c', align = 'center', render = function() return 'mid' end })\ngband.ui.statusline.add({ id = 'b', align = 'bottom', render = function() return 'b' end })",
    );
    let shown = shown(&config, drawn(40));
    assert!(shown.contains(&line(13, "mid")), "{shown:?}");
    assert!(shown.contains(&line(23, "b")), "{shown:?}");
}

#[test]
fn error_item_first_and_kept() {
    let (_scratch, config) = loaded(
        "error-item",
        "gband.hl.set('StatusLineError', { fg = 1 })\ngband.ui.statusline.add({ id = 'a', priority = 100, render = function() return 'aaaa' end })",
    );
    let mut state = drawn(30);
    state.error = Some("hello: boom\nmore".to_owned());
    let bar = presented(&config, state.clone());
    assert_eq!(shown_rows(&bar), [line(0, "error"), line(1, "aaaa")]);
    assert_eq!(bar.lines[0][0].style.fg, Some(Color::Index(1)));
    assert!(config.runtime.error_item_shown());
    state.height = 1;
    assert_eq!(shown(&config, state.clone()), [line(0, "error")]);
    state.height = 24;
    state.width = 20;
    presented(&config, state.clone());
    assert!(!config.runtime.error_item_shown());
    state.width = 30;
    state.error = None;
    clean(&config.runtime.set_state(state));
    assert_eq!(shown_rows(&latest(&config)), [line(0, "aaaa")]);
    assert!(!config.runtime.error_item_shown());
}

#[test]
fn error_change_relays_the_line_without_renders() {
    let (_scratch, config) = loaded(
        "error-change",
        "gband.ui.statusline.add({ id = 'a', render = counted('a', 'a') })",
    );
    presented(&config, drawn(30));
    let mut state = drawn(30);
    state.error = Some("bad".to_owned());
    clean(&config.runtime.set_state(state));
    assert_eq!(
        shown_rows(&latest(&config)),
        [line(0, "error"), line(1, "a")]
    );
    assert_eq!(renders(&config, "a"), 1);
}

#[test]
fn user_file_without_segments() {
    let (_scratch, config) = loaded("no-segments", "");
    let bar = presented(&config, drawn(30));
    assert!(shown_rows(&bar).is_empty());
    assert_eq!(bar.slot.size, 20);
}

#[test]
fn redraw_on_an_event() {
    let (_scratch, config) = loaded(
        "redraw-event",
        "gband.ui.statusline.add({ id = 'a', redraw_on = { 'FocusChanged' }, render = counted('a', 'a') })\ngband.ui.statusline.add({ id = 'b', render = counted('b', 'b') })",
    );
    presented(&config, drawn(30));
    let focus = Event::FocusChanged {
        window: Some(WindowId(1)),
        previous: Some(WindowId(2)),
    };
    clean(&config.runtime.emit(&focus));
    clean(&config.runtime.emit(&Event::BandChanged {
        band: BandId(2),
        previous: BandId(1),
    }));
    assert_eq!(renders(&config, "a"), 2);
    assert_eq!(renders(&config, "b"), 1);
}

#[test]
fn render_follows_the_handlers() {
    let (_scratch, config) = loaded(
        "after-handlers",
        "gband.on('FocusChanged', function() mark = 'handled' end)\ngband.ui.statusline.add({ id = 'a', redraw_on = { 'FocusChanged' }, render = function() return mark end })",
    );
    presented(&config, drawn(30));
    clean(&config.runtime.emit(&Event::FocusChanged {
        window: Some(WindowId(1)),
        previous: None,
    }));
    assert_eq!(row(&latest(&config), 0), "handled");
}

#[test]
fn user_event_trigger() {
    let (_scratch, config) = loaded(
        "user-event",
        "gband.ui.statusline.add({ id = 'a', redraw_on = { 'User' }, render = counted('a', 'a') })",
    );
    presented(&config, drawn(30));
    clean(&run_job(&config, "gband.emit('mine')"));
    assert_eq!(renders(&config, "a"), 2);
}

#[test]
fn not_on_frames_or_takes() {
    let (_scratch, config) = loaded(
        "takes",
        "gband.ui.statusline.add({ id = 'a', render = counted('a', 'a') })",
    );
    presented(&config, drawn(30));
    for _ in 0..20 {
        config.runtime.take_bars();
        clean(&config.runtime.set_state(drawn(30)));
    }
    assert_eq!(renders(&config, "a"), 1);
}

#[test]
fn interval() {
    let (_scratch, config) = loaded(
        "interval",
        "gband.ui.statusline.add({ id = 'a', redraw_interval = 1000, render = counted('a', 'a') })",
    );
    let start = Instant::now();
    presented(&config, drawn(30));
    for step in 0..=35 {
        clean(
            &config
                .runtime
                .fire_timers(start + Duration::from_millis(100 * step)),
        );
    }
    assert_eq!(renders(&config, "a"), 1 + 3);
}

#[test]
fn no_timer_until_the_first_render() {
    let (_scratch, config) = loaded(
        "interval-off",
        "gband.ui.statusline.add({ id = 'a', redraw_interval = 100, render = counted('a', 'a') })",
    );
    assert_eq!(config.runtime.next_timer(), None);
    presented(&config, drawn(30));
    assert!(config.runtime.next_timer().is_some());
}

#[test]
fn height_change() {
    let (_scratch, config) = loaded(
        "height",
        "gband.ui.statusline.add({ id = 'a', render = counted('a', 'a') })\ngband.ui.statusline.add({ id = 'b', render = counted('b', 'b') })",
    );
    presented(&config, drawn(80));
    clean(&config.runtime.set_state(sized(60, 24)));
    clean(
        &config
            .runtime
            .emit(&Event::TerminalResized { cols: 60, rows: 24 }),
    );
    assert_eq!(renders(&config, "a"), 1);
    clean(&config.runtime.set_state(sized(60, 30)));
    assert_eq!(renders(&config, "a"), 2);
    assert_eq!(renders(&config, "b"), 2);
    let heights: Vec<i64> = eval(&config, "return { heights.a, heights.b }");
    assert_eq!(heights, [30, 30]);
}

#[test]
fn highlight_change_renders_every_component() {
    let (_scratch, config) = loaded(
        "highlight-change",
        "gband.ui.statusline.add({ id = 'a', render = counted('a', 'a') })",
    );
    presented(&config, drawn(30));
    clean(&run_job(&config, "gband.hl.set('StatusLine', { fg = 1 })"));
    assert_eq!(renders(&config, "a"), 2);
    assert_eq!(latest(&config).base.fg, Some(Color::Index(1)));
}

#[test]
fn added_after_the_load() {
    let (_scratch, config) = loaded("added-later", "");
    presented(&config, drawn(30));
    clean(&run_job(
        &config,
        "gband.ui.statusline.add({ id = 'late', render = function() renders.late = (renders.late or 0) + 1 return 'late' end })",
    ));
    assert_eq!(renders(&config, "late"), 1);
    assert_eq!(row(&latest(&config), 0), "late");
}

#[test]
fn failing_component() {
    let scratch = Scratch::new("failing");
    let file = scratch.client_plugin("window",
        "gband.ui.statusline.add({\n  order = 1,\n  render = function()\n\n\n\n\n\n    error('boom')\n  end,\n})",
    );
    scratch.write(&format!(
        "{SETUP}gband.ui.statusline.add({{ id = 'ok', order = 2, render = function() return 'ok' end }})"
    ));
    let config = scratch.loaded();
    clean(&config.runtime.set_state(drawn(40)));
    let outcome = config.runtime.refresh_statusline();
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    let message = format!("window: {}:9: boom", file.display());
    assert_eq!(error.to_string(), message);
    assert_eq!(shown_rows(&latest(&config)), [line(0, "ok")]);
    let enabled: bool = eval(&config, "return gband.ui.statusline.list()[2].enabled");
    assert!(!enabled);
    let mut state = drawn(40);
    state.error = Some(message.clone());
    state.errors = vec![message];
    assert_eq!(shown(&config, state), [line(0, "error"), line(1, "ok")]);
}

#[test]
fn invalid_return_disables() {
    let (_scratch, config) = loaded(
        "invalid-return",
        "gband.ui.statusline.add({ id = 'a', render = counted('a', 42) })",
    );
    clean(&config.runtime.set_state(drawn(30)));
    let outcome = config.runtime.refresh_statusline();
    assert_eq!(outcome.errors.len(), 1);
    assert!(
        outcome.errors[0].message.contains('a'),
        "{:?}",
        outcome.errors
    );
    clean(&config.runtime.refresh_statusline());
    assert_eq!(renders(&config, "a"), 1);
}

#[test]
fn looping_render() {
    let scratch = Scratch::new("looping");
    scratch.client_plugin("spin",
        "gband.ui.statusline.add({ order = 1, render = function() while true do end end })\ngband.on('FocusChanged', function() spin_handled = true end)",
    );
    scratch.write(&format!(
        "{JOB}{COUNT}{SETUP}gband.ui.statusline.add({{ id = 'b', order = 2, render = counted('b', 'b') }})"
    ));
    let config = scratch.load_with_budget(500_000).unwrap();
    clean(&config.runtime.set_state(drawn(40)));
    let outcome = config.runtime.refresh_statusline();
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_eq!(error.plugin.as_deref(), Some("spin"));
    assert!(error.message.contains("instruction limit"), "{error}");
    assert_eq!(renders(&config, "b"), 1);
    assert_eq!(shown_rows(&latest(&config)), [line(0, "b")]);
    clean(&config.runtime.emit(&Event::FocusChanged {
        window: None,
        previous: None,
    }));
    let handled: Option<bool> = global(&config, "spin_handled");
    assert_eq!(handled, None);
}

#[test]
fn failed_plugin_components_are_hidden() {
    let scratch = Scratch::new("failed-plugin");
    scratch.client_plugin("window",
        "gband.ui.statusline.add({ render = function() return 'window' end })\ngband.bind('alt+p', function() while true do end end)",
    );
    scratch.write(&format!("{JOB}{SETUP}"));
    let config = scratch.load_with_budget(500_000).unwrap();
    assert_eq!(shown(&config, drawn(30)), [line(0, "window")]);
    let gband_lua::Binding::Callback(spin) = config.keymap["root"]
        .iter()
        .find(|(chord, _)| *chord == gband_lua::Chord::Key(key("alt+p")))
        .unwrap()
        .1
    else {
        panic!();
    };
    assert!(!config.runtime.call(spin).errors.is_empty());
    clean(&config.runtime.refresh_statusline());
    assert!(shown_rows(&latest(&config)).is_empty());
}

#[test]
fn band_segment() {
    let (_scratch, config) = loaded("band", "gband.plugin('gband.statusline.band')");
    let entry: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[1]\nreturn { e.id, e.align, tostring(e.priority), tostring(e.order), e.hl, e.plugin }",
    );
    assert_eq!(
        entry,
        ["band", "top", "20", "10", "StatusLineSegment", "band"]
    );
    let mut state = drawn(30);
    state.band.index = 2;
    state.band.number = 7;
    assert_eq!(shown(&config, state), [line(0, "band 2")]);
}

#[test]
fn mode_segment() {
    let (_scratch, config) = loaded("mode", "gband.plugin('gband.statusline.mode')");
    let entry: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[1]\nreturn { e.id, e.align, tostring(e.priority), tostring(e.order), e.hl }",
    );
    assert_eq!(entry, ["mode", "top", "30", "20", "StatusLineAccent"]);
    assert!(shown(&config, drawn(30)).is_empty());
    let mut state = drawn(30);
    state.table = "move".to_owned();
    assert_eq!(shown(&config, state), [line(0, "move")]);
}

#[test]
fn position_segment() {
    let (_scratch, config) = loaded("position", "gband.plugin('gband.statusline.position')");
    let entry: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[1]\nreturn { e.id, e.align, tostring(e.priority), tostring(e.order), e.hl }",
    );
    assert_eq!(entry, ["position", "bottom", "10", "10", "StatusLineMuted"]);
    assert!(shown(&config, drawn(30)).is_empty());
    let mut state = drawn(30);
    state.column = Some(ColumnState { index: 3, count: 7 });
    assert_eq!(shown(&config, state), [line(23, "3/7")]);
}

#[test]
fn clock_segment() {
    let (_scratch, config) = loaded(
        "clock",
        "gband.plugin('gband.statusline.clock', { format = '<%Y>', interval = 500, align = 'top' })",
    );
    let entry: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[1]\nreturn { e.id, e.align, tostring(e.priority), tostring(e.order), e.hl }",
    );
    assert_eq!(entry, ["clock", "top", "5", "20", "StatusLineMuted"]);
    let year: String = eval(&config, "return os.date('<%Y>')");
    assert_eq!(shown(&config, drawn(30)), [(0, year)]);
    let next = config.runtime.next_timer().unwrap();
    assert!(next <= Instant::now() + Duration::from_millis(500));
}

#[test]
fn clock_defaults_to_the_bottom() {
    let (_scratch, config) = loaded("clock-bottom", "gband.plugin('gband.statusline.clock')");
    let align: String = eval(&config, "return gband.ui.statusline.list()[1].align");
    assert_eq!(align, "bottom");
}

#[test]
fn clock_is_not_a_default() {
    let (_scratch, config) = default_config("clock-default");
    assert!(!ids(&config).contains(&"clock".to_owned()));
}

#[test]
fn invalid_segment_options_fail_setup() {
    for (module, opts, mentions) in [
        ("band", "{ align = 'middle' }", "align"),
        ("band", "{ align = 'left' }", "align"),
        ("mode", "{ priority = 'high' }", "priority"),
        ("position", "{ hl = 3 }", "hl"),
        ("clock", "{ format = 3 }", "format"),
        ("clock", "{ interval = 10 }", "redraw_interval"),
        ("band", "'left'", "band"),
    ] {
        let scratch = Scratch::new("invalid-segment");
        scratch.write(&format!(
            "ok = gband.plugin('gband.statusline.{module}', {opts})"
        ));
        let config = scratch.loaded();
        assert!(!global::<bool>(&config, "ok"), "{module} {opts}");
        let error = plugin_error(&config);
        assert_eq!(error.plugin.as_deref(), Some(module));
        assert!(error.message.contains(mentions), "{error}");
        assert!(ids(&config).is_empty());
    }
}

#[test]
fn sample_window_plugin() {
    let example = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/plugins/window")
        .canonicalize()
        .unwrap();
    let scratch = Scratch::new("sample-window");
    scratch.write(&format!(
        "{SETUP}table.insert(gband.runtimepath, {:?})\ngband.colorscheme('dusk')\ngband.plugin('window')",
        example.display().to_string()
    ));
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "dusk"
    );
    let style: Vec<String> = eval(
        &config,
        "local s = gband.hl.get('WindowSegment', { resolve = true })\nreturn { s.fg, tostring(s.bold) }",
    );
    assert_eq!(style, ["#f6c177", "true"]);
    let mut state = drawn(30);
    state.window = Some(3);
    let bar = presented(&config, state);
    let (row, text) = shown_rows(&bar).pop().unwrap();
    assert_eq!(text, "window 3");
    assert_eq!(
        bar.lines[row][0].style.fg,
        Some(Color::Rgb(0xf6, 0xc1, 0x77))
    );
    let mut state = drawn(30);
    state.window = Some(4);
    clean(&config.runtime.set_state(state));
    clean(&config.runtime.emit(&Event::FocusChanged {
        window: Some(WindowId(4)),
        previous: Some(WindowId(3)),
    }));
    assert_eq!(shown_rows(&latest(&config)).pop().unwrap().1, "window 4");
}

#[test]
fn sample_window_plugin_default_group() {
    let example = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/plugins/window")
        .canonicalize()
        .unwrap();
    let scratch = Scratch::new("sample-window-default");
    scratch.write(&format!(
        "table.insert(gband.runtimepath, {:?})\ngband.plugin('window')",
        example.display().to_string()
    ));
    let config = scratch.loaded();
    let link: String = eval(&config, "return gband.hl.get('WindowSegment').link");
    assert_eq!(link, "StatusLineAccent");
}

#[test]
fn waiting_agents_counted() {
    let (_scratch, config) = loaded(
        "waiting-agents",
        "gband.ui.statusline.add({
  id = 'agents',
  redraw_on = { 'WindowStateChanged' },
  render = function(ctx)
    local waiting = 0
    for _, entry in ipairs(ctx.windows) do
      if entry.state.agent == 'waiting' then
        waiting = waiting + 1
      end
    end
    first = ctx.windows[1].window .. '/' .. ctx.windows[1].band
    return 'waiting ' .. waiting
  end,
})",
    );
    let mut layout = gband_core::layout::Layout::new();
    let band = layout.bands()[0].id;
    let mut after = None;
    for _ in 0..3 {
        let window = layout.allocate_window();
        layout.open(
            window,
            band,
            after,
            None,
            &gband_core::layout::LayoutOptions::default(),
        );
        after = Some(window);
    }
    let waiting = || {
        std::collections::BTreeMap::from([(
            "agent".to_owned(),
            gband_protocol::Value::string("waiting"),
        )])
    };
    let state = ViewState {
        layout: std::sync::Arc::new(layout),
        states: std::sync::Arc::new(std::collections::BTreeMap::from([
            (WindowId(1), waiting()),
            (WindowId(3), waiting()),
        ])),
        ..drawn(40)
    };
    assert_eq!(shown(&config, state), [line(0, "waiting 2")]);
    assert_eq!(eval::<String>(&config, "return first"), "1/1");
}

#[test]
fn span_over_the_base() {
    let (_scratch, config) = loaded(
        "span-over-base",
        "gband.hl.set('StatusLine', { fg = 7, bg = 236 })\ngband.hl.set('Three', { fg = 3, bold = true })\ngband.ui.statusline.add({ id = 'a', hl = 'Three', render = function() return 'x' end })",
    );
    let bar = presented(&config, drawn(30));
    assert_eq!(
        bar.lines[0][0].style,
        Style {
            fg: Some(Color::Index(3)),
            bg: Some(Color::Index(236)),
            bold: true,
            ..Style::default()
        }
    );
    assert_eq!(bar.base.fg, Some(Color::Index(7)));
}
