## 1. Reconcile the specs

- [ ] 1.1 Compare each MODIFIED requirement in this change's delta specs with the same requirement in `openspec/specs/` on `<base>`, carry over any wording another change landed since this proposal, and verify with `openspec validate band-ends --strict`

## 2. Option

- [ ] 2.1 Add `BandEnds { Loop, Bounce, Stop }` with serde `kebab-case` and `Loop` as default beside `CenterFocusedColumn` in `crates/core/src/view.rs`, and verify it compiles with `cargo check -p gband-core`
- [ ] 2.2 Replace `loop_bands` with `band_ends` in `crates/lua/src/options.rs`: the field, default, patch, reset, read, name lists and `gband.opt.list()` entry. Verify with tests in `crates/lua/tests/options.rs` for each "Options and their values" scenario on `band_ends`, including the retired `loop_bands` assignment
- [ ] 2.3 Set `gband.opt.band_ends = "loop"` in `crates/lua/src/defaults.lua` and `docs/internals/11-default-configs.md`, and verify the copied-defaults test in `crates/lua/tests/config.rs` and the docs block count still pass

## 3. View

- [ ] 3.1 Replace `View::set_loop_bands` with `View::set_band_ends`. Make `neighbour_column` wrap unless `Stop` and the loop test pass only under `Loop`, and pass the option from `crates/client/src/lib.rs` in `configure` and `sync`. Verify with `cargo check --workspace`
- [ ] 3.2 Move every `set_loop_bands(false)` in `crates/core/tests/view.rs` to `set_band_ends(BandEnds::Stop)`, and verify `cargo test -p gband-core` passes
- [ ] 3.3 Add tests in `crates/core/tests/view.rs` for the bounce scenarios in "Focus across columns" and "Camera": both directions, one column, the camera at 0 and 40, each window drawn once, and the camera at -20 under `"always"`. Verify they pass
- [ ] 3.4 Rename `loop_bands_off_stops_focus_at_the_last_column` in `crates/client/tests/actions.rs` to load `band_ends = "stop"`, add a bounce case that ends on the first window, and verify `cargo test -p gband-client --test actions` passes

## 4. Animation

- [ ] 4.1 Add a test in `crates/client/tests/animation.rs` for "Bounce scrolls back across the band", asserting every frame's drawn camera lies between 40 and 0 and never rises, and verify it passes

## 5. Lua specs

- [ ] 5.1 Rename `tests/lua/loop_bands_spec.lua` to `tests/lua/band_ends_spec.lua`, update its registration in `tests/lua_specs.rs`, switch its stop case to `band_ends = "stop"`, and add a bounce case showing `2,3` then `1,2` after `prefix l` on three windows. Verify with `cargo test --test lua_specs band_ends`

## 6. Documentation and gate

- [ ] 6.1 Replace `loop_bands` with `band_ends` and its three values in `README.md` (the feature list and the client options table), `docs/plugins.md` (the client option list and its paragraph) and `docs/internals/08-key-styles.md` (bands mode steps stop at the ends). Verify `rg loop_bands README.md docs crates tests examples` finds nothing
- [ ] 6.2 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`, and verify both pass
- [ ] 6.3 Attach a client with `gband.opt.band_ends = "bounce"` to a session with three windows, press Ctrl+Space then `l` from the last column, and verify the camera scrolls back to the first column with no window drawn twice
