## Why

A later change, `floating-key-style`, adds a key style in which every window floats and the user resizes a window by dragging its border, as on MS Windows. There a drag on a side border moves only that edge, and a drag on a corner, the corner cell or the cell beside it along either side, moves the two edges that meet there. Today `drag_resize_window` always picks the edges itself, from the third of the box that the press lies in, so a press on the bottom border near the left side moves both the left and the bottom edge. Lua cannot choose the edges, so the new key style cannot behave as a desktop does.

## What Changes

- `drag_resize_window` accepts an optional target, `{ edges = { ... } }`. `edges` is a list of edge names from `"left"`, `"right"`, `"top"` and `"bottom"`, and names exactly the edges the gesture moves, wherever the press lies in the box.
- The list holds at least one name, no name twice, not both `left` and `right`, and not both `top` and `bottom`. Any other list, an unknown name, a field other than `edges`, or a target that is not a table is an error at the line of the call, and nothing is dispatched.
- The named edges follow the pointer by the rules that already apply to each kind of box: a tile's width and height, with the camera rule for a tile's left edge; a floating window's box; and a floating plugin window's box, with its `on_resize`. A named top or bottom edge of a tile changes the window's height, as the layout's height rules define, whichever window of the column it is.
- Without a target, `drag_resize_window` picks the edges by thirds of the box, as today.
- The action still starts a gesture only while the client handles a mouse press. A call with a valid target at any other time does nothing.
- A target on `drag_window` or `drag_band` stays an error, and the specs now say so by name.
- No default binding changes. This change does not raise `gband.api_version`, since it only adds an optional argument.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `mouse`: "Resize by dragging" moves exactly the edges a target names, and picks them by thirds only without a target.
- `lua-control`: "Action targets" adds the `edges` target of `drag_resize_window` and its errors, and names the client actions that take no target.

## Impact

- `crates/core/src/action.rs`: an `Edges` type, and `ClientAction::DragResize` carries the edges a target names.
- `crates/lua/src/control.rs` and `crates/lua/src/actions.rs`: reading and checking the `edges` target.
- `crates/client/src/mouse.rs`, `crates/client/src/lib.rs` and `crates/client/src/bindings.rs`: a gesture uses the named edges instead of picking them.
- Tests in `crates/lua/tests/`, `crates/client/tests/actions.rs` and `tests/lua/mouse_spec.lua`.
- `docs/plugins.md`, `README.md` and `docs/tutorial/02-actions.md`.
- No protocol, server or default configuration change, and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- README.md
- crates/client/src/bindings.rs
- crates/client/src/lib.rs
- crates/client/src/mouse.rs
- crates/client/tests/actions.rs
- crates/core/src/action.rs
- crates/lua/src/actions.rs
- crates/lua/src/control.rs
- crates/lua/tests/config.rs
- crates/lua/tests/control.rs
- crates/lua/tests/keymap.rs
- crates/lua/tests/plugin_windows.rs
- docs/plugins.md
- docs/tutorial/02-actions.md
- tests/lua/mouse_spec.lua
- openspec/changes/drag-resize-edges/
