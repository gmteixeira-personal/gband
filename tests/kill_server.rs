mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Child;
use std::time::{Duration, Instant};

use common::*;

fn shell_of(env: &TestEnv, server: &Child) -> i32 {
    let server = server.id() as i32;
    wait_until(|| !children(server).is_empty(), "the server's shell");
    let shells = children(server);
    assert_eq!(shells.len(), 1, "{shells:?}");
    env.track_shell(shells[0]);
    shells[0]
}

fn wait_exit(server: &mut Child) -> std::process::ExitStatus {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        if let Some(status) = server.try_wait().unwrap() {
            return status;
        }
        assert!(Instant::now() < deadline, "the server did not exit");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn kill_server_stops_a_running_server() {
    let env = TestEnv::new("kill-running");
    let mut server = env.start_server("/bin/sh");
    let shell = shell_of(&env, &server);

    let output = env.command(GBAND, &["kill-server"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty(), "{:?}", output.stdout);
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);

    assert_eq!(wait_exit(&mut server).code(), Some(0));
    assert!(!is_running(shell));
    assert!(!env.socket().exists());
    assert!(env.log_text("client").contains("client started"));
    assert!(env.log_text("server").contains("received SIGTERM"));
}

#[test]
fn kill_server_without_a_server_fails_and_creates_nothing() {
    let env = TestEnv::new("kill-none");
    let output = env.command(GBAND, &["kill-server"]).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains(env.socket().to_str().unwrap()), "{stderr}");
    assert!(!env.runtime_home().exists());
    assert!(env.log_text("client").contains("client started"));
}

#[test]
fn program_ignoring_sighup_is_killed() {
    let env = TestEnv::new("kill-trap");
    let pane = env.root.join("pane.sh");
    fs::write(&pane, "#!/bin/sh\ntrap '' HUP\nsleep 100\n").unwrap();
    fs::set_permissions(&pane, fs::Permissions::from_mode(0o755)).unwrap();
    let mut server = env.start_server(pane.to_str().unwrap());
    let shell = shell_of(&env, &server);
    wait_until(|| !children(shell).is_empty(), "the pane's sleep");
    let sleep = children(shell)[0];
    env.track_shell(sleep);

    let started = Instant::now();
    let output = env.command(GBAND, &["kill-server"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(wait_exit(&mut server).code(), Some(0));
    assert!(started.elapsed() < Duration::from_secs(3));
    assert!(!is_running(shell));
    assert!(!is_running(sleep));
}

#[test]
fn next_attach_after_kill_server_starts_a_fresh_shell() {
    let env = TestEnv::new("kill-fresh");
    let mut first = Attached::start(&env, 80, 24);
    first.wait_for_prompt();
    let old_shell = first.shell_pid(&env);

    let output = env.command(GBAND, &["kill-server"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(first.wait_exit(), 0);
    first.wait_for_text("[exited]");

    let mut second = Attached::start(&env, 80, 24);
    second.wait_for_prompt();
    assert_ne!(second.shell_pid(&env), old_shell);
}
