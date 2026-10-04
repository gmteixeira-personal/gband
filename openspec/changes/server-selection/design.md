## Context

The server and the client take a runtime directory and join fixed names to it: `gband_protocol::socket_path` appends `default.sock`, and `gband_server::lock` appends `default.lock`. `src/paths.rs` resolves and checks that directory. The server already refuses a socket path longer than `SUN_PATH_MAX`. The client starts a missing server with `<exe> server` and stops a replaced one with `<exe> kill-server`, so both child processes resolve their socket again from the environment. Debug clients replace any server whose `ExecutableId` differs from their own.

The end-to-end harnesses use fixed scratch roots, `/tmp/gband-e2e-<name>` in `tests/common/mod.rs` and `/tmp/gband-subcommands-<name>` in `tests/subcommands.rs`, and delete them on start. The crate tests already use `CARGO_TARGET_TMPDIR`, which is per worktree.

## Goals / Non-Goals

**Goals:**
- Any number of servers per user, each addressed by a name or a socket path.
- A worktree can start, use and stop its own server with `-S <name>`, without affecting any other server.
- A debug client never stops a server built from another executable path.
- Concurrent gates in different worktrees do not interfere.

**Non-Goals:**
- `-s` and several sessions per server. `multi-session` owns them, and this change leaves `-s` unparsed.
- Environment variables that select a server, such as `GBAND_SERVER`. Only `GBAND`, which the server already sets in its panes, takes part in resolution.
- Listing running servers.
- Moving the default socket. No option still means `default.sock` and `default.lock` in the runtime directory.

## Decisions

### One resolved socket path, passed down

`src/main.rs` resolves a `PathBuf` once, in the order the command-line spec gives, and passes it to the server, the client and `kill`. `ServerConfig` and `ClientConfig` replace `runtime_dir` with `socket: PathBuf`. `gband_server::kill` takes the socket path. `socket_path(runtime_dir)` stays for the default and for tests, and a new `lock_path(socket)` in `gband-protocol` derives the lock: `with_extension("lock")` when the extension is `sock`, otherwise the path with `.lock` appended.

Alternative: keep passing a runtime directory and give each named server a subdirectory, `<runtime>/<name>/default.sock`. Rejected, because `-p` has no directory of its own, and two rules would let `-S x` and `-p <runtime>/x.sock` hold different locks on one socket.

### Children get `-p`

`start_server` and `stop_server` run `<exe> -p <socket> server` and `<exe> -p <socket> kill-server`. The child then reaches exactly the socket the client chose, whatever the option or fallback that chose it.

### `GBAND` as the fallback

A pane's `GBAND` already holds its server's socket path. Using it when no option is given makes `kill-server` inside a pane stop that pane's server, as tmux does with `TMUX`. The nesting guard compares the resolved path with `GBAND`, so a bare `gband attach` in a pane is refused as before, and `gband -S other attach` is allowed.

### Names and paths are checked by clap

A `ServerName` newtype in `src/paths.rs` implements `FromStr` with the 1 to 64 byte `[A-Za-z0-9_-]` rule, used as clap's `value_parser`. `-S` and `-p` are `global = true` and in one `ArgGroup` that allows at most one. A bad name or both options exit 2 with clap's usage. The 107-byte limit is checked after resolution, because it depends on `XDG_RUNTIME_DIR`. It exits 1, and the server's existing check stays as the last guard.

### The runtime directory is prepared only when it holds the socket

`paths::prepare` runs when the socket's parent is the runtime directory. A `-p` parent is never created. A missing one fails on bind with the directory named.

### Private socket, checked peer

The server binds under `umask(0o177)` and restores the previous mask, so the socket is `0600` from the moment it exists. It does this before it starts the pane threads. `chmod` after bind was rejected because a connection could arrive in between. After connecting, the client reads `SO_PEERCRED` through `rustix::net::sockopt::socket_peercred` and compares the uid with `getuid()`. This matters for `-p` in a shared directory such as `/tmp`, where another user could have created the socket first.

### Same-path replacement without a protocol change

The client reads the server's pid from the lock file, as `kill` already does, and reads `/proc/<pid>/exe`. A rebuilt executable reads back as `<path> (deleted)`, so the suffix is stripped before the path is compared with the client's `current_exe()`. Holding the lock proves the process is alive, so the pid has not been reused. An unreadable link means no replacement.

Alternative: add the executable path to the server's `Info`. Rejected, because it bumps the wire protocol for a debug-only rule, and `multi-session` and `foundation-seams` both change the handshake.

### The note repeats the selection

The `STALE_SERVER_NOTE` and the version-mismatch message are built from the selection the user typed. `-S x` becomes `gband -S x kill-server`, and `-p p` becomes `gband -p p kill-server`. When the path came from a fallback, the text stays `gband kill-server`.

### Log events carry the socket

`main` enters a `tracing` span with a `socket` field right after resolution and holds it for the command's life. Every event in the `fmt` layer then shows the socket without changes at each call site. The spawned server opens its own span the same way.

### Test scratch roots per worktree

Both harnesses add an 8-hex-digit hash of `env!("CARGO_MANIFEST_DIR")` to their root, for example `/tmp/gband-e2e-<hash>-<name>`. The roots stay short, which matters because sockets live under them. They also stay stable within a worktree, so stale directories are still reused and removed. `CARGO_TARGET_TMPDIR` was rejected, because a worktree path plus `target/tmp/` plus the test name pushes the socket path past 107 bytes.

## Risks / Trade-offs

- [`/proc/<pid>/exe` is Linux-only] → On another platform the read fails, and a debug client then behaves like a release one. That is safe, but it never replaces a server.
- [A forgotten `-S` lands on `default`] → The same-path rule keeps a debug client from stopping a server from another path. The client attaches and prints the different-build note instead.
- [Two `-p` paths that differ only after the final dot share one lock, for example `/tmp/a` and `/tmp/a.sock`] → The second server is refused with the socket path named. Nothing is corrupted.
- [A server left running after its worktree is deleted] → The user stops it with `gband -S <name> kill-server`. Listing servers is a non-goal for now.
- [`foundation-seams` moves start and replace into `UnixTransport`, and `multi-session` rewrites the spawn] → Each touches the same functions. Whichever lands second carries `socket: PathBuf` through, and passes `-p` to children.
