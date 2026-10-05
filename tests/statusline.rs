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

fn reload(env: &TestEnv, client: &Attached, extra: &str) {
    let seen = reloads(env);
    env.write_config(&format!("{DEFAULTS}\n{extra}\n"));
    wait_until(|| reloads(env) > seen, "a configuration reload");
    client.wait_for_prompt();
}

fn tput_lines(client: &mut Attached, expected: &str) {
    thread::sleep(Duration::from_millis(300));
    client.run("clear; tput lines");
    client.wait_for_line(expected);
}

#[test]
fn height_excludes_the_status_line() {
    let env = TestEnv::new("statusline-height");
    let mut client = attached(&env);
    client.wait_for("the status line on the bottom row", |screen| {
        let bottom = row(screen, 23);
        bottom.starts_with("band 1") && bottom.trim_end().ends_with("1/1")
    });
    let tiles = client.tiles();
    assert_eq!(tiles.len(), 1);
    assert_eq!((tiles[0].top, tiles[0].bottom), (0, 22));
    tput_lines(&mut client, "21");
}

#[test]
fn turning_the_status_line_off() {
    let env = TestEnv::new("statusline-off");
    let mut client = attached(&env);
    client.wait_for_text("band 1");
    reload(&env, &client, "gband.opt.statusline_position = 'off'");
    client.wait_for("a tile of 40×24", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 1 && tiles[0].bottom == 23 && tiles[0].right == 39
    });
    assert!(!client.contents().contains("band 1"));
    tput_lines(&mut client, "22");
}

#[test]
fn moving_the_status_line() {
    let env = TestEnv::new("statusline-top");
    let mut client = attached(&env);
    client.wait_for_text("band 1");
    reload(&env, &client, "gband.opt.statusline_position = 'top'");
    client.wait_for("the status line on the top row", |screen| {
        let tiles = tiles(screen);
        row(screen, 0).starts_with("band 1") && tiles.len() == 1 && tiles[0].top == 1
    });
    tput_lines(&mut client, "21");
}

#[test]
fn mode_segment_while_the_prefix_is_held() {
    let env = TestEnv::new("statusline-mode");
    let mut client = attached(&env);
    client.wait_for_text("band 1");
    client.send(b"\x00");
    client.wait_for("the prefix table in the status line", |screen| {
        row(screen, 23).starts_with("band 1 │ prefix")
    });
    client.send(b"x");
    client.wait_for("the prefix table gone", |screen| {
        let bottom = row(screen, 23);
        bottom.starts_with("band 1 │ C-space prefix") && !bottom.contains("│ prefix")
    });
}

#[test]
fn hints_in_the_default_line() {
    let env = TestEnv::new("statusline-hints");
    let client = attached(&env);
    client.wait_for("the hints on the bottom row", |screen| {
        let bottom = row(screen, 23);
        bottom.starts_with("band 1 │ C-space prefix ")
            && bottom.ends_with("1/1")
            && bottom.chars().count() == 80
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
    client.wait_for("the error item", |screen| {
        let bottom = row(screen, 23);
        bottom.starts_with("spin: ") && bottom.contains("instruction limit")
    });
    client.run("echo alive");
    client.wait_for_line("alive");
    client.send(b"\x00\r");
    client.wait_for("two tiles", |screen| tiles(screen).len() == 2);
}
