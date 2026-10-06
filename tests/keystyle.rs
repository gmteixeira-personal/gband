mod common;

use common::*;

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
