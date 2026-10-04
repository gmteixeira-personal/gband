mod common;

use common::*;

fn events<'a>(log: &'a str, start: &str) -> Vec<&'a str> {
    log.lines().filter(|line| !line.contains(start)).collect()
}

#[test]
fn named_servers_run_side_by_side() {
    let env = TestEnv::new("side-by-side");
    let mut a = env.start_named_server("a", "/bin/sh");
    let a_shell = env.shell_of(&a);
    let mut b = env.start_named_server("b", "/bin/sh");
    let b_shell = env.shell_of(&b);

    let b_socket = env.socket_named("b");
    let second = env
        .command(GBAND, &["server", "-p", b_socket.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(second.status.code(), Some(1));
    let stderr = String::from_utf8(second.stderr).unwrap();
    assert!(stderr.contains("already running"), "{stderr}");
    assert!(stderr.contains(b_socket.to_str().unwrap()), "{stderr}");

    let output = env
        .command(GBAND, &["-S", "a", "kill-server"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(wait_process_exit(&mut a).code(), Some(0));
    assert!(!is_running(a_shell));
    assert!(is_running(b_shell));
    assert!(b.try_wait().unwrap().is_none());
    assert!(b_socket.exists());
}

#[test]
fn log_events_name_their_server() {
    let env = TestEnv::new("log-sockets");
    let mut a = env.start_named_server("a", "/bin/sh");
    env.shell_of(&a);
    let mut b = env.start_named_server("b", "/bin/sh");
    env.shell_of(&b);
    let output = env
        .command(GBAND, &["-S", "a", "kill-server"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    wait_process_exit(&mut a);
    let client = env.log_text("client");
    let output = env
        .command(
            GBAND,
            &["-p", env.socket_named("b").to_str().unwrap(), "kill-server"],
        )
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    wait_process_exit(&mut b);

    let server = env.log_text("server");
    let server_events = events(&server, "server started");
    assert!(server_events.iter().any(|line| line.contains("/a.sock")));
    assert!(server_events.iter().any(|line| line.contains("/b.sock")));
    assert!(
        server_events
            .iter()
            .all(|line| line.contains("/a.sock") || line.contains("/b.sock")),
        "{server}"
    );

    let client_events = events(&client, "client started");
    assert!(!client_events.is_empty(), "{client}");
    assert!(
        client_events.iter().all(|line| line.contains("/a.sock")),
        "{client}"
    );
}
