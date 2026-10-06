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

fn row(screen: &Grid, index: usize) -> String {
    screen
        .contents()
        .lines()
        .nth(index)
        .unwrap_or_default()
        .to_owned()
}

fn bar(screen: &Grid, index: usize) -> String {
    row(screen, index)
        .chars()
        .take(20)
        .collect::<String>()
        .trim_end()
        .to_owned()
}

fn mode(screen: &Grid, label: &str) -> bool {
    (0..3).any(|index| bar(screen, index) == label)
}

fn navigation(screen: &Grid) -> bool {
    mode(screen, "navigation")
}

fn interactive(screen: &Grid) -> bool {
    mode(screen, "C-space navigation")
}

fn shows_error(screen: &Grid) -> bool {
    row(screen, 0).starts_with("error ")
}

fn prompt_shows(screen: &Grid, text: &str) -> bool {
    screen
        .contents()
        .lines()
        .any(|line| line.contains(&format!(":{text}")))
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

fn run_line(client: &mut Attached, line: &str) {
    client.send(b"\x00:");
    client.send(line.as_bytes());
    client.wait_for("the prompt holding the line", |screen| {
        prompt_shows(screen, line)
    });
    client.send(b"\r");
}

#[test]
fn run_an_action() {
    let env = TestEnv::new("prompt-action");
    let mut client = attached(&env);
    labelled(&mut client, "W1");
    client.send(b"\x00n");
    client.wait_for("the second window", |screen| {
        untouched_prompt(screen) && !focused_shows(screen, "W1")
    });
    labelled(&mut client, "W2");
    run_line(&mut client, "gband.action.focus_column_left()");
    client.wait_for(
        "the first window focused with the prompt closed",
        |screen| {
            focused_shows(screen, "W1") && interactive(screen) && !prompt_shows(screen, "gband")
        },
    );
    assert!(untouched_prompt(&client.screen()), "{}", client.contents());
    client.send(b"\x00l\r");
    client.wait_for("the second window focused", |screen| {
        focused_shows(screen, "W2") && interactive(screen)
    });
    assert!(untouched_prompt(&client.screen()), "{}", client.contents());
}

#[test]
fn enter_a_mode() {
    let env = TestEnv::new("prompt-mode");
    let mut client = attached(&env);
    run_line(&mut client, "gband.keymap.enter(\"prefix\")");
    client.wait_for("navigation mode", |screen| {
        navigation(screen) && !prompt_shows(screen, "gband")
    });
}

#[test]
fn endless_loop() {
    let env = TestEnv::new("prompt-endless");
    let mut client = attached(&env);
    run_line(&mut client, "while true do end");
    client.wait_for("the error item", |screen| {
        shows_error(screen) && !prompt_shows(screen, "while")
    });
    assert!(
        env.log_text("client")
            .contains("prompt:1: instruction limit exceeded"),
        "{}",
        env.log_text("client")
    );
    run_line(&mut client, "gband.keymap.enter(\"prefix\")");
    client.wait_for("navigation mode after the stopped line", navigation);
}

#[test]
fn colon_in_interactive_mode() {
    let env = TestEnv::new("prompt-colon");
    let mut client = attached(&env);
    client.run("echo a:b");
    client.wait_for_line("a:b");
    thread::sleep(Duration::from_millis(300));
    let screen = client.screen();
    assert!(interactive(&screen), "{}", client.contents());
    assert!(!screen.contents().contains("lua"), "{}", client.contents());
}
