## Context

`BandView` in `crates/core/src/view.rs` holds `camera` and `travel`. `View::aim` sets both after every focus, layout, area or terminal change. The client turns `travel` into the drawn camera: `Targets::new` reads `view.travel()`, the camera spring animates toward it, and `Placement::new` in `crates/client/src/render.rs` draws each tile at `drawn_copy(tile.x - camera, strip, width)`, which takes the position modulo the current strip width. The drawing matches the view only while `travel` equals `camera` modulo the current strip width. `aim` kept that true for a fixed width, by adding the camera's move to `travel`. It broke as soon as the width changed with `travel` a nonzero number of laps from `camera`. See proposal.md for the symptom.

`View::slide` already keeps the relation, since it derives `camera` from `travel` with the current strip. The non-looping branch of `aim` sets `travel` equal to `camera`, and `Presentation` in `crates/client/src/animation.rs` shifts the spring by whole laps when a strip stops looping.

## Goals / Non-Goals

**Goals:**
- After every `aim` on a looping strip, `travel` equals `camera` modulo the current strip width.
- When the width does not change, `travel` moves exactly as before, so scrolling across the seam animates as the animations spec requires.

**Non-Goals:**
- Reworking how the client animates a strip whose width changes mid-scroll.

## Decisions

**Round `travel` to the nearest consistent value in `aim`.** `aim` computes the unrounded `travel` as before, `travel + moved - camera`, then picks the whole number of current widths `laps` nearest to `travel - moved`, rounding halves up, and sets `travel` to `moved + laps * width`. With an unchanged width, `travel - moved` is already a whole multiple, so `laps` is that multiple and `travel` is unchanged. With a changed width, `travel` lands within half a width of where the animation was heading, so the camera spring scrolls the short way.

Alternatives considered:
- Reset `travel` to `camera` whenever the strip width changes. The view does not keep the previous width, so it would have to store it. The reset would also jump `travel` by laps, and a spring that was mid-scroll would then sweep the whole strip.
- Have the client draw from `camera` instead of `travel`. `travel` exists so a move across the seam scrolls in the direction of the move. Drawing from `camera` would scroll the long way round on every crossing.

**Test in `crates/core/tests/view.rs`.** The client's drawn camera is `travel` and its strip is `View::strip`, so asserting `travel` modulo the strip equals `camera`, plus the drawn positions from `drawn_in`, pins the drawn frame without a client in the loop. The tests mirror the two new spec scenarios.

## Risks / Trade-offs

- [The camera spring may scroll a short distance when a column's width changes after a seam crossing, where before it held still at a wrong position.] → The rest position is now the one the spec requires. The scroll is at most half a strip width.
