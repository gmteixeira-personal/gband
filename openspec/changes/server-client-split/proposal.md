## Why

The spike showed that `portable-pty`, `vt100`, `tui-term`, `ratatui` and `crossterm` work together in one process. Every later feature of gband rests on what this change builds: panes, the ribbon, agent awareness and remote use all need a server that owns the programs and a client that only presents them. Its main promise is that a session outlives its client, and nothing can be built on gband until that promise holds. This is build step 1 of `summary.txt`.

## What Changes

- `gband server` becomes a real session server. It listens on a Unix socket in the user's runtime directory, runs the user's shell in a PTY, and keeps that pane's screen in its own `vt100` grid. It keeps reading the PTY while no client is attached. It exits when the shell exits. A second server for the same socket refuses to start.
- `gband attach` becomes a real client. It connects to the socket and starts a detached server first when none is running. It exchanges protocol versions, receives a full snapshot of the pane, then applies `vt100` state diffs to its own grid, and renders that grid full screen with `tui-term`. It sends keys, pastes and resizes back to the server.
- The server encodes keys and pastes into PTY bytes with the existing `gband-core` encoder, using the modes in its own grid.
- Several clients may attach at once. Each one sees the pane and can type into it. The PTY takes the size of the client that most recently attached or resized.
- Pressing Ctrl+A then `d` detaches the client and leaves the session running. Pressing Ctrl+A twice sends one Ctrl+A to the pane.
- When the client is killed, the session continues. A new `gband attach` shows the same shell process with its screen, cursor and input modes exactly as they were. Output the shell wrote while no client was attached is included.
- `gband attach` refuses to run inside a gband pane, because attaching a session to itself would feed its own output back into it.
- After the handshake, the server tells each client its pid and which executable it runs from. When that executable differs from the client's, as after a rebuild or an upgrade:
  - A **debug build** of `gband attach` stops the old server, starts one from its own executable and attaches to that. While developing, the client always talks to the code just built. The old server is also replaced when it speaks a different protocol version.
  - A **release build** attaches anyway and, on leaving, prints a note that the server runs a different build. An upgrade never ends a session without the user asking.
- `gband kill-server` stops the running server and its shell. It works whatever protocol version the server speaks, because it uses only the pid the server records in its lock file and a signal. The server handles SIGTERM by hanging up its program and ending the session as if the program had exited.
- `gband-protocol` gains the wire format: length-prefixed `postcard` frames, a fixed handshake and the message set.
- `examples/spike.rs` is removed, because `gband attach` replaces it. Its log of escape sequences that `vt100` does not handle moves into the server.
- **BREAKING**: `gband server` and `gband attach` no longer exit at once without output. Both now run until the session ends or the client detaches.

## Capabilities

### New Capabilities

- `session-server`: the server process. Covers its socket, single-instance guard and pid record, PTY and shell, authoritative grid, PTY sizing, state sync to clients, lifecycle, stopping on SIGTERM and `gband kill-server`.
- `client-attach`: the client process. Covers starting a server, the handshake, handling a server from a different build, rendering, input forwarding, the detach key, nesting refusal and how the client exits.
- `wire-protocol`: the framing, versioning and messages exchanged between client and server.

### Modified Capabilities

- `command-line`: `server` and `attach` stop being stubs, and the requirement that both exit at once with status 0 and no output is replaced. `kill-server` joins them, and `--help` lists all three.
- `logging`: `kill-server` logs to the `client` file series. The scenario that expects `gband server` to exit with status 0 under an unparseable `GBAND_LOG` changes, because a server now runs until its shell exits.

## Impact

- Code: `src/main.rs`, `src/lib.rs`, new `src/paths.rs` and `src/executable.rs`, `crates/protocol/src/`, `crates/server/src/`, `crates/client/src/` and serde derives on the key types in `crates/core/src/input.rs`.
- Tests: `tests/subcommands.rs` changes for the new lifecycle. New integration tests drive `gband attach` in a PTY and run `gband kill-server`. New unit tests cover the protocol crate and the server.
- Dependencies: `rustix` is added to `gband-client`, to detach the server it starts, and to the root package, to read the uid for the fallback runtime directory. The root package drops the `tui-term`, `ratatui` and `crossterm` dev-dependencies the spike used. No new crate enters the workspace.
- Runtime: a socket and lock file appear under `$XDG_RUNTIME_DIR/gband/`.
- Out of scope: answering terminal queries (`terminal-query-replies`), scrollback, several panes, a configurable prefix key, SSH transport, reconnection with acknowledgements, and keeping panes alive when the server crashes.

## Coordination

### Author
- gmteixeira

### Depends On
- single-process-spike

### Expected Files
- Cargo.toml
- Cargo.lock
- src/main.rs
- src/lib.rs
- src/logging.rs
- src/paths.rs
- src/executable.rs
- crates/core/src/input.rs
- crates/protocol/src/
- crates/protocol/tests/
- crates/server/Cargo.toml
- crates/server/src/
- crates/server/tests/
- crates/client/Cargo.toml
- crates/client/src/
- crates/client/tests/
- tests/subcommands.rs
- tests/attach.rs
- tests/common/mod.rs
- tests/kill_server.rs
- examples/spike.rs
- openspec/changes/server-client-split/
