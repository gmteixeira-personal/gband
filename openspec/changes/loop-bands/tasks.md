## 1. Reconcile the specs

- [ ] 1.1 Once `floating-windows` is archived, compare this change's "Focus across columns", "Camera" and "Shown panes" deltas with those requirements in `openspec/specs/layout-view/spec.md` on `<base>`, carry over any wording `floating-windows` changed after this proposal, and verify with `openspec validate loop-bands --strict`
- [ ] 1.2 Compare this change's "Options" delta with `openspec/specs/configuration/spec.md` on `<base>`, keep every option row and type phrase that has landed since, such as those of `sidebars-borders-steps`, and verify with `openspec validate loop-bands --strict`

## 2. Option

- [ ] 2.1 Add the client option `loop_bands`, a boolean defaulting to `true`, to `crates/lua/src/options.rs`, with its `gband.set` patch field, its `gband.opt` read and reset, and its `gband.opt.list()` entry, and verify with a test in `crates/lua/tests/options.rs` covering the default, `gband.set { loop_bands = false }` and the wrong-type fallback
- [ ] 2.2 Set `gband.opt.loop_bands = true` in `crates/lua/src/defaults.lua` and verify that the copied-defaults test in `crates/lua/tests/config.rs` still passes
- [ ] 2.3 Add `View::set_loop_bands` in `crates/core/src/view.rs` and pass the option from `Client::configure` in `crates/client/src/lib.rs`, and for a newly created view, and verify with a client test that `loop_bands = false` in `init.lua` stops focus at the last column

## 3. Focus goes round

- [ ] 3.1 Make `View::neighbour_column` go from the last column to the first and from the first to the last while `loop_bands` is on, leaving a one-column band unchanged, and verify with tests in `crates/core/tests/view.rs` for each "Focus across columns" scenario, including the short band and looping off
- [ ] 3.2 Set `loop_bands` off in existing tests in `crates/core/tests/view.rs` and `crates/client/tests/actions.rs` that assert focus stops at an edge, and verify `cargo test -p gband-core -p gband-client` passes

## 4. Looping camera

- [ ] 4.1 Add the loop test, strip width less the widest column at least the viewport width, beside `column_spans`, and verify with a test for three 40-cell columns on 80 cells, which loop, and two, which do not
- [ ] 4.2 In `View::follow`, pick the focused column's copy as the Camera requirement defines, the neighbour of the previous drawn copy on a left or right move and otherwise the smallest move, then run `centred` or `revealed` on that copy, and verify with tests for every "Camera" scenario across the seam under `"never"`, `"always"` and `"on-overflow"`
- [ ] 4.3 Hold the camera in `0..strip_width` while looping and keep the unreduced travelled camera beside it, reset when the band stops looping, and verify with tests that two right moves from the third column read 80 then 0 from `View::camera()`, and 80 then 120 as the travelled camera
- [ ] 4.4 Make `View::shown` use each tile's drawn copy while looping, and verify with the "First column shown after the last" scenario

## 5. Drawing and animation

- [ ] 5.1 Carry the looping strip width in `animation::Targets` and `DrawnBand`, target the spring at the travelled camera, and re-base the spring by whole strip widths when a band stops looping, and verify with tests in `crates/client/tests/animation.rs` for the two "Camera scroll" seam scenarios, asserting no frame scrolls against the move
- [ ] 5.2 Make `render::Placement::new` place each tile at its drawn copy while looping, and verify with a render snapshot in `crates/client/tests/render.rs` of three 1/2 columns on 80×24 with the camera at 80, showing the third column then the first
- [ ] 5.3 Verify the floating layer stays put across the seam with a render test of a floating box at column 10 while the camera scrolls from 40 to 80

## 6. Documentation and gate

- [ ] 6.1 Add `loop_bands` to the client options table in `README.md` and to the client option list in `docs/plugins.md`, and verify both name the default `true`
- [ ] 6.2 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`, and verify both pass
- [ ] 6.3 Attach a client to a session with three panes, press Ctrl+Space then `l` repeatedly from the last column and Ctrl+Space then `h` from the first, and verify the strip scrolls without end in each direction with no pane drawn twice
