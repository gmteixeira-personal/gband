## Why

A user who forgets a key has only the status line's hints, which a narrow terminal cuts short, and nothing that runs a binding by choosing it. The plugin-windows API can draw a scrollable list that takes keys, so gband can ship a key list as a bundled plugin: a floating plugin window that lists the `prefix` bindings and runs the one the user chooses.

## What Changes

- **Key list plugin**: a bundled client plugin, `gband.keylist`, set up by the default configuration. It registers the action `keylist.open`, which the default configuration binds to Ctrl+Space then `?`. The action opens a focused floating plugin window titled `prefix keys`, sized to fit its lines within the ribbon area and at most 15 rows high, with the cursor line on. When the key list is already open, the action focuses it and opens no second one.
  - Each line shows a binding of the `prefix` table, in the order `gband.keymap.list` returns them: its key in the hints segment's short form, such as `C-space`, then its description, or its action's description or name when it has none.
  - j, k and the arrow keys move the cursor line, as the plugin window defaults already do.
  - Enter runs the selected binding's action against the focused window behind the list, and keeps the list open and focused. A floating plugin window that the action opens takes focus. A binding to a function has no action the list can run, so its line is drawn muted and Enter does nothing on it. Enter on the list's own line does nothing.
  - `q` and Escape close the list.
  - The plugin's groups `KeyListKey` and `KeyListMuted` link to the status line's groups, and the plugin gives those groups their defaults, so the list draws correctly without a status line.
- **`q` closes the focused floating plugin window**:
  - `close_window` resolved against the view closes the focused floating plugin window when one is focused, in the client alone, as `gband.win.close` does, even on a band with no window. Otherwise it closes the focused window, tiled or floating. So Ctrl+Space then `q` closes the key list when the list has focus, and Enter on the list's `q` line closes the list.
  - A focused floating plugin window with no `keys` entry for `q` closes on `q`, as it already does on Escape.
- The default bindings gain `prefix ?`, placed after the floating window bindings and before `prefix D`.

Out of scope:
- Running function bindings from the list. `gband.keymap.list` gives no handle to a binding's function. navigation-mode adds that.
- Key lists of tables other than `prefix`, filtering, and mouse input.
- Closing the list when another binding moves focus away from it. The list then stays drawn, and Ctrl+Space then `?` focuses it again.

## Capabilities

### New Capabilities
- `key-list`: the bundled key list plugin: its action, its floating plugin window, its lines, its keys, and its highlight groups.

### Modified Capabilities
- `actions`: `close_window` resolved against the view closes the focused floating plugin window when one is focused, even on a band with no window.
- `plugin-windows`: `q` is a default key that closes a focused floating plugin window; Ctrl+Space then `q` closes it rather than the window behind it.
- `configuration`: the default configuration sets up `gband.keylist` before its bindings and binds `prefix ?`.
- `client-attach`: the default key table gains `?`, and `q` closes the focused floating plugin window when one is focused, otherwise the focused window.
- `plugins`: the modules bundled with gband, which `require` falls back to, gain `gband.keylist` and `gband.keyform`.

## Impact

- `crates/lua/src/defaults.lua`: the `gband.keylist` setup and the `prefix ?` binding.
- `crates/lua/src/runtime/gband/keylist.lua`, a new bundled module, listed in `crates/lua/src/bundled.rs`.
- `crates/lua/src/runtime/gband/keyform.lua`, a new bundled module holding the key form that `crates/lua/src/runtime/gband/statusline/hints.lua` has today.
- `crates/lua/src/runtime/gband/win.lua`: the `q` default.
- `crates/lua/src/runtime.rs`: an entry point that closes the focused floating plugin window and runs its `on_close`.
- `crates/client/src/lib.rs`: `close_window` while a floating plugin window is focused, including with no window focused.
- `crates/client/src/bindings.rs`: its tests assume every default binding names a built-in action, and `prefix ?` binds a registered one.
- Tests in `crates/lua/tests/`, `crates/client/tests/` and `tests/`, which compare default bindings.
- `docs/plugins.md` and `README.md`: the key list and `prefix ?`.
- No new dependency, and no protocol change.

## Coordination

### Author
- gmteixeira

### Depends On
- rename-panes-to-windows
- center-column

### Expected Files
- README.md
- crates/client/src/bindings.rs
- crates/client/src/lib.rs
- crates/client/tests/
- crates/lua/src/bundled.rs
- crates/lua/src/defaults.lua
- crates/lua/src/runtime.rs
- crates/lua/src/runtime/gband/keyform.lua
- crates/lua/src/runtime/gband/keylist.lua
- crates/lua/src/runtime/gband/statusline/hints.lua
- crates/lua/src/runtime/gband/win.lua
- crates/lua/tests/
- docs/plugins.md
- tests/
- openspec/changes/key-list/
