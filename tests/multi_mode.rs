mod common;

use std::fs;
use std::path::PathBuf;
use std::process::Stdio;

use common::*;
use gband_lua::DEFAULTS;

struct Received(PathBuf);

impl Received {
    fn new(env: &TestEnv) -> Self {
        let dir = env.data_home().join("received");
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn command(&self, word: &str) -> String {
        format!(
            "echo {word}; echo {word} >> {}/$GBAND_WINDOW",
            self.0.display()
        )
    }

    fn of(&self, window: u32) -> String {
        fs::read_to_string(self.0.join(window.to_string())).unwrap_or_default()
    }

    fn wait(&self, windows: &[u32], word: &str) {
        wait_until(
            || {
                windows
                    .iter()
                    .all(|&window| self.of(window).lines().any(|line| line == word))
            },
            &format!("windows {windows:?} receiving {word}"),
        );
    }
}

fn eval(env: &TestEnv, chunk: &str) -> String {
    let output = env
        .command(GBAND, &["eval", chunk])
        .env_remove("GBAND_SESSION")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn focused_window(env: &TestEnv) -> u32 {
    eval(env, "return gband.view().window").parse().unwrap()
}

fn started(env: &TestEnv) -> (Attached, u32) {
    let mut client = Attached::start(env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(env);
    let first = focused_window(env);
    (client, first)
}

fn open_column(env: &TestEnv, client: &mut Attached) -> u32 {
    let before = focused_window(env);
    client.send(b"\x00n\r");
    wait_until(|| focused_window(env) != before, "the new window focused");
    client.wait_for_prompt();
    focused_window(env)
}

fn open_floating(env: &TestEnv, client: &Attached) -> u32 {
    let before = focused_window(env);
    eval(env, "gband.action.open_window({ floating = true })");
    wait_until(|| focused_window(env) != before, "the new window focused");
    client.wait_for_prompt();
    focused_window(env)
}

fn two_columns(name: &str) -> (TestEnv, Attached, Received, [u32; 2]) {
    let env = TestEnv::new(name);
    let (mut client, first) = started(&env);
    let second = open_column(&env, &mut client);
    let received = Received::new(&env);
    (env, client, received, [first, second])
}

fn blank(screen: &Grid) -> bool {
    let tiles = tiles(screen);
    tiles.len() == 2
        && tiles
            .iter()
            .all(|tile| tile.lines(screen).iter().all(|line| line.is_empty()))
}

fn multi_on(client: &mut Attached) {
    client.send(b"\x00m");
    client.wait_for("the multi mode letter", |screen| {
        sidebar_mode(screen) == "M"
    });
}

#[test]
fn typing_reaches_every_window() {
    let (env, mut client, received, [first, second]) = two_columns("multi-typing");
    let floating = open_floating(&env, &client);
    multi_on(&mut client);
    client.run(&received.command("hi"));
    received.wait(&[first, second, floating], "hi");
}

#[test]
fn other_bands_receive_nothing() {
    let (env, mut client, received, [first, second]) = two_columns("multi-bands");
    client.send(b"\x00un\r");
    client.wait_for_prompt();
    let third = focused_window(&env);
    client.send(b"\x00i\r");
    wait_until(|| focused_window(&env) == second, "band 1 viewed");
    multi_on(&mut client);
    client.run(&received.command("hi"));
    received.wait(&[first, second], "hi");
    client.send(b"\x00u\r");
    wait_until(|| focused_window(&env) == third, "band 2 viewed");
    client.run(&received.command("done"));
    received.wait(&[third], "done");
    assert_eq!(received.of(third), "done\n");
}

#[test]
fn paste_reaches_every_window() {
    let (_env, mut client, received, windows) = two_columns("multi-paste");
    multi_on(&mut client);
    let mut paste = b"\x1b[200~".to_vec();
    paste.extend(received.command("pasted").as_bytes());
    paste.extend(b"\x1b[201~");
    client.send(&paste);
    client.send(b"\r");
    received.wait(&windows, "pasted");
}

#[test]
fn minimized_window_receives_the_input() {
    let env = TestEnv::new("multi-minimized");
    let (mut client, first) = started(&env);
    eval(&env, "gband.action.toggle_window_floating()");
    let second = open_floating(&env, &client);
    let received = Received::new(&env);
    eval(&env, &format!("gband.window.minimize({first})"));
    multi_on(&mut client);
    client.run(&received.command("hi"));
    received.wait(&[first, second], "hi");
    eval(&env, &format!("gband.window.focus({first})"));
    wait_until(
        || focused_window(&env) == first,
        "the restored window focused",
    );
    client.wait_for_line("hi");
}

#[test]
fn band_change_moves_the_input() {
    let (env, mut client, received, [first, second]) = two_columns("multi-band-change");
    client.send(b"\x00un\r");
    client.wait_for_prompt();
    let third = focused_window(&env);
    let fourth = open_column(&env, &mut client);
    client.send(b"\x00i\r");
    wait_until(|| focused_window(&env) == second, "band 1 viewed");
    multi_on(&mut client);
    client.send(b"\x00u");
    wait_until(|| focused_window(&env) == fourth, "band 2 viewed");
    client.send(b"\x1b");
    client.wait_for("multi mode again", |screen| sidebar_mode(screen) == "M");
    client.run(&received.command("hi"));
    received.wait(&[third, fourth], "hi");
    client.send(b"\x00i\r");
    wait_until(|| focused_window(&env) == second, "band 1 viewed again");
    client.run(&received.command("done"));
    received.wait(&[first, second], "done");
    assert_eq!(received.of(first), "done\n");
    assert_eq!(received.of(second), "done\n");
}

#[test]
fn plugin_window_takes_the_keys() {
    let (_env, mut client, received, windows) = two_columns("multi-plugin-window");
    multi_on(&mut client);
    client.send(b"\x00:");
    client.wait_for_text("┌lua");
    client.send(b"1");
    client.wait_for_text(":1");
    client.send(b"\x1b");
    client.wait_for("the prompt closed", |screen| {
        !screen.contents().contains("┌lua")
    });
    client.run(&received.command("done"));
    received.wait(&windows, "done");
    for window in windows {
        assert_eq!(received.of(window), "done\n");
    }
}

#[test]
fn prefix_key_to_every_window() {
    let (_env, mut client, _received, _windows) = two_columns("multi-prefix");
    multi_on(&mut client);
    client.run("clear; cat -v");
    client.wait_for("cat in both windows", blank);
    client.send(b"\x00\x00\r");
    client.wait_for("^@ in both windows", |screen| {
        tiles_with_line(screen, "^@") == 2
    });
}

#[test]
fn mouse_reaches_one_window() {
    let (_env, mut client, _received, _windows) = two_columns("multi-mouse");
    multi_on(&mut client);
    client.run(r"clear; printf '\033[?1000h\033[?1006h'; cat -v");
    client.wait_for("cat in both windows", blank);
    let mut shown = client.tiles();
    shown.sort_by_key(|tile| tile.left);
    let second = shown[1];
    let (col, row) = (second.left + 5, second.top + 3);
    client.send(format!("\x1b[<0;{};{}M", col + 1, row + 1).as_bytes());
    client.send(format!("\x1b[<0;{};{}m", col + 1, row + 1).as_bytes());
    client.send(b"x\r");
    client.wait_for("x in both windows", |screen| {
        tiles(screen)
            .iter()
            .filter(|tile| tile.lines(screen).iter().any(|line| line.ends_with('x')))
            .count()
            == 2
    });
    let screen = client.screen();
    let mut shown = tiles(&screen);
    shown.sort_by_key(|tile| tile.left);
    let pressed = |tile: &Tile| {
        tile.lines(&screen)
            .iter()
            .any(|line| line.contains("^[[<0;"))
    };
    assert!(!pressed(&shown[0]), "{}", screen.contents());
    assert!(pressed(&shown[1]), "{}", screen.contents());
}

#[test]
fn another_client_is_unaffected() {
    let (env, mut first_client, received, [first, second]) = two_columns("multi-other-client");
    let mut other = Attached::start(&env, 80, 24);
    other.wait_for_prompt();
    multi_on(&mut first_client);
    other.run(&received.command("hi"));
    wait_until(
        || received.of(first) == "hi\n" || received.of(second) == "hi\n",
        "the other client's window receiving hi",
    );
    let (focused, unfocused) = if received.of(first) == "hi\n" {
        (first, second)
    } else {
        (second, first)
    };
    first_client.run(&received.command("done"));
    received.wait(&[first, second], "done");
    assert_eq!(received.of(focused), "hi\ndone\n");
    assert_eq!(received.of(unfocused), "done\n");
}

#[test]
fn bound_by_a_user() {
    let env = TestEnv::new("multi-bound");
    env.write_config(&format!(
        "{DEFAULTS}\ngband.bind('alt+m', gband.action.toggle_multi)\n"
    ));
    let (mut client, first) = started(&env);
    let second = open_column(&env, &mut client);
    let received = Received::new(&env);
    client.send(b"\x1bm");
    client.wait_for("the multi mode letter", |screen| {
        sidebar_mode(screen) == "M"
    });
    client.run(&received.command("hi"));
    received.wait(&[first, second], "hi");
}

#[test]
fn multi_mode_by_name() {
    let env = TestEnv::new("multi-by-name");
    env.write_config(&format!(
        "{DEFAULTS}\ngband.keymap.set('root', 'f5', gband.action.toggle_multi)\n"
    ));
    let (mut client, first) = started(&env);
    let second = open_column(&env, &mut client);
    let received = Received::new(&env);
    client.send(b"\x1b[15~");
    client.wait_for("the multi mode letter", |screen| {
        sidebar_mode(screen) == "M"
    });
    client.run(&received.command("hi"));
    received.wait(&[first, second], "hi");
    let log = env.log_text("server");
    assert!(!log.contains("ERROR"), "{log}");
}
