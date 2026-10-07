mod common;

use std::sync::Arc;

use common::*;
use gband_core::action::Action;
use gband_core::geometry::Size;
use gband_core::input::{Key, KeyCode, Modifiers, MouseButton, WheelDirection};
use gband_core::layout::{Layout, LayoutOptions, Proportion, WindowId};
use gband_core::view::ViewAction;
use gband_lua::plugin_windows::{
    FloatingFrame, Frame, PluginBox, PluginMouse, PluginMouseKind, Run, TiledFrame,
};
use gband_lua::{
    BandState, Binding, Border, BorderChars, BoxCell, CharSet, Chord, Color, Config, Dispatch,
    Event, Outcome, PluginWindowRequest, Sides, Style, ViewState,
};
use mlua::Table;

fn layout() -> Layout {
    let mut layout = Layout::new();
    let window = layout.allocate_window();
    let band = layout.bands()[0].id;
    layout.open(window, band, None, None, &LayoutOptions::default());
    layout
}

fn state() -> ViewState {
    ViewState {
        table: "root".to_owned(),
        band: BandState {
            number: 1,
            index: 1,
            count: 2,
        },
        window: Some(1),
        width: 80,
        layout: Arc::new(layout()),
        area: Size::new(80, 24),
        ribbon: Size::new(80, 23),
        ..ViewState::default()
    }
}

struct Client {
    _scratch: Scratch,
    config: Config,
}

impl Client {
    fn new(name: &str, source: &str) -> Self {
        let scratch = Scratch::new(name);
        scratch.write(&format!("{JOB}{source}"));
        Self::loaded(scratch)
    }

    fn loaded(scratch: Scratch) -> Self {
        let config = scratch.loaded();
        clean(&config.runtime.set_state(state()));
        config.runtime.take_frames();
        Self {
            _scratch: scratch,
            config,
        }
    }

    fn run(&self, code: &str) -> Outcome {
        let outcome = run_job(&self.config, code);
        clean(&outcome);
        outcome
    }

    fn eval<T: mlua::FromLua>(&self, source: &str) -> T {
        eval(&self.config, source)
    }

    fn global<T: mlua::FromLua>(&self, name: &str) -> T {
        global(&self.config, name)
    }

    fn frames(&self) -> Vec<(u32, Option<Frame>)> {
        self.config.runtime.take_frames()
    }

    fn float(&self) -> FloatingFrame {
        match self.frames().pop() {
            Some((_, Some(Frame::Floating(frame)))) => frame,
            other => panic!("expected a float frame, got {other:?}"),
        }
    }

    fn mouse(&self, plugin_window: u32, kind: PluginMouseKind, row: u16) -> Outcome {
        self.config.runtime.plugin_window_mouse(
            plugin_window,
            &PluginMouse {
                kind,
                content: Some((1, row)),
                boxed: None,
                modifiers: Modifiers::NONE,
            },
        )
    }

    fn key(&self, plugin_window: u32, name: &str) -> Outcome {
        self.config
            .runtime
            .plugin_window_key(plugin_window, key(name))
    }
}

fn rows(lines: &[Vec<Run>]) -> Vec<String> {
    lines
        .iter()
        .map(|runs| runs.iter().map(|run| run.text.as_str()).collect::<String>())
        .collect()
}

fn trimmed(lines: &[Vec<Run>]) -> Vec<String> {
    rows(lines)
        .into_iter()
        .map(|row| row.trim_end().to_owned())
        .collect()
}

fn numbered(count: usize) -> String {
    let lines: Vec<String> = (1..=count).map(|n| format!("'{n}'")).collect();
    format!("{{ {} }}", lines.join(", "))
}

#[test]
fn open_while_loading() {
    let scratch = Scratch::new("win-loading");
    let path = scratch.write("\n\n\n\ngband.win.open({})\n");
    let error = scratch.load().map(|_| ()).unwrap_err();
    assert_error_at(&error, &path, 5, "binding function");
}

#[test]
fn info_and_list_while_loading() {
    for (index, code) in ["gband.win.list()", "gband.win.info(1)"].iter().enumerate() {
        let scratch = Scratch::new(&format!("win-info-loading-{index}"));
        let path = scratch.write(&format!("\n{code}\n"));
        let error = scratch.load().map(|_| ()).unwrap_err();
        assert_error_at(&error, &path, 2, "gband.win");
    }
}

#[test]
fn unknown_plugin_window() {
    let client = Client::new("win-unknown", "");
    let outcome = run_job(&client.config, "gband.win.set_lines(42, {})");
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert!(error.message.contains("window 42"), "{error}");
    client.run("gband.win.close(42)");
}

#[test]
fn numbers_are_not_reused() {
    let client = Client::new("win-numbers", "");
    client.run("first = gband.win.open({})\ngband.win.close(first)\nsecond = gband.win.open({})");
    let (first, second): (u32, u32) = (client.global("first"), client.global("second"));
    assert_ne!(first, second);
    let listed: Vec<u32> = client.eval("return gband.win.list()");
    assert_eq!(listed, [second]);
}

#[test]
fn bad_options_are_errors() {
    let client = Client::new("win-options", "");
    for (code, mentions) in [
        ("gband.win.open({ kind = 'tiled', row = 2 })", "row"),
        ("gband.win.open({ border = false, band = 1 })", "band"),
        (
            "gband.win.open({ keys = { ['ctrl+shift+1'] = print } })",
            "ctrl+shift+1",
        ),
        (
            "gband.win.open({ keys = { leftmouse = print } })",
            "leftmouse",
        ),
        ("gband.win.open({ kind = 'popup' })", "kind"),
        ("gband.win.open({ width = 0 })", "width"),
        ("gband.win.open({ col = -1 })", "col"),
        ("gband.win.open({ lines = { 5 } })", "line 1"),
        ("gband.win.open({ colour = 1 })", "colour"),
        ("gband.win.open({ on_input = 'text' })", "on_input"),
        ("gband.win.open({ on_mouse = 3 })", "on_mouse"),
        ("gband.win.open({ kind = 'tiled', after = 9 })", "9"),
        (
            "gband.win.open({ kind = 'tiled', column_width = 0 })",
            "column_width",
        ),
    ] {
        let outcome = run_job(&client.config, code);
        let [error] = outcome.errors.as_slice() else {
            panic!("{code}: {:?}", outcome.errors);
        };
        assert!(error.message.contains(mentions), "{code}: {error}");
        assert!(outcome.dispatched.is_empty());
    }
    let listed: Vec<u32> = client.eval("return gband.win.list()");
    assert!(listed.is_empty());
}

#[test]
fn invalid_border_table() {
    let client = Client::new("win-border-table-errors", "");
    for (code, mentions) in [
        (
            "gband.win.open({ border = { sides = { 'middle' } } })",
            "middle",
        ),
        (
            "gband.win.open({ border = { chars = 'dotted' } })",
            "dotted",
        ),
        ("gband.win.open({ border = { chars = { '+' } } })", "chars"),
        ("gband.win.open({ border = { colour = 1 } })", "colour"),
        ("gband.win.open({ border = 'rounded' })", "border"),
    ] {
        let outcome = run_job(&client.config, code);
        let [error] = outcome.errors.as_slice() else {
            panic!("{code}: {:?}", outcome.errors);
        };
        assert!(error.message.contains(mentions), "{code}: {error}");
    }
    let listed: Vec<u32> = client.eval("return gband.win.list()");
    assert!(listed.is_empty());
}

#[test]
fn border_table_frames_its_window() {
    let client = Client::new("win-border-table", "");
    client.run(
        "win = gband.win.open({ width = 20, height = 5, border = { sides = { 'top' }, chars = 'rounded' } })",
    );
    let frame = client.float();
    assert_eq!(
        frame.border,
        Some(Border {
            sides: Sides::parse(["top"]).unwrap(),
            chars: BorderChars::Named(CharSet::Rounded),
        })
    );
    let size: Vec<u32> =
        client.eval("local info = gband.win.info(win) return { info.cols, info.rows }");
    assert_eq!(size, [18, 3]);
    client.run("gband.win.set_config(win, { border = { chars = 'double' } })");
    assert_eq!(
        client.float().border,
        Some(Border {
            sides: Sides::ALL,
            chars: BorderChars::Named(CharSet::Double),
        })
    );
    client.run("gband.win.set_config(win, { border = false })");
    assert_eq!(client.float().border, None);
    client.run("gband.win.set_config(win, { border = true })");
    assert_eq!(client.float().border, Some(Border::default()));
}

#[test]
fn styled_line() {
    let client = Client::new(
        "win-styled",
        "gband.hl.set('PluginWindow', { fg = 7 })\ngband.hl.set('Key', { fg = 3, bold = true })",
    );
    client.run(
        "gband.win.open({ border = false, width = 20, height = 1, lines = { { { text = 'C-h', hl = 'Key' }, ' left' } } })",
    );
    let frame = client.float();
    let plugin_window = Style {
        fg: Some(Color::Index(7)),
        ..Style::default()
    };
    let key = Style {
        fg: Some(Color::Index(3)),
        bold: true,
        ..Style::default()
    };
    assert_eq!(frame.base, plugin_window);
    assert_eq!(
        frame.lines,
        [vec![
            Run {
                text: "C-h".to_owned(),
                style: key,
            },
            Run {
                text: " left".to_owned(),
                style: plugin_window,
            },
            Run {
                text: " ".repeat(12),
                style: plugin_window,
            },
        ]]
    );
}

#[test]
fn long_line_is_cut_and_controls_removed() {
    let client = Client::new("win-cut", "");
    client.run(
        "gband.win.open({ border = false, width = 5, height = 3, lines = { 'abcdefgh', 'a\\tb', '日本語' } })",
    );
    assert_eq!(rows(&client.float().lines), ["abcde", "ab   ", "日本 "]);
}

#[test]
fn scroll_a_long_list() {
    let client = Client::new("win-scroll", "");
    client.run(&format!(
        "win = gband.win.open({{ height = 12, lines = {} }})",
        numbered(25)
    ));
    client.frames();
    client.run("gband.win.scroll(win, 30)");
    let shown = trimmed(&client.float().lines);
    assert_eq!(shown.first().map(String::as_str), Some("16"));
    assert_eq!(shown.last().map(String::as_str), Some("25"));
    assert_eq!(shown.len(), 10);
}

#[test]
fn cursor_keeps_itself_shown() {
    let client = Client::new("win-cursor", "");
    client.run(&format!(
        "win = gband.win.open({{ height = 12, cursorline = true, lines = {} }})",
        numbered(25)
    ));
    client.frames();
    client.run("gband.win.set_cursor(win, 14)");
    let frame = client.float();
    let shown = trimmed(&frame.lines);
    assert_eq!((shown[0].as_str(), shown[9].as_str()), ("5", "14"));
    assert!(frame.lines[9].iter().all(|run| run.style.reverse));
    assert!(frame.lines[8].iter().all(|run| !run.style.reverse));
}

#[test]
fn fewer_lines_than_rows() {
    let client = Client::new("win-few", "");
    client.run("win = gband.win.open({ height = 12, lines = { 'a', 'b', 'c' } })");
    client.run("gband.win.scroll(win, 5)");
    let shown = trimmed(&client.float().lines);
    assert_eq!(shown[..4], ["a", "b", "c", ""]);
}

#[test]
fn key_entry_runs_with_the_plugin_window() {
    let client = Client::new("win-key-entry", "");
    client.run(
        "win = gband.win.open({ cursorline = true, lines = { 'a', 'b', 'c' }, keys = { enter = function(id) got = id; cursor = gband.win.info(id).cursor end } })\n\
         gband.win.set_cursor(win, 3)",
    );
    let win: u32 = client.global("win");
    clean(&client.key(win, "enter"));
    assert_eq!(client.global::<u32>("got"), win);
    assert_eq!(client.global::<u32>("cursor"), 3);
}

#[test]
fn default_keys() {
    let client = Client::new("win-default-keys", "");
    client.run(&format!(
        "win = gband.win.open({{ height = 12, lines = {} }})",
        numbered(25)
    ));
    let win: u32 = client.global("win");
    let top = || client.eval::<u32>("return gband.win.info(win).top");
    clean(&client.key(win, "down"));
    assert_eq!(top(), 2);
    clean(&client.key(win, "pagedown"));
    assert_eq!(top(), 12);
    clean(&client.key(win, "end"));
    assert_eq!(top(), 16);
    clean(&client.key(win, "home"));
    assert_eq!(top(), 1);
    let outcome = client.key(win, "x");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert_eq!(top(), 1);
}

#[test]
fn j_and_k_scroll_like_the_arrow_keys() {
    let client = Client::new("win-jk", "");
    client.run(&format!(
        "win = gband.win.open({{ height = 12, lines = {} }})",
        numbered(25)
    ));
    let win: u32 = client.global("win");
    let top = || client.eval::<u32>("return gband.win.info(win).top");
    let mut seen = Vec::new();
    for name in ["j", "j", "k"] {
        clean(&client.key(win, name));
        seen.push(top());
    }
    assert_eq!(seen, [2, 3, 2]);
    for name in ["h", "l"] {
        clean(&client.key(win, name));
        assert_eq!(top(), 2);
    }
}

#[test]
fn j_and_k_move_the_cursor_line() {
    let client = Client::new("win-jk-cursor", "");
    client.run(&format!(
        "win = gband.win.open({{ height = 12, cursorline = true, lines = {} }})",
        numbered(25)
    ));
    let win: u32 = client.global("win");
    let cursor = || client.eval::<u32>("return gband.win.info(win).cursor");
    clean(&client.key(win, "j"));
    clean(&client.key(win, "j"));
    assert_eq!(cursor(), 3);
    clean(&client.key(win, "k"));
    assert_eq!(cursor(), 2);
}

#[test]
fn keys_entry_wins_over_j() {
    let client = Client::new("win-jk-override", "");
    client.run(&format!(
        "win = gband.win.open({{ height = 12, lines = {}, keys = {{ j = function() pressed = true end }} }})",
        numbered(25)
    ));
    let win: u32 = client.global("win");
    clean(&client.key(win, "j"));
    assert!(client.global::<bool>("pressed"));
    assert_eq!(client.eval::<u32>("return gband.win.info(win).top"), 1);
}

#[test]
fn default_keys_move_the_cursor_line() {
    let client = Client::new("win-cursor-keys", "");
    client.run(&format!(
        "win = gband.win.open({{ height = 12, cursorline = true, lines = {} }})",
        numbered(25)
    ));
    let win: u32 = client.global("win");
    let at = || {
        client.eval::<Vec<u32>>("local info = gband.win.info(win) return { info.top, info.cursor }")
    };
    clean(&client.key(win, "down"));
    assert_eq!(at(), [1, 2]);
    clean(&client.key(win, "pagedown"));
    assert_eq!(at(), [11, 12]);
    clean(&client.key(win, "end"));
    assert_eq!(at(), [16, 25]);
    clean(&client.key(win, "up"));
    assert_eq!(at(), [16, 24]);
}

#[test]
fn escape_closes_a_float() {
    let client = Client::new("win-escape", "");
    client.run("win = gband.win.open({ on_close = function(id) closed = id end })");
    let win: u32 = client.global("win");
    client.frames();
    let outcome = client.key(win, "escape");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert_eq!(client.global::<u32>("closed"), win);
    assert_eq!(client.frames(), [(win, None)]);
    assert!(
        client
            .eval::<Vec<u32>>("return gband.win.list()")
            .is_empty()
    );
}

#[test]
fn q_closes_a_float() {
    let client = Client::new("win-q", "");
    client.run("win = gband.win.open({ on_close = function(id) closed = id end })");
    let win: u32 = client.global("win");
    client.frames();
    let outcome = client.key(win, "q");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert_eq!(client.global::<u32>("closed"), win);
    assert_eq!(client.frames(), [(win, None)]);
}

const RECORDER: &str = "typed = {}\n\
     record = function(id, text) typed[#typed + 1] = { id, text } end\n";

impl Client {
    fn typed(&self) -> Vec<(u32, String)> {
        self.eval::<Vec<Table>>("return typed")
            .iter()
            .map(|entry| (entry.get(1).unwrap(), entry.get(2).unwrap()))
            .collect()
    }

    fn paste(&self, plugin_window: u32, text: &str) -> Outcome {
        self.config.runtime.plugin_window_paste(plugin_window, text)
    }
}

#[test]
fn typed_characters_go_to_on_input() {
    let client = Client::new("win-on-input", RECORDER);
    client.run(&format!(
        "win = gband.win.open({{ height = 12, lines = {}, on_input = record }})",
        numbered(25)
    ));
    let win: u32 = client.global("win");
    for pressed in [
        key("j"),
        key("q"),
        Key::new(KeyCode::Char('A'), Modifiers::SHIFT),
        key("space"),
    ] {
        let outcome = client.config.runtime.plugin_window_key(win, pressed);
        clean(&outcome);
        assert!(outcome.dispatched.is_empty());
    }
    let expected: Vec<(u32, String)> = ["j", "q", "A", " "]
        .iter()
        .map(|text| (win, (*text).to_owned()))
        .collect();
    assert_eq!(client.typed(), expected);
    assert_eq!(client.eval::<Vec<u32>>("return gband.win.list()"), [win]);
    assert_eq!(client.eval::<u32>("return gband.win.info(win).top"), 1);
}

#[test]
fn keys_entry_wins_over_on_input() {
    let client = Client::new("win-on-input-keys", RECORDER);
    client.run(
        "win = gband.win.open({ keys = { j = function() pressed = true end }, on_input = record })",
    );
    let win: u32 = client.global("win");
    clean(&client.key(win, "j"));
    assert!(client.global::<bool>("pressed"));
    assert!(client.typed().is_empty());
}

#[test]
fn keys_without_text_keep_their_defaults() {
    let client = Client::new("win-on-input-defaults", RECORDER);
    client.run(&format!(
        "win = gband.win.open({{ height = 12, lines = {}, on_input = record }})",
        numbered(25)
    ));
    let win: u32 = client.global("win");
    clean(&client.key(win, "down"));
    assert_eq!(client.eval::<u32>("return gband.win.info(win).top"), 2);
    clean(&client.key(win, "escape"));
    assert!(
        client
            .eval::<Vec<u32>>("return gband.win.list()")
            .is_empty()
    );
    assert!(client.typed().is_empty());
}

#[test]
fn ctrl_and_alt_keys_are_not_text() {
    let client = Client::new("win-on-input-modifiers", RECORDER);
    client.run("win = gband.win.open({ on_input = record })");
    let win: u32 = client.global("win");
    for name in ["ctrl+x", "alt+x"] {
        let outcome = client.key(win, name);
        clean(&outcome);
        assert!(outcome.dispatched.is_empty());
    }
    assert!(client.typed().is_empty());
}

#[test]
fn paste_goes_to_on_input() {
    let client = Client::new("win-paste", RECORDER);
    client.run("win = gband.win.open({ on_input = record })");
    let win: u32 = client.global("win");
    let outcome = client.paste(win, "a\nb");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert_eq!(client.typed(), [(win, "a\nb".to_owned())]);
}

#[test]
fn paste_without_on_input() {
    let client = Client::new("win-paste-dropped", RECORDER);
    client.run("win = gband.win.open({ lines = { 'a' } })");
    let win: u32 = client.global("win");
    let outcome = client.paste(win, "hello");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert_eq!(client.eval::<Vec<u32>>("return gband.win.list()"), [win]);
    assert!(client.typed().is_empty());
}

#[test]
fn failing_input_handler_is_a_plugin_error() {
    let scratch = Scratch::new("win-failing-input");
    scratch.client_plugin("demo",
        "gband.bind('alt+p', function() win = gband.win.open({ on_input = function(_, text) seen = text; error('broken') end }) end)",
    );
    let client = Client::loaded(scratch);
    let chord = Chord::Key(key("alt+p"));
    let Some((_, Binding::Callback(callback))) = client.config.keymap["root"]
        .iter()
        .find(|(bound, _)| *bound == chord)
    else {
        panic!("alt+p is not bound");
    };
    clean(&client.config.runtime.call(*callback));
    let win: u32 = client.global("win");
    let outcome = client.key(win, "a");
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_eq!(error.plugin.as_deref(), Some("demo"));
    assert!(error.message.contains("broken"), "{error}");
    assert_eq!(client.eval::<Vec<u32>>("return gband.win.list()"), [win]);
    assert_eq!(client.key(win, "b").errors.len(), 1);
    assert_eq!(client.global::<String>("seen"), "b");
}

#[test]
fn failing_key_function_is_a_plugin_error() {
    let scratch = Scratch::new("win-failing-key");
    scratch.client_plugin("demo",
        "gband.bind('alt+p', function() win = gband.win.open({ keys = { x = function() error('broken') end } }) end)",
    );
    let client = Client::loaded(scratch);
    let chord = Chord::Key(key("alt+p"));
    let Some((_, Binding::Callback(callback))) = client.config.keymap["root"]
        .iter()
        .find(|(bound, _)| *bound == chord)
    else {
        panic!("alt+p is not bound");
    };
    clean(&client.config.runtime.call(*callback));
    let win: u32 = client.global("win");
    let outcome = client.key(win, "x");
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_eq!(error.plugin.as_deref(), Some("demo"));
    assert!(error.message.contains("broken"), "{error}");
    assert_eq!(client.eval::<Vec<u32>>("return gband.win.list()"), [win]);
}

#[test]
fn centered_float() {
    let client = Client::new("win-centered", "");
    client.run("win = gband.win.open({ width = 40, height = 11, lines = { 'a', 'b', 'c' } })");
    let frame = client.float();
    assert_eq!(
        (frame.row, frame.col, frame.width, frame.height),
        (6, 20, 40, 11)
    );
    assert_eq!(frame.lines.len(), 9);
    assert!(
        rows(&frame.lines)
            .iter()
            .all(|row| row.chars().count() == 38)
    );
    let info: Table = client.eval("return gband.win.info(win)");
    let field = |name: &str| info.get::<mlua::Value>(name).unwrap();
    assert_eq!(info.get::<String>("kind").unwrap(), "floating");
    assert!(info.get::<bool>("focused").unwrap());
    for (name, value) in [
        ("top", 1),
        ("cursor", 1),
        ("line_count", 3),
        ("cols", 38),
        ("rows", 9),
        ("row", 6),
        ("col", 20),
        ("width", 40),
        ("height", 11),
    ] {
        assert_eq!(field(name).as_i64(), Some(value), "{name}");
    }
    assert!(field("window").is_nil());
}

#[test]
fn default_size_is_half_the_ribbon() {
    let client = Client::new("win-default-size", "");
    client.run("gband.win.open({})");
    let frame = client.float();
    assert_eq!((frame.width, frame.height), (40, 11));
}

#[test]
fn float_cut_to_the_ribbon() {
    let client = Client::new("win-cut-box", "");
    client.run("gband.win.open({ col = 70, row = 100, width = 20, height = 5 })");
    let frame = client.float();
    assert_eq!((frame.col, frame.row), (60, 18));
}

#[test]
fn resize_a_float() {
    let client = Client::new("win-resize", "");
    client.run(
        "win = gband.win.open({ on_resize = function(id, cols, rows) resized = { id, cols, rows } end })",
    );
    client.run("gband.win.set_config(win, { width = 30, height = 12, title = 'Keys' })");
    let frame = client.float();
    assert_eq!((frame.width, frame.height), (30, 12));
    assert_eq!(frame.title.as_deref(), Some("Keys"));
    let win: u32 = client.global("win");
    assert_eq!(client.global::<Vec<u32>>("resized"), [win, 28, 10]);
}

#[test]
fn ribbon_resize_places_floats_again() {
    let client = Client::new("win-ribbon", "");
    client.run(
        "win = gband.win.open({ on_resize = function(id, cols, rows) resized = { cols, rows } end })",
    );
    client.frames();
    let mut smaller = state();
    smaller.ribbon = Size::new(40, 11);
    clean(&client.config.runtime.set_state(smaller));
    let frame = client.float();
    assert_eq!(
        (frame.row, frame.col, frame.width, frame.height),
        (3, 10, 20, 5)
    );
    assert_eq!(client.global::<Vec<u16>>("resized"), [18, 3]);
}

#[test]
fn set_config_on_a_tiled_plugin_window_is_an_error() {
    let client = Client::new("win-window-config", "");
    client.run("win = gband.win.open({ kind = 'tiled' })");
    let outcome = run_job(&client.config, "gband.win.set_config(win, { width = 3 })");
    assert_eq!(outcome.errors.len(), 1, "{:?}", outcome.errors);
}

#[test]
fn tiled_plugin_window_request_and_contents() {
    let client = Client::new("win-window", "");
    let outcome = client.run(
        "win = gband.win.open({ kind = 'tiled', lines = { 'hello' }, column_width = 1/4, on_resize = function(id, cols, rows) resized = { id, cols, rows } end })",
    );
    let win: u32 = client.global("win");
    assert_eq!(
        outcome.dispatched,
        [Dispatch::PluginWindow(PluginWindowRequest::Open {
            plugin_window: win,
            target: None,
            width: Some(Proportion::new(1, 4)),
            focus: true,
        })]
    );
    assert!(client.frames().is_empty());
    clean(
        &client
            .config
            .runtime
            .plugin_window_opened(win, Some(WindowId(5))),
    );
    assert_eq!(
        client.eval::<Option<u32>>("return gband.win.info(win).window"),
        Some(5)
    );
    assert!(
        client
            .eval::<Option<u32>>("return gband.win.info(win).cols")
            .is_none()
    );
    clean(&client.config.runtime.window_resized(win, Size::new(18, 22)));
    assert_eq!(client.global::<Vec<u32>>("resized"), [win, 18, 22]);
    let frames = client.frames();
    let [
        (
            id,
            Some(Frame::Tiled(TiledFrame {
                cols,
                rows: height,
                lines,
                ..
            })),
        ),
    ] = frames.as_slice()
    else {
        panic!("{frames:?}");
    };
    assert_eq!((*id, *cols, *height), (win, 18, 22));
    assert_eq!(trimmed(lines)[0], "hello");
    assert_eq!(lines.len(), 22);
    let layout: Option<u32> = client.eval(
        "for _, band in ipairs(gband.layout().bands) do for _, column in ipairs(band.columns) do for _, window in ipairs(column.windows) do if window.plugin_window then return window.window end end end end",
    );
    assert_eq!(layout, None);
}

#[test]
fn tiled_plugin_window_targets() {
    let client = Client::new("win-window-target", "");
    let outcome = client.run("win = gband.win.open({ kind = 'tiled', after = 1, focus = false })");
    let win: u32 = client.global("win");
    assert_eq!(
        outcome.dispatched,
        [Dispatch::PluginWindow(PluginWindowRequest::Open {
            plugin_window: win,
            target: Some((gband_core::layout::BandId(1), Some(WindowId(1)))),
            width: None,
            focus: false,
        })]
    );
}

#[test]
fn tiled_plugin_window_with_no_window_closes() {
    let client = Client::new("win-window-none", "");
    client.run("win = gband.win.open({ kind = 'tiled', on_close = function(id) closed = id end })");
    let win: u32 = client.global("win");
    let outcome = client.config.runtime.plugin_window_opened(win, None);
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert_eq!(client.global::<u32>("closed"), win);
}

#[test]
fn closing_plugin_windows() {
    let client = Client::new("win-close", "");
    client.run(
        "float = gband.win.open({})\n\
         window = gband.win.open({ kind = 'tiled', on_close = function(id) closed = id end })",
    );
    let (float, window): (u32, u32) = (client.global("float"), client.global("window"));
    client.frames();
    let outcome =
        client.run("gband.win.close(float)\ngband.win.close(window)\ngband.win.close(window)");
    assert_eq!(
        outcome.dispatched,
        [Dispatch::PluginWindow(PluginWindowRequest::Close {
            plugin_window: window
        })]
    );
    assert_eq!(client.global::<u32>("closed"), window);
    assert_eq!(client.frames(), [(float, None), (window, None)]);
    let focused: Option<u32> = client.eval("return gband.view().plugin_window");
    assert_eq!(focused, None);
}

#[test]
fn window_closed_by_the_user() {
    let client = Client::new("win-window-closed", "");
    client.run("win = gband.win.open({ kind = 'tiled', on_close = function(id) closed = id end })");
    let win: u32 = client.global("win");
    clean(
        &client
            .config
            .runtime
            .plugin_window_opened(win, Some(WindowId(5))),
    );
    let outcome = client.config.runtime.window_closed(win);
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert_eq!(client.global::<u32>("closed"), win);
}

#[test]
fn float_takes_focus_and_moving_focus_leaves_it() {
    let client = Client::new("win-focus", "");
    client.run("win = gband.win.open({})");
    let win: u32 = client.global("win");
    assert!(client.float().focused);
    let plugin_window: Option<u32> = client.eval("return gband.view().plugin_window");
    let window: Option<u32> = client.eval("return gband.view().window");
    assert_eq!((plugin_window, window), (Some(win), Some(1)));
    clean(&client.config.runtime.emit(&Event::FocusChanged {
        window: Some(WindowId(2)),
        previous: Some(WindowId(1)),
    }));
    assert!(!client.float().focused);
    assert!(!client.eval::<bool>("return gband.win.info(win).focused"));
}

#[test]
fn own_key_holds_the_float_focus_until_the_next_key() {
    let client = Client::new("win-hold", "");
    client.run(
        "win = gband.win.open({ keys = { enter = function() gband.action.focus_column_right() end } })",
    );
    let win: u32 = client.global("win");
    let outcome = client.key(win, "enter");
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusRight))]
    );
    let focus_changed = || {
        clean(&client.config.runtime.emit(&Event::FocusChanged {
            window: Some(WindowId(2)),
            previous: Some(WindowId(1)),
        }))
    };
    let focused = || client.eval::<bool>("return gband.win.info(win).focused");
    focus_changed();
    assert!(focused());
    clean(&client.config.runtime.release_plugin_windows());
    focus_changed();
    assert!(!focused());
}

#[test]
fn default_key_does_not_hold_the_float_focus() {
    let client = Client::new("win-no-hold", "");
    client.run("win = gband.win.open({ lines = { 'a', 'b' } })");
    let win: u32 = client.global("win");
    clean(&client.key(win, "down"));
    clean(&client.config.runtime.emit(&Event::BandChanged {
        band: gband_core::layout::BandId(2),
        previous: gband_core::layout::BandId(1),
    }));
    assert!(!client.eval::<bool>("return gband.win.info(win).focused"));
}

#[test]
fn focus_a_tiled_plugin_window() {
    let client = Client::new("win-focus-window", "");
    client.run("float = gband.win.open({})\nwindow = gband.win.open({ kind = 'tiled' })");
    let window: u32 = client.global("window");
    clean(
        &client
            .config
            .runtime
            .plugin_window_opened(window, Some(WindowId(1))),
    );
    let outcome = client.run("gband.win.focus(window)");
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusWindow(
            WindowId(1)
        )))]
    );
    let focused: Option<u32> = client.eval("return gband.view().plugin_window");
    assert_eq!(focused, Some(window));
    let plugin_window: Option<u32> = client.eval(
        "for _, column in ipairs(gband.layout().bands[1].columns) do for _, item in ipairs(column.windows) do return item.plugin_window end end",
    );
    assert_eq!(plugin_window, Some(window));
}

#[test]
fn later_float_stacks_on_top() {
    let client = Client::new("win-stack", "");
    client.run("first = gband.win.open({})\nsecond = gband.win.open({ focus = false })");
    let (first, second): (u32, u32) = (client.global("first"), client.global("second"));
    let z = |frames: Vec<(u32, Option<Frame>)>, id: u32| {
        frames
            .into_iter()
            .find_map(|(plugin_window, frame)| match frame {
                Some(Frame::Floating(frame)) if plugin_window == id => {
                    Some((frame.z, frame.focused))
                }
                _ => None,
            })
            .unwrap()
    };
    let frames = client.frames();
    let (first_z, first_focused) = z(frames.clone(), first);
    let (second_z, second_focused) = z(frames, second);
    assert!(second_z > first_z);
    assert!(first_focused && !second_focused);
    client.run("gband.win.focus(second)");
    let frames = client.frames();
    let (raised, focused) = z(frames, second);
    assert!(raised > second_z && focused);
}

#[test]
fn plugin_window_groups_have_defaults() {
    let client = Client::new("win-groups", "");
    let specs: Vec<String> = client.eval(
        "local out = {}\nfor _, name in ipairs({ 'PluginWindow', 'PluginWindowBorder', 'PluginWindowTitle', 'PluginWindowCursorLine' }) do\n  local spec = gband.hl.get(name)\n  local fields = {}\n  for field, value in pairs(spec) do fields[#fields + 1] = field .. '=' .. tostring(value) end\n  table.sort(fields)\n  out[#out + 1] = name .. ':' .. table.concat(fields, ',')\nend\nreturn out",
    );
    assert_eq!(
        specs,
        [
            "PluginWindow:",
            "PluginWindowBorder:fg=8",
            "PluginWindowTitle:bold=true",
            "PluginWindowCursorLine:reverse=true",
        ]
    );
}

#[test]
fn theme_overrides_the_border() {
    let client = Client::new(
        "win-theme",
        "gband.hl.set('PluginWindowBorder', { fg = '#ff0000' })",
    );
    client.run("gband.win.open({ title = 'T' })");
    let frame = client.float();
    assert_eq!(frame.border_style.fg, Some(Color::Rgb(0xff, 0, 0)));
    assert_eq!(frame.title_style.fg, Some(Color::Rgb(0xff, 0, 0)));
    assert!(frame.title_style.bold);
}

#[test]
fn highlight_change_redraws() {
    let client = Client::new("win-redraw", "");
    client.run("gband.win.open({ lines = { 'a' } })");
    client.frames();
    client.run("gband.hl.set('PluginWindow', { fg = 4 })");
    assert_eq!(client.float().base.fg, Some(Color::Index(4)));
}

#[test]
fn failed_plugin_closes_its_plugin_windows() {
    let scratch = Scratch::new("win-failed");
    scratch.client_plugin("spin",
        "gband.bind('alt+o', function() win = gband.win.open({ kind = 'tiled', on_close = function() closed = true end }) end)\n\
         gband.bind('alt+s', function() while true do end end)",
    );
    let config = scratch.load_with_budget(100_000).unwrap();
    clean(&config.runtime.set_state(state()));
    let root = &config.keymap["root"];
    let callback = |name: &str| {
        let chord = Chord::Key(key(name));
        match root.iter().find(|(bound, _)| *bound == chord) {
            Some((_, Binding::Callback(callback))) => *callback,
            _ => panic!("{name} is not bound"),
        }
    };
    clean(&config.runtime.call(callback("alt+o")));
    let win: u32 = global(&config, "win");
    let outcome = config.runtime.call(callback("alt+s"));
    assert!(!outcome.errors.is_empty());
    assert_eq!(
        outcome.dispatched,
        [Dispatch::PluginWindow(PluginWindowRequest::Close {
            plugin_window: win
        })]
    );
    assert!(eval::<Vec<u32>>(&config, "return gband.win.list()").is_empty());
    assert!(eval::<Option<bool>>(&config, "return closed").is_none());
}

fn guide_examples() -> Vec<String> {
    let guide = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/plugins.md"),
    )
    .unwrap();
    let start = guide.find("## Layout and view").unwrap();
    let end = guide.find("## Side bars").unwrap();
    guide[start..end]
        .split("```lua\n")
        .skip(1)
        .map(|block| block.split("```").next().unwrap().to_owned())
        .collect()
}

#[test]
fn guide_examples_run() {
    let examples = guide_examples();
    assert_eq!(examples.len(), 6);
    for (index, example) in examples.iter().enumerate() {
        let scratch = Scratch::new(&format!("win-guide-{index}"));
        scratch.write(example);
        let config = scratch.loaded();
        assert!(config.errors.is_empty(), "{:?}", config.errors);
        for focused in [1, 2] {
            let mut layout = layout();
            let band = layout.bands()[0].id;
            let second = layout.allocate_window();
            layout.open(
                second,
                band,
                Some(WindowId(1)),
                None,
                &LayoutOptions::default(),
            );
            clean(&config.runtime.set_state(ViewState {
                window: Some(focused),
                layout: Arc::new(layout),
                ..state()
            }));
            for (_, binding) in config.keymap.get("prefix").into_iter().flatten() {
                let Binding::Callback(callback) = binding else {
                    continue;
                };
                let outcome = config.runtime.call(*callback);
                clean(&outcome);
            }
            clean(&config.runtime.emit(&Event::WindowOpened {
                window: second,
                band,
            }));
        }
    }
}

#[test]
fn click_a_line_runs_on_mouse() {
    let client = Client::new("win-on-mouse", "");
    client.run(&format!(
        "win = gband.win.open({{ height = 12, lines = {}, on_mouse = function(id, e) got = {{ id = id, kind = e.kind, button = e.button, line = e.line, col = e.content_col }} end }})\n\
         gband.win.scroll(win, 4)",
        numbered(25)
    ));
    let win: u32 = client.global("win");
    assert_eq!(client.eval::<u32>("return gband.win.info(win).top"), 5);
    clean(&client.mouse(win, PluginMouseKind::Press(MouseButton::Left), 2));
    let got: String = client
        .eval("return table.concat({ got.id, got.kind, got.button, got.line, got.col }, ',')");
    assert_eq!(got, format!("{win},press,left,7,1"));
    assert_eq!(client.eval::<u32>("return gband.win.info(win).cursor"), 1);
    clean(&client.mouse(win, PluginMouseKind::Scroll(WheelDirection::Down), 2));
    let got: String = client.eval(
        "return table.concat({ got.kind, gband.win.info(win).top, tostring(got.button) }, ',')",
    );
    assert_eq!(got, "scroll,5,nil");
}

#[test]
fn on_mouse_receives_the_box_cell() {
    let client = Client::new("win-on-mouse-box", "");
    client.run(
        "log = {}
win = gband.win.open({ width = 20, height = 10, on_mouse = function(_, e)
  log[#log + 1] = table.concat({ e.kind, tostring(e.box_col), tostring(e.box_row), tostring(e.box_width), tostring(e.box_height) }, ',')
end })",
    );
    let win: u32 = client.global("win");
    for (kind, boxed) in [
        (
            PluginMouseKind::Press(MouseButton::Left),
            Some(BoxCell {
                col: 19,
                row: 0,
                width: 20,
                height: 10,
            }),
        ),
        (PluginMouseKind::Drag(MouseButton::Left), None),
    ] {
        clean(&client.config.runtime.plugin_window_mouse(
            win,
            &PluginMouse {
                kind,
                content: None,
                boxed,
                modifiers: Modifiers::NONE,
            },
        ));
    }
    assert_eq!(
        client.global::<Vec<String>>("log"),
        ["press,19,0,20,10", "drag,nil,nil,nil,nil"]
    );
}

#[test]
fn wheel_scrolls_without_on_mouse() {
    let client = Client::new("win-wheel", "");
    client.run(&format!(
        "win = gband.win.open({{ height = 12, focus = false, lines = {} }})",
        numbered(25)
    ));
    let win: u32 = client.global("win");
    clean(&client.mouse(win, PluginMouseKind::Scroll(WheelDirection::Down), 0));
    assert_eq!(client.eval::<u32>("return gband.win.info(win).top"), 2);
    assert!(!client.eval::<bool>("return gband.win.info(win).focused"));
    assert_eq!(trimmed(&client.float().lines)[..2], ["2", "3"]);
    clean(&client.mouse(win, PluginMouseKind::Scroll(WheelDirection::Up), 0));
    assert_eq!(client.eval::<u32>("return gband.win.info(win).top"), 1);
}

#[test]
fn click_moves_the_cursor_line() {
    let client = Client::new("win-click", "");
    client.run(&format!(
        "win = gband.win.open({{ height = 12, focus = false, cursorline = true, lines = {} }})",
        numbered(10)
    ));
    let win: u32 = client.global("win");
    clean(&client.mouse(win, PluginMouseKind::Press(MouseButton::Left), 3));
    assert_eq!(client.eval::<u32>("return gband.win.info(win).cursor"), 4);
    assert!(client.eval::<bool>("return gband.win.info(win).focused"));
    clean(&client.mouse(win, PluginMouseKind::Press(MouseButton::Right), 5));
    assert_eq!(client.eval::<u32>("return gband.win.info(win).cursor"), 4);
    clean(&client.mouse(win, PluginMouseKind::Press(MouseButton::Left), 9));
    assert_eq!(client.eval::<u32>("return gband.win.info(win).cursor"), 10);
}

#[test]
fn set_box_moves_and_resizes_in_one_step() {
    let client = Client::new("win-set-box", "resized = 0\n");
    client.run(
        "win = gband.win.open({ width = 20, height = 10, col = 5, row = 3, on_resize = function() resized = resized + 1 end })",
    );
    let win: u32 = client.global("win");
    let placed = PluginBox {
        col: 9,
        row: 5,
        width: 20,
        height: 10,
    };
    clean(&client.config.runtime.set_plugin_window_box(win, placed));
    let info = || -> String {
        client.eval(
            "local i = gband.win.info(win) return table.concat({ i.col, i.row, i.width, i.height, resized }, ',')",
        )
    };
    assert_eq!(info(), "9,5,20,10,0");
    let resized = PluginBox {
        width: 30,
        height: 12,
        ..placed
    };
    clean(&client.config.runtime.set_plugin_window_box(win, resized));
    assert_eq!(info(), "9,5,30,12,1");
    let beyond = PluginBox { col: 75, ..resized };
    clean(&client.config.runtime.set_plugin_window_box(win, beyond));
    assert_eq!(info(), "50,5,30,12,1");
}

#[test]
fn display_width() {
    let scratch = Scratch::new("display-width");
    scratch.write("cells = gband.ui.width('a日b')");
    let config = scratch.loaded();
    assert_eq!(global::<i64>(&config, "cells"), 4);
}

#[test]
fn truncate_at_a_wide_character() {
    let scratch = Scratch::new("truncate-wide");
    scratch.write("cut = gband.ui.truncate('日本語', 4)");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "cut"), "日…");
}

#[test]
fn focus_before_the_layout_holds_the_window() {
    let client = Client::new("win-focus-early", "");
    client.run("win = gband.win.open({ kind = 'tiled', focus = false })");
    let win: u32 = client.global("win");
    clean(
        &client
            .config
            .runtime
            .plugin_window_opened(win, Some(WindowId(5))),
    );
    let outcome = run_job(&client.config, "gband.win.focus(win)");
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusWindow(
            WindowId(5)
        )))]
    );
}

#[test]
fn reloads_keep_plugin_windows_drawn() {
    let scratch = Scratch::new("reload-plugin-window");
    scratch.write(JOB);
    for _ in 0..2 {
        let config = scratch.loaded();
        clean(&config.runtime.set_state(state()));
        clean(&run_job(&config, "gband.win.open({ lines = { 'hi' } })"));
        let frames = config.runtime.take_frames();
        assert!(
            matches!(frames.as_slice(), [(_, Some(Frame::Floating(_)))]),
            "{frames:?}"
        );
    }
}
