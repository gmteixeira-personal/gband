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

fn navigation(screen: &Grid) -> bool {
    sidebar_mode(screen) == "N"
}

fn interactive(screen: &Grid) -> bool {
    sidebar_mode(screen) == "I"
}

fn shows_error(screen: &Grid) -> bool {
    sidebar_error(screen)
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
    client.send(b"\x00n\r");
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
    client.wait_for("the error marker", |screen| {
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

fn titled(env: &TestEnv) -> Attached {
    let mut client = Attached::start_with(env, &env.executable, &["attach"], 80, 24, |command| {
        command.env_remove("GBAND_WINDOW_TITLES")
    });
    client.wait_for_prompt();
    client.shell_pid(env);
    client
}

fn title(screen: &Grid) -> Option<String> {
    let tile = tiles(screen).into_iter().find(|tile| tile.focused)?;
    let row = screen
        .contents()
        .lines()
        .nth(usize::from(tile.top))?
        .to_owned();
    let border: String = row
        .chars()
        .skip(usize::from(tile.left) + 1)
        .take(usize::from(tile.right - tile.left - 1))
        .collect();
    Some(border.trim_end_matches('─').to_owned())
}

fn floating_line(screen: &Grid, title: &str) -> Option<String> {
    let contents = screen.contents();
    let lines: Vec<&str> = contents.lines().collect();
    let top = lines
        .iter()
        .position(|line| line.contains(&format!("┌{title}")))?;
    let row = lines.get(top + 1)?;
    let start = row.find('│')? + '│'.len_utf8();
    let end = row.rfind('│')?;
    Some(row.get(start..end)?.trim_end().to_owned())
}

fn rename_line(screen: &Grid) -> Option<String> {
    floating_line(screen, "rename")
}

fn rename_to(client: &mut Attached, name: &str) {
    client.send(b"\x00N");
    client.wait_for("the rename prompt", |screen| rename_line(screen).is_some());
    client.send(b"\x15");
    client.wait_for("an empty rename prompt", |screen| {
        rename_line(screen).as_deref() == Some("")
    });
    client.send(name.as_bytes());
    client.wait_for("the typed name", |screen| {
        rename_line(screen).as_deref() == Some(name)
    });
    client.send(b"\r");
}

#[test]
fn rename_the_focused_window() {
    let env = TestEnv::new("prompt-rename");
    let mut client = titled(&env);
    client.wait_for("the title sh", |screen| {
        title(screen).as_deref() == Some("sh")
    });
    rename_to(&mut client, "logs");
    client.wait_for("the title logs", |screen| {
        rename_line(screen).is_none() && title(screen).as_deref() == Some("logs")
    });
    client.send(b"\x00N");
    client.wait_for("the prompt holding the manual name", |screen| {
        rename_line(screen).as_deref() == Some("logs")
    });
    client.send(b"x\x1b");
    client.wait_for("the prompt closed with the name kept", |screen| {
        rename_line(screen).is_none() && title(screen).as_deref() == Some("logs")
    });
    client.send(b"\x00N");
    client.wait_for("the rename prompt again", |screen| {
        rename_line(screen).as_deref() == Some("logs")
    });
    client.send(b"\x15\r");
    client.wait_for("the automatic name back", |screen| {
        rename_line(screen).is_none() && title(screen).as_deref() == Some("sh")
    });
}

#[test]
fn lua_prompt_replaces_the_rename_prompt() {
    let env = TestEnv::new("prompt-rename-replaced");
    let mut client = attached(&env);
    client.send(b"\x00N");
    client.wait_for("the rename prompt", |screen| rename_line(screen).is_some());
    client.send(b"ab\x00:");
    client.wait_for("only the Lua prompt", |screen| {
        rename_line(screen).is_none() && floating_line(screen, "lua").as_deref() == Some(":")
    });
    client.send(b"x");
    client.wait_for("the Lua prompt focused", |screen| prompt_shows(screen, "x"));
}
