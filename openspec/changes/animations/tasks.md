## 1. Spring

- [ ] 1.1 Add `crates/client/src/animation.rs` with a closed-form critically damped `Spring` (`ω = √800`): start, value and velocity at an `Instant`, retarget from the current value and velocity, rest after 400 ms or within half a cell below 10 cells per second, and rounding to whole cells. Verify with tests in `crates/client/tests/animation.rs`: a 40-cell move from rest draws 0 at the start, 17 at 50 ms and 37 at 150 ms, is at rest on 40 by 400 ms, and never decreases.
- [ ] 1.2 Test retargeting in `crates/client/tests/animation.rs`: a spring moving from 0 toward 40, retargeted to 0 at 20, draws within one cell of its pre-retarget value on the next sample and later rests on 0.

## 2. Presentation state

- [ ] 2.1 Add `Presentation` to `animation.rs`: the drawn camera, the drawn vertical position, per-pane `x`, `y`, `width` and `height` springs for the drawn workspace, and the `(WorkspaceId, camera)` of a workspace being left. `update(now, targets)` retargets changed springs, places new panes at rest, drops absent ones, and `is_animating()` reports any spring in motion. Verify with tests in `crates/client/tests/animation.rs` for a camera move, a pane that slides after an open, a pane dropped after a close, and a width change.
- [ ] 2.2 Handle the workspace cases in `update`: a switch sets the vertical target to `index × terminal rows` and records the left workspace and its drawn camera; an index shift of the same viewed workspace moves value and target together; a viewed workspace whose predecessor left the layout snaps. Verify with tests for a switch down, two switches in a row, a workspace removed above the viewed one, and the left workspace removed mid-switch.
- [ ] 2.3 Add a snap request that puts every spring at rest on its target at the next `update`, and a disabled mode in which every `update` snaps. Verify with tests that a snapped or disabled presentation draws the targets and reports no animation.
- [ ] 2.4 Report whether the drawn camera, the vertical position and the focused pane's springs are all at rest, for cursor placement. Verify with a test that a scrolling camera reports motion and a settled one does not.

## 3. Rendering

- [ ] 3.1 Change `Ribbon` and `render` in `crates/client/src/render.rs` to draw from drawn values: bands of workspaces with a top row and a camera, tiles at drawn rectangles cut to their band and the terminal, the focused tile drawn last. Verify the existing snapshots in `crates/client/tests/render.rs` pass unchanged with a presentation at rest.
- [ ] 3.2 Draw a tile's border at its drawn size and pass the drawn interior to the grid drawing resize-semantics adds, so the grid is cut or blank-padded at the moving border. Verify with new snapshots: a tile morphing wider than its grid, and one narrower than its grid.
- [ ] 3.3 Hide the cursor while the presentation reports the camera, the vertical position or the focused pane in motion. Verify with the `Mid-scroll frame` snapshot: the second of three 40-cell tiles fills screen columns 20 to 59 with the camera drawn at 20, and no cursor.
- [ ] 3.4 Add snapshots of a workspace switch mid-slide, with the bottom of W1 above the top of W2, and of two overlapping tiles where the focused one is on top.

## 4. Client loop

- [ ] 4.1 Keep a `Presentation` in `Display` in `crates/client/src/lib.rs`, compute targets from the layout, the view and the terminal before each draw, and call `update(now, targets)`. Request a snap on the first view, on a layout whose area differs from the previous one, and on a terminal resize. Update `crates/client/tests/actions.rs` for any changed `Display` constructor and verify `cargo test -p gband-client` passes.
- [ ] 4.2 Add a frame timer branch to the attach loop's `tokio::select!`, armed only while `is_animating()`, firing 16 ms after the last draw. Verify by running the client with `GBAND_LOG=debug` that frames stop once the camera settles, and that `cargo test` passes.
- [ ] 4.3 Parse `GBAND_ANIMATIONS` in `gband_client::run` with a pure function over `Option<&str>`: unset or `on` enables, `off` disables, any other value logs a warning naming it and enables. Verify with unit tests of the parser in `crates/client/tests/animation.rs`.

## 5. End to end

- [ ] 5.1 Add a test to `tests/attach.rs` that attaches with `GBAND_ANIMATIONS=fast` through `Attached::start_with` and finds a warning naming `fast` in the client log, and one that attaches with `GBAND_ANIMATIONS=off`, opens a second pane and sees the settled two-tile screen. Verify with `cargo test --test attach`.
- [ ] 5.2 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` and verify both pass, with the existing end-to-end tests unchanged.
- [ ] 5.3 Attach by hand and check the camera scroll, a workspace switch, opening and closing a column, consume and expel, and cycling a width all move smoothly and settle, and that `GBAND_ANIMATIONS=off` restores instant jumps.
