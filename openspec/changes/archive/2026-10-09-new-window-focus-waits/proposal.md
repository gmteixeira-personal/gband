## Why

`cmd_runs_a_server_command_for_the_chosen_client` in `tests/control.rs` failed 2 times under CPU load, once in about 56 runs and once in 82, both with `no pid line in the focused window` at `crates/harness/src/terminal.rs:204`. Its helper `two_windows_first_focused` opens a window with prefix `n` and then waits only for two tiles. A client focuses a new window when the server's `Opened` message arrives, and that message comes separately from the layout update. So the screen can show two tiles while the first window still has focus. A screen dump of a failing run showed `wait_for_prompt` and `shell_pid` both accepting the first window's old `sh-5.3$` prompt and `pid=` line. Focus then moved to the new window, which was still blank. The helper in `tests/config.rs` that it copies waits for the second tile to be focused (`second_focused`) and has no such race.

Two tests on `dev` use the same weak wait after opening a window: `send_text_runs_a_command_in_another_window` in `tests/plugin_windows.rs` waits for two tiles, and `number_follows_the_layout` in `tests/window_names.rs` waits for two titles. Each then types into the focused window or acts relative to it. Neither has been seen to fail, but both can type into the old window.

## What Changes

- `two_windows_first_focused` in `tests/control.rs` waits until the second tile is focused before it waits for the prompt.
- `send_text_runs_a_command_in_another_window` in `tests/plugin_windows.rs` and `number_follows_the_layout` in `tests/window_names.rs` also wait until the new window's tile is focused before they type.
- Test code only: no behaviour, spec or documentation changes.

Out of scope:
- Changing the client so that a new window's layout and its focus arrive in one frame. The intermediate frame is harmless to people, and each test can wait for the state it reads.
- Changing `shell_pid` in `crates/harness`. Once every caller waits for the focus it means, the old window's pid line can no longer satisfy the wait.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. The change edits tests only, so `.openspec.yaml` sets `skip_specs: true`.

## Impact

- `tests/control.rs`, `tests/plugin_windows.rs`, `tests/window_names.rs`: stronger waits after opening a window.
- No product code, wire protocol, Lua API or documentation change.

## Coordination

### Author
- gmteixeira

### Depends On
- reload-and-control

### Expected Files
- openspec/changes/new-window-focus-waits/
- tests/control.rs
- tests/plugin_windows.rs
- tests/window_names.rs
