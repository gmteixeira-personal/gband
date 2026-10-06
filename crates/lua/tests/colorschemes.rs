mod common;

use common::*;
use gband_lua::Config;

const DUSK: &str = "gband.hl.set('SidebarMode', { fg = '#112233' })\nran = (ran or '') .. 'dusk'\n";

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
        resolved_fg(&config, "SidebarMode").as_deref(),
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
    scratch.write("ok = gband.colorscheme('nord')");
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "ok"));
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "nord"
    );
    assert_eq!(
        eval::<String>(&config, "return gband.palette.get().bg"),
        "#2e3440"
    );
}

#[test]
fn shadowing_a_bundled_theme() {
    let scratch = Scratch::new("shadow-theme");
    scratch.user_file("colors/gruvbox.lua", "ran = 'user'");
    scratch.write("gband.colorscheme('gruvbox')");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "ran"), "user");
}

#[test]
fn default_sets_the_sidebar_groups() {
    let scratch = Scratch::new("default-sidebar");
    scratch.user_file("keystyle.lua", "return 'modal'");
    let config = scratch.loaded();
    for (group, expected) in [
        ("SidebarMode", "{ bold = true }"),
        ("SidebarBand", "{ dim = true }"),
        ("SidebarBandActive", "{ bold = true }"),
        ("SidebarError", "{ fg = 1, bold = true }"),
    ] {
        let same: bool = eval(
            &config,
            &format!(
                "local got, want = gband.hl.get('{group}'), {expected}
                for k, v in pairs(want) do if got[k] ~= v then return false end end
                for k in pairs(got) do if want[k] == nil then return false end end
                return true"
            ),
        );
        assert!(same, "{group}");
    }
}

#[test]
fn default_sets_nothing() {
    let scratch = Scratch::new("default-nothing");
    scratch.user_file("keystyle.lua", "return 'modal'");
    let config = scratch.loaded();
    let mode: (Option<bool>, i64) = (
        eval(&config, "return gband.hl.get('SidebarMode').bold"),
        eval(
            &config,
            "local n = 0 for _ in pairs(gband.hl.get('SidebarMode')) do n = n + 1 end return n",
        ),
    );
    assert_eq!(mode, (Some(true), 1));
    let border: Vec<String> = eval(
        &config,
        "local s = gband.hl.get('WindowBorderFocused') return { s.fg, tostring(s.bold) }",
    );
    assert_eq!(border, ["#b1b9f9", "true"]);
    let empty: bool = eval(&config, "return next(gband.palette.get()) == nil");
    assert!(empty);
}

#[test]
fn back_to_the_defaults() {
    let scratch = Scratch::new("back-to-default");
    scratch.write(&format!("gband.colorscheme('gruvbox')\n{JOB}"));
    let config = scratch.loaded();
    clean(&run_job(&config, "ok = gband.colorscheme('default')"));
    assert!(global::<bool>(&config, "ok"));
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "default"
    );
    let explicit: bool = eval(
        &config,
        "for _, g in ipairs({ 'Bar', 'SidebarMode', 'WindowBorder', 'ErrorBanner', 'PluginWindow' }) do
          if gband.hl.get(g, { resolve = true }).fg == '#ebdbb2' then return true end
        end
        return false",
    );
    assert!(!explicit);
    let empty: bool = eval(&config, "return next(gband.palette.get()) == nil");
    assert!(empty);
}

#[test]
fn bars_on_the_terminals_background() {
    let scratch = Scratch::new("bar-unset");
    let config = scratch.loaded();
    let empty: bool = eval(&config, "return next(gband.hl.get('Bar') or {}) == nil");
    assert!(empty);
    let fields: i64 = eval(
        &config,
        "local n = 0 for _ in pairs(gband.hl.get('Bar', { resolve = true })) do n = n + 1 end return n",
    );
    assert_eq!(fields, 0);
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
fn palette_replaced() {
    let scratch = Scratch::new("palette-replaced");
    scratch.user_file("theme.lua", "return \"gruvbox\"\n");
    scratch.user_file("colors/dusk.lua", DUSK);
    scratch.write("before = gband.palette.get().bg\ngband.colorscheme('dusk')");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "before"), "#282828");
    let empty: bool = eval(&config, "return next(gband.palette.get()) == nil");
    assert!(empty);
}

#[test]
fn explicit_settings_replaced() {
    let scratch = Scratch::new("replaced");
    scratch.user_file("colors/dusk.lua", "gband.hl.set('SidebarMode', { fg = 1 })");
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
    scratch.write("gband.colorscheme('dusk')\ngband.hl.set('SidebarBandActive', { fg = 2 })");
    let config = scratch.loaded();
    assert_eq!(
        resolved_fg(&config, "SidebarBandActive").as_deref(),
        Some("2")
    );
    let bold: Option<bool> = eval(
        &config,
        "return gband.hl.get('SidebarBandActive', { resolve = true }).bold",
    );
    assert_eq!(bold, None);
}

#[test]
fn plugin_default_kept() {
    let scratch = Scratch::new("plugin-default");
    scratch.user_file("colors/dusk.lua", DUSK);
    scratch.client_plugin("window", "gband.hl.default('WindowSegment', { fg = 4 })");
    scratch.write(JOB);
    let config = scratch.loaded();
    clean(&run_job(&config, "gband.colorscheme('dusk')"));
    assert_eq!(resolved_fg(&config, "WindowSegment").as_deref(), Some("4"));
}

#[test]
fn colorscheme_overrides_a_plugin_default() {
    let scratch = Scratch::new("overrides");
    scratch.user_file(
        "colors/dusk.lua",
        "gband.hl.set('WindowSegment', { fg = '#00ff00' })",
    );
    scratch.client_plugin("window", "gband.hl.default('WindowSegment', { fg = 4 })");
    scratch.write("gband.colorscheme('dusk')");
    let config = scratch.loaded();
    assert_eq!(
        resolved_fg(&config, "WindowSegment").as_deref(),
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
fn saved_theme_at_start() {
    let scratch = Scratch::new("saved-start");
    scratch.user_file("theme.lua", "return \"nord\"\n");
    scratch
        .write("during = gband.colorscheme()\ngband.bind('alt+h', gband.action.focus_column_left)");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "during"), "nord");
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "nord"
    );
}

#[test]
fn init_file_replaces_the_start_theme() {
    let scratch = Scratch::new("init-replaces");
    scratch.user_file("theme.lua", "return \"nord\"\n");
    scratch.write("gband.colorscheme('dracula')");
    let config = scratch.loaded();
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "dracula"
    );
}

#[test]
fn saved_theme_missing() {
    let scratch = Scratch::new("saved-missing");
    scratch.user_file("theme.lua", "return \"absent\"\n");
    let config = scratch.loaded();
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "default"
    );
    let [error] = config.errors.as_slice() else {
        panic!("{:?}", config.errors);
    };
    assert_eq!(error.plugin.as_deref(), Some("colors/absent"));
    assert_eq!(error.location, None);
}

#[test]
fn failing_colorscheme() {
    let scratch = Scratch::new("failing");
    scratch.user_file("theme.lua", "return \"gruvbox\"\n");
    let broken = scratch.user_file(
        "colors/broken.lua",
        "\n\ngband.hl.set('SidebarMode', { fg = 1 })\ngband.palette.set({})\nerror('boom')",
    );
    scratch.write(
        "ok = gband.colorscheme('broken')\ngband.bind('alt+h', gband.action.focus_column_left)",
    );
    let config = scratch.loaded();
    assert!(!global::<bool>(&config, "ok"));
    assert!(config.keymap["root"].len() == 1);
    assert_eq!(
        resolved_fg(&config, "SidebarMode").as_deref(),
        Some("#fabd2f")
    );
    assert_eq!(
        eval::<String>(&config, "return gband.palette.get().bg"),
        "#282828"
    );
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "gruvbox"
    );
    let [error] = config.errors.as_slice() else {
        panic!("{:?}", config.errors);
    };
    assert_eq!(
        error.to_string(),
        format!("colors/broken: {}:5: boom", broken.display())
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
        "gband.hl.set('SidebarMode', { fg = 1 })\nerror('boom')",
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
    scratch.client_plugin("theme", "gband.colorscheme('broken')\nafter = true");
    scratch.client_plugin("zeta", "later = true");
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "after"));
    assert!(global::<bool>(&config, "later"));
    assert_eq!(config.errors[0].plugin.as_deref(), Some("colors/broken"));
}
