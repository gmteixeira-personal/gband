## 1. Configuration watch fallback

- [ ] 1.1 In `crates/lua/src/watch.rs`, split `watch` into `watch_with(config, changed, backend)` with `Backend::Native` and `Backend::Poll`, and keep the watcher as `Arc<Mutex<Box<dyn notify::Watcher + Send>>>`. With `Backend::Native`, fall back to a `PollWatcher` with a 250 ms interval when creating the native watcher fails, or when the first `watch` call fails with `ErrorKind::MaxFilesWatch`, and log `watching the configuration by polling: <error>` as a warning. A path that does not exist keeps today's behaviour. Verify with `cargo test -p gband-lua watch`
- [ ] 1.2 Add a unit test beside the existing ones in `watch.rs` that builds the watcher with `Backend::Poll`, writes `user/init.lua`, and checks with `next` that a good configuration loads within one second, then that a second write reloads again. Verify with `cargo test -p gband-lua watch`, then run that test 20 times in a loop and check that every run passes
- [ ] 1.3 Check the "Notification limit reached" scenario by hand: run the `settings` test binary with the inotify instance limit lowered to 10 in a nested user namespace mapped back to your own uid, so the shell prompt still ends in `$`: `unshare -U -r bash -c 'echo 10 >/proc/sys/user/max_inotify_instances && exec unshare -U --map-user="$0" --map-group="$1" "$2"' "$(id -u)" "$(id -g)" target/debug/deps/settings-<hash>`. Before the fix it fails 7 tests with `timed out waiting for the configuration to reload`. Check that all 27 pass

## 2. Mouse waits

- [ ] 2.1 In `tests/mouse.rs`, make `alt_wheel_switches_bands` and `middle_drag_up_switches_bands_in_navigation_mode` wait, after prefix `u n Enter`, until the single tile is focused and lacks `AAA`, then call `wait_for_prompt` before `client.run("echo BBB")`. Verify with `cargo test --test mouse`, then run the `mouse` test binary 20 times in a podman container limited to `--cpus=2 --memory=4g --memory-swap=4g` with `RUST_TEST_THREADS=16`, and check that neither test fails. Before the fix, `dev` failed 21 of 40 such runs

## 3. Probe waits

- [ ] 3.1 In `crates/harness/src/terminal.rs`, add `Attached::wait_for_child(&self, shell: i32, program: &str)`, which waits up to `TIMEOUT` until a child of `shell`, as `children` in `crates/harness/src/env.rs` lists them, has `/proc/<child>/comm` equal to `program`, and panics naming the program on timeout. Verify with `cargo build --tests`
- [ ] 3.2 In `tests/config.rs`, make `echo_keys` read `client.last_pid()` before it types `clear; cat -v`, and replace its 300 ms sleep with `wait_for_child(pid, "cat")`. Verify with `cargo test --test config`, then run the `config` test binary 8 times in a podman container limited to `--cpus=1 --memory=4g --memory-swap=4g` with `RUST_TEST_THREADS=24`, and check that no test fails on a probe wait (`^@`, `^A`, `^@q`, `^[h`, `^[j`). Before the fix, `dev` failed 2 to 6 of 8 such runs
- [ ] 3.3 In `tests/attach.rs`, replace the 300 ms sleep after each `cat -v` command in `prefix_key_twice_sends_one_ctrl_space`, `ctrl_enter_reaches_a_program_that_asked_for_modify_other_keys`, `reattach_keeps_modify_other_keys` and `ctrl_enter_is_enter_without_modify_other_keys` with `wait_for_child` on the pid `shell_pid` returned. Keep the sleeps between probe keys and the one after the second attach. Verify with `cargo test --test attach`

## 4. Gate

- [ ] 4.1 Run `.claude/flow-gate` and check that fmt, clippy and the workspace tests pass
