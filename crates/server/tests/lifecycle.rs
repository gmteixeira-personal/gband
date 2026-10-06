use std::fs;
use std::os::unix::net::UnixListener;
use std::time::Duration;

use gband_test_support::*;
use std::os::unix::fs::PermissionsExt;

use gband_protocol::{ClientMessage, ServerMessage, lock_path, socket_path};

#[tokio::test(flavor = "multi_thread")]
async fn second_server_is_refused_and_first_keeps_running() {
    let server = TestServer::start("second", &["/bin/sh"]).await;
    let error = gband_server::run(config(&server.runtime_dir, &["/bin/sh"]))
        .await
        .unwrap_err();
    let message = format!("{error:#}");
    assert!(
        message.contains(server.socket().to_str().unwrap()),
        "{message}"
    );
    let mut client = server.attach(80, 24).await;
    client.type_line("echo still-alive").await;
    client.wait_for_text("still-alive\n").await;
    client.type_line("exit").await;
    server.finished().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn stale_socket_is_replaced() {
    let runtime_dir = runtime_dir("stale");
    let socket = socket_path(&runtime_dir);
    drop(UnixListener::bind(&socket).unwrap());
    while std::os::unix::net::UnixStream::connect(&socket).is_ok() {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(socket.exists());
    let server = TestServer::start_in(runtime_dir, &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    client.type_line("exit").await;
    client.drain_to_end().await;
    assert!(client.exited);
    server.finished().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn unspawnable_program_fails_before_the_socket_exists() {
    let runtime_dir = runtime_dir("bad-prog");
    let error = gband_server::run(config(&runtime_dir, &["/nonexistent/gband-program"]))
        .await
        .unwrap_err();
    assert!(format!("{error:#}").contains("/nonexistent/gband-program"));
    assert!(!socket_path(&runtime_dir).exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn lock_file_holds_the_server_pid() {
    let server = TestServer::start("pid", &["/bin/sh"]).await;
    let record = fs::read_to_string(lock_path(&server.socket())).unwrap();
    assert_eq!(record, format!("{}\n", std::process::id()));
    let mut client = server.attach(80, 24).await;
    client.type_line("exit").await;
    server.finished().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn explicit_socket_records_its_pid_beside_it() {
    let runtime_dir = runtime_dir("explicit");
    let config = gband_server::ServerConfig {
        socket: runtime_dir.join("s"),
        ..config(&runtime_dir, &["/bin/sh"])
    };
    let server = TestServer::start_with(runtime_dir, config).await;
    let record = fs::read_to_string(server.runtime_dir.join("s.lock")).unwrap();
    assert_eq!(record, format!("{}\n", std::process::id()));
    let mut client = server.attach(80, 24).await;
    client.type_line("exit").await;
    server.finished().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn socket_is_private() {
    let server = TestServer::start("private", &["/bin/sh"]).await;
    let mode = fs::metadata(server.socket()).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    let mut client = server.attach(80, 24).await;
    client.type_line("exit").await;
    server.finished().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn output_without_a_client_is_drained() {
    let runtime_dir = runtime_dir("drain");
    let marker = runtime_dir.join("finished");
    let script = format!(
        "yes 0123456789abcdef | head -c 1048576; echo; echo last-line; touch {}; exec sleep 100",
        marker.display()
    );
    let server = TestServer::start_in(runtime_dir, &["/bin/sh", "-c", &script]).await;
    wait_for_file(&marker).await;
    let mut client = server.attach(80, 24).await;
    client.wait_for_text("last-line").await;
    let contents = client.screen().contents();
    assert!(
        contents.lines().any(|line| line == "last-line"),
        "{contents}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn info_follows_accepted_with_configured_values() {
    let server = TestServer::start("info", &["/bin/sh"]).await;
    let client = server.attach(80, 24).await;
    assert_eq!(
        client.info,
        ServerMessage::Info {
            pid: std::process::id(),
            executable: IDENTITY,
        }
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn both_clients_receive_exited_and_the_socket_is_removed() {
    let server = TestServer::start("exit", &["/bin/sh"]).await;
    let mut first = server.attach(80, 24).await;
    let mut second = server.attach(80, 24).await;
    first.type_line("echo bye; exit").await;
    first.drain_to_end().await;
    second.drain_to_end().await;
    assert!(first.exited);
    assert!(second.exited);
    let socket = server.socket();
    server.finished().await.unwrap();
    assert!(!socket.exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn program_exiting_with_no_client_ends_the_server() {
    let runtime_dir = runtime_dir("true");
    gband_server::run(config(&runtime_dir, &["/bin/true"]))
        .await
        .unwrap();
    assert!(!socket_path(&runtime_dir).exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn detaching_leaves_the_program_running() {
    let server = TestServer::start("detach", &["/bin/sh"]).await;
    let mut leaving = server.attach(80, 24).await;
    let mut staying = server.attach(80, 24).await;
    leaving.type_line("echo $$ > pid").await;
    leaving.send(&ClientMessage::Detach).await;
    assert!(leaving.peer.closes().await);
    tokio::time::sleep(Duration::from_millis(200)).await;
    staying.type_line("echo still-here").await;
    staying.wait_for_text("still-here\n").await;
    assert!(!server.handle.is_finished());
    let mut late = server.attach(80, 24).await;
    late.type_line("exit").await;
    server.finished().await.unwrap();
}

#[test]
fn kill_without_a_server_reports_it_and_creates_nothing() {
    let parent = runtime_dir("kill-none");
    let missing = parent.join("gband");
    let error = gband_server::kill(&socket_path(&missing)).unwrap_err();
    let message = format!("{error:#}");
    assert!(
        message.contains(socket_path(&missing).to_str().unwrap()),
        "{message}"
    );
    assert!(!missing.exists());

    let socket = socket_path(&parent);
    fs::write(lock_path(&socket), "12345\n").unwrap();
    let message = format!("{:#}", gband_server::kill(&socket).unwrap_err());
    assert!(message.contains("no server is running"), "{message}");
}
