mod common;

use common::*;
use gband_lua::{Bar, BarSide, Color, Config, Style, ViewState};

fn terminal(cols: u16, rows: u16) -> ViewState {
    ViewState {
        table: "root".to_owned(),
        width: cols,
        height: rows,
        ..ViewState::default()
    }
}

fn client(name: &str, source: &str) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    scratch.write(&format!("{JOB}{source}"));
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    clean(&config.runtime.set_state(terminal(80, 24)));
    (scratch, config)
}

fn bars(config: &Config) -> Vec<Bar> {
    config.runtime.take_bars().expect("the bars were presented")
}

fn texts(bar: &Bar) -> Vec<String> {
    bar.lines
        .iter()
        .map(|runs| runs.iter().map(|run| run.text.as_str()).collect())
        .collect()
}

fn failed_job(config: &Config, code: &str, mentions: &str) {
    let outcome = run_job(config, code);
    let [error] = outcome.errors.as_slice() else {
        panic!("{code}: {:?}", outcome.errors);
    };
    assert!(error.message.contains(mentions), "{code}: {error}");
}

#[test]
fn unknown_bar() {
    let (_scratch, config) = client("unknown", "");
    failed_job(&config, "gband.bar.set_lines('absent', {})", "absent");
    failed_job(&config, "gband.bar.set_config('absent', {})", "absent");
    failed_job(&config, "gband.bar.info('absent')", "absent");
    let removed: bool = eval(&config, "return gband.bar.remove('absent')");
    assert!(!removed);
}

#[test]
fn added_while_loading() {
    let (_scratch, config) = client(
        "loading",
        "gband.bar.add({ id = 'list', side = 'left', size = 20, lines = { 'hello' } })",
    );
    let [bar] = bars(&config).try_into().unwrap();
    assert_eq!(bar.id, "list");
    assert_eq!(bar.slot.side, BarSide::Left);
    assert_eq!(bar.slot.size, 20);
    assert_eq!(texts(&bar), ["hello"]);
}

#[test]
fn nothing_is_presented_until_a_bar_changes() {
    let (_scratch, config) = client("unchanged", "gband.bar.add({ id = 'list', side = 'left' })");
    assert_eq!(bars(&config).len(), 1);
    clean(&run_job(&config, "local unused = 1"));
    assert!(config.runtime.take_bars().is_none());
    clean(&run_job(&config, "gband.bar.set_lines('list', { 'x' })"));
    assert_eq!(texts(&bars(&config)[0]), ["x"]);
}

#[test]
fn plugin_bars_are_namespaced() {
    let scratch = Scratch::new("namespaced");
    scratch.client_plugin(
        "tabs",
        "plain = gband.bar.add({ side = 'left' })\nnamed = gband.bar.add({ id = 'list', side = 'right', size = 20 })",
    );
    scratch.write("");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    assert_eq!(global::<String>(&config, "plain"), "tabs");
    assert_eq!(global::<String>(&config, "named"), "tabs.list");
    let plugin: String = eval(&config, "return gband.bar.info('tabs.list').plugin");
    assert_eq!(plugin, "tabs");
}

#[test]
fn invalid_bars() {
    let scratch = Scratch::new("invalid");
    let path = scratch.write(
        "gband.bar.add({ id = 'info', side = 'left' })\nok, duplicate = pcall(gband.bar.add, { id = 'info', side = 'left' })\nok, top = pcall(gband.bar.add, { id = 'top', side = 'top' })\ngband.bar.add({ side = 'left' })",
    );
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &path, 4, "`id`");
    let (_scratch, config) = client(
        "invalid-calls",
        "gband.bar.add({ id = 'info', side = 'left' })",
    );
    for (code, mentions) in [
        ("gband.bar.add({ id = 'info', side = 'left' })", "info"),
        ("gband.bar.add({ id = 'top', side = 'top' })", "side"),
        ("gband.bar.add({ id = 'none' })", "side"),
        (
            "gband.bar.add({ id = 'wide', side = 'left', size = 0 })",
            "size",
        ),
        (
            "gband.bar.add({ id = 'wide', side = 'left', size = 1.5 })",
            "size",
        ),
        (
            "gband.bar.add({ id = 'odd', side = 'left', order = 'first' })",
            "order",
        ),
        (
            "gband.bar.add({ id = 'odd', side = 'left', hl = 'not a group' })",
            "hl",
        ),
        (
            "gband.bar.add({ id = 'odd', side = 'left', lines = { 5 } })",
            "line 1",
        ),
        (
            "gband.bar.add({ id = 'odd', side = 'left', colour = 1 })",
            "colour",
        ),
        ("gband.bar.add({ id = '', side = 'left' })", "id"),
        (
            "gband.bar.add({ id = 'odd', side = 'left', on_resize = 1 })",
            "on_resize",
        ),
        ("gband.bar.add('info')", "table"),
        ("gband.bar.set_config('info', { lines = {} })", "lines"),
        ("gband.bar.set_config('info', { size = -2 })", "size"),
        ("gband.bar.set_lines('info', 'x')", "lines"),
    ] {
        failed_job(&config, code, mentions);
    }
    let count: i64 = eval(&config, "return #gband.bar.list()");
    assert_eq!(count, 1);
    let size: i64 = eval(&config, "return gband.bar.info('info').size");
    assert_eq!(size, 1);
}

#[test]
fn widen_a_bar() {
    let (_scratch, config) = client(
        "widen",
        "gband.bar.add({ id = 'side', side = 'left', size = 10 })",
    );
    clean(&run_job(
        &config,
        "gband.bar.set_config('side', { size = 20 })",
    ));
    let placed: Vec<i64> = eval(
        &config,
        "local info = gband.bar.info('side') return { info.col, info.width }",
    );
    assert_eq!(placed, [0, 20]);
    assert_eq!(bars(&config)[0].slot.size, 20);
    clean(&run_job(
        &config,
        "gband.bar.set_config('side', { side = 'right', order = 3, hl = 'Title' })",
    ));
    let info: Vec<String> = eval(
        &config,
        "local info = gband.bar.info('side') return { info.side, tostring(info.order), info.hl, tostring(info.col) }",
    );
    assert_eq!(info, ["right", "3", "Title", "60"]);
}

#[test]
fn remove_a_bar() {
    let (_scratch, config) = client(
        "remove",
        "gband.bar.add({ id = 'side', side = 'left', size = 20 })",
    );
    let removed: bool = eval(&config, "return gband.bar.remove('side')");
    assert!(removed);
    clean(&run_job(&config, "local unused = 1"));
    assert!(bars(&config).is_empty());
    let count: i64 = eval(&config, "return #gband.bar.list()");
    assert_eq!(count, 0);
}

#[test]
fn info_of_a_shown_bar() {
    let (_scratch, config) = client(
        "info",
        "gband.bar.add({ id = 'info', side = 'right', size = 12 })",
    );
    let info: Vec<String> = eval(
        &config,
        "local info = gband.bar.info('info')
         info.size = 99
         local again = gband.bar.info('info')
         return { again.side, tostring(again.size), tostring(again.shown), tostring(again.col), tostring(again.width), tostring(again.height), tostring(again.plugin) }",
    );
    assert_eq!(info, ["right", "12", "true", "68", "12", "24", "nil"]);
}

#[test]
fn info_of_a_hidden_bar() {
    let (_scratch, config) = client(
        "hidden",
        "gband.bar.add({ id = 'wide', side = 'left', size = 80 })",
    );
    let info: Vec<String> = eval(
        &config,
        "local info = gband.bar.info('wide') return { tostring(info.shown), tostring(info.col), tostring(info.width) }",
    );
    assert_eq!(info, ["false", "nil", "nil"]);
}

#[test]
fn list_is_sorted_by_id() {
    let (_scratch, config) = client(
        "list",
        "gband.bar.add({ id = 'b', side = 'left' })\ngband.bar.add({ id = 'a', side = 'right' })",
    );
    let ids: Vec<String> = eval(
        &config,
        "local ids = {} for _, bar in ipairs(gband.bar.list()) do ids[#ids + 1] = bar.id end return ids",
    );
    assert_eq!(ids, ["a", "b"]);
}

const RESIZES: &str = "resizes = {}
gband.bar.add({ id = 'side', side = 'left', size = 20, on_resize = function(id, width, height)
  resizes[#resizes + 1] = id .. ' ' .. width .. 'x' .. height
end })
";

fn resizes(config: &Config) -> Vec<String> {
    global(config, "resizes")
}

#[test]
fn on_resize_runs_after_loading_and_when_the_size_changes() {
    let (_scratch, config) = client("resize", RESIZES);
    assert_eq!(resizes(&config), ["side 20x24"]);
    clean(&config.runtime.set_state(terminal(80, 24)));
    assert_eq!(resizes(&config), ["side 20x24"]);
    clean(&config.runtime.set_state(terminal(100, 30)));
    assert_eq!(resizes(&config), ["side 20x24", "side 20x30"]);
    clean(&config.runtime.set_state(terminal(20, 30)));
    assert_eq!(resizes(&config), ["side 20x24", "side 20x30", "side 0x0"]);
}

#[test]
fn on_resize_error_is_a_plugin_error() {
    let scratch = Scratch::new("resize-error");
    scratch.client_plugin(
        "tabs",
        "gband.bar.add({ side = 'left', size = 4, on_resize = function() error('boom') end })",
    );
    scratch.write("");
    let config = scratch.loaded();
    let outcome = config.runtime.set_state(terminal(80, 24));
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_eq!(error.plugin.as_deref(), Some("tabs"));
    assert!(error.message.contains("boom"), "{error}");
}

#[test]
fn failed_plugin_loses_its_bar() {
    let scratch = Scratch::new("failed");
    scratch.plugin("tabs", &manifest("tabs"));
    scratch.plugin_file(
        "tabs",
        "lua/tabs/init.lua",
        "return { name = 'tabs', setup = function() gband.bar.add({ side = 'left', size = 20 }) error('boom') end }",
    );
    scratch.write("gband.plugin('tabs')");
    let config = scratch.loaded();
    assert_eq!(config.errors.len(), 1, "{:?}", config.errors);
    clean(&config.runtime.set_state(terminal(80, 24)));
    assert!(bars(&config).is_empty());
    let count: i64 = eval(&config, "return #gband.bar.list()");
    assert_eq!(count, 0);
}

#[test]
fn line_cut_at_the_bars_edge() {
    let (_scratch, config) = client(
        "cut",
        "gband.bar.add({ id = 'side', side = 'left', size = 4, lines = { 'one', 'three', 'x' } })",
    );
    assert_eq!(texts(&bars(&config)[0]), ["one", "thre", "x"]);
}

#[test]
fn lines_beyond_the_rows_are_left_out() {
    let (_scratch, config) = client(
        "rows",
        "local lines = {} for row = 1, 30 do lines[row] = tostring(row) end
gband.bar.add({ id = 'side', side = 'left', size = 4, lines = lines })",
    );
    assert_eq!(bars(&config)[0].lines.len(), 24);
}

#[test]
fn styled_bar() {
    let (_scratch, config) = client(
        "styled",
        "gband.hl.set('Bar', { bg = 236 })\ngband.hl.set('Title', { bold = true })
gband.bar.add({ id = 'side', side = 'left', size = 10, lines = { { { text = 'gband', hl = 'Title' } } } })",
    );
    let bar = &bars(&config)[0];
    let base = Style {
        bg: Some(Color::Index(236)),
        ..Style::default()
    };
    assert_eq!(bar.base, base);
    assert_eq!(bar.lines[0][0].text, "gband");
    assert_eq!(bar.lines[0][0].style, Style { bold: true, ..base });
}

#[test]
fn theme_sets_a_bars_base() {
    let (_scratch, config) = client(
        "theme",
        "gband.bar.add({ id = 'side', side = 'left', size = 10 })",
    );
    assert_ne!(bars(&config)[0].base.bg, Some(Color::Index(236)));
    clean(&run_job(&config, "gband.hl.set('Bar', { bg = 236 })"));
    assert_eq!(bars(&config)[0].base.bg, Some(Color::Index(236)));
}
