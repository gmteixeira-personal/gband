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
    client.send(b"\x00n\r");
    client.wait_for("the second tile focused", second_focused);
    client.wait_for_prompt();
    client.shell_pid(env);
    client.send(b"\x00h\r");
    client.wait_for("the first tile focused", |screen| {
        focused_left(screen) == Some(1)
    });
    client
}

fn focused_left(screen: &Grid) -> Option<u16> {
    tiles(screen)
        .into_iter()
        .find(|tile| tile.focused)
        .map(|tile| tile.left)
}

fn second_focused(screen: &Grid) -> bool {
    focused_left(screen) == Some(40)
}

fn first_row(screen: &Grid) -> String {
    screen
        .contents()
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned()
}

fn error_logged(env: &TestEnv, expected: &str) {
    wait_until(
        || env.log_text("client").contains(expected),
        "the client log to record the error",
    );
}

fn tops(screen: &Grid) -> Vec<(u16, bool)> {
    (0..screen.size().cols)
        .filter_map(|col| {
            let cell = cell(screen, 0, col)?;
            (cell.symbol() == "╭").then(|| (col, cell.modifier.contains(Modifier::BOLD)))
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
    let env = TestEnv::without_settings("config-harness");
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
    let env = TestEnv::without_settings("config-first-run");
    let client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    assert_eq!(fs::read_to_string(env.defaults_lua()).unwrap(), DEFAULTS);
    for (style, preset) in gband_lua::KEY_STYLES {
        let path = gband_lua::key_style_file(&env.config_dir(), style);
        assert_eq!(fs::read_to_string(path).unwrap(), preset);
    }
    assert!(entries(&env.config_dir().join("user")).is_empty());
    for role in ["client", "server"] {
        let log = env.log_text(role);
        assert!(!log.contains("cannot prepare"), "{log}");
        assert!(!log.contains("configuration error"), "{log}");
    }
    let defaults = env.config_dir().join("defaults");
    for file in [
        "lua/gband/sidebar.lua",
        "lua/gband/win.lua",
        "lua/gband/prelude.lua",
        "lua/gband/keylist.lua",
        "lua/gband/desktop.lua",
        "colors/nord.lua",
        "colors/gruvbox.lua",
    ] {
        assert_eq!(
            fs::read_to_string(defaults.join(file)).unwrap(),
            bundled(file),
            "{file}"
        );
    }
}

fn bundled(file: &str) -> &'static str {
    gband_lua::bundled_files()
        .find(|(path, _)| path == file)
        .unwrap_or_else(|| panic!("{file} is not bundled"))
        .1
}

fn sidebar_letter(screen: &Grid) -> String {
    first_row(screen)
        .chars()
        .next()
        .map(String::from)
        .unwrap_or_default()
}

const SIDEBAR: &str = "lua/gband/sidebar.lua";
const ROOT_LETTER: &str = "    return multi and \"M\" or \"I\"\n";

#[test]
fn edited_bundled_sources_are_restored() {
    let env = TestEnv::new("config-sources-restored");
    let copy = env.config_dir().join("defaults").join(SIDEBAR);
    fs::create_dir_all(copy.parent().unwrap()).unwrap();
    fs::write(&copy, "error('edited')\n").unwrap();
    let client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    assert_eq!(fs::read_to_string(&copy).unwrap(), bundled(SIDEBAR));
    client.wait_for("the sidebar", |screen| sidebar_letter(screen) == "I");
}

#[test]
fn copies_have_no_effect() {
    let env = TestEnv::new("config-copies-no-effect");
    let client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.wait_for("the sidebar", |screen| sidebar_letter(screen) == "I");
    let copy = env.config_dir().join("defaults").join(SIDEBAR);
    let edited = bundled(SIDEBAR).replacen(ROOT_LETTER, "    return \"i\"\n", 1);
    assert_ne!(edited, bundled(SIDEBAR));
    fs::write(&copy, edited).unwrap();
    let seen = reloads(&env, "client");
    env.write_config(DEFAULTS);
    wait_for_reload(&env, "client", seen);
    client.wait_for_prompt();
    thread::sleep(Duration::from_millis(300));
    client.wait_for("the bundled sidebar", |screen| {
        sidebar_letter(screen) == "I"
    });
}

#[test]
fn copy_to_override() {
    let env = TestEnv::new("config-copy-override");
    let client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.wait_for("the sidebar", |screen| sidebar_letter(screen) == "I");
    let source = fs::read_to_string(env.config_dir().join("defaults").join(SIDEBAR)).unwrap();
    let edited = source.replacen(ROOT_LETTER, "    return \"i\"\n", 1);
    assert_ne!(edited, source);
    let copy = env.config_dir().join("user").join(SIDEBAR);
    fs::create_dir_all(copy.parent().unwrap()).unwrap();
    fs::write(&copy, edited).unwrap();
    let seen = reloads(&env, "client");
    env.write_config(DEFAULTS);
    wait_for_reload(&env, "client", seen);
    client.wait_for("the copied sidebar", |screen| sidebar_letter(screen) == "i");
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
    let config_dir = env.config_dir();
    fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o555)).unwrap();
    let _writable = Writable(&config_dir);
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
    client.send(b"\x00n");
    client.wait_for("the second tile focused", second_focused);
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
    client.send(b"\x00n\r");
    client.wait_for("the second tile focused", second_focused);
    client.wait_for_prompt();
    client.shell_pid(&env);
    let seen = reloads(&env, "client");
    let copy = fs::read_to_string(env.defaults_lua()).unwrap();
    env.write_config(&format!("{copy}\ngband.set {{ prefix = 'ctrl+b' }}\n"));
    wait_for_reload(&env, "client", seen);
    client.send(b"\x02q\r");
    client.wait_for("one tile", |screen| {
        tops(screen) == [(1, true)] && first_row(screen).chars().skip(41).all(|c| c == ' ')
    });
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
    client.send(b"\x00n");
    client.wait_for("the second tile focused", second_focused);
}

#[test]
fn no_configuration_file() {
    let env = TestEnv::new("config-none");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00n");
    client.wait_for("the second tile focused", second_focused);
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
    client.wait_for("the second tile focused", second_focused);
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
fn syntax_error_is_shown_as_the_error_marker() {
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
    client.wait_for("the error marker", sidebar_error);
    error_logged(&env, &expected);
}

#[test]
fn broken_file_at_start_keeps_the_defaults() {
    let env = TestEnv::new("config-broken-start");
    env.write_config("gband.set { prefix = 'ctrl+b' }\nerror('broken')\n");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for("the error marker", sidebar_error);
    error_logged(&env, "init.lua:2: broken");
    client.wait_for_text("$");
    client.send(b"\x02\r");
    thread::sleep(Duration::from_millis(300));
    client.send(b"\x00n");
    client.wait_for("the second tile focused", |screen| {
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
    client.send(b"\x00l\r");
    client.wait_for("the second tile focused", second_focused);
    let seen = reloads(&env, "client");
    env.write_config(&with_defaults(
        "gband.bind('alt+j', gband.action.focus_column_left)\nlocal = 1",
    ));
    client.wait_for("the error marker", sidebar_error);
    error_logged(&env, "init.lua:2:");
    assert_eq!(reloads(&env, "client"), seen);
    client.send(b"\x1bh");
    client.wait_for("the first tile focused", |screen| {
        focused_top(screen) == Some(1)
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
    client.wait_for("the second tile focused", second_focused);
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
    client.wait_for("the second tile focused", second_focused);
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
    client.send(b"\x00n");
    client.wait_for("a second column of width 1/3", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused && tiles[0].left == 1 && tiles[1].left == 27
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
    client.send(b"\x00n\r");
    client.wait_for("the second tile focused", second_focused);
    client.wait_for_prompt();
    client.send(b"\x1bg");
    client.wait_for("the first tile focused", |screen| {
        focused_left(screen) == Some(1)
    });
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
    client.wait_for("the error marker", sidebar_error);
    error_logged(&env, "spin: ");
    client.wait_for_text("$");
    client.send(b"\x00n");
    client.wait_for("the second tile focused", |screen| {
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

fn open_second_window(client: &mut Attached, env: &TestEnv) -> (u16, u16) {
    client.wait_for_prompt();
    client.shell_pid(env);
    client.send(b"\x00n");
    let columns = Cell::new((0, 0));
    client.wait_for("the second tile focused", |screen| {
        let focused = tiles(screen)
            .into_iter()
            .find(|tile| tile.focused && tile.left > 20);
        if let Some(tile) = &focused {
            columns.set((tile.left, tile.right - tile.left + 1));
        }
        focused.is_some()
    });
    columns.get()
}

#[test]
fn width_set_for_the_server() {
    let env = TestEnv::new("config-server-width");
    env.write_server_config("gband.opt.default_column_width = 1/3");
    let mut client = Attached::start(&env, 80, 24);
    assert_eq!(open_second_window(&mut client, &env), (27, 26));
}

#[test]
fn no_server_configuration_file() {
    let env = TestEnv::new("config-no-server-file");
    let mut client = Attached::start(&env, 80, 24);
    assert_eq!(open_second_window(&mut client, &env), (40, 39));
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
    client.wait_for("the error marker", sidebar_error);
    error_logged(&env, &expected);
    client.wait_for_text("$");
    client.send(b"\x00n");
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
    client.wait_for("the error marker", sidebar_error);
    error_logged(&env, &expected);
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
    client.wait_for("the error marker", sidebar_error);
    error_logged(&env, "init.lua:1: client only");
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
    let (left, width) = open_second_window(&mut client, &env);
    assert_eq!((left, width), (40, 26));
}

fn label_plugin(env: &TestEnv, label: &str) {
    env.write_client_plugin(
        "label",
        &format!("gband.bar.add({{ side = 'right', size = 6, lines = {{ '{label}' }} }})\n"),
    );
}

fn loads(env: &TestEnv) -> Vec<(String, u64)> {
    let output = env
        .command(GBAND, &["errors", "--json"])
        .env_remove("GBAND_SESSION")
        .output()
        .unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    entries
        .iter()
        .map(|entry| {
            (
                entry["process"].as_str().unwrap().to_owned(),
                entry["load"].as_u64().unwrap(),
            )
        })
        .collect()
}

fn own_errors(env: &TestEnv) -> String {
    let output = env
        .command(GBAND, &["errors"])
        .env_remove("GBAND_SESSION")
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn forced_reload_reads_a_changed_plugin_and_returns_to_typing() {
    let env = TestEnv::new("config-forced-plugin");
    label_plugin(&env, "old");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_text("old");
    client.wait_for_prompt();
    label_plugin(&env, "new");
    client.send(b"\x00!");
    client.wait_for_text("new");
    client.run("echo hi");
    client.wait_for_line("hi");
}

#[test]
fn forced_reload_reads_a_changed_server_plugin() {
    let env = TestEnv::new("config-forced-server-plugin");
    let emitting = |word: &str| {
        env.write_server_plugin(
            "note",
            &format!("gband.on('WindowOpened', function() gband.emit('note', '{word}') end)\n"),
        );
    };
    emitting("one");
    env.write_client_plugin(
        "show",
        "local bar = gband.bar.add({ id = 'note', side = 'right', size = 6 })\n\
         gband.on('ServerEvent', function(ev) gband.bar.set_lines(bar, { ev.data }) end, { pattern = 'note' })\n",
    );
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    emitting("two");
    client.send(b"\x00!");
    wait_until(
        || loads(&env).first() == Some(&("server".to_owned(), 2)),
        "the server's forced load",
    );
    client.send(b"\x00n\r");
    client.wait_for_text("two");
    assert!(!client.contents().contains("one"), "{}", client.contents());
}

#[test]
fn broken_plugin_keeps_the_configuration_in_use() {
    let env = TestEnv::new("config-forced-broken");
    env.write_config(&with_defaults(
        "gband.bind('alt+l', gband.action.focus_column_right)",
    ));
    env.write_client_plugin("fragile", "local fine = true\n");
    let mut client = two_windows_first_focused(&env);
    env.write_client_plugin("fragile", "error('bad')\n");
    client.send(b"\x00!");
    wait_until(|| own_errors(&env).contains("bad"), "the plugin error");
    client.wait_for("the error marker", sidebar_error);
    client.send(b"\x1bl");
    client.wait_for("the second tile focused", second_focused);
}

#[test]
fn forced_reload_leaves_other_clients_alone() {
    let env = TestEnv::new("config-forced-others");
    label_plugin(&env, "old");
    let mut first = Attached::start(&env, 80, 24);
    first.wait_for_text("old");
    first.wait_for_prompt();
    let second = Attached::start(&env, 80, 24);
    second.wait_for_text("old");
    label_plugin(&env, "new");
    first.send(b"\x00!");
    first.wait_for_text("new");
    thread::sleep(Duration::from_millis(300));
    let contents = second.contents();
    assert!(
        contents.contains("old") && !contents.contains("new"),
        "{contents}"
    );
    let loaded = loads(&env);
    assert_eq!(
        loaded.iter().filter(|(_, load)| *load == 2).count(),
        2,
        "{loaded:?}"
    );
}
