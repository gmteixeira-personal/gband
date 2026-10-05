## 0. Specs

- [ ] 0.1 For each MODIFIED requirement in this change's delta specs, compare the block with the current `openspec/specs/<capability>/spec.md` block of the same requirement, as rename-panes-to-windows and the changes archived before this one left it, and fold in every requirement text and scenario this change did not write. Verify that the only differences left are this change's own and that `openspec validate center-column --strict` passes

## 1. Core view

- [ ] 1.1 Add `ViewAction::CenterColumn` to `crates/core/src/view.rs`. In `View::apply`, locate the focused window's column in the viewed band, take its span from `column_spans(band, scene.area)` and set the band's camera to `centred(span, viewport)`, then settle as for every other action. Leave the view unchanged when no window is focused. Verify with `crates/core/tests/view.rs` tests for every scenario of the layout-view "Center the focused column" requirement, under the `"never"` policy and, for "Center a column at the right edge", under `"on-overflow"` too.

## 2. Lua and default bindings

- [ ] 2.1 Add `center_column` to `ACTIONS` in `crates/lua/src/actions.rs`, after `focus_band_up`, as `Action::View(ViewAction::CenterColumn)` with the description `center the focused column`. Verify that `crates/lua/tests/config.rs`'s full action list, extended with `center_column`, passes, and that the server's `gband.action.center_column` raises the client-side error, as `sides.rs` already does for view actions.
- [ ] 2.2 Bind `prefix c` to `action.center_column` in `crates/lua/src/defaults.lua`, after the `prefix i` binding. Add `('c', Action::View(ViewAction::CenterColumn))` to `default_keys_follow_the_spec` in `crates/client/src/bindings.rs`. Verify that test passes with the prefix table one entry longer.
- [ ] 2.3 Add `center_column = "center"` to the labels in `crates/lua/src/runtime/gband/statusline/hints.lua`. Update `PREFIX_HINTS` in `crates/lua/tests/key_hints.rs` to show `c center` after `i band up`, and add a test for the "Center label" scenario. Verify with `cargo test -p gband-lua key_hints`.
- [ ] 2.4 Add a test to `crates/client/tests/actions.rs` that a binding naming `gband.action.center_column` moves the camera of a three-column display and sends nothing to the server. Verify with `cargo test -p gband-client actions`.

## 3. End to end and docs

- [ ] 3.1 Add `leader_c_centers_the_column` to `tests/attach.rs`: on an 80×24 terminal with one window of the default width, send `\x00c` and wait for a single tile whose left edge is column 20. Verify that the test passes.
- [ ] 3.2 Add `center_column` to the action table in `README.md` and to the short label table in `docs/plugins.md`, keeping its two column pairs balanced. Verify by reading the rendered tables.
- [ ] 3.3 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. Verify all three pass.
