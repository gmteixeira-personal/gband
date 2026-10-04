#![allow(dead_code)]

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use gband_core::geometry::Size;
use gband_core::input::{Key, KeyCode};
use gband_core::layout::{Layout, PaneId, SessionAction};
use gband_protocol::{
    ClientMessage, Decoder, ExecutableId, Hello, HelloReply, PROTOCOL_VERSION, ServerMessage,
    SessionName, SessionSummary, encode, socket_path,
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

pub fn session(name: &str) -> SessionName {
    name.parse().unwrap()
}

pub fn config(runtime_dir: &Path, program: &[&str]) -> ServerConfig {
    ServerConfig {
        socket: socket_path(runtime_dir),
        session: SessionName::default(),
        program: program.iter().map(OsString::from).collect(),
        cwd: runtime_dir.to_path_buf(),
        executable: IDENTITY,
    }
}

pub struct TestServer {
    pub runtime_dir: PathBuf,
    socket: PathBuf,
    pub handle: JoinHandle<Result<()>>,
}

impl TestServer {
    pub async fn start(name: &str, program: &[&str]) -> Self {
        let runtime_dir = runtime_dir(name);
        Self::start_in(runtime_dir, program).await
    }

    pub async fn start_in(runtime_dir: PathBuf, program: &[&str]) -> Self {
        let config = config(&runtime_dir, program);
        Self::start_with(runtime_dir, config).await
    }

    pub async fn start_with(runtime_dir: PathBuf, config: ServerConfig) -> Self {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
        let socket = config.socket.clone();
        let handle = tokio::spawn(gband_server::run(config));
        let deadline = Instant::now() + TIMEOUT;
        while UnixStream::connect(&socket).await.is_err() {
            assert!(!handle.is_finished(), "server stopped before listening");
            assert!(Instant::now() < deadline, "server never listened");
            sleep(Duration::from_millis(10)).await;
        }
        Self {
            runtime_dir,
            socket,
            handle,
        }
    }

    pub fn socket(&self) -> PathBuf {
        self.socket.clone()
    }

    pub async fn attach(&self, cols: u16, rows: u16) -> TestClient {
        TestClient::attach(&self.socket(), cols, rows).await
    }

    pub async fn attach_to(&self, name: &str, cwd: &Path, cols: u16, rows: u16) -> TestClient {
        TestClient::attach_to(&self.socket(), session(name), cwd, cols, rows).await
    }

    pub async fn list(&self) -> Vec<SessionSummary> {
        let (mut stream, mut decoder, _) = TestClient::accepted(&self.socket(), 0, 0).await;
        write_frame(&mut stream, &ClientMessage::ListSessions).await;
        let reply = read_frame(&mut stream, &mut decoder).await;
        let Some(ServerMessage::Sessions(sessions)) = reply else {
            panic!("expected sessions, got {reply:?}");
        };
        assert!(closes(&mut stream).await);
        sessions
    }

    pub async fn listed(&self) -> Vec<(String, u32, u32)> {
        self.list()
            .await
            .into_iter()
            .map(|summary| (summary.name.to_string(), summary.panes, summary.clients))
            .collect()
    }

    pub async fn kill(&self, name: &str) -> ServerMessage {
        let (mut stream, mut decoder, _) = TestClient::accepted(&self.socket(), 0, 0).await;
        write_frame(
            &mut stream,
            &ClientMessage::KillSession {
                session: session(name),
            },
        )
        .await;
        let reply = timeout(TIMEOUT, read_frame(&mut stream, &mut decoder))
            .await
            .expect("no answer to the kill request")
            .expect("connection closed before answering");
        assert!(closes(&mut stream).await);
        reply
    }

    pub async fn screens(&self) -> HashMap<PaneId, vt100::Screen> {
        self.screens_of("default").await
    }

    pub async fn screens_of(&self, name: &str) -> HashMap<PaneId, vt100::Screen> {
        let mut client = self.attach_to(name, &self.runtime_dir, 0, 0).await;
        let screens = client
            .parsers
            .iter()
            .map(|(&pane, parser)| (pane, parser.screen().clone()))
            .collect();
        client.send(&ClientMessage::Detach).await;
        screens
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
    pub parsers: HashMap<PaneId, vt100::Parser>,
    pub layout: Layout,
    pub area: Size,
    pub focus: Vec<PaneId>,
    pub info: ServerMessage,
    pub exited: bool,
}

impl TestClient {
    pub async fn connect(socket: &Path) -> UnixStream {
        UnixStream::connect(socket).await.unwrap()
    }

    pub async fn attach(socket: &Path, cols: u16, rows: u16) -> Self {
        Self::attach_to(socket, SessionName::default(), Path::new("/"), cols, rows).await
    }

    pub async fn accepted(
        socket: &Path,
        cols: u16,
        rows: u16,
    ) -> (UnixStream, Decoder, ServerMessage) {
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
        assert!(matches!(info, ServerMessage::Info { .. }), "{info:?}");
        (stream, decoder, info)
    }

    pub async fn attach_to(
        socket: &Path,
        session: SessionName,
        cwd: &Path,
        cols: u16,
        rows: u16,
    ) -> Self {
        let (mut stream, decoder, info) = Self::accepted(socket, cols, rows).await;
        write_frame(
            &mut stream,
            &ClientMessage::Attach {
                session,
                cwd: cwd.to_path_buf(),
            },
        )
        .await;
        let mut client = Self {
            stream,
            decoder,
            parsers: HashMap::new(),
            layout: Layout::new(),
            area: Size::new(0, 0),
            focus: Vec::new(),
            info,
            exited: false,
        };
        match client.receive().await {
            Some(ServerMessage::Layout { .. }) => {}
            other => panic!("expected a layout, got {other:?}"),
        }
        while client
            .layout
            .panes()
            .any(|pane| !client.parsers.contains_key(&pane))
        {
            match client.receive().await {
                Some(ServerMessage::Snapshot { .. }) => {}
                other => panic!("expected a snapshot, got {other:?}"),
            }
        }
        client
    }

    pub fn panes(&self) -> Vec<PaneId> {
        self.layout.panes().collect()
    }

    pub fn first(&self) -> PaneId {
        self.layout
            .panes()
            .next()
            .expect("the layout holds no pane")
    }

    pub fn screen(&self) -> &vt100::Screen {
        self.pane_screen(self.first())
    }

    pub fn pane_screen(&self, pane: PaneId) -> &vt100::Screen {
        self.parsers
            .get(&pane)
            .unwrap_or_else(|| panic!("no screen for pane {pane}"))
            .screen()
    }

    pub async fn send(&mut self, message: &ClientMessage) {
        write_frame(&mut self.stream, message).await;
    }

    pub async fn act(&mut self, action: SessionAction) {
        self.send(&ClientMessage::Action(action)).await;
    }

    pub async fn key(&mut self, key: Key) {
        let pane = self.first();
        self.key_to(pane, key).await;
    }

    pub async fn key_to(&mut self, pane: PaneId, key: Key) {
        self.send(&ClientMessage::Key { pane, key }).await;
    }

    pub async fn type_line(&mut self, line: &str) {
        let pane = self.first();
        self.type_line_to(pane, line).await;
    }

    pub async fn type_line_to(&mut self, pane: PaneId, line: &str) {
        self.send(&ClientMessage::Paste {
            pane,
            text: line.to_string(),
        })
        .await;
        self.key_to(pane, Key::plain(KeyCode::Enter)).await;
    }

    pub async fn open_after(&mut self, after: PaneId) -> PaneId {
        let workspace = self.layout.workspaces()[self.layout.locate(after).unwrap().workspace].id;
        let seen = self.focus.len();
        self.act(SessionAction::OpenPane {
            workspace,
            after: Some(after),
        })
        .await;
        self.wait_until(|client| client.focus.len() > seen).await;
        let opened = self.focus[seen];
        self.wait_for_prompt(opened).await;
        opened
    }

    pub async fn receive(&mut self) -> Option<ServerMessage> {
        let message: ServerMessage = read_frame(&mut self.stream, &mut self.decoder).await?;
        match &message {
            ServerMessage::Layout { cols, rows, layout } => {
                self.layout = layout.clone();
                self.area = Size::new(*cols, *rows);
                self.parsers.retain(|pane, _| layout.contains(*pane));
            }
            ServerMessage::Snapshot {
                pane,
                cols,
                rows,
                contents,
            } => {
                assert!(self.layout.contains(*pane), "snapshot before its layout");
                let mut parser = vt100::Parser::new(*rows, *cols, 0);
                parser.process(contents);
                self.parsers.insert(*pane, parser);
            }
            ServerMessage::Update { pane, contents } => self
                .parsers
                .get_mut(pane)
                .unwrap_or_else(|| panic!("update before a snapshot of pane {pane}"))
                .process(contents),
            ServerMessage::Focus(pane) => {
                assert!(self.parsers.contains_key(pane), "focus before a snapshot");
                self.focus.push(*pane);
            }
            ServerMessage::Exited => self.exited = true,
            ServerMessage::Info { .. }
            | ServerMessage::Sessions(_)
            | ServerMessage::Killed
            | ServerMessage::NoSuchSession => panic!("unexpected {message:?} after attaching"),
        }
        Some(message)
    }

    fn describe(&self) -> String {
        self.layout
            .panes()
            .filter_map(|pane| {
                self.parsers
                    .get(&pane)
                    .map(|parser| format!("pane {pane}:\n{}", parser.screen().contents()))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub async fn wait_until(&mut self, predicate: impl Fn(&Self) -> bool) {
        let deadline = Instant::now() + TIMEOUT;
        while !predicate(self) {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match timeout(remaining, self.receive()).await {
                Ok(Some(_)) => {}
                Ok(None) => panic!("connection closed while waiting\n{}", self.describe()),
                Err(_) => panic!("timed out waiting\n{}", self.describe()),
            }
        }
    }

    pub async fn wait_for(&mut self, predicate: impl Fn(&vt100::Screen) -> bool) {
        self.wait_until(|client| predicate(client.screen())).await;
    }

    pub async fn wait_for_pane(
        &mut self,
        pane: PaneId,
        predicate: impl Fn(&vt100::Screen) -> bool,
    ) {
        self.wait_until(|client| {
            client
                .parsers
                .get(&pane)
                .is_some_and(|p| predicate(p.screen()))
        })
        .await;
    }

    pub async fn wait_for_prompt(&mut self, pane: PaneId) {
        self.wait_for_pane(pane, |screen| screen.contents().trim_end().ends_with('$'))
            .await;
    }

    pub async fn wait_for_text(&mut self, text: &str) {
        self.wait_for(|screen| screen.contents().contains(text))
            .await;
    }

    pub async fn wait_for_pane_text(&mut self, pane: PaneId, text: &str) {
        self.wait_for_pane(pane, |screen| screen.contents().contains(text))
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
        let expected = server.screens().await;
        let matches = expected.len() == client.parsers.len()
            && expected.iter().all(|(pane, screen)| {
                client
                    .parsers
                    .get(pane)
                    .is_some_and(|parser| same_screen(parser.screen(), screen))
            });
        if matches {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "client never matched the server\nclient:\n{}",
            client.describe()
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
