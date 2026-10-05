## Why

Every pane in gband sits on a band's strip. A pane that should stay in sight while the strip scrolls, such as a log tail, a calculator or a quick shell, has no place to go. niri solves this with a floating layer drawn over each workspace's scrolling layout, with one key to float or tile a window and one key to move focus between the two layers. gband follows niri where its specs are silent, so it should gain the same layer. Scripters need the same control over floating panes that they have over tiled ones.

## What Changes

- **Floating panes**: each band gains a floating layer, drawn over its strip. A floating pane is a full pane with its own program and PTY, placed in a box in the screen area that does not scroll with the camera. Its box has a position, a width that is a proportion of the screen area's width, a full-width flag, and a height in rows.
- **Float and tile a pane**: a new session action, `toggle_pane_floating`, moves a pane between the strip and the floating layer. A pane floated for the first time keeps its tile's size and is centred in the screen area. A pane floated again returns to the box it last had. A pane tiled again gets a new column right of the client's tiled focus, at its floating width. The default key is Ctrl+Space then `v`.
- **Switch layers**: a new view action, `switch_focus_floating_tiled`, moves this client's focus between the floating layer and the tiled layer of the viewed band, as niri's `switch-focus-between-floating-and-tiling` does. The default key is Ctrl+Space then `V`.
- **One set of operations for both kinds**: the existing actions apply to floating panes too.
  - The four focus actions move focus between floating panes by direction while the floating layer has focus.
  - The width actions change a floating pane's box width by the same rules as a column's width.
  - The height actions change its box height by the same step as a tiled pane's height.
  - Close works unchanged.
  - Consume or expel changes nothing on a floating pane, as in niri.
- **Move actions, for both kinds**: four new session actions, as in niri.
  - `move_column_left` and `move_column_right` swap a tiled pane's column with its neighbour.
  - `move_pane_down` and `move_pane_up` swap a tiled pane with its neighbour in the column.
  - On a floating pane, each of the four moves the box one step in its direction, kept inside the screen area.
  - The default keys are Ctrl+Space then Ctrl+H, Ctrl+L, Ctrl+J and Ctrl+K, and Ctrl+Space then Ctrl with the left, right, down and up arrow keys.
- **Stacking**: each client draws the floating panes it has focused above the others, the most recently focused on top. Floating panes are drawn over the tiles and under the client's plugin floats.
- **Lua**:
  - `gband.action` gains the six new actions, and they all accept targets.
  - `open_pane` accepts `floating = true`.
  - `gband.pane.set_position(pane, { col =, row = })` places a floating pane exactly.
  - `gband.layout()` lists each band's floating panes with their boxes.
  - `gband.view()` reports whether the focused pane floats.
- **Events**: new layout events report a pane floated, a pane tiled, a floating box changed, and a column moved.
- **BREAKING**: the layout message carries the floating layer, so the protocol version becomes 6. Clients and servers of version 5 refuse each other, as the handshake defines.

Out of scope:
- Opening a `gband.win` pane window directly as floating. A plugin pane can be floated afterwards like any pane.
- Moving a pane to another band, and window rules that open panes floating by program.
- Mouse dragging and animating floating boxes. Floating panes are drawn at rest.
- Arrow-key forms of the existing focus bindings. They stay a separate change.

## Capabilities

### New Capabilities
- `floating-panes`: the floating layer of a band. It covers floating and tiling a pane, the box and its geometry, size and move operations on a floating pane, exact placement, focus between floating panes and between the two layers, and stacking.

### Modified Capabilities
- `layout`: a band also holds floating panes, and a band is empty only when it holds no tiled and no floating pane. The open pane action can open a floating pane. Columns and stacked panes can be moved.
- `layout-view`: the view tracks a layer for each band, shows floating panes, resolves open pane against the tiled focus, and follows panes that change layer.
- `actions`: the six new built-in actions and their descriptions, and how the new actions resolve against the view.
- `session-server`: the new session actions.
- `session-events`: the new layout events.
- `wire-protocol`: the layout message carries floating panes, the new client actions, and protocol version 6.
- `client-attach`: floating panes are drawn between the tiles and the floats, and the default key table gains `v`, `V`, Ctrl+H, Ctrl+J, Ctrl+K, Ctrl+L and the Ctrl arrow keys.
- `key-hints`: short labels for the new actions.
- `lua-control`: floating panes in `gband.layout()` and `gband.view()`, targets of the new actions, `open_pane`'s `floating` field, and `gband.pane.set_position`.
- `animations`: floating panes are drawn at rest, and a pane changing layer does not animate.

## Impact

- `crates/core/src/layout.rs`, `geometry.rs`, `view.rs`, `action.rs`, `event.rs`: the floating layer, box geometry, move operations, layer focus and the new actions and events.
- `crates/core/tests/`: layout, geometry, view and event tests.
- `crates/protocol/src/message.rs` and `crates/protocol/tests/messages.rs`: protocol version 6 and the new actions.
- `crates/server/src/session.rs`: PTY sizes of floating panes, and the focus reply for a pane opened floating.
- `crates/client/src/render.rs`, `lib.rs`, `animation.rs`: drawing floating panes, stacking, shown panes and cursor.
- `crates/lua/src/actions.rs`, `control.rs`, `defaults.lua`, `runtime/gband/statusline/hints.lua`: the new actions, targets, layout and view fields, `set_position`, default bindings and hint labels.
- `tests/`: end-to-end coverage of floating panes through the binary.
- `README.md` and `docs/`: the action table and Lua reference.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/core/src/layout.rs
- crates/core/src/geometry.rs
- crates/core/src/view.rs
- crates/core/src/action.rs
- crates/core/src/event.rs
- crates/core/tests/layout.rs
- crates/core/tests/geometry.rs
- crates/core/tests/view.rs
- crates/core/tests/events.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/src/session.rs
- crates/server/src/event.rs
- crates/server/tests/resize.rs
- crates/server/tests/panes.rs
- crates/server/tests/events.rs
- crates/client/src/render.rs
- crates/client/src/lib.rs
- crates/client/src/animation.rs
- crates/client/tests/render.rs
- crates/client/tests/animation.rs
- crates/client/tests/actions.rs
- crates/client/tests/snapshots/
- crates/lua/src/actions.rs
- crates/lua/src/control.rs
- crates/lua/src/defaults.lua
- crates/lua/src/runtime/gband/statusline/hints.lua
- crates/lua/tests/config.rs
- crates/lua/tests/control.rs
- crates/lua/tests/key_hints.rs
- tests/floating.rs
- README.md
- docs/plugins.md
- openspec/changes/floating-windows/
