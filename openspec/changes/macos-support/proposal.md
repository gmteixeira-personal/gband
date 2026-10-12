## Why

gband 0.3.2 fails to compile on macOS, so a Mac user can neither run gband locally nor attach from a Mac to a Linux server with `-r`. Three calls exist only on Linux: `rustix::net::sockopt::socket_peercred`, `OFlags::PATH`, and every read under `/proc`, which macOS does not have.

## What Changes

- gband builds and runs on macOS, on Apple silicon and Intel, with the same features as on Linux.
- The client's check of the user serving a socket, and the test channel's check of its listener, read the peer's uid with `getpeereid` on macOS and `SO_PEERCRED` on Linux.
- The server creates its socket with mode `0600` from the moment it appears on macOS too, without `/proc/self/fd` and without changing the file mode creation mask.
- An explicit socket path on macOS may hold at most 103 bytes, the limit of the macOS socket address, less the bytes the private bind directory adds. A longer path is refused with one line naming the limit.
- The foreground command, the agent detection and the debug-build server replacement read a process's arguments, command name and executable path from the macOS kernel with `sysctl` `KERN_PROCARGS2` and `libproc`.
- The executable identity on macOS is taken from the executable file when the process starts, as on Linux.
- The test harness and the root tests read process state, children, sessions and environments through the same per-platform functions, so `cargo test --workspace` runs on macOS.
- The README names macOS as supported, names the Xcode Command Line Tools as the C compiler, and says that macOS binds Ctrl+Space to switching the input source, which hides gband's prefix key until the shortcut is turned off.

## Capabilities

### New Capabilities
- `platforms`: the operating systems gband supports, and the per-platform sources of peer credentials and process information.

### Modified Capabilities
- `session-server`: "Socket location" states the longest explicit socket path per platform: 107 bytes on Linux, and on macOS 103 bytes less the private bind directory's share.
- `window-names`: "Foreground command" reads the leader's arguments and command name on macOS from the kernel, and keeps the fallback to the started command.
- `agent-state`: "Agent windows" reads the leader's arguments on macOS from the kernel.

## Impact

- A new crate, `crates/os` (`gband-os`), holds every per-platform call: peer uid, process arguments, command name, executable path, environment, state and children. It depends on `libc` and `rustix`.
- `crates/server/src/lib.rs`: socket binding without `/proc` on macOS. `crates/server/src/process.rs`: process reads through `gband-os`.
- `crates/client/src/transport.rs` and `src/channel.rs`: peer uid and server executable path through `gband-os`.
- `src/executable.rs`: identity from the open executable at start on macOS.
- `crates/harness/src/env.rs`, `crates/harness/src/terminal.rs`, `tests/common/mod.rs` and `tests/plugin_testing.rs`: process reads through `gband-os`.
- `README.md`: supported platforms, macOS install and the Ctrl+Space note.
- No CI exists. The implementer runs `/implement` and `/ready` on a Mac, so `.claude/flow-gate` proves macOS. `/integrate` runs no gate, so the tasks also run `.claude/flow-gate` in a Linux container on the Mac before `/ready`.

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
- src/executable.rs
- tests/common/mod.rs
- tests/plugin_testing.rs
- tests/subcommands.rs
- README.md
- openspec/changes/macos-support/
