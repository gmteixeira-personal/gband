mod common;

use std::fs;
use std::thread;
use std::time::Duration;

use common::*;
use gband_lua::DEFAULTS;

const PREFIX: &[u8] = b"\x00";
const LEFT: u16 = 0;
const MOTION: u16 = 32;

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

fn click(client: &mut Attached, cell: (u16, u16)) {
    sgr(client, LEFT, cell, 'M');
    sgr(client, LEFT, cell, 'm');
}

fn drag(client: &mut Attached, from: (u16, u16), to: (u16, u16)) {
    sgr(client, LEFT, from, 'M');
    sgr(client, LEFT + MOTION, to, 'M');
    sgr(client, LEFT, to, 'm');
}

fn mark(client: &mut Attached, mark: &str) {
    client.run(&format!("export M={mark}; echo {mark}"));
    client.wait_for_line(mark);
}

fn open_window(client: &mut Attached, name: &str) {
    client.send(&[PREFIX, b"n\r"].concat());
    client.wait_for("a new focused window", |screen| {
        let lines = focused_lines(screen);
        let written: Vec<&String> = lines.iter().filter(|line| !line.is_empty()).collect();
        written.len() == 1 && written[0].ends_with('$')
    });
    mark(client, name);
}

fn consume_left(client: &mut Attached, stacked: usize) {
    client.send(&[PREFIX, b"[\r"].concat());
    client.wait_for(&format!("a column of {stacked} windows"), |screen| {
        tiles(screen).iter().filter(|tile| tile.left == 0).count() == stacked
    });
}

fn shows(screen: &Grid, left: u16, mark: &str) -> bool {
    let marked = |line: &String| line.split_whitespace().next() == Some(mark);
    tiles(screen)
        .iter()
        .any(|tile| tile.left == left && tile.lines(screen).iter().any(marked))
}

fn one_tab(screen: &Grid, line: &str) -> bool {
    let shown = tiles(screen);
    shown.len() == 1
        && (shown[0].top, shown[0].bottom, shown[0].left, shown[0].right) == (0, 23, 0, 39)
        && shows(screen, 0, line)
}

fn tabbed(env: &TestEnv, count: usize) -> Attached {
    let mut client = attached(env);
    mark(&mut client, "T1");
    for index in 2..=count {
        open_window(&mut client, &format!("T{index}"));
        consume_left(&mut client, index);
    }
    client.send(&[PREFIX, b"w\r"].concat());
    let last = format!("T{count}");
    client.wait_for("the column tabbed", |screen| one_tab(screen, &last));
    client
}

fn focus_tab(client: &mut Attached, keys: &[u8], line: &str) {
    client.send(&[PREFIX, keys, b"\r"].concat());
    client.wait_for(&format!("{line} shown"), |screen| one_tab(screen, line));
}

fn left_edge(screen: &Grid, left: u16) -> String {
    let contents = screen.contents();
    contents
        .lines()
        .map(|line| line.chars().nth(usize::from(left)).unwrap_or(' '))
        .collect()
}

fn size_of(client: &mut Attached, mark: &str) {
    client.run("clear; echo $M $(tput cols)x$(tput lines)");
    client.wait_for_line(&format!("{mark} 38x22"));
}

#[test]
fn turn_tabs_on() {
    let env = env("tabbed-on");
    let mut client = tabbed(&env, 3);
    size_of(&mut client, "T3");
    focus_tab(&mut client, b"k", "T2");
    size_of(&mut client, "T2");
    focus_tab(&mut client, b"k", "T1");
    size_of(&mut client, "T1");
}

#[test]
fn switching_tabs_resizes_nothing() {
    let env = env("tabbed-switch");
    let mut client = tabbed(&env, 2);
    let winches = env.work.join("winches");
    let trap = format!("trap 'echo $M >> {}' WINCH", winches.display());
    size_of(&mut client, "T2");
    client.run(&trap);
    focus_tab(&mut client, b"k", "T1");
    size_of(&mut client, "T1");
    client.run(&trap);
    focus_tab(&mut client, b"j", "T2");
    size_of(&mut client, "T2");
    focus_tab(&mut client, b"k", "T1");
    size_of(&mut client, "T1");
    thread::sleep(Duration::from_millis(300));
    assert_eq!(fs::read_to_string(&winches).unwrap_or_default(), "");
}

#[test]
fn two_clients_two_tabs() {
    let env = env("tabbed-two-clients");
    let mut first = tabbed(&env, 2);
    focus_tab(&mut first, b"k", "T1");
    let mut second = Attached::start(&env, 80, 24);
    second.wait_for("the first tab", |screen| one_tab(screen, "T1"));
    focus_tab(&mut second, b"j", "T2");
    first.wait_for("the first tab still", |screen| one_tab(screen, "T1"));
    second.wait_for("the second tab still", |screen| one_tab(screen, "T2"));
}

fn mouse_reporting(client: &mut Attached, mark: &str) {
    client.run(&format!(
        "clear; printf '\\033[?1000h\\033[?1006h'; echo {mark}"
    ));
    client.wait_for_line(mark);
}

#[test]
fn click_a_tab() {
    let env = env("tabbed-click");
    let mut client = tabbed(&env, 3);
    focus_tab(&mut client, b"k", "T2");
    mouse_reporting(&mut client, "R2");
    focus_tab(&mut client, b"k", "T1");
    mouse_reporting(&mut client, "R1");
    click(&mut client, (0, 12));
    client.wait_for("the second tab focused", |screen| one_tab(screen, "R2"));
    client.run("echo typed");
    client.wait_for_line("typed");
    assert!(!client.contents().contains("[<"), "{}", client.contents());
    focus_tab(&mut client, b"k", "R1");
    assert!(!client.contents().contains("[<"), "{}", client.contents());
}

#[test]
fn click_between_segments() {
    let env = env("tabbed-click-gap");
    let mut client = tabbed(&env, 3);
    focus_tab(&mut client, b"kk", "T1");
    click(&mut client, (0, 8));
    client.run("echo still");
    client.wait_for_line("still");
    assert!(one_tab(&client.screen(), "T1"));
}

#[test]
fn drag_a_tab_out_of_its_column() {
    let env = env("tabbed-drag-out");
    let mut client = tabbed(&env, 3);
    client.send(&[PREFIX, b"kk"].concat());
    client.wait_for("the first tab shown", |screen| one_tab(screen, "T1"));
    drag(&mut client, (0, 20), (70, 10));
    client.wait_for("the third tab in its own column", |screen| {
        tiles(screen).len() == 2 && shows(screen, 0, "T1") && shows(screen, 40, "T3")
    });
    let edge = left_edge(&client.screen(), 0);
    assert_eq!(edge.chars().nth(1), Some('█'), "{edge}");
    assert_eq!(edge.chars().nth(20), Some('░'), "{edge}");
    assert_eq!(
        edge.chars().filter(|&cell| cell == ' ').count(),
        1,
        "{edge}"
    );
}

#[test]
fn drop_into_a_tabbed_column() {
    let env = env("tabbed-drop-into");
    let mut client = attached(&env);
    mark(&mut client, "T1");
    open_window(&mut client, "T2");
    open_window(&mut client, "T3");
    client.send(&[PREFIX, b"[w"].concat());
    client.wait_for("tabbed column B", |screen| {
        let shown = tiles(screen);
        shown.len() == 2 && shows(screen, 0, "T1") && shows(screen, 40, "T3")
    });
    client.send(b"h");
    client.wait_for("the first column focused", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && tile.left == 0)
    });
    drag(&mut client, (10, 10), (60, 18));
    client.wait_for("one tabbed column", |screen| one_tab(screen, "T1"));
    let edge = left_edge(&client.screen(), 0);
    let expected = format!("╭{} {} {}╰", "░".repeat(7), "░".repeat(7), "█".repeat(6));
    assert_eq!(edge, expected);
}
