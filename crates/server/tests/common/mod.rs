#![allow(dead_code)]

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use gband_core::input::{Key, KeyCode};
use gband_protocol::{
    ClientMessage, Decoder, ExecutableId, Hello, HelloReply, PROTOCOL_VERSION, ServerMessage,
    encode, socket_path,
};
use gband_server::ServerConfig;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::task::JoinHandle;
use tokio::time::{Instant, sleep, timeout};

pub const TIMEOUT: Duration = Duration::from_secs(10);
pub const IDENTITY: ExecutableId = ExecutableId {
    device: 7,
    inode: 4242,
};

pub fn runtime_dir(name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("srv")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    path
}

pub fn config(runtime_dir: &Path, program: &[&str]) -> ServerConfig {
    ServerConfig {
        runtime_dir: runtime_dir.to_path_buf(),
        program: program.iter().map(OsString::from).collect(),
        cwd: runtime_dir.to_path_buf(),
        executable: IDENTITY,
    }
}

pub struct TestServer {
    pub runtime_dir: PathBuf,
    pub handle: JoinHandle<Result<()>>,
}

impl TestServer {
    pub async fn start(name: &str, program: &[&str]) -> Self {
        let runtime_dir = runtime_dir(name);
        Self::start_in(runtime_dir, program).await
    }

    pub async fn start_in(runtime_dir: PathBuf, program: &[&str]) -> Self {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
        let handle = tokio::spawn(gband_server::run(config(&runtime_dir, program)));
        let socket = socket_path(&runtime_dir);
        let deadline = Instant::now() + TIMEOUT;
        while UnixStream::connect(&socket).await.is_err() {
            assert!(!handle.is_finished(), "server stopped before listening");
            assert!(Instant::now() < deadline, "server never listened");
            sleep(Duration::from_millis(10)).await;
        }
        Self {
            runtime_dir,
            handle,
        }
    }

    pub fn socket(&self) -> PathBuf {
        socket_path(&self.runtime_dir)
    }

    pub async fn attach(&self, cols: u16, rows: u16) -> TestClient {
        TestClient::attach(&self.socket(), cols, rows).await
    }

    pub async fn screen(&self) -> vt100::Screen {
        let mut client = self.attach(0, 0).await;
        let screen = client.parser.screen().clone();
        client.send(&ClientMessage::Detach).await;
        screen
    }

    pub async fn finished(self) -> Result<()> {
        timeout(TIMEOUT, self.handle)
            .await
            .expect("server did not stop")
            .unwrap()
    }
}

pub struct TestClient {
    pub stream: UnixStream,
    pub decoder: Decoder,
    pub parser: vt100::Parser,
    pub info: ServerMessage,
    pub exited: bool,
}

impl TestClient {
    pub async fn connect(socket: &Path) -> UnixStream {
        UnixStream::connect(socket).await.unwrap()
    }

    pub async fn attach(socket: &Path, cols: u16, rows: u16) -> Self {
        let mut stream = Self::connect(socket).await;
        write_frame(
            &mut stream,
            &Hello {
                version: PROTOCOL_VERSION,
                cols,
                rows,
            },
        )
        .await;
        let mut decoder = Decoder::new();
        let reply: HelloReply = read_frame(&mut stream, &mut decoder).await.unwrap();
        assert_eq!(
            reply,
            HelloReply::Accepted {
                version: PROTOCOL_VERSION
            }
        );
        let info: ServerMessage = read_frame(&mut stream, &mut decoder).await.unwrap();
        let mut client = Self {
            stream,
            decoder,
            parser: vt100::Parser::new(24, 80, 0),
            info,
            exited: false,
        };
        match client.receive().await {
            Some(ServerMessage::Snapshot { .. }) => {}
            other => panic!("expected a snapshot, got {other:?}"),
        }
        client
    }

    pub async fn send(&mut self, message: &ClientMessage) {
        write_frame(&mut self.stream, message).await;
    }

    pub async fn type_line(&mut self, line: &str) {
        self.send(&ClientMessage::Paste(line.to_string())).await;
        self.send(&ClientMessage::Key(Key::plain(KeyCode::Enter)))
            .await;
    }

    pub async fn receive(&mut self) -> Option<ServerMessage> {
        let message: ServerMessage = read_frame(&mut self.stream, &mut self.decoder).await?;
        match &message {
            ServerMessage::Snapshot {
                cols,
                rows,
                contents,
            } => {
                self.parser = vt100::Parser::new(*rows, *cols, 0);
                self.parser.process(contents);
            }
            ServerMessage::Update(contents) => self.parser.process(contents),
            ServerMessage::Exited => self.exited = true,
            ServerMessage::Info { .. } => panic!("unexpected second info"),
        }
        Some(message)
    }

    pub async fn wait_for(&mut self, predicate: impl Fn(&vt100::Screen) -> bool) {
        let deadline = Instant::now() + TIMEOUT;
        while !predicate(self.parser.screen()) {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match timeout(remaining, self.receive()).await {
                Ok(Some(_)) => {}
                Ok(None) => panic!(
                    "connection closed while waiting; screen:\n{}",
                    self.parser.screen().contents()
                ),
                Err(_) => panic!(
                    "timed out waiting; screen:\n{}",
                    self.parser.screen().contents()
                ),
            }
        }
    }

    pub async fn wait_for_text(&mut self, text: &str) {
        self.wait_for(|screen| screen.contents().contains(text))
            .await;
    }

    pub async fn pump(&mut self, duration: Duration) -> bool {
        let deadline = Instant::now() + duration;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match timeout(remaining, self.receive()).await {
                Ok(Some(_)) => {}
                Ok(None) => return false,
                Err(_) => return true,
            }
        }
    }

    pub async fn drain_to_end(&mut self) {
        timeout(TIMEOUT, async { while self.receive().await.is_some() {} })
            .await
            .expect("connection did not close");
    }
}

pub fn same_screen(a: &vt100::Screen, b: &vt100::Screen) -> bool {
    a.size() == b.size()
        && a.contents_formatted() == b.contents_formatted()
        && a.cursor_position() == b.cursor_position()
        && a.hide_cursor() == b.hide_cursor()
        && a.application_cursor() == b.application_cursor()
        && a.bracketed_paste() == b.bracketed_paste()
}

pub async fn assert_converges(server: &TestServer, client: &mut TestClient) {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        client.pump(Duration::from_millis(100)).await;
        let expected = server.screen().await;
        if same_screen(client.parser.screen(), &expected) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "client never matched the server\nclient:\n{}\nserver:\n{}",
            client.parser.screen().contents(),
            expected.contents()
        );
    }
}

pub async fn wait_for_file(path: &Path) {
    let deadline = Instant::now() + TIMEOUT;
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "{} never appeared",
            path.display()
        );
        sleep(Duration::from_millis(20)).await;
    }
}

pub async fn write_frame<T: Serialize>(stream: &mut UnixStream, message: &T) {
    stream.write_all(&encode(message).unwrap()).await.unwrap();
}

pub async fn read_frame<T: DeserializeOwned>(
    stream: &mut UnixStream,
    decoder: &mut Decoder,
) -> Option<T> {
    let mut buffer = vec![0; 64 * 1024];
    loop {
        if let Some(message) = decoder.next_message().unwrap() {
            return Some(message);
        }
        let n = stream.read(&mut buffer).await.ok()?;
        if n == 0 {
            return None;
        }
        decoder.feed(&buffer[..n]).unwrap();
    }
}

pub async fn closes(stream: &mut UnixStream) -> bool {
    let mut buffer = [0; 256];
    let result = timeout(TIMEOUT, async {
        loop {
            match stream.read(&mut buffer).await {
                Ok(0) | Err(_) => return,
                Ok(_) => {}
            }
        }
    })
    .await;
    result.is_ok()
}
