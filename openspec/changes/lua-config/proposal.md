## Why

gband promises Lua configuration, but every setting is still a Rust constant: the key binding table is fixed in the client, the width presets and the default column width are fixed in the layout, and the camera has one policy. This change embeds the Lua runtime for its first job, configuration and key bindings, so a user can change them without rebuilding. The defaults move into a `defaults.lua` written against the same public API, which proves the API covers everything gband does out of the box.

## What Changes

- Load `$XDG_CONFIG_HOME/gband/init.lua` (`~/.config/gband/init.lua` by default) after an embedded `defaults.lua`. Both the client and the server evaluate it; each applies the parts it owns.
- Add a `gband` Lua table with:
  - `gband.set { ... }`: options validated into typed Rust structs through mlua's serde support. The options are `prefix`, `default_column_width`, `width_presets` and `center_focused_column`.
  - `gband.bind(keys, action)`: binds a key, such as `"alt+h"`, or a prefix sequence, such as `"prefix h"`, to an action value or a Lua function. `gband.unbind(keys)` removes a binding.
  - `gband.action.*`: one value per existing action, callable from inside a binding function.
  - `gband.spawn { cmd = ... }`: opens a pane running a given command, from inside a binding function.
- Replace the hardcoded `BINDINGS` table, `PREFIX`, `WIDTH_PRESETS` and `DEFAULT_WIDTH` with configuration. `defaults.lua` reproduces today's behaviour exactly.
- Support direct bindings, which act without the prefix, alongside prefix bindings. The prefix key itself is the `prefix` option.
- Add the `center_focused_column` camera policy: `"never"` (today's behaviour and the default), `"always"` and `"on-overflow"`, following niri.
- Let the open pane session action carry an optional program, so `gband.spawn` can run a command other than the shell.
- Report a configuration error with its file and line in a one-line client banner and in the log. Keep running the last configuration that loaded, or the defaults when none has.
- Watch `init.lua` and reload it when it changes.

## Capabilities

### New Capabilities
- `configuration`: the Lua configuration file, its evaluation order, the `gband` API, option validation, error reporting, the last good configuration and live reload.

### Modified Capabilities
- `client-attach`: key bindings come from the configuration, support direct and prefix bindings and Lua functions, and the default table becomes the content of `defaults.lua`.
- `actions`: a binding names one action or a Lua function, and every action has a Lua name.
- `layout`: the width presets and the default column width come from the configuration.
- `layout-view`: the camera follows the `center_focused_column` policy.
- `session-server`: open pane may name the program to run instead of the user's shell.
- `wire-protocol`: the open pane action carries an optional program.

## Impact

- New crate `crates/lua` (`gband-lua`) holding the Lua API, option structs, key parsing, `defaults.lua` and the loader with its file watcher. The client and the server depend on it; `mlua` and `notify` are already workspace dependencies.
- `crates/client`: `bindings.rs` loses its table and becomes a binding resolver over the loaded configuration; the client runs Lua binding functions and draws the error banner.
- `crates/core`: `layout.rs` takes widths from options instead of constants; `view.rs` gains the camera policy and a signed camera; `action.rs` and `SessionAction::OpenPane` gain the program.
- `crates/server`: loads and reloads the configuration, and passes options to the layout and programs to pane spawning.
- The wire format of open pane changes. Client and server come from one build, and the handshake already refuses a mismatched server.

## Non-goals

- Layout rules, presentation Lua, themes, key tables beyond the single prefix, and a which-key popup.
- Lua modules loaded with `require` from the configuration directory, and reload on changes to files other than `init.lua`.
- A guard against runaway Lua scripts.
- Per-project configuration files.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- Cargo.toml
- Cargo.lock
- README.md
- crates/lua/
- crates/core/src/action.rs
- crates/core/src/layout.rs
- crates/core/src/view.rs
- crates/core/tests/events.rs
- crates/core/tests/geometry.rs
- crates/core/tests/layout.rs
- crates/core/tests/view.rs
- crates/client/Cargo.toml
- crates/client/src/animation.rs
- crates/client/src/bindings.rs
- crates/client/src/lib.rs
- crates/client/src/render.rs
- crates/client/tests/actions.rs
- crates/client/tests/animation.rs
- crates/client/tests/render.rs
- crates/client/tests/snapshots/render__centred_first_column_leaves_the_strip_start_empty.snap
- crates/client/tests/snapshots/render__configuration_error_banner_covers_the_bottom_row.snap
- crates/server/Cargo.toml
- crates/server/src/lib.rs
- crates/server/src/registry.rs
- crates/server/src/session.rs
- crates/server/tests/panes.rs
- crates/server/tests/programs.rs
- crates/server/tests/resize.rs
- crates/protocol/tests/messages.rs
- crates/test-support/src/lib.rs
- src/main.rs
- tests/common/mod.rs
- tests/config.rs
- openspec/changes/lua-config/
