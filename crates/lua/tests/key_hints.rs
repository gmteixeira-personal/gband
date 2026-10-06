mod common;

use common::*;
use gband_lua::{Color, Config, StatusLine, Style, ViewState};

const HINTS: &str = "gband.plugin('gband.statusline.hints')";

fn loaded(name: &str, source: &str) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    scratch.write(source);
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    (scratch, config)
}

fn defaults_with(name: &str, opts: &str) -> (Scratch, Config) {
    let source = gband_lua::DEFAULTS.replace(
        "gband.plugin(\"gband.statusline.hints\")",
        &format!("gband.plugin(\"gband.statusline.hints\", {opts})"),
    );
    loaded(name, &source)
}

fn state(table: &str, width: u16) -> ViewState {
    let mut state = drawn(width);
    state.table = table.to_owned();
    state
}

fn shown(config: &Config, table: &str, width: u16) -> String {
    let line = presented(config, state(table, width));
    text(&line, width).trim_end().to_owned()
}

fn style_at(line: &StatusLine, col: u16) -> Style {
    line.spans
        .iter()
        .find(|span| span.col == col)
        .map(|span| span.style)
        .unwrap_or_else(|| panic!("no span at {col}: {line:?}"))
}

fn resolved(config: &Config, group: &str) -> Vec<String> {
    eval(
        config,
        &format!(
            "local s = gband.hl.get('{group}', {{ resolve = true }})\nlocal out = {{}} for k, v in pairs(s) do out[#out + 1] = k .. '=' .. tostring(v) end\ntable.sort(out)\nreturn out"
        ),
    )
}

const PREFIX_HINTS: &str = "h left  l right  j down  k up  u band down  i band up  c center  enter new  q close  [ stack left  ] stack right  r width  f full  - narrower  = wider  _ shorter  + taller  R reset height  v float  V layer  C-h move left  C-l move right  C-j move down  C-k move up  C-left move left  C-right move right  C-down move down  C-up move up  D detach  C-space send prefix";

#[test]
fn component_entry() {
    let (_scratch, config) = loaded("entry", HINTS);
    let entry: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[1]\nreturn { e.id, e.align, tostring(e.priority), tostring(e.order), e.hl, e.plugin, tostring(e.fill) }",
    );
    assert_eq!(
        entry,
        ["hints", "left", "0", "30", "KeyHintLabel", "hints", "true"]
    );
}

#[test]
fn options_replace_the_settings() {
    let (_scratch, config) = loaded(
        "options",
        "gband.plugin('gband.statusline.hints', { align = 'right', priority = 4, order = 2 })",
    );
    let entry: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[1]\nreturn { e.align, tostring(e.priority), tostring(e.order) }",
    );
    assert_eq!(entry, ["right", "4", "2"]);
}

#[test]
fn invalid_option() {
    for (opts, mentions) in [
        ("{ labels = 3 }", "labels"),
        ("{ labels = { close_window = 3 } }", "labels"),
        ("{ labels = { close_window = true } }", "labels"),
        ("{ labels = { 'kill' } }", "labels"),
        ("{ align = 'middle' }", "align"),
        ("{ priority = 'high' }", "priority"),
        ("{ order = {} }", "order"),
        ("{ root = 'no' }", "root"),
        ("'left'", "hints"),
    ] {
        let scratch = Scratch::new("invalid");
        scratch.write(&format!(
            "ok = gband.plugin('gband.statusline.hints', {opts})"
        ));
        let config = scratch.loaded();
        assert!(!global::<bool>(&config, "ok"), "{opts}");
        let error = config.errors.first().expect("a plugin error");
        assert_eq!(error.plugin.as_deref(), Some("hints"), "{opts}");
        assert!(error.message.contains(mentions), "{opts}: {error}");
    }
}

#[test]
fn linked_defaults() {
    let (_scratch, config) = loaded("linked", HINTS);
    let links: Vec<String> = eval(
        &config,
        "return { gband.hl.get('KeyHintKey').link, gband.hl.get('KeyHintLabel').link }",
    );
    assert_eq!(links, ["StatusLineAccent", "StatusLineSegment"]);
    assert_eq!(
        resolved(&config, "KeyHintKey"),
        resolved(&config, "StatusLineAccent")
    );
}

#[test]
fn theme_override() {
    let scratch = Scratch::new("theme");
    scratch.user_file(
        "colors/dusk.lua",
        "gband.hl.set('KeyHintKey', { fg = 'yellow' })",
    );
    scratch.write(&format!(
        "gband.colorscheme('dusk')\ngband.keymap.set('prefix', 'h', gband.action.focus_column_left)\n{HINTS}"
    ));
    let config = scratch.loaded();
    let line = presented(&config, state("root", 40));
    assert_eq!(style_at(&line, 0).fg, Some(Color::Index(3)));
}

fn keys(name: &str, bindings: &[&str]) -> (Scratch, Config) {
    let mut source = String::new();
    for key in bindings {
        source.push_str(&format!(
            "gband.keymap.set('keys', '{key}', function() end, {{ desc = 'x' }})\n"
        ));
    }
    source.push_str(HINTS);
    loaded(name, &source)
}

#[test]
fn key_form() {
    for (written, form) in [
        ("ctrl+space", "C-space"),
        ("shift+d", "D"),
        ("Alt+PageUp", "A-pageup"),
        ("alt++", "A-+"),
        ("+", "+"),
        ("escape", "esc"),
        ("ctrl+alt+x", "C-A-x"),
        ("shift+alt+ctrl+tab", "C-A-S-tab"),
        ("R", "R"),
        ("[", "["),
    ] {
        let (_scratch, config) = keys("key-form", &[written]);
        assert_eq!(shown(&config, "keys", 40), format!("{form} x"), "{written}");
    }
}

#[test]
fn prefix_in_a_named_table() {
    let (_scratch, config) = loaded(
        "prefix-named",
        &format!(
            "gband.opt.prefix = 'ctrl+b'\ngband.keymap.set('move', 'prefix', function() end, {{ desc = 'back' }})\n{HINTS}"
        ),
    );
    assert_eq!(shown(&config, "move", 40), "C-b back");
}

#[test]
fn prefix_hint_follows_the_option() {
    let (_scratch, config) = loaded(
        "prefix-option",
        &format!(
            "gband.opt.prefix = 'ctrl+b'\ngband.keymap.set('prefix', 'h', gband.action.focus_column_left)\n{HINTS}"
        ),
    );
    assert_eq!(shown(&config, "root", 40), "C-b prefix");
}

#[test]
fn root_with_the_defaults() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    assert_eq!(shown(&config, "root", 80), "band 1 │ C-space prefix");
}

#[test]
fn prefix_table_with_the_defaults() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    assert_eq!(
        shown(&config, "prefix", 400),
        format!("band 1 │ prefix │ {PREFIX_HINTS}")
    );
}

#[test]
fn named_table() {
    let (_scratch, config) = loaded(
        "named",
        &format!(
            "gband.keymap.set('move', 'h', function() end, {{ desc = 'west' }})\ngband.keymap.set('move', 'l', function() end, {{ desc = 'east' }})\n{HINTS}"
        ),
    );
    assert_eq!(shown(&config, "move", 40), "h west  l east");
}

#[test]
fn no_prefix_bindings() {
    let (_scratch, config) = loaded(
        "no-prefix",
        &format!("gband.keymap.set('root', 'alt+h', gband.action.focus_column_left)\n{HINTS}"),
    );
    assert_eq!(shown(&config, "root", 40), "A-h left");
}

#[test]
fn root_hints_turned_off() {
    let (_scratch, config) = defaults_with("root-off", "{ root = false }");
    assert_eq!(shown(&config, "root", 80), "band 1");
    let shown = shown(&config, "prefix", 80);
    assert!(
        shown.starts_with("band 1 │ prefix │ h left  l right"),
        "{shown}"
    );
}

#[test]
fn groups() {
    let (_scratch, config) = loaded(
        "groups",
        &format!(
            "gband.keymap.set('prefix', 'h', gband.action.focus_column_left)\n{HINTS}\ngband.hl.set('KeyHintKey', {{ fg = 1 }})\ngband.hl.set('KeyHintLabel', {{ fg = 2 }})"
        ),
    );
    let line = presented(&config, state("root", 40));
    assert_eq!(text(&line, 40).trim_end(), "C-space prefix");
    let spans: Vec<(&str, Option<Color>)> = line
        .spans
        .iter()
        .map(|span| (span.text.as_str(), span.style.fg))
        .collect();
    assert_eq!(
        spans,
        [
            ("C-space", Some(Color::Index(1))),
            (" prefix", Some(Color::Index(2)))
        ]
    );
}

#[test]
fn separators_and_ellipsis_in_the_label_group() {
    let (_scratch, config) = loaded(
        "separator-group",
        &format!(
            "gband.keymap.set('move', 'h', function() end, {{ desc = 'west' }})\ngband.keymap.set('move', 'l', function() end, {{ desc = 'east' }})\ngband.keymap.set('move', 'j', function() end, {{ desc = 'down' }})\n{HINTS}\ngband.hl.set('KeyHintLabel', {{ fg = 2 }})"
        ),
    );
    let line = presented(&config, state("move", 16));
    assert_eq!(text(&line, 16).trim_end(), "h west  l east …");
    assert_eq!(style_at(&line, 6).fg, Some(Color::Index(2)));
    assert_eq!(style_at(&line, 14).fg, Some(Color::Index(2)));
}

#[test]
fn default_description_gives_the_short_label() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    assert!(shown(&config, "prefix", 400).contains("  r width  "));
}

#[test]
fn center_label() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    assert!(shown(&config, "prefix", 400).contains("  c center  "));
}

#[test]
fn own_description() {
    let (_scratch, config) = loaded(
        "own",
        &format!(
            "gband.keymap.set('prefix', 'g', gband.action.focus_column_left, {{ desc = 'go west' }})\n{HINTS}"
        ),
    );
    assert_eq!(shown(&config, "prefix", 40), "g go west");
}

#[test]
fn floating_keys() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    let shown = shown(&config, "prefix", 400);
    for hint in ["v float", "V layer", "C-h move left", "C-left move left"] {
        assert!(shown.contains(&format!("  {hint}  ")), "{hint}: {shown}");
    }
}

#[test]
fn label_option() {
    let (_scratch, config) = defaults_with("label", "{ labels = { close_window = 'kill' } }");
    let shown = shown(&config, "prefix", 400);
    assert!(shown.contains("  q kill  "), "{shown}");
}

#[test]
fn label_option_hides_an_action() {
    let (_scratch, config) = defaults_with("label-hide", "{ labels = { send_prefix = false } }");
    let shown = shown(&config, "prefix", 400);
    assert!(!shown.contains("send prefix"), "{shown}");
    assert!(shown.ends_with("D detach"), "{shown}");
}

fn registered(name: &str, opts: &str) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    scratch.client_plugin("hello",
        &format!(
            "gband.action.register('greet', function() end{opts})\ngband.keymap.set('root', 'alt+g', gband.action['hello.greet'])"
        ),
    );
    scratch.write(&format!(
        "gband.keymap.set('prefix', 'h', gband.action.focus_column_left)\n{HINTS}"
    ));
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    (scratch, config)
}

#[test]
fn registered_action() {
    let (_scratch, config) = registered("registered", ", { desc = 'say hi' }");
    assert_eq!(shown(&config, "root", 40), "C-space prefix  A-g say hi");
}

#[test]
fn registered_action_without_a_description() {
    let (_scratch, config) = registered("registered-bare", "");
    assert_eq!(
        shown(&config, "root", 40),
        "C-space prefix  A-g hello.greet"
    );
}

#[test]
fn function_without_a_description() {
    let (_scratch, config) = loaded(
        "function-bare",
        &format!(
            "gband.keymap.set('prefix', 'x', function() end)\ngband.keymap.set('prefix', 'y', function() end, {{ desc = '' }})\ngband.keymap.set('prefix', 'h', gband.action.focus_column_left)\n{HINTS}"
        ),
    );
    assert_eq!(shown(&config, "prefix", 40), "h left");
}

#[test]
fn some_hints_left_out() {
    let (_scratch, config) = loaded(
        "left-out",
        &gband_lua::DEFAULTS
            .replace("gband.plugin(\"gband.statusline.band\")", "")
            .replace("gband.plugin(\"gband.statusline.mode\")", "")
            .replace("gband.plugin(\"gband.statusline.position\")", ""),
    );
    assert_eq!(shown(&config, "prefix", 20), "h left  l right …");
}

#[test]
fn all_hints_fit() {
    let (_scratch, config) = loaded(
        "all-fit",
        &format!(
            "gband.keymap.set('move', 'h', function() end, {{ desc = 'west' }})\ngband.keymap.set('move', 'l', function() end, {{ desc = 'east' }})\n{HINTS}"
        ),
    );
    assert_eq!(shown(&config, "move", 14), "h west  l east");
    assert_eq!(shown(&config, "move", 13), "h west …");
}

#[test]
fn nothing_fits() {
    let (_scratch, config) = loaded(
        "nothing",
        &format!("gband.keymap.set('prefix', 'h', gband.action.focus_column_left)\n{HINTS}"),
    );
    let line = presented(&config, state("root", 3));
    assert!(line.spans.is_empty(), "{line:?}");
}

#[test]
fn wide_character_hint() {
    let (_scratch, config) = loaded(
        "wide",
        &format!(
            "gband.keymap.set('move', 'h', function() end, {{ desc = '日本' }})\ngband.keymap.set('move', 'l', function() end, {{ desc = 'east' }})\n{HINTS}"
        ),
    );
    assert_eq!(shown(&config, "move", 8), "h 日本 …");
    let line = presented(&config, state("move", 7));
    assert!(line.spans.is_empty(), "{line:?}");
}
