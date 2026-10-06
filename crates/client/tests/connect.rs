use std::path::Path;
use std::time::{Duration, Instant};

use gband_client::{ClientConfig, UnixTransport, connect, kill_session, list_sessions};
use gband_core::geometry::Size;
use gband_core::layout::WindowId;
use gband_protocol::{ClientMessage, ExecutableId, ServerMessage, SessionName, socket_path};
use gband_server::ServerConfig;
use gband_test_support::{TestServer, config, runtime_dir};

const SERVER_IDENTITY: ExecutableId = ExecutableId {
    device: 1,
    inode: 1,
};
const CLIENT_IDENTITY: ExecutableId = ExecutableId {
    device: 2,
    inode: 2,
};

fn server_config(runtime_dir: &Path) -> ServerConfig {
    ServerConfig {
        executable: SERVER_IDENTITY,
        ..config(runtime_dir, &["/bin/sh"])
    }
}

fn client_config(session: SessionName, replace: bool) -> ClientConfig {
    ClientConfig {
        session,
        identity: CLIENT_IDENTITY,
        replace_mismatched: replace,
        kill_command: "gband kill-server".to_owned(),
    }
}

fn transport(runtime_dir: &Path, executable: &str, start_server: bool) -> UnixTransport {
    UnixTransport {
        socket: socket_path(runtime_dir),
        executable_path: executable.into(),
        session: SessionName::default(),
        start_server,
        log_dir: runtime_dir.join("log"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn server_that_exits_at_once_fails_fast() {
    let runtime_dir = runtime_dir("client-false");
    let started = Instant::now();
    let error = connect(
        &client_config(SessionName::default(), true),
        &transport(&runtime_dir, "/bin/false", true),
        Size::new(80, 24),
    )
    .await
    .err()
    .expect("connect succeeded without a server");
    assert!(started.elapsed() < Duration::from_secs(2));
    let message = format!("{error:#}");
    assert!(
        message.contains(socket_path(&runtime_dir).to_str().unwrap()),
        "{message}"
    );
    assert!(message.contains("server log"), "{message}");
}

#[tokio::test(flavor = "multi_thread")]
async fn mismatched_server_is_kept_when_replacing_is_off() {
    let runtime_dir = runtime_dir("client-stale");
    let config = server_config(&runtime_dir);
    let server = TestServer::start_with(runtime_dir, config).await;
    let runtime_dir = server.runtime_dir.to_path_buf();

    let mut connection = connect(
        &client_config(SessionName::default(), false),
        &transport(&runtime_dir, "/bin/false", true),
        Size::new(80, 24),
    )
    .await
    .unwrap();
    assert!(connection.stale_server);
    assert_eq!(connection.pid, std::process::id());
    assert_eq!(connection.executable, SERVER_IDENTITY);

    connection
        .send(&ClientMessage::Attach {
            session: SessionName::default(),
            cwd: runtime_dir.clone(),
        })
        .await
        .unwrap();
    assert!(matches!(
        connection.receive().await.unwrap(),
        ServerMessage::Layout { .. }
    ));
    connection
        .send(&ClientMessage::Paste {
            window: WindowId(1),
            text: "exit".into(),
        })
        .await
        .unwrap();
    connection
        .send(&ClientMessage::Key {
            window: WindowId(1),
            key: gband_core::input::Key::plain(gband_core::input::KeyCode::Enter),
        })
        .await
        .unwrap();
    server.finished().await.unwrap();
}

#[test]
fn requests_without_a_server_fail_and_start_none() {
    let runtime_dir = runtime_dir("client-absent");
    let config = client_config(SessionName::default(), false);
    let transport = transport(&runtime_dir, "/bin/false", false);
    let socket = socket_path(&runtime_dir);
    let messages = [
        format!("{:#}", list_sessions(&config, &transport).unwrap_err()),
        format!("{:#}", kill_session(&config, &transport).unwrap_err()),
    ];
    for message in messages {
        assert!(message.contains("no server is running"), "{message}");
        assert!(message.contains(socket.to_str().unwrap()), "{message}");
    }
    assert!(!socket.exists());
}

#[test]
fn requests_reach_a_running_server() {
    let runtime_dir = runtime_dir("client-requests");
    let tokio = tokio::runtime::Runtime::new().unwrap();
    let server = tokio.spawn(gband_server::run(server_config(&runtime_dir)));
    let socket = socket_path(&runtime_dir);
    while !socket.exists() {
        std::thread::sleep(Duration::from_millis(10));
    }
    let config = client_config(SessionName::default(), false);
    let transport = transport(&runtime_dir, "/bin/false", false);

    let sessions = list_sessions(&config, &transport).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].name, SessionName::default());
    assert_eq!((sessions[0].windows, sessions[0].clients), (1, 0));

    let nope = client_config("nope".parse().unwrap(), false);
    let message = format!("{:#}", kill_session(&nope, &transport).unwrap_err());
    assert!(message.contains("nope"), "{message}");

    kill_session(&config, &transport).unwrap();
    tokio
        .block_on(async { tokio::time::timeout(Duration::from_secs(10), server).await })
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(!socket.exists());
}
