## Context

`View::aim` in `crates/core/src/view.rs` places the camera. On a looping strip it picks the focused column's copy from `anchor`, the previously focused column and the heading, which only `FocusLeft` and `FocusRight` supply. Every other focus change, including the server focusing a window it just opened, falls back to the copy needing the smallest camera move, with the smaller start on a tie. When the strip's width less its widest column equals the view's width, the two candidate copies of a just-opened column tie, and the left one wins.

`revealed` implements `"never"`: it keeps the camera while the focused column is wholly inside the view. `View::sync` runs after every layout message and every terminal or bar change, and each run places the camera through `follow` and `aim`.

## Goals / Non-Goals

**Goals:**
- An opened column lands next to the column it opened beside, at every width.
- Under the never placement, a non-looping strip leaves no blank cells right of its end while the camera could show more of it, after any change.

**Non-Goals:**
- No change to `"always"`. It centres the focused column on purpose, and a blank strip beside a centred last column is its intended look.
- No change for looping strips: they have no end to show past.

## Decisions

**Treat an opened column like a focus move toward it.** When `settle` focuses a window that the view has not seen before, and its column is next to the previously focused column, pass `follow` the heading toward it and the previous column as `anchor`. `aim` then takes the copy next to the previous column's drawn copy, the path `FocusRight` already uses. The view's `recency` map records every window it has focused, so a window absent from it and newly in the layout counts as opened. Rejected alternative: changing the tie-break to the larger start, which would only flip the problem to windows opened on the left.

**Pull back on every never placement, not only on area changes.** A blank strip appears whenever the strip's end moves left of the view's right edge: an area or ribbon change, the last column closing or narrowing, or a window leaving the band. Applying the pull-back in `aim`'s non-looping branch, after `revealed` places the camera, covers every one of them, and needs no record of the last area. Rejected alternative: storing the last area and viewport in `View` and pulling back only when they change, which leaves the blank strip after closing a column.

**Skip it when centring.** The pull-back runs when `aim` is not centring: under `"never"`, and under `"on-overflow"` when it moves as `"never"`. `"always"`, `"on-overflow"`'s centring path and `center_column` keep their centred camera. The camera moves to `max(0, strip end − viewport)` only when that is less than the current camera, so it never moves right and never below 0. The focused column stays wholly in view, because it ends at or before the strip's end.

## Risks / Trade-offs

- [Both this change and ribbon-screen-area MODIFY the layout-view Camera requirement, and ribbon-screen-area adds the bars requirement this change modifies] → `Depends On: ribbon-screen-area`. The first task re-copies both requirements from `origin/dev` once ribbon-screen-area has archived, then reapplies this change's edits.
- [A window opened by a plugin far from the focused column gets no anchor] → It keeps the smallest-move rule, as before. Only a column next to the previously focused one is anchored.
- [Closing the last column now moves the camera, so the remaining columns slide right] → This is the intended effect, and the animations capability glides to the new target as for any camera move.
- [Animations] → The pull-back changes the camera target. The animations capability already glides or snaps to a new camera target, and an area change snaps, so no new animation rule is needed.
