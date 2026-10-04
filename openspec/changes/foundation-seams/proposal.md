## Why

A few interfaces are cheap to add while gband is small and hard to change once code depends on them. The next steps add Lua, agent awareness, persistence and an SSH transport. Each step plugs into one of these interfaces: the terminal grid, the byte stream between client and server, the hello, pane identity, and the action and event flow. Today `vt100` is used directly in nine files, and the client reaches the server only through a `UnixStream`. Key bindings resolve to a client-private command enum, and layout changes are visible only as a new layout value. Each later step would otherwise retrofit these seams across the code that layout-core is writing now. This change adds them right after layout-core, together with the test infrastructure that keeps them honest.

## What Changes

- **Protocol version first.** The hello already begins with the client's protocol version. The server now decodes that leading version on its own and answers before it decodes the rest of the hello. Only the leading version and the server's answer stay frozen across versions. The hello's other fields can then change in a later version, for example to carry the client terminal's capabilities, and an older server still rejects that client cleanly instead of closing on an undecodable frame.
- **Transport trait.** The client reaches the server through a `Transport` that yields a reader half and a writer half. The Unix socket is its only implementation. Starting a server when none is running, and replacing a server from another build, move behind it. The server serves any pair of byte-stream halves, so a later `gband proxy` only copies bytes. Framed message reading and writing moves into `gband-protocol`. Today the client, the server and two test harnesses each repeat that code.
- **Emulator trait.** A new crate, `gband-emulator`, holds an `Emulator` trait and its `vt100` implementation. The trait covers feeding output, resizing, input modes, the cursor, cells for rendering, snapshots, diffs from a checkpoint, and bytes the emulator must write back to the program. The server, the client and every test use the trait. No other crate depends on `vt100`. A contract test suite, generic over the trait, defines what any replacement must pass.
- **Stable pane identifiers.** layout-core already allocates pane identifiers that are never reused. This change adds that a pane keeps its identifier for its whole life: through consume, expel, width changes and the removal of other workspaces.
- **One Action enum.** `gband-core` gains `Action`, with three kinds: `View`, `Session` and `Client`. Every key binding resolves to an `Action`, and the client dispatches every action through one function that routes it by kind. Session commands without a target are resolved against the view in core, not in the client loop. Later sources, such as Lua, mouse, `gband msg` and a command palette, produce the same `Action` values.
- **Session events.** Every layout operation in core returns the events it caused: pane opened, closed or moved, column width changed, and workspace added or removed. The server publishes these events, plus pane exits and client attach and detach, in order on one event bus. Logging is its only subscriber for now. Lua, agents and persistence subscribe to it later.
- **Test infrastructure.**
  - A `gband-test-support` crate holds one `TestServer` and one `TestClient`, shared by the server, client and end-to-end tests. Today each test suite has its own copy.
  - New integration tests start a real server, attach the test client, and run commands such as `printf` with colours, wide characters, cursor movement and the alternate screen. They check that the client's grid matches the server's.
  - A relay test attaches through a transport that copies bytes to the socket, the same shape `gband proxy` will have.
  - Client rendering is covered by `insta` snapshot tests of ratatui `TestBackend` buffers.

Not in this change: Lua, agent awareness, persistence and the SSH transport. They plug into the action, event and transport seams added here.

## Capabilities

### New Capabilities

- `terminal-emulator`: the contract every terminal grid in gband meets. Covers reproduction from a snapshot, diffs from a checkpoint, resizing, reported input modes, and replies written back to the program.
- `transport`: how the client reaches the server. Covers byte-stream transports, the Unix socket transport, and a relay that copies bytes without changing the session's behaviour.
- `actions`: the three kinds of action, where each kind runs, and dispatch of every action source through one path.
- `session-events`: the events the session emits for layout changes, pane exits and client attach and detach, and the order in which subscribers receive them.

### Modified Capabilities

- `wire-protocol`: the handshake freezes only the hello's leading version and the server's answer. The server reads the version before it decodes the rest of the hello.
- `layout`: a pane keeps its identifier for its whole life. This capability is introduced by layout-core.

## Impact

- Code:
  - A new `crates/emulator/` crate.
  - New `action` and `event` modules in `crates/core/src/`, and layout operations that return events.
  - Framed I/O in `crates/protocol/src/`.
  - A transport module and one action dispatch path in `crates/client/src/`. `ClientConfig` loses its socket fields to `UnixTransport`.
  - In `crates/server/src/`, connections that serve generic stream halves, an event bus, and the emulator in place of `vt100` and `callbacks.rs`.
  - `src/main.rs` builds the Unix transport.
- Tests:
  - A new `crates/test-support/` crate replaces `crates/server/tests/common/` and the protocol helpers duplicated in `crates/client/tests/`.
  - New tests: emulator contract, actions, events, handshake prefix, printf convergence, relay attach, and render snapshots.
  - `tests/common/mod.rs` reads the outer terminal through the emulator.
- Dependencies: `insta` as a dev-dependency. `tokio` with I/O utilities in `gband-protocol`. `vt100` moves out of the server, the client and the root manifest into `gband-emulator`.
- Compatibility: the protocol version is unchanged and the hello's encoding is unchanged. A server from this change accepts and rejects exactly the clients a layout-core server does. It differs only by answering a hello whose fields after the version do not decode.
- Interaction with `terminal-query-replies`: that change writes query replies from `crates/server/src/callbacks.rs`. If it lands first, its replies move into the emulator's write-back output, and its tests must still pass unchanged.
- Out of scope: Lua, agents, persistence, the SSH transport and `gband proxy`, client capability negotiation, and reconnection.

## Coordination

### Author
- gmteixeira

### Depends On
- layout-core

### Expected Files
- Cargo.toml
- Cargo.lock
- crates/emulator/
- crates/test-support/
- crates/core/src/
- crates/core/tests/view.rs
- crates/core/tests/layout.rs
- crates/core/tests/events.rs
- crates/protocol/Cargo.toml
- crates/protocol/src/io.rs
- crates/protocol/src/lib.rs
- crates/protocol/tests/io.rs
- crates/server/
- crates/client/
- src/main.rs
- tests/common/mod.rs
- openspec/changes/foundation-seams/
