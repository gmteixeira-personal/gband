## Why

gband follows niri, and niri calls the things on its strips windows. gband called them panes, the word tmux and Zellij use for the regions they split a screen into. Since the amended v0.1.0 README, the docs call them windows and call what `gband.win` draws plugin windows. The code, the Lua API, the specs and the sample plugins still say pane, so the docs have to explain that the API names disagree with them. This change makes every name agree with the docs.

## What Changes

- The specs call every pane a window. A pane window, the `gband.win` window that sits in the layout, becomes a tiled plugin window. A plugin pane, the window in the layout that shows one and runs no program, becomes a drawn window. Every other use of "window" for what `gband.win` draws becomes "plugin window". Requirement names follow, such as "Pane program" becoming "Window program" and "Window API" becoming "Plugin window API", and so do scenario titles and Purpose sections.
- **BREAKING**: the Lua actions `open_pane`, `close_pane`, `focus_pane_down`, `focus_pane_up`, `grow_pane_height`, `shrink_pane_height` and `reset_pane_height` become `open_window`, `close_window`, `focus_window_down`, `focus_window_up`, `grow_window_height`, `shrink_window_height` and `reset_window_height`. The names that the open changes add follow the same pattern, such as `toggle_pane_floating` becoming `toggle_window_floating`.
- **BREAKING**: `gband.pane` becomes `gband.window`, and `gband.pane_state` becomes `gband.window_state`.
- **BREAKING**: the events `PaneOpened`, `PaneClosed`, `PaneExited`, `PaneOutput`, `PaneInput` and `PaneStateChanged` become `WindowOpened`, `WindowClosed`, `WindowExited`, `WindowOutput`, `WindowInput` and `WindowStateChanged`. Every `pane` field becomes `window` and every `panes` field becomes `windows`: in event payloads, action targets, `gband.layout()`, `gband.view()`, the status line's render context, `gband.sessions()` and `gband.win.info()`.
- **BREAKING**: the plugin window names make room first. `gband.view().window` and the layout's `window` field become `plugin_window`. `gband.win.open`'s kind `"pane"` becomes `"tiled"`. The highlight groups `Window`, `WindowBorder`, `WindowTitle` and `WindowCursorLine` become `PluginWindow`, `PluginWindowBorder`, `PluginWindowTitle` and `PluginWindowCursorLine`. `gband.win` keeps its name.
- **BREAKING**: programs get `GBAND_WINDOW` in place of `GBAND_PANE`.
- The old Lua names are not kept as aliases. Using one is an error at the line of the use that names its replacement, so a 0.1.0 configuration fails loudly and says how to fix it.
- The CLI help, `list-sessions` output, error messages, the default configuration's key descriptions and the status line hints say window.
- The sample plugin `examples/plugins/pane` becomes `examples/plugins/window`, with the group `WindowSegment`. The other samples, the README, `docs/plugins.md`, `docs/testing.md`, `summary.txt` and the OpenSpec project context use the new names. The README line that says some API names still say pane goes away.
- The Rust code follows: `Pane` becomes `Window`, `PaneId` becomes `WindowId`, and so on. The client's existing plugin window types, such as `Window` and `WindowRequest`, become `PluginWindow` and `PluginWindowRequest` first, so the names are free.
- The wire encoding does not change. Postcard encodes fields and variants by position, so the renamed identifiers produce the same bytes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `actions`, `animations`, `colorschemes`, `command-line`, `configuration`, `input-encoding`, `key-hints`, `layout`, `layout-view`, `lua-events`, `plugin-bridge`, `plugin-testing`, `plugins`, `server-runtime`, `session-events`, `session-server`, `status-line`, `terminal-emulator`, `test-channel`, `transport`, `wire-protocol`: every requirement that says pane says window, and the Lua names they quote take their new form.
- `client-attach`: the same, and the keys that go to a focused plugin window are named as such.
- `plugin-windows`: what `gband.win` draws is a plugin window throughout. Its kinds are float and tiled, and its highlight groups take the `PluginWindow` prefix.
- `lua-control`: the same renames, and a new requirement that makes every removed pane name an error that names its replacement.

## Impact

- Every crate under `crates/`, `src/` and `tests/`, including file renames such as `crates/server/src/pane.rs` to `window.rs`, and the plugin window files `windows.rs` to `plugin_windows.rs`.
- The client snapshot tests and the sample plugins' screenshots whose names or contents say pane.
- `examples/plugins/`, `README.md`, `docs/`, `summary.txt`, `openspec/config.yaml` and every main spec under `openspec/specs/`.
- Users: a 0.1.0 `user/init.lua`, `user/server.lua` or plugin that uses a pane name stops loading, with an error naming the new name. A script that reads `GBAND_PANE` gets nothing. The release notes of the next release list the renames.
- Open changes: floating-windows, navigation-mode, center-column, loop-bands and sidebars-borders-steps add or quote pane names. This change starts after all five are archived, and renames what they added.

## Coordination

### Author
- gmteixeira

### Depends On
- floating-windows
- navigation-mode
- center-column
- loop-bands
- sidebars-borders-steps

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
