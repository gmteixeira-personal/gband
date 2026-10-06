## Why

In navigation mode, a middle-button drag slides the viewed band sideways and ignores vertical movement, so switching bands still needs the keyboard. niri lets one touchpad gesture do both: a horizontal swipe scrolls the columns of the current workspace, a vertical swipe moves between workspaces, and the direction of the larger movement decides which, so a gesture never does both at once. gband's band slide should work the same way.

## What Changes

- **Axis lock**: a `drag_band` gesture, including `drag_window` pressed on empty ribbon, moves nothing until the pointer has moved far enough from the press to tell its axis. The axis whose movement is larger, with a row counted as two columns because a cell is about twice as tall as it is wide, is locked for the rest of the gesture. Movement on the other axis is then ignored until the release.
- **Horizontal**: unchanged from today. The camera follows the pointer, and the release focuses the column at the middle of the view.
- **Vertical (new)**: the bands follow the pointer row for row, the way the band switch draws them, so the band above or below slides into view as the pointer moves down or up. Content follows the pointer, as the horizontal slide does: dragging up brings the band below into view. The drawn position stops at the first band and at the last, empty band. Each band is drawn with the camera that viewing it would give. Nothing is sent and no focus changes until the release.
- **Release of a vertical drag**: the client views the band whose region holds the terminal's middle row, with focus going as viewing the band below or above defines, and the band switch animation carries on from where the drag left the bands. Releasing over the band viewed at the press slides back to it and changes nothing else.
- **Descriptions**: `drag_band`'s description becomes `slide the band or switch bands with the mouse`, in `gband.action.list()`, the modal key style's binding and the default mouse bindings table.
- The band sidebar, from band-sidebar, marks the newly viewed band at the release, as it does for any change of the viewed band. It shows nothing during the drag.

Out of scope:
- Velocity or flick: the release picks the band from where the bands are drawn, not from how fast the pointer moved.
- Vertical wheel or Mod+wheel band switching. The wheel keeps reaching programs, as the mouse capability decided.
- Dragging across bands while a window is lifted.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `mouse`: "Slide the band by dragging" gains the axis lock and the vertical band switch, and its vertical-movement-changes-nothing rule goes.
- `animations`: "Drawing during a gesture" draws the bands at the drag's vertical position with no animation, and the release continues from it as the band switch animates.
- `actions`: `drag_band`'s description in "Lua names of actions".
- `client-attach`: the `middlemouse` row of "Default mouse bindings".

## Impact

- `crates/client/src/mouse.rs`: the slide gesture gains a pending axis and a vertical branch.
- `crates/client/src/animation.rs`: `Hold` gains a vertical position the gesture sets, and the drawn bands include every band the vertical position brings into the terminal.
- `crates/client/src/lib.rs`: a vertical drag ends when its band leaves the layout.
- `crates/lua/src/actions.rs`, `crates/lua/src/runtime/gband/keystyle/modal.lua`: the description.
- Tests: `crates/client/tests/actions.rs`, `crates/client/tests/animation.rs`, `tests/mouse.rs`.
- Docs: `README.md`, `docs/plugins.md`.
- No protocol change: the gesture is client-side and sends nothing.

## Coordination

### Author
- gmteixeira

### Depends On
- band-sidebar

### Expected Files
- openspec/changes/drag-bands/
- crates/client/src/mouse.rs
- crates/client/src/animation.rs
- crates/client/src/lib.rs
- crates/client/tests/actions.rs
- crates/client/tests/animation.rs
- crates/lua/src/actions.rs
- crates/lua/src/runtime/gband/keystyle/modal.lua
- tests/mouse.rs
- README.md
- docs/plugins.md
