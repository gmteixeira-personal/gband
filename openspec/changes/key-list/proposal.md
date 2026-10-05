## Why

A user who forgets a key has only the status line's hints, which a narrow terminal cuts short, and nothing that runs a binding by choosing it. The plugin-windows API can now draw a scrollable list that takes keys, so gband can ship a key list as a bundled plugin. The list is a window, and gband's other windows are its panes, so the user-facing text should call all of them windows: `q` closes the window that has focus, whether that is a pane or the key list.

## What Changes

- **Key list plugin**: a bundled client plugin, `gband.keylist`, set up by the default configuration. It registers the action `keylist.open`, which the default configuration binds to Ctrl+Space then `?`. The action opens a focused float titled `prefix keys`, 50 columns by 15 rows, with the cursor line on. Each line shows a binding of the `prefix` table, in the order `gband.keymap.list` returns them: its key, then its description, or its action's name when it has no description.
  - j, k and the arrow keys move the cursor line, as the window defaults already do.
  - Enter runs the selected binding's action and keeps the list open and focused. A binding to a function has no action the list can run; its line is drawn muted and Enter does nothing on it. Enter on the list's own line does nothing.
  - `q` and Escape close the list.
- **Panes are called windows**: every user-facing description says "window" where it said "pane". This covers the built-in actions' descriptions, which the default bindings, `gband.action.list()`, the key list and the hints segment show, the description of the `default_column_width` option, and the README's action table. For example, `close_pane` becomes "close the window" and `focus_pane_down` becomes "focus the window below".
- **`q` closes the focused window**:
  - `close_pane` resolved against the view closes the focused float when one is focused, in the client alone, as `gband.win.close` does. Otherwise it closes the focused pane, as today. So Ctrl+Space then `q` closes the key list when the list has focus, and Enter on the list's `q` line closes the list.
  - A focused float with no `keys` entry for `q` closes on `q`, as it already does on Escape.
- The default bindings gain `prefix ?`, placed after `prefix R` and before `prefix D`.

Out of scope:
- Renaming Lua or protocol identifiers. `close_pane`, `open_pane`, the `focus_pane_*` and `*_pane_height` actions, `gband.pane`, and the wire messages keep their names. Only the text shown to the user changes.
- Running function bindings from the list. `gband.keymap.list` gives no handle to a binding's function.
- Key lists of tables other than `prefix`, filtering, and mouse input.
- The testing demo's `e` shortcut, which typed into the first pane, and its `P` pane.

## Capabilities

### New Capabilities
- `key-list`: the bundled key list plugin: its action, its float, its lines, its keys, and its highlight groups.

### Modified Capabilities
- `actions`: the built-in actions' descriptions say "window"; `close_pane` resolved against the view closes the focused float when one is focused.
- `plugin-windows`: `q` is a default key that closes a focused float; Ctrl+Space then `q` closes the focused float rather than the pane behind it.
- `configuration`: the default configuration sets up `gband.keylist` and binds `prefix ?`; the `default_column_width` option's description says "window".
- `client-attach`: the default key table gains `?`, and `q` closes the focused window.

## Impact

- `crates/lua/src/actions.rs`: the action descriptions.
- `crates/lua/src/options.rs`: the `default_column_width` description.
- `crates/lua/src/defaults.lua`: the descriptions, the `gband.keylist` setup and the `prefix ?` binding.
- `crates/lua/src/runtime/gband/keylist.lua`, a new bundled module, listed in `crates/lua/src/bundled.rs`.
- `crates/lua/src/runtime/gband/win.lua`: the `q` default.
- `crates/client/src/lib.rs`: `close_pane` while a float is focused.
- Tests in `crates/lua/tests/`, `crates/client/tests/` and `tests/`, which compare descriptions or default bindings.
- `docs/plugins.md` and `README.md`.
- No new dependency, and no protocol change.
