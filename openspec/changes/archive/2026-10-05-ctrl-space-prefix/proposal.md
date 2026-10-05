## Why

The default prefix key is Ctrl+A, which shells and editors use heavily: readline and Emacs move to the start of the line with it, and Vim increments a number. Every user who keeps the default has to press Ctrl+A twice for one of the most common editing keys. Ctrl+Space is rarely bound by programs that run in a terminal, is easy to reach with either hand, and leaves Ctrl+A to the pane.

## What Changes

- **BREAKING**: the default prefix key becomes Ctrl+Space. The `prefix` option defaults to `"ctrl+space"`, and the default configuration sets `prefix = "ctrl+space"`, so `defaults/init.lua` written on first run shows it.
- Ctrl+A is no longer special in the default configuration. It reaches the focused pane as `\x01` like any other key.
- Pressing Ctrl+Space twice sends one Ctrl+Space, `\x00`, to the focused pane, through the existing `prefix prefix` binding to `send_prefix`.
- Every prefix binding keeps its key: Ctrl+Space then `h` replaces Ctrl+A then `h`, and so on through the whole default table.
- A user who wants Ctrl+A back sets `prefix = "ctrl+a"` in `user/init.lua`. A `user/init.lua` copied from the old defaults already does so and keeps working unchanged.
- No change to key names, the bindings API, or the input encoding: `ctrl+space` is already a valid key name, and Ctrl+Space already encodes as `\x00`.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `configuration`: the `prefix` option's default, and the scenarios that press or name the default prefix.
- `client-attach`: the default prefix in the key bindings table, the literal-prefix scenario, and every scenario that presses the default prefix.
- `actions`: the scenario that compares a dispatched action with pressing the default prefix then `q`.

## Impact

- `crates/lua`: the default configuration file and `Options::default()` name Ctrl+Space. The tests that pin the default prefix change with them.
- `crates/client`: the keymap and action tests that use the default keymap press Ctrl+Space and expect `\x00` for the literal prefix.
- `tests/attach.rs`, `tests/config.rs`, `tests/sessions.rs`: end-to-end tests send `\x00` instead of `\x01` as the prefix.
- `README.md`: the options table shows the new default.
- Builds on config-directory, whose specs and loader these edits assume. It must be archived first.

## Non-goals

- A migration or warning for users who relied on Ctrl+A without a `user/init.lua`.
- Telling Ctrl+Space apart from Ctrl+@ or Ctrl+2, which a terminal without an extended keyboard protocol reports as the same byte.
- Enabling the kitty keyboard protocol or any other extended key reporting.

## Coordination

### Author
- gmteixeira

### Depends On
- config-directory

### Expected Files
- crates/lua/src/defaults.lua
- crates/lua/src/options.rs
- crates/lua/tests/config.rs
- crates/client/src/bindings.rs
- crates/client/tests/actions.rs
- tests/attach.rs
- tests/config.rs
- tests/sessions.rs
- README.md
- openspec/changes/ctrl-space-prefix/
