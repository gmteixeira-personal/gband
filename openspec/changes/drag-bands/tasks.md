## 1. Axis lock

- [ ] 1.1 In `crates/client/src/mouse.rs`, turn `Motion::Slide` into `Slide { travel, axis: Option<Axis> }` with `Axis::Horizontal` and `Axis::Vertical { band }`, lock the axis in `gesture_motion` on the first motion with `|dx| >= 2` or `|dy| >= 1` (horizontal when `|dx| >= 2 * |dy|`), do nothing before the lock, and keep `release_slide` for no axis and the horizontal axis; verify `slide_the_band_and_settle` and `partly_shown_column_snaps_after_a_slide` in `crates/client/tests/actions.rs` still pass unchanged
- [ ] 1.2 Add the "Horizontal axis ignores vertical movement" and "Small movement locks no axis" scenarios to `crates/client/tests/actions.rs`, and rewrite `left_drag_on_empty_ribbon_slides_and_vertical_motion_does_not` so its first motion is sideways and a later vertical motion leaves the camera alone; verify with `cargo test -p gband-client --test actions`

## 2. Holding the bands

- [ ] 2.1 In `crates/client/src/animation.rs`, add `Hold::vertical: Option<HeldBands>` with the drawn top and an optional peek band's id, camera and strip; make `Presentation::update` set the `vertical` spring at rest at the held top and `drawn()` emit the peek band at its own index; verify with a test in `crates/client/tests/animation.rs` that a held top of 6 on a 24-row band height draws B1 from row -6 and B2 from row 18
- [ ] 2.2 Add `Presentation::release_bands`, which moves the held peek band onto `leaving` with its drawn camera and lets `vertical` retarget from the held top; verify with animation tests for "Release continues the switch" (B2 starts at row 8 on the first frame and rises to 0, B1 keeping its camera) and "Release slides back" (B1 returns to row 0 and B2 is no longer drawn once at rest)
- [ ] 2.3 Verify with an animation test that with animations off the release of a held top of 16 draws only the newly viewed band on the next frame

## 3. Vertical drag

- [ ] 3.1 In `gesture_motion`'s vertical branch, recompute the viewed band's index from the layout by `BandId`, set the held top to `index * band_height - dy` clamped to `0..=last_index * band_height`, and fill the peek from a cloned `View` with `FocusBandDown` or `FocusBandUp` applied; verify with an actions test that dragging from row 20 to row 4 holds top 16, sends nothing, and leaves B1 viewed and focused
- [ ] 3.2 In `end_gesture`'s vertical branch, compute `(top + band_height / 2) / band_height`, clear the hold, apply `FocusBandDown` or `FocusBandUp` when it names the adjacent band, and call `release_bands` otherwise; verify with actions tests for "Drag up to the band below", "Short drag returns", "Drag down to the band above", "Vertical axis ignores sideways movement" and "First band stops the drag"
- [ ] 3.3 In `Display::end_gesture_for_layout` (`crates/client/src/lib.rs`), end a vertical slide whose band left the layout and clear the vertical hold; verify with an actions test that removes the viewed band mid-drag and finds no gesture and no hold afterwards
- [ ] 3.4 Verify with an actions test that a left drag with `drag_window` on empty ribbon from row 22 to row 2 views B2 and sends no message

## 4. Descriptions and end-to-end

- [ ] 4.1 Change `drag_band`'s description to `slide the band or switch bands with the mouse` in `crates/lua/src/actions.rs` and the `middlemouse` binding's `desc` in `crates/lua/src/runtime/gband/keystyle/modal.lua`; verify with `cargo test -p gband-lua`
- [ ] 4.2 Add an end-to-end test to `tests/mouse.rs` for "Middle drag up switches bands in navigation mode": with the modal key style and two bands holding windows, a middle drag from the bottom row to the top row views the second band and navigation mode stays active; verify with `cargo test --test mouse`
- [ ] 4.3 Update the mouse table and `drag_band` row in `README.md` and the drag actions paragraph in `docs/plugins.md` to say a middle drag slides the band sideways or switches bands vertically, whichever axis the drag starts on; verify by reading both against the mouse spec
- [ ] 4.4 Run `cargo clippy --workspace --all-targets` and `cargo test --workspace`, and check that every scenario of the change's delta specs maps to a passing test
