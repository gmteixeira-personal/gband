## Context

See proposal.md for the motivation. Every Linux-only call in the tree, after the pull of `eab6e3c`:

| site | call | role |
|---|---|---|
| `crates/client/src/transport.rs:128` | `socket_peercred` | client checks the user serving the socket |
| `src/channel.rs:18` | `socket_peercred` | test channel checks its listener |
| `crates/server/src/lib.rs:219` | `OFlags::PATH` | holds the private bind directory |
| `crates/server/src/lib.rs:222` | `/proc/self/fd/<fd>/s` | binds through the held directory |
| `crates/server/src/lib.rs:258` | `/proc/self/status` | test reads the umask |
| `crates/server/src/process.rs:19,30,36,61` | `/proc/<pid>/cmdline`, `comm` | foreground command, agent arguments, tests |
| `crates/client/src/transport.rs:145` | `/proc/<pid>/exe` | debug build compares the server's executable path |
| `src/executable.rs:8` | `/proc/self/exe` | executable identity, already falls back to `current_exe` |
| `crates/harness/src/env.rs:289,298` | `/proc/<pid>/stat`, `task/*/children` | zombie check, child list |
| `crates/harness/src/terminal.rs:212` | `/proc/<pid>/comm` | waits for a program under a shell |
| `tests/common/mod.rs:69` | `/proc/<pid>/stat` field 6 | session id |
| `tests/plugin_testing.rs:425,431` | `/proc`, `/proc/<pid>/environ` | finds processes a test run left behind |

The first three fail to compile on macOS. The rest compile and fail at run time. `rustix` 1.1.5 gates `socket_peercred` on `linux_kernel` and `OFlags::PATH` on Linux, FreeBSD, Fuchsia, Redox and Emscripten. `libc` 0.2.190, already in `Cargo.lock`, exposes `getpeereid`, `proc_pidpath`, `proc_pidinfo` with `PROC_PIDTBSDINFO`, `proc_listchildpids`, `proc_listallpids`, `proc_name` and `KERN_PROCARGS2` for Apple targets.

The macOS socket address holds 104 bytes of path, so 103 usable, against 108 and 107 on Linux. macOS has no `bindat`, and binding through a held directory needs `/proc/self/fd`.

## Goals / Non-Goals

**Goals:**
- One crate holds every call that differs per platform, so no other crate carries a `cfg(target_os)`.
- Linux behaviour and Linux code paths stay byte for byte what they are now.
- `.claude/flow-gate` passes on macOS and on Linux.

**Non-Goals:**
- FreeBSD, other BSDs and Windows.
- A CI workflow. No CI exists, and adding one is a separate decision.
- Changes to the wire protocol. A Mac client and a Linux server of the same version already speak the same bytes.

## Decisions

### A new crate `gband-os` holds the platform calls

`crates/os` exposes plain functions, each with a `#[cfg(target_os = "linux")]` body and a `#[cfg(target_os = "macos")]` body:

- `peer_uid(fd) -> io::Result<Uid>`: `socket_peercred` on Linux, `libc::getpeereid` on macOS.
- `arguments(pid) -> Option<Vec<Vec<u8>>>`: `/proc/<pid>/cmdline` on Linux. On macOS, `sysctl` with `CTL_KERN, KERN_PROCARGS2, pid` returns `argc` as an `i32`, the executable path, NUL padding, then `argc` NUL-terminated arguments and the environment.
- `command_name(pid) -> Option<String>`: `/proc/<pid>/comm` on Linux, `proc_name` on macOS.
- `executable_path(pid) -> Option<PathBuf>`: `read_link("/proc/<pid>/exe")` on Linux, `proc_pidpath` on macOS.
- `environment(pid) -> Option<Vec<Vec<u8>>>`, `is_zombie(pid) -> bool`, `children(pid) -> Vec<i32>` and `all_pids() -> Vec<i32>` for the tests: `/proc` on Linux, and `KERN_PROCARGS2`, `proc_pidinfo` `PROC_PIDTBSDINFO` with `pbi_status == SZOMB`, `proc_listchildpids` and `proc_listallpids` on macOS.

The session id in `tests/common/mod.rs` moves to `rustix::process::getsid`, which is POSIX and needs no crate function.

The alternative, `cfg` blocks at each call site, spreads 13 sites across 7 files and needs a Mac to review any of them. The `sysinfo` crate covers most of this but reads all processes on refresh, pulls in many dependencies, and does not give the peer uid. The `libproc` crate covers the process half only, and `libc` already has every symbol.

The `" (deleted)"` suffix handling stays in `crates/client/src/transport.rs`, because it is about gband's replaced executable and not about the platform. `proc_pidpath` on macOS returns the path the process was started from, with no suffix, so the strip is a no-op there.

### Binding on macOS: a private directory and a check through a held descriptor

On macOS `bind_within` opens the private directory with `O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC`, binds at `<directory>/s` by its full path, then checks with `fstatat(held, "s", AT_SYMLINK_NOFOLLOW)` that the held directory holds a socket. `chmodat` and `renameat` then act through the held descriptor as on Linux. The private directory has mode `0700` from its creation, so the socket is unreachable to others before the `chmod` to `0600`. The umask is never touched.

When another user replaced the private directory between creation and bind, the bound socket is absent from the held directory, the check fails, and the server reports that it cannot listen. Another user can only do that in a parent directory that is writable by them and lacks the sticky bit, which the `/tmp` the spec names does not.

The alternative, `fchmod` on the unbound socket, has no effect on the file `bind` creates on macOS. Setting the umask around `bind` breaks the spec's requirement that the mask never changes.

### Length limits on macOS

On macOS the bind path is `<parent>/.gband-bind-<pid>-<n>/s`. The pid has at most 5 digits on macOS and `<n>` stays below 100, so the private part adds at most 23 bytes to the directory path. 103 less 23 gives the 80-byte directory limit in the session-server delta. `SUN_PATH_MAX` becomes 107 on Linux and 103 on macOS, and the length check in `run_with_events` gains the directory check on macOS. Both run before `lock::acquire`, as the 107-byte check does now.

### Executable identity at start

`src/executable.rs` keeps `/proc/self/exe` on Linux. On macOS it reads `current_exe()` once and `fstat`s the opened file. `identity()` is already called at start, so the identity names the file the process started from, as the wire-protocol spec requires, as long as nothing replaces the executable in the milliseconds before that call.

### Tests on macOS

`std::env::temp_dir()` on macOS is `$TMPDIR`, a path of about 49 bytes under `/var/folders/`. Tests that bind a socket under a `gband-scratch` directory may break the 80-byte directory limit. `gband-scratch` uses `/tmp` on macOS when `$TMPDIR` would leave less than 30 bytes for a socket name, and the long-path tests build their paths to the platform's limits.

The umask test in `crates/server/src/lib.rs` reads `/proc/self/status`. macOS cannot read the umask without setting it, so that test runs on Linux only. The behaviour it guards is the same code on both platforms.

## Risks / Trade-offs

- [The implementer's Mac gates macOS, and nothing gates Linux in the same run] → A task runs `.claude/flow-gate` in a Linux container on the Mac before `/ready`, using the Rust image, so the Linux paths stay proven.
- [macOS binds Ctrl+Space to "Select the previous input source", so the default prefix key never reaches the terminal] → The README says so and names the System Settings entry that turns it off. Changing the default prefix is out of scope.
- [Terminal.app and iTerm2 send Option as a character unless "Use Option as Meta key" is on, so `alt+` bindings do not fire] → The README names the setting in each terminal.
- [`KERN_PROCARGS2` returns `EINVAL` for another user's process and for a zombie] → Both count as absent, and the window-names fallback applies.
- [Snapshot tests may differ on macOS: the harness sets `SHELL=/bin/sh` at `crates/harness/src/env.rs:179`, which is bash 3.2 in POSIX mode on macOS and bash 5 on Fedora] → A snapshot that differs is fixed in the harness or the test, never by a second per-platform snapshot.

## Migration Plan

None. Linux users see no change. The change ships in the next release, and the README names macOS from then on.
