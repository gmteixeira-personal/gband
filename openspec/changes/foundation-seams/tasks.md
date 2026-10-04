## 1. Emulator crate

- [ ] 1.1 Create `crates/emulator` (`gband-emulator`) with the `Emulator` trait, the `Vt100` implementation and `pub type Grid = Vt100`, and move `LoggingCallbacks` from `crates/server/src/callbacks.rs` into it. Add it to the workspace dependencies in the root `Cargo.toml`. Verify with `cargo build -p gband-emulator`
- [ ] 1.2 Add `crates/emulator/tests/contract.rs`, generic over `E: Emulator` and instantiated for `Grid`. It covers every terminal-emulator spec scenario: coloured text, wide characters, the alternate screen with modes, cursor movement after a checkpoint, an empty diff, shrinking, and no write-back for plain output. Verify that `cargo test -p gband-emulator` passes
- [ ] 1.3 Move the server to `Grid`. `Pane` holds the emulator, `screen()` returns a checkpoint, `modes()` comes from the trait, and `connection.rs` builds snapshots and updates with `snapshot` and `diff`. The pane input thread writes `take_write_back` bytes to the PTY after each `process`. Delete `crates/server/src/callbacks.rs` and drop `vt100` from `crates/server/Cargo.toml`. Verify that `cargo test -p gband-server` passes
- [ ] 1.4 Move the client to `Grid`. `Display` keeps `HashMap<PaneId, Grid>`, and `render.rs` takes `&Grid` and renders `grid.screen()` through `PseudoTerminal` with the `tui_term::widget::Screen` bound. Drop `vt100` from `crates/client/Cargo.toml`. Verify that `cargo test -p gband-client` passes
- [ ] 1.5 If `terminal-query-replies` has landed, move its replies from the server's callbacks into the emulator's write-back queue, and verify that `crates/server/tests/queries.rs` passes unchanged. Otherwise mark this task done with no change

## 2. Framed I/O, transport and shared test harness

- [ ] 2.1 Add `crates/protocol/src/io.rs` with `MessageReader` (`recv`, `fill`, `try_recv`) and `MessageWriter` (`send`), and add `tokio` with `io-util` to `crates/protocol/Cargo.toml`. Verify with `crates/protocol/tests/io.rs` tests that read messages split across reads, read two messages from one read, report a clean end of stream as `None`, and report an oversized frame as an error
- [ ] 2.2 Replace `read_message` and `send` in `crates/server/src/connection.rs` with the framed halves, and make `serve` take an `AsyncRead` half and an `AsyncWrite` half. Verify that `cargo test -p gband-server` passes
- [ ] 2.3 Add `crates/client/src/transport.rs` with `Link`, `Transport` and `UnixTransport`, and move `connect_or_start` and `stop_server` behind it. Change `gband_client::run` to take `ClientConfig` and a transport, remove `runtime_dir` and `executable_path` from `ClientConfig`, and build `UnixTransport` in `src/main.rs`. Replace `read_frame` and `write_frame` in `connect.rs` with the framed halves. Verify that `cargo test --workspace` passes
- [ ] 2.4 Create `crates/test-support` (`gband-test-support`, `publish = false`) from `crates/server/tests/common/mod.rs`, with clients on the framed halves and one `Grid` per pane, plus `RelayTransport`. Point the server and client tests at it, delete `crates/server/tests/common/`, and make `tests/common/mod.rs` read the outer terminal through `Grid`. Replace the root `Cargo.toml`'s `vt100` dev-dependency with `gband-emulator` and `gband-test-support`. Verify that `cargo test --workspace` passes
- [ ] 2.5 Verify that `cargo tree -i vt100 -e normal` lists only `gband-emulator` and `tui-term` as dependents, and that `rg -n 'vt100' --type rust crates/server crates/client crates/test-support tests src` finds nothing
- [ ] 2.6 Add `crates/client/tests/transport.rs`. A client attaches through a `RelayTransport` whose reader and writer are separate `tokio::io::duplex` pipes, sees `relayed` after `printf 'relayed\n'` runs in the pane, detaches, and the server is still running. Verify that it passes

## 3. Version-first hello

- [ ] 3.1 Make the server decode the first frame's leading version with `postcard::take_from_bytes::<u32>` before it decodes the rest of the hello. It rejects a differing version whatever follows, and closes the connection without an answer on an empty payload or a matching version with an undecodable remainder. Verify with tests in `crates/server/tests/sync.rs`: a frame of version 3 followed by `0xff 0xff 0xff` is rejected with version 2, a version 2 frame with a truncated size is closed without an answer, an empty frame is closed without an answer, and `non_hello_first_frame_is_closed` accepts a rejection before the close
- [ ] 3.2 Verify that the hello and hello-reply pinned-bytes tests in `crates/protocol/tests/messages.rs` pass unchanged and that `PROTOCOL_VERSION` is still 2

## 4. Actions

- [ ] 4.1 Add `crates/core/src/action.rs` with `Action`, `SessionCommand` and `ClientAction`, all deriving `Serialize` and `Deserialize`, and add `View::resolve`. Verify with `crates/core/tests/view.rs` tests for resolving to the focused pane, every command with nothing focused, and open pane on the empty workspace
- [ ] 4.2 Change `crates/client/src/bindings.rs` to `BINDINGS: &[(Key, Action)]` with `Command::Run(Action)`, replacing `Binding` and `SendPrefix` with `ClientAction::SendKey(PREFIX)`. Replace `Display::resolve` with `View::resolve`, and make `dispatch(display, action)` in `crates/client/src/lib.rs` the one routing function. Verify that the existing binding unit tests pass after the type change, and add one that checks every table entry names an action of the kind the client-attach table gives
- [ ] 4.3 Add a test in `crates/client/tests/` that drives a client against a test server, dispatches `Action::Session(SessionCommand::ClosePane)` with the second of two panes focused, and verifies that the layout then holds only the first pane. Verify that `tests/attach.rs` still passes

## 5. Events

- [ ] 5.1 Add `crates/core/src/event.rs` with `LayoutEvent`, and make `Layout::open`, `remove` and `apply` return `Vec<LayoutEvent>`. Update the server's `Session` and the core tests that tested the boolean. Verify with `crates/core/tests/events.rs` tests for each session-events scenario: open in the empty workspace, the last pane of a middle workspace closing, consume into a neighbour, consume at the edge, and cycling and toggling widths
- [ ] 5.2 Add a test to `crates/core/tests/layout.rs` for the pane identity scenario, and extend the fixed operation table test to check that every pane keeps its identifier until it is removed
- [ ] 5.3 Add `crates/server/src/event.rs` with `SessionEvent`, the broadcast bus, `run_with_events` and the debug log subscriber. Send layout events and pane exits from `Session` in order, with each exit before its `PaneClosed`, and attach and detach events from the connection tasks. Verify with `crates/server/tests/events.rs` tests that a subscriber sees pane opened when a client opens a pane, the exit before pane closed when a shell exits, attach then detach for one client, and `Lagged` without blocking the session when it reads nothing while 2000 events are produced

## 6. Rendering snapshots and printf convergence

- [ ] 6.1 Add `insta` to the workspace dev-dependencies and to `crates/client`. Convert `crates/client/tests/render.rs` to `insta` snapshots rendered through `Terminal<TestBackend>`, and add cases for a coloured pane, a wide character cut at the clip edge, and a stacked column. Review each new snapshot by eye before accepting it, and commit `crates/client/tests/snapshots/`. Verify that `cargo test -p gband-client` passes with no pending snapshots
- [ ] 6.2 Add `crates/server/tests/programs.rs`, which runs `printf` programs and asserts that the test client's grid equals the server's for SGR colours and attributes, wide characters, cursor addressing and erase, the alternate screen with application cursor mode, a scroll region, and output longer than one screen. Verify that it passes

## 7. Finish

- [ ] 7.1 Run `.claude/flow-gate` and verify that formatting, clippy with `-D warnings` and the workspace tests all pass
- [ ] 7.2 Check every new or changed Rust file against the repository's coding rules, which forbid narrative comments, and verify that no comment remains except one explaining an upstream API restriction
- [ ] 7.3 Run `gband attach` in a real terminal, open two panes, run `nvim` in one and `ls --color` in the other, detach with Ctrl+A then `D`, reattach, and verify that both panes are restored with their colours and that arrow keys still work in nvim
