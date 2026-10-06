## Why

Scripts can fill the status line, but they cannot take any other part of the screen, and the status line's rows are placed by the client itself. Every window border is a hardcoded box. The grow and shrink steps are fixed at one tenth of the screen area. Each client should be able to shape its own screen without changing the size of any window.

## What Changes

- **Side bars**: a new client API, `gband.bar`, lets Lua reserve columns at the left or right of the terminal and write styled lines there.
  - Bars on one side stack from the terminal's edge inward by `order`.
  - The ribbon area becomes the columns the bars leave, at the terminal's full height.
  - Bars never change the reported size, the screen area or any window's size. Tiles keep their sizes, and the camera scrolls inside the narrower ribbon.
  - Top and bottom bars are not offered, because they would change window heights.
  - Each client has its own bars.
- **Status line as a side bar**: the status line becomes a left bar that a bundled plugin, `gband.statusline`, adds when it is set up.
  - Setup options: `side` (`"left"` or `"right"`), `min_width`, `max_width` and `order`.
  - Its width follows the widest segment, between `min_width` (20 by default) and `max_width` (40 by default).
  - Segments stack vertically, one row per line, in `top`, `center` and `bottom` groups. When rows run out, the lowest priority is dropped first.
  - A component may return several lines. The hints segment wraps its hints into lines at the bar's width.
  - Leaving the plugin out removes the status line.
- **Errors**: while an error is reported, the status line shows the word `error` in red instead of the message.
  - `gband.errors()` returns the client's error list.
  - A new bundled plugin, `gband.errors`, registers the action and command `errors.open`. They open the error list in a plugin window, floating by default or tiled with `kind = "tiled"`. `kind` is a setup option and a field of the command's arguments table, as in `gband.cmd.run("errors.open", { kind = "tiled" })`.
  - Without a status line, the latest error still shows on the ribbon's bottom row.
- **BREAKING**: the client options `statusline_position`, `statusline_height` and `statusline_separator` are removed. A user file that still sets them gets a configuration error naming the option, and loads anyway.
- **BREAKING**: component `align` takes `"top"`, `"center"` or `"bottom"` in place of `"left"`, `"center"` or `"right"`. The bundled segments and the example plugins move to the new values. Segments no longer have separators.
- **BREAKING**: the default status line moves from the bottom row to a 20-column left bar.
- **Border drawing per client**: new client options choose which sides of a border are drawn and with which characters:
  - `tile_border_sides` and `tile_border_chars` apply to tiled windows.
  - `floating_border_sides` and `floating_border_chars` apply to floating windows.

  Sides are any of `top`, `right`, `bottom` and `left`. Characters are `"plain"`, `"rounded"`, `"double"` or `"thick"`, or a list of eight one-cell strings. A side that is not drawn still takes its cell, which is left blank. Window sizes therefore never depend on border settings, and two clients with different borders show the same windows.
- **Floating plugin window borders**: the `border` field of a floating plugin window also accepts a table `{ sides =, chars = }`, with the same meaning.
- **Resize steps per client**: new client options `width_step` and `height_step` set how much grow and shrink change a column's width and a window's height.
  - Both default to 1/10, today's step.
  - Each grow or shrink request carries its step, so two clients can use different steps. The resized window is shared by every client as before.
  - Lua action targets accept `step` for the four grow and shrink actions. An action from the server's Lua uses 1/10 unless its target names a step.
- **BREAKING**: the grow and shrink messages carry a step, so the protocol version goes up.
- Width presets, which `prefix r` cycles through, are already a script option, `width_presets`. They are unchanged.

Out of scope:
- Top and bottom bars, bars that float over the ribbon, and bars that change window sizes.
- A default key binding for `errors.open`, which would change the client-attach default key table that two open changes already edit. Users bind it themselves for now.
- Border thickness other than one cell, and highlight groups for window borders. The focused and other borders keep today's styles.
- Steps for moving a floating window.

## Capabilities

### New Capabilities
- `bars`: side bars. Covers the `gband.bar` API, placing bars and the ribbon area they leave, keeping window sizes, bar contents and drawing, resize callbacks, reloads, plugin ownership, and the `Bar` highlight group.
- `borders`: how a border's sides and characters are chosen and drawn. Covers the border options of tiled and floating windows and the border table of a floating plugin window.
- `error-list`: the bundled `gband.errors` plugin, its action and command, and the plugin window that lists the client's errors.

### Modified Capabilities
- `status-line`: the status line is a side bar added by the bundled `gband.statusline` plugin. Components are laid out vertically and may return several lines. The context gains `total_height` and `height`. The error item shows `error`.
- `key-hints`: the hints segment aligns to the top and wraps its hints into lines at the available width and height, so a hint label scenario that shows hints on one line asks for a segment wide enough for them.
- `configuration`: the status line options are removed and the border and step options are added. The client keeps an error list, read with `gband.errors()`. The default configuration sets up `gband.errors` and `gband.statusline`.
- `client-attach`: the reported size is always the terminal size. The ribbon area is what the bars leave, and tiles keep their sizes inside it. Tiles and floating windows are drawn with the client's border options. Scenarios of sending input and of the key bindings that counted the status line's row, or named screen columns with the default status line, follow the left bar.
- `actions`: grow and shrink are sent with the client's step, or a target's.
- `lua-control`: grow and shrink action targets accept `step`.
- `layout`: growing and shrinking a column's width or a window's height use the step the request names.
- `floating-windows`: growing and shrinking a floating window's box use the step the request names. A new box stays two tenths of the area shorter than the area, whatever the step.
- `plugin-windows`: a floating plugin window's `border` accepts a border table.
- `server-runtime`: the server's grow and shrink action targets accept `step`, with the same ranges as the client's.
- `plugins`: `bar` and `errors` are client-only fields of `gband`.
- `wire-protocol`: grow and shrink actions carry a step, and the protocol version goes up by one.
- `plugin-testing`: the case environment's configuration scenario sets up the status line on the right instead of setting `statusline_position`.

## Impact

- Lua runtime: new `gband/bar.lua`; `gband/statusline.lua` becomes the `gband.statusline` plugin with a vertical layout drawn through a bar; the segment plugins and `hints.lua` move to vertical alignment and wrapping; new `gband/errors.lua`; `gband/win.lua` reads border tables; `defaults.lua` sets up the new plugins; the default colorscheme sets `Bar`.
- Lua host (Rust): bar placement and presentation, the error list and `gband.errors()`, removal of the status line options, the border and step options, and `step` in action targets.
- Client: `placement.rs` places side bars instead of a status line; `lib.rs` reports the terminal size and offsets the ribbon area from the left; `render.rs` draws bars and draws borders from sides and characters; grow and shrink requests carry steps.
- Core, protocol and server: step amounts in the grow and shrink actions, the protocol version, and the server applying the received step.
- Docs and examples: `README.md`, `docs/plugins.md` and the example plugins move to side bars, vertical alignment, the error list, borders and steps.
- Tests: Lua, client and end-to-end status line tests move to the side-bar status line. Tests that set the removed options, read the status line on the bottom row, or name screen columns of the default configuration move too, and so do the Lua screen tests and the example plugins' tests and screenshots.

## Coordination

### Author
- gmteixeira

### Depends On
- floating-windows
- loop-bands
- navigation-mode
- rename-panes-to-windows

### Expected Files
- openspec/changes/sidebars-borders-steps/
- crates/core/src/action.rs
- crates/core/src/layout.rs
- crates/core/tests/layout.rs
- crates/core/tests/events.rs
- crates/core/tests/view.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/tests/resize.rs
- crates/server/tests/drawn_windows.rs
- crates/server/tests/sync.rs
- crates/lua/src/actions.rs
- crates/lua/src/api.rs
- crates/lua/src/bars.rs
- crates/lua/src/border.rs
- crates/lua/src/bundled.rs
- crates/lua/src/control.rs
- crates/lua/src/keymap.rs
- crates/lua/src/defaults.lua
- crates/lua/src/lib.rs
- crates/lua/src/options.rs
- crates/lua/src/runtime.rs
- crates/lua/src/sides.rs
- crates/lua/src/ui.rs
- crates/lua/src/plugin_windows.rs
- crates/lua/src/runtime/gband/
- crates/lua/tests/
- crates/client/src/bindings.rs
- crates/client/src/connect.rs
- crates/client/src/lib.rs
- crates/client/src/placement.rs
- crates/client/src/render.rs
- crates/client/tests/actions.rs
- crates/client/tests/events.rs
- crates/client/tests/plugin_windows.rs
- crates/client/tests/bars.rs
- crates/client/tests/render.rs
- crates/client/tests/statusline.rs
- crates/client/tests/snapshots/
- tests/statusline.rs
- tests/plugin_windows.rs
- tests/attach.rs
- tests/navigation.rs
- tests/config.rs
- tests/floating.rs
- tests/plugin_runtime.rs
- tests/plugin_testing.rs
- tests/lua/statusline_spec.lua
- tests/lua/screenshots/statusline_spec/
- tests/lua/keylist_spec.lua
- tests/lua/screenshots/keylist_spec/
- tests/lua/loop_bands_spec.lua
- README.md
- docs/plugins.md
- docs/testing.md
- examples/plugins/agent-status/client.lua
- examples/plugins/agent-status/tests/agent_status_spec.lua
- examples/plugins/agent-status/tests/screenshots/agent_status_spec/a-prompt-marks-the-window-waiting.txt
- examples/plugins/hello/tests/screenshots/hello_spec/greet-opens-a-window-that-prints-the-greeting.txt
- examples/plugins/window/lua/window/init.lua
- examples/plugins/window/README.md
- examples/plugins/window/tests/window_spec.lua
- examples/plugins/window/tests/screenshots/window_spec/
