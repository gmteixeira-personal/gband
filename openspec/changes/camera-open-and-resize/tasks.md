## 1. Deltas against the current specs

- [ ] 1.1 After merging the latest `origin/dev`, which holds ribbon-screen-area's archive, copy the layout-view requirement "Camera" and the bars requirement "Bars narrow the screen area", whole, from `openspec/specs/` into this change's deltas. Reapply this change's edits: the opened-column copy rule, the never pull-back bullet and the six new Camera scenarios, and "Camera follows the narrower ribbon" on screen columns 50 to 79. Verify with `openspec validate camera-open-and-resize --strict`, with no archive warning.

## 2. Opened column

- [ ] 2.1 In `crates/core/src/view.rs`, when `settle` focuses a window absent from `recency` whose column is next to the previously focused column, pass `follow` the heading toward it and the previous window, so `aim` anchors the copy. Verify with a test in `crates/core/tests/view.rs` for "Opening across the seam": camera 40, the second column on cells 0 to 39 and the new one on 40 to 79.
- [ ] 2.2 Restate the right-hand case of `tests/lua/loop_bands_spec.lua` so the view starts at "2,3" whatever the area's parity, and run it with both the default sidebar and a no-bar configuration. Verify with `gband test tests/lua/loop_bands_spec.lua`.

## 3. Pull back to the strip's end

- [ ] 3.1 In `crates/core/src/view.rs`, in `aim`'s non-looping branch, when not centring, move the camera to `max(0, strip end − viewport)` when that is below the camera. Verify with `crates/core/tests/view.rs` tests for "Closing the last column pulls the camera back", "Shrinking the last column pulls the camera back", "Narrower area pulls the camera back", "Pulled back no further than 0" and "Centred camera keeps its blank strip", and check that every existing view test still passes.
- [ ] 3.2 Restate `camera_follows_the_narrower_ribbon` in `crates/client/tests/bars.rs`: the second tile on screen columns 50 to 79 and no blank cells on the right. Verify with `cargo test -p gband-client --test bars`.

## 4. Whole suite

- [ ] 4.1 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets`. Regenerate any Lua screenshot that changes with `gband test --update`, and check that each diff only moves a tile to the strip's end. Verify that every test passes.
