mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::thread;
use std::time::Duration;

use common::*;

const LABELS: [&str; 5] = [
    "theme    ",
    "sidebar  ",
    "keys     ",
    "I on new ",
    "reload   ",
];
const RELOAD: usize = 4;

fn position(screen: &Grid, text: &str) -> Option<(u16, u16)> {
    screen
        .contents()
        .lines()
        .enumerate()
        .find_map(|(row, line)| {
            let start = line.find(text)?;
            let col = line[..start].chars().count();
            Some((u16::try_from(row).ok()?, u16::try_from(col).ok()?))
        })
}

fn line_text(screen: &Grid, label: &str) -> Option<String> {
    screen.contents().lines().find_map(|line| {
        let start = line.find(label)?;
        let rest = &line[start..];
        Some(rest.split('│').next().unwrap_or(rest).trim_end().to_owned())
    })
}

fn settings_open(screen: &Grid) -> bool {
    LABELS[..3]
        .iter()
        .all(|label| position(screen, label).is_some())
}

fn look(screen: &Grid, label: &str) -> Option<String> {
    let (row, col) = position(screen, label)?;
    let cell = screen.screen().cell(row, col)?;
    Some(format!("{:?} {}", cell.bgcolor(), cell.inverse()))
}

fn cursor_line(screen: &Grid) -> Option<usize> {
    if !settings_open(screen) {
        return None;
    }
    let looks: Vec<String> = LABELS
        .iter()
        .filter_map(|label| look(screen, label))
        .collect();
    (0..looks.len())
        .find(|&index| looks.iter().filter(|other| **other == looks[index]).count() == 1)
}

fn navigation(screen: &Grid) -> bool {
    sidebar_mode(screen) == "N"
}

fn interactive(screen: &Grid) -> bool {
    sidebar_mode(screen) == "I"
}

fn focused_shows(screen: &Grid, label: &str) -> bool {
    focused_lines(screen).iter().any(|line| line == label)
}

fn untouched_prompt(screen: &Grid) -> bool {
    focused_lines(screen)
        .iter()
        .rfind(|line| !line.is_empty())
        .is_some_and(|line| line.ends_with('$'))
}

fn save_interactive_on_new(env: &TestEnv, value: bool) {
    fs::write(
        env.config_dir().join("user").join("interactive_on_new.lua"),
        format!("return {value}\n"),
    )
    .unwrap();
}

fn labelled(env: &TestEnv) -> Attached {
    let mut client = Attached::start(env, 80, 24);
    client.wait_for_prompt();
    client.run("clear; echo W1");
    client.wait_for_line("W1");
    client
}

fn new_window(client: &mut Attached) {
    client.send(b"\x00n");
    client.wait_for("a second focused window", |screen| {
        tiles(screen).len() == 2 && untouched_prompt(screen) && !focused_shows(screen, "W1")
    });
}

fn sidebar(screen: &Grid, row: usize) -> String {
    screen
        .contents()
        .lines()
        .nth(row)
        .unwrap_or_default()
        .chars()
        .take(1)
        .collect()
}

fn reloads(env: &TestEnv) -> usize {
    env.log_text("client")
        .matches("configuration reloaded")
        .count()
}

fn user_file(env: &TestEnv, name: &str) -> Option<String> {
    fs::read_to_string(env.config_dir().join("user").join(name)).ok()
}

fn offered(env: &TestEnv) -> Attached {
    let client = Attached::start(env, 80, 24);
    client.wait_for("the settings window", |screen| {
        settings_open(screen) && cursor_line(screen) == Some(0)
    });
    client
}

fn opened(env: &TestEnv) -> Attached {
    let mut client = Attached::start(env, 80, 24);
    client.wait_for_prompt();
    client.send(b"\x00s");
    client.wait_for("the settings window", |screen| {
        settings_open(screen) && cursor_line(screen) == Some(0)
    });
    client
}

fn press(client: &mut Attached, keys: &[u8], line: usize) {
    for &key in keys {
        client.send(&[key]);
        thread::sleep(Duration::from_millis(50));
    }
    client.wait_for("the cursor line", |screen| {
        cursor_line(screen) == Some(line)
    });
}

fn reopened(client: &Attached, env: &TestEnv, before: usize, line: usize, shown: &str) {
    wait_until(|| reloads(env) > before, "the configuration to reload");
    client.wait_for("the settings window again", |screen| {
        cursor_line(screen) == Some(line)
            && line_text(screen, LABELS[line]).as_deref() == Some(shown)
    });
}

fn escape(client: &mut Attached) {
    client.send(b"\x1b");
    client.wait_for("the settings window to close", |screen| {
        !settings_open(screen)
    });
}

#[test]
fn first_start() {
    let env = TestEnv::without_settings("settings-first-start");
    let client = offered(&env);
    let screen = client.screen();
    assert_eq!(
        line_text(&screen, "theme").as_deref(),
        Some("theme    default")
    );
    assert_eq!(
        line_text(&screen, "keys").as_deref(),
        Some("keys     modal")
    );
    drop(screen);
    assert!(user_file(&env, "theme.lua").is_none());
    assert!(user_file(&env, "keystyle.lua").is_none());
}

#[test]
fn offered_again_on_the_next_start() {
    let env = TestEnv::without_settings("settings-offered-again");
    let mut client = offered(&env);
    escape(&mut client);
    client.send(b"\x00D");
    client.wait_for_text("[detached]");
    client.wait_exit();
    let again = offered(&env);
    drop(again);
}

#[test]
fn no_offer_with_a_setting_saved() {
    let env = TestEnv::without_settings("settings-no-offer");
    env.save_theme("nord");
    let client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    thread::sleep(Duration::from_millis(300));
    assert!(!settings_open(&client.screen()), "{}", client.contents());
}

#[test]
fn no_offer_with_an_own_configuration() {
    let env = TestEnv::without_settings("settings-own");
    env.write_config("gband.bind('alt+h', gband.action.focus_column_left)");
    let client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    thread::sleep(Duration::from_millis(500));
    assert!(!settings_open(&client.screen()), "{}", client.contents());
}

#[test]
fn open_with_the_prefix_and_move() {
    let env = TestEnv::new("settings-open");
    let mut client = opened(&env);
    press(&mut client, b"j", 1);
    escape(&mut client);
    assert!(user_file(&env, "sidebar.lua").is_none());
}

#[test]
fn open_with_the_direct_key_style() {
    let env = TestEnv::new("settings-direct");
    env.save_key_style("direct");
    let client = opened(&env);
    drop(client);
}

#[test]
fn turn_the_sidebar_off() {
    let env = TestEnv::new("settings-sidebar-off");
    let mut client = opened(&env);
    assert_eq!(sidebar(&client.screen(), 0), "I");
    press(&mut client, b"j", 1);
    let before = reloads(&env);
    client.send(b"\r");
    reopened(&client, &env, before, 1, "sidebar  off");
    assert_eq!(
        user_file(&env, "sidebar.lua").as_deref(),
        Some("return false\n")
    );
    escape(&mut client);
    client.wait_for("no sidebar", |screen| {
        tiles(screen).iter().any(|tile| tile.left == 0)
    });
}

#[test]
fn switch_the_key_style() {
    let env = TestEnv::new("settings-key-style");
    let mut client = opened(&env);
    press(&mut client, b"jj", 2);
    let before = reloads(&env);
    client.send(b"l");
    reopened(&client, &env, before, 2, "keys     direct");
    assert_eq!(
        user_file(&env, "keystyle.lua").as_deref(),
        Some("return \"direct\"\n")
    );
    assert!(position(&client.screen(), LABELS[3]).is_none());
    escape(&mut client);
    client.send(b"\x00");
    thread::sleep(Duration::from_millis(100));
    client.wait_for("the direct style's label", |screen| {
        sidebar(screen, 0) == "P"
    });
}

fn switch_key_style(name: &str, saved: &str, key: &[u8], expected: &str) {
    let env = TestEnv::new(name);
    env.save_key_style(saved);
    let mut client = opened(&env);
    press(&mut client, b"jj", 2);
    let before = reloads(&env);
    client.send(key);
    reopened(&client, &env, before, 2, &format!("keys     {expected}"));
    assert_eq!(
        user_file(&env, "keystyle.lua"),
        Some(format!("return \"{expected}\"\n"))
    );
    assert_eq!(
        position(&client.screen(), LABELS[3]).is_some(),
        expected == "modal"
    );
}

#[test]
fn window_with_the_floating_style() {
    let env = TestEnv::new("settings-floating-window");
    env.save_key_style("floating");
    let client = opened(&env);
    let screen = client.screen();
    assert_eq!(
        line_text(&screen, LABELS[2]).as_deref(),
        Some("keys     floating")
    );
    assert!(position(&screen, LABELS[3]).is_none());
    assert_eq!(position(&screen, LABELS[0]), Some((10, 26)));
    assert_eq!(
        line_text(&screen, LABELS[RELOAD]).as_deref(),
        Some("reload   ok")
    );
    assert_eq!(position(&screen, "└"), Some((14, 25)));
}

#[test]
fn switch_to_the_floating_style() {
    switch_key_style("settings-to-floating", "direct", b"\r", "floating");
}

#[test]
fn back_from_the_modal_style() {
    switch_key_style("settings-back-from-modal", "modal", b"h", "floating");
}

#[test]
fn forward_from_the_floating_style() {
    switch_key_style(
        "settings-forward-from-floating",
        "floating",
        b"\x1b[C",
        "modal",
    );
}

#[test]
fn windows_float_after_switching_the_style() {
    let env = TestEnv::new("settings-floating-windows");
    let mut client = labelled(&env);
    new_window(&mut client);
    client.send(b"s");
    client.wait_for("the settings window", |screen| {
        settings_open(screen) && cursor_line(screen) == Some(0)
    });
    press(&mut client, b"jj", 2);
    let before = reloads(&env);
    client.send(b"h");
    reopened(&client, &env, before, 2, "keys     floating");
    assert_eq!(
        user_file(&env, "keystyle.lua").as_deref(),
        Some("return \"floating\"\n")
    );
    client.wait_for("both windows floating", |screen| {
        let contents = screen.contents();
        contents.matches('╰').count() == 2
            && contents.contains("[_][□][X]")
            && tiles(screen).iter().all(|tile| tile.top > 0)
    });
}

#[test]
fn pick_a_theme_from_the_list() {
    let env = TestEnv::new("settings-theme-list");
    env.save_theme("gruvbox");
    let mut client = opened(&env);
    client.send(b"\r");
    client.wait_for("the theme list", |screen| {
        position(screen, "catppuccin-macchiato").is_some()
    });
    client.send(b"k");
    thread::sleep(Duration::from_millis(100));
    let before = reloads(&env);
    client.send(b"\r");
    reopened(&client, &env, before, 0, "theme    nord");
    assert_eq!(
        user_file(&env, "theme.lua").as_deref(),
        Some("return \"nord\"\n")
    );
}

#[test]
fn no_reopen_for_an_unrelated_reload() {
    let env = TestEnv::new("settings-unrelated");
    let extra = env.config_dir().join("user").join("lua").join("extra.lua");
    fs::create_dir_all(extra.parent().unwrap()).unwrap();
    let mut client = opened(&env);
    escape(&mut client);
    let before = reloads(&env);
    fs::write(&extra, "return {}").unwrap();
    wait_until(|| reloads(&env) > before, "the configuration to reload");
    thread::sleep(Duration::from_millis(300));
    assert!(!settings_open(&client.screen()), "{}", client.contents());
}

#[test]
fn no_reopen_after_a_failed_reload() {
    let env = TestEnv::new("settings-failed-reload");
    env.write_config(
        "if gband.settings.sidebar() == false then error('boom') end\n\
         gband.keystyle.use()\n\
         gband.plugin('gband.sidebar')\n",
    );
    let mut client = opened(&env);
    press(&mut client, b"j", 1);
    client.send(b"\r");
    wait_until(
        || user_file(&env, "sidebar.lua").as_deref() == Some("return false\n"),
        "the sidebar setting saved",
    );
    wait_until(
        || env.log_text("client").contains("boom"),
        "the failed reload",
    );
    thread::sleep(Duration::from_millis(300));
    assert_eq!(
        line_text(&client.screen(), "sidebar").as_deref(),
        Some("sidebar  on")
    );
    escape(&mut client);
    let before = reloads(&env);
    fs::write(
        env.config_dir().join("user").join("sidebar.lua"),
        "return true\n",
    )
    .unwrap();
    wait_until(|| reloads(&env) > before, "the configuration to reload");
    thread::sleep(Duration::from_millis(300));
    assert!(!settings_open(&client.screen()), "{}", client.contents());
}

#[test]
fn opened_from_the_lua_prompt_leaves_the_user_file() {
    let env = TestEnv::new("settings-prompt");
    let source = "gband.keystyle.use()\n\
                  gband.plugin('gband.sidebar')\n\
                  gband.bind('alt+h', gband.action.focus_column_left)\n";
    env.write_config(source);
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    let line = "gband.settings.open()";
    client.send(b"\x00:");
    client.send(line.as_bytes());
    client.wait_for("the prompt holding the line", |screen| {
        screen.contents().contains(&format!(":{line}"))
    });
    client.send(b"\r");
    client.wait_for("the settings window", |screen| {
        settings_open(screen) && cursor_line(screen) == Some(0)
    });
    press(&mut client, b"jj", 2);
    let before = reloads(&env);
    client.send(b"\r");
    reopened(&client, &env, before, 2, "keys     direct");
    assert_eq!(fs::read_to_string(env.user_lua()).unwrap(), source);
}

#[test]
fn cannot_save() {
    let env = TestEnv::new("settings-read-only");
    let user = env.config_dir().join("user");
    fs::set_permissions(&user, fs::Permissions::from_mode(0o555)).unwrap();
    if fs::write(user.join("probe"), "").is_ok() {
        fs::set_permissions(&user, fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }
    let mut client = opened(&env);
    press(&mut client, b"j", 1);
    client.send(b"\r");
    client.wait_for("the error marker", |screen| sidebar(screen, 23) == "!");
    fs::set_permissions(&user, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(settings_open(&client.screen()), "{}", client.contents());
    let log = env.log_text("client");
    assert!(log.contains("user/sidebar.lua"), "{log}");
    assert!(user_file(&env, "sidebar.lua").is_none());
}

#[test]
fn turn_interactive_on_new_on() {
    let env = TestEnv::new("settings-interactive-fresh");
    let mut client = opened(&env);
    press(&mut client, b"jjj", 3);
    let before = reloads(&env);
    client.send(b"\r");
    reopened(&client, &env, before, 3, "I on new on");
    assert_eq!(
        user_file(&env, "interactive_on_new.lua").as_deref(),
        Some("return true\n")
    );
    escape(&mut client);
    new_window(&mut client);
    client.wait_for("interactive mode after the new window", interactive);
}

#[test]
fn turn_interactive_on_new_off() {
    let env = TestEnv::new("settings-interactive-off");
    save_interactive_on_new(&env, true);
    let mut client = opened(&env);
    press(&mut client, b"jjj", 3);
    let before = reloads(&env);
    client.send(b"\r");
    reopened(&client, &env, before, 3, "I on new off");
    assert_eq!(
        user_file(&env, "interactive_on_new.lua").as_deref(),
        Some("return false\n")
    );
    escape(&mut client);
    new_window(&mut client);
    client.wait_for("navigation mode after the new window", navigation);
}

#[test]
fn turn_interactive_on_new_back_on() {
    let env = TestEnv::new("settings-interactive-on");
    save_interactive_on_new(&env, false);
    let mut client = opened(&env);
    press(&mut client, b"jjj", 3);
    let before = reloads(&env);
    client.send(b"h");
    reopened(&client, &env, before, 3, "I on new on");
    assert_eq!(
        user_file(&env, "interactive_on_new.lua").as_deref(),
        Some("return true\n")
    );
}

#[test]
fn open_a_window_and_stay_in_navigation_mode() {
    let env = TestEnv::new("settings-interactive-navigation");
    save_interactive_on_new(&env, false);
    let mut client = labelled(&env);
    new_window(&mut client);
    client.wait_for("navigation mode after the new window", navigation);
    client.send(b"h");
    client.wait_for("the first window focused in navigation mode", |screen| {
        focused_shows(screen, "W1") && navigation(screen)
    });
    assert!(untouched_prompt(&client.screen()), "{}", client.contents());
}

#[test]
fn direct_key_style_ignores_the_setting() {
    let env = TestEnv::new("settings-interactive-direct");
    env.save_key_style("direct");
    save_interactive_on_new(&env, false);
    let mut client = labelled(&env);
    new_window(&mut client);
    client.run("echo direct-root");
    client.wait_for_line("direct-root");
}

#[test]
fn own_configuration_with_the_modal_preset() {
    let env = TestEnv::new("settings-interactive-own");
    env.write_config("gband.keystyle.use(\"modal\")\n");
    save_interactive_on_new(&env, true);
    let mut client = labelled(&env);
    new_window(&mut client);
    client.run("echo modal-root");
    client.wait_for_line("modal-root");
}

fn on_the_reload_line(env: &TestEnv) -> Attached {
    let mut client = opened(env);
    press(&mut client, b"jjjj", RELOAD);
    assert_eq!(
        line_text(&client.screen(), LABELS[RELOAD]).as_deref(),
        Some("reload   ok")
    );
    client
}

fn reopened_on_the_reload_line(client: &Attached, shown: &str) {
    client.wait_for("the settings window on its reload line", |screen| {
        cursor_line(screen) == Some(RELOAD)
            && line_text(screen, LABELS[RELOAD]).as_deref() == Some(shown)
    });
}

#[test]
fn back_on_the_reload_line() {
    let env = TestEnv::new("settings-reload-line");
    env.write_client_plugin(
        "label",
        "gband.bar.add({ side = 'right', size = 6, lines = { 'old' } })\n",
    );
    let mut client = on_the_reload_line(&env);
    client.wait_for_text("old");
    let files = fs::read_dir(env.config_dir().join("user")).unwrap().count();
    env.write_client_plugin(
        "label",
        "gband.bar.add({ side = 'right', size = 6, lines = { 'new' } })\n",
    );
    let before = reloads(&env);
    client.send(b"\r");
    wait_until(|| reloads(&env) > before, "the configuration to reload");
    client.wait_for_text("new");
    reopened_on_the_reload_line(&client, "reload   ok");
    assert_eq!(
        fs::read_dir(env.config_dir().join("user")).unwrap().count(),
        files
    );
    assert_eq!(user_file(&env, "sidebar.lua"), None);
}

#[test]
fn failed_reload_counted() {
    let env = TestEnv::new("settings-reload-failed");
    env.write_client_plugin("fragile", "local fine = true\n");
    let mut client = on_the_reload_line(&env);
    env.write_client_plugin("fragile", "error('bad')\n");
    client.send(b"\r");
    reopened_on_the_reload_line(&client, "reload   1 error");
}

#[test]
fn server_error_counted() {
    let env = TestEnv::new("settings-reload-server-error");
    env.write_server_plugin("fragile", "local fine = true\n");
    let mut client = on_the_reload_line(&env);
    env.write_server_plugin("fragile", "error('bad')\n");
    client.send(b"\r");
    reopened_on_the_reload_line(&client, "reload   1 error");
}
