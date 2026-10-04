## Context

layout-core has landed. Its code is the base for this change:

- `vt100` appears in nine files. The server's `Pane` holds a `vt100::Parser<LoggingCallbacks>`, and `connection.rs` diffs `vt100::Screen` clones. The client keeps `HashMap<PaneId, vt100::Parser>`, and `render.rs` draws `vt100::Screen` through `tui-term`'s `PseudoTerminal`. `crates/server/tests/common/mod.rs` and `tests/common/mod.rs` each build their own parsers.
- Framed reads and writes exist three times: `read_message` and `send` in `crates/server/src/connection.rs`, `read_frame` and `write_frame` in `crates/client/src/connect.rs`, and again in `crates/server/tests/common/mod.rs`.
- The client's transport is `UnixStream`, through `connect_or_start` and `stop_server` in `connect.rs`. The server's `connection::serve` takes a `UnixStream`.
- The server reads the hello with `read_message::<Hello>`. A hello whose fields after the version do not decode is closed without an answer.
- `crates/client/src/bindings.rs` defines `Binding::{View, Session, Detach, SendPrefix}` and `SessionCommand`. `Display::resolve` in `crates/client/src/lib.rs` turns a `SessionCommand` into a `SessionAction`. `dispatch` matches on leader outcomes and bindings together.
- `Layout::open`, `remove` and `apply` return `bool`. The server's `Session` calls `publish` when they return true.
- `PaneId(pub u32)` is allocated from `Layout::next_pane` and never reused.

## Goals / Non-Goals

**Goals:**
- Replacing `vt100` touches one crate. A contract suite tells the replacement whether it is correct.
- Adding a transport, such as SSH, means implementing one trait in the client. The server is unchanged.
- A later protocol version can change the hello's fields after the version and still be rejected cleanly by this server.
- Lua, a command line and mouse input can produce the same actions keys produce, through one dispatch path.
- Lua, agents and persistence can subscribe to an ordered stream of session events without touching the code that changes the layout.
- One test harness serves the server, client and end-to-end tests, and rendering is covered by snapshot tests.

**Non-Goals:**
- Lua, agents, persistence, `gband proxy` and SSH.
- Client capability negotiation in the hello. The hello's shape stays as it is. This change only makes it changeable later.
- Events in the client. View changes stay local and emit nothing for now.
- Named, user-facing action strings. Lua will need them, and that change chooses them.
- Opaque pane identifiers. `PaneId` stays a `u32` newtype. At one pane per second, the 2^32 identifiers last over a century.

## Decisions

### Emulator crate and trait

A new crate, `crates/emulator` (`gband-emulator`), depends on `vt100`, `gband-core` and `tracing`. It holds the trait and the one implementation:

```rust
pub trait Emulator: Send + 'static {
    type Checkpoint: Clone + Send + Sync + 'static;
    type Screen;
    fn new(size: Size) -> Self;
    fn process(&mut self, bytes: &[u8]);
    fn resize(&mut self, size: Size);
    fn size(&self) -> Size;
    fn modes(&self) -> Modes;
    fn cursor(&self) -> Option<(u16, u16)>;
    fn screen(&self) -> &Self::Screen;
    fn checkpoint(&self) -> Self::Checkpoint;
    fn snapshot(&self) -> Vec<u8>;
    fn diff(&self, since: &Self::Checkpoint) -> Vec<u8>;
    fn take_write_back(&mut self) -> Vec<u8>;
}
pub type Grid = Vt100;
```

- **`Grid` is the alias every caller names.** The server, the client and the tests are written against `Grid` and the trait's methods, not generic over `E: Emulator`. A generic parameter threaded through `Pane`, `State`, `Display` and `Ribbon` would add noise and buy nothing while one implementation exists. Replacing the library means writing a new implementation and changing the alias. Alternative: generics everywhere. Rejected for now. The trait still bounds the alias, so no caller can use a method outside the trait.
- **`Size` and `Modes` come from `gband-core`.** `Modes` already drives `encode_key` and `encode_paste`, so the emulator reports the type the encoder takes.
- **Snapshots and diffs are terminal output, not a library format.** `Vt100` returns `state_formatted()` and `state_diff(&prev)`. The checkpoint is a `vt100::Screen` clone. The wire carries plain escape sequences, so replacing the library on one side does not change the protocol. A replacement may produce different bytes, as long as feeding them reproduces the screen.
- **Rendering stays with `tui-term`.** The trait's `Screen` is unbounded. The client requires `<Grid as Emulator>::Screen: tui_term::widget::Screen` where it renders. `tui-term`'s `Screen` and `Cell` traits are already library-neutral: `vt100::Screen` implements them behind `tui-term`'s default feature, and a replacement implements them for its own screen. Keeping the bound in the client keeps `ratatui` out of the server's build. Alternative: a gband cell type copied out of the emulator per frame. Rejected: it copies every visible cell per frame to replace a trait that already exists.
- **Callbacks move into the crate.** `LoggingCallbacks` moves from `crates/server/src/callbacks.rs` into `gband-emulator`, so unhandled sequences are still logged wherever a grid runs. `take_write_back` returns the bytes that callbacks queued for the program. Today that is nothing. If `terminal-query-replies` has landed, its replies move from the server's callbacks into this queue. The server's pane input thread then writes them to the PTY instead of the server writing them from callbacks. Either way, write-back goes through the pane's one writer, in order with keys.
- **A contract suite in `crates/emulator/tests/contract.rs`** runs every scenario of the terminal-emulator spec against a generic `fn check<E: Emulator>()` and instantiates it for `Grid`. A replacement must pass it unchanged before the alias can move.
- **No crate but `gband-emulator` depends on `vt100`.** The server, the client and the root manifest drop it, `tui-term` keeps its own `vt100` feature, and `cargo tree -i vt100 -e normal` names only `gband-emulator` and `tui-term`. Task 2.5 checks this.

### Transport

`crates/protocol/src/io.rs` adds framed halves, generic over `tokio::io` traits, and replaces the three copies:

```rust
pub struct MessageReader<R> { reader: R, decoder: Decoder, buffer: Vec<u8> }
impl<R: AsyncRead + Unpin> MessageReader<R> { pub async fn recv<T: DeserializeOwned>(&mut self) -> Result<Option<T>, IoError>; pub fn try_recv<T>(&mut self) -> Result<Option<T>, IoError>; pub async fn fill(&mut self) -> Result<bool, IoError>; }
pub struct MessageWriter<W> { writer: W }
impl<W: AsyncWrite + Unpin> MessageWriter<W> { pub async fn send<T: Serialize>(&mut self, message: &T) -> Result<(), IoError>; }
```

`fill` reads once and feeds the decoder, and `try_recv` drains decoded messages. The server's `select!` loop needs the read and the decoding apart, as it has them today. `gband-protocol` gains `tokio` with `io-util` only.

`crates/client/src/transport.rs` holds the trait and the Unix implementation:

```rust
pub struct Link { pub reader: Box<dyn AsyncRead + Send + Unpin>, pub writer: Box<dyn AsyncWrite + Send + Unpin> }
pub trait Transport {
    fn open(&self) -> impl Future<Output = Result<Link>> + Send;
    fn replace_server(&self) -> impl Future<Output = Result<()>> + Send;
}
pub struct UnixTransport { pub runtime_dir: PathBuf, pub executable_path: PathBuf }
```

- **Two halves, not one duplex stream.** An SSH proxy's child process has a separate stdin and stdout, so the trait takes that shape now. A `UnixStream` is split with `into_split`.
- **Starting and replacing a server belong to the transport.** For a Unix socket, opening starts a local server, and replacing stops the local one with the executable's `kill-server`. Over SSH, both happen on the remote host. `ClientConfig` keeps `identity`, `replace_mismatched` and `log_dir`. `runtime_dir` and `executable_path` move into `UnixTransport`, and `gband_client::run(config, transport)` takes it. `src/main.rs` builds the Unix transport.
- **The server is transport-agnostic through its connection handler.** `connection::serve` takes `impl AsyncRead` and `impl AsyncWrite` halves, so tests could serve any pipe, and a later proxy only copies bytes. The listener stays a Unix socket: every remote client reaches it through a proxy on the server's host, so no second listener is planned. Alternative: a server-side transport trait. Rejected, because nothing would implement it.
- **The relay test** lives in `crates/client/tests/transport.rs`. Its `RelayTransport` connects to the socket and returns the far ends of two `tokio::io::duplex` pipes. Two tasks copy each pipe to and from the socket. This is the byte path of `gband proxy`, with no SSH.

### Version-first hello

The server reads the first frame's payload as bytes. It decodes the leading `u32` with `postcard::take_from_bytes`, and compares the version before it decodes the remainder as the rest of a `Hello`. `Hello`'s field order already puts `version` first, so the encoding and its pinned-bytes test do not change, and the protocol version stays 2. The client keeps encoding the whole `Hello` at once.

A first frame that is some other message now often reads as a version mismatch, and gets a rejection before the close, where it used to get only the close. Both end the connection, and the rejection names the server's version, which helps the peer more. `non_hello_first_frame_is_closed` accepts a rejection followed by the close.

### Actions in core

`crates/core/src/action.rs`:

```rust
pub enum Action { View(ViewAction), Session(SessionCommand), Client(ClientAction) }
pub enum SessionCommand { OpenPane, ClosePane, ConsumeOrExpel(Direction), CycleWidth, ToggleFullWidth }
pub enum ClientAction { Detach, SendKey(Key) }
```

All three derive `Serialize` and `Deserialize`, so a later Lua or `gband msg` change can carry them without new types.

- `SessionCommand` moves from `bindings.rs` to core. `View::resolve(&self, SessionCommand) -> Option<SessionAction>` replaces `Display::resolve`, so the resolution rules are unit tested in `crates/core/tests/view.rs`.
- `SendPrefix` becomes `ClientAction::SendKey(PREFIX)`. A later binding can send any key, and the prefix stays data.
- `BINDINGS` becomes `&[(Key, Action)]`. `Leader::handle` returns `Command::{Send(Key), Run(Action), Discard}`. `dispatch(display, action) -> Step` in `crates/client/src/lib.rs` is the single function that routes an action by kind. The key path calls it for `Command::Run`. A future source calls it directly.
- The wire's `SessionAction` is unchanged.

### Events

`crates/core/src/event.rs` defines `LayoutEvent`:

```rust
pub enum LayoutEvent {
    PaneOpened { pane, workspace }, PaneClosed { pane, workspace },
    PaneMoved { pane, workspace, column, row },
    ColumnWidthChanged { workspace, column, width, full_width },
    WorkspaceAdded { workspace, index }, WorkspaceRemoved { workspace },
}
```

`Layout::open`, `remove` and `apply` return `Vec<LayoutEvent>` instead of `bool`. Empty means unchanged, so callers test `is_empty()` where they tested the boolean. The operations build the list as they change the structure. `normalize` reports the workspaces it adds and removes. Alternative: diff two layouts after each operation. Rejected, because a diff cannot tell a pane that moved from one that closed while another opened, and core already knows what it did.

`PaneMoved` is reported only for the pane an action moved. Columns that shift index because a neighbour was added or removed are not reported. Subscribers that need positions read the layout.

The server's `crates/server/src/event.rs` wraps them:

```rust
pub enum SessionEvent {
    Layout(LayoutEvent),
    PaneExited { pane: PaneId, status: String },
    ClientAttached { client: u64 },
    ClientDetached { client: u64 },
}
```

- **One `tokio::sync::broadcast` channel, capacity 1024.** The `Session` task sends layout events and pane exits in the order it applies them. It sends `PaneExited` before the `PaneClosed` that the removal produces. Connection tasks send attach events after the handshake accepts the client, and detach events when they end, using the client counter `serve` already has. `broadcast` gives a lagging receiver `RecvError::Lagged(n)` and never blocks the sender, which is what the spec requires.
- **The log subscriber** is a task spawned in `gband_server::run` that records every event at debug level.
- **Tests subscribe through `gband_server::run_with_events(config, sender)`.** `run` creates its own sender and calls it. Later in-process subscribers, such as the Lua host and persistence, take a receiver from the same sender.

### Test infrastructure

- **`crates/test-support`** (`gband-test-support`, `publish = false`) holds `TestServer`, `TestClient`, `runtime_dir`, `same_screen`, `assert_converges`, `wait_for_file` and `RelayTransport`. It is built from `crates/server/tests/common/mod.rs`, and its clients use `MessageReader` and `MessageWriter` and keep a `Grid` per pane. The server, client and root tests depend on it as a dev-dependency. `crates/server/tests/common/` is deleted. `tests/common/mod.rs` keeps its PTY-driven `Attached` harness, which tests the real binary, and reads the outer terminal through `Grid`.
- **printf convergence tests** in `crates/server/tests/programs.rs`. Each test starts a server whose program is `sh -c '<printf command>; sleep 100'`, attaches a `TestClient`, waits for the text, and asserts `same_screen` against the server's grid. The cases are SGR colours and attributes, wide characters, cursor addressing and erase, the alternate screen with application cursor mode, a scroll region, and output longer than one screen.
- **Render snapshots** in `crates/client/tests/render.rs`, using `insta` with `Buffer`'s `Debug` output, which includes styles. The existing render cases become snapshots, and new cases cover a coloured pane, a wide character cut at the clip edge, and a stacked column. They render through `ratatui::Terminal<TestBackend>`, calling `render::render` inside `draw`, so the frame path the real client uses is the one tested. Snapshots live in `crates/client/tests/snapshots/` and are committed.
- `insta` is a workspace dev-dependency, version `1`, with no extra features.

### Order of work

The tasks run in this order: emulator, then framed I/O and transport, then the hello, actions, events and test-support. The emulator moves first because the test-support crate and every harness name `Grid`. The test-support crate is extracted early, right after framed I/O exists, so each later step's tests are written once, against the shared harness.

## Risks / Trade-offs

- [`take_write_back` is empty until query replies exist, so it could rot] → The contract suite asserts it is empty for plain output. `terminal-query-replies` will be its first producer, and that change's tests will exercise it.
- [A type alias is a weaker seam than generics] → The trait is the only API the alias exposes, because callers import `Grid` and the `Emulator` trait, never `vt100`. Task 2.5 verifies that with `cargo tree` and an `rg` check.
- [`Layout` operations now allocate a `Vec` per call] → They run only on user actions and exits.
- [`broadcast` keeps the 1024 most recent events per lagging subscriber] → The log subscriber never lags in practice. Persistence and Lua must treat `Lagged` as "re-read the layout", which their own changes will specify.
- [Rejecting a non-hello first frame instead of only closing it] → No legitimate peer sends one. The rejection is a valid answer that every version can decode.
- [A larger change touching every crate] → Each task group leaves the workspace building and its tests passing, so the change can be reviewed and gated group by group.
