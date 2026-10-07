## Why

A later Lua-only change, `floating-key-style`, adds a key style that works like a desktop such as MS Windows. Every window floats, and the user moves and resizes a window by dragging its border, and right-clicks the border for a menu. Clicks inside a window's content must keep the interactive-mode defaults, so the program inside keeps its mouse. The stable API blocks this. A gesture starts only from a mouse binding, and a binding of `leftmouse` in `root` replaces the default for every press, content clicks included. Lua also cannot tell where in a window's box a border press landed, so it cannot tell the top border from a side, a corner or a title-bar button.

## What Changes

- **Declining a mouse binding.** A function bound to a mouse name that returns `false` declines the press or wheel step. The event then takes exactly what it would take if the active table did not bind that name:
  - in `root`, the interactive-mode default, including a plugin window's `on_mouse` and its defaults;
  - in a mode, it is discarded and the mode stays active;
  - in a table that is neither `root` nor a mode, a press is discarded and the sequence ends;
  - in any table, a wheel step goes to its target and leaves the active table unchanged.
- The actions the declining function dispatched stand, and run before the fallback. A declined press starts no gesture, so a drag action that the function dispatched does nothing. A declined wheel step starts no 150 ms cooldown. Any other return value, nil included, keeps today's behaviour. A key binding's return value stays ignored.
- **The box cell in the mouse fields.** For the targets `"window"` and `"plugin_window"`, the mouse fields gain:
  - `box_col` and `box_row`: the cell within the target's box, border included, counted from 0 at the box's top-left cell;
  - `box_width` and `box_height`: the box as drawn in the frame the event was resolved against.

  They are nil for other targets. A plugin window's `on_mouse` table gains the same four fields, and so does the event that the windows provider's `mouse(id, event)` receives.
- **BREAKING (API version):** a function bound to a mouse name that returns `false` replaces the default today, and will stop replacing it. That changes a documented effect, so `gband.api_version` becomes `2`. Every bundled plugin, example plugin and tutorial plugin declares `api = 2`. The "API and stability" section of `docs/plugins.md` lists the change for version 2. No protocol change.

Out of scope:
- The floating key style itself, its title bar, and its context menu.
- Which edges `drag_resize_window` picks from a border press. The sibling change that owns "Resize by dragging" handles that.
- A return value for `on_mouse`, for event handlers, or for key bindings.
- `gband.keymap.run` with a mouse name. It runs no mouse event, so it ignores the return value as it does today.

## Capabilities

### New Capabilities
- none

### Modified Capabilities
- `mouse`: "Mouse names in key tables" defines declining and its fallback in each kind of table, the cooldown, and where the release and motions of a declined press go. "Interactive mode defaults" applies to a press that `root` declines. "Drag gestures" starts no gesture from a declining function. "Wheel" sends a declined wheel step to its target.
- `configuration`: "Binding functions" gives a function bound to a mouse name a meaningful return value, and keeps every other binding function's return value ignored. It also states that a registered action bound to a mouse name gets the mouse payload, as it already does.
- `lua-events`: "Built-in events" adds `box_col`, `box_row`, `box_width` and `box_height` to the mouse fields.
- `plugin-windows`: "Mouse in plugin windows" runs `on_mouse` for a press that `root` declines, and adds the four box fields to its table.
- `lua-api`: "API version rule" reads version 2 and lists its change.
- `plugins`: "Side and API version" sets `gband.api_version` to 2, and "Plugin modules" uses `api = 1` for its mismatch scenario.
- `server-runtime`: "Server Lua state" sets the server's `gband.api_version` to 2.

## Impact

- `crates/lua/src/runtime.rs`: `call_pressed` and `call_scrolled` report whether the function declined, through a new `Outcome::declined`. `API_VERSION` becomes 2.
- `crates/lua/src/events.rs` and `crates/lua/src/plugin_windows.rs`: `Pointer` and `PluginMouse` carry the box cell and size and write the four fields.
- `crates/client/src/bindings.rs`: `MouseCommand::Run` tells what an unbound press would take. `Leader::handle_wheel` no longer ends the sequence before the function has run.
- `crates/client/src/mouse.rs`: the fallback after a decline, no gesture and no cooldown for a declined event, and the box fields from the hit region.
- Bundled Lua: `api = 2` in `crates/lua/src/runtime/gband/errors.lua`, `keylist.lua` and `prompt.lua`. No other bundled Lua changes, because `gband.win` passes the provider's event table to `on_mouse` unchanged.
- Tests: the client and Lua crates' mouse tests, the end-to-end mouse spec, the plugin and version tests, and the tutorial chapter tests.
- Docs: `docs/plugins.md`, `docs/testing.md`, `README.md`, `docs/tutorial/01-keys.md`, `docs/tutorial/09-plugin.md`, `docs/internals/06-window-provider.md`, `docs/internals/09-sidebar-and-errors.md` and `docs/internals/10-keylist-and-prompt.md`.
- No protocol change and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- README.md
- crates/client/src/bindings.rs
- crates/client/src/mouse.rs
- crates/client/tests/actions.rs
- crates/client/tests/events.rs
- crates/lua/src/events.rs
- crates/lua/src/lib.rs
- crates/lua/src/plugin_windows.rs
- crates/lua/src/runtime.rs
- crates/lua/src/runtime/gband/errors.lua
- crates/lua/src/runtime/gband/keylist.lua
- crates/lua/src/runtime/gband/prompt.lua
- crates/lua/tests/events.rs
- crates/lua/tests/plugin_windows.rs
- crates/lua/tests/plugins.rs
- docs/internals/06-window-provider.md
- docs/internals/09-sidebar-and-errors.md
- docs/internals/10-keylist-and-prompt.md
- docs/plugins.md
- docs/testing.md
- docs/tutorial/01-keys.md
- docs/tutorial/09-plugin.md
- examples/plugins/hello/lua/hello/init.lua
- examples/plugins/window/lua/window/init.lua
- examples/tutorial/01-keys/tests/keys_spec.lua
- examples/tutorial/01-keys/user/init.lua
- examples/tutorial/09-plugin/plugins/marks/lua/marks/init.lua
- examples/tutorial/10-server/plugins/marks/lua/marks/init.lua
- examples/tutorial/11-testing/plugins/marks/lua/marks/init.lua
- openspec/changes/mouse-binding-fallthrough/
- tests/lua/mouse_spec.lua
- tests/mouse.rs
