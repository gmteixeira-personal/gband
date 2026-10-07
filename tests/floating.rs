mod common;

use common::*;
use gband_lua::DEFAULTS;

const PREFIX: &[u8] = b"\x00";

fn env(name: &str) -> TestEnv {
    let env = TestEnv::new(name);
    env.write_config(&DEFAULTS.replace("gband.plugin(\"gband.sidebar\")", ""));
    env
}

fn attached(env: &TestEnv) -> Attached {
    let mut client = Attached::start(env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(env);
    client
}

fn press(client: &mut Attached, key: &[u8]) {
    client.send(&[PREFIX, key].concat());
}

fn leave(client: &mut Attached) {
    client.send(b"\r");
}

fn has_line(tile: &Tile, screen: &Grid, text: &str) -> bool {
    tile.lines(screen).iter().any(|line| line == text)
}

fn spans(tile: &Tile) -> (u16, u16, u16, u16) {
    (tile.left, tile.right, tile.top, tile.bottom)
}

fn second_window(client: &mut Attached) {
    press(client, b"n\r");
    client.wait_for("two tiles", |screen| tiles(screen).len() == 2);
    client.wait_for_prompt();
}

#[test]
fn float_the_focused_window() {
    let env = env("floating-float");
    let mut client = attached(&env);
    press(&mut client, b"v");
    client.wait_for("the window in its box", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && spans(tile) == (20, 59, 2, 21))
    });
    leave(&mut client);
    client.run("echo floated");
    client.wait_for_line("floated");
}

#[test]
fn switch_to_the_tiled_layer_and_back() {
    let env = env("floating-layers");
    let mut client = attached(&env);
    second_window(&mut client);
    press(&mut client, b"v");
    for (left, right) in [(20, 59), (28, 67), (36, 75), (40, 79)] {
        if left > 20 {
            client.send(b"\x0c");
        }
        client.wait_for("the box one step right", |screen| {
            tiles(screen)
                .iter()
                .any(|tile| tile.focused && spans(tile) == (left, right, 2, 21))
        });
    }
    client.send(b"V");
    client.wait_for("the tiled window focused", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && tile.left == 0)
    });
    leave(&mut client);
    client.run("echo tiled");
    client.wait_for("tiled in the first window only", |screen| {
        let found = tiles(screen);
        found
            .iter()
            .any(|tile| tile.left == 0 && has_line(tile, screen, "tiled"))
            && !found
                .iter()
                .any(|tile| tile.left == 40 && has_line(tile, screen, "tiled"))
    });
    press(&mut client, b"V");
    client.wait_for("the floating window focused", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && tile.left == 40)
    });
}

#[test]
fn move_a_column_with_ctrl_h_and_ctrl_right() {
    let env = env("floating-move-column");
    let mut client = attached(&env);
    client.run("echo first-window");
    client.wait_for_line("first-window");
    second_window(&mut client);
    client.run("echo second-window");
    client.wait_for_line("second-window");
    press(&mut client, b"\x08");
    client.wait_for("B then A", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.left == 0 && tile.focused && has_line(tile, screen, "second-window"))
    });
    client.send(b"\x1b[1;5C");
    client.wait_for("A then B", |screen| {
        let found = tiles(screen);
        found
            .iter()
            .any(|tile| tile.left == 0 && has_line(tile, screen, "first-window"))
            && found.iter().any(|tile| {
                tile.left == 40 && tile.focused && has_line(tile, screen, "second-window")
            })
    });
    leave(&mut client);
    client.run("echo still-clean");
    client.wait_for_line("still-clean");
    assert!(
        !client
            .focused_lines()
            .iter()
            .any(|line| line.contains("[1;5C") || line.contains("^H"))
    );
}

#[test]
fn move_a_window_with_ctrl_j() {
    let env = env("floating-move-window");
    let mut client = attached(&env);
    client.run("echo upper");
    client.wait_for_line("upper");
    second_window(&mut client);
    client.run("echo lower");
    client.wait_for_line("lower");
    press(&mut client, b"[");
    client.wait_for("one stacked column", |screen| {
        let found = tiles(screen);
        found.len() == 2 && found.iter().all(|tile| tile.left == 0)
    });
    client.send(b"k");
    client.wait_for("the upper window focused", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && has_line(tile, screen, "upper"))
    });
    client.send(b"\x0a");
    client.wait_for("the windows swapped", |screen| {
        let mut found = tiles(screen);
        found.sort_by_key(|tile| tile.top);
        found.len() == 2
            && has_line(&found[0], screen, "lower")
            && found[1].focused
            && has_line(&found[1], screen, "upper")
    });
}

#[test]
fn move_a_floating_window() {
    let env = env("floating-move-box");
    let mut client = attached(&env);
    press(&mut client, b"v");
    client.wait_for("the box at column 20", |screen| {
        tiles(screen).iter().any(|tile| tile.left == 20)
    });
    client.send(b"\x0c");
    client.wait_for("the box at column 28", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && spans(tile) == (28, 67, 2, 21))
    });
}
