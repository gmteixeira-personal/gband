mod common;

use common::*;
use gband_lua::{Bar, Color, Config, ViewState};

const HINTS: &str = "gband.plugin('gband.statusline', { min_width = 40, max_width = 40 })\ngband.plugin('gband.statusline.hints')";

fn hints_at(width: u16) -> String {
    format!(
        "gband.plugin('gband.statusline', {{ min_width = {width}, max_width = {width} }})\ngband.plugin('gband.statusline.hints')"
    )
}

fn loaded(name: &str, source: &str) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    scratch.write(source);
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    (scratch, config)
}

fn defaults_with(name: &str, opts: &str) -> (Scratch, Config) {
    let source = gband_lua::DEFAULTS
        .replace(
            "gband.plugin(\"gband.statusline.hints\")",
            &format!("gband.plugin(\"gband.statusline.hints\", {opts})"),
        )
        .replace(
            "gband.plugin(\"gband.statusline\")",
            "gband.plugin(\"gband.statusline\", { min_width = 40 })",
        );
    loaded(name, &source)
}

fn state(table: &str, rows: u16) -> ViewState {
    let mut state = sized(200, rows);
    state.table = table.to_owned();
    state
}

fn bar(config: &Config, table: &str, rows: u16) -> Bar {
    presented(config, state(table, rows))
}

fn lines(config: &Config, table: &str, rows: u16) -> Vec<String> {
    shown_rows(&bar(config, table, rows))
        .into_iter()
        .map(|(_, text)| text)
        .collect()
}

fn shown(config: &Config, table: &str) -> String {
    lines(config, table, 24).join("\n")
}

fn hints(config: &Config, table: &str) -> Vec<String> {
    lines(config, table, 80)
        .iter()
        .flat_map(|row| row.split("  "))
        .map(str::to_owned)
        .collect()
}

fn resolved(config: &Config, group: &str) -> Vec<String> {
    eval(
        config,
        &format!(
            "local s = gband.hl.get('{group}', {{ resolve = true }})\nlocal out = {{}} for k, v in pairs(s) do out[#out + 1] = k .. '=' .. tostring(v) end\ntable.sort(out)\nreturn out"
        ),
    )
}

const PREFIX_HINTS: &str = "h left  l right  j down  k up  u band down  i band up  c center  n open a window  q close  [ stack left  ] stack right  r width  f full  - narrower  = wider  _ shorter  + taller  R reset height  v float  V layer  C-h move left  C-l move right  C-j move down  C-k move up  C-left move left  C-right move right  C-down move down  C-up move up  ? list the keys  : run Lua  D detach  esc interactive mode  enter interactive mode  left left  right right  down down  up up  C-space send the prefix key";

#[test]
fn component_entry() {
    let (_scratch, config) = loaded("entry", HINTS);
    let entry: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[1]\nreturn { e.id, e.align, tostring(e.priority), tostring(e.order), e.hl, e.plugin, tostring(e.fill) }",
    );
    assert_eq!(
        entry,
        ["hints", "top", "0", "30", "KeyHintLabel", "hints", "true"]
    );
}

#[test]
fn options_replace_the_settings() {
    let (_scratch, config) = loaded(
        "options",
        "gband.plugin('gband.statusline.hints', { align = 'bottom', priority = 4, order = 2 })",
    );
    let entry: Vec<String> = eval(
        &config,
        "local e = gband.ui.statusline.list()[1]\nreturn { e.align, tostring(e.priority), tostring(e.order) }",
    );
    assert_eq!(entry, ["bottom", "4", "2"]);
}

#[test]
fn invalid_option() {
    for (opts, mentions) in [
        ("{ labels = 3 }", "labels"),
        ("{ labels = { close_window = 3 } }", "labels"),
        ("{ labels = { close_window = true } }", "labels"),
        ("{ labels = { 'kill' } }", "labels"),
        ("{ align = 'middle' }", "align"),
        ("{ align = 'left' }", "align"),
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
    let bar = bar(&config, "root", 24);
    assert_eq!(bar.lines[0][0].style.fg, Some(Color::Index(3)));
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
        assert_eq!(shown(&config, "keys"), format!("{form} x"), "{written}");
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
    assert_eq!(shown(&config, "move"), "C-b back");
}

#[test]
fn prefix_hint_follows_the_option() {
    let (_scratch, config) = loaded(
        "prefix-option",
        &format!(
            "gband.opt.prefix = 'ctrl+b'\ngband.keymap.set('prefix', 'h', gband.action.focus_column_left)\n{HINTS}"
        ),
    );
    assert_eq!(shown(&config, "root"), "C-b prefix");
}

#[test]
fn root_with_the_defaults() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    assert_eq!(shown(&config, "root"), "band 1\nC-space navigation");
}

#[test]
fn prefix_table_that_is_not_a_mode() {
    let (_scratch, config) = loaded(
        "prefix-not-a-mode",
        &format!("gband.keymap.set('prefix', 'h', gband.action.focus_column_left)\n{HINTS}"),
    );
    assert_eq!(shown(&config, "root"), "C-space prefix");
}

#[test]
fn prefix_table_with_the_defaults() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    let first = lines(&config, "prefix", 40);
    assert_eq!(
        first[..6],
        [
            "band 1",
            "navigation",
            "h left  l right",
            "j down  k up",
            "u band down",
            "i band up  c center"
        ]
    );
    let all: Vec<&str> = PREFIX_HINTS.split("  ").collect();
    let prompt = all.iter().position(|hint| *hint == ": run Lua").unwrap();
    let hints = hints(&config, "prefix");
    assert_eq!(hints[..2], ["band 1", "navigation"]);
    assert_eq!(hints[2..hints.len() - 1], all[..prompt]);
    assert_eq!(hints.last().unwrap(), ": run Lua …");
    let (_scratch, wide) = defaults_with("prefix-wide", "{}");
    let hints = self::hints(&wide, "prefix");
    assert_eq!(hints[2..], all);
}

#[test]
fn named_table() {
    let (_scratch, config) = loaded(
        "named",
        &format!(
            "gband.keymap.set('move', 'h', function() end, {{ desc = 'west' }})\ngband.keymap.set('move', 'l', function() end, {{ desc = 'east' }})\n{HINTS}"
        ),
    );
    assert_eq!(shown(&config, "move"), "h west  l east");
}

#[test]
fn no_prefix_bindings() {
    let (_scratch, config) = loaded(
        "no-prefix",
        &format!("gband.keymap.set('root', 'alt+h', gband.action.focus_column_left)\n{HINTS}"),
    );
    assert_eq!(shown(&config, "root"), "A-h left");
}

#[test]
fn root_hints_turned_off() {
    let (_scratch, config) = defaults_with("root-off", "{ root = false }");
    assert_eq!(shown(&config, "root"), "band 1");
    let lines = lines(&config, "prefix", 24);
    assert_eq!(
        lines[..3],
        ["band 1", "navigation", "h left  l right  j down  k up"]
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
    let bar = bar(&config, "root", 24);
    let spans: Vec<(&str, Option<Color>)> = bar.lines[0]
        .iter()
        .map(|run| (run.text.as_str(), run.style.fg))
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
            "gband.keymap.set('move', 'h', function() end, {{ desc = 'west' }})\ngband.keymap.set('move', 'l', function() end, {{ desc = 'east' }})\ngband.keymap.set('move', 'j', function() end, {{ desc = 'down' }})\n{}\ngband.hl.set('KeyHintLabel', {{ fg = 2 }})",
            hints_at(16)
        ),
    );
    let bar = bar(&config, "move", 1);
    assert_eq!(rows(&bar), ["h west  l east …"]);
    let line = &bar.lines[0];
    assert_eq!(line[2].text, "  ");
    assert_eq!(line[2].style.fg, Some(Color::Index(2)));
    assert_eq!(line[5].text, " …");
    assert_eq!(line[5].style.fg, Some(Color::Index(2)));
}

#[test]
fn default_description_gives_the_short_label() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    assert!(hints(&config, "prefix").contains(&"r width".to_owned()));
}

#[test]
fn center_label() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    assert!(hints(&config, "prefix").contains(&"c center".to_owned()));
}

#[test]
fn own_description() {
    let (_scratch, config) = loaded(
        "own",
        &format!(
            "gband.keymap.set('prefix', 'g', gband.action.focus_column_left, {{ desc = 'go west' }})\n{HINTS}"
        ),
    );
    assert_eq!(shown(&config, "prefix"), "g go west");
}

#[test]
fn floating_keys() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    let hints = hints(&config, "prefix");
    for hint in ["v float", "V layer", "C-h move left", "C-left move left"] {
        assert!(hints.contains(&hint.to_owned()), "{hint}: {hints:?}");
    }
}

#[test]
fn label_option() {
    let (_scratch, config) = defaults_with("label", "{ labels = { close_window = 'kill' } }");
    assert!(hints(&config, "prefix").contains(&"q kill".to_owned()));
}

#[test]
fn label_option_hides_an_action() {
    let (_scratch, config) = defaults_with("label-hide", "{ labels = { detach = false } }");
    let hints = hints(&config, "prefix");
    assert!(
        !hints.iter().any(|hint| hint.contains("detach")),
        "{hints:?}"
    );
    let list = hints
        .iter()
        .position(|hint| hint == "? list the keys")
        .unwrap();
    assert_eq!(hints[list + 1], ": run Lua");
    assert_eq!(hints[list + 2], "esc interactive mode");
    assert_eq!(hints.last().unwrap(), "C-space send the prefix key");
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
    assert_eq!(shown(&config, "root"), "C-space prefix  A-g say hi");
}

#[test]
fn registered_action_without_a_description() {
    let (_scratch, config) = registered("registered-bare", "");
    assert_eq!(shown(&config, "root"), "C-space prefix  A-g hello.greet");
}

#[test]
fn function_without_a_description() {
    let (_scratch, config) = loaded(
        "function-bare",
        &format!(
            "gband.keymap.set('prefix', 'x', function() end)\ngband.keymap.set('prefix', 'y', function() end, {{ desc = '' }})\ngband.keymap.set('prefix', 'h', gband.action.focus_column_left)\n{HINTS}"
        ),
    );
    assert_eq!(shown(&config, "prefix"), "h left");
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
    assert_eq!(
        lines(&config, "prefix", 2),
        ["h left  l right", "j down  k up …"]
    );
}

#[test]
fn all_hints_fit() {
    let source = |width| {
        format!(
            "gband.keymap.set('move', 'h', function() end, {{ desc = 'west' }})\ngband.keymap.set('move', 'l', function() end, {{ desc = 'east' }})\n{}",
            hints_at(width)
        )
    };
    let (_scratch, config) = loaded("all-fit", &source(14));
    assert_eq!(lines(&config, "move", 1), ["h west  l east"]);
    let (_scratch, config) = loaded("all-fit-narrow", &source(13));
    assert_eq!(lines(&config, "move", 1), ["h west …"]);
    assert_eq!(lines(&config, "move", 2), ["h west", "l east"]);
}

#[test]
fn last_line_makes_room_for_the_ellipsis() {
    let (_scratch, config) = loaded(
        "ellipsis-room",
        &format!(
            "gband.keymap.set('move', 'h', function() end, {{ desc = 'wester' }})\ngband.keymap.set('move', 'l', function() end, {{ desc = 'east' }})\ngband.keymap.set('move', 'j', function() end, {{ desc = 'down' }})\n{}",
            hints_at(16)
        ),
    );
    assert_eq!(lines(&config, "move", 1), ["h wester …"]);
}

#[test]
fn nothing_fits() {
    let (_scratch, config) = loaded(
        "nothing",
        &format!(
            "gband.keymap.set('prefix', 'h', gband.action.focus_column_left)\n{}",
            hints_at(3)
        ),
    );
    assert!(lines(&config, "root", 24).is_empty());
}

#[test]
fn no_rows_left() {
    let (_scratch, config) = loaded(
        "no-rows",
        &format!(
            "gband.keymap.set('prefix', 'h', gband.action.focus_column_left)\ngband.ui.statusline.add({{ id = 'a', order = 1, priority = 5, render = function(ctx) return 'a' end }})\ngband.ui.statusline.add({{ id = 'probe', fill = true, priority = -1, render = function(ctx) seen = ctx.height end }})\n{HINTS}"
        ),
    );
    assert_eq!(lines(&config, "root", 1), ["a"]);
    assert_eq!(global::<i64>(&config, "seen"), 0);
}

#[test]
fn wide_character_hint() {
    let source = |width| {
        format!(
            "gband.keymap.set('move', 'h', function() end, {{ desc = '日本' }})\ngband.keymap.set('move', 'l', function() end, {{ desc = 'east' }})\n{}",
            hints_at(width)
        )
    };
    let (_scratch, config) = loaded("wide", &source(8));
    assert_eq!(lines(&config, "move", 1), ["h 日本 …"]);
    let (_scratch, config) = loaded("wide-narrow", &source(7));
    assert!(lines(&config, "move", 1).is_empty());
}
