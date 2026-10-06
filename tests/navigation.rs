mod common;

use std::thread;
use std::time::Duration;

use common::*;

fn attached(env: &TestEnv) -> Attached {
    let mut client = Attached::start(env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(env);
    client
}

fn bar(screen: &Grid, row: usize) -> String {
    let line = screen
        .contents()
        .lines()
        .nth(row)
        .unwrap_or_default()
        .to_owned();
    line.chars()
        .take(20)
        .collect::<String>()
        .trim_end()
        .to_owned()
}

fn navigation(screen: &Grid) -> bool {
    bar(screen, 1) == "navigation"
}

fn interactive(screen: &Grid) -> bool {
    bar(screen, 1) == "C-space navigation"
}

fn labelled(client: &mut Attached, label: &str) {
    client.run(&format!("clear; echo {label}"));
    client.wait_for_line(label);
}

fn focused_shows(screen: &Grid, label: &str) -> bool {
    focused_lines(screen).iter().any(|line| line == label)
}

fn untouched_prompt(screen: &Grid) -> bool {
    focused_lines(screen)
        .iter()
        .rfind(|line| !line.is_empty())
        .is_some_and(|line| line.ends_with('$'))
}

fn position(screen: &Grid, text: &str) -> Option<(u16, u16)> {
    screen
        .contents()
        .lines()
        .enumerate()
        .find_map(|(row, line)| {
            let start = line.find(text)?;
            let col = line[..start].chars().count();
            Some((u16::try_from(row).ok()?, u16::try_from(col).ok()?))
        })
}

fn inverse_at(screen: &Grid, text: &str) -> bool {
    position(screen, text).is_some_and(|(row, col)| {
        screen
            .screen()
            .cell(row, col)
            .is_some_and(|cell| cell.inverse())
    })
}

#[test]
fn repeated_focus_moves() {
    let env = TestEnv::new("navigation-focus");
    let mut client = attached(&env);
    labelled(&mut client, "W1");
    for index in 2..=4 {
        client.send(b"\x00n");
        client.wait_for("a new focused window", |screen| {
            untouched_prompt(screen) && !focused_shows(screen, &format!("W{}", index - 1))
        });
        client.wait_for_prompt();
        labelled(&mut client, &format!("W{index}"));
    }
    client.send(b"\x00hhh\r");
    client.wait_for("the first window focused", |screen| {
        focused_shows(screen, "W1")
    });
    client.send(b"\x00lll");
    client.wait_for("the fourth window focused in navigation mode", |screen| {
        focused_shows(screen, "W4") && navigation(screen)
    });
    assert!(untouched_prompt(&client.screen()));
}

#[test]
fn repeated_resize_then_escape() {
    let env = TestEnv::new("navigation-resize");
    let mut client = attached(&env);
    client.send(b"\x00==");
    client.wait_for("a tile 56 columns wide", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 1 && tiles[0].left == 20 && tiles[0].right == 75
    });
    client.send(b"\x1b");
    client.wait_for("interactive mode", interactive);
    thread::sleep(Duration::from_millis(300));
    client.run("clear; tput cols");
    client.wait_for_line("54");
}

#[test]
fn enter_returns_to_interactive_mode() {
    let env = TestEnv::new("navigation-enter");
    let mut client = attached(&env);
    client.send(b"\x00\r");
    client.wait_for("interactive mode", interactive);
    client.run("echo still-one");
    client.wait_for_line("still-one");
    assert_eq!(client.tiles().len(), 1);
}

#[test]
fn n_opens_a_window_in_interactive_mode() {
    let env = TestEnv::new("navigation-open");
    let mut client = attached(&env);
    client.send(b"\x00n");
    client.wait_for("the second tile focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 1 && tiles[0].focused && tiles[0].left == 40
    });
    client.wait_for_prompt();
    client.run("echo window=$GBAND_WINDOW");
    client.wait_for_focused("the second window number", |lines| {
        lines.iter().any(|line| line.starts_with("window="))
    });
    assert!(interactive(&client.screen()));
}

#[test]
fn question_mark_opens_the_key_list() {
    let env = TestEnv::new("navigation-key-list");
    let mut client = attached(&env);
    client.send(b"\x00?");
    client.wait_for("the key list on its first line", |screen| {
        position(screen, "┌navigation keys").is_some()
            && inverse_at(screen, "focus the column to the left")
    });
    client.send(b"\x1b[B");
    client.wait_for("the cursor line on the second line", |screen| {
        inverse_at(screen, "focus the column to the right")
            && !inverse_at(screen, "focus the column to the left")
    });
}
