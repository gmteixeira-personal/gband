use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gband_client::{ClientConfig, connect, kill_session, list_sessions};
use gband_core::layout::PaneId;
use gband_protocol::{ClientMessage, ExecutableId, ServerMessage, SessionName, socket_path};
use gband_server::ServerConfig;

const SERVER_IDENTITY: ExecutableId = ExecutableId {
    device: 1,
    inode: 1,
};
const CLIENT_IDENTITY: ExecutableId = ExecutableId {
    device: 2,
    inode: 2,
};

fn runtime_dir(name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("cli")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    path
}

fn client_config(runtime_dir: &Path, executable: &str, replace: bool) -> ClientConfig {
    ClientConfig {
        socket: socket_path(runtime_dir),
        session: SessionName::default(),
        start_server: true,
        executable_path: executable.into(),
        identity: CLIENT_IDENTITY,
        replace_mismatched: replace,
        log_dir: runtime_dir.join("log"),
        kill_command: "gband kill-server".to_owned(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn server_that_exits_at_once_fails_fast() {
    let runtime_dir = runtime_dir("false");
    let started = Instant::now();
    let error = connect(&client_config(&runtime_dir, "/bin/false", true))
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
    let runtime_dir = runtime_dir("stale");
    let server = tokio::spawn(gband_server::run(ServerConfig {
        socket: socket_path(&runtime_dir),
        session: SessionName::default(),
        program: vec![OsString::from("/bin/sh")],
        cwd: runtime_dir.clone(),
        executable: SERVER_IDENTITY,
    }));
    let socket = socket_path(&runtime_dir);
    while !socket.exists() {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let mut connection = connect(&client_config(&runtime_dir, "/bin/false", false))
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
            pane: PaneId(1),
            text: "exit".into(),
        })
        .await
        .unwrap();
    connection
        .send(&ClientMessage::Key {
            pane: PaneId(1),
            key: gband_core::input::Key::plain(gband_core::input::KeyCode::Enter),
        })
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(10), server)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}

#[test]
fn requests_without_a_server_fail_and_start_none() {
    let runtime_dir = runtime_dir("absent");
    let config = ClientConfig {
        start_server: false,
        ..client_config(&runtime_dir, "/bin/false", false)
    };
    let socket = socket_path(&runtime_dir);
    let messages = [
        format!("{:#}", list_sessions(&config).unwrap_err()),
        format!("{:#}", kill_session(&config).unwrap_err()),
    ];
    for message in messages {
        assert!(message.contains("no server is running"), "{message}");
        assert!(message.contains(socket.to_str().unwrap()), "{message}");
    }
    assert!(!socket.exists());
}

#[test]
fn requests_reach_a_running_server() {
    let runtime_dir = runtime_dir("requests");
    let tokio = tokio::runtime::Runtime::new().unwrap();
    let server = tokio.spawn(gband_server::run(ServerConfig {
        socket: socket_path(&runtime_dir),
        session: SessionName::default(),
        program: vec![OsString::from("/bin/sh")],
        cwd: runtime_dir.clone(),
        executable: SERVER_IDENTITY,
    }));
    let socket = socket_path(&runtime_dir);
    while !socket.exists() {
        std::thread::sleep(Duration::from_millis(10));
    }
    let config = ClientConfig {
        start_server: false,
        ..client_config(&runtime_dir, "/bin/false", false)
    };

    let sessions = list_sessions(&config).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].name, SessionName::default());
    assert_eq!((sessions[0].panes, sessions[0].clients), (1, 0));

    let nope = ClientConfig {
        session: "nope".parse().unwrap(),
        ..config
    };
    let message = format!("{:#}", kill_session(&nope).unwrap_err());
    assert!(message.contains("nope"), "{message}");

    let config = ClientConfig {
        session: SessionName::default(),
        ..nope
    };
    kill_session(&config).unwrap();
    tokio
        .block_on(async { tokio::time::timeout(Duration::from_secs(10), server).await })
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(!socket.exists());
}
