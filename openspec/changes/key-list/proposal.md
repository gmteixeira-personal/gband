## Why

A user who forgets a key has only the status line's hints, which a narrow terminal cuts short, and nothing that runs a binding by choosing it. The plugin-windows API can now draw a scrollable list that takes keys, so gband can ship a key list as a bundled plugin.

The key list shows the actions' descriptions, and those still say pane, while the README now calls gband's panes windows, as niri does. What `gband.win` draws, the key list included, is a plugin window, never a bare window. The descriptions should say window before the key list puts them on screen.

## What Changes

- **Key list plugin**: a bundled client plugin, `gband.keylist`, set up by the default configuration. It registers the action `keylist.open`, which the default configuration binds to Ctrl+Space then `?`. The action opens a focused float titled `prefix keys`, sized to fit its lines within the ribbon area and at most 15 rows high, with the cursor line on. When the key list is already open, the action focuses it and opens no second one.
  - Each line shows a binding of the `prefix` table, in the order `gband.keymap.list` returns them: its key in the hints segment's short form, such as `C-space`, then its description, or its action's name when it has no description.
  - j, k and the arrow keys move the cursor line, as the plugin window defaults already do.
  - Enter runs the selected binding's action, against the focused pane behind the list, and keeps the list open and focused. A float that the action opens takes focus. A binding to a function has no action the list can run, so its line is drawn muted and Enter does nothing on it. Enter on the list's own line does nothing.
  - `q` and Escape close the list.
  - The plugin's groups `KeyListKey` and `KeyListMuted` link to the status line's groups, and the plugin gives those groups their defaults, so the list draws correctly without a status line.
- **Descriptions say window**: every built-in action's description says "window" where it said "pane". This covers the descriptions that the default bindings, `gband.action.list()`, the key list and the hints segment show, including the floating-panes actions, and the description of the `default_column_width` option. For example, `close_pane` becomes "close the window" and `toggle_pane_floating` becomes "float or tile the window". Error messages and every other text keep their wording until rename-panes-to-windows.
- **`q` closes the focused float**:
  - `close_pane` resolved against the view closes the focused float when one is focused, in the client alone, as `gband.win.close` does, even on a band with no pane. Otherwise it closes the focused pane, tiled or floating, as today. So Ctrl+Space then `q` closes the key list when the list has focus, and Enter on the list's `q` line closes the list.
  - A focused float with no `keys` entry for `q` closes on `q`, as it already does on Escape.
- The default bindings gain `prefix ?`, placed after `prefix R` and the floating-panes bindings, and before `prefix D`.

Out of scope:
- Renaming Lua or protocol identifiers. `close_pane`, `open_pane`, the `focus_pane_*` and `*_pane_height` actions, `gband.pane` and the wire messages keep their names here. rename-panes-to-windows renames them later.
- Running function bindings from the list. `gband.keymap.list` gives no handle to a binding's function. navigation-mode adds that.
- Key lists of tables other than `prefix`, filtering, and mouse input.
- Closing the list when another binding moves focus away from it. The list then stays drawn, and Ctrl+Space then `?` focuses it again.

## Capabilities

### New Capabilities
- `key-list`: the bundled key list plugin: its action, its float, its lines, its keys, and its highlight groups.

### Modified Capabilities
- `actions`: the built-in actions' descriptions say "window"; `close_pane` resolved against the view closes the focused float when one is focused, even on a band with no pane.
- `plugin-windows`: `q` is a default key that closes a focused float; Ctrl+Space then `q` closes the focused float rather than the pane behind it.
- `configuration`: the default configuration sets up `gband.keylist` before its bindings and binds `prefix ?`.
- `client-attach`: the default key table gains `?`, and `q` closes the focused float when one is focused, otherwise the focused pane.

## Impact

- `crates/lua/src/actions.rs`: the action descriptions.
- `crates/lua/src/options.rs`: the `default_column_width` description.
- `crates/lua/src/defaults.lua`: the descriptions, the `gband.keylist` setup and the `prefix ?` binding.
- `crates/lua/src/runtime/gband/keylist.lua`, a new bundled module, listed in `crates/lua/src/bundled.rs`.
- `crates/lua/src/runtime/gband/statusline/hints.lua`: its key form moves to a module both segments use.
- `crates/lua/src/runtime/gband/win.lua`: the `q` default.
- `crates/lua/src/runtime.rs`: an entry point that closes the focused float and runs its `on_close`.
- `crates/client/src/lib.rs`: `close_pane` while a float is focused, including with no pane focused.
- `crates/client/src/bindings.rs`: its tests assume every default binding names a built-in action, and `prefix ?` binds a registered one.
- Tests in `crates/lua/tests/`, `crates/client/tests/` and `tests/`, which compare descriptions or default bindings.
- `docs/plugins.md` and `README.md`: the key list, `prefix ?` and the new descriptions.
- A `user/init.lua` copied from the old defaults keeps descriptions such as `"close the pane"`. They no longer match the actions' descriptions, so the hints segment shows that text in full instead of the short label. Copying `defaults/init.lua` again, or updating the descriptions, restores the labels. The README says so.
- No new dependency, and no protocol change.

## Coordination

### Author
- gmteixeira

### Depends On
- center-column

### Expected Files
- README.md
- crates/client/src/bindings.rs
- crates/client/src/lib.rs
- crates/client/tests/
- crates/lua/src/actions.rs
- crates/lua/src/bundled.rs
- crates/lua/src/defaults.lua
- crates/lua/src/options.rs
- crates/lua/src/runtime.rs
- crates/lua/src/runtime/gband/keyform.lua
- crates/lua/src/runtime/gband/keylist.lua
- crates/lua/src/runtime/gband/statusline/hints.lua
- crates/lua/src/runtime/gband/win.lua
- crates/lua/tests/
- docs/plugins.md
- tests/
- openspec/changes/key-list/
