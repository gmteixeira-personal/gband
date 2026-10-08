mod common;

use common::*;

fn resolved(config: &gband_lua::Config, group: &str) -> Vec<String> {
    let mut fields: Vec<String> = eval(
        config,
        &format!(
            "local s = gband.hl.get('{group}', {{ resolve = true }})\nlocal out = {{}} for k, v in pairs(s) do out[#out + 1] = k .. '=' .. tostring(v) end\nreturn out"
        ),
    );
    fields.sort();
    fields
}

#[test]
fn actions_registered() {
    let scratch = Scratch::new("desktop-actions");
    scratch.write("gband.plugin('gband.desktop')");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    let actions: Vec<Vec<String>> = eval(
        &config,
        "local list = {} for _, a in ipairs(gband.action.list()) do if a.name:find('^desktop%.') then list[#list + 1] = { a.name, a.desc } end end return list",
    );
    assert_eq!(
        actions,
        [
            [
                "desktop.leader",
                "open the window list, or send the prefix key from it"
            ],
            ["desktop.list", "list the windows"],
            [
                "desktop.menu",
                "open the menu for the cell under the pointer"
            ],
            [
                "desktop.press",
                "move, resize or press a button of a floating window"
            ],
        ]
    );
}

#[test]
fn unknown_option() {
    let scratch = Scratch::new("desktop-unknown-option");
    scratch.write("ok = gband.plugin('gband.desktop', { buttons = false })");
    let config = scratch.loaded();
    assert!(!global::<bool>(&config, "ok"));
    let error = config
        .errors
        .iter()
        .find(|error| error.plugin.as_deref() == Some("desktop"))
        .unwrap_or_else(|| panic!("no error names desktop: {:?}", config.errors));
    assert!(error.message.contains("buttons"), "{error}");
}

#[test]
fn press_action_from_a_key() {
    let scratch = Scratch::new("desktop-press-from-key");
    scratch.write(&format!(
        "{JOB}gband.keystyle.use('floating')\ngband.keymap.set('prefix', 'x', gband.action['desktop.press'])"
    ));
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    let outcome = run_job(&config, "gband.keymap.run('prefix', 'x')");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty(), "{:?}", outcome.dispatched);
    let outcome = run_job(&config, "gband.action['desktop.menu']()");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty(), "{:?}", outcome.dispatched);
}

#[test]
fn defaults_without_a_colorscheme() {
    let scratch = Scratch::new("desktop-defaults");
    scratch.user_file("colors/blank.lua", "");
    scratch.write("gband.colorscheme('blank')\ngband.keystyle.use('floating')");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    assert_eq!(resolved(&config, "DesktopButton"), ["bold=true"]);
    assert_eq!(resolved(&config, "DesktopClose"), ["bold=true", "fg=1"]);
    assert_eq!(resolved(&config, "DesktopMinimized"), ["dim=true"]);
}
