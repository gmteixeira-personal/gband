## Why

Every layout key today costs two presses: Ctrl+Space, then the key, after which typing goes back to the focused window. Moving across four columns or growing a column by three steps means pressing the prefix four or three times. gband needs modes: a navigation mode, entered once with the leader, where hjkl move and `-` and `=` resize as often as the user presses them, and an interactive mode, where keys type into the focused window. The mode logic must stay in Lua: the default configuration declares the mode and its bindings decide when to leave it, so a user can reshape it without touching Rust.

## What Changes

- **Modes**: a key table can be declared a mode with `gband.keymap.mode(table, opts)`, while the configuration loads. `opts.label` is an optional display name. While a mode is active, a bound key runs its binding and the mode stays active. An unbound key is discarded and the mode stays active. The mode ends only when a binding enters another table with `gband.keymap.enter`. A table that is not a mode keeps today's one-key behaviour.
- **Leaving to root**: `gband.keymap.enter("root")` is always allowed, even when `root` holds no binding. It is how a Lua binding returns to interactive mode.
- **Labels**: `gband.keymap.label(table)` returns a mode's label, or the table's name when it has none. The bundled mode segment shows the label of the active table. The hints segment labels the prefix key's root hint with the `prefix` table's label.
- **Running a binding from Lua**: `gband.keymap.run(table, key)` runs a key's binding as if the key were pressed in that table, function bindings included, and sends nothing to any window.
- **Key list**:
  - Enter runs every binding through `gband.keymap.run`, so function bindings run too and are no longer muted. Only the list's own `?` line stays muted.
  - Opening the list enters `root`, so its keys reach the list and not navigation mode.
  - Its title uses the `prefix` table's label: `navigation keys`.
- **Default configuration**: the `prefix` table becomes a mode labelled `navigation`. Ctrl+Space enters it and the status line shows `navigation`. Every existing `prefix` binding stays in it and keeps the mode active, so `h`, `j`, `k`, `l`, `-`, `=`, `_`, `+` and the rest repeat. The new and changed keys, each a plain Lua function in the configuration where it leaves the mode:
  - Escape and Enter call `gband.keymap.enter("root")`, returning to interactive mode.
  - `n` dispatches `open_window`, then enters `root`.
  - Ctrl+Space dispatches `send_prefix`, then enters `root`.
  - The arrow keys focus left, right, down and up beside `h`, `l`, `j` and `k`.
  - `i` keeps viewing the band above and does not leave the mode.
- **BREAKING**: Enter in navigation mode no longer opens a window. `n` replaces it.
- **BREAKING**: after Ctrl+Space and a layout key, typed keys no longer reach the focused window until a binding returns to interactive mode.

Out of scope:
- Renaming the `prefix` table or the `prefix` key word in bindings. User configurations that bind `prefix x` keep working, as one-key bindings unless they declare `prefix` a mode.
- A timeout that leaves a mode, per-mode cursor styles, and mouse input.
- Other modes in the default configuration, such as a resize-only mode.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `configuration`: `gband.keymap.mode`, `gband.keymap.label`, `gband.keymap.run`, `gband.keymap.enter("root")`, and the default configuration's mode declaration and function bindings.
- `client-attach`: how a key is handled while a mode is active, and the default key table of navigation mode.
- `key-hints`: the prefix key's root hint uses the `prefix` table's label, and the defaults' hints change.
- `status-line`: the mode segment shows the active table's label.
- `key-list`: Enter runs function bindings, opening the list enters `root`, and the title uses the label.

## Impact

- `crates/lua/src/keymap.rs`: mode declarations, labels, `run`, `enter("root")`, and the modes passed out with the key tables.
- `crates/lua/src/lib.rs` and `api.rs`: the configuration carries the set of modes, and `run` queues an action binding or calls a function binding in the current callback.
- `crates/lua/src/defaults.lua`: the `navigation` mode and its bindings.
- `crates/lua/src/runtime/gband/statusline/mode.lua`, `hints.lua` and `keylist.lua`: labels, running bindings, entering `root`.
- `crates/client/src/bindings.rs` and `lib.rs`: the leader keeps a mode active, and the client passes the configuration's modes to its keymap.
- Tests in `crates/lua/tests/`, `crates/client/tests/`, `tests/`, `tests/lua/` and `examples/plugins/agent-status/tests/` that press Ctrl+Space and then type, open a window with Ctrl+Space then Enter, or expect `C-space prefix` and `prefix keys` with the defaults.
- `README.md` and `docs/plugins.md`: modes and the default keys.
- No protocol change and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- floating-windows
- key-list
- rename-panes-to-windows

### Expected Files
- crates/lua/src/keymap.rs
- crates/lua/src/lib.rs
- crates/lua/src/runtime.rs
- crates/lua/src/defaults.lua
- crates/lua/src/runtime/gband/statusline/mode.lua
- crates/lua/src/runtime/gband/statusline/hints.lua
- crates/lua/src/runtime/gband/keylist.lua
- crates/lua/tests/keymap.rs
- crates/lua/tests/config.rs
- crates/lua/tests/statusline.rs
- crates/lua/tests/key_hints.rs
- crates/lua/tests/plugins.rs
- crates/client/src/bindings.rs
- crates/client/src/lib.rs
- crates/client/tests/actions.rs
- crates/client/tests/events.rs
- crates/client/tests/statusline.rs
- crates/client/tests/plugin_windows.rs
- crates/client/tests/snapshots/statusline__default_status_line.snap
- crates/client/tests/snapshots/statusline__prefix_hints_cut.snap
- crates/client/tests/snapshots/statusline__top_placement.snap
- tests/attach.rs
- tests/config.rs
- tests/statusline.rs
- tests/plugin_windows.rs
- tests/floating.rs
- tests/plugin_runtime.rs
- tests/plugin_testing.rs
- tests/navigation.rs
- tests/lua/keylist_spec.lua
- tests/lua/screenshots/keylist_spec/
- tests/lua/statusline_spec.lua
- tests/lua/screenshots/statusline_spec/default-status-line.txt
- tests/lua/loop_bands_spec.lua
- examples/plugins/agent-status/tests/agent_status_spec.lua
- examples/plugins/agent-status/tests/screenshots/agent_status_spec/a-prompt-marks-the-window-waiting.txt
- README.md
- docs/plugins.md
- docs/testing.md
- openspec/changes/navigation-mode/
