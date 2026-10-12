## Context

See proposal.md for the motivation. Every Linux-only call in the tree at `eab6e3c`:

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
| `src/logging.rs:73` | `directories::ProjectDirs` | log directory; ignores `XDG_STATE_HOME` on macOS |
| `crates/harness/src/env.rs:289,298` | `/proc/<pid>/stat`, `task/*/children` | zombie check, child list |
| `crates/harness/src/terminal.rs:212` | `/proc/<pid>/comm` | waits for a program under a shell |
| `tests/common/mod.rs:69` | `/proc/<pid>/stat` field 6 | session id |
| `tests/plugin_testing.rs:425,431` | `/proc`, `/proc/<pid>/environ` | finds processes a test run left behind |

The first three fail to compile on macOS. The rest compile and fail at run time. `rustix` 1.1.5 gates `socket_peercred` on `linux_kernel` and `OFlags::PATH` on Linux, FreeBSD, Fuchsia, Redox and Emscripten. `libc` 0.2.190, already in `Cargo.lock`, exposes `getpeereid`, `proc_pidpath`, `proc_pidinfo` with `PROC_PIDTBSDINFO`, `proc_listchildpids`, `proc_listallpids`, `proc_name` and `KERN_PROCARGS2` for Apple targets.

The macOS socket address holds 104 bytes of path, so 103 usable, against 108 and 107 on Linux. macOS has no `bindat`, and binding through a held directory needs `/proc/self/fd`. Two places check the length today, `src/main.rs:93` for every subcommand and `crates/server/src/lib.rs:78` in the server, both against `SUN_PATH_MAX = 107`.

`src/executable.rs` needs no change: on macOS `/proc/self/exe` fails and the existing `current_exe` fallback returns the same file, at the same moment, at start.

## Goals / Non-Goals

**Goals:**
- Production code outside `gband-os` carries no `cfg(target_os)`. A test carries one only for behaviour that exists on one platform, as "Linux guards" lists.
- Linux behaviour stays what it is now, and each Linux body moves into `gband-os` unchanged.
- `.claude/flow-gate` passes on macOS, on arm64 Linux and on x86-64 Linux.

**Non-Goals:**
- FreeBSD, other BSDs and Windows.
- A CI workflow. No CI exists, and adding one is a separate decision.
- Changes to the wire protocol. A Mac client and a Linux server of the same version already speak the same bytes.
- Dropping the unused `notify-rust` dependency of `gband-client`. It builds on macOS.

## Decisions

### A new crate `gband-os` holds the platform calls

`crates/os` has `src/linux.rs` and `src/macos.rs`, selected in `src/lib.rs` with `#[cfg(target_os = ...)]` and re-exported under one set of names. `src/lib.rs` also holds `#[cfg(not(any(target_os = "linux", target_os = "macos")))] compile_error!("gband supports Linux and macOS")`, so an unsupported target fails at one line naming both.

| function | Linux | macOS |
|---|---|---|
| `peer_uid(fd)` | `socket_peercred` | `libc::getpeereid` |
| `arguments(pid)` | `/proc/<pid>/cmdline` | `sysctl` `CTL_KERN, KERN_PROCARGS2, pid`: `argc` as `i32`, the exec path, NUL padding, then `argc` NUL-terminated arguments and the environment |
| `command_name(pid)` | `/proc/<pid>/comm` | `proc_name` |
| `executable_path(pid)` | `read_link("/proc/<pid>/exe")` | `proc_pidpath`, falling back to the exec path of `KERN_PROCARGS2` |
| `environment(pid)` | `/proc/<pid>/environ` | the environment of `KERN_PROCARGS2` |
| `is_zombie(pid)` | `/proc/<pid>/stat` state `Z` | `proc_pidinfo` `PROC_PIDTBSDINFO`, `pbi_status == SZOMB` |
| `children(pid)` | `/proc/<pid>/task/*/children` | `proc_listchildpids` |
| `all_pids()` | `read_dir("/proc")` | `proc_listallpids` |
| `bind_private(directory, name)` | the current `bind_within`, unchanged | see "Binding on macOS" |
| `SOCKET_PATH_MAX` | 107 | 103 |
| `SOCKET_DIRECTORY_MAX` | `None` | `Some(80)` |

The session id in `tests/common/mod.rs` moves to `rustix::process::getsid`, which is POSIX.

The alternative, `cfg` blocks at each call site, spreads 13 sites across 8 files and needs a Mac to review any of them. The `sysinfo` crate reads every process on refresh, pulls in many dependencies, and does not give the peer uid. The `libproc` crate covers the process half only, and `libc` already has every symbol.

The `" (deleted)"` suffix strip stays in `crates/client/src/transport.rs`, because it concerns gband's replaced executable. On macOS no path carries the suffix, so the strip changes nothing there.

### Linux guards

- Each Linux body in `crates/os/src/linux.rs` is the code removed from its call site, copied unchanged apart from the function signature. The task that moves a body compares the removed hunk with the new function in `git diff --no-ext-diff` before ticking.
- No assertion of an existing test changes. `tests/subcommands.rs` `socket_path_over_the_limit_is_refused` reads `gband_os::SOCKET_PATH_MAX` and still expects 107 on Linux.
- A test of behaviour both platforms share has no `cfg` and runs on both, so it checks the Linux body and the macOS body: the `gband-os` tests of peer uid, arguments, command name, environment, children, zombies and `bind_private`, and every existing test.
- A test of macOS-only behaviour runs only on macOS, through `#[cfg(target_os = "macos")]` or by living in `crates/os/src/macos.rs`: the 103-byte and 80-byte limits, the log directory scenarios on macOS, and the executable path after a rebuild.
- A test of Linux-only behaviour runs only on Linux, through `#[cfg(target_os = "linux")]`: the umask test in `crates/server/src/lib.rs`, and the `ProjectDirs` comparison in `src/logging.rs`. Neither is deleted.
- `.claude/flow-gate` runs in an arm64 Linux container after every task group, and in an amd64 Linux container before `/ready`. The amd64 run matters because `c_char` is unsigned on arm64 Linux and signed on x86-64, so code that compiles on one can fail on the other.
- The integrator runs `.claude/flow-gate` on this x86-64 Linux machine against `origin/change/macos-support` before `/integrate`, because `/integrate` runs no gate.

### Binding on macOS: a private directory and a check through a held descriptor

On macOS `bind_private` opens the private directory with `O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC`, binds at `<directory>/s` by its full path, then checks with `fstatat(held, "s", AT_SYMLINK_NOFOLLOW)` that the held directory holds a socket. `chmodat` and `renameat` then act through the held descriptor, as on Linux. The private directory has mode `0700` from its creation, so no other user can reach the socket before the `chmod` to `0600`. The umask is never touched.

When another user replaced the private directory between creation and bind, the bound socket is absent from the held directory, the check fails, and the server reports that it cannot listen. Only a parent directory writable by that user and lacking the sticky bit allows the replacement. `/tmp` has the sticky bit.

Two alternatives were set aside. Setting the umask around `bind` breaks the session-server requirement that the mask never changes. `fchmod` on the socket before `bind` has no documented effect on the mode of the file `bind` creates on macOS, so the spec's mode guarantee would rest on unverified behaviour.

### A fixed-width private directory name

The private directory name today is `.gband-bind-<pid>-<n>`, where `<n>` comes from `NEXT`, a counter shared by every bind in the process. Tests start many servers in one process, so `<n>` grows past 99 and the name's length is unbounded.

The name becomes `.gband-bind-<pid % 100000, 5 digits>-<NEXT % 100, 2 digits>`: 20 bytes on every platform. A clash with another process's directory gives `AlreadyExists`, which the existing 100-attempt loop already retries with the next counter. The bind path is then the directory's path plus 23 bytes (`/`, 20 bytes, `/s`), so an 80-byte directory gives a 103-byte bind path. The name change is the only edit to shared bind code. The Linux tests that find leftovers match the unchanged prefix `.gband-bind-`.

### Length limits in one place

`src/main.rs` and `crates/server/src/lib.rs` check `gband_os::SOCKET_PATH_MAX`, then `gband_os::SOCKET_DIRECTORY_MAX` against the path's parent. Both checks run before `lock::acquire`, as the 107-byte check does now. A path over both macOS limits names the 103-byte one, as the command-line delta requires. `SUN_PATH_MAX` leaves `gband-server`.

### The log directory follows XDG on both platforms

`src/logging.rs` gets `log_directory_from(xdg_state_home, home)`, the shape of `config_dir_from` in `crates/lua/src/directory.rs`: an absolute `XDG_STATE_HOME` gives `<it>/gband/log`, otherwise `<home>/.local/state/gband/log`. `home` is `HOME`, as `config_dir_from` reads it. No `HOME` keeps today's error, `cannot determine the home directory`.

On Linux this gives the answer `ProjectDirs::state_dir` gives. `directories` moves to a Linux-only dev-dependency, and a Linux test asserts that `log_directory()` equals `ProjectDirs::from("", "", "gband")` `state_dir()` joined with `log`, for the test's own environment. That test is the guard that the Linux answer did not move.

The alternative, macOS's `~/Library/Logs/gband`, follows Apple's convention. It would need its own override for tests, and it would split from the configuration, data and runtime directories, which already follow XDG on macOS.

### Executable path after a rebuild on macOS

On Linux, `/proc/<pid>/exe` names the old path with `" (deleted)"` after `cargo build` replaces the file. On macOS, `proc_pidpath` reads the path of the process's text vnode. Its result for a vnode whose file was unlinked or replaced is unverified. `executable_path` therefore falls back to the exec path in `KERN_PROCARGS2`, the path given to `execve`, which never changes after the exec. The client starts the server from its own absolute `current_exe`, so that path is absolute.

A `gband-os` test on the Mac pins the behaviour: copy `/bin/sleep` to a scratch path, start it, replace the file with a new copy, and assert that `executable_path` returns the scratch path.

### Tests on macOS

`std::env::temp_dir()` on macOS is `$TMPDIR`, about 49 bytes under `/var/folders/`. `gband-scratch` places its directories under `/tmp` when `temp_dir()` is longer than 20 bytes. The rule names no platform. On Linux with `TMPDIR` unset, `temp_dir()` is `/tmp`, so nothing moves.

## Risks / Trade-offs

- [`executable_path` returns neither path after a rebuild on macOS] → The pinning test fails. The implementer stops and asks before changing the platforms delta's "Debug server replaced on macOS".
- [macOS binds Ctrl+Space to "Select the previous input source", so the default prefix key never reaches the terminal] → The README says so and names the System Settings entry that turns it off. Changing the default prefix is out of scope.
- [Terminal.app and iTerm2 send Option as a character unless "Use Option as Meta key" or "Esc+" is on, so `alt+` bindings do not fire] → The README names the setting in each terminal.
- [`KERN_PROCARGS2` returns `EINVAL` for another user's process and for a zombie] → Both count as absent, and the window-names fallback applies.
- [Snapshot tests may differ on macOS: the harness sets `SHELL=/bin/sh` at `crates/harness/src/env.rs:179`, which is bash 3.2 in POSIX mode on macOS and bash 5 on Fedora] → A snapshot that differs is fixed in the harness or the test, never by a second per-platform snapshot.
- [Only one Mac tests the change] → The README names that Mac's macOS version and processor, as the platforms delta requires, and claims nothing for others.

## Migration Plan

None for users. Linux users see no change. Before `/integrate`, the integrator runs `.claude/flow-gate` on x86-64 Linux in a checkout of `origin/change/macos-support`. A failure there goes back to the Mac as a conflict would, by running `/ready` again after the fix.
