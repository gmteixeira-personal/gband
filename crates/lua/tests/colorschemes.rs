mod common;

use common::*;
use gband_lua::Config;

const DUSK: &str = "gband.hl.set('StatusLine', { fg = '#112233' })\nran = (ran or '') .. 'dusk'\n";

fn resolved_fg(config: &Config, name: &str) -> Option<String> {
    eval(
        config,
        &format!(
            "local fg = gband.hl.get('{name}', {{ resolve = true }}).fg\nreturn fg and tostring(fg)"
        ),
    )
}

#[test]
fn user_colorscheme() {
    let scratch = Scratch::new("user");
    scratch.user_file("colors/dusk.lua", DUSK);
    scratch.write("ok = gband.colorscheme('dusk')");
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "ok"));
    assert_eq!(global::<String>(&config, "ran"), "dusk");
    assert_eq!(
        resolved_fg(&config, "StatusLine").as_deref(),
        Some("#112233")
    );
}

#[test]
fn earlier_entry_wins() {
    let scratch = Scratch::new("earlier");
    scratch.user_file("colors/dusk.lua", "ran = 'user'");
    scratch.plugin_file("theme", "colors/dusk.lua", "ran = 'plugin'");
    scratch.write("gband.colorscheme('dusk')");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "ran"), "user");
}

#[test]
fn plugin_directory_colorscheme() {
    let scratch = Scratch::new("plugin-dir");
    scratch.plugin_file("theme", "colors/dusk.lua", "ran = 'plugin'");
    scratch.write("gband.colorscheme('dusk')");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "ran"), "plugin");
}

#[test]
fn bundled_fallback() {
    let scratch = Scratch::new("bundled");
    scratch.user_file("colors/plain.lua", "");
    scratch.write("gband.colorscheme('plain')\nok = gband.colorscheme('default')");
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "ok"));
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "default"
    );
    assert_eq!(
        resolved_fg(&config, "StatusLine").as_deref(),
        Some("#c0caf5")
    );
}

#[test]
fn bundled_default_sets_every_built_in_group() {
    let scratch = Scratch::new("bundled-groups");
    let config = scratch.loaded();
    for group in [
        "StatusLine",
        "StatusLineSegment",
        "StatusLineSeparator",
        "StatusLineMuted",
        "StatusLineAccent",
        "StatusLineError",
    ] {
        let set: bool = eval(&config, &format!("return gband.hl.get('{group}') ~= nil"));
        assert!(set, "{group}");
    }
    let hex: bool = eval(
        &config,
        "local fg = gband.hl.get('StatusLine').fg return type(fg) == 'string' and fg:sub(1, 1) == '#'",
    );
    assert!(hex);
}

#[test]
fn shadowing_the_bundled_colorscheme() {
    let scratch = Scratch::new("shadow");
    scratch.user_file("colors/default.lua", "ran = (ran or 0) + 1");
    scratch.write("gband.colorscheme('default')");
    let config = scratch.loaded();
    assert_eq!(global::<i64>(&config, "ran"), 2);
}

#[test]
fn explicit_settings_replaced() {
    let scratch = Scratch::new("replaced");
    scratch.user_file("colors/dusk.lua", "gband.hl.set('StatusLine', { fg = 1 })");
    scratch.write("gband.hl.set('Title', { fg = 1 })\ngband.colorscheme('dusk')");
    let config = scratch.loaded();
    let title: Option<mlua::Table> = eval(&config, "return gband.hl.get('Title')");
    assert!(title.is_none());
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "dusk"
    );
}

#[test]
fn user_settings_after_the_colorscheme() {
    let scratch = Scratch::new("after");
    scratch.user_file("colors/dusk.lua", DUSK);
    scratch.write("gband.colorscheme('dusk')\ngband.hl.set('StatusLineAccent', { fg = 2 })");
    let config = scratch.loaded();
    assert_eq!(
        resolved_fg(&config, "StatusLineAccent").as_deref(),
        Some("2")
    );
    let bold: Option<bool> = eval(
        &config,
        "return gband.hl.get('StatusLineAccent', { resolve = true }).bold",
    );
    assert_eq!(bold, None);
}

#[test]
fn plugin_default_kept() {
    let scratch = Scratch::new("plugin-default");
    scratch.user_file("colors/dusk.lua", DUSK);
    scratch.plugin_file(
        "pane",
        "plugin/pane.lua",
        "gband.hl.default('PaneSegment', { fg = 4 })",
    );
    scratch.write(JOB);
    let config = scratch.loaded();
    clean(&run_job(&config, "gband.colorscheme('dusk')"));
    assert_eq!(resolved_fg(&config, "PaneSegment").as_deref(), Some("4"));
}

#[test]
fn colorscheme_overrides_a_plugin_default() {
    let scratch = Scratch::new("overrides");
    scratch.user_file(
        "colors/dusk.lua",
        "gband.hl.set('PaneSegment', { fg = '#00ff00' })",
    );
    scratch.plugin_file(
        "pane",
        "plugin/pane.lua",
        "gband.hl.default('PaneSegment', { fg = 4 })",
    );
    scratch.write("gband.colorscheme('dusk')");
    let config = scratch.loaded();
    assert_eq!(
        resolved_fg(&config, "PaneSegment").as_deref(),
        Some("#00ff00")
    );
}

#[test]
fn default_at_start() {
    let scratch = Scratch::new("start");
    scratch.write("during = gband.colorscheme()");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "during"), "default");
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "default"
    );
}

#[test]
fn failing_colorscheme() {
    let scratch = Scratch::new("failing");
    let broken = scratch.user_file(
        "colors/broken.lua",
        "\n\ngband.hl.set('StatusLine', { fg = 1 })\nerror('boom')",
    );
    scratch.write(
        "ok = gband.colorscheme('broken')\ngband.bind('alt+h', gband.action.focus_column_left)",
    );
    let config = scratch.loaded();
    assert!(!global::<bool>(&config, "ok"));
    assert!(config.keymap["root"].len() == 1);
    assert_eq!(
        resolved_fg(&config, "StatusLine").as_deref(),
        Some("#c0caf5")
    );
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "default"
    );
    let [error] = config.errors.as_slice() else {
        panic!("{:?}", config.errors);
    };
    assert_eq!(
        error.to_string(),
        format!("colors/broken: {}:4: boom", broken.display())
    );
}

#[test]
fn syntax_error_in_a_colorscheme() {
    let scratch = Scratch::new("syntax");
    let broken = scratch.user_file("colors/broken.lua", "gband.hl.set(");
    scratch.write("ok = gband.colorscheme('broken')");
    let config = scratch.loaded();
    assert!(!global::<bool>(&config, "ok"));
    let [error] = config.errors.as_slice() else {
        panic!("{:?}", config.errors);
    };
    assert_eq!(error.plugin.as_deref(), Some("colors/broken"));
    assert_eq!(error.location.as_ref().map(|(path, _)| path), Some(&broken));
}

#[test]
fn looping_colorscheme_is_stopped() {
    let scratch = Scratch::new("looping");
    scratch.user_file("colors/spin.lua", "while true do end");
    scratch.write("ok = gband.colorscheme('spin')\nafter = true");
    let config = scratch.load_with_budget(200_000).unwrap();
    assert!(!global::<bool>(&config, "ok"));
    assert!(global::<bool>(&config, "after"));
    let [error] = config.errors.as_slice() else {
        panic!("{:?}", config.errors);
    };
    assert_eq!(error.plugin.as_deref(), Some("colors/spin"));
    assert!(error.message.contains("instruction limit"), "{error}");
}

#[test]
fn missing_colorscheme() {
    let scratch = Scratch::new("missing");
    scratch.write(JOB);
    let config = scratch.loaded();
    let outcome = run_job(&config, "result = gband.colorscheme('absent')");
    assert!(!global::<bool>(&config, "result"));
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_eq!(error.plugin.as_deref(), Some("colors/absent"));
    assert!(error.to_string().starts_with("colors/absent: "), "{error}");
}

#[test]
fn invalid_colorscheme_name() {
    let scratch = Scratch::new("invalid-name");
    let path = scratch.write("\nok = gband.colorscheme('../etc')");
    let config = scratch.loaded();
    assert!(!global::<bool>(&config, "ok"));
    let [error] = config.errors.as_slice() else {
        panic!("{:?}", config.errors);
    };
    assert_eq!(error.plugin.as_deref(), Some("colors/../etc"));
    assert_error_at(error, &path, 2, "../etc");
}

#[test]
fn switch_at_run_time() {
    let scratch = Scratch::new("switch");
    scratch.user_file("colors/dusk.lua", DUSK);
    scratch.write(&format!(
        "{JOB}log = {{}}\ngband.on('ColorschemeChanged', function(e) log[#log + 1] = e.name .. ' ' .. e.previous end)\ngband.on('HighlightChanged', function(e) log[#log + 1] = 'hl ' .. e.group end)"
    ));
    let config = scratch.loaded();
    clean(&run_job(&config, "gband.colorscheme('dusk')"));
    assert_eq!(global::<Vec<String>>(&config, "log"), ["dusk default"]);
}

#[test]
fn failed_switch_emits_nothing() {
    let scratch = Scratch::new("failed-switch");
    scratch.user_file(
        "colors/broken.lua",
        "gband.hl.set('StatusLine', { fg = 1 })\nerror('boom')",
    );
    scratch.write(&format!(
        "{JOB}log = {{}}\ngband.on('ColorschemeChanged', function(e) log[#log + 1] = e.name end)\ngband.on('HighlightChanged', function(e) log[#log + 1] = e.group end)"
    ));
    let config = scratch.loaded();
    let outcome = run_job(&config, "gband.colorscheme('broken')");
    assert_eq!(outcome.errors.len(), 1);
    assert_eq!(global::<Vec<String>>(&config, "log"), Vec::<String>::new());
}

#[test]
fn colorscheme_errors_disable_no_plugin() {
    let scratch = Scratch::new("no-disable");
    scratch.user_file("colors/broken.lua", "error('boom')");
    scratch.plugin_file(
        "theme",
        "plugin/theme.lua",
        "gband.colorscheme('broken')\nafter = true",
    );
    scratch.plugin_file("theme", "plugin/z.lua", "later = true");
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "after"));
    assert!(global::<bool>(&config, "later"));
    assert_eq!(config.errors[0].plugin.as_deref(), Some("colors/broken"));
}
