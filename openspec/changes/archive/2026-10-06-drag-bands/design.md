## Context

See proposal.md for why. The code today:

- **Slide gesture.** `drag_band`, and `drag_window` on `Target::Ribbon`, start `Motion::Slide { travel }` (`crates/client/src/mouse.rs:883`). `gesture_motion` calls `View::slide(travel - dx)` and holds the camera spring with `Hold { camera: true, .. }`. `end_gesture` calls `View::release_slide` (`crates/core/src/view.rs:163`). `dy` is never read.
- **Band switch drawing.** `Presentation` (`crates/client/src/animation.rs`) keeps one `vertical` spring and a `leaving` list of `(band, camera, strip)`. `drawn()` emits a `DrawnBand` for each leaving band and for the viewed band at `index * band_height - vertical`. `band_height` is the ribbon's height. A band switch pushes the old band onto `leaving` and retargets `vertical` without snapping, so a switch started mid-motion continues from the drawn value.
- **Hold.** `Hold { camera, windows }` snaps the named springs to their targets on every `update`. Nothing can hold `vertical` at an arbitrary value.
- **Viewing a band.** `View` is `Clone`. `View::apply(ViewAction::FocusBandDown | FocusBandUp)` implements layout-view's "Switch band", including the focus rule and the camera the new band settles to.
- **Gesture end on layout change.** `Display::end_gesture_for_layout` (`crates/client/src/lib.rs:434`) ends a gesture whose window left the layout. A slide names no window.
- **band-sidebar**, a dependency, adds a one-column side bar. It changes the ribbon's width, not its height, so `band_height` stays the terminal's height.

## Goals / Non-Goals

**Goals:**
- One gesture, two axes, decided once. The horizontal branch stays exactly today's code path, so its tests keep passing unchanged.
- The vertical drag reuses the band switch's drawing, so the release hands over to the existing spring with no jump.
- The release changes the view through the existing `FocusBandDown`/`FocusBandUp` view actions, so focus and camera follow "Switch band" with no second implementation.

**Non-Goals:**
- Velocity tracking or flick projection.
- Switching more than one band in one drag. The pointer cannot travel a full band height inside the terminal, so the release can only reach the adjacent band.

## Decisions

### The axis is a field of the slide motion
`Motion::Slide` becomes `Slide { travel, axis: Option<Axis> }`, with `Axis::Horizontal` and `Axis::Vertical { band }`, where `band` is the `BandId` viewed at the press. `gesture_motion` locks `axis` on the first motion with `|dx| >= 2 || |dy| >= 1`, choosing horizontal when `|dx| >= 2 * |dy|`. Before the lock it does nothing. After it, it runs only the locked branch.

A row counts as two columns because terminal cells are about 1:2. Without the weighting, a mostly sideways drag that drifts one row would lock vertical. The thresholds of 2 columns and 1 row are the smallest that make the weighted comparison meaningful: a lone 1-column move is too small to tell the axis.

`end_gesture` keeps today's `release_slide` for `None` and `Horizontal`. A click with no motion therefore still focuses the middle column, as it does now.

*Alternative:* re-decide the axis on every motion from the total movement. Rejected: the bands and the camera would jump between modes mid-drag, the opposite of the "never both" the user asked for.

### The presentation can hold the vertical position
`Hold` gains `vertical: Option<HeldBands>`, where `HeldBands { top: i64, peek: Option<(BandId, i64, Option<u32>)> }` gives the drawn vertical position and the neighbour band's id, camera and strip. While it is set, `Presentation::update` sets the `vertical` spring at rest at `top` after the usual update, and `drawn()` emits the peek band beside the viewed one, at its own index.

At the release, the gesture takes the hold away before changing the view:
- When the release views a new band, `Shown::update` sees a band change. It pushes the old band onto `leaving` and retargets `vertical` from the held value, which is today's switch path. The peek band is the new viewed band, so it needs nothing.
- When the release keeps the band, the peek band is pushed onto `leaving` with its drawn camera, and `vertical` retargets back to the viewed band's top. `leaving` is already cleared once `vertical` comes to rest.

A new `Presentation::release_bands` does the second case, so `Shown`'s fields stay private to `animation.rs`.

*Alternative:* let the gesture write the `vertical` spring directly. Rejected: `update` retargets the spring on every frame, so the write would need ordering rules that a hold already provides for the camera.

### The neighbour's camera comes from a cloned view
On each vertical motion, the client clones the `View`, applies `FocusBandDown` or `FocusBandUp` toward the neighbour that the drawn position reveals, and reads `travel()` and `strip()` from the clone. That is exactly the camera the release will set, so the neighbour does not jump when it becomes the viewed band. The clone is cheap: a few small maps.

Only one neighbour can be visible, since `|dy|` is below the band height. Moving up reveals the band below and moving down the band above; at `dy == 0` there is no peek.

### The release uses the existing view actions
The gesture computes `index = (top + band_height / 2) / band_height` and compares it with the viewed band's index. One below applies `ViewAction::FocusBandDown`, one above `FocusBandUp`, equal changes nothing. Both actions already implement "Switch band" and the camera settle.

### Bands shifting during the drag
`Axis::Vertical` stores the `BandId`, not the index. Each motion recomputes the viewed band's index from the current layout, so a band added or removed above shifts the drawn position with it. `end_gesture_for_layout` ends a vertical slide whose band left the layout, clearing the hold, as it already ends a gesture whose window left.

## Risks / Trade-offs

- **Half a terminal to switch.** With no velocity, switching needs the pointer to cross the middle row, 12 rows on a 24-row terminal. → The spec states the rule, and a flick rule can be added later without changing the axis lock or the drawing.
- **Small sideways moves are now ignored.** A 1-column move no longer slides the band until the axis locks. → The camera jumps to `press - dx` at the lock, so no movement is lost.
- **Empty-ribbon left drags change meaning.** A left drag on empty ribbon that moves mostly vertically now switches bands instead of doing nothing. → That is the intended `drag_band` behaviour, and the spec says so.
- **Animations off.** `update` snaps every spring, so the held position draws at once and the release jumps to the target. → The animations spec's "Release with animations off" scenario pins this.

## Migration Plan

None. The gesture is client-side, no message or protocol version changes, and user bindings to `drag_band` gain the vertical axis with no edit.
