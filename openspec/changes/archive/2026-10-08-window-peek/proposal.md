## Why

A later Lua-only change, `desktop-list-tweaks`, gives the floating key style's window list a live preview and orders it by recent use. Moving the selection, with a key or by hovering a line, shows the selected window at once, on top, even when it is minimized or in another band. Escape returns exactly to the state before the list opened. The stable Lua API blocks this:

- Focusing a window raises it, records it in this client's stacking order and restores it when it is minimized. Lua cannot read the stacking order, so Escape cannot put it back.
- Lua cannot read when this client last focused each window, so it cannot order the list by recent use.
- A floating plugin window's `on_mouse` gets no event when the pointer moves over it with no button held, so hover cannot move the selection.
- A focus change that a floating plugin window's `on_mouse` causes is not specified to keep that plugin window focused, so a preview would take the keys away from the list.

## What Changes

- **Peek**: `gband.window.focus(window, { peek = true })` dispatches a new view action. It focuses the window as a focus does: it views the window's band, shows its layer as active, moves the camera and emits `FocusChanged`. It records nothing: the focus order, the stacking order, the window each band remembers in each layer, the layer each band remembers and the set of minimized windows stay as they were. While a floating window is peeked, this client draws it above every other floating window of its band, also when it is minimized, and it is a shown window and a pointer target. A peeked tiled window is drawn as the focused tile is.
- **End of a peek**: the peek ends with a new, recorded focus when the view focuses any window by other means, when another window is peeked, or when the viewed band changes. A normal focus of the peeked window records the focus, raises and restores the window, and emits no `FocusChanged`, since the focused window does not change. The peek ends with no new focus when the client views the band it already views by name, when this client minimizes the peeked window, when the peeked window leaves the layout, and when a reload loads a new configuration, which closes every plugin window. Focus then returns to the window this client last focused in the viewed band and records nothing. The window returns to its place in the stacking order, or is hidden again when it is minimized. An attach starts with no peek.
- **`gband.view()`** holds `peek = true` while the focused window is peeked, and no `peek` otherwise.
- **Options of `gband.window.focus`**: an optional table that holds at most `peek`, a boolean. Another field, a `peek` that is not a boolean, or an argument that is not a table is an error at the line of the call.
- **Focus order**: each client's view holds a focus counter, 0 at attach, that rises by one each time it records a focus of a window. Each window table and each floating table of `gband.layout()` holds `last_focus`, the counter's value at the window's last recorded focus, and no `last_focus` for a window this client never focused. A band's floating stacking order is: the windows never focused in floating-list order, then the others by ascending `last_focus`. A reload keeps the counter.
- **Hover, opt-in**: `gband.win.open` takes an optional boolean `hover`, for both kinds, `false` by default. Only a plugin window opened with `hover = true` gets `on_mouse` with the kind `"move"`, for each motion with no button held to another cell over it, in any key table or mode, with the same fields as other kinds and no `button`. `gband.win.info` reports `hover`. `gband.win.set_config` does not take it. The frame that `gband.core.present_window` takes gains an optional `hover`, and the client reports moves to the windows provider's `mouse` only for a plugin window whose frame sets it. No built-in event is added: a motion with no button held still emits none.
- **Focus kept after `on_mouse` of a hover window**: for a floating plugin window opened with `hover = true`, the rule that keeps it focused after one of its `keys` functions changes focus or the band also covers its `on_mouse`, for every kind of event.
- Every change is an addition. A plugin window without `hover`, and a windows provider that never sets `hover` in a frame, behave exactly as today, so `gband.api_version` stays 2. No wire protocol change.

Out of scope:
- The window list's preview, its order and its Escape, which `desktop-list-tweaks` builds on this API, with `hover = true` on its menus.
- A `MouseMoved` built-in event.
- Restoring the camera when a peek of a tiled window ends.
- Sending a motion with no button held only to the target under the pointer: it still also reaches the focused window's program, as today.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `layout-view`: "Per-client view" holds the focus order and the peek. New requirements "Focus order" and "Peek a window". "Shown windows" shows a peeked minimized window. "Follow layout changes" sends focus as "Peek a window" does when the peeked window leaves. "View a named band" ends a peek on the band already viewed.
- `floating-windows`: "Layers of a view" shows the peeked window's layer only while the peek lasts. "Focus follows a window between layers" leaves a peeked window to "Peek a window". "Stacking" reads the focus order and draws the peeked window on top. "Minimize a floating window" draws and targets a peeked minimized window, does not restore it on a peek, and ends the peek when the peeked window is minimized.
- `mouse`: "Pointer targets" includes a peeked minimized window, on top of the stacking order.
- `client-attach`: "Ribbon presentation" draws a peeked minimized window.
- `animations`: "Floating windows at rest" shows and hides a peeked minimized window at once.
- `lua-control`: "Read the layout" adds `last_focus` and states the stacking order. "Read the view" adds `peek`. "Focus and view by number" adds the options of `gband.window.focus`. "Minimize a window by number" restores only on a focus that is not a peek.
- `plugin-windows`: "Focused plugin window" keeps a hover window focused after its `on_mouse`. "Mouse in plugin windows" adds the option `hover` and the kind `"move"`. "Plugin window information" adds `hover`.

## Impact

- `crates/core/src/view.rs`: `ViewAction::Peek`, the peek in `View`, a public focus order, a `hidden` predicate, and the end of a peek in every focus path and on a reload.
- `crates/client/src/lib.rs`: `last_focus` and `peek` in the Lua view state, hidden windows read from the view, and the end of a peek on a successful reload. `crates/client/src/plugin_windows.rs` and `mouse.rs`: which plugin windows take hover, and a motion with no button held over one of them runs the provider's `mouse` with `"move"`.
- `crates/lua/src/control.rs`, `ui.rs` and `plugin_windows.rs`: the options of `gband.window.focus`, `last_focus`, `peek`, the frame's `hover` and `PluginMouseKind::Move`.
- Bundled Lua: `gband.win` takes `hover`, puts it in its frames and `gband.win.info`, and holds a focused hover window for every `on_mouse` kind. No other bundled Lua changes; `desktop-list-tweaks` sets `hover` on the desktop menus.
- Tests: core view, client actions, render, events, plugin windows and animation, Lua control and plugin windows tests, and a new Lua spec, `tests/lua/peek_spec.lua`.
- Docs: `docs/plugins.md`, `README.md`, `docs/tutorial/05-layout.md` and `07-plugin-windows.md`, and `docs/internals/05-plugin-windows.md` and `06-window-provider.md`.
- No server code, protocol, dependency or API version change.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- README.md
- crates/client/src/lib.rs
- crates/client/src/mouse.rs
- crates/client/src/plugin_windows.rs
- crates/client/tests/actions.rs
- crates/client/tests/animation.rs
- crates/client/tests/events.rs
- crates/client/tests/plugin_windows.rs
- crates/client/tests/render.rs
- crates/client/tests/snapshots/render__peeked_minimized_floating_window_is_drawn_on_top.snap
- crates/core/src/view.rs
- crates/core/tests/view.rs
- crates/lua/src/control.rs
- crates/lua/src/plugin_windows.rs
- crates/lua/src/runtime/gband/win.lua
- crates/lua/src/ui.rs
- crates/lua/tests/control.rs
- crates/lua/tests/plugin_windows.rs
- docs/internals/05-plugin-windows.md
- docs/internals/06-window-provider.md
- docs/plugins.md
- docs/tutorial/05-layout.md
- docs/tutorial/07-plugin-windows.md
- openspec/changes/window-peek/
- tests/lua/peek_spec.lua
- tests/lua/screenshots/peek_spec/a-peek-shows-a-minimized-window-on-top--peeked.txt
- tests/lua/screenshots/peek_spec/hovering-a-line-peeks-its-window--hovered.txt
- tests/lua_specs.rs
