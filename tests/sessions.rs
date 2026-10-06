mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Output, Stdio};
use std::thread;
use std::time::Duration;

use common::*;
use rustix::process::{Pid, Signal};

fn attach_session(env: &TestEnv, args: &[&str], cwd: &std::path::Path) -> Attached {
    let cwd = cwd.to_path_buf();
    Attached::start_with(env, GBAND, args, 80, 24, move |command| command.cwd(cwd))
}

fn list(env: &TestEnv) -> String {
    let output = env.command(GBAND, &["list-sessions"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn kill_session(env: &TestEnv, name: &str) -> Output {
    env.command(GBAND, &["kill-session", "-s", name])
        .output()
        .unwrap()
}

#[test]
fn named_sessions_share_one_server() {
    let env = TestEnv::new("sessions");
    let mut work = attach_session(&env, &["attach", "-s", "work"], &env.work);
    work.wait_for_prompt();
    work.shell_pid(&env);
    assert_eq!(list(&env), "work\t1\t1\n");

    let elsewhere = env.root.join("elsewhere");
    fs::create_dir_all(&elsewhere).unwrap();
    let mut play = attach_session(&env, &["attach", "-s", "play"], &elsewhere);
    play.wait_for_prompt();
    play.shell_pid(&env);
    play.run(&format!(
        "[ \"$(pwd)\" = '{}' ] && echo same-$((6 * 7))",
        elsewhere.display()
    ));
    play.wait_for_line("same-42");
    play.run("echo session=$GBAND_SESSION");
    play.wait_for_line("session=play");
    assert_eq!(list(&env), "play\t1\t1\nwork\t1\t1\n");

    work.run("sleep 100");
    thread::sleep(Duration::from_millis(300));
    work.send(b"\x00D");
    assert_eq!(work.wait_exit(), 0);
    work.wait_for_text("[detached]");
    assert_eq!(list(&env), "play\t1\t1\nwork\t1\t0\n");

    let again = attach_session(&env, &["-s", "work", "attach"], &env.work);
    again.wait_for_focused("sleep 100 still running", |lines| {
        let last = lines.iter().rfind(|line| !line.is_empty());
        last.is_some_and(|line| line.ends_with("$ sleep 100"))
    });

    let output = kill_session(&env, "play");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(play.wait_exit(), 0);
    play.wait_for_text("[exited]");
    assert_eq!(list(&env), "work\t1\t1\n");
}

#[test]
fn server_starts_its_session_at_the_given_size() {
    let env = TestEnv::new("sessions-size");
    let size = env.root.join("size");
    let window = env.root.join("window.sh");
    fs::write(
        &window,
        format!("#!/bin/sh\nstty size > {}\nsleep 100\n", size.display()),
    )
    .unwrap();
    fs::set_permissions(&window, fs::Permissions::from_mode(0o755)).unwrap();
    let mut server = env
        .command(GBAND, &["server", "--size", "100x30"])
        .env("SHELL", &window)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    env.shell_of(&server);
    wait_until(
        || fs::read_to_string(&size).is_ok_and(|text| text.ends_with('\n')),
        "the window's size",
    );
    assert_eq!(fs::read_to_string(&size).unwrap(), "28 48\n");
    let output = env.command(GBAND, &["kill-server"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(wait_process_exit(&mut server).code(), Some(0));
}

#[test]
fn killing_an_unknown_session_names_it() {
    let env = TestEnv::new("sessions-nope");
    let mut server = env.start_server("/bin/sh");
    let output = kill_session(&env, "nope");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("nope"), "{stderr}");
    assert_eq!(list(&env), "default\t1\t0\n");

    let output = kill_session(&env, "default");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(wait_process_exit(&mut server).code(), Some(0));
    assert!(!env.socket().exists());
}

#[test]
fn sigterm_ends_every_session() {
    let env = TestEnv::new("sessions-term");
    let mut default = Attached::start(&env, 80, 24);
    default.wait_for_prompt();
    let first = default.shell_pid(&env);
    let mut work = attach_session(&env, &["attach", "-s", "work"], &env.work);
    work.wait_for_prompt();
    let second = work.shell_pid(&env);

    let server = env.server_pid();
    rustix::process::kill_process(Pid::from_raw(server).unwrap(), Signal::TERM).unwrap();
    assert_eq!(default.wait_exit(), 0);
    default.wait_for_text("[exited]");
    assert_eq!(work.wait_exit(), 0);
    work.wait_for_text("[exited]");
    wait_until(|| !is_running(server), "the server to exit");
    assert!(!is_running(first));
    assert!(!is_running(second));
    assert!(!env.socket().exists());
}
