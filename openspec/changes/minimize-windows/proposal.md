## Why

A later Lua-only change, `floating-key-style`, adds a third key style that works like a desktop such as MS Windows: every window floats, each has a minimize button, and a window list shows every window and restores a minimized one when clicked. Today nothing can hide a window without closing it: the layout has no hidden state, and `gband.window.set_position` keeps a floating box inside the screen area. This change adds the hidden state and the Lua surface that the key style builds on.

## What Changes

- **Minimized windows, per client**: each client's view holds a set of floating windows it has minimized, as it holds its own stacking order. A minimized window is not drawn by that client, is not among its shown windows, is not a pointer target, and is out of its stacking order. Minimizing changes nothing in the shared layout, no PTY size and nothing in another client's view or drawing, and sends the server no action. A client starts each attach with no minimized window; a reload keeps the set.
- **Only floating windows**: the `minimize_window` action on a tiled window does nothing. `gband.window.minimize` on a tiled window is an error at the line of the call, as `gband.window.set_position` is.
- **Focus skips minimized windows**: focus between floating windows, switching layers, entering a band and the fallback after the focused floating window leaves all skip them. A floating layer whose windows this client has all minimized counts as holding no window. Minimizing the focused window moves focus as when the focused floating window leaves the layout.
- **Restore by focus**: focusing a minimized window by any means (`gband.window.focus`, a click on a window list entry that calls it, a focus message from the server) restores it and puts it on top of this client's stacking order. Tiling it, or its leaving the layout, also takes it out of the set. No separate restore function is added.
- **New view action** `minimize_window`, described as `minimize the focused floating window`. Neither the modal nor the direct key style binds it: they have no window list, so a minimized window would have no built-in way back. `floating-key-style` binds it.
- **New Lua function** `gband.window.minimize(window)`, dispatched like `gband.window.focus`. A window not in the client's layout or not floating there is an error at the line of the call. A window already minimized is left as it is.
- **Reading the state**: each table in a band's `floating` list from `gband.layout()` holds `minimized = true` when this client has minimized that window.
- `gband.api_version` does not rise for this change: it only adds an action, a function and a field, which the lua-api capability's "API version rule" lets a build add without a new version.
- No new built-in event and no wire protocol change.

Out of scope:
- The window list, the minimize button and the `floating-key-style` key style itself.
- Minimizing tiled windows, sharing the minimized set between clients, and keeping it across a detach and a later attach.
- A built-in event for minimize and restore. The built-in event list belongs to another change; see design.md.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `floating-windows`: a new requirement, "Minimize a floating window", defines the per-client set, what it hides, what restores a window and what leaves the set. "Layers of a view", "Switch layers", "Focus between floating windows", "Focus follows a window between layers" and "Stacking" skip minimized windows.
- `layout-view`: "Per-client view" holds the minimized set and allows no focus while only minimized windows remain. "Initial view" starts with none. "Switch band" skips a minimized window. "Shown windows" leaves minimized windows out. "Focus a named window" restores a minimized window.
- `mouse`: "Pointer targets" skips the boxes of minimized windows.
- `lua-control`: "Read the layout" adds `minimized` to floating tables. "Dispatch order" names `gband.window.minimize`. A new requirement, "Minimize a window by number", defines `gband.window.minimize`.
- `actions`: "Lua names of actions" adds `minimize_window` and its description.
- `animations`: "Floating windows at rest" draws no minimized window, and minimizing and restoring do not animate.
- `client-attach`: "Ribbon presentation" draws no floating window the client has minimized.

## Impact

- `crates/core/src/view.rs`: the minimized set in `View`, the `ViewAction::Minimize` variant, and every focus, stacking and shown-window helper skipping minimized windows.
- `crates/client/src/lib.rs`: the minimized set in the Lua view state, and ending a gesture or selection on a window this client minimizes.
- `crates/lua/src/actions.rs`, `control.rs`, `ui.rs`: the `minimize_window` action, `gband.window.minimize`, and the `minimized` field of `gband.layout()`.
- Tests: `crates/core/tests/view.rs`, `crates/client/tests/actions.rs`, `render.rs`, `events.rs` and `animation.rs`, `crates/lua/tests/control.rs` and `config.rs`, `crates/server/tests/resize.rs`, and a new Lua spec, `tests/lua/minimize_spec.lua`.
- Docs: `README.md`, `docs/plugins.md` and `docs/tutorial/05-layout.md`.
- No server code, protocol or bundled Lua change.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- README.md
- crates/client/src/lib.rs
- crates/client/tests/actions.rs
- crates/client/tests/animation.rs
- crates/client/tests/events.rs
- crates/client/tests/render.rs
- crates/client/tests/snapshots/render__minimized_floating_window_is_not_drawn.snap
- crates/core/src/view.rs
- crates/core/tests/view.rs
- crates/lua/src/actions.rs
- crates/lua/src/control.rs
- crates/lua/src/ui.rs
- crates/lua/tests/config.rs
- crates/lua/tests/control.rs
- crates/server/tests/resize.rs
- docs/plugins.md
- docs/tutorial/05-layout.md
- openspec/changes/minimize-windows/
- tests/lua/minimize_spec.lua
- tests/lua/screenshots/minimize_spec/focus-restores-a-minimized-window--restored.txt
- tests/lua/screenshots/minimize_spec/minimizing-hides-the-floating-window--hidden.txt
- tests/lua_specs.rs
