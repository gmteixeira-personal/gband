mod common;

use std::time::{Duration, Instant};

use common::*;
use gband_core::layout::{BandId, WindowId};
use gband_lua::{Color, ColumnState, Config, Event, StatusLine, Style, ViewState};

const COUNT: &str = "renders = {}\nlocal function counted(id, output)\n  return function(ctx)\n    renders[id] = (renders[id] or 0) + 1\n    widths = widths or {}\n    widths[id] = ctx.total_width\n    return output\n  end\nend\n";

fn loaded(name: &str, source: &str) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    scratch.write(&format!("{JOB}{COUNT}{source}"));
    let config = scratch.loaded();
    (scratch, config)
}

fn renders(config: &Config, id: &str) -> i64 {
    eval::<Option<i64>>(config, &format!("return renders['{id}']")).unwrap_or(0)
}

fn shown(config: &Config, state: ViewState) -> String {
    let width = state.width;
    let line = presented(config, state);
    text(&line, width)
}

fn ids(config: &Config) -> Vec<String> {
    eval(
        config,
        "local ids = {} for _, c in ipairs(gband.ui.statusline.list()) do ids[#ids + 1] = c.id end return ids",
    )
}

fn style_at(line: &StatusLine, col: u16) -> Style {
    line.spans
        .iter()
        .find(|span| span.col == col)
        .map(|span| span.style)
        .unwrap_or_else(|| panic!("no span at {col}: {line:?}"))
}

fn plugin_error(config: &Config) -> &gband_lua::ConfigError {
    config
        .errors
        .first()
        .unwrap_or_else(|| panic!("no error was reported"))
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
        "gband.ui.statusline.add({ align = 'right', priority = 3, order = 4, hl = 'WindowSegment', render = function() end })",
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
            "right",
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
    assert_eq!(defaults, ["left", "0", "0", "StatusLineSegment"]);
    let copy: String = eval(
        &config,
        "gband.ui.statusline.list()[1].align = 'right'\nreturn gband.ui.statusline.list()[1].align",
    );
    assert_eq!(copy, "left");
}

#[test]
fn remove_a_bundled_segment() {
    let scratch = Scratch::new("remove");
    let config = gband_lua::load_defaults(
        &scratch.locations(),
        gband_lua::Side::Client,
        &gband_lua::LoadOptions::default(),
    )
    .unwrap();
    let mut state = drawn(80);
    state.table = "prefix".to_owned();
    assert!(shown(&config, state.clone()).contains("prefix"));
    clean(&config.runtime.set_state(state.clone()));
    let lua = config.runtime.lua();
    lua.load("gband.bind = nil").exec().unwrap();
    let removed: bool = lua
        .load("return gband.ui.statusline.remove('mode')")
        .eval()
        .unwrap();
    assert!(removed);
    let line = config.runtime.take_line().unwrap();
    assert!(!text(&line, 80).contains("prefix"), "{}", text(&line, 80));
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
    scratch.write("gband.colorscheme('plain')");
    let config = scratch.loaded();
    let accent: Vec<String> = eval(
        &config,
        "local s = gband.hl.get('StatusLineAccent', { resolve = true })\nlocal out = {} for k, v in pairs(s) do out[#out + 1] = k .. '=' .. tostring(v) end\nreturn out",
    );
    assert_eq!(accent, ["bold=true"]);
    let line = presented(&config, drawn(10));
    assert!(line.base.reverse);
    assert_eq!(line.base.fg, None);
}

#[test]
fn plain_string() {
    let (_scratch, config) = loaded(
        "plain",
        "gband.colorscheme('none-such')\ngband.hl.set('StatusLine', { fg = 7, bg = 236 })\ngband.hl.set('StatusLineMuted', { fg = 3, bold = true })\ngband.ui.statusline.add({ id = 'a', hl = 'StatusLineMuted', render = function() return 'hi' end })",
    );
    let line = presented(&config, drawn(20));
    assert_eq!(text(&line, 20).trim_end(), "hi");
    let style = style_at(&line, 0);
    assert_eq!(style.fg, Some(Color::Index(3)));
    assert_eq!(style.bg, Some(Color::Index(236)));
    assert!(style.bold);
    assert_eq!(line.base.fg, Some(Color::Index(7)));
    assert_eq!(line.base.bg, Some(Color::Index(236)));
}

#[test]
fn escape_sequence_removed() {
    let (_scratch, config) = loaded(
        "escape",
        "gband.ui.statusline.add({ id = 'a', render = function() return { { text = 'a\\27[31mb\\194\\133' } } end })",
    );
    assert_eq!(shown(&config, drawn(20)).trim_end(), "a[31mb");
}

#[test]
fn spans_with_their_own_groups() {
    let (_scratch, config) = loaded(
        "spans",
        "gband.hl.set('One', { fg = 1 })\ngband.ui.statusline.add({ id = 'a', render = function() return { 'x', { text = 'y', hl = 'One' } } end })",
    );
    let line = presented(&config, drawn(20));
    assert_eq!(text(&line, 20).trim_end(), "xy");
    assert_eq!(style_at(&line, 1).fg, Some(Color::Index(1)));
}

#[test]
fn hidden_component() {
    let (_scratch, config) = loaded(
        "hidden",
        "for _, pair in ipairs({ { 'a', 'A' }, { 'b', '' }, { 'c', 'C' }, { 'd', { { text = '' } } }, { 'e', nil } }) do\n  gband.ui.statusline.add({ id = pair[1], render = counted(pair[1], pair[2]) })\nend",
    );
    assert_eq!(shown(&config, drawn(20)).trim_end(), "A │ C");
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
        drawn: true,
        error: None,
        ..ViewState::default()
    };
    presented(&config, state);
    let seen: Vec<String> = eval(
        &config,
        "return { seen.id, seen.side, tostring(seen.total_width), seen.table, tostring(seen.band.number), tostring(seen.band.index), tostring(seen.band.count), tostring(seen.column.index), tostring(seen.column.count), tostring(seen.window) }",
    );
    assert_eq!(
        seen,
        ["a", "client", "100", "prefix", "5", "2", "3", "3", "5", "7"]
    );
    let empty: bool = eval(&config, "return seen.width == 100");
    assert!(empty);
}

#[test]
fn context_without_a_column() {
    let (_scratch, config) = loaded(
        "context-empty",
        "gband.ui.statusline.add({ id = 'a', render = function(ctx) column, window = ctx.column, ctx.window end })",
    );
    presented(&config, drawn(20));
    let column: Option<mlua::Table> = global(&config, "column");
    assert!(column.is_none());
    let window: Option<i64> = global(&config, "window");
    assert!(window.is_none());
}

#[test]
fn available_width() {
    let (_scratch, config) = loaded(
        "available",
        "gband.ui.statusline.add({ id = 'a', order = 1, render = function() return '0123456789' end })\ngband.ui.statusline.add({ id = 'b', order = 2, render = function(ctx) width = ctx.width end })",
    );
    presented(&config, drawn(80));
    clean(&config.runtime.refresh_statusline());
    assert_eq!(global::<i64>(&config, "width"), 67);
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

const POSITION: &str = "gband.ui.statusline.add({ id = 'p', align = 'right', redraw_on = { 'FocusChanged' }, render = function(ctx) return ctx.column.index .. '/' .. ctx.column.count end })\n";

#[test]
fn fill_renders_after_the_others() {
    let (_scratch, config) = loaded(
        "fill-after",
        "order = {}\ngband.ui.statusline.add({ id = 'f', fill = true, order = 2, redraw_on = { 'KeyTableChanged' }, render = function(ctx) order[#order + 1] = 'f' fill_width = ctx.width return 'f' end })\ngband.ui.statusline.add({ id = 'm', order = 1, redraw_on = { 'KeyTableChanged' }, render = function(ctx) order[#order + 1] = 'm' if ctx.table ~= 'root' then return ctx.table end end })",
    );
    presented(&config, drawn(40));
    assert_eq!(global::<i64>(&config, "fill_width"), 40);
    clean(&run_job(&config, "order = {}"));
    let mut state = drawn(40);
    state.table = "prefix".to_owned();
    clean(&config.runtime.set_state(state));
    clean(&config.runtime.emit(&Event::KeyTableChanged {
        table: "prefix".to_owned(),
        previous: "root".to_owned(),
    }));
    let order: Vec<String> = global(&config, "order");
    assert_eq!(order, ["m", "f"]);
    assert_eq!(global::<i64>(&config, "fill_width"), 40 - 6 - 3);
}

#[test]
fn fill_follows_another_components_width() {
    let (_scratch, config) = loaded(
        "fill-follows",
        &format!(
            "{POSITION}gband.ui.statusline.add({{ id = 'f', fill = true, render = function(ctx) renders.f = (renders.f or 0) + 1 fill_width = ctx.width return 'f' end }})"
        ),
    );
    presented(&config, column_state(2, 3));
    assert_eq!(renders(&config, "f"), 1);
    let before = global::<i64>(&config, "fill_width");
    assert_eq!(before, 80 - 3 - 1);
    focus(&config, column_state(10, 12));
    assert_eq!(renders(&config, "f"), 2);
    assert_eq!(global::<i64>(&config, "fill_width"), before - 2);
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
    assert_eq!(renders(&config, "f2"), 2);
    focus(&config, column_state(10, 12));
    assert_eq!(renders(&config, "f1"), 3);
    assert_eq!(renders(&config, "f2"), 3);
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
fn default_segments() {
    let (_scratch, config) = default_config("default-segments");
    let shown = shown(&config, default_state());
    assert!(shown.starts_with("band 1 "), "{shown:?}");
    assert!(shown.ends_with(" 2/3"), "{shown:?}");
    assert_eq!(shown.chars().count(), 80);
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
    let line = config.runtime.take_line().unwrap();
    assert!(
        text(&line, 80).starts_with("band 1 │ prefix "),
        "{}",
        text(&line, 80)
    );
    assert!(style_at(&line, 9).bold);
}

#[test]
fn reorder_a_segment() {
    let (_scratch, config) = loaded(
        "reorder",
        "gband.plugin('gband.statusline.mode', { align = 'right', order = 1 })\ngband.plugin('gband.statusline.position')",
    );
    let mut state = default_state();
    state.table = "prefix".to_owned();
    assert!(shown(&config, state).ends_with("prefix │ 2/3"));
}

#[test]
fn lowest_priority_dropped_first() {
    let (_scratch, config) = loaded(
        "dropped",
        "gband.ui.statusline.add({ id = 'a', priority = 1, render = function() return 'aaaaaaaa' end })\ngband.ui.statusline.add({ id = 'b', priority = 2, render = function() return 'bbbbbbbb' end })",
    );
    assert_eq!(shown(&config, drawn(18)).trim_end(), "bbbbbbbb");
    assert_eq!(shown(&config, drawn(19)).trim_end(), "aaaaaaaa │ bbbbbbbb");
    let enabled: bool = eval(&config, "return gband.ui.statusline.list()[1].enabled");
    assert!(enabled);
}

#[test]
fn equal_priority_drops_the_last_added() {
    let (_scratch, config) = loaded(
        "dropped-tie",
        "gband.ui.statusline.add({ id = 'a', render = function() return 'aaaa' end })\ngband.ui.statusline.add({ id = 'b', align = 'right', render = function() return 'bbbb' end })",
    );
    assert_eq!(shown(&config, drawn(8)).trim_end(), "aaaa");
}

#[test]
fn last_component_cut() {
    let (_scratch, config) = loaded(
        "cut",
        "gband.ui.statusline.add({ id = 'a', render = function() return 'abcdefghij' end })",
    );
    assert_eq!(shown(&config, drawn(6)), "abcde…");
}

#[test]
fn spans_cut_across_groups() {
    let (_scratch, config) = loaded(
        "cut-spans",
        "gband.ui.statusline.add({ id = 'a', render = function() return { 'abc', 'defgh' } end })",
    );
    assert_eq!(shown(&config, drawn(6)), "abcde…");
    assert_eq!(shown(&config, drawn(4)), "abc…");
}

#[test]
fn wide_characters() {
    let (_scratch, config) = loaded(
        "wide",
        "gband.ui.statusline.add({ id = 'a', render = function() return '日本語' end })",
    );
    let line = presented(&config, drawn(5));
    assert_eq!(text(&line, 5), "日本…");
    assert_eq!(gband_lua::ui::width(&line.spans[0].text), 5);
}

#[test]
fn center_region() {
    let (_scratch, config) = loaded(
        "center",
        "gband.ui.statusline.add({ id = 'a', align = 'center', render = function() return 'mid' end })",
    );
    let line = presented(&config, drawn(40));
    assert_eq!(line.spans[0].col, 18);
}

#[test]
fn center_region_keeps_clear_of_the_sides() {
    let (_scratch, config) = loaded(
        "center-clear",
        "gband.ui.statusline.add({ id = 'l', render = function() return 'llllllllllllllllll' end })\ngband.ui.statusline.add({ id = 'c', align = 'center', render = function() return 'mid' end })\ngband.ui.statusline.add({ id = 'r', align = 'right', render = function() return 'r' end })",
    );
    let line = presented(&config, drawn(40));
    let center = line.spans.iter().find(|span| span.text == "mid").unwrap();
    assert_eq!(center.col, 19);
}

#[test]
fn separator_option() {
    let (_scratch, config) = loaded(
        "separator",
        "gband.opt.statusline_separator = ' | '\ngband.hl.set('StatusLineSeparator', { fg = 2 })\ngband.ui.statusline.add({ id = 'a', render = function() return 'a' end })\ngband.ui.statusline.add({ id = 'b', render = function() return 'b' end })",
    );
    let line = presented(&config, drawn(10));
    assert_eq!(text(&line, 10).trim_end(), "a | b");
    assert_eq!(style_at(&line, 1).fg, Some(Color::Index(2)));
}

#[test]
fn error_item_first_and_kept() {
    let (_scratch, config) = loaded(
        "error-item",
        "gband.hl.set('StatusLineError', { fg = 1 })\ngband.ui.statusline.add({ id = 'a', priority = 100, render = function() return 'aaaa' end })",
    );
    let mut state = drawn(30);
    state.error = Some("hello: boom\nmore".to_owned());
    let line = presented(&config, state.clone());
    assert_eq!(text(&line, 30).trim_end(), "hello: boom │ aaaa");
    assert_eq!(style_at(&line, 0).fg, Some(Color::Index(1)));
    state.width = 8;
    assert_eq!(shown(&config, state.clone()), "hello: …");
    state.error = None;
    clean(&config.runtime.set_state(state));
    let line = config.runtime.take_line().unwrap();
    assert_eq!(text(&line, 8).trim_end(), "aaaa");
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
    let line = config.runtime.take_line().unwrap();
    assert_eq!(text(&line, 30).trim_end(), "bad │ a");
    assert_eq!(renders(&config, "a"), 1);
}

#[test]
fn user_file_without_segments() {
    let (_scratch, config) = loaded("no-segments", "");
    let line = presented(&config, drawn(20));
    assert!(line.spans.is_empty());
    assert_eq!(line.height, 1);
}

#[test]
fn redraw_on_an_event() {
    let (_scratch, config) = loaded(
        "redraw-event",
        "gband.ui.statusline.add({ id = 'a', redraw_on = { 'FocusChanged' }, render = counted('a', 'a') })\ngband.ui.statusline.add({ id = 'b', render = counted('b', 'b') })",
    );
    presented(&config, drawn(20));
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
    presented(&config, drawn(20));
    clean(&config.runtime.emit(&Event::FocusChanged {
        window: Some(WindowId(1)),
        previous: None,
    }));
    let line = config.runtime.take_line().unwrap();
    assert_eq!(text(&line, 20).trim_end(), "handled");
}

#[test]
fn user_event_trigger() {
    let (_scratch, config) = loaded(
        "user-event",
        "gband.ui.statusline.add({ id = 'a', redraw_on = { 'User' }, render = counted('a', 'a') })",
    );
    presented(&config, drawn(20));
    clean(&run_job(&config, "gband.emit('mine')"));
    assert_eq!(renders(&config, "a"), 2);
}

#[test]
fn not_on_frames_or_takes() {
    let (_scratch, config) = loaded(
        "takes",
        "gband.ui.statusline.add({ id = 'a', render = counted('a', 'a') })",
    );
    presented(&config, drawn(20));
    for _ in 0..20 {
        config.runtime.take_line();
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
    presented(&config, drawn(20));
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
fn no_timer_while_not_drawn() {
    let (_scratch, config) = loaded(
        "interval-off",
        "gband.ui.statusline.add({ id = 'a', redraw_interval = 100, render = counted('a', 'a') })",
    );
    assert_eq!(config.runtime.next_timer(), None);
    presented(&config, drawn(20));
    assert!(config.runtime.next_timer().is_some());
    let mut state = drawn(20);
    state.drawn = false;
    clean(&config.runtime.set_state(state));
    assert_eq!(config.runtime.next_timer(), None);
    clean(&config.runtime.refresh_statusline());
    assert_eq!(renders(&config, "a"), 1);
}

#[test]
fn no_render_while_not_drawn() {
    let (_scratch, config) = loaded(
        "not-drawn",
        "gband.ui.statusline.add({ id = 'a', redraw_on = { 'FocusChanged' }, render = counted('a', 'a') })",
    );
    let mut state = drawn(20);
    state.drawn = false;
    clean(&config.runtime.set_state(state));
    clean(&config.runtime.refresh_statusline());
    clean(&config.runtime.emit(&Event::FocusChanged {
        window: None,
        previous: None,
    }));
    assert_eq!(renders(&config, "a"), 0);
    assert_eq!(config.runtime.take_line(), None);
}

#[test]
fn width_change() {
    let (_scratch, config) = loaded(
        "width",
        "gband.ui.statusline.add({ id = 'a', render = counted('a', 'a') })\ngband.ui.statusline.add({ id = 'b', render = counted('b', 'b') })",
    );
    presented(&config, drawn(80));
    clean(
        &config
            .runtime
            .emit(&Event::TerminalResized { cols: 80, rows: 30 }),
    );
    assert_eq!(renders(&config, "a"), 1);
    clean(&config.runtime.set_state(drawn(60)));
    clean(
        &config
            .runtime
            .emit(&Event::TerminalResized { cols: 60, rows: 30 }),
    );
    assert_eq!(renders(&config, "a"), 2);
    assert_eq!(renders(&config, "b"), 2);
    let widths: Vec<i64> = eval(&config, "return { widths.a, widths.b }");
    assert_eq!(widths, [60, 60]);
}

#[test]
fn highlight_change_renders_every_component() {
    let (_scratch, config) = loaded(
        "highlight-change",
        "gband.ui.statusline.add({ id = 'a', render = counted('a', 'a') })",
    );
    presented(&config, drawn(20));
    clean(&run_job(&config, "gband.hl.set('StatusLine', { fg = 1 })"));
    assert_eq!(renders(&config, "a"), 2);
    let line = config.runtime.take_line().unwrap();
    assert_eq!(line.base.fg, Some(Color::Index(1)));
}

#[test]
fn added_after_the_load() {
    let (_scratch, config) = loaded("added-later", "");
    presented(&config, drawn(20));
    clean(&run_job(
        &config,
        "gband.ui.statusline.add({ id = 'late', render = function() renders.late = (renders.late or 0) + 1 return 'late' end })",
    ));
    assert_eq!(renders(&config, "late"), 1);
    let line = config.runtime.take_line().unwrap();
    assert_eq!(text(&line, 20).trim_end(), "late");
}

#[test]
fn failing_component() {
    let scratch = Scratch::new("failing");
    let file = scratch.client_plugin("window",
        "gband.ui.statusline.add({\n  order = 1,\n  render = function()\n\n\n\n\n\n    error('boom')\n  end,\n})",
    );
    scratch.write(
        "gband.ui.statusline.add({ id = 'ok', order = 2, render = function() return 'ok' end })",
    );
    let config = scratch.loaded();
    clean(&config.runtime.set_state(drawn(40)));
    let outcome = config.runtime.refresh_statusline();
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    let message = format!("window: {}:9: boom", file.display());
    assert_eq!(error.to_string(), message);
    let line = config.runtime.take_line().unwrap();
    assert_eq!(text(&line, 40).trim_end(), "ok");
    let enabled: bool = eval(&config, "return gband.ui.statusline.list()[2].enabled");
    assert!(!enabled);
    let mut state = drawn(400);
    state.error = Some(message.clone());
    let shown = shown(&config, state);
    assert!(shown.starts_with(&format!("{message} │ ok")), "{shown}");
}

#[test]
fn invalid_return_disables() {
    let (_scratch, config) = loaded(
        "invalid-return",
        "gband.ui.statusline.add({ id = 'a', render = counted('a', 42) })",
    );
    clean(&config.runtime.set_state(drawn(20)));
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
        "{JOB}{COUNT}gband.ui.statusline.add({{ id = 'b', order = 2, render = counted('b', 'b') }})"
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
    let line = config.runtime.take_line().unwrap();
    assert_eq!(text(&line, 40).trim_end(), "b");
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
    scratch.write(JOB);
    let config = scratch.load_with_budget(500_000).unwrap();
    assert_eq!(shown(&config, drawn(20)).trim_end(), "window");
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
    let line = config.runtime.take_line().unwrap();
    assert!(line.spans.is_empty(), "{line:?}");
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
        ["band", "left", "20", "10", "StatusLineSegment", "band"]
    );
    let mut state = drawn(20);
    state.band.index = 2;
    state.band.number = 7;
    assert_eq!(shown(&config, state).trim_end(), "band 2");
}

#[test]
fn mode_segment() {
    let (_scratch, config) = loaded("mode", "gband.plugin('gband.statusline.mode')");
    let entry: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[1]\nreturn { e.id, e.align, tostring(e.priority), tostring(e.order), e.hl }",
    );
    assert_eq!(entry, ["mode", "left", "30", "20", "StatusLineAccent"]);
    assert_eq!(shown(&config, drawn(20)).trim_end(), "");
    let mut state = drawn(20);
    state.table = "move".to_owned();
    assert_eq!(shown(&config, state).trim_end(), "move");
}

#[test]
fn position_segment() {
    let (_scratch, config) = loaded("position", "gband.plugin('gband.statusline.position')");
    let entry: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[1]\nreturn { e.id, e.align, tostring(e.priority), tostring(e.order), e.hl }",
    );
    assert_eq!(entry, ["position", "right", "10", "10", "StatusLineMuted"]);
    assert_eq!(shown(&config, drawn(20)).trim(), "");
    let mut state = drawn(20);
    state.column = Some(ColumnState { index: 3, count: 7 });
    assert_eq!(shown(&config, state).trim_start(), "3/7");
}

#[test]
fn clock_segment() {
    let (_scratch, config) = loaded(
        "clock",
        "gband.plugin('gband.statusline.clock', { format = '<%Y>', interval = 500, align = 'left' })",
    );
    let entry: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[1]\nreturn { e.id, e.align, tostring(e.priority), tostring(e.order), e.hl }",
    );
    assert_eq!(entry, ["clock", "left", "5", "20", "StatusLineMuted"]);
    let year: String = eval(&config, "return os.date('<%Y>')");
    assert_eq!(shown(&config, drawn(20)).trim_end(), year);
    let next = config.runtime.next_timer().unwrap();
    assert!(next <= Instant::now() + Duration::from_millis(500));
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
        "table.insert(gband.runtimepath, {:?})\ngband.colorscheme('dusk')\ngband.plugin('window')",
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
    let line = presented(&config, state);
    assert_eq!(text(&line, 30).trim_start(), "window 3");
    assert_eq!(line.spans[0].style.fg, Some(Color::Rgb(0xf6, 0xc1, 0x77)));
    let mut state = drawn(30);
    state.window = Some(4);
    clean(&config.runtime.set_state(state));
    clean(&config.runtime.emit(&Event::FocusChanged {
        window: Some(WindowId(4)),
        previous: Some(WindowId(3)),
    }));
    let line = config.runtime.take_line().unwrap();
    assert_eq!(text(&line, 30).trim_start(), "window 4");
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
    let line = presented(&config, state);
    assert!(text(&line, 40).contains("waiting 2"), "{}", text(&line, 40));
    assert_eq!(eval::<String>(&config, "return first"), "1/1");
}
