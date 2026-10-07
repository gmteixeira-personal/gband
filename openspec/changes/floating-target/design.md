## Context

See proposal.md for the motivation. The state this change builds on:

- `SessionAction::ToggleFloating { window, after }` in `crates/core/src/layout.rs` is the one session action that moves a window between layers. `Layout::apply` floats it when `place(window)` is `Place::Tiled` and tiles it when it is `Place::Floating`. It is `Serialize` and `Deserialize`, and the client sends it in `ClientMessage::Action`.
- The server applies session actions in arrival order in `Session::act` in `crates/server/src/session.rs`, and publishes a layout only when `Layout::apply` returns events. An action that changes nothing sends no layout.
- The client's Lua builds the action in `toggle_floating_target` in `crates/lua/src/control.rs`. The server's Lua builds it in `session_target` in `crates/lua/src/actions.rs`. A binding resolves it in `View::resolve` in `crates/core/src/view.rs`.
- `Controls::apply` in `crates/client/src/lib.rs` fills a missing `after` with `Display::tiled_beside` before it sends a targeted toggle.
- The client emits `LayoutChanged` in `changes` in `crates/client/src/lib.rs`, when `Observed.shape` differs. `shape` holds each band's columns and floating box records. It does not hold the screen area, which `Display.area` takes from each `ServerMessage::Layout`. The server publishes a new layout whenever the area changes, in `Session::handle` for `Command::Area`.
- The server's Lua has no layout event. Its built-in events, in the server-runtime capability's "Server events", are about sessions, windows and clients.

## Goals / Non-Goals

**Goals:**
- One rule for "float it" and "tile it", applied where requests are ordered, so concurrent requests from any client or from the server's Lua cannot undo each other.
- Keep every existing call, binding and wire message meaning what it means today.
- Let a client notice every change of the screen area with the event it already uses for layout changes.

**Non-Goals:**
- A server-side layout event, or any other change to the server's events.
- A separate screen area event, or a payload for `LayoutChanged`.
- Resolving `{ floating = true }` without `window` against the view.
- Changing the `prefix v` binding or any preset. Bindings keep toggling.
- Updating `floating-key-style`. That change adopts `floating = true` and `LayoutChanged` in its own revision.

## Decisions

### The layer travels with the toggle
`SessionAction::ToggleFloating` gains `floating: Option<bool>`. `None` toggles. `Some(true)` names the floating layer and `Some(false)` the tiled layer. `Layout::apply` matches the window's place with the named layer:

| place | `None` | `Some(true)` | `Some(false)` |
|---|---|---|---|
| tiled | float | float | nothing |
| floating | tile after `after` | nothing | tile after `after` |
| not in the layout | nothing | nothing | nothing |

"Nothing" returns no events, so the server sends no layout, as for any action that changes nothing. Every caller of `Layout::apply` gets the same rule, the server's session and the trial applies in `Display` alike.

- *Alternative*: two new session actions, float and tile. Rejected. One Lua action would map onto three wire actions, and the session-server and wire-protocol tables would grow two rows that mostly repeat toggle floating.
- *Alternative*: idempotence in the client, which sends a toggle only when its own layout shows the other layer. Rejected. Two clients that both see window 3 tiled both send a toggle, and the second tiles it again. Only the server orders the requests.

### The client always sends
The client sends `toggle_window_floating` with its layer whatever layer its own layout shows. Skipping a request that the client's layout already satisfies would save one small message, but that layout can be older than the server's. If another client tiled window 3 a moment ago, a skipped float would leave it tiled, although this client's request came last. With every request sent, the last request the server receives decides, as the session-server delta states.

### `after` with a layer
`Controls::apply` fills a missing `after` only when `floating` is not `Some(true)`, and keeps `floating` in the action it sends. A float never uses `after`, so it carries none on the wire.

`after` together with `floating = true` is an error in both Lua sides, as it already is for `open_window`: it can have no effect, so it is a mistake in the call. `after` with `floating = false`, or with no `floating`, stays valid on a window the client sees tiled, and is ignored there, because the server may see that window floating.

### Reading the target
`toggle_floating_target` in the client and the `ToggleFloating` branch of `session_target` in the server take `floating` in their allowed fields and check, in this order:

1. Unknown fields, as today.
2. A missing `window`, as today in the client; the server already requires `window` through `number("window", true)`.
3. `floating` that is neither nil nor a boolean: "the `floating` of `toggle_window_floating` must be a boolean, found …", the message `open_window` uses.
4. `after` with `floating = true`: "the target of `toggle_window_floating` cannot hold `after` with `floating = true`", the message `open_window` uses.
5. In the client, the window and `after` checks against its layout, as today.

`window` stays required. `step` targets resolve a missing `window` against the view, but the desktop style always names the window, and a binding that wants the focused window passes `gband.view().window`. Allowing it later is an addition.

### Protocol version 11
postcard writes no field tags, so the added field changes the encoding of every toggle floating message. A version 10 server would misdecode a version 11 client's toggle and close the connection, so `PROTOCOL_VERSION` becomes 11 and the handshake tells the two apart, as the archived `floating-windows` change did when the layout gained floating windows. The wire-protocol delta raises the version in "Handshake", adds the "Version 10 client meets a version 11 server" scenario, and lets toggle floating name a layer in "Client messages". No other open change raises the protocol version.

### `LayoutChanged` and the screen area
`Observed` gains `area: Size`, filled from `Display.area`, and `changes` pushes `LayoutChanged` when `shape` or `area` differs. The event keeps its place among the others: after `WindowClosed` and `WindowOpened`, before `BandChanged`, `FocusChanged` and `TerminalResized`. A layout that changes both the area and the shape emits one `LayoutChanged`.

This covers every way the area changes: another client's resize, another client attaching with another reported size, a bar that changes this client's reported size, and this client's own resize. For its own resize, `TerminalResized` runs when the client handles the new terminal size, and `LayoutChanged` runs later, when the server's layout with the new area arrives. The first layout after attaching still emits nothing, because there is no previous observation.

- *Alternative*: a new event such as `ScreenAreaChanged` with `cols` and `rows`. Rejected. A handler that fits windows to the area already reads `gband.layout()` on `LayoutChanged`, and `cols` and `rows` are part of what `gband.layout()` reports. A second event would make it listen to two events for one kind of change.
- *Alternative*: compare the placed boxes instead of the box records. Rejected. It misses a larger area that clamps no box, which is the case a maximized window needs.

### The server side does not change
The server has no layout event to widen. Its Lua reads the area through `gband.session()` and is not told about layout changes today, so emitting one only for the area would be a new event with no caller. Keeping the scope to the client's `LayoutChanged` is enough for the floating key style, which runs in the client.

### `gband.api_version` does not rise
The lua-api capability's "API version rule" raises the version when a build removes a documented function, table, field, primitive, provider function, module export, event or payload field, or changes the documented arguments, return values, errors or effect of one. A build that only adds keeps it. Read literally:

- `floating` removes nothing. Every call that worked before works the same. `floating` was a field the action does not take, an error, and an addition that makes a former error valid is what `drag-resize-edges`, `window-decorations` and `floating-key-style` read as adding. The new errors check only the new field.
- `LayoutChanged` removes nothing, and its payload stays empty. Of the four things whose change raises the version, an event has no arguments, return values or errors of its own, and its effect, running its handlers, is unchanged. "Emitted when" is the condition of emission, which the rule does not list. Every layout that emitted `LayoutChanged` still emits it, once and in the same order. The event now also runs for a layout that differs only in `cols` and `rows`, which `gband.layout()` already documents as part of the layout. That widens the event as making a former error valid widens a function, so it is an addition.

So `gband.api_version` stays at the value the integration branch holds, which is 2 once `mouse-binding-fallthrough` lands, and the "API and stability" table gains no row. No delta of this change states the version.

### Documentation
- `docs/plugins.md` is the reference. "Action targets" gets the `floating` field of `toggle_window_floating`, its errors and an example. The events table's `LayoutChanged` row names the screen area's size, and the paragraph after the mouse fields says that a resize by any client, this one included, emits it. "Session actions in the server" gets `floating`.
- `README.md` lists what a target names in the sentence after the action table, so that sentence names `floating`. It lists no events.
- `docs/tutorial/02-actions.md` teaches targets and links to the reference, so it gains one sentence and no code block. A code block would have to appear in `examples/tutorial/02-actions/user/init.lua` and in every later chapter's copy.
- `docs/tutorial/05-layout.md` teaches `gband.layout()`, so it gains one sentence on `cols`, `rows` and `LayoutChanged`, with no code block, for the same reason.
- `docs/tutorial/04-events.md` teaches `WindowClosed` and `User` events and links to the full list, and `docs/tutorial/10-server.md` teaches no session action, so neither changes.
- `docs/internals/` does not change. No chapter describes action targets, `LayoutChanged` or the protocol. Chapter 08 quotes the presets, whose `prefix v` binding stays a toggle.

## Risks / Trade-offs

- [`ToggleFloating` gains a field, so every construction of it must change, mostly in tests] → The compiler finds each one. `ast-grep` adds `floating: None` to each literal in one pass, and the tasks list every file.
- [Another change also raises the protocol version before this one lands] → At `/ready`, take the next number above the base and update the Handshake delta and its scenario to match.
- [A reviewer reads "emitted when" as an event's documented effect, so `LayoutChanged` would need a new API version] → The rule raises the version once per release. If this change lands in the same release as `mouse-binding-fallthrough`, the integrator adds the `LayoutChanged` condition to the version 2 row instead of raising it to 3.
- [A `LayoutChanged` handler now also runs when only the area changes] → Its payload is empty, so a handler must already read the layout again. Running it once more for a layout that did change gives the result a later read would give. Area changes are rare: a resize, an attach or a bar change.
- [After its own resize, a client runs `TerminalResized` and then `LayoutChanged`] → The docs say so. A handler that needs only the area can listen to `LayoutChanged` alone, which also covers other clients.
- [`floating-key-style` was drafted around a toggle and around `TerminalResized`] → It depends on this change and should switch its float rules to `floating = true` and refit boxes on `LayoutChanged`. That is a revision of that change, not part of this one.

## Migration Plan

The protocol version rises, so a running version 10 server refuses a new client. The client already tells the user to stop the old server, and `gband kill-server` does it. No configuration, saved file or plugin needs a change: every existing call keeps its meaning. Rollback is a revert of the change's commits, which brings back protocol version 10.
