## Context

The proposal gives the motivation. `gband server` and `gband attach` are stubs that log and exit (`src/main.rs`). Of the four library crates, only `gband-core` and `gband-client` hold code. `gband-core` has the key encoder (`encode_key`, `encode_paste`, `Modes`), and `gband-client` translates `crossterm` key events (`key_from_event`). `examples/spike.rs` shows the working single-process loop: a PTY reader thread feeds a `vt100::Parser`, `tui-term` draws it, and input passes through the encoder.

Two `vt100` 0.16.2 facts carry this design. `Screen` is `Clone`. `Screen::state_formatted()` and `Screen::state_diff(&prev)` produce bytes that, fed into a parser, reproduce the visible contents, the cursor and the input modes, either from scratch or from `prev`. A diff assumes both screens have the same size.

The workspace already declares every crate this design uses: `tokio` (`full`), `postcard`, `serde`, `portable-pty`, `vt100`, `tui-term`, `ratatui`, `crossterm` and `rustix` (`process`, `termios`). `rust-version` is 1.89, which provides `std::fs::File::try_lock`.

## Goals / Non-Goals

**Goals:**
- One pane served end to end over a local Unix socket, with attach, detach and reattach.
- A wire format whose handshake later versions can still recognise.
- Every behaviour in the specs covered by an automated test, including the kill-and-reattach acceptance criterion.

**Non-Goals:**
- Several sessions per user. The socket is always `default.sock`, and naming sessions comes later.
- Acknowledged sync. Over a local stream socket, the last frame written is the state the client will reach, so the server diffs against what it last *sent*. Acknowledgements arrive with the SSH transport and reconnection.
- Signal handling in the server beyond SIGTERM. A server killed by any other signal leaves a stale socket, which the next server replaces.
- Restoring a terminal left raw by a client killed with SIGKILL. No process remains to restore it, and the user runs `reset`.

## Decisions

### Crate responsibilities

| crate | owns | I/O |
|---|---|---|
| `gband-protocol` | message types, the hello pair, frame encoder and incremental decoder, `PROTOCOL_VERSION` | none |
| `gband-server` | `run(ServerConfig)`: lock and pid record, socket, PTY, grid, client tasks, SIGTERM. `kill(runtime_dir)`: the stop procedure behind `kill-server` | tokio, blocking threads for the PTY |
| `gband-client` | `run(ClientConfig) -> Outcome`: connect, auto-start, handshake, build comparison and replacement, render, input, prefix key | tokio, one blocking thread for `crossterm` events |
| `gband` (root) | CLI, logging, `paths::runtime_dir()`, `executable::identity()`, the nesting and terminal checks, mapping `Outcome` to the printed lines and exit status | — |

`ServerConfig` carries the socket path, the program's argv, its working directory and the executable identity to report. The root package resolves these from the environment, so the server's tests can pass `/bin/sh` and an arbitrary identity without changing process-wide environment variables, which is racy across test threads. `ClientConfig` carries the runtime directory, the executable used to start a server, the client's own executable identity, and whether a mismatch replaces the server. The root sets that last flag from `cfg!(debug_assertions)`, so tests of `gband-client` can exercise both behaviours from one build.

The lock-file format has one owner. `gband-server` writes the pid record and provides `kill`, and `gband kill-server` calls it. The client stops a server by running its own executable with `kill-server` and capturing that command's stderr. It already runs its own executable to start a server, so it needs no dependency on `gband-server`. Alternative: `gband-client` depends on `gband-server` for `kill`. Rejected, because that would put server code in the client for one function.

Alternative: put the path logic in `gband-protocol`. Rejected because that crate has no I/O and no `directories` dependency, and only the binary needs the path.

### Runtime directory and single instance

`paths::runtime_dir()` returns `$XDG_RUNTIME_DIR/gband` when that variable is set to an absolute path. Otherwise it returns `/tmp/gband-<uid>`, with the uid from `rustix::process::getuid`. `rustix` is added to the root package for this. The server creates the directory with `DirBuilder::mode(0o700)`, then checks that its owner is the user and that `mode & 0o077 == 0`. The check runs even when the directory already existed, because only the directory's permissions protect the socket.

The single-instance guard is an exclusive `File::try_lock` on `default.lock`, held for the server's whole life. Only the lock holder removes an existing `default.sock` and binds a new one. A connect probe ("is anyone listening?") was rejected: two `gband attach` invocations racing to start a server would both see no listener and both bind. With the lock, the loser exits 1 and both clients connect to the winner. Abstract-namespace sockets were rejected because they are Linux-only and cannot be protected by file permissions.

After taking the lock, the server truncates `default.lock` and writes its pid. `kill` opens `default.lock` without creating it or the directory. If `try_lock` succeeds, no server holds it, so `kill` releases it and reports that no server is running. If the lock is held, `kill` reads the pid, retrying for up to 1 s while the file is still empty, because a server that has just taken the lock may not have written it yet. It then sends SIGTERM with `rustix::process::kill_process` and polls `try_lock` every 25 ms for up to 5 s. The lock being held proves that the recorded process is alive, so the pid cannot have been reused when the signal is sent. Polling the lock rather than the socket file waits until the process has actually gone.

The server spawns the shell **before** binding. A program that cannot be spawned then fails the server before any client can connect, and the client detects that the child it started has exited, so it fails fast instead of waiting 5 seconds. A socket path longer than the 108-byte `sun_path` limit is reported with the path rather than as a bare `EINVAL`.

### Server structure

```
PTY reader thread ──bytes──▶ Mutex<vt100::Parser> ──bump──▶ watch<u64> (generation)
child-wait thread ──status──▶ watch<Option<ExitStatus>>
input task ◀──mpsc<Input>── client tasks        (encodes under the parser lock, writes the PTY)
accept loop ──▶ one task per client: handshake → snapshot → select { generation changed, exit, frame read }
```

- The `portable-pty` reader and writer are blocking, so reading and child-waiting use `std::thread`. The parser is behind a `std::sync::Mutex` that is never held across an `.await`.
- **Coalescing comes from `watch`.** A client task wakes on a generation change and locks the parser. It clones the `Screen` and releases the lock, then computes the update against the `Screen` it last sent and writes the frame. While that write is pending, later changes collapse into one generation bump. A slow client therefore receives fewer, larger updates and converges on the current screen, which is the mosh-style behaviour the spec requires. The alternative, a broadcast of raw PTY bytes, was rejected: a slow client would need an unbounded backlog, and a reattaching client would need the whole history.
- **A size change becomes a snapshot.** When the current screen's size differs from the last-sent screen's size, the task sends a snapshot instead of an update, because `state_diff` assumes equal sizes. No separate resize signal is needed.
- **Keys are encoded on the server.** Client tasks forward `Key` and `Paste` messages to one input task. That task locks the parser, reads `application_cursor()` and `bracketed_paste()`, encodes, releases the lock and writes. The modes are therefore those of the authoritative screen when the bytes reach the PTY. A client's copy of the screen lags by at least one frame, so encoding on the client could use stale modes just after a program switches DECCKM. One input task also gives a single order across clients.
- **Resize** applies `MasterPty::resize` and `Screen::set_size` together under the parser lock. The size is recorded per client, and the most recent hello or resize from any client wins.
- **SIGTERM** is caught with `tokio::signal::unix`. The handler sends SIGHUP to the program through `ChildKiller`, then SIGKILL after 2 s if the exit watch has not been set. Termination then follows the ordinary exit path below, so clients see `Exited` and the socket is removed exactly as when the shell exits by itself.
- **Exit** comes from the child-wait thread, which sets the exit watch. Each client task sends a last update, then `Exited`, then closes. The server waits up to one second for those tasks, removes `default.sock`, logs the status and returns `Ok`. The main loop exits 0. `default.lock` stays in place, which is harmless because the lock, not the file, is the guard.
- The spike's `LoggingCallbacks` moves into `gband-server` unchanged. The unhandled-sequence log is a property of the authoritative grid.

### Wire format

The hello pair are standalone structs, not variants of the message enums:

```rust
struct Hello { version: u32, cols: u16, rows: u16 }
enum HelloReply { Accepted { version: u32 }, Rejected { version: u32 } }
enum ClientMessage { Key(Key), Paste(String), Resize { cols: u16, rows: u16 }, Detach }
enum ServerMessage { Info { pid: u32, executable: ExecutableId }, Snapshot { cols: u16, rows: u16, contents: Vec<u8> }, Update(Vec<u8>), Exited }
struct ExecutableId { device: u64, inode: u64 }
```

`Info` is an ordinary message rather than part of `HelloReply`, so it can grow with later protocol versions. A version-mismatched server is detected by the hello alone and replaced without reading `Info`, because `kill` needs only the lock file.

`postcard` is not self-describing, so adding a variant to an enum changes what older peers can decode. Keeping the hello pair out of the enums keeps their encoding frozen. A test pins the exact bytes of `Hello { version: 1, .. }` and both `HelloReply` variants. Every later protocol change bumps `PROTOCOL_VERSION`, and the handshake rejects mismatches. This change requires an exact version match. The summary's "additive messages" goal needs a self-describing format or explicit capability flags, and that waits until a second version exists to be compatible with.

### Executable identity

`executable::identity()` stats `/proc/self/exe` once at startup and returns its `st_dev` and `st_ino`. The `/proc` link resolves to the inode the process was started from, even after that file is replaced on disk. Cargo writes every relinked binary as a new file, so a rebuild always yields a new inode, and a no-op build leaves the inode alone. On a platform without `/proc`, it falls back to the metadata of `std::env::current_exe()`, which is correct as long as the file has not yet been replaced when the process starts.

Alternatives: a build id from a build script was rejected because the root package's build script does not rerun when only `crates/server` changes, so it would miss exactly the rebuilds that matter. Hashing the 40 MB executable was rejected as needless startup cost when the inode already answers the question. Comparing `CARGO_PKG_VERSION` was rejected because it does not change between development builds.

`Key`, `KeyCode` and `Modifiers` in `gband-core` gain `Serialize` and `Deserialize`. `gband-core` already depends on `serde` with `derive`.

Framing is a 4-byte big-endian length and a `postcard` payload of at most 16 MiB. The protocol crate's `Decoder` accepts arbitrary byte chunks and yields whole messages. It rejects an oversized length as soon as the header is complete, before buffering any payload. Server and client both read socket chunks into it, so the split-frame behaviour is tested once, without I/O.

### Client structure

1. The root package checks `GBAND` (nesting), then `isatty` on stdin and stdout. Neither check connects to anything.
2. Connect to `default.sock`. On `NotFound` or `ConnectionRefused`, spawn `current_exe() server` with null stdio and a `pre_exec` that calls `rustix::process::setsid()`. The server then has no controlling terminal and gets no SIGHUP when the client's window closes. Retry the connect every 25 ms for up to 5 s, and stop early when `try_wait` reports the child has exited. `rustix` is added to `gband-client`'s dependencies.
3. Send `Hello` with `crossterm::terminal::size()`, and read `HelloReply` and then `Info`, **before** `ratatui::init()`. A rejection or a failed replacement is then reported on an untouched terminal.
4. When `HelloReply` is `Rejected`, or `Info.executable` differs from the client's own identity, and the replace flag is set and no replacement has happened yet in this invocation: send `Detach` if accepted, close, run `<executable> kill-server`, and go back to step 2, which finds no server and starts one. Otherwise a rejection ends the client with the version error, and a differing identity sets the `stale_server` flag that `main` reads after the terminal is restored.
5. `ratatui::init()`, which enters raw mode and the alternate screen and installs a panic hook that restores both. Then enable bracketed paste. A guard value undoes both on every return path.
6. The main loop selects over server frames and input events. Input events come from a blocking thread around `crossterm::event::read`, because the `event-stream` feature is not enabled and is not needed. A snapshot replaces the parser with `vt100::Parser::new(rows, cols, 0)` and processes `contents`. An update processes its bytes. The client draws after each batch, with `PseudoTerminal` as the spike does, and places the real cursor at the pane cursor unless the cursor is hidden or outside the terminal.
7. The prefix key is a two-state machine (`Normal`, `AfterPrefix`) in its own module, tested without a terminal. It runs before `key_from_event`'s output reaches the socket.
8. `Outcome::{Detached, Exited, LostServer}` returns to `main` with the `stale_server` flag. After the terminal is restored, `main` prints `[detached]`, `[exited]` or `[lost server]` to stdout, then the different-build note to stderr when the flag is set, and maps the outcome to its exit status.

### Removing the spike

`gband attach` covers everything the spike did. The root package keeps `portable-pty` and `vt100` as dev-dependencies, because the end-to-end tests run `gband attach` inside a PTY and read its screen through a `vt100` parser. The test harness is the spike turned around. `tui-term`, `ratatui` and `crossterm` leave the root's dev-dependencies, and `gband-core` leaves them too if no root test still uses it.

### Testing

- `gband-protocol` unit tests: round trip of every message, every split point of a frame, oversized header, garbage payload, and the pinned hello bytes.
- `gband-server` integration tests (`crates/server/tests/`) run `run(ServerConfig)` in-process on a socket under `CARGO_TARGET_TMPDIR`, with `/bin/sh` as the program, and speak the protocol directly. They cover `Info` carrying the configured pid and identity before the snapshot, the pid record in `default.lock`, `kill` with no server, the snapshot, live updates, slow-client convergence, two clients typing, resize-to-snapshot, the latest-client size rule, version rejection, a non-hello first frame, an oversized frame, the single-instance guard, stale-socket replacement, directory permission refusal, draining output while detached, the `Exited` message and socket removal.
- Root end-to-end tests (`tests/attach.rs`) run the built `gband attach` in a `portable-pty` with `XDG_RUNTIME_DIR` and `XDG_STATE_HOME` pointing at a temporary directory and `SHELL=/bin/sh`. They cover first attach starting a server, kill-and-reattach showing the same screen and shell pid, detach and reattach, `[exited]`, `[lost server]`, the prefix key, nesting refusal, the no-terminal refusal the server-cannot-start timeout, and build mismatch. The mismatch tests copy the built binary to a temporary path, so the copy has a new inode. They start the server from the copy, then attach with the original. They check that a debug-build client replaces the server, with a different shell pid, and that a client with the replace flag off attaches to the old shell and prints the note after `[detached]`. The replace flag comes from `cfg!(debug_assertions)`, so the second case cannot run end to end in a debug test build. Steps 2 to 4 are therefore a separate `connect(&ClientConfig)` that returns the open connection, its `Info` and the `stale_server` flag without touching the terminal. A `gband-client` integration test, with `gband-server` as a dev-dependency only, runs a server in-process under a fake identity and checks `connect` with the flag off.
- `tests/kill_server.rs` runs `gband server` as a subprocess with `SHELL=/bin/sh`, then `gband kill-server`. It checks exit status 0, that the shell pid is gone, that the socket file is removed, and that a second `gband kill-server` exits 1 without creating a missing runtime directory. A pane running `trap '' HUP; sleep 100` checks the SIGKILL fallback. Every test reads the shell's pid with `echo $$` and kills that shell in a drop guard. The server exits with its program, so this also ends the server.
- `tests/subcommands.rs` runs `gband server` with `SHELL=/bin/true` and its own `XDG_RUNTIME_DIR`, and expects `gband attach` without a terminal to exit 1.

## Risks / Trade-offs

- [`state_formatted` does not carry everything a terminal holds: the window title, the alternate-screen flag and scrollback] → A client sees the active grid's cells, which is what it draws. Scrollback is out of scope. The title waits for chrome that shows it.
- [A full-screen redraw costs a full-size update for every client] → Updates are bounded by screen size, not output volume, and coalescing caps them at one in flight per client.
- [The prefix Ctrl+A shadows readline's beginning-of-line] → Ctrl+A twice sends it. A configurable prefix comes with the Lua keybinding work.
- [fish still waits 10 s for a DA1 reply at every shell start] → It is now paid once per session rather than once per attach. `terminal-query-replies` removes it.
- [A test that fails midway can leave a server running] → Each test's server lives in its own runtime directory, and a drop guard kills its shell, which ends the server.
- [A debug build replaces a mismatched server without asking, ending its shell and anything running in it] → That is the intent while developing. A release build never does it, and `--release` builds used during development get the note instead.
- [Two debug clients from different builds attaching in turn keep replacing each other's server] → Each invocation replaces at most once, and only when its own identity differs. Two builds in use at once is not a development flow worth protecting.
- [A copied or reinstalled identical binary has a new inode and reads as a different build] → It costs at most a note or, in a debug build, one restart.
- [The pane inherits the outer terminal's identity, such as `TERM_PROGRAM`, from whichever client started the server] → `pane-environment` follow-up, unchanged by this design.

## Migration Plan

None. Both subcommands were stubs with no users. Rollback is reverting the change.
