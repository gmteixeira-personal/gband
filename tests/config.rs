mod common;

use std::fs;
use std::thread;
use std::time::Duration;

use common::*;
use gband_test_support::cell;
use ratatui::style::Modifier;

const RELOADED: &str = "configuration reloaded";

fn reloads(env: &TestEnv, role: &str) -> usize {
    env.log_text(role).matches(RELOADED).count()
}

fn wait_for_reload(env: &TestEnv, role: &str, seen: usize) {
    wait_until(|| reloads(env, role) > seen, "a configuration reload");
}

fn two_panes_first_focused(env: &TestEnv) -> Attached {
    let mut client = Attached::start(env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(env);
    client.send(b"\x01\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused
    });
    client.wait_for_prompt();
    client.shell_pid(env);
    client.send(b"\x01h");
    client.wait_for("the first tile focused", |screen| tiles(screen)[0].focused);
    client
}

fn bottom_row(screen: &Grid) -> String {
    screen
        .contents()
        .lines()
        .nth(usize::from(screen.size().rows) - 1)
        .unwrap_or_default()
        .to_owned()
}

fn tops(screen: &Grid) -> Vec<(u16, bool)> {
    (0..screen.size().cols)
        .filter_map(|col| {
            let cell = cell(screen, 0, col)?;
            (cell.symbol() == "┌").then(|| (col, cell.modifier.contains(Modifier::BOLD)))
        })
        .collect()
}

fn focused_top(screen: &Grid) -> Option<u16> {
    tops(screen)
        .into_iter()
        .find_map(|(col, focused)| focused.then_some(col))
}

fn echo_keys(client: &mut Attached) {
    client.run("clear; cat -v");
    thread::sleep(Duration::from_millis(300));
}

#[test]
fn harness_isolates_the_configuration() {
    let env = TestEnv::new("config-harness");
    let command = env.command(GBAND, &["list-sessions"]);
    let named = command
        .get_envs()
        .find(|(name, _)| *name == "XDG_CONFIG_HOME")
        .and_then(|(_, value)| value);
    assert_eq!(named, Some(env.config_home().as_os_str()));
    assert!(env.config_home().starts_with(&env.root));
    assert!(!env.init_lua().exists());
    env.write_config("");
    assert!(env.init_lua().exists());
}

#[test]
fn no_configuration_file() {
    let env = TestEnv::new("config-none");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x01\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused && tiles[1].left == 40
    });
    assert!(!env.log_text("client").contains("configuration error"));
    assert!(!env.log_text("server").contains("configuration error"));
}

#[test]
fn xdg_config_home_is_honoured() {
    let env = TestEnv::new("config-xdg");
    env.write_config("gband.set { width_presets = { 1/4, 1/2 } }");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.send(b"\x01r");
    client.wait_for("a column of width 1/4", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 1 && tiles[0].right - tiles[0].left + 1 == 20
    });
}

#[test]
fn direct_binding_acts_without_the_prefix() {
    let env = TestEnv::new("config-direct");
    env.write_config("gband.bind('alt+h', gband.action.focus_column_left)");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x01\r");
    client.wait_for("the second tile focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused
    });
    client.wait_for_prompt();
    client.run("clear");
    client.wait_for_prompt();
    client.send(b"\x1bh");
    client.wait_for("the first tile focused", |screen| tiles(screen)[0].focused);
    client.send(b"echo left");
    client.wait_for_focused("the typed line", |lines| {
        lines.iter().any(|line| line.ends_with("$ echo left"))
    });
    let screen = client.screen();
    let second = tiles(&screen)[1].lines(&screen);
    let typed = second.iter().rfind(|line| !line.is_empty());
    assert!(typed.is_some_and(|line| line.ends_with('$')), "{second:?}");
}

#[test]
fn spawn_a_command_line() {
    let env = TestEnv::new("config-spawn");
    env.write_config(
        "gband.bind('alt+n', function() gband.spawn({ cmd = \"/bin/sh -c 'echo spawned; sleep 5'\" }) end)",
    );
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x1bn");
    client.wait_for("the spawned pane focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused && tiles[1].left == 40
    });
    client.wait_for_line("spawned");
}

#[test]
fn syntax_error_is_shown_on_the_bottom_row() {
    let env = TestEnv::new("config-syntax");
    let home = env.root.join("home");
    let path = home.join(".config").join("gband").join("init.lua");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "\n\n\n\n\n\n\n\n\n\n\nlocal = 1\n").unwrap();
    let client = Attached::start_with(&env, GBAND, &["attach"], 120, 24, |command| {
        command.env_remove("XDG_CONFIG_HOME");
        command.env("HOME", &home);
    });
    let expected = format!("{}:12:", path.display());
    client.wait_for("the error banner", |screen| {
        bottom_row(screen).starts_with(&expected)
    });
    wait_until(
        || env.log_text("client").contains(&expected),
        "the client log to record the error",
    );
    wait_until(
        || env.log_text("server").contains(&expected),
        "the server log to record the error",
    );
}

#[test]
fn broken_file_at_start_keeps_the_defaults() {
    let env = TestEnv::new("config-broken-start");
    env.write_config("gband.set { prefix = 'ctrl+b' }\nerror('broken')\n");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for("the error banner", |screen| {
        bottom_row(screen).contains("init.lua:2: broken")
    });
    client.wait_for_text("$");
    client.send(b"\x02\r");
    thread::sleep(Duration::from_millis(300));
    client.send(b"\x01\r");
    client.wait_for("two tiles with the second focused", |screen| {
        focused_top(screen) == Some(40) && tops(screen).len() == 2
    });
}

#[test]
fn broken_edit_keeps_the_running_configuration() {
    let env = TestEnv::new("config-broken-edit");
    env.write_config("gband.bind('alt+h', gband.action.focus_column_left)");
    let mut client = two_panes_first_focused(&env);
    client.send(b"\x01l");
    client.wait_for("the second tile focused", |screen| tiles(screen)[1].focused);
    let seen = reloads(&env, "client");
    env.write_config("gband.bind('alt+j', gband.action.focus_column_left)\nlocal = 1\n");
    client.wait_for("the error banner", |screen| {
        bottom_row(screen).contains("init.lua:2:")
    });
    assert_eq!(reloads(&env, "client"), seen);
    client.send(b"\x1bh");
    client.wait_for("the first tile focused", |screen| {
        focused_top(screen) == Some(0)
    });
    echo_keys(&mut client);
    client.send(b"\x1bj\r");
    client.wait_for_text("^[j");
}

#[test]
fn new_binding_without_restart() {
    let env = TestEnv::new("config-new-binding");
    fs::create_dir_all(env.config_home()).unwrap();
    let mut client = two_panes_first_focused(&env);
    let seen = reloads(&env, "client");
    env.write_config("gband.bind('alt+l', gband.action.focus_column_right)");
    wait_for_reload(&env, "client", seen);
    client.send(b"\x1bl");
    client.wait_for("the second tile focused", |screen| tiles(screen)[1].focused);
}

#[test]
fn editor_replaces_the_file() {
    let env = TestEnv::new("config-rename");
    env.write_config("");
    let mut client = two_panes_first_focused(&env);
    let (client_seen, server_seen) = (reloads(&env, "client"), reloads(&env, "server"));
    let temporary = env.init_lua().with_file_name("init.lua~");
    fs::write(
        &temporary,
        "gband.bind('alt+l', gband.action.focus_column_right)",
    )
    .unwrap();
    fs::rename(&temporary, env.init_lua()).unwrap();
    wait_for_reload(&env, "client", client_seen);
    wait_for_reload(&env, "server", server_seen);
    client.send(b"\x1bl");
    client.wait_for("the second tile focused", |screen| tiles(screen)[1].focused);
}

#[test]
fn file_removed_restores_the_defaults() {
    let env = TestEnv::new("config-removed");
    env.write_config("gband.bind('alt+h', gband.action.focus_column_left)");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    let seen = reloads(&env, "client");
    fs::remove_file(env.init_lua()).unwrap();
    wait_for_reload(&env, "client", seen);
    echo_keys(&mut client);
    client.send(b"\x1bh\r");
    client.wait_for_line("^[h");
}
