## Why

gband 0.3.2 fails to compile on macOS, so a Mac user can neither run gband locally nor attach from a Mac to a Linux server with `-r`. Three calls exist only on Linux: `rustix::net::sockopt::socket_peercred`, `OFlags::PATH`, and every read under `/proc`, which macOS does not have. A fourth difference compiles and misplaces files: the `directories` crate ignores `XDG_STATE_HOME` on macOS, so logs land in `~/Library/Application Support/gband/log`.

## What Changes

- gband builds and runs on macOS with the same features as on Linux. The README names the macOS version and processor the change was tested on.
- The client's check of the user serving a socket, and the test channel's check of its listener, read the peer's uid with `getpeereid` on macOS and `SO_PEERCRED` on Linux.
- The server creates its socket with mode `0600` from the moment it appears on macOS too, without `/proc/self/fd` and without changing the file mode creation mask.
- On macOS every subcommand that addresses a server refuses a socket path over 103 bytes, the limit of the macOS socket address, or in a directory whose path is over 80 bytes, the room the server's private bind directory leaves. Linux keeps its 107-byte limit and no directory limit.
- The foreground command, the agent detection and the debug-build server replacement read a process's arguments, command name and executable path from the macOS kernel with `sysctl` `KERN_PROCARGS2` and `libproc`.
- The log directory follows `XDG_STATE_HOME` and `~/.local/state` on macOS as on Linux, as the configuration directory already follows `XDG_CONFIG_HOME` and `~/.config`.
- The executable identity needs no change: its existing `current_exe` fallback returns the same file on macOS at start.
- The test harness and the root tests read process state, children, sessions and environments through the same per-platform functions, so `cargo test --workspace` runs on macOS.
- Linux keeps its behaviour and its code: each Linux body moves unchanged behind `#[cfg(target_os = "linux")]`, a build for any other operating system stops with `compile_error!`, and the Linux gate runs after every task group that changes code.
- The README names macOS as supported, names the Xcode Command Line Tools as the C compiler, and says that macOS binds Ctrl+Space to switching the input source, which hides gband's prefix key until the shortcut is turned off.

## Capabilities

### New Capabilities
- `platforms`: the operating systems gband supports, and the per-platform sources of peer credentials and process information.

### Modified Capabilities
- `command-line`: "Server selection" states the socket path limits per platform: 107 bytes on Linux, and on macOS 103 bytes with a directory path of at most 80 bytes.
- `session-server`: "Socket location" takes its length limits from "Server selection", and gains macOS scenarios for them.
- `logging`: "Log directory location" applies the XDG state directory on macOS too, and drops the clause for a platform with no state directory.
- `window-names`: "Foreground command" reads the leader's arguments and command name on macOS from the kernel, and keeps the fallback to the started command.
- `agent-state`: "Agent windows" reads the leader's arguments on macOS from the kernel.

## Impact

- A new crate, `crates/os` (`gband-os`), holds every per-platform call: peer uid, process arguments, command name, executable path, environment, state and children. It depends on `libc` and `rustix`, and refuses to build for any operating system but Linux and macOS.
- `crates/server/src/lib.rs`: socket binding without `/proc` on macOS, a fixed-width private directory name, and the per-platform limits. `crates/server/src/process.rs`: process reads through `gband-os`.
- `src/main.rs`: the per-platform socket path limits for every subcommand.
- `src/logging.rs`: the log directory from `XDG_STATE_HOME` and the home directory. `directories` becomes a Linux-only dev-dependency that pins the Linux answer.
- `crates/client/src/transport.rs` and `src/channel.rs`: peer uid and server executable path through `gband-os`.
- `crates/harness/src/env.rs`, `crates/harness/src/terminal.rs`, `crates/scratch/src/lib.rs`, `tests/common/mod.rs`, `tests/plugin_testing.rs` and `tests/subcommands.rs`: process reads through `gband-os`, short scratch paths on macOS, and per-platform limits.
- `README.md`: supported platforms, macOS install, and the Ctrl+Space and Option key notes.
- No CI exists. The implementer runs `/implement` and `/ready` on a Mac, so `.claude/flow-gate` proves macOS. The tasks run the Linux gate in arm64 and amd64 Linux containers on that Mac. `/integrate` runs no gate, so the integrator runs `.claude/flow-gate` on x86-64 Linux against `origin/change/macos-support` before `/integrate`.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- Cargo.toml
- Cargo.lock
- crates/os/
- crates/client/Cargo.toml
- crates/client/src/transport.rs
- crates/server/Cargo.toml
- crates/server/src/lib.rs
- crates/server/src/process.rs
- crates/harness/Cargo.toml
- crates/harness/src/env.rs
- crates/harness/src/terminal.rs
- crates/scratch/src/lib.rs
- src/channel.rs
- src/logging.rs
- src/main.rs
- tests/common/mod.rs
- tests/plugin_testing.rs
- tests/subcommands.rs
- README.md
- openspec/changes/macos-support/
