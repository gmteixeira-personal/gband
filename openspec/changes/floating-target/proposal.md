## Why

A later Lua-only change, `floating-key-style`, adds a key style in which every window floats. It must float any tiled window that appears, wherever it came from. Two gaps in the stable API block it. `toggle_window_floating` only toggles, so when two floating-style clients of one session both float the same new window, the second request tiles it again. And no event tells a client that the session's screen area changed size because another client resized its terminal, so a style that keeps a window maximized or tiled to half the screen cannot fit it to the new area.

## What Changes

- `toggle_window_floating`'s target gains an optional boolean `floating`. `floating = true` floats `window` when it is tiled and changes nothing when it already floats. `floating = false` tiles `window` when it floats, after `after` or by today's rule, and changes nothing when it is already tiled. Without `floating`, the action toggles as today.
- The server applies the request, so it is race-free. Two requests to float the same window float it once, in any order and from any client or the server's Lua.
- The client sends the request whatever layer its own layout shows. It does not skip a request that its layout already satisfies, because that layout can be older than the server's.
- A `floating` that is not a boolean, and `after` together with `floating = true`, are errors at the line of the call. A target without `window` stays an error.
- The server's `gband.action.toggle_window_floating` takes the same optional `floating`, with the same errors.
- The toggle floating session action carries an optional layer on the wire. **BREAKING**: the protocol version becomes 11. Clients and servers of version 10 refuse each other, as the handshake defines.
- `LayoutChanged` also runs when the size of the screen area in a layout the client receives differs from the previous layout's: the `cols` and `rows` that `gband.layout()` reports. This covers another client's resize, another client attaching, and this client's own resize once the server's new layout arrives. The server's Lua has no layout event, so the server side does not change.
- `gband.api_version` does not rise. Both changes only add to the documented API: a field that was an error becomes valid, and an event with the same empty payload runs in one more situation. design.md explains this reading of the rule.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `floating-windows`: "Float a window" and "Tile a window" cover a request that names the layer, and make a request for the layer the window is already in change nothing.
- `lua-control`: "Action targets" adds `floating` to the target of `toggle_window_floating`, its errors, and its scenarios.
- `lua-events`: "Built-in events" emits `LayoutChanged` when the size of the screen area changes.
- `server-runtime`: "Server session actions" adds `floating` to the server's `toggle_window_floating`.
- `session-server`: "Session actions" lets toggle floating name a layer, and says that a request for the layer the window is already in changes nothing.
- `wire-protocol`: "Handshake" sets protocol version 11, and "Client messages" lets toggle floating name a layer.

## Impact

- `crates/core/src/layout.rs`: `SessionAction::ToggleFloating` gains `floating: Option<bool>`, and `Layout::apply` floats or tiles only when the window is in the other layer. Every construction of the variant gains the field.
- `crates/protocol/src/message.rs`: `PROTOCOL_VERSION` becomes 11.
- `crates/lua/src/control.rs` and `crates/lua/src/actions.rs`: reading and checking `floating` in the client's and the server's targets.
- `crates/client/src/lib.rs`: the `after` fill-in keeps `floating`, and the observed layout includes the screen area's size, so `LayoutChanged` runs when it changes.
- Tests in `crates/core/tests/`, `crates/protocol/tests/messages.rs`, `crates/server/tests/`, `crates/client/tests/`, `crates/lua/tests/`, and a new `tests/lua/floating_target_spec.lua`.
- `docs/plugins.md`, `README.md`, `docs/tutorial/02-actions.md` and `docs/tutorial/05-layout.md`.
- No new dependency, no default binding change, and no change to the server's events.

## Coordination

### Author
- gmteixeira

### Depends On
- mouse-binding-fallthrough
- drag-resize-edges

### Expected Files
- README.md
- crates/client/src/lib.rs
- crates/client/tests/actions.rs
- crates/client/tests/animation.rs
- crates/client/tests/events.rs
- crates/client/tests/render.rs
- crates/core/src/action.rs
- crates/core/src/layout.rs
- crates/core/src/view.rs
- crates/core/tests/events.rs
- crates/core/tests/layout.rs
- crates/core/tests/view.rs
- crates/lua/src/actions.rs
- crates/lua/src/control.rs
- crates/lua/tests/control.rs
- crates/lua/tests/server.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/tests/events.rs
- crates/server/tests/resize.rs
- crates/server/tests/scripting.rs
- crates/server/tests/windows.rs
- docs/plugins.md
- docs/tutorial/02-actions.md
- docs/tutorial/05-layout.md
- tests/lua/floating_target_spec.lua
- tests/lua/screenshots/floating_target_spec/float-twice-in-one-callback--floated.txt
- tests/lua_specs.rs
- openspec/changes/floating-target/
