mod common;

use std::thread;
use std::time::Duration;

use common::*;
use gband_lua::DEFAULTS;

const RELOADED: &str = "configuration reloaded";

fn reloads(env: &TestEnv) -> usize {
    env.log_text("client").matches(RELOADED).count()
}

fn cell(screen: &Grid, row: u16, col: usize) -> String {
    screen
        .contents()
        .lines()
        .nth(usize::from(row))
        .unwrap_or_default()
        .chars()
        .nth(col)
        .map(String::from)
        .unwrap_or_default()
        .trim_end()
        .to_owned()
}

fn attached(env: &TestEnv) -> Attached {
    let mut client = Attached::start(env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(env);
    client
}

fn reload(env: &TestEnv, client: &Attached, source: &str) {
    let seen = reloads(env);
    env.write_config(source);
    wait_until(|| reloads(env) > seen, "a configuration reload");
    client.wait_for_prompt();
}

fn tput(client: &mut Attached, what: &str, expected: &str) {
    thread::sleep(Duration::from_millis(300));
    client.run(&format!("clear; tput {what}"));
    client.wait_for_line(expected);
}

fn sidebar_at(screen: &Grid, col: usize) -> bool {
    cell(screen, 0, col) == "I" && cell(screen, 2, col) == "1" && cell(screen, 3, col) == "2"
}

#[test]
fn height_beside_the_sidebar() {
    let env = TestEnv::new("sidebar-height");
    let mut client = attached(&env);
    client.wait_for("the sidebar on the left", |screen| sidebar_at(screen, 0));
    let tiles = client.tiles();
    assert_eq!(tiles.len(), 1);
    assert_eq!((tiles[0].top, tiles[0].bottom), (0, 23));
    assert_eq!((tiles[0].left, tiles[0].right), (1, 39));
    tput(&mut client, "lines", "22");
    tput(&mut client, "cols", "37");
}

#[test]
fn default_sidebar() {
    let env = TestEnv::new("sidebar-default");
    let client = attached(&env);
    client.wait_for("the default sidebar", |screen| {
        sidebar_at(screen, 0)
            && cell(screen, 1, 0).is_empty()
            && (4..24).all(|row| cell(screen, row, 0).is_empty())
    });
}

#[test]
fn turning_the_sidebar_off() {
    let env = TestEnv::new("sidebar-off");
    let mut client = attached(&env);
    client.wait_for("the sidebar on the left", |screen| sidebar_at(screen, 0));
    reload(
        &env,
        &client,
        &DEFAULTS.replace("gband.plugin(\"gband.sidebar\")", ""),
    );
    client.wait_for("a tile of 40×24 from column 0", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 1 && tiles[0].left == 0 && tiles[0].right == 39 && tiles[0].bottom == 23
    });
    tput(&mut client, "cols", "38");
}

#[test]
fn moving_the_sidebar() {
    let env = TestEnv::new("sidebar-right");
    let mut client = attached(&env);
    client.wait_for("the sidebar on the left", |screen| sidebar_at(screen, 0));
    reload(
        &env,
        &client,
        &DEFAULTS.replace(
            "gband.plugin(\"gband.sidebar\")",
            "gband.plugin(\"gband.sidebar\", { side = \"right\" })",
        ),
    );
    client.wait_for("the sidebar on the right", |screen| {
        let tiles = tiles(screen);
        sidebar_at(screen, 79) && tiles.len() == 1 && tiles[0].left == 0
    });
    tput(&mut client, "lines", "22");
}

#[test]
fn mode_letter_in_navigation_mode() {
    let env = TestEnv::new("sidebar-mode");
    let mut client = attached(&env);
    client.wait_for("the sidebar on the left", |screen| sidebar_at(screen, 0));
    client.send(b"\x00");
    client.wait_for("navigation mode in the sidebar", |screen| {
        cell(screen, 0, 0) == "N"
    });
    client.send(b"\x1b");
    client.wait_for("interactive mode again", |screen| cell(screen, 0, 0) == "I");
}
