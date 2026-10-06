## Why

Two camera placements leave the view in a worse spot than the layout allows. On a looping strip, a window opened right of the focused column can land at the copy on the far left. With no bar on an 80-column terminal, the third half-width window then shows as "3,1" instead of "2,3", because the two copies tie on the camera move and the tie goes to the smaller start. After the screen area or the client's ribbon narrows, for example when a bar is added beside two half-width columns, the `"never"` policy keeps the camera while the focused column stays visible. A blank strip is then left right of the strip's end: on 80×24 with a left bar of 20, the tiles of 30 sit on screen columns 30 to 59 and columns 60 to 79 are blank.

## What Changes

- A column the layout opens beside the previously focused column takes the copy next to that column's drawn copy, on the side it opened, as a focus move left or right already does. The opening tie disappears, and the view shows "2,3" at every width.
- After a change of the screen area or of the client's terminal width, when the strip does not loop, the camera is above 0, and the view shows cells right of the strip's end, the camera moves left so the strip's end meets the terminal's right edge, but not below 0. Under `"always"`, which centres the focused column on purpose, this does not apply.
- The bars scenario "Camera follows the narrower ribbon" is restated to the tile on screen columns 50 to 79 with no blank strip.

## Capabilities

### New Capabilities

### Modified Capabilities
- `layout-view`: the Camera requirement places an opened column next to the previously focused column's drawn copy, and pulls the camera back after an area or terminal width change that leaves a blank strip right of a non-looping strip.
- `bars`: the scenario "Camera follows the narrower ribbon" shows no blank strip.

## Impact

- Core: `crates/core/src/view.rs` (`settle`, `follow` and `aim` choose the opened column's copy, and the view remembers the last area and viewport it saw so it can tell an area change). Tests in `crates/core/tests/view.rs`.
- Client tests: `crates/client/tests/bars.rs` ("Camera follows the narrower ribbon"); Lua spec `tests/lua/loop_bands_spec.lua` (the starting view after opening three windows).
- This change depends on ribbon-screen-area. Both modify the layout-view Camera requirement, which ribbon-screen-area changes to loop at the terminal's width less one, and ribbon-screen-area adds the bars requirement "Bars narrow the screen area". The deltas for both requirements are therefore re-copied from `origin/dev` after ribbon-screen-area archives, at implementation time.

## Coordination

### Author
- gmteixeira

### Depends On
- ribbon-screen-area

### Expected Files
- openspec/changes/camera-open-and-resize/
- crates/core/src/view.rs
- crates/core/tests/view.rs
- crates/client/tests/bars.rs
- tests/lua/loop_bands_spec.lua
- tests/lua/screenshots/
