mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use common::*;
use gband_lua::DEFAULTS;

const PREFIX: &[u8] = b"\x00";
const SGR_1000: &str = "\\033[?1000h\\033[?1006h";
const SGR_1002: &str = "\\033[?1002h\\033[?1006h";

const LEFT: u16 = 0;
const MIDDLE: u16 = 1;
const RIGHT: u16 = 2;
const MOTION: u16 = 32;
const ALT: u16 = 8;
const WHEEL_UP: u16 = 64;
const WHEEL_DOWN: u16 = 65;

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

fn sgr(client: &mut Attached, code: u16, (col, row): (u16, u16), end: char) {
    client.send(format!("\x1b[<{code};{};{}{end}", col + 1, row + 1).as_bytes());
}

fn click(client: &mut Attached, code: u16, cell: (u16, u16)) {
    sgr(client, code, cell, 'M');
    sgr(client, code, cell, 'm');
}

fn drag(client: &mut Attached, code: u16, from: (u16, u16), to: (u16, u16)) {
    sgr(client, code, from, 'M');
    sgr(client, code + MOTION, to, 'M');
    sgr(client, code, to, 'm');
}

fn content(tile: &Tile, col: u16, row: u16) -> (u16, u16) {
    (tile.left + 1 + col, tile.top + 1 + row)
}

fn tile_at(client: &Attached, left: u16) -> Tile {
    client
        .tiles()
        .into_iter()
        .find(|tile| tile.left == left)
        .unwrap_or_else(|| panic!("no tile at {left}:\n{}", client.contents()))
}

fn focused(client: &Attached) -> Tile {
    client
        .tiles()
        .into_iter()
        .find(|tile| tile.focused)
        .unwrap_or_else(|| panic!("no focused tile:\n{}", client.contents()))
}

fn open_window(client: &mut Attached) {
    client.send(&[PREFIX, b"n"].concat());
    client.wait_for("a new focused window", |screen| {
        let lines = focused_lines(screen);
        let written: Vec<&String> = lines.iter().filter(|line| !line.is_empty()).collect();
        written.len() == 1 && written[0].ends_with('$')
    });
}

fn recording(client: &mut Attached, env: &TestEnv, setup: &str, bytes: usize) -> PathBuf {
    let file = env.work.join("got");
    let _ = fs::remove_file(&file);
    client.run(&format!(
        "{setup}echo ready; stty raw -echo; head -c {bytes} > got; stty sane; echo done"
    ));
    client.wait_for_line("ready");
    file
}

fn wait_file(path: &Path, expected: &[u8]) {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let got = fs::read(path).unwrap_or_default();
        if got == expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "{} holds {:?}, expected {:?}",
            path.display(),
            String::from_utf8_lossy(&got),
            String::from_utf8_lossy(expected)
        );
        thread::sleep(Duration::from_millis(20));
    }
}

fn tile_has(client: &Attached, tile: &Tile, text: &str) -> bool {
    tile.lines(&client.screen())
        .iter()
        .any(|line| line.contains(text))
}

fn row_of(client: &Attached, tile: &Tile, text: &str) -> u16 {
    let lines = tile.lines(&client.screen());
    lines
        .iter()
        .position(|line| line == text)
        .unwrap_or_else(|| panic!("no line {text:?} in {lines:?}")) as u16
}

#[test]
fn click_to_focus() {
    let env = env("mouse-click-focus");
    let mut client = attached(&env);
    open_window(&mut client);
    assert_eq!(focused(&client).left, 40);
    let first = tile_at(&client, 0);
    click(&mut client, LEFT, content(&first, 3, 3));
    client.wait_for("the first tile focused", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && tile.left == 0)
    });
    client.run("echo left");
    client.wait_for_line("left");
    assert!(!tile_has(&client, &tile_at(&client, 40), "left"));
}

#[test]
fn click_a_floating_window() {
    let env = env("mouse-click-float");
    let mut client = attached(&env);
    open_window(&mut client);
    client.send(&[PREFIX, b"v"].concat());
    client.wait_for("the floating box", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && (tile.left, tile.top) == (20, 2))
    });
    client.send(b"V\r");
    client.wait_for("the tiled layer", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && tile.left == 0)
    });
    click(&mut client, LEFT, (30, 10));
    client.wait_for("the floating box focused on top", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && (tile.left, tile.top) == (20, 2))
    });
    client.run("echo floated");
    client.wait_for_line("floated");
}

#[test]
fn forwarded_click() {
    let env = env("mouse-forward");
    let mut client = attached(&env);
    let file = recording(&mut client, &env, &format!("printf '{SGR_1000}'; "), 18);
    let tile = focused(&client);
    click(&mut client, LEFT, content(&tile, 4, 2));
    wait_file(&file, b"\x1b[<0;5;3M\x1b[<0;5;3m");
}

#[test]
fn drag_outside_the_window() {
    let env = env("mouse-forward-outside");
    let mut client = attached(&env);
    open_window(&mut client);
    let file = recording(&mut client, &env, &format!("printf '{SGR_1002}'; "), 19);
    let tile = focused(&client);
    assert_eq!(tile.left, 40);
    sgr(&mut client, LEFT, content(&tile, 2, 3), 'M');
    sgr(
        &mut client,
        LEFT + MOTION,
        (tile.left - 1, tile.top + 1 + 3),
        'M',
    );
    wait_file(&file, b"\x1b[<0;3;4M\x1b[<32;1;4M");
}

#[test]
fn alt_selects_in_a_mouse_program() {
    let env = env("mouse-alt-select");
    let mut client = attached(&env);
    let file = recording(
        &mut client,
        &env,
        &format!("printf '{SGR_1000}'; echo abcdef; "),
        1,
    );
    let program = focused(&client);
    let row = row_of(&client, &program, "abcdef");
    open_window(&mut client);
    let program = tile_at(&client, 0);
    let shell = tile_at(&client, 40);
    drag(
        &mut client,
        LEFT + ALT,
        content(&program, 1, row),
        content(&program, 3, row),
    );
    click(&mut client, RIGHT, content(&shell, 2, 2));
    client.wait_for("the selection pasted", |screen| {
        tiles(screen)
            .iter()
            .find(|tile| tile.left == 40)
            .is_some_and(|tile| {
                tile.lines(screen)
                    .iter()
                    .any(|line| line.ends_with("$ bcd"))
            })
    });
    assert_eq!(fs::read(&file).unwrap_or_default(), b"");
}

#[test]
fn wheel_over_an_unfocused_mouse_program() {
    let env = env("mouse-wheel-unfocused");
    let mut client = attached(&env);
    let file = recording(&mut client, &env, &format!("printf '{SGR_1000}'; "), 10);
    open_window(&mut client);
    let program = tile_at(&client, 0);
    sgr(&mut client, WHEEL_DOWN, content(&program, 0, 0), 'M');
    wait_file(&file, b"\x1b[<65;1;1M");
    assert_eq!(focused(&client).left, 40);
}

#[test]
fn wheel_in_navigation_mode_reaches_the_program() {
    let env = env("mouse-wheel-navigation");
    let mut client = attached(&env);
    let file = recording(&mut client, &env, &format!("printf '{SGR_1000}'; "), 10);
    let program = focused(&client);
    client.send(PREFIX);
    sgr(&mut client, WHEEL_UP, content(&program, 2, 1), 'M');
    wait_file(&file, b"\x1b[<64;3;2M");
    client.send(b"n");
    client.wait_for("navigation mode still active", |screen| {
        tiles(screen).len() == 2
    });
}

#[test]
fn wheel_with_alt() {
    let env = env("mouse-wheel-alt");
    let mut client = attached(&env);
    let file = recording(&mut client, &env, &format!("printf '{SGR_1000}'; "), 10);
    let program = focused(&client);
    sgr(&mut client, WHEEL_DOWN + ALT, content(&program, 0, 0), 'M');
    wait_file(&file, b"\x1b[<73;1;1M");
}

#[test]
fn wheel_over_a_program_without_mouse_reporting() {
    let env = env("mouse-wheel-none");
    let mut client = attached(&env);
    let file = recording(&mut client, &env, "", 1);
    let shell = focused(&client);
    sgr(&mut client, WHEEL_UP, content(&shell, 0, 0), 'M');
    client.send(b"x");
    wait_file(&file, b"x");
}

fn copy_ls(client: &mut Attached) {
    client.run("echo ls");
    client.wait_for_line("ls");
    let shell = focused(client);
    let row = row_of(client, &shell, "ls");
    drag(
        client,
        LEFT,
        content(&shell, 0, row),
        content(&shell, 1, row),
    );
}

#[test]
fn right_click_pastes() {
    let env = env("mouse-right-paste");
    let mut client = attached(&env);
    copy_ls(&mut client);
    open_window(&mut client);
    let file = recording(&mut client, &env, "", 2);
    let first = tile_at(&client, 0);
    click(&mut client, LEFT, content(&first, 0, 0));
    client.wait_for("the first tile focused", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && tile.left == 0)
    });
    let reader = tile_at(&client, 40);
    click(&mut client, RIGHT, content(&reader, 5, 5));
    wait_file(&file, b"ls");
    assert_eq!(focused(&client).left, 40);
}

#[test]
fn buffer_survives_a_reload() {
    let env = env("mouse-buffer-reload");
    let mut client = attached(&env);
    copy_ls(&mut client);
    env.write_config(&DEFAULTS.replace("gband.plugin(\"gband.sidebar\")", "\n"));
    wait_until(
        || env.log_text("client").contains("configuration reloaded"),
        "the reload",
    );
    let file = recording(&mut client, &env, "", 2);
    let shell = focused(&client);
    click(&mut client, RIGHT, content(&shell, 5, 5));
    wait_file(&file, b"ls");
}

#[test]
fn drag_a_floating_window_in_navigation_mode() {
    let env = env("mouse-move-float");
    let mut client = attached(&env);
    open_window(&mut client);
    client.send(&[PREFIX, b"v"].concat());
    client.wait_for("the floating box", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && (tile.left, tile.top) == (20, 2))
    });
    drag(&mut client, LEFT, (30, 5), (36, 7));
    client.wait_for("the box moved", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && (tile.left, tile.right, tile.top) == (26, 65, 4))
    });
}

fn marked_columns(client: &mut Attached, count: usize) {
    let marks = ["AAA", "BBB", "CCC"];
    client.run(&format!("echo {}", marks[0]));
    client.wait_for_line(marks[0]);
    for mark in &marks[1..count] {
        open_window(client);
        client.run(&format!("echo {mark}"));
        client.wait_for_line(mark);
    }
}

#[test]
fn drop_between_two_columns() {
    let env = env("mouse-drop-between");
    let mut client = attached(&env);
    marked_columns(&mut client, 3);
    client.send(&[PREFIX, b"hh"].concat());
    client.wait_for("the first column drawn at the left", |screen| {
        tiles(screen).iter().any(|tile| {
            tile.focused && tile.left == 0 && tile.lines(screen).iter().any(|line| line == "AAA")
        })
    });
    drag(&mut client, LEFT, (10, 10), (75, 10));
    client.wait_for("B then A", |screen| {
        let shown = tiles(screen);
        let has = |left: u16, text: &str| {
            shown
                .iter()
                .any(|tile| tile.left == left && tile.lines(screen).iter().any(|line| line == text))
        };
        has(0, "BBB") && has(40, "AAA")
    });
}

#[test]
fn drop_into_a_column() {
    let env = env("mouse-drop-into");
    let mut client = attached(&env);
    marked_columns(&mut client, 2);
    client.send(&[PREFIX, b"h"].concat());
    client.wait_for("the first column focused", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && tile.left == 0)
    });
    drag(&mut client, LEFT, (10, 10), (60, 18));
    client.wait_for("A below B in one column", |screen| {
        let shown = tiles(screen);
        shown.len() == 2
            && shown[0].left == shown[1].left
            && shown
                .iter()
                .max_by_key(|tile| tile.top)
                .is_some_and(|tile| tile.lines(screen).iter().any(|line| line == "AAA"))
    });
}

#[test]
fn release_without_moving_only_focuses() {
    let env = env("mouse-release-still");
    let mut client = attached(&env);
    marked_columns(&mut client, 2);
    client.send(PREFIX);
    click(&mut client, LEFT, (10, 10));
    client.wait_for("the first column focused", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && tile.left == 0)
    });
    let spans: Vec<(u16, u16)> = client
        .tiles()
        .iter()
        .map(|tile| (tile.left, tile.top))
        .collect();
    assert_eq!(spans, [(0, 0), (40, 0)]);
    assert!(tile_has(&client, &tile_at(&client, 0), "AAA"));
}

fn focused_has(screen: &Grid, text: &str) -> bool {
    tiles(screen)
        .iter()
        .any(|tile| tile.focused && tile.lines(screen).iter().any(|line| line == text))
}

#[test]
fn middle_drag_up_switches_bands_in_navigation_mode() {
    let env = env("mouse-drag-bands");
    let mut client = attached(&env);
    client.run("echo AAA");
    client.wait_for_line("AAA");
    client.send(&[PREFIX, b"un"].concat());
    client.wait_for("a window in the second band", |screen| {
        let shown = tiles(screen);
        shown.len() == 1 && !focused_has(screen, "AAA")
    });
    client.run("echo BBB");
    client.wait_for_line("BBB");
    client.send(&[PREFIX, b"i"].concat());
    client.wait_for("the first band viewed", |screen| focused_has(screen, "AAA"));
    drag(&mut client, MIDDLE, (40, 23), (40, 0));
    client.wait_for("the second band viewed", |screen| {
        focused_has(screen, "BBB")
    });
    client.send(b"i");
    client.wait_for("navigation mode still active", |screen| {
        focused_has(screen, "AAA")
    });
}
