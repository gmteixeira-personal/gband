## 1. Tests

- [ ] 1.1 In `crates/core/tests/view.rs`, add `strip_width_change_after_a_lap_keeps_the_travel_on_the_camera`: four 40-cell columns in the 80-column scene, focus left from the first column, check that `(camera, travel)` is `(120, -40)`, sync with a 100-column area and terminal, then check that the fourth column is focused, the camera is 120, `travel` modulo the strip is the camera, `travel` is -80, and `drawn_in` gives the third, fourth and first columns at -20, 30 and 80. Verify with `cargo test -p gband-core --test view lap_keeps` and check that it fails before the fix
- [ ] 1.2 Add `column_width_change_after_a_lap_keeps_the_travel_on_the_camera`: the same start, then set the first column's width to `Proportion::ONE_THIRD` and sync, then check that the fourth column is focused, the camera is 106, `travel` modulo the strip is the camera, and `drawn` gives the fourth, first and second columns at 0, 40 and 66. Verify with `cargo test -p gband-core --test view lap_keeps` and check that it fails before the fix

## 2. Fix

- [ ] 2.1 In `View::aim` in `crates/core/src/view.rs`, on a looping strip, compute `travel + moved - camera`, round its distance from `moved` to the nearest whole number of strip widths, and set `travel` to `moved` plus that many widths. Verify with `cargo test -p gband-core` and `cargo test -p gband-client`, all passing, including both new tests
- [ ] 2.2 Check by hand with `target/debug/gband -S manual attach`: open four windows, give one another width with prefix `r`, move focus left from the first column until it wraps, then toggle the terminal's full screen and grow and shrink the focused column. Check that the focused window keeps both side borders

## 3. Gate

- [ ] 3.1 Run `.claude/flow-gate` and check that fmt, clippy and the workspace tests pass
