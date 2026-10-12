Run every task on a Mac. Task 5.2 runs the Linux gate in a container on that Mac.

## 1. The gband-os crate

- [ ] 1.1 Create `crates/os` as `gband-os`, depending on `libc` and `rustix` from the workspace, and add `libc = "0.2.190"` to `[workspace.dependencies]` and `gband-os` to the workspace list. Verify `cargo build -p gband-os` succeeds on macOS
- [ ] 1.2 Add `peer_uid`, with `socket_peercred` on Linux and `getpeereid` on macOS, as design.md's "A new crate `gband-os` holds the platform calls" defines. Verify with a unit test that connects a `UnixStream::pair` and reads the current uid
- [ ] 1.3 Add `arguments`, `command_name` and `executable_path`, with `/proc` on Linux and `KERN_PROCARGS2`, `proc_name` and `proc_pidpath` on macOS. Verify with unit tests on a spawned `sleep` with `arg0` set: arguments, command name `sleep`, executable path `/bin/sleep`, and `None` for each after the process is reaped and for `i32::MAX`
- [ ] 1.4 Add `environment`, `is_zombie`, `children` and `all_pids`, with `/proc` on Linux and `KERN_PROCARGS2`, `PROC_PIDTBSDINFO`, `proc_listchildpids` and `proc_listallpids` on macOS. Verify with unit tests: a spawned child with `GBAND_OS_PROBE=1` appears in `children(std::process::id())`, in `all_pids()`, and holds the variable in `environment`, and an exited unreaped child reads as a zombie

## 2. Client, server and test channel

- [ ] 2.1 Replace `socket_peercred` in `crates/client/src/transport.rs` and `src/channel.rs` with `gband_os::peer_uid`. Verify `cargo build` succeeds on macOS and `cargo test -p gband-client` passes
- [ ] 2.2 Replace `/proc/<pid>/exe` in `crates/client/src/transport.rs` `server_executable` with `gband_os::executable_path`, keeping the replaced-executable suffix strip. Verify the platforms delta's "Debug server replaced on macOS" by hand: start `gband` from a debug build, detach, `touch src/main.rs`, `cargo build`, attach, and confirm the client log records the replacement
- [ ] 2.3 Make `crates/server/src/process.rs` read arguments and command names through `gband_os`, and move its tests' readiness wait off `/proc/<pid>/cmdline`. Verify `cargo test -p gband-server process` passes on macOS, and the window-names delta's "Command in the foreground on macOS" and the agent-state delta's "Claude Code run by node on macOS" by hand
- [ ] 2.4 On macOS, bind as design.md's "Binding on macOS: a private directory and a check through a held descriptor" defines, keeping the Linux `bind_within` unchanged. Restrict the umask test to Linux. Verify `cargo test -p gband-server` passes on macOS, including the eight concurrent servers test and `longest_socket_path_binds`
- [ ] 2.5 Make `SUN_PATH_MAX` 107 on Linux and 103 on macOS, and on macOS refuse a socket directory path over 80 bytes, as design.md's "Length limits on macOS" defines. Make `tests/subcommands.rs` `socket_path_over_the_limit_is_refused` expect the platform's limit, and add macOS tests for the session-server delta's "Long explicit socket path on macOS", "Socket path over the macOS limit" and "Socket directory over the macOS limit". Verify `cargo test --test subcommands` passes
- [ ] 2.6 On macOS, take the identity in `src/executable.rs` from the opened `current_exe()` file. Verify `identity_matches_the_current_executable` passes on macOS

## 3. Test harness and root tests

- [ ] 3.1 Make `crates/harness/src/env.rs` `is_running` and `children`, and `crates/harness/src/terminal.rs` `wait_for_child`, use `gband_os`. Verify `cargo test -p gband-harness` passes on macOS
- [ ] 3.2 Make `tests/common/mod.rs` `session_of` use `rustix::process::getsid`, and `tests/plugin_testing.rs` `processes_of` use `gband_os::all_pids` and `environment`. Verify `cargo test --test plugin_testing` passes on macOS, `nothing_left_behind` included
- [ ] 3.3 Make `gband-scratch` place scratch directories under `/tmp` on macOS when `$TMPDIR` would push a test socket over the macOS limits, as design.md's "Tests on macOS" defines. Verify no test fails on macOS with a socket length error
- [ ] 3.4 Run `cargo test --workspace` on macOS and fix every remaining failure in the code or the test, never with a per-platform snapshot. Verify the run passes twice in a row

## 4. Documentation

- [ ] 4.1 In `README.md`, name Linux and macOS as supported in "Installing", name the Xcode Command Line Tools (`xcode-select --install`) as the macOS C compiler, and add a macOS note: turn off "Select the previous input source" under System Settings, Keyboard, Keyboard Shortcuts, Input Sources so Ctrl+Space reaches gband, and turn on "Use Option as Meta key" in Terminal.app or "Esc+" for the Option key in iTerm2 for `alt+` bindings. Verify by reading it against the platforms delta's "Supported operating systems"

## 5. Final checks

- [ ] 5.1 Run `.claude/flow-gate` on macOS and verify it passes
- [ ] 5.2 Run `.claude/flow-gate` on Linux in a container on the Mac, with the worktree mounted and a separate target directory: `podman run --rm -v "$PWD":/src -w /src -e CARGO_TARGET_DIR=/tmp/target docker.io/library/rust:1 bash -c 'rustup component add rustfmt clippy && ./.claude/flow-gate'`. Verify it passes
- [ ] 5.3 Verify the platforms delta's "Local session on macOS" and "Mac client, Linux server" by hand: run `gband` in Terminal.app, open two windows, detach, attach, then `gband -r user@host` against a Linux host running the same build
