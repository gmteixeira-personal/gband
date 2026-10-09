## Why

The gate (`.claude/flow-gate`) fails on `dev` when other sessions run their own gates at the same time. On 2026-10-09 it failed 4 runs in a row while readying input-before-output, each time on tests that change never touched. A pristine `origin/dev` build (4114105) reproduces three separate causes:

1. **Configuration reloads stop when inotify instances run out.** Every gband client and server creates an inotify instance to watch its configuration. `fs.inotify.max_user_instances` is 128 on this machine, and the limit applies to all of the user's processes. Several gates running at once use all 128. `watch` in `crates/lua/src/watch.rs` then logs `cannot watch the configuration: Too many open files (os error 24)` and never reloads, which breaks the "Reload on change" requirement. A user namespace with the limit lowered to 10 gives the same 7 failures the gate showed in `tests/settings.rs`, each `timed out waiting for the configuration to reload`. `tests/config.rs` `editor_replaces_the_file` and `tests/mouse.rs` `buffer_survives_a_reload` failed the same way. Anyone running many gband processes, or other programs that watch files, can reach this limit.
2. **Two mouse tests type before the new window has focus.** `alt_wheel_switches_bands` and `middle_drag_up_switches_bands_in_navigation_mode` in `tests/mouse.rs` send prefix `u n Enter`, wait for a single tile without `AAA`, and type `echo BBB`. Prefix `u` moves the view to the empty last band, where the client has no focused window. The first frame of the band-switch animation still draws the leaving band's tile, unfocused, and that frame satisfies the wait. The client drops keys typed while it has no focused window, so `echo BBB` is lost. In a container limited to 2 CPUs, the `mouse` binary failed 21 of 40 runs on `dev`.
3. **`cat -v` probes send keys before `cat` runs.** `echo_keys` in `tests/config.rs` types `clear; cat -v`, sleeps 300 ms and sends the key it probes. When bash has not yet run the line, readline still owns the terminal with `ICRNL` off. `cat -v` then prints `^@^M` with no line end, and `wait_for_line("^@")` times out. In a container limited to 1 CPU, the `config` binary failed 2 to 6 of 8 runs on `dev`. Four tests in `tests/attach.rs` use the same fixed sleep.

## What Changes

- Every process watches its configuration by polling, as niri does, instead of through inotify. It checks the `.lua` files under `user` once a second, reading no file contents, and reloads within two seconds of a change. No limit on file-change notifications can stop it, and the `notify` dependency goes.
- The configuration spec's "Reload on change" states that changes are found by polling, allows two seconds instead of one, and gains a scenario. The `settings` and `key-style` scenarios that wait for a reload allow two seconds too.
- The two mouse tests wait for a focused tile and its prompt before they type.
- The harness gains a wait for a named program to run as a child of the focused window's shell. `echo_keys` in `tests/config.rs` and the four `cat -v` tests in `tests/attach.rs` use it instead of a fixed sleep before they send the probe.

Out of scope:
- Raising `fs.inotify.max_user_instances`. Neither gband nor its tests can change a system setting.
- Changing the client to buffer keys typed while no window is focused, or to skip drawing the leaving band's first frame. The frame does no harm in normal use, and the tests can wait for the state they need.
- The `cat -v` cases in `tests/keystyle.rs`. They open the floating window list before sending the probe, which takes a round trip to the server, and they have not been seen to fail.
- Flakes in `tests/lua_specs.rs`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `configuration`: "Reload on change" requires changes to be found by polling, without operating-system file-change notifications, within two seconds instead of one, and gains a scenario.
- `settings`: the scenarios of "Settings window keys" that wait for a reload allow two seconds.
- `key-style`: "Saved key style"'s scenario "Saving reloads the configuration" allows two seconds.

## Impact

- `crates/lua/src/watch.rs`: the polling thread and its unit test.
- `Cargo.toml`, `crates/lua/Cargo.toml`, `Cargo.lock`: `notify` removed.
- `crates/harness/src/terminal.rs`: the wait for a child program.
- `tests/mouse.rs`, `tests/config.rs`, `tests/attach.rs`: stronger waits.
- No wire protocol change, no Lua API change, no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- openspec/changes/gate-load-flakes/
- crates/lua/src/watch.rs
- crates/lua/Cargo.toml
- Cargo.toml
- Cargo.lock
- crates/harness/src/terminal.rs
- tests/mouse.rs
- tests/config.rs
- tests/attach.rs
