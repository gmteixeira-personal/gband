Run every task on a Mac. "The Linux gate" below means this command, run from the worktree root:

```sh
podman run --rm --platform linux/arm64 -v "$PWD":/src -w /src -e CARGO_TARGET_DIR=/tmp/target docker.io/library/rust:1 bash -c 'rustup component add rustfmt clippy && ./.claude/flow-gate'
```

## 1. The gband-os crate

- [ ] 1.1 Create `crates/os` as `gband-os` with `src/lib.rs`, `src/linux.rs` and `src/macos.rs`, depending on `libc` and `rustix`. Add `libc = "0.2.190"` to `[workspace.dependencies]` and `gband-os` to the workspace list. Put the `compile_error!` for other operating systems in `src/lib.rs`, as design.md's "A new crate `gband-os` holds the platform calls" defines. Verify `cargo build -p gband-os` succeeds on macOS, and, after `rustup target add x86_64-unknown-freebsd`, `cargo check -p gband-os --target x86_64-unknown-freebsd` fails with the error naming Linux and macOS
- [ ] 1.2 Add `peer_uid`. Verify with a unit test that connects a `UnixStream::pair` and reads the current uid
- [ ] 1.3 Add `arguments`, `command_name` and `executable_path`. Verify with unit tests on a spawned `sleep` with `arg0` set: its arguments, the command name `sleep`, the executable path `/bin/sleep`, and `None` for each after the process is reaped and for `i32::MAX`
- [ ] 1.4 Add the pinning test from design.md's "Executable path after a rebuild on macOS": copy `/bin/sleep` to a scratch path, start it, replace the file with a new copy, and assert `executable_path` returns the scratch path. If neither `proc_pidpath` nor the `KERN_PROCARGS2` exec path passes, stop and ask before going on
- [ ] 1.5 Add `environment`, `is_zombie`, `children` and `all_pids`. Verify with unit tests: a spawned child with `GBAND_OS_PROBE=1` appears in `children(std::process::id())` and in `all_pids()`, `environment` holds the variable, and an exited unreaped child reads as a zombie
- [ ] 1.6 Add `SOCKET_PATH_MAX` and `SOCKET_DIRECTORY_MAX`, and move `bind_within` from `crates/server/src/lib.rs` into `src/linux.rs` as `bind_private`, unchanged. Write the macOS `bind_private` as design.md's "Binding on macOS" defines. Verify with a unit test that binds in a fresh `0700` directory, finds the socket at mode `0600`, and finds the umask unchanged
- [ ] 1.7 Run the Linux gate and verify it passes

## 2. Client, server and test channel

- [ ] 2.1 Replace `socket_peercred` in `crates/client/src/transport.rs` and `src/channel.rs` with `gband_os::peer_uid`. Verify `cargo test -p gband-client` passes on macOS
- [ ] 2.2 Replace `/proc/<pid>/exe` in `crates/client/src/transport.rs` `server_executable` with `gband_os::executable_path`, keeping the replaced-executable suffix strip. Verify the platforms delta's "Debug server replaced on macOS" by hand: start `gband` from a debug build, detach, `touch src/main.rs`, `cargo build`, attach, and confirm the client log records the replacement
- [ ] 2.3 Make `crates/server/src/process.rs` read arguments and command names through `gband_os`, and move its tests' readiness wait off `/proc/<pid>/cmdline`. Verify `cargo test -p gband-server process` passes on macOS, and check the window-names delta's "Command in the foreground on macOS" and the agent-state delta's "Claude Code run by node on macOS" by hand
- [ ] 2.4 Make `crates/server/src/lib.rs` bind through `gband_os::bind_private`, and give the private directory the fixed-width name from design.md's "A fixed-width private directory name". Restrict the umask test to Linux. Verify `cargo test -p gband-server` passes on macOS, the eight concurrent servers test and `longest_socket_path_binds` included
- [ ] 2.5 Replace `SUN_PATH_MAX` in `src/main.rs` and `crates/server/src/lib.rs` with the two `gband_os` limits, as design.md's "Length limits in one place" defines. Make `tests/subcommands.rs` `socket_path_over_the_limit_is_refused` expect `gband_os::SOCKET_PATH_MAX`, and add macOS tests for the command-line delta's "Path too long on macOS" and "Directory too long on macOS" and the session-server delta's "Long explicit socket path on macOS". Verify `cargo test --test subcommands` passes
- [ ] 2.6 Replace `ProjectDirs` in `src/logging.rs` with `log_directory_from`, move `directories` to a Linux-only dev-dependency, and add the Linux test that compares `log_directory()` with `ProjectDirs`, as design.md's "The log directory follows XDG on both platforms" defines. Verify the logging delta's "XDG_STATE_HOME set on macOS" with a test, and that `~/Library/Application Support/gband` is not created by `cargo test --workspace`
- [ ] 2.7 Compare every removed Linux hunk with its function in `crates/os/src/linux.rs` in `git diff --no-ext-diff`, as design.md's "Linux guards" defines, and verify each body is unchanged
- [ ] 2.8 Run the Linux gate and verify it passes

## 3. Test harness and root tests

- [ ] 3.1 Make `crates/harness/src/env.rs` `is_running` and `children`, and `crates/harness/src/terminal.rs` `wait_for_child`, use `gband_os`. Verify `cargo test -p gband-harness` passes on macOS
- [ ] 3.2 Make `tests/common/mod.rs` `session_of` use `rustix::process::getsid`, and `tests/plugin_testing.rs` `processes_of` use `gband_os::all_pids` and `environment`. Verify `cargo test --test plugin_testing` passes on macOS, `nothing_left_behind` included
- [ ] 3.3 Make `gband-scratch` use `/tmp` when `temp_dir()` is longer than 20 bytes, as design.md's "Tests on macOS" defines. Verify no test fails on macOS with a socket length error
- [ ] 3.4 Run `cargo test --workspace` on macOS and fix every remaining failure in the code or the test, never with a per-platform snapshot and never by changing an existing assertion. Verify the run passes twice in a row
- [ ] 3.5 Run the Linux gate and verify it passes

## 4. Documentation

- [ ] 4.1 In `README.md`, name Linux and macOS as supported in "Installing", with the macOS version (`sw_vers -productVersion`) and processor (`uname -m`) of this Mac as the tested ones. Name the Xcode Command Line Tools (`xcode-select --install`) as the macOS C compiler. Add a macOS note: turn off "Select the previous input source" under System Settings, Keyboard, Keyboard Shortcuts, Input Sources so Ctrl+Space reaches gband, and turn on "Use Option as Meta key" in Terminal.app or "Esc+" for the Option key in iTerm2 for `alt+` bindings. Verify by reading it against the platforms delta's "Supported operating systems"

## 5. Final checks

- [ ] 5.1 Run `.claude/flow-gate` on macOS and verify it passes
- [ ] 5.2 Run the Linux gate, then run it again with `--platform linux/amd64`, and verify both pass
- [ ] 5.3 Verify the platforms delta's "Local session on macOS" and "Mac client, Linux server" by hand: run `gband` in Terminal.app, open two windows, detach, attach, then `gband -r user@host` against a Linux host running the same build
- [ ] 5.4 In the `/ready` report, ask the integrator to run `.claude/flow-gate` on x86-64 Linux against `origin/change/macos-support` before `/integrate`, as design.md's "Migration Plan" defines
