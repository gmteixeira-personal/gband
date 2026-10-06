## Context

The view lives in `crates/core/src/view.rs`. `View::neighbour_column` stops at the band's ends, `View::follow` places the camera through `centred` and `revealed` on one `Span` per column from `geometry::column_spans`, and `View::shown` filters tiles against `camera..camera + viewport`. The client copies `view.camera()` into `animation::Targets`, springs a drawn camera toward it, and `render::Placement` draws each tile at `tile.x - band.camera`. Options reach the view through `Client::configure` in `crates/client/src/lib.rs`, as `center_focused_column` does.

This change depends on `floating-windows`. Its layer split decides where looping applies: the floating layer keeps its boxes in terminal cells, and only the tiled strip loops. The delta specs are written over the `floating-windows` wording of "Focus across columns", "Camera" and "Shown windows".

## Goals / Non-Goals

**Goals:**
- Keep looping a pure client view rule, with no protocol or server change.
- Keep every existing camera rule unchanged when the strip does not loop, so today's tests pass with `loop_bands` off or on a short strip.

**Non-Goals:**
- Session actions at the strip's edge, such as consume or expel, stay as they are.
- No looping between bands.

## Decisions

**Loop test.** A band loops when `loop_bands` is on and `strip_width - widest >= viewport`, where `strip_width` is the last span's end. This is exactly the condition under which no tile can have two copies inside the view at once: two copies of a column of width `w` are `strip_width` apart, so both are visible only when `strip_width - w < viewport`. Alternative considered: always loop and draw a window twice when the strip is short. Rejected by the user: a window and its cursor drawn twice is confusing. Focus still goes round on a short strip.

**Copies, not a modulo camera in the policies.** `follow` picks one copy of the focused column's span, `span.x + k * strip_width`, then runs `centred` or `revealed` unchanged on that shifted span. For a left or right focus move, the copy is the neighbour of the previously focused column's drawn copy on the side of the move. Picking the smallest move instead can scroll against the key: with three 40-cell columns on 80 cells and the camera at 80, moving right to the second column ties between camera 40 and camera 120. For every other change, the copy needing the smallest move is used, with the lower start on a tie. `center_column`, from the `center-column` change, goes through the same copy choice when both land.

**Held camera and travelled camera.** `BandView` keeps two values: `camera`, held in `0..strip_width` while looping and returned by `View::camera()`, and `travel`, the unreduced camera that adds each move's signed distance. `View::travel()` feeds `Targets.camera`, so the spring scrolls the real distance and direction across the seam. Alternative considered: give the animator only the held camera and let it pick the equivalent nearest the drawn one. Rejected: a move longer than half the strip, possible with one very wide column, would scroll the wrong way. When a band stops looping, `travel` is reset to `camera`. `Targets` then carries the strip width it last looped with, and the animator shifts its spring by the whole multiple of that width that removes the difference, so nothing jumps.

**Drawing at the drawn copy.** `DrawnBand` gains the strip width when the band loops. `Placement::new` reduces `tile.x - camera` modulo the strip width into `0..strip_width`, and subtracts one strip width when the result is not inside the terminal. The loop test guarantees at most one copy is inside, so this choice is unique. `View::shown` applies the same reduction. Tile movement and resize morph keep their drawn positions in strip coordinates and are reduced at placement only.

**Option plumbing.** `loop_bands` is a client `bool` in `crates/lua/src/options.rs`, beside `center_focused_column`, declared in `defaults.lua` as `gband.opt.loop_bands = true`. `View::set_loop_bands` mirrors `set_center_focused_column`.

## Risks / Trade-offs

- [The loop test flips while columns open, close or resize, so a band can start or stop looping mid-animation] → The held camera and the spring re-base keep the frame continuous. At rest the drawn state always equals the target.
- [`travel` grows without bound after many laps] → An `i64` will not overflow in practice. It is also reset whenever the band stops looping, and when the client terminal resizes, since a resize snaps every animation.
- [Requirement text overlaps other changes] → `floating-windows` rewrites the same layout-view requirements, so this change depends on it. `sidebars-borders-steps` also rewrites configuration's "Options". Whichever of the two archives second must carry both option rows. The tasks re-check this before `/ready`.
- [Default on changes behaviour for existing users] → `loop_bands = false` restores today's behaviour. Tests that assert focus stops at the edge set it off explicitly.
