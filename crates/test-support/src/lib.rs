use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Result, bail};
use gband_client::{Connection, Link, Transport};
use gband_core::geometry::Size;
use gband_core::input::{Key, KeyCode};
use gband_core::layout::{Layout, LayoutOptions, SessionAction, WindowId};
pub use gband_emulator::{Emulator, Grid};
use gband_protocol::{
    ClientMessage, ExecutableId, Hello, HelloReply, IoError, MessageReader, MessageWriter,
    PROTOCOL_VERSION, ServerMessage, SessionName, SessionSummary, Value, socket_path,
};
pub use gband_scratch::Scratch;
use gband_server::{INITIAL_AREA, ServerConfig};
use ratatui::buffer::Cell;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::task::JoinHandle;
use tokio::time::{Instant, sleep, timeout};
use tui_term::widget::{Cell as _, Screen};

pub const TIMEOUT: Duration = Duration::from_secs(10);
pub const IDENTITY: ExecutableId = ExecutableId {
    device: 7,
    inode: 4242,
};

const RELAY_BUFFER_LEN: usize = 64 * 1024;

pub type Reader = MessageReader<Box<dyn AsyncRead + Send + Unpin>>;
pub type Writer = MessageWriter<Box<dyn AsyncWrite + Send + Unpin>>;

pub fn runtime_dir(name: &str) -> Scratch {
    Scratch::new("srv", name)
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
        area: INITIAL_AREA,
        executable: IDENTITY,
        options: tokio::sync::watch::channel(LayoutOptions::default()).1,
        scripting: None,
        channel: None,
    }
}

pub fn cell(grid: &Grid, row: u16, col: u16) -> Option<Cell> {
    let source = Screen::cell(grid.screen(), row, col)?;
    let mut cell = Cell::default();
    source.apply(&mut cell);
    Some(cell)
}

pub struct TestServer {
    socket: PathBuf,
    pub handle: JoinHandle<Result<()>>,
    pub runtime_dir: Scratch,
}

impl TestServer {
    pub async fn start(name: &str, program: &[&str]) -> Self {
        let runtime_dir = runtime_dir(name);
        Self::start_in(runtime_dir, program).await
    }

    pub async fn start_in(runtime_dir: Scratch, program: &[&str]) -> Self {
        let config = config(&runtime_dir, program);
        Self::start_with(runtime_dir, config).await
    }

    pub async fn start_with(runtime_dir: Scratch, config: ServerConfig) -> Self {
        let socket = config.socket.clone();
        Self::start_running(runtime_dir, socket, gband_server::run(config)).await
    }

    pub async fn start_running(
        runtime_dir: Scratch,
        socket: PathBuf,
        server: impl Future<Output = Result<()>> + Send + 'static,
    ) -> Self {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
        let handle = tokio::spawn(server);
        let deadline = Instant::now() + TIMEOUT;
        while UnixStream::connect(&socket).await.is_err() {
            assert!(!handle.is_finished(), "server stopped before listening");
            assert!(Instant::now() < deadline, "server never listened");
            sleep(Duration::from_millis(10)).await;
        }
        Self {
            socket,
            handle,
            runtime_dir,
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
        let (mut peer, _) = TestClient::accepted(&self.socket(), 0, 0).await;
        peer.send(&ClientMessage::ListSessions).await;
        let reply = peer.recv().await;
        let Some(ServerMessage::Sessions(sessions)) = reply else {
            panic!("expected sessions, got {reply:?}");
        };
        assert!(peer.closes().await);
        sessions
    }

    pub async fn listed(&self) -> Vec<(String, u32, u32)> {
        self.list()
            .await
            .into_iter()
            .map(|summary| (summary.name.to_string(), summary.windows, summary.clients))
            .collect()
    }

    pub async fn kill(&self, name: &str) -> ServerMessage {
        let (mut peer, _) = TestClient::accepted(&self.socket(), 0, 0).await;
        peer.send(&ClientMessage::KillSession {
            session: session(name),
        })
        .await;
        let reply = timeout(TIMEOUT, peer.recv())
            .await
            .expect("no answer to the kill request")
            .expect("connection closed before answering");
        assert!(peer.closes().await);
        reply
    }

    pub async fn screens(&self) -> HashMap<WindowId, Grid> {
        self.screens_of("default").await
    }

    pub async fn screens_of(&self, name: &str) -> HashMap<WindowId, Grid> {
        let mut client = self.attach_to(name, &self.runtime_dir, 0, 0).await;
        let screens = std::mem::take(&mut client.grids);
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

pub struct Peer {
    pub reader: Reader,
    pub writer: Writer,
}

impl Peer {
    pub async fn connect(socket: &Path) -> Self {
        let (reader, writer) = UnixStream::connect(socket).await.unwrap().into_split();
        Self::new(Box::new(reader), Box::new(writer))
    }

    pub fn new(
        reader: Box<dyn AsyncRead + Send + Unpin>,
        writer: Box<dyn AsyncWrite + Send + Unpin>,
    ) -> Self {
        Self {
            reader: MessageReader::new(reader),
            writer: MessageWriter::new(writer),
        }
    }

    pub async fn send<T: Serialize>(&mut self, message: &T) {
        self.writer.send(message).await.unwrap();
    }

    pub async fn send_bytes(&mut self, bytes: &[u8]) {
        self.writer.get_mut().write_all(bytes).await.unwrap();
    }

    pub async fn recv<T: DeserializeOwned>(&mut self) -> Option<T> {
        match self.reader.recv().await {
            Ok(message) => message,
            Err(IoError::Io(_)) => None,
            Err(error) => panic!("undecodable frame: {error}"),
        }
    }

    pub async fn closes(&mut self) -> bool {
        timeout(TIMEOUT, async {
            loop {
                match self.reader.fill().await {
                    Ok(true) => {}
                    Ok(false) | Err(IoError::Io(_)) => return,
                    Err(error) => panic!("undecodable frame: {error}"),
                }
            }
        })
        .await
        .is_ok()
    }
}

impl From<Connection> for Peer {
    fn from(connection: Connection) -> Self {
        Self {
            reader: connection.reader,
            writer: connection.writer,
        }
    }
}

pub struct TestClient {
    pub peer: Peer,
    pub grids: HashMap<WindowId, Grid>,
    pub layout: Layout,
    pub area: Size,
    pub focus: Vec<WindowId>,
    pub opened: Vec<(u32, Option<WindowId>)>,
    pub info: ServerMessage,
    pub exited: bool,
    pub bridge: Vec<ServerMessage>,
    pub states: HashMap<WindowId, BTreeMap<String, Value>>,
    pub names: HashMap<WindowId, (String, Option<String>)>,
}

impl TestClient {
    pub async fn attach(socket: &Path, cols: u16, rows: u16) -> Self {
        Self::attach_to(socket, SessionName::default(), Path::new("/"), cols, rows).await
    }

    pub async fn accepted(socket: &Path, cols: u16, rows: u16) -> (Peer, ServerMessage) {
        let mut peer = Peer::connect(socket).await;
        peer.send(&Hello {
            version: PROTOCOL_VERSION,
            cols,
            rows,
        })
        .await;
        let reply: HelloReply = peer.recv().await.unwrap();
        assert_eq!(
            reply,
            HelloReply::Accepted {
                version: PROTOCOL_VERSION
            }
        );
        let info: ServerMessage = peer.recv().await.unwrap();
        assert!(matches!(info, ServerMessage::Info { .. }), "{info:?}");
        (peer, info)
    }

    pub async fn attach_to(
        socket: &Path,
        session: SessionName,
        cwd: &Path,
        cols: u16,
        rows: u16,
    ) -> Self {
        let (peer, info) = Self::accepted(socket, cols, rows).await;
        Self::attach_over(peer, info, session, cwd).await
    }

    pub async fn attach_over(
        mut peer: Peer,
        info: ServerMessage,
        session: SessionName,
        cwd: &Path,
    ) -> Self {
        peer.send(&ClientMessage::Attach {
            session,
            cwd: cwd.to_path_buf(),
        })
        .await;
        let mut client = Self {
            peer,
            grids: HashMap::new(),
            layout: Layout::new(),
            area: Size::new(0, 0),
            focus: Vec::new(),
            opened: Vec::new(),
            info,
            exited: false,
            bridge: Vec::new(),
            states: HashMap::new(),
            names: HashMap::new(),
        };
        match client.receive().await {
            Some(ServerMessage::Layout { .. }) => {}
            other => panic!("expected a layout, got {other:?}"),
        }
        while client
            .layout
            .windows()
            .any(|window| !client.grids.contains_key(&window))
        {
            match client.receive().await {
                Some(ServerMessage::Snapshot { .. }) => {}
                other => panic!("expected a snapshot, got {other:?}"),
            }
        }
        let mut states = false;
        loop {
            match client.receive().await {
                Some(ServerMessage::Requirements(_)) => return client,
                Some(ServerMessage::WindowName { .. }) if !states => {}
                Some(ServerMessage::WindowState { .. }) => states = true,
                other => panic!(
                    "expected window names, then window states, then requirements, got {other:?}"
                ),
            }
        }
    }

    pub fn windows(&self) -> Vec<WindowId> {
        self.layout.windows().collect()
    }

    pub fn first(&self) -> WindowId {
        self.layout
            .windows()
            .next()
            .expect("the layout holds no window")
    }

    pub fn screen(&self) -> &Grid {
        self.window_screen(self.first())
    }

    pub fn window_screen(&self, window: WindowId) -> &Grid {
        self.grids
            .get(&window)
            .unwrap_or_else(|| panic!("no screen for window {window}"))
    }

    pub async fn send(&mut self, message: &ClientMessage) {
        self.peer.send(message).await;
    }

    pub async fn act(&mut self, action: SessionAction) {
        self.send(&ClientMessage::Action(action)).await;
    }

    pub async fn show(&mut self, windows: &[WindowId]) {
        self.send(&ClientMessage::Shown(windows.to_vec())).await;
    }

    pub async fn show_all(&mut self) {
        let windows = self.windows();
        self.show(&windows).await;
    }

    pub async fn key(&mut self, key: Key) {
        let window = self.first();
        self.key_to(window, key).await;
    }

    pub async fn key_to(&mut self, window: WindowId, key: Key) {
        self.send(&ClientMessage::Key { window, key }).await;
    }

    pub async fn type_line(&mut self, line: &str) {
        let window = self.first();
        self.type_line_to(window, line).await;
    }

    pub async fn type_line_to(&mut self, window: WindowId, line: &str) {
        self.send(&ClientMessage::Paste {
            window,
            text: line.to_string(),
        })
        .await;
        self.key_to(window, Key::plain(KeyCode::Enter)).await;
    }

    pub async fn open_after(&mut self, after: WindowId) -> WindowId {
        let band = self.layout.bands()[self.layout.locate(after).unwrap().band].id;
        let seen = self.focus.len();
        self.act(SessionAction::open(band, Some(after), None)).await;
        self.wait_until(|client| client.focus.len() > seen).await;
        let opened = self.focus[seen];
        self.wait_for_prompt(opened).await;
        opened
    }

    pub async fn receive(&mut self) -> Option<ServerMessage> {
        let message: ServerMessage = self.peer.recv().await?;
        self.apply(&message);
        Some(message)
    }

    pub fn apply(&mut self, message: &ServerMessage) {
        match message {
            ServerMessage::Layout { cols, rows, layout } => {
                self.layout = layout.clone();
                self.area = Size::new(*cols, *rows);
                self.grids.retain(|window, _| layout.contains(*window));
                self.names.retain(|window, _| layout.contains(*window));
            }
            ServerMessage::Snapshot {
                window,
                cols,
                rows,
                contents,
            } => {
                assert!(self.layout.contains(*window), "snapshot before its layout");
                let mut grid = Grid::new(Size::new(*cols, *rows));
                grid.process(contents);
                self.grids.insert(*window, grid);
            }
            ServerMessage::Update { window, contents } => self
                .grids
                .get_mut(window)
                .unwrap_or_else(|| panic!("update before a snapshot of window {window}"))
                .process(contents),
            ServerMessage::Focus(window) => {
                assert!(self.grids.contains_key(window), "focus before a snapshot");
                self.focus.push(*window);
            }
            ServerMessage::Opened { request, window } => {
                if let Some(window) = window {
                    assert!(self.grids.contains_key(window), "opened before a snapshot");
                }
                self.opened.push((*request, *window));
            }
            ServerMessage::Exited => self.exited = true,
            ServerMessage::WindowName {
                window,
                automatic,
                manual,
            } => {
                assert!(self.grids.contains_key(window), "name before a snapshot");
                self.names
                    .insert(*window, (automatic.clone(), manual.clone()));
            }
            ServerMessage::WindowState { window, key, value } => {
                let state = self.states.entry(*window).or_default();
                match value {
                    Some(value) => state.insert(key.clone(), value.clone()),
                    None => state.remove(key),
                };
                self.bridge.push(message.clone());
            }
            ServerMessage::Event { .. }
            | ServerMessage::Result { .. }
            | ServerMessage::Requirements(_)
            | ServerMessage::ServerError(_) => self.bridge.push(message.clone()),
            ServerMessage::Info { .. }
            | ServerMessage::Sessions(_)
            | ServerMessage::Killed
            | ServerMessage::NoSuchSession => panic!("unexpected {message:?} after attaching"),
        }
    }

    pub fn describe(&self) -> String {
        self.layout
            .windows()
            .filter_map(|window| {
                self.grids
                    .get(&window)
                    .map(|grid| format!("window {window}:\n{}", grid.contents()))
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

    pub async fn wait_for(&mut self, predicate: impl Fn(&Grid) -> bool) {
        self.wait_until(|client| predicate(client.screen())).await;
    }

    pub async fn wait_for_window(&mut self, window: WindowId, predicate: impl Fn(&Grid) -> bool) {
        self.wait_until(|client| client.grids.get(&window).is_some_and(&predicate))
            .await;
    }

    pub async fn wait_for_prompt(&mut self, window: WindowId) {
        self.wait_for_window(window, |screen| screen.contents().trim_end().ends_with('$'))
            .await;
    }

    pub async fn wait_for_text(&mut self, text: &str) {
        self.wait_for(|screen| screen.contents().contains(text))
            .await;
    }

    pub async fn wait_for_line(&mut self, line: &str) {
        self.wait_for(|screen| screen.contents().lines().any(|candidate| candidate == line))
            .await;
    }

    pub async fn wait_for_window_text(&mut self, window: WindowId, text: &str) {
        self.wait_for_window(window, |screen| screen.contents().contains(text))
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

pub fn same_screen(a: &Grid, b: &Grid) -> bool {
    a.size() == b.size()
        && a.snapshot() == b.snapshot()
        && a.cursor() == b.cursor()
        && a.modes() == b.modes()
}

pub async fn assert_converges(server: &TestServer, client: &mut TestClient) {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        client.pump(Duration::from_millis(100)).await;
        let expected = server.screens().await;
        let matches = expected.len() == client.grids.len()
            && expected.iter().all(|(window, screen)| {
                client
                    .grids
                    .get(window)
                    .is_some_and(|grid| same_screen(grid, screen))
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

pub struct RelayTransport {
    pub socket: PathBuf,
}

impl fmt::Display for RelayTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a relay to {}", self.socket.display())
    }
}

impl Transport for RelayTransport {
    async fn open(&self, _: Size) -> Result<Link> {
        let (mut socket_reader, mut socket_writer) =
            UnixStream::connect(&self.socket).await?.into_split();
        let (reader, mut to_client) = tokio::io::duplex(RELAY_BUFFER_LEN);
        let (mut from_client, writer) = tokio::io::duplex(RELAY_BUFFER_LEN);
        tokio::spawn(async move {
            let _ = tokio::io::copy(&mut socket_reader, &mut to_client).await;
            let _ = to_client.shutdown().await;
        });
        tokio::spawn(async move {
            let _ = tokio::io::copy(&mut from_client, &mut socket_writer).await;
            let _ = socket_writer.shutdown().await;
        });
        Ok(Link {
            reader: Box::new(reader),
            writer: Box::new(writer),
        })
    }

    fn may_replace_server(&self) -> bool {
        false
    }

    async fn replace_server(&self) -> Result<()> {
        bail!("{self} cannot replace the server")
    }
}
