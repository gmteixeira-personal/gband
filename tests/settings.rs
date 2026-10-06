mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::thread;
use std::time::Duration;

use common::*;

const LABELS: [&str; 3] = ["theme    ", "sidebar  ", "keys     "];

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
    LABELS.iter().all(|label| position(screen, label).is_some())
}

fn look(screen: &Grid, label: &str) -> Option<String> {
    let (row, col) = position(screen, label)?;
    let cell = screen.screen().cell(row, col)?;
    Some(format!("{:?} {}", cell.bgcolor(), cell.inverse()))
}

fn cursor_line(screen: &Grid) -> Option<usize> {
    let looks: Vec<String> = LABELS
        .iter()
        .map(|label| look(screen, label))
        .collect::<Option<_>>()?;
    (0..3).find(|&index| looks.iter().filter(|other| **other == looks[index]).count() == 1)
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
    escape(&mut client);
    client.send(b"\x00");
    thread::sleep(Duration::from_millis(100));
    client.wait_for("the direct style's label", |screen| {
        sidebar(screen, 0) == "P"
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
