## 1. Core layout: floating layer and moves

- [ ] 1.1 Add `FloatingPane` and `Band::floating` to `crates/core/src/layout.rs`. Make `Band::is_empty`, `Band::panes`, `Layout::panes` and `Layout::contains` cover floating panes, and add `Layout::place` returning `Tiled(Location)` or `Floating { band, index }`. Verify with tests in `crates/core/tests/layout.rs` for "Float the only pane of a band", "Floating pane keeps its band" and "Floating pane opened in the empty band".
- [ ] 1.2 Add `geometry::placed`, `height_step` and `width_step` in `crates/core/src/geometry.rs`, and move the tiled height step onto `height_step`. Verify with `crates/core/tests/geometry.rs` cases for every "Box geometry" scenario, and with the existing height tests passing unchanged.
- [ ] 1.3 Extract the cycle, step and set width rules into free functions over `(Proportion, bool)` used by both `Column` and `FloatingPane`. Verify that the existing "Column widths" tests pass, and that new tests cover "Cycle a floating width".
- [ ] 1.4 Implement `ToggleFloating { pane, after }` in both directions, with the first-float size taken from the tile, centring, the `Layout::boxes` memory, and tiling after `after` or as the band's first column. Verify with tests for every scenario of "Float a pane" and "Tile a pane".
- [ ] 1.5 Extend `OpenPane` with `floating`, placing the new pane at the default width, half the area's rows, centred. Verify with the "Open a floating pane" layout test.
- [ ] 1.6 Route the width and height actions and `SetHeight`/`SetWidth` through `Layout::place`, applying the floating rules to boxes. Make consume or expel a no-op on a floating pane. Verify with tests for every "Size a floating pane" scenario.
- [ ] 1.7 Add `MoveColumn` and `MovePane`, swapping columns or rows for tiled panes with heights travelling, and stepping the box for floating panes. Add `SetPosition`. Verify with tests for "Move a column", "Move a pane within its column", "Move a floating pane" and "Place exactly".
- [ ] 1.8 Add `ColumnMoved`, `PaneFloated`, `PaneTiled` and `FloatingBoxChanged` to `crates/core/src/event.rs`, and emit them with the swap's two `PaneMoved`. Verify with `crates/core/tests/events.rs` cases for every new scenario of the session-events delta.
- [ ] 1.9 Audit every `Layout::locate` caller across the workspace (`rg -n 'locate\('`) and switch to `place` or `contains` where floating panes must count. Verify that `cargo test -p gband-core` passes and that no caller drops actions on a floating pane.

## 2. Core view: layers, focus and stacking

- [ ] 2.1 Replace `BandView` with `{ layer, tiled, floating, camera }` in `crates/core/src/view.rs`. Make `focused()`, `enter`, `focus_pane` and `view_band` honour layers, including the remembered layer and the band holding only floating panes. Verify with `crates/core/tests/view.rs` cases for "Layers of a view" and the modified "Switch band" scenarios.
- [ ] 2.2 Add `ViewAction::SwitchLayer` and directional focus between floating panes on doubled centres, with the tie rules. Verify with tests for every "Switch layers" and "Focus between floating panes" scenario.
- [ ] 2.3 Extend `settle` for panes changing layer, a closed focused floating pane, and an empty active layer. Verify with tests for every scenario of "Focus follows a pane between layers" and "Follow layout changes".
- [ ] 2.4 Make the camera follow the tiled focus, and make `resolve` use the tiled focus for `OpenPane` and `ToggleFloating`. Verify with the "Floating focus keeps the camera", "Open pane from the floating layer" and "Tile the focused floating pane" tests.
- [ ] 2.5 Add floating boxes to `View::shown`, and add `View::stacking()` built from `recency`. Verify with the "Floating pane shown", "Box below a small terminal" and every "Stacking" scenario tests.

## 3. Protocol and server

- [ ] 3.1 Bump `PROTOCOL_VERSION` to 6 in `crates/protocol/src/message.rs`. Add round-trip tests in `crates/protocol/tests/messages.rs` for the new actions, `OpenPane.floating`, and a layout with floating panes. Verify with `cargo test -p gband-protocol`.
- [ ] 3.2 Resize shown floating panes from `placed()` in `Session::settle`, `crates/server/src/session.rs`, and start a floating pane's PTY at its box size. Verify with a `crates/server/tests/resize.rs` case for "Floating pane takes its box size".
- [ ] 3.3 Log the new layout events in the server's session-event formatting, and send the focus reply for a pane opened floating. Verify with `crates/server/tests/panes.rs` cases for "Open a floating pane with focus" and "Float a pane for every client", and a `crates/server/tests/events.rs` case for the new events.

## 4. Client rendering

- [ ] 4.1 Draw floating panes in `crates/client/src/render.rs` after the tiles and before the `gband.win` floats, in `View::stacking()` order, ignoring the camera, with the focused border style. Verify with `crates/client/tests/render.rs` snapshots for "Floating pane over two tiles", "Floating pane ignores the camera" and "Float over a floating pane".
- [ ] 4.2 Hide the cursor when a floating pane drawn above the focused pane covers its cell. Verify with a render test for "Cursor under a floating pane".
- [ ] 4.3 Keep animation springs to tiles only, and make panes changing layer appear at rest. Verify with `crates/client/tests/animation.rs` cases for "Float a pane between two columns" and "Move a floating pane".
- [ ] 4.4 Wire `ViewAction::SwitchLayer` and the new session commands through `crates/client/src/lib.rs` dispatch. Verify with a `crates/client/tests/actions.rs` case resolving each new action against a view.

## 5. Lua surface

- [ ] 5.1 Add the six built-in actions to `ACTIONS` in `crates/lua/src/actions.rs` with the delta's names and descriptions. Verify with `crates/lua/tests/config.rs` cases for "Every action is named" and "Toggle floating by name".
- [ ] 5.2 Extend targets in `crates/lua/src/control.rs`: `pane` for the move actions, `{ pane, after }` for `toggle_pane_floating`, and `floating` for `open_pane` with its `after` error. Verify with `crates/lua/tests/control.rs` cases for every new "Action targets" scenario.
- [ ] 5.3 Add `floating` to each band of `gband.layout()` and to `gband.view()`. Verify with the "Floating pane" and "Floating focus" control tests.
- [ ] 5.4 Add `gband.pane.set_position` with its validation and dispatch order. Verify with tests for every "Place a floating pane" scenario and the "Dispatch order" rule.
- [ ] 5.5 Add the ten default bindings after `prefix R` in `crates/lua/src/defaults.lua`: `prefix v`, `prefix V`, `prefix ctrl+h/l/j/k` and `prefix ctrl+left/right/down/up`, each with its action's description. Verify that "Defaults reproduce the built-in behaviour" and "Every default binding is described" pass.
- [ ] 5.6 Add the six short labels to `crates/lua/src/runtime/gband/statusline/hints.lua`. Verify with a `crates/lua/tests/key_hints.rs` case for "Floating keys".

## 6. End-to-end and docs

- [ ] 6.1 Add `tests/floating.rs`, driving the binary for "Float the focused pane", "Switch to the tiled layer and back", "Move a column with Ctrl+H and Ctrl+Right", "Move a pane with Ctrl+J" and "Move a floating pane". Verify with `cargo test --test floating`.
- [ ] 6.2 Document the new actions and keys in the `README.md` action table, and `open_pane`'s `floating`, `toggle_pane_floating`'s target, `gband.pane.set_position` and the new `gband.layout()`/`gband.view()` fields in `docs/plugins.md`, including the hints label table. Verify by reading both against the specs.
- [ ] 6.3 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, and confirm that all pass.
- [ ] 6.4 Run a manual check in a real terminal: float a pane with Ctrl+Space then `v`, switch layers with `V`, move the box with Ctrl+Space then Ctrl+L and Ctrl+Right, resize it with `=` and `+`, tile it again, and attach a second client to confirm independent stacking. Record the result.
