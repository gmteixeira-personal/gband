mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::thread;
use std::time::Duration;

use common::*;

const MODAL_LINE: &str = "modal   C-space enters a mode, keys repeat until Escape";
const DIRECT_LINE: &str = "direct  C-space then one key per action, back to typing";

fn row(screen: &Grid, index: usize) -> String {
    screen
        .contents()
        .lines()
        .nth(index)
        .unwrap_or_default()
        .to_owned()
}

fn sidebar(screen: &Grid, index: usize) -> String {
    row(screen, index).chars().take(1).collect()
}

fn chooser_open(screen: &Grid) -> bool {
    let contents = screen.contents();
    contents.contains(MODAL_LINE) && contents.contains(DIRECT_LINE)
}

fn mode(screen: &Grid, letter: &str) -> bool {
    sidebar(screen, 0) == letter
}

fn focused_shows(screen: &Grid, label: &str) -> bool {
    focused_lines(screen).iter().any(|line| line == label)
}

fn second_window(screen: &Grid) -> bool {
    let lines = focused_lines(screen);
    !lines.iter().any(|line| line == "W1")
        && lines
            .iter()
            .rfind(|line| !line.is_empty())
            .is_some_and(|line| line.ends_with('$'))
}

fn labelled(client: &mut Attached, label: &str) {
    client.run(&format!("clear; echo {label}"));
    client.wait_for_line(label);
}

fn saved(env: &TestEnv) -> Option<String> {
    fs::read_to_string(env.key_style_lua()).ok()
}

fn offered(env: &TestEnv) -> Attached {
    let client = Attached::start(env, 80, 24);
    client.wait_for("the key style chooser", |screen| {
        chooser_open(screen) && mode(screen, "I")
    });
    client
}

fn reloads(env: &TestEnv) -> usize {
    env.log_text("client")
        .matches("configuration reloaded")
        .count()
}

fn pick_direct(client: &mut Attached, env: &TestEnv) {
    let before = reloads(env);
    client.send(b"j");
    thread::sleep(Duration::from_millis(100));
    client.send(b"\r");
    client.wait_for("the chooser to close", |screen| !chooser_open(screen));
    wait_until(
        || saved(env).as_deref() == Some("return \"direct\"\n"),
        "the direct style saved",
    );
    wait_until(|| reloads(env) > before, "the configuration to reload");
}

fn escape(client: &mut Attached) {
    client.send(b"\x1b");
    client.wait_for("the chooser to close", |screen| !chooser_open(screen));
}

#[test]
fn first_start_offers_the_chooser() {
    let env = TestEnv::without_key_style("keystyle-first-start");
    let client = offered(&env);
    assert_eq!(saved(&env), None);
    drop(client);
}

#[test]
fn pick_the_direct_style() {
    let env = TestEnv::without_key_style("keystyle-pick-direct");
    let mut client = offered(&env);
    pick_direct(&mut client, &env);
    client.wait_for_prompt();
    client.shell_pid(&env);
    labelled(&mut client, "W1");
    client.send(b"\x00n");
    client.wait_for("the second window", second_window);
    labelled(&mut client, "W2");
    client.send(b"\x00h");
    client.wait_for("the first window focused", |screen| {
        focused_shows(screen, "W1")
    });
    client.run("echo left");
    client.wait_for("left in the first window", |screen| {
        focused_lines(screen).iter().any(|line| line == "left")
    });
}

#[test]
fn escape_saves_nothing_and_the_next_start_offers_again() {
    let env = TestEnv::without_key_style("keystyle-escape");
    let mut client = offered(&env);
    escape(&mut client);
    assert_eq!(saved(&env), None);
    client.wait_for_prompt();
    client.send(b"\x00D");
    client.wait_for_text("[detached]");
    client.wait_exit();
    let again = offered(&env);
    drop(again);
}

#[test]
fn no_offer_after_a_reload() {
    let env = TestEnv::without_key_style("keystyle-reload");
    let extra = env.config_dir().join("user").join("lua").join("extra.lua");
    fs::create_dir_all(extra.parent().unwrap()).unwrap();
    let mut client = offered(&env);
    escape(&mut client);
    let before = reloads(&env);
    fs::write(&extra, "return {}").unwrap();
    wait_until(|| reloads(&env) > before, "the configuration to reload");
    thread::sleep(Duration::from_millis(300));
    assert!(!chooser_open(&client.screen()), "{}", client.contents());
}

#[test]
fn no_offer_with_a_saved_style() {
    let env = TestEnv::new("keystyle-saved");
    let client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.wait_for("the sidebar", |screen| mode(screen, "I"));
    thread::sleep(Duration::from_millis(300));
    assert!(!chooser_open(&client.screen()), "{}", client.contents());
}

#[test]
fn no_offer_with_an_own_configuration() {
    let env = TestEnv::without_key_style("keystyle-own");
    env.write_config("gband.bind('alt+h', gband.action.focus_column_left)");
    let client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    thread::sleep(Duration::from_millis(500));
    assert!(!chooser_open(&client.screen()), "{}", client.contents());
}

#[test]
fn chosen_from_the_lua_prompt_leaves_the_user_file() {
    let env = TestEnv::without_key_style("keystyle-prompt");
    let source = "gband.keystyle.use()\n\
                  gband.plugin('gband.sidebar')\n\
                  gband.bind('alt+h', gband.action.focus_column_left)\n";
    env.write_config(source);
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    labelled(&mut client, "W1");
    client.send(b"\x00n");
    client.wait_for("the second window", second_window);
    labelled(&mut client, "W2");
    let line = "gband.keystyle.choose()";
    client.send(b"\x00:");
    client.send(line.as_bytes());
    client.wait_for("the prompt holding the line", |screen| {
        screen.contents().contains(&format!(":{line}"))
    });
    client.send(b"\r");
    client.wait_for("the chooser", chooser_open);
    pick_direct(&mut client, &env);
    assert_eq!(fs::read_to_string(env.user_lua()).unwrap(), source);
    client.send(b"\x1bh");
    client.wait_for("the first window focused", |screen| {
        focused_shows(screen, "W1")
    });
    client.send(b"\x00l");
    client.wait_for("the second window focused", |screen| {
        focused_shows(screen, "W2")
    });
    client.run("echo direct");
    client.wait_for("typed text in the second window", |screen| {
        focused_lines(screen).iter().any(|line| line == "direct")
    });
}

#[test]
fn cannot_save() {
    let env = TestEnv::without_key_style("keystyle-read-only");
    let user = env.config_dir().join("user");
    fs::create_dir_all(&user).unwrap();
    fs::set_permissions(&user, fs::Permissions::from_mode(0o555)).unwrap();
    if fs::write(user.join("probe"), "").is_ok() {
        return;
    }
    let mut client = offered(&env);
    client.send(b"j");
    thread::sleep(Duration::from_millis(100));
    client.send(b"\r");
    client.wait_for("the error marker", |screen| {
        !chooser_open(screen) && sidebar(screen, 23) == "!"
    });
    fs::set_permissions(&user, fs::Permissions::from_mode(0o755)).unwrap();
    let log = env.log_text("client");
    assert!(log.contains("user/keystyle.lua"), "{log}");
    assert_eq!(saved(&env), None);
    client.send(b"\x00");
    client.wait_for("navigation mode", |screen| mode(screen, "N"));
}

fn sgr(client: &mut Attached, code: u16, (col, row): (u16, u16), end: char) {
    client.send(format!("\x1b[<{code};{};{}{end}", col + 1, row + 1).as_bytes());
}

fn focused_tile(client: &Attached) -> Tile {
    client
        .tiles()
        .into_iter()
        .find(|tile| tile.focused)
        .unwrap_or_else(|| panic!("no focused tile:\n{}", client.contents()))
}

fn direct_with_two_windows(name: &str) -> (TestEnv, Attached) {
    let env = TestEnv::new(name);
    env.save_key_style("direct");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    labelled(&mut client, "W1");
    client.send(b"\x00n");
    client.wait_for("the second window", second_window);
    labelled(&mut client, "W2");
    (env, client)
}

#[test]
fn drag_after_the_prefix_with_the_direct_key_style() {
    let (_env, mut client) = direct_with_two_windows("keystyle-direct-drag");
    client.send(b"\x00v");
    client.wait_for("the floating box", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && tile.top > 0)
    });
    let window = focused_tile(&client);
    let (left, top) = (window.left, window.top);
    client.send(b"\x00");
    sgr(&mut client, 0, (left + 10, top + 3), 'M');
    sgr(&mut client, 32, (left + 16, top + 3), 'M');
    sgr(&mut client, 0, (left + 16, top + 3), 'm');
    client.wait_for("the box moved 6 cells right", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && (tile.left, tile.top) == (left + 6, top))
    });
    client.run("echo after");
    client.wait_for("keys reach the window", |screen| {
        focused_lines(screen).iter().any(|line| line == "after")
    });
}

#[test]
fn other_press_after_the_prefix_with_the_direct_key_style() {
    let (_env, mut client) = direct_with_two_windows("keystyle-direct-press");
    let second = focused_tile(&client);
    client.send(b"\x00");
    let cell = (second.left - 20, second.top + 3);
    sgr(&mut client, 16, cell, 'M');
    sgr(&mut client, 16, cell, 'm');
    client.send(b"x");
    client.wait_for("x in the focused window", |screen| {
        focused_lines(screen)
            .iter()
            .any(|line| line.ends_with("$ x"))
    });
    assert_eq!(focused_tile(&client).left, second.left);
}
