## Why

gband follows niri, and niri calls the things on its strips windows. gband called them panes, the word tmux and Zellij use for the regions they split a screen into. Since the amended v0.1.0 README, the docs call them windows and call what `gband.win` draws plugin windows. The code, the Lua API, the specs and the sample plugins still say pane, so the docs have to explain that the API names disagree with them. This change makes every name agree with the docs.

## What Changes

- gband has two things on screen: windows and plugin windows. Each is tiled or floating. The specs call every pane a window, and every floating pane a floating window. A float, the `gband.win` window drawn over one client's ribbon, becomes a floating plugin window. A pane window, the `gband.win` window that sits in the layout, becomes a tiled plugin window. A plugin pane, the window in the layout that shows one and runs no program, becomes a drawn window. Every other use of "window" for what `gband.win` draws becomes "plugin window". Requirement names follow, such as "Pane program" becoming "Window program" and "Window API" becoming "Plugin window API", and so do scenario titles and Purpose sections. The floating-panes capability becomes floating-windows.
- **BREAKING**: the Lua actions `open_pane`, `close_pane`, `focus_pane_down`, `focus_pane_up`, `move_pane_down`, `move_pane_up`, `toggle_pane_floating`, `grow_pane_height`, `shrink_pane_height` and `reset_pane_height` become `open_window`, `close_window`, `focus_window_down`, `focus_window_up`, `move_window_down`, `move_window_up`, `toggle_window_floating`, `grow_window_height`, `shrink_window_height` and `reset_window_height`.
- **BREAKING**: `gband.pane` becomes `gband.window`, and `gband.pane_state` becomes `gband.window_state`.
- **BREAKING**: the events `PaneOpened`, `PaneClosed`, `PaneExited`, `PaneOutput`, `PaneInput` and `PaneStateChanged` become `WindowOpened`, `WindowClosed`, `WindowExited`, `WindowOutput`, `WindowInput` and `WindowStateChanged`. Every `pane` field becomes `window` and every `panes` field becomes `windows`: in event payloads, action targets, `gband.layout()`, `gband.view()`, the status line's render context, `gband.sessions()` and `gband.win.info()`.
- **BREAKING**: the plugin window names make room first. `gband.view().window` and the layout's `window` field become `plugin_window`. `gband.win.open`'s kinds `"float"` and `"pane"` become `"floating"` and `"tiled"`. The highlight groups `Window`, `WindowBorder`, `WindowTitle` and `WindowCursorLine` become `PluginWindow`, `PluginWindowBorder`, `PluginWindowTitle` and `PluginWindowCursorLine`. `gband.win` keeps its name.
- **BREAKING**: programs get `GBAND_WINDOW` in place of `GBAND_PANE`.
- The old Lua names are not kept as aliases. Using one is an error at the line of the use that names its replacement, so a 0.1.0 configuration fails loudly and says how to fix it.
- The CLI help, `list-sessions` output, error messages, the default configuration's key descriptions and the status line hints say window.
- The sample plugin `examples/plugins/pane` becomes `examples/plugins/window`, with the group `WindowSegment`. The other samples, the README, `docs/plugins.md`, `docs/testing.md`, `summary.txt` and the OpenSpec project context use the new names, and say floating plugin window where they said float. The README line that says some API names still say pane goes away.
- The Rust code follows: `Pane` becomes `Window`, `PaneId` becomes `WindowId`, and so on. The client's existing plugin window types, such as `Window` and `WindowRequest`, become `PluginWindow` and `PluginWindowRequest` first, so the names are free.
- The wire encoding does not change. Postcard encodes fields and variants by position, so the renamed identifiers produce the same bytes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `actions`, `animations`, `colorschemes`, `command-line`, `configuration`, `input-encoding`, `key-hints`, `layout`, `layout-view`, `lua-events`, `plugin-bridge`, `plugin-testing`, `plugins`, `server-runtime`, `session-events`, `session-server`, `status-line`, `terminal-emulator`, `test-channel`, `transport`, `wire-protocol`: every requirement that says pane says window, and the Lua names they quote take their new form.
- `client-attach`: the same, and the keys that go to a focused plugin window are named as such.
- `plugin-windows`: what `gband.win` draws is a plugin window throughout. Its kinds are floating and tiled, and its highlight groups take the `PluginWindow` prefix.
- `floating-panes`: renamed to `floating-windows`, and every floating pane is a floating window.
- `lua-control`: the same renames, and a new requirement that makes every removed pane name an error that names its replacement.

## Impact

- Every crate under `crates/`, `src/` and `tests/`, including file renames such as `crates/server/src/pane.rs` to `window.rs`, and the plugin window files `windows.rs` to `plugin_windows.rs`.
- The client snapshot tests and the sample plugins' screenshots whose names or contents say pane.
- `examples/plugins/`, `README.md`, `docs/`, `summary.txt`, `openspec/config.yaml` and every main spec under `openspec/specs/`.
- Users: a 0.1.0 `user/init.lua`, `user/server.lua` or plugin that uses a pane name stops loading, with an error naming the new name. A script that reads `GBAND_PANE` gets nothing. The release notes of the next release list the renames.
- Open changes: key-list, center-column, loop-bands, navigation-mode and sidebars-borders-steps depend on this change and are written with the new names. This change starts first.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- README.md
- crates/client/
- crates/core/
- crates/harness/
- crates/lua/
- crates/protocol/
- crates/server/
- crates/test-support/
- docs/plugins.md
- docs/testing.md
- examples/plugins/
- openspec/config.yaml
- openspec/specs/
- src/main.rs
- src/paths.rs
- summary.txt
- tests/
- openspec/changes/rename-panes-to-windows/
