## Why

Every view change in gband is a jump today. The camera snaps to the focused column, the next workspace replaces the current one in a single frame, and tiles teleport when a column opens, closes, moves or changes width. layout-core left animation out on purpose. In niri, motion is how the user keeps track of where things went on a strip that is wider than the screen, and gband follows niri where its specs are silent. The layout, the view and the tile geometry are now stable, and resize-semantics settles how widths and pane sizes change. Animations can now be layered on top as pure presentation, without changing what the server or the view decides.

## What Changes

- **Camera scroll.** When the viewed workspace's camera moves, the client glides the drawn strip from where it is to the new camera position over a few frames, instead of jumping.
- **Workspace switch.** Viewing the workspace above or below slides the old workspace out and the new one in vertically. A second switch during a slide continues from where the slide is.
- **Column movement.** When the layout changes, each tile of the viewed workspace slides from where it was drawn to its new position. This covers opening, closing, consume and expel, and width changes that shift neighbouring columns.
- **Resize morph.** When a tile's width or height changes, its border grows or shrinks to the new size over the same frames. The pane's grid is drawn from the tile's top-left corner, cut at the moving border, with blank cells where the border is wider than the grid. This is visual only. When the server resizes the PTY stays as resize-semantics defines it.
- **One motion model.** Every animation uses the same critically damped spring, after niri's defaults, and settles within 300 ms. Drawn positions are whole cells. A retargeted animation keeps its current position and velocity.
- **Frames only while moving.** The client draws extra frames only while an animation is running, and stops when every animation has settled.
- **Snap cases.** The first frame after attach, a change of the screen area, and a change of the client terminal's size never animate. The view's decisions do not change: focus and the camera target move at once, and key input goes to the newly focused pane straight away.
- **Turning them off.** `GBAND_ANIMATIONS=off` in the client's environment disables every animation. Unset or `on` enables them. Any other value is logged as a warning and enables them. A later Lua configuration replaces this variable.

## Capabilities

### New Capabilities

- `animations`: how the client moves from one drawn state to the next. Covers the camera scroll, the workspace switch, column movement and the resize morph, the shared spring and its settling, retargeting, the cases that snap, frame scheduling, the cursor during motion, and `GBAND_ANIMATIONS`.

### Modified Capabilities

- `client-attach`: presenting the ribbon draws each tile at its animated position and size, and may draw two workspaces during a switch. The cursor is hidden while the focused tile moves.

## Impact

- Code:
  - A new `crates/client/src/animation.rs` with the spring, the animated camera, workspace and tile state, and a clock passed in by the caller.
  - `crates/client/src/render.rs` draws from the animated state: a camera offset, a vertical workspace offset, and a per-tile position and size.
  - `crates/client/src/lib.rs` reads `GBAND_ANIMATIONS` when the client starts, retargets the animations from the layout and view before each frame, and schedules frames on a timer while any animation runs.
- Tests: unit tests for the spring and the animation state in `crates/client/tests/animation.rs`. Render snapshot tests of drawn states in `crates/client/tests/render.rs`. An end-to-end test of the variable in `tests/attach.rs`. The other end-to-end tests already wait for the settled screen, so they need no change.
- Dependencies: none added.
- Protocol: unchanged. The server sends the same messages and is unaware of animations.
- Interaction with `resize-semantics`: that change defines when PTYs resize, how a grid whose size differs from its tile is drawn, and which panes a client reports as shown. The resize morph only moves the tile's border between the old and the new tile size, and draws the grid by resize-semantics' rule. Shown panes keep following the view's camera, not the drawn one.
- Out of scope: open and close effects such as fading or growing a new tile from nothing, a reduced mode over SSH (no SSH transport exists yet), configurable curves and durations, and Lua configuration.

## Coordination

### Author
- gmteixeira

### Depends On
- resize-semantics

### Expected Files
- crates/client/src/animation.rs
- crates/client/src/lib.rs
- crates/client/src/render.rs
- crates/client/tests/animation.rs
- crates/client/tests/render.rs
- crates/client/tests/actions.rs
- crates/client/tests/snapshots/
- tests/attach.rs
- openspec/changes/animations/
