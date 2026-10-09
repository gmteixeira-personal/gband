## 1. Waits for the new window's focus

- [x] 1.1 In `tests/control.rs`, make `two_windows_first_focused` wait, after `client.send(b"\x00n\r")`, until there are two tiles and the second is focused, as `second_focused` in `tests/config.rs` does, before `wait_for_prompt` and `shell_pid`. Verify with `cargo test --test control`, then run `cargo test --test control eval_acts_in_the_client cmd_runs_a_server_command_for_the_chosen_client` 50 times in a loop and check that every run passes
- [x] 1.2 In `tests/plugin_windows.rs`, make `send_text_runs_a_command_in_another_window` wait until the second of two tiles is focused, in place of waiting for two tiles, before `wait_for_prompt`. Verify with `cargo test --test plugin_windows send_text_runs_a_command_in_another_window`
- [x] 1.3 In `tests/window_names.rs`, make `number_follows_the_layout` wait until the titles read `sh #1` and `sh #2` and the second tile is focused, before `wait_for_prompt`. Verify with `cargo test --test window_names number_follows_the_layout`

## 2. Gate

- [x] 2.1 Run `.claude/flow-gate` and check that fmt, clippy and the workspace tests pass
