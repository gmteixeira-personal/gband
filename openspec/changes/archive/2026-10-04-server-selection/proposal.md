## Why

Every gband server listens on the same `default.sock`, so a user has at most one server. Each change worktree builds its own `target/debug/gband`, and a debug client replaces any server whose executable identity differs from its own. Two worktrees therefore stop each other's servers, and their panes, on every attach. A worktree needs a server of its own that it starts, uses and stops without touching any other.

## What Changes

- Two global options select the server a command addresses: `-S <name>` (`--server`) for a named server in the runtime directory, and `-p <path>` (`--socket`) for an explicit socket path. They are mutually exclusive and apply to `server`, `attach` and `kill-server`.
- A server name is 1 to 64 bytes of ASCII letters, digits, `_` and `-`, the same rule as session names in `multi-session`. `-S <name>` listens on `<runtime dir>/<name>.sock` and guards it with `<name>.lock`. `-S default` and no option keep today's paths.
- Without an option, a command inside a gband pane addresses the server of that pane, which `GBAND` names. Outside a pane it addresses `default`.
- The lock and pid record of a server derive from its socket path, so `-S x` and `-p <runtime dir>/x.sock` share one guard.
- The server creates its socket with mode `0600`. Before it sends anything, the client checks that the process listening on the socket runs as the same user. A socket path longer than 107 bytes is refused.
- `gband attach` starts a missing server, and stops a replaced one, at the socket it selected.
- **BREAKING** for the nesting guard: `gband attach` inside a pane is refused only when it addresses the pane's own server. Attaching to another server from inside a pane is allowed.
- A debug client replaces a server from another build only when that server runs from the same executable path, which means a rebuild of the same binary. A server from any other executable is treated as a release build treats it. The notes that tell the user to run `gband kill-server` repeat the selection option the client was given.
- Every log event records the socket of the server it concerns, because all servers share one log file series.
- End-to-end tests isolate their scratch directories per worktree, so concurrent gates do not delete each other's directories.

Not in this change: `-s` stays reserved for `multi-session`. Environment variables that select a server are not added.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `command-line`: adds the server selection options and the order in which a command resolves its server.
- `session-server`: the socket location, ownership checks, single-server guard, pid record and `kill-server` follow the selected server, and the socket is created private.
- `client-attach`: connecting, starting a server, the nesting guard, the version note and replacing a server from another build follow the selected server. The client checks the server's user, and replaces only a rebuild of its own executable.
- `logging`: events record the socket of the server they concern.

## Impact

- Code:
  - `src/main.rs` parses the options and resolves the socket path. `src/paths.rs` validates names and derives the lock path.
  - `crates/protocol/src/lib.rs`: socket and lock paths come from the selection, not from fixed names.
  - `crates/server/src/lib.rs` and `crates/server/src/lock.rs`: the server takes its socket path and lock path, and binds with mode `0600`.
  - `crates/client/src/connect.rs`: peer user check, start and stop with `-p`, and the same-executable rule for replacement.
- Tests: new end-to-end tests in `tests/servers.rs`. `tests/common/mod.rs` and `tests/subcommands.rs` scope their scratch directories to the worktree.
- Interaction with `foundation-seams`: it moves starting and replacing a server into `UnixTransport` and takes the runtime directory out of `ClientConfig`. Whichever change lands second carries the socket path through the other's structure.
- Interaction with `multi-session`: it shares the name rule, and it spawns `server -s <name>`. When it lands second, it passes `-p` on as this change does.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- Cargo.toml
- src/main.rs
- src/paths.rs
- crates/protocol/src/lib.rs
- crates/server/src/lib.rs
- crates/server/src/lock.rs
- crates/server/src/pane.rs
- crates/server/src/session.rs
- crates/server/tests/common/mod.rs
- crates/server/tests/lifecycle.rs
- crates/client/src/lib.rs
- crates/client/src/connect.rs
- crates/client/tests/connect.rs
- tests/common/mod.rs
- tests/subcommands.rs
- tests/attach.rs
- tests/kill_server.rs
- tests/servers.rs
- openspec/changes/server-selection/
