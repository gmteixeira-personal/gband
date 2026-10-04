## 1. Wire protocol in gband-protocol

- [ ] 1.1 Derive `Serialize` and `Deserialize` on `Key`, `KeyCode` and `Modifiers` in `crates/core/src/input.rs`, and verify that `cargo test -p gband-core` still passes
- [ ] 1.2 Add `PROTOCOL_VERSION = 1`, `Hello`, `HelloReply`, `ClientMessage` and `ServerMessage` to `crates/protocol/src/`, as the design describes, and verify with tests in `crates/protocol/tests/` that every message round-trips through `postcard` and that the encoded bytes of `Hello { version: 1, cols: 80, rows: 24 }` and both `HelloReply` variants equal pinned literals
- [ ] 1.3 Add the frame encoder and the chunk-fed `Decoder` with the 16 MiB limit, and verify with tests that a frame split at every byte boundary decodes to exactly one message, that two frames in one chunk decode in order, that a header announcing 17 MiB is rejected before any payload is buffered, and that an undecodable payload is an error

## 2. Runtime directory

- [ ] 2.1 Add `src/paths.rs`, exposed from `src/lib.rs`, resolving the runtime directory from `XDG_RUNTIME_DIR` or `/tmp/gband-<uid>`, and verify with unit tests covering an absolute `XDG_RUNTIME_DIR`, a relative one and an unset one
- [ ] 2.2 Add the directory preparation the server uses: create with mode `0700`, then refuse a directory not owned by the user or with any group or other permission bit. Verify with tests that a missing directory is created `0700` and that a `0777` directory is refused with an error naming the path and its mode

## 3. Session server in gband-server

- [ ] 3.1 Add `ServerConfig` and `run`, which takes the `default.lock` lock, spawns the program in an 80×24 PTY with `TERM`, `COLORTERM` and `GBAND` set, then replaces any socket file and binds. Verify with tests in `crates/server/tests/` that a second `run` on the same directory fails with an error naming the socket path while the first keeps running, that a leftover socket file with no holder is replaced, and that a program that cannot be spawned fails before the socket exists
- [ ] 3.2 Move the spike's `LoggingCallbacks` into the server and feed the PTY reader thread's output into the shared parser with a generation `watch`. Verify with a test that 1 MiB of output written with no client attached completes, and that the screen then holds its last line
- [ ] 3.3 Add the per-client task: handshake, snapshot, then updates on each generation change diffed against the last-sent screen, with a snapshot instead whenever the size differs. Verify with tests that a client applying the snapshot and updates to its own parser matches the server's `contents()`, cursor position and input modes after `printf` output, and that a client that stops reading during 10 000 lines matches once it resumes
- [ ] 3.4 Reject a mismatched version with `HelloReply::Rejected` and close, and close on a non-hello first frame, an oversized frame or an undecodable frame. Verify with tests for each case that the server keeps serving a second, valid client
- [ ] 3.5 Add the input task that encodes `Key` and `Paste` with the parser's current modes and writes the PTY. Verify with tests that Up is written as `\x1bOA` after the program prints `\x1b[?1h`, and that keys from two clients both reach the program
- [ ] 3.6 Apply hello sizes and `Resize` to the PTY and the parser, with the most recent one winning. Verify with tests that a second client's hello size becomes the PTY size, that a resize from the first client then wins, that a detach changes nothing, and that `stty size` in the program reports the latest size
- [ ] 3.7 On program exit, send each client a last update and `Exited`, remove the socket file and return `Ok`. On `Detach`, close that client only. Verify with tests that both attached clients receive `Exited`, that the socket file is gone, and that a client detaching leaves the program running

## 4. Client in gband-client

- [ ] 4.1 Add the prefix-key state machine as its own module, and verify with unit tests that Ctrl+A then `d` yields detach, that Ctrl+A twice yields one Ctrl+A key, that Ctrl+A then `x` yields nothing, and that keys without the prefix pass through
- [ ] 4.2 Add `rustix` to `crates/client/Cargo.toml` and the connect-or-start logic: on `NotFound` or `ConnectionRefused`, spawn the given executable with `server`, null stdio and `setsid`, then retry every 25 ms for up to 5 s, stopping early when the child exits. Verify with a test that a child exiting at once fails well within 5 s with an error naming the socket path
- [ ] 4.3 Add `ClientConfig`, `Outcome` and `run`: handshake before touching the terminal, then `ratatui::init`, bracketed paste, a restore guard, and a main loop that applies snapshots and updates to its parser, draws with `PseudoTerminal`, places the cursor, and sends keys, pastes and resizes. Verify with `cargo build -p gband-client` and with the end-to-end tests in section 5

## 5. Binary and end-to-end tests

- [ ] 5.1 Wire `gband server` to `gband_server::run` with `$SHELL` or the login shell and the current directory. Wire `gband attach` to the nesting check, the terminal check and `gband_client::run`, printing `[detached]`, `[exited]` or `[lost server]` after the terminal is restored, with the exit statuses the spec gives. Verify with `cargo run -- --help`, which still lists only `server` and `attach`
- [ ] 5.2 Update `tests/subcommands.rs` so every `gband server` run uses `SHELL=/bin/true` and its own `XDG_RUNTIME_DIR`, and `missing_directory_is_created` expects `gband attach` without a terminal to exit 1 with one stderr line. Verify that `cargo test --test subcommands` passes
- [ ] 5.3 Add `tests/attach.rs`, which runs `gband attach` in a `portable-pty` and reads its screen through `vt100` with `SHELL=/bin/sh` and temporary `XDG_RUNTIME_DIR` and `XDG_STATE_HOME`. Each test kills its shell in a drop guard. Verify with tests that the first attach starts a server and shows a prompt whose `pwd` is the client's directory, and that killing the client with SIGKILL and attaching again shows the same screen, the same cursor position and the same `echo $$`
- [ ] 5.4 Extend `tests/attach.rs`, and verify with tests that Ctrl+A then `d` prints `[detached]` with status 0 and a reattach shows `sleep 100` still running, that `exit` prints `[exited]` with status 0, that killing the server prints `[lost server]` with status 1, that `gband attach` with `GBAND` set exits 1, and that a client's terminal resize makes `tput cols` print the new width
- [ ] 5.5 Remove `examples/spike.rs`, and remove the `tui-term`, `ratatui` and `crossterm` dev-dependencies from the root `Cargo.toml`, plus `gband-core` if no root test uses it. Verify that `cargo build --all-targets` succeeds and that `cargo tree -p gband -e dev --depth 1` no longer lists the removed crates

## 6. Finish

- [ ] 6.1 Run `.claude/flow-gate` and verify that formatting, clippy with `-D warnings` and the workspace tests all pass
- [ ] 6.2 Check every new or changed Rust file against the repository's coding rules, which forbid narrative comments, and verify that no comment remains except one explaining an upstream API restriction
- [ ] 6.3 Run `gband attach` in a real terminal, run `ls` and start `sleep 100`, close the terminal window, open a new one, run `gband attach`, and verify that the `ls` output and the running `sleep` are there exactly as left
