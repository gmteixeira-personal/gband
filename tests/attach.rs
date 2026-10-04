mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::thread;
use std::time::Duration;

use common::*;
use rustix::process::{Pid, Signal};

#[test]
fn first_attach_starts_a_server_in_the_client_directory() {
    let env = TestEnv::new("first");
    let mut client = Attached::start(&env, 100, 30);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("pwd");
    client.wait_for_line(env.work.to_str().unwrap());
    assert!(env.log_text("client").contains("client started"));
    assert!(env.log_text("server").contains("server started"));
}

#[test]
fn killed_client_reattaches_to_the_same_screen_and_shell() {
    let env = TestEnv::new("reattach");
    let mut first = Attached::start(&env, 80, 24);
    first.wait_for_prompt();
    let pid = first.shell_pid(&env);
    first.run("ls /");
    first.wait_for_text("usr");
    first.run("printf 'left here'");
    first.wait_for_text("left here");
    thread::sleep(Duration::from_millis(300));
    let before = first.screen();
    first.kill();

    let mut second = Attached::start(&env, 80, 24);
    second.wait_for("the same screen", |screen| {
        screen.contents() == before.contents()
            && screen.cursor_position() == before.cursor_position()
    });
    second.run("clear; echo pid=$$");
    second.wait_for("the shell pid", |screen| screen.contents().contains("pid="));
    assert_eq!(second.last_pid(), pid);
}

#[test]
fn detach_leaves_the_session_running() {
    let env = TestEnv::new("detach");
    let mut first = Attached::start(&env, 80, 24);
    first.wait_for_prompt();
    first.shell_pid(&env);
    first.run("sleep 100");
    thread::sleep(Duration::from_millis(300));
    first.send(b"\x01D");
    assert_eq!(first.wait_exit(), 0);
    first.wait_for_text("[detached]");
    assert!(!first.screen().alternate_screen());

    let second = Attached::start(&env, 80, 24);
    second.wait_for_focused("sleep 100 still running", |lines| {
        let last = lines.iter().rfind(|line| !line.is_empty());
        last.is_some_and(|line| line.ends_with("$ sleep 100"))
    });
}

#[test]
fn hung_up_client_leaves_the_session_running() {
    let env = TestEnv::new("hangup");
    let mut first = Attached::start(&env, 80, 24);
    first.wait_for_prompt();
    let pid = first.shell_pid(&env);
    let server = env.server_pid();
    let client = first.child.process_id().unwrap() as i32;
    assert_ne!(session_of(server), session_of(client));

    rustix::process::kill_process(Pid::from_raw(client).unwrap(), Signal::HUP).unwrap();
    first.wait_exit();
    assert!(is_running(server));
    assert!(is_running(pid));

    let mut second = Attached::start(&env, 80, 24);
    second.wait_for_prompt();
    second.run("clear; echo pid=$$");
    second.wait_for("the shell pid", |screen| screen.contents().contains("pid="));
    assert_eq!(second.last_pid(), pid);
}

#[test]
fn prefix_key_passes_ctrl_a_and_discards_unbound_keys() {
    let env = TestEnv::new("prefix");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("clear");
    client.send(b"abc");
    client.wait_for_text("abc");
    client.send(b"\x01");
    thread::sleep(Duration::from_millis(200));
    client.send(b"\x01");
    thread::sleep(Duration::from_millis(200));
    client.send(b"echo ");
    client.send(b"\x01x\r");
    client.wait_for_line("abc");
}

#[test]
fn exit_prints_exited() {
    let env = TestEnv::new("exited");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("exit");
    assert_eq!(client.wait_exit(), 0);
    client.wait_for_text("[exited]");
    assert!(!client.screen().alternate_screen());
    wait_until(|| !env.socket().exists(), "the socket to be removed");
}

#[test]
fn killed_server_prints_lost_server() {
    let env = TestEnv::new("lost");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    let server = env.server_pid();
    rustix::process::kill_process(Pid::from_raw(server).unwrap(), Signal::KILL).unwrap();
    assert_eq!(client.wait_exit(), 1);
    client.wait_for_text("[lost server]");
    assert!(!client.screen().alternate_screen());
}

#[test]
fn attach_inside_a_pane_is_refused() {
    let env = TestEnv::new("nested");
    let mut client = Attached::start_with(&env, GBAND, 80, 24, |command| {
        command.env("GBAND", "/somewhere/default.sock");
    });
    assert_eq!(client.wait_exit(), 1);
    client.wait_for_text("inside a gband pane");
    assert!(!env.socket().exists());
}

#[test]
fn terminal_resize_reaches_the_program() {
    let env = TestEnv::new("resize");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.resize(70, 24);
    thread::sleep(Duration::from_millis(300));
    client.run("clear; tput cols");
    client.wait_for_line("33");
}

#[test]
fn debug_client_replaces_a_server_from_another_build() {
    let env = TestEnv::new("rebuild");
    let copy = env.root.join("gband-copy");
    fs::copy(GBAND, &copy).unwrap();
    fs::set_permissions(&copy, fs::Permissions::from_mode(0o755)).unwrap();

    let mut old = Attached::start_with(&env, &copy, 80, 24, |_| {});
    old.wait_for_prompt();
    let old_shell = old.shell_pid(&env);
    let old_server = env.server_pid();
    old.send(b"\x01D");
    assert_eq!(old.wait_exit(), 0);

    let mut new = Attached::start(&env, 80, 24);
    new.wait_for_prompt();
    let new_shell = new.shell_pid(&env);
    assert_ne!(new_shell, old_shell);
    assert_ne!(env.server_pid(), old_server);
    wait_until(|| !is_running(old_shell), "the old shell to stop");
    wait_until(|| !is_running(old_server), "the old server to stop");
}

fn pane_number(lines: &[String]) -> Option<u32> {
    lines
        .iter()
        .find_map(|line| line.strip_prefix("pane="))
        .and_then(|number| number.parse().ok())
}

fn open_second_pane(env: &TestEnv) -> Attached {
    let mut client = Attached::start(env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(env);
    client.send(b"\x01\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused && tiles[1].left == 40
    });
    client.wait_for_prompt();
    client.shell_pid(env);
    client
}

#[test]
fn lowercase_d_does_not_detach() {
    let env = TestEnv::new("lower-d");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x01d");
    thread::sleep(Duration::from_millis(300));
    assert!(client.child.try_wait().unwrap().is_none());
    client.run("echo still-attached");
    client.wait_for_line("still-attached");
}

#[test]
fn leader_enter_opens_a_focused_pane() {
    let env = TestEnv::new("open");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("echo pane=$GBAND_PANE");
    client.wait_for_focused("the first pane number", |lines| {
        pane_number(lines).is_some()
    });
    let first = pane_number(&client.focused_lines()).unwrap();

    client.send(b"\x01\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused && tiles[1].left == 40
    });
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("echo pane=$GBAND_PANE");
    client.wait_for_focused("the second pane number", |lines| {
        pane_number(lines).is_some()
    });
    assert_ne!(pane_number(&client.focused_lines()), Some(first));
}

#[test]
fn leader_h_focuses_the_left_pane() {
    let env = TestEnv::new("focus-left");
    let mut client = open_second_pane(&env);
    client.send(b"\x01h");
    client.wait_for("the first tile focused", |screen| tiles(screen)[0].focused);
    client.run("echo left");
    client.wait_for_line("left");
    let screen = client.screen();
    let tiles = tiles(&screen);
    assert!(!tiles[1].lines(&screen).iter().any(|line| line == "left"));
}

#[test]
fn leader_q_closes_the_focused_pane() {
    let env = TestEnv::new("close");
    let mut client = open_second_pane(&env);
    let second = client.last_pid();
    client.send(b"\x01q");
    client.wait_for("one tile left", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 1 && tiles[0].focused && tiles[0].left == 0
    });
    wait_until(|| !is_running(second), "the closed shell to stop");
    client.run("echo after-close");
    client.wait_for_line("after-close");
}

#[test]
fn leader_u_and_i_switch_workspaces() {
    let env = TestEnv::new("workspaces");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("echo first-workspace");
    client.wait_for_line("first-workspace");

    client.send(b"\x01u");
    client.wait_for("an empty workspace", |screen| tiles(screen).is_empty());
    client.send(b"\x01\r");
    client.wait_for("a pane in the second workspace", |screen| {
        tiles(screen).len() == 1
    });
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("echo second-workspace");
    client.wait_for_line("second-workspace");

    client.send(b"\x01i");
    client.wait_for_focused("the first workspace", |lines| {
        lines.iter().any(|line| line == "first-workspace")
    });
    client.run("echo back-on-first");
    client.wait_for_line("back-on-first");

    client.send(b"\x01u");
    client.wait_for_focused("the second workspace", |lines| {
        lines.iter().any(|line| line == "second-workspace")
    });
}

#[test]
fn kill_server_ends_a_session_of_two_panes() {
    let env = TestEnv::new("kill-two");
    let mut client = open_second_pane(&env);
    let status = env.command(GBAND, &["kill-server"]).status().unwrap();
    assert!(status.success());
    assert_eq!(client.wait_exit(), 0);
    client.wait_for_text("[exited]");
    wait_until(|| !env.socket().exists(), "the socket to be removed");
}
