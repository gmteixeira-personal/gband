mod common;

use common::*;
use gband_lua::DEFAULTS;

const BINDINGS: &str = r#"
gband.bind('alt+p', function()
  gband.win.open({ kind = 'tiled', lines = { 'hello from a plugin' }, column_width = 1/2 })
end)
gband.bind('alt+f', function()
  local win
  win = gband.win.open({
    width = 30, height = 5, title = 'Float',
    lines = { 'waiting' },
    keys = { x = function() gband.win.set_lines(win, { 'typed x' }) end },
  })
end)
gband.bind('alt+t', function() gband.pane.send_text(1, 'echo sent-$((40 + 2))\n') end)
"#;

fn env(name: &str) -> TestEnv {
    let env = TestEnv::new(name);
    env.write_config(&format!("{DEFAULTS}\n{BINDINGS}"));
    env
}

fn attached(env: &TestEnv) -> Attached {
    let mut client = Attached::start(env, 100, 24);
    client.wait_for_prompt();
    client.shell_pid(env);
    client
}

fn has_tile_line(screen: &Grid, text: &str) -> bool {
    tiles(screen)
        .iter()
        .any(|tile| tile.lines(screen).iter().any(|line| line == text))
}

#[test]
fn tiled_plugin_window_is_seen_by_two_clients() {
    let env = env("windows-two-clients");
    let mut first = attached(&env);
    let second = attached(&env);
    first.send(b"\x1bp");
    for client in [&first, &second] {
        client.wait_for("the plugin pane's tile", |screen| {
            tiles(screen).len() == 2 && has_tile_line(screen, "hello from a plugin")
        });
    }
    first.wait_for_focused("the plugin pane focused", |lines| {
        lines
            .first()
            .is_some_and(|line| line == "hello from a plugin")
    });
}

#[test]
fn tiled_plugin_window_closes_with_the_prefix_and_q() {
    let env = env("windows-close");
    let mut client = attached(&env);
    client.send(b"\x1bp");
    client.wait_for_focused("the plugin pane focused", |lines| {
        lines
            .first()
            .is_some_and(|line| line == "hello from a plugin")
    });
    client.send(b"\x00q");
    client.wait_for("one tile again", |screen| tiles(screen).len() == 1);
    client.run("echo still-here");
    client.wait_for_line("still-here");
}

#[test]
fn focused_float_takes_the_keys() {
    let env = env("windows-float");
    let mut client = attached(&env);
    client.send(b"\x1bf");
    client.wait_for_text("waiting");
    client.send(b"x");
    client.wait_for_text("typed x");
    client.send(b"\x1b");
    client.wait_for("the float closed", |screen| {
        !screen.contents().contains("typed x")
    });
    client.run("echo after-float");
    client.wait_for_line("after-float");
    assert!(
        !client
            .focused_lines()
            .iter()
            .any(|line| line.contains("xecho"))
    );
}

#[test]
fn send_text_runs_a_command_in_another_pane() {
    let env = env("windows-send-text");
    let mut client = attached(&env);
    client.send(b"\x00\r");
    client.wait_for("two tiles", |screen| tiles(screen).len() == 2);
    client.wait_for_prompt();
    client.send(b"\x1bt");
    client.wait_for("the first pane printed", |screen| {
        has_tile_line(screen, "sent-42")
    });
    assert!(!client.focused_lines().iter().any(|line| line == "sent-42"));
}

#[test]
fn plugin_pane_leaves_when_its_owner_detaches() {
    let env = env("windows-owner");
    let mut first = attached(&env);
    let second = attached(&env);
    first.send(b"\x1bp");
    second.wait_for("the plugin pane's tile", |screen| {
        tiles(screen).len() == 2 && has_tile_line(screen, "hello from a plugin")
    });
    first.send(b"\x00D");
    first.wait_exit();
    second.wait_for("the plugin pane gone", |screen| {
        tiles(screen).len() == 1 && !has_tile_line(screen, "hello from a plugin")
    });
}
