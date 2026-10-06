## Why

Camera placement leaves the view in a worse spot than the layout allows in two ways. On a looping strip, a window opened right of the focused column can land at the copy on the far left. With no bar on an 80-column terminal, the third half-width window then shows as "3,1" instead of "2,3", because the two copies tie on the camera move and the tie goes to the smaller start. When the strip's end moves left while the focused column stays visible, the `"never"` policy keeps the camera and leaves a blank strip right of the strip's end. This happens when the screen area or the client's ribbon narrows: on 80×24 with a left bar of 20 added beside two half-width columns, the tiles of 30 sit on screen columns 30 to 59 and columns 60 to 79 are blank. It happens too when the last column closes or narrows.

## What Changes

- A column the layout opens beside the previously focused column takes the copy next to that column's drawn copy, on the side it opened, as a focus move left or right already does. The opening tie disappears, and the view shows "2,3" at every width.
- Under `"never"`, and `"on-overflow"` when it moves as `"never"`, a non-looping strip with the camera above 0 never shows cells right of its end. After every change, of focus, of the layout, of the screen area or of the terminal's width, the camera moves left so the strip's end meets the terminal's right edge, but not below 0. Closing or narrowing the last column, adding a bar and narrowing the area all leave no blank strip. Under `"always"`, and after `center_column`, which centre the focused column on purpose, this does not apply.
- The bars scenario "Camera follows the narrower ribbon" is restated to the tile on screen columns 50 to 79 with no blank strip.

## Capabilities

### New Capabilities

### Modified Capabilities
- `layout-view`: the Camera requirement places an opened column next to the previously focused column's drawn copy, and, under the never placement, keeps a non-looping strip's end from leaving blank cells at the terminal's right edge.
- `bars`: the scenario "Camera follows the narrower ribbon" shows no blank strip.
- `mouse`: the release of a band slide that leaves the focused column in view pulls a non-looping strip's camera back to the strip's end, as the never placement does.

## Impact

- Core: `crates/core/src/view.rs` (`settle`, `follow` and `aim` choose the opened column's copy, and `aim` pulls a non-looping strip's camera back under the never placement). Tests in `crates/core/tests/view.rs`.
- Client tests: `crates/client/tests/bars.rs` ("Camera follows the narrower ribbon"), `crates/client/tests/actions.rs` ("Slide past the strip's end pulls back"); Lua spec `tests/lua/loop_bands_spec.lua` (the starting view after opening three windows).
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
