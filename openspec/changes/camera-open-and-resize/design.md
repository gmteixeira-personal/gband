## Context

`View::aim` in `crates/core/src/view.rs` places the camera. On a looping strip it picks the focused column's copy from `anchor`, the previously focused column and the heading, which only `FocusLeft` and `FocusRight` supply. Every other focus change, including the server focusing a window it just opened, falls back to the copy needing the smallest camera move, with the smaller start on a tie. When the strip's width less its widest column equals the view's width, the two candidate copies of a just-opened column tie, and the left one wins.

`revealed` implements `"never"`: it keeps the camera while the focused column is wholly inside the view. `View::sync` runs after every layout message and every terminal or bar change, but the view does not remember the area or viewport it last saw, so it cannot tell an area change from any other sync.

## Goals / Non-Goals

**Goals:**
- An opened column lands next to the column it opened beside, at every width.
- After the area or the viewport changes, a non-looping strip leaves no blank cells right of its end while the camera could show more of it.

**Non-Goals:**
- No change to `"always"`. It centres the focused column on purpose, and a blank strip beside a centred last column is its intended look.
- No pull-back after focus moves, closes or width changes that keep the area. Those keep today's camera.
- No change for looping strips: they have no end to show past.

## Decisions

**Treat an opened column like a focus move toward it.** When `settle` focuses a window that the view has not seen before, and its column is next to the previously focused column, pass `follow` the heading toward it and the previous column as `anchor`. `aim` then takes the copy next to the previous column's drawn copy, the path `FocusRight` already uses. The view's `recency` map records every window it has focused, so a window absent from it and newly in the layout counts as opened. Rejected alternative: changing the tie-break to the larger start, which would only flip the problem to windows opened on the left.

**Remember the last area and viewport in `View`.** `sync` compares the scene's `area` and `viewport` with the stored pair, stores the new pair, and on a difference applies the pull-back after `follow` has placed the camera. Rejected alternative: having `Display` call a separate method on resize, which would miss an area change that arrives in a layout message from another client.

**Pull back only under the never placement.** The pull-back runs when the policy is `"never"`, or `"on-overflow"`, which moves as `"never"` on every change but a focus move between columns. The camera moves to `max(0, strip end − viewport)` only when that is less than the current camera, so it never moves right and never below 0. The focused column stays wholly in view, because it ends at or before the strip's end.

## Risks / Trade-offs

- [Both this change and ribbon-screen-area MODIFY the layout-view Camera requirement, and ribbon-screen-area adds the bars requirement this change modifies] → `Depends On: ribbon-screen-area`. The first task re-copies both requirements from `origin/dev` once ribbon-screen-area has archived, then reapplies this change's edits.
- [A window opened by a plugin far from the focused column gets no anchor] → It keeps the smallest-move rule, as before. Only a column next to the previously focused one is anchored.
- [Animations] → The pull-back changes the camera target. The animations capability already glides or snaps to a new camera target, and an area change snaps, so no new animation rule is needed.
