mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
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
    let (contents, cursor) = {
        let before = first.screen();
        (before.contents(), before.cursor())
    };
    first.kill();

    let mut second = Attached::start(&env, 80, 24);
    second.wait_for("the same screen", |screen| {
        screen.contents() == contents && screen.cursor() == cursor
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
    first.send(b"\x00D");
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
fn prefix_key_twice_sends_one_ctrl_space() {
    let env = TestEnv::new("prefix-literal");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("clear; cat -v");
    thread::sleep(Duration::from_millis(300));
    client.send(b"\x00");
    thread::sleep(Duration::from_millis(200));
    client.send(b"\x00");
    thread::sleep(Duration::from_millis(200));
    client.send(b"\r");
    client.wait_for_line("^@");
    assert!(
        client
            .focused_lines()
            .iter()
            .all(|line| !line.contains("^@^@"))
    );
}

#[test]
fn ctrl_a_reaches_the_window_and_unbound_keys_are_discarded() {
    let env = TestEnv::new("prefix-ctrl-a");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("clear");
    client.send(b"abc");
    client.wait_for_text("abc");
    client.send(b"\x01");
    thread::sleep(Duration::from_millis(200));
    client.send(b"echo ");
    client.send(b"\x00x\r");
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
fn attach_inside_a_window_is_refused() {
    let env = TestEnv::new("nested");
    let socket = env.socket();
    let mut client = Attached::start_with(&env, GBAND, &["attach"], 80, 24, |command| {
        command.env("GBAND", &socket);
    });
    assert_eq!(client.wait_exit(), 1);
    client.wait_for_text("inside a gband window");
    assert!(!socket.exists());
}

#[test]
fn named_attach_inside_its_own_window_is_refused() {
    let env = TestEnv::new("nested-named");
    let feature = env.socket_named("feature");
    let args = ["-S", "feature", "attach"];
    let mut client = Attached::start_with(&env, GBAND, &args, 80, 24, |command| {
        command.env("GBAND", &feature);
    });
    assert_eq!(client.wait_exit(), 1);
    client.wait_for_text("inside a gband window");
    assert!(!feature.exists());
}

#[test]
fn named_attach_from_a_window_of_another_server_attaches() {
    let env = TestEnv::new("nested-other");
    let default = env.socket();
    let args = ["-S", "feature", "attach"];
    let mut client = Attached::start_with(&env, GBAND, &args, 80, 24, |command| {
        command.env("GBAND", &default);
    });
    client.wait_for_prompt();
    client.shell_pid(&env);
    assert!(env.socket_named("feature").exists());
    assert!(!default.exists());
}

#[test]
fn named_server_starts_on_demand() {
    let env = TestEnv::new("named");
    let args = ["-S", "feature", "attach"];
    let mut client = Attached::start_with(&env, GBAND, &args, 160, 30, |_| {});
    client.wait_for_prompt();
    thread::sleep(Duration::from_millis(300));
    client.shell_pid(&env);
    client.run("echo $GBAND");
    client.wait_for_line(env.socket_named("feature").to_str().unwrap());
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
fn leader_equals_grows_the_column() {
    let env = TestEnv::new("grow");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00=");
    client.wait_for("a tile 48 columns wide", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 1 && tiles[0].left == 0 && tiles[0].right == 47
    });
    thread::sleep(Duration::from_millis(300));
    client.run("clear; tput cols");
    client.wait_for_line("46");
}

fn install(path: &Path) {
    let staged = path.with_extension("staged");
    fs::copy(GBAND, &staged).unwrap();
    fs::set_permissions(&staged, fs::Permissions::from_mode(0o755)).unwrap();
    fs::rename(&staged, path).unwrap();
}

fn detach(client: &mut Attached) {
    client.send(b"\x00D");
    assert_eq!(client.wait_exit(), 0);
    client.wait_for_text("[detached]");
}

fn note_follows_detached(client: &Attached, note: &str) {
    client.wait_for_text(note);
    let contents = client.contents();
    assert!(
        contents.find("[detached]") < contents.find(note),
        "{contents}"
    );
}

#[test]
fn debug_client_replaces_a_server_after_a_rebuild() {
    let env = TestEnv::new("rebuild");
    let build = env.root.join("gband-build");
    install(&build);

    let mut old = Attached::start_with(&env, &build, &["attach"], 80, 24, |_| {});
    old.wait_for_prompt();
    let old_shell = old.shell_pid(&env);
    let old_server = env.server_pid();
    detach(&mut old);

    install(&build);
    let mut new = Attached::start_with(&env, &build, &["attach"], 80, 24, |_| {});
    new.wait_for_prompt();
    let new_shell = new.shell_pid(&env);
    assert_ne!(new_shell, old_shell);
    assert_ne!(env.server_pid(), old_server);
    wait_until(|| !is_running(old_shell), "the old shell to stop");
    wait_until(|| !is_running(old_server), "the old server to stop");
}

#[test]
fn debug_client_from_another_path_keeps_the_server() {
    let env = TestEnv::new("other-path");
    let mut first = Attached::start(&env, 120, 24);
    first.wait_for_prompt();
    let shell = first.shell_pid(&env);
    let server = env.server_pid();
    detach(&mut first);

    let other = env.root.join("gband-other");
    install(&other);
    let mut second = Attached::start_with(&env, &other, &["attach"], 120, 24, |_| {});
    second.wait_for_prompt();
    assert_eq!(second.shell_pid(&env), shell);
    detach(&mut second);
    note_follows_detached(
        &second,
        "gband: the server runs a different gband build; stop it with gband kill-server and \
         attach again",
    );
    assert_eq!(env.server_pid(), server);
    assert!(is_running(shell));
}

#[test]
fn note_names_the_selected_server() {
    let env = TestEnv::new("note-named");
    let other = env.root.join("gband-other");
    install(&other);
    let args = ["-S", "feature", "attach"];
    let mut first = Attached::start_with(&env, &other, &args, 120, 24, |_| {});
    first.wait_for_prompt();
    first.shell_pid(&env);
    detach(&mut first);

    let mut second = Attached::start_with(&env, GBAND, &args, 120, 24, |_| {});
    second.wait_for_prompt();
    detach(&mut second);
    note_follows_detached(
        &second,
        "stop it with gband -S feature kill-server and attach again",
    );
}

fn window_number(lines: &[String]) -> Option<u32> {
    lines
        .iter()
        .find_map(|line| line.strip_prefix("window="))
        .and_then(|number| number.parse().ok())
}

fn open_second_window(env: &TestEnv) -> Attached {
    let mut client = Attached::start(env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(env);
    client.send(b"\x00\r");
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
    client.send(b"\x00d");
    thread::sleep(Duration::from_millis(300));
    assert!(client.child.try_wait().unwrap().is_none());
    client.run("echo still-attached");
    client.wait_for_line("still-attached");
}

#[test]
fn leader_enter_opens_a_focused_window() {
    let env = TestEnv::new("open");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("echo window=$GBAND_WINDOW");
    client.wait_for_focused("the first window number", |lines| {
        window_number(lines).is_some()
    });
    let first = window_number(&client.focused_lines()).unwrap();

    client.send(b"\x00\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused && tiles[1].left == 40
    });
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("echo window=$GBAND_WINDOW");
    client.wait_for_focused("the second window number", |lines| {
        window_number(lines).is_some()
    });
    assert_ne!(window_number(&client.focused_lines()), Some(first));
}

#[test]
fn leader_h_focuses_the_left_window() {
    let env = TestEnv::new("focus-left");
    let mut client = open_second_window(&env);
    client.send(b"\x00h");
    client.wait_for("the first tile focused", |screen| tiles(screen)[0].focused);
    client.run("echo left");
    client.wait_for_line("left");
    let screen = client.screen();
    let tiles = tiles(&screen);
    assert!(!tiles[1].lines(&screen).iter().any(|line| line == "left"));
}

#[test]
fn leader_q_closes_the_focused_window() {
    let env = TestEnv::new("close");
    let mut client = open_second_window(&env);
    let second = client.last_pid();
    client.send(b"\x00q");
    client.wait_for("one tile left", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 1 && tiles[0].focused && tiles[0].left == 0
    });
    wait_until(|| !is_running(second), "the closed shell to stop");
    client.run("echo after-close");
    client.wait_for_line("after-close");
}

#[test]
fn leader_u_and_i_switch_bands() {
    let env = TestEnv::new("bands");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("echo first-band");
    client.wait_for_line("first-band");

    client.send(b"\x00u");
    client.wait_for("an empty band", |screen| tiles(screen).is_empty());
    client.send(b"\x00\r");
    client.wait_for("a window in the second band", |screen| {
        tiles(screen).len() == 1
    });
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.run("echo second-band");
    client.wait_for_line("second-band");

    client.send(b"\x00i");
    client.wait_for_focused("the first band", |lines| {
        lines.iter().any(|line| line == "first-band")
    });
    client.run("echo back-on-first");
    client.wait_for_line("back-on-first");

    client.send(b"\x00u");
    client.wait_for_focused("the second band", |lines| {
        lines.iter().any(|line| line == "second-band")
    });
}

#[test]
fn kill_server_ends_a_session_of_two_windows() {
    let env = TestEnv::new("kill-two");
    let mut client = open_second_window(&env);
    let status = env.command(GBAND, &["kill-server"]).status().unwrap();
    assert!(status.success());
    assert_eq!(client.wait_exit(), 0);
    client.wait_for_text("[exited]");
    wait_until(|| !env.socket().exists(), "the socket to be removed");
}

#[test]
fn unknown_animations_value_is_logged() {
    let env = TestEnv::new("animations-unknown");
    let mut client = Attached::start_with(&env, GBAND, &["attach"], 80, 24, |command| {
        command.env("GBAND_ANIMATIONS", "fast");
    });
    client.wait_for_prompt();
    client.shell_pid(&env);
    assert!(
        env.log_text("client")
            .contains("GBAND_ANIMATIONS value \"fast\"")
    );
}

#[test]
fn animations_off_opens_a_window_without_motion() {
    let env = TestEnv::new("animations-off");
    let mut client = Attached::start_with(&env, GBAND, &["attach"], 80, 24, |command| {
        command.env("GBAND_ANIMATIONS", "off");
    });
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused && tiles[1].left == 40
    });
    client.wait_for_prompt();
    client.shell_pid(&env);
}
