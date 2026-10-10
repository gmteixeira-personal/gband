## Why

On a band whose strip loops, the client sometimes draws the focused window cut off at the terminal's left edge, with its left border and the left part of its shell prompt missing. On 2026-10-10 the user saw it after toggling the terminal's full screen with columns of different widths; detaching and attaching again cleared it until it came back.

The view keeps two numbers per band in `crates/core/src/view.rs`: `camera`, held within the strip's width, which the policies aim from, and `travel`, which the client animates and draws from (`Targets::camera` in `crates/client/src/animation.rs`). `travel` differs from `camera` by a whole number of strip widths once focus has crossed the seam. When the strip's width then changes, from a column width change, a screen area change or a terminal resize, `View::aim` adds the camera's move to `travel` but keeps the old multiple of the old width. The client then draws every tile shifted by that multiple taken modulo the new width. With four 40-cell columns in an 80-column terminal, focus left from the first column and a widening to 100 columns, the camera is 120 and `travel` stays -40, so the focused column is drawn at cell -10 instead of 30. Attaching again builds a new view whose `travel` equals its `camera`, which is why it cleared.

## What Changes

- `View::aim` sets a looping band's `travel` to the camera's new position plus the whole number of current strip widths nearest the old `travel` plus the camera's move. `travel` then always equals the held camera modulo the current strip width, so the client draws each tile where the camera places it.
- Two tests in `crates/core/tests/view.rs` cover the widened terminal and a narrowed column after focus crossed the seam.
- The layout-view spec's Camera requirement states that a change of the strip's width keeps each tile at its drawn copy for the held camera, with two scenarios.

Out of scope:
- The animation frames drawn while a column's width changes. Tiles and the camera animate together as before; only the frame they come to rest on changes.
- Bands other than the viewed one. Their `travel` is corrected the next time the client views them, since entering a band aims its camera.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `layout-view`: "Camera" states that a change of a looping strip's width keeps each tile at its drawn copy for the held camera, however often the camera crossed the seam, and gains two scenarios.

## Impact

- `crates/core/src/view.rs`: `View::aim`.
- `crates/core/tests/view.rs`: two tests.
- No wire protocol change, no Lua API change, no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- openspec/changes/loop-strip-travel/
- crates/core/src/view.rs
- crates/core/tests/view.rs
