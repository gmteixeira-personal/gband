## Context

The layout lives in `gband_core::layout::Layout`, a list of `Band`s that each hold `Vec<Column>`. The server owns it and sends it whole, serialized with serde and postcard, in every `ServerMessage::Layout`. No layout events travel on the wire. Each client keeps a `View` (`crates/core/src/view.rs`) with one `BandView { focus, camera }` per band, a recency map of focused panes, and a remembered `Location` used to clamp focus after a close. Geometry is pure: `geometry::tiles(band, area)` gives every tile, and the server's `Session::settle` resizes shown panes from it. The client draws tiles in `render.rs`, then `gband.win` floats by `z`, then the status line or error banner. Animations retarget per-tile springs from the viewed band's tiles.

`gband.win` floats are client-only UI and are not panes. This change adds floating *panes*, which are shared session state. The two must not be confused in code or docs. The code says "floating pane" and "box" for the new thing and keeps "float" for `gband.win`.

## Goals / Non-Goals

**Goals:**
- Keep the floating layer in core, as pure data with pure geometry. Core tests can cover every layout and view rule without a server.
- Keep focus and stacking per client, like the rest of the view.
- Give every tiled action a floating meaning in one place: the layout's dispatch on where the named pane lives.

**Non-Goals:**
- Animating floating boxes. Boxes are drawn at rest, per the animations delta.
- Sharing z-order between clients.
- Any change to `gband.win` floats beyond drawing them above floating panes.

## Decisions

### Floating panes live in the band, beside the columns
`Band` gains `floating: Vec<FloatingPane>`, where `FloatingPane { pane, col: u16, row: u16, width: Proportion, full_width: bool, rows: u16 }`. `Band::is_empty` checks both lists, so `normalize` keeps the dynamic band rule without other changes.

- *Alternative*: a session-wide floating list keyed by band. Rejected. It splits a band's contents across two structures, and band removal would have to sweep both.
- *Alternative*: a floating pane as a one-pane `Column` with a flag. Rejected. Column widths, strip spans and `tiles()` would all need to skip it, and every existing loop over columns would grow a filter.

### Remembered boxes are layout state
`Layout` gains `boxes: BTreeMap<PaneId, FloatingPane>`. It holds the box record a tiled pane had when it last floated, so floating it again restores that box, as niri does. An entry is written on tiling and dropped when the pane leaves the layout. The map is part of the serialized layout, so every client and a later reattach see the same state. Lua does not expose it.

### Box geometry mirrors tile geometry
`geometry::placed(floating, area) -> Box` returns the placed `x`, `y`, `width` and `height`, plus `terminal_size()`, with the clamping the floating-panes spec defines. The stored record is never rewritten by a shrinking area, so growing the area back restores the box. Moves and `set_position` store the clamped placed value, so a box pushed against an edge does not keep a hidden offset.

`Session::settle` iterates `placed()` for each band's floating panes beside `tiles()`. The pane-resize rules then apply unchanged.

### Locating a pane returns where it lives
`Location` stays the tiled position. A new `Place` enum, `Tiled(Location)` or `Floating { band, index }`, is returned by `Layout::place(pane)`. `Layout::apply` matches on it:

- Width actions run `Column` methods on a column, or the same rules on the box. The cycle, step and set logic moves to free functions over `(Proportion, bool)`, shared by `Column` and `FloatingPane`, so the two cannot drift.
- Height actions on a box use the tiled step formula. That formula moves into `geometry` as `height_step(area)`, beside a new `width_step(area)` for horizontal moves.
- Consume or expel on a floating pane returns no events.
- Move actions swap columns or rows for a tiled pane, and step the box for a floating pane.

- *Alternative*: separate floating actions, such as `grow_floating_width`. Rejected. The user asked for one set of operations, and niri reuses its actions the same way.

### New actions in core
- `SessionCommand` gains `ToggleFloating`, `MoveColumn(Direction)` and `MovePane(Vertical)`, with a new `Vertical { Up, Down }`.
- `SessionAction` gains `ToggleFloating { pane, after: Option<PaneId> }`, `MoveColumn { pane, direction }`, `MovePane { pane, direction }` and `SetPosition { pane, col, row }`.
- `OpenPane` gains `floating: bool`.
- `ViewAction` gains `SwitchLayer`.

`SessionCommand::on_pane` cannot resolve `ToggleFloating`, because it needs the tiled focus. `View::resolve` handles it, together with `OpenPane`.

### The view tracks a layer per band
`BandView` becomes `{ layer: Layer, tiled: Option<PaneId>, floating: Option<PaneId>, camera }`. `View::focused()` returns the pane of the active layer. `resolve` uses `tiled` as the open-after and tile-after pane.

`settle` gains the floating cases:
- When the focused floating pane has left, focus goes to the most recent floating pane by `recency`, else the last in the list, else the tiled layer.
- An empty active layer falls back to the other layer.
- A focused pane that changed layer keeps its focus and flips the layer.

Directional focus in the floating layer compares doubled centres, `2*x + w` and `2*y + h`, to stay in integers. `follow` (the camera) reads `tiled` instead of `focused()`, so the camera holds still while the floating layer is active.

The client's stacking order comes from the same `recency` map. Never-focused floating panes sort first by list index, and focused ones sort by tick. No new state is needed.

### Rendering and shown panes
`View::shown` adds every floating pane of the viewed band whose placed box intersects the client terminal, without the camera offset. `render.rs` draws tiles, then floating panes in stacking order through the existing `draw_tile` path, then `gband.win` floats, then the banner.

Border styles reuse `FOCUSED_BORDER` and `UNFOCUSED_BORDER`. The cursor check gains a test for "covered by a floating pane drawn above the focused pane". The animation `Targets` keeps building springs from `tiles()` only. A pane that became floating therefore drops out of the tile springs, and one that became tiled appears at rest, which matches the spec.

### Events
`LayoutEvent` gains these variants:
- `ColumnMoved { band, from, to }`
- `PaneFloated { pane, band, record }`
- `PaneTiled { pane, band, column, width, full_width }`
- `FloatingBoxChanged { pane, band, record }`

Swapping two panes in a column emits two `PaneMoved`. Lua's `LayoutChanged` already fires on any layout message, so `crates/lua/src/events.rs` needs no new event. The session event log formats the new variants.

### Lua surface
- `actions.rs`: `ACTIONS` grows from 19 to 25 entries, in the order of the actions spec table.
- `control.rs`:
  - `targeted()` learns `toggle_pane_floating`'s `{ pane, after }` and the move actions' `{ pane }`.
  - `open_target()` learns `floating`.
  - `gband.layout()` adds `floating` per band.
  - `gband.view()` adds `floating`.
  - `gband.pane.set_position` is a new dispatch, `Dispatch::Session(SetPosition)`, validated against the client's layout.
- `defaults.lua` adds the ten bindings after `prefix R`.
- `hints.lua` adds the six short labels.

### Default keys
`prefix v` and `prefix V` follow niri's `Mod+V` and `Mod+Shift+V`. The move actions bind `prefix ctrl+h/j/k/l`, as the user chose, and `prefix ctrl+left/right/down/up`, because the project rule is that every directional action takes both hjkl and arrows. crossterm decodes `\x08`, `\x0a`, `\x0b` and `\x0c` in raw mode as Ctrl with h, j, k and l, not as Backspace or Enter, so these bindings match on plain terminals. The input-encoding spec already pins Ctrl+H as distinct from Backspace.

### Protocol version 6
The layout's serialized shape changes, and postcard has no field tags, so an old peer would misdecode it. `PROTOCOL_VERSION` becomes 6, and the handshake test pins it.

## Risks / Trade-offs

- **[Protocol number collides with another open change]** `split-plugin-runtime` and `plugin-testing` also touch `crates/protocol/src/message.rs` and may bump the version. → At `/ready`, take the next number above whatever `<base>` holds, and update the spec delta's number and scenario to match.
- **[Overlap with `key-list`]** `key-list` rewrites action descriptions from "pane" to "window" in `actions.rs` and `defaults.lua`. → Whichever change merges second rewords the six new descriptions to match. The merge conflict is textual and local.
- **[Ctrl+J looks like Enter on some setups]** Outside raw mode, or under terminals that translate `\n`, Ctrl+J may arrive as Enter. → gband always runs the client in raw mode. `prefix ctrl+down` is the fallback binding.
- **[A box can cover the focused tiled pane's cursor]** → The cursor hides when a floating pane covers its cell, as the client-attach delta defines.
- **[A stale remembered box after the area shrinks]** → The record is clamped at placement time, never stored clamped by the area. Floating again uses the record and places it inside the current area.
- **[`Location`-based code paths miss floating panes]** Examples are `View::clamped`, `Layout::locate` callers in the server, and Lua's layout reader. → Keep `locate` tiled-only by name. Add `place` and `contains` that cover both kinds. Audit every `locate` call during the core task. `contains` must cover floating panes, or the server would drop actions on them as "not in the layout".

## Migration Plan

There is no persisted state. A running server of version 5 refuses new clients with the usual version message, and the user restarts it. Rollback is reverting the change. Nothing on disk changes shape.
