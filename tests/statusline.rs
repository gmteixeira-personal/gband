mod common;

use std::thread;
use std::time::Duration;

use common::*;
use gband_lua::DEFAULTS;

const RELOADED: &str = "configuration reloaded";

fn reloads(env: &TestEnv) -> usize {
    env.log_text("client").matches(RELOADED).count()
}

fn row(screen: &Grid, index: u16) -> String {
    screen
        .contents()
        .lines()
        .nth(usize::from(index))
        .unwrap_or_default()
        .to_owned()
}

fn attached(env: &TestEnv) -> Attached {
    attached_with_width(env, 80)
}

fn attached_with_width(env: &TestEnv, cols: u16) -> Attached {
    let mut client = Attached::start(env, cols, 24);
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

fn bar(screen: &Grid, index: u16) -> String {
    row(screen, index)
        .chars()
        .take(20)
        .collect::<String>()
        .trim_end()
        .to_owned()
}

fn tput(client: &mut Attached, what: &str, expected: &str) {
    thread::sleep(Duration::from_millis(300));
    client.run(&format!("clear; tput {what}"));
    client.wait_for_line(expected);
}

#[test]
fn height_beside_the_status_line() {
    let env = TestEnv::new("statusline-height");
    let mut client = attached(&env);
    client.wait_for("the status line on the left", |screen| {
        bar(screen, 0) == "band 1" && bar(screen, 23) == "1/1"
    });
    let tiles = client.tiles();
    assert_eq!(tiles.len(), 1);
    assert_eq!((tiles[0].top, tiles[0].bottom), (0, 23));
    assert_eq!((tiles[0].left, tiles[0].right), (20, 59));
    tput(&mut client, "lines", "22");
    tput(&mut client, "cols", "38");
}

#[test]
fn turning_the_status_line_off() {
    let env = TestEnv::new("statusline-off");
    let mut client = attached(&env);
    client.wait_for_text("band 1");
    reload(
        &env,
        &client,
        &DEFAULTS.replace("gband.plugin(\"gband.statusline\")", ""),
    );
    client.wait_for("a tile of 40×24 from column 0", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 1 && tiles[0].left == 0 && tiles[0].right == 39 && tiles[0].bottom == 23
    });
    assert!(!client.contents().contains("band 1"));
    tput(&mut client, "cols", "38");
}

#[test]
fn moving_the_status_line() {
    let env = TestEnv::new("statusline-right");
    let mut client = attached(&env);
    client.wait_for_text("band 1");
    reload(
        &env,
        &client,
        &DEFAULTS.replace(
            "gband.plugin(\"gband.statusline\")",
            "gband.plugin(\"gband.statusline\", { side = \"right\" })",
        ),
    );
    client.wait_for("the status line on the right", |screen| {
        let tiles = tiles(screen);
        row(screen, 0)
            .chars()
            .skip(60)
            .collect::<String>()
            .starts_with("band 1")
            && tiles.len() == 1
            && tiles[0].left == 0
    });
    tput(&mut client, "lines", "22");
}

#[test]
fn mode_segment_in_navigation_mode() {
    let env = TestEnv::new("statusline-mode");
    let mut client = attached(&env);
    client.wait_for_text("band 1");
    client.send(b"\x00");
    client.wait_for("navigation mode in the status line", |screen| {
        bar(screen, 0) == "band 1" && bar(screen, 1) == "navigation"
    });
    client.send(b"x");
    thread::sleep(Duration::from_millis(200));
    assert_eq!(bar(&client.screen(), 1), "navigation");
    client.send(b"\r");
    client.wait_for("navigation mode gone", |screen| {
        bar(screen, 1) == "C-space navigation"
    });
}

#[test]
fn default_segments() {
    let env = TestEnv::new("statusline-hints");
    let client = attached(&env);
    client.wait_for("the default segments", |screen| {
        bar(screen, 0) == "band 1"
            && bar(screen, 1) == "C-space navigation"
            && (2..23).all(|index| bar(screen, index).is_empty())
            && bar(screen, 23) == "1/1"
    });
}

#[test]
fn looping_component_is_stopped() {
    let env = TestEnv::new("statusline-loop");
    env.write_client_plugin(
        "spin",
        "gband.ui.statusline.add({ render = function() while true do end end })",
    );
    let mut client = attached_with_width(&env, 200);
    client.wait_for("the error item", |screen| bar(screen, 0) == "error");
    wait_until(
        || {
            let log = env.log_text("client");
            log.contains("spin: ") && log.contains("instruction limit")
        },
        "the client log to record the stopped component",
    );
    client.run("echo alive");
    client.wait_for_line("alive");
    client.send(b"\x00n");
    client.wait_for("the second tile focused", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && tile.left == 100)
    });
}
