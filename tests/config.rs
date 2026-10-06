mod common;

use std::cell::Cell;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::thread;
use std::time::Duration;

use common::*;
use gband_lua::DEFAULTS;
use gband_test_support::cell;
use ratatui::style::Modifier;

const RELOADED: &str = "configuration reloaded";

fn with_defaults(source: &str) -> String {
    format!("{source}\n{DEFAULTS}")
}

fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn reloads(env: &TestEnv, role: &str) -> usize {
    env.log_text(role).matches(RELOADED).count()
}

fn wait_for_reload(env: &TestEnv, role: &str, seen: usize) {
    wait_until(|| reloads(env, role) > seen, "a configuration reload");
}

fn two_windows_first_focused(env: &TestEnv) -> Attached {
    let mut client = Attached::start(env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(env);
    client.send(b"\x00\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused
    });
    client.wait_for_prompt();
    client.shell_pid(env);
    client.send(b"\x00h");
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
    let data = command
        .get_envs()
        .find(|(name, _)| *name == "XDG_DATA_HOME")
        .and_then(|(_, value)| value);
    assert_eq!(data, Some(env.data_home().as_os_str()));
    assert!(env.data_home().starts_with(&env.root));
    assert!(!env.config_dir().exists());
    let client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    assert!(env.defaults_lua().exists());
    assert!(!env.user_lua().exists());
    env.write_config("");
    assert!(env.user_lua().exists());
}

#[test]
fn first_run() {
    let env = TestEnv::new("config-first-run");
    let client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    assert_eq!(fs::read_to_string(env.defaults_lua()).unwrap(), DEFAULTS);
    assert!(entries(&env.config_dir().join("user")).is_empty());
    for role in ["client", "server"] {
        let log = env.log_text(role);
        assert!(!log.contains("cannot prepare"), "{log}");
        assert!(!log.contains("configuration error"), "{log}");
    }
}

struct Writable<'a>(&'a Path);

impl Drop for Writable<'_> {
    fn drop(&mut self) {
        let _ = fs::set_permissions(self.0, fs::Permissions::from_mode(0o755));
    }
}

#[test]
fn read_only_configuration_directory() {
    let env = TestEnv::new("config-read-only");
    let config_home = env.config_home();
    fs::create_dir_all(&config_home).unwrap();
    fs::set_permissions(&config_home, fs::Permissions::from_mode(0o555)).unwrap();
    let _writable = Writable(&config_home);
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    wait_until(
        || {
            env.log_text("client")
                .contains("cannot prepare the configuration directory")
        },
        "the client log to record the failure",
    );
    client.send(b"\x00\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused
    });
}

#[test]
fn user_file_replaces_the_defaults() {
    let env = TestEnv::new("config-replace");
    env.write_config("gband.bind('alt+h', gband.action.focus_column_left)");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    echo_keys(&mut client);
    client.send(b"\x1bh\x00q\r");
    client.wait_for_line("^@q");
}

#[test]
fn another_prefix_key_in_a_copy_of_the_defaults() {
    let env = TestEnv::new("config-copied-prefix");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused
    });
    client.wait_for_prompt();
    client.shell_pid(&env);
    let seen = reloads(&env, "client");
    let copy = fs::read_to_string(env.defaults_lua()).unwrap();
    env.write_config(&format!("{copy}\ngband.set {{ prefix = 'ctrl+b' }}\n"));
    wait_for_reload(&env, "client", seen);
    client.send(b"\x02q");
    client.wait_for("one tile", |screen| tiles(screen).len() == 1);
    client.wait_for_prompt();
    echo_keys(&mut client);
    client.send(b"\x00\r");
    client.wait_for_line("^@");
}

#[test]
fn defaults_file_edited_while_running() {
    let env = TestEnv::new("config-defaults-edited");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    fs::write(env.defaults_lua(), "gband.set { prefix = 'ctrl+b' }\n").unwrap();
    thread::sleep(Duration::from_secs(1));
    assert_eq!(reloads(&env, "client"), 0);
    client.send(b"\x00\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused
    });
}

#[test]
fn no_configuration_file() {
    let env = TestEnv::new("config-none");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00\r");
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
    env.write_server_config("gband.set { width_presets = { 1/4, 1/2 } }");
    env.write_config("gband.bind('prefix r', gband.action.cycle_column_width)");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.send(b"\x00r");
    client.wait_for("a column of width 1/4", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 1 && tiles[0].right - tiles[0].left + 1 == 20
    });
}

#[test]
fn default_prefix() {
    let env = TestEnv::new("config-default-prefix");
    env.write_config(
        "gband.bind('prefix q', gband.action.close_window)\ngband.bind('prefix enter', gband.action.open_window)",
    );
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused
    });
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00q");
    client.wait_for("one tile", |screen| tiles(screen).len() == 1);
    client.wait_for_prompt();
    echo_keys(&mut client);
    client.send(b"\x01\r");
    client.wait_for_line("^A");
}

#[test]
fn direct_binding_acts_without_the_prefix() {
    let env = TestEnv::new("config-direct");
    env.write_config(
        "gband.bind('alt+h', gband.action.focus_column_left)\ngband.bind('prefix enter', gband.action.open_window)",
    );
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00\r");
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
    client.wait_for("the spawned window focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused && tiles[1].left == 40
    });
    client.wait_for_line("spawned");
}

#[test]
fn syntax_error_is_shown_on_the_bottom_row() {
    let env = TestEnv::new("config-syntax");
    let home = env.root.join("home");
    let path = home
        .join(".config")
        .join("gband")
        .join("user")
        .join("init.lua");
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
    client.send(b"\x00\r");
    client.wait_for("two tiles with the second focused", |screen| {
        focused_top(screen) == Some(40) && tops(screen).len() == 2
    });
}

#[test]
fn broken_edit_keeps_the_running_configuration() {
    let env = TestEnv::new("config-broken-edit");
    env.write_config(&with_defaults(
        "gband.bind('alt+h', gband.action.focus_column_left)",
    ));
    let mut client = two_windows_first_focused(&env);
    client.send(b"\x00l");
    client.wait_for("the second tile focused", |screen| tiles(screen)[1].focused);
    let seen = reloads(&env, "client");
    env.write_config(&with_defaults(
        "gband.bind('alt+j', gband.action.focus_column_left)\nlocal = 1",
    ));
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
    let mut client = two_windows_first_focused(&env);
    let seen = reloads(&env, "client");
    env.write_config("gband.bind('alt+l', gband.action.focus_column_right)");
    wait_for_reload(&env, "client", seen);
    client.send(b"\x1bl");
    client.wait_for("the second tile focused", |screen| tiles(screen)[1].focused);
}

#[test]
fn editor_replaces_the_file() {
    let env = TestEnv::new("config-rename");
    env.write_config(DEFAULTS);
    let mut client = two_windows_first_focused(&env);
    let (client_seen, server_seen) = (reloads(&env, "client"), reloads(&env, "server"));
    let temporary = env.user_lua().with_file_name("init.lua~");
    fs::write(
        &temporary,
        with_defaults("gband.bind('alt+l', gband.action.focus_column_right)"),
    )
    .unwrap();
    fs::rename(&temporary, env.user_lua()).unwrap();
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
    fs::remove_file(env.user_lua()).unwrap();
    wait_for_reload(&env, "client", seen);
    echo_keys(&mut client);
    client.send(b"\x1bh\r");
    client.wait_for_line("^[h");
}

#[test]
fn plugin_sets_a_width_option() {
    let env = TestEnv::new("config-plugin-width");
    env.write_server_plugin("widths", "gband.opt.default_column_width = 1/3\n");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00\r");
    client.wait_for("a second column of width 1/3", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused && tiles[1].left == 26
    });
    assert!(!env.log_text("server").contains("configuration error"));
}

#[test]
fn default_configuration_with_a_plugin() {
    let env = TestEnv::new("config-default-plugin");
    env.write_client_plugin(
        "keys",
        "gband.keymap.set('root', 'alt+g', gband.action.focus_column_left)\n",
    );
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused
    });
    client.wait_for_prompt();
    client.send(b"\x1bg");
    client.wait_for("the first tile focused", |screen| tiles(screen)[0].focused);
}

#[test]
fn infinite_loop_in_setup() {
    let env = TestEnv::new("config-setup-loop");
    env.write_plugin_file(
        "spin",
        "lua/spin/init.lua",
        "return { setup = function() while true do end end }\n",
    );
    env.write_config("gband.plugin('spin')\n");
    let mut client = Attached::start(&env, 120, 24);
    client.wait_for("the plugin error banner", |screen| {
        let row = bottom_row(screen);
        row.starts_with("spin: ") && row.contains("instruction limit exceeded")
    });
    client.wait_for_text("$");
    client.send(b"\x00\r");
    client.wait_for("two tiles with the second focused", |screen| {
        tops(screen).len() == 2 && focused_top(screen) == Some(60)
    });
    wait_until(
        || {
            env.log_text("client")
                .contains("instruction limit exceeded")
        },
        "the log to record the stopped plugin",
    );
}

fn open_second_window(client: &mut Attached, env: &TestEnv) -> (u16, u16, u16) {
    client.wait_for_prompt();
    client.shell_pid(env);
    client.send(b"\x00\r");
    let columns = Cell::new((0, 0, 0));
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        let found = tiles.len() == 2 && tiles[1].focused;
        if found {
            columns.set((
                tiles[0].right - tiles[0].left + 1,
                tiles[1].left,
                tiles[1].right - tiles[1].left + 1,
            ));
        }
        found
    });
    columns.get()
}

#[test]
fn width_set_for_the_server() {
    let env = TestEnv::new("config-server-width");
    env.write_server_config("gband.opt.default_column_width = 1/3");
    let mut client = Attached::start(&env, 80, 24);
    let (_, left, _) = open_second_window(&mut client, &env);
    assert_eq!(left, 26);
}

#[test]
fn no_server_configuration_file() {
    let env = TestEnv::new("config-no-server-file");
    let mut client = Attached::start(&env, 80, 24);
    let (first, left, _) = open_second_window(&mut client, &env);
    assert_eq!((first, left), (40, 40));
    assert!(!env.log_text("server").contains("configuration error"));
    assert!(env.server_lua().parent().unwrap().is_dir());
    assert_eq!(
        fs::read_to_string(env.config_dir().join("defaults").join("server.lua")).unwrap(),
        gband_lua::DEFAULTS_SERVER
    );
}

#[test]
fn broken_server_file_at_start() {
    let env = TestEnv::new("config-broken-server");
    env.write_server_config("gband.opt.default_column_width = 1/3\nerror('broken')\n");
    let mut client = Attached::start(&env, 120, 24);
    let expected = format!("server: {}:2: broken", env.server_lua().display());
    client.wait_for("the server error banner", |screen| {
        bottom_row(screen).starts_with(&expected)
    });
    client.wait_for_text("$");
    client.send(b"\x00\r");
    client.wait_for("a second column of width 1/2", |screen| {
        tops(screen).len() == 2 && focused_top(screen) == Some(60)
    });
}

#[test]
fn server_error_in_the_client() {
    let env = TestEnv::new("config-server-syntax");
    env.write_server_config("gband.set {}\nlocal = 1\n");
    let client = Attached::start(&env, 120, 24);
    let expected = format!("server: {}:2:", env.server_lua().display());
    client.wait_for("the server error banner", |screen| {
        bottom_row(screen).starts_with(&expected)
    });
    wait_until(
        || {
            env.log_text("server")
                .contains(&format!("{}:2:", env.server_lua().display()))
        },
        "the server log to record the error",
    );
}

#[test]
fn client_file_not_evaluated() {
    let env = TestEnv::new("config-client-not-in-server");
    env.write_config("error('client only')\n");
    let client = Attached::start(&env, 120, 24);
    client.wait_for("the client error banner", |screen| {
        bottom_row(screen).contains("init.lua:1: client only")
    });
    wait_until(
        || env.log_text("server").contains("listening"),
        "the server to start",
    );
    assert!(!env.log_text("server").contains("client only"));
}

#[test]
fn new_default_width() {
    let env = TestEnv::new("config-new-width");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    let seen = reloads(&env, "server");
    env.write_server_config("gband.opt.default_column_width = 1/3");
    wait_for_reload(&env, "server", seen);
    let (first, left, second) = open_second_window(&mut client, &env);
    assert_eq!((first, left), (40, 40));
    assert!((26..=27).contains(&second), "{second}");
}
