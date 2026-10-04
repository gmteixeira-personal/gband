use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gband_client::{ClientConfig, connect};
use gband_protocol::{ClientMessage, ExecutableId, socket_path};
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
        runtime_dir: runtime_dir.to_path_buf(),
        executable_path: executable.into(),
        identity: CLIENT_IDENTITY,
        replace_mismatched: replace,
        log_dir: runtime_dir.join("log"),
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
        runtime_dir: runtime_dir.clone(),
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
        .send(&ClientMessage::Paste("exit".into()))
        .await
        .unwrap();
    connection
        .send(&ClientMessage::Key(gband_core::input::Key::plain(
            gband_core::input::KeyCode::Enter,
        )))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(10), server)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}
