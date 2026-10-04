## 1. Paths and selection

- [x] 1.1 Add `lock_path(socket: &Path) -> PathBuf` to `crates/protocol/src/lib.rs`, replacing a final `.sock` with `.lock` and otherwise appending `.lock`. Verify with unit tests that `default.sock` gives `default.lock`, `x.sock` gives `x.lock`, `/tmp/s` gives `/tmp/s.lock` and `/tmp/s.socket` gives `/tmp/s.socket.lock`
- [x] 1.2 Add `ServerName` to `src/paths.rs` with `FromStr` enforcing 1 to 64 bytes of `[A-Za-z0-9_-]`, and a `resolve_socket(socket, server, gband, runtime_dir)` function that applies the command-line spec's order and resolves a relative path against the working directory. Verify with unit tests that `feature`, `a_b-1` and a 64-byte name parse, that an empty name, a 65-byte name, `a.b`, `a/b` and `..` are refused, and that each rule of the order wins over the ones below it, including an empty `GBAND`
- [x] 1.3 Add `-S`/`--server` and `-p`/`--socket` to the CLI in `src/main.rs` as global options in one group, and refuse a resolved path over 107 bytes with exit 1. Verify with tests in `tests/subcommands.rs` that both options exit 2 with usage, that `-S a.b` exits 2 naming `a.b`, that a 120-byte `-p` exits 1 naming the limit, and that `--help` lists both options

## 2. Server on a socket path

- [x] 2.1 Replace `runtime_dir` in `ServerConfig` with `socket`, take the lock from `lock_path(socket)` in `crates/server/src/lock.rs`, and make `gband_server::kill` take the socket path. In `src/main.rs`, prepare the runtime directory only when it is the socket's parent. Update `crates/server/tests/` callers. Verify with `cargo test -p gband-server`, and with a test that a server on `-p <dir>/s` writes its pid to `<dir>/s.lock`
- [x] 2.2 Bind the listener under `umask(0o177)` in `crates/server/src/lib.rs`, restoring the previous mask, and enable rustix's `fs` and `net` features in `Cargo.toml`. Verify with a test that the socket file's mode is `0600`
- [x] 2.3 Enter a `tracing` span carrying the socket path in `src/main.rs` right after resolution. Verify with an end-to-end test that the server log lines of two servers named `a` and `b` each name their own socket, and that a `-S a kill-server` run's client log lines name `a.sock`

## 3. Client on a socket path

- [x] 3.1 Replace `runtime_dir` in `ClientConfig` with `socket`, and make `start_server` and `stop_server` in `crates/client/src/connect.rs` pass `-p <socket>`. Verify with `cargo test -p gband-client`, and with an end-to-end test that `gband -S feature attach` starts a server whose pane prints the `feature.sock` path for `echo $GBAND`
- [x] 3.2 Check `SO_PEERCRED` after every connect and refuse a socket whose peer uid differs. Verify with a unit test that the check passes on a socket pair of the same process and that a forced mismatch yields the error naming the socket and uid
- [x] 3.3 Narrow the nesting guard in `src/main.rs` to `GBAND` equal to the resolved socket. Verify with end-to-end tests that a bare `gband attach` with `GBAND` set to the default socket exits 1, that `-S feature` with `GBAND=<runtime>/feature.sock` exits 1, and that `-S feature` with `GBAND` set to the default socket attaches
- [x] 3.4 Before a debug replacement, read the pid from the lock file, read `/proc/<pid>/exe` with a trailing ` (deleted)` stripped, and replace only when it equals `current_exe()`. Otherwise mark the server stale as a release build does. Verify with the existing rebuild test in `tests/attach.rs`, and with a new test that a debug client run from a copy of the binary at another path attaches to the original's server, leaves its shell running and prints the different-build note after `[detached]`
- [x] 3.5 Build the different-build note and the version-mismatch message from the selection the user typed. Verify with an end-to-end test that `-S feature` produces `gband -S feature kill-server` in the note

## 4. Isolation

- [x] 4.1 Add an 8-hex-digit hash of `CARGO_MANIFEST_DIR` to the scratch roots in `tests/common/mod.rs` and `tests/subcommands.rs`. Verify that `cargo test --workspace` passes and that the roots under `/tmp` carry the hash
- [x] 4.2 Verify with an end-to-end test in `tests/servers.rs` that servers named `a` and `b` run side by side, that `gband -S a kill-server` leaves `b` and its shell running, and that a second `gband server -p <runtime>/b.sock` is refused while `b` runs
- [x] 4.3 Run `.claude/flow-gate` and confirm that it passes
