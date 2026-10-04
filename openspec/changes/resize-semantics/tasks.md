## 1. Width steps

- [ ] 1.1 Add `enum Step { Grow, Shrink }` to `crates/core/src/layout.rs`, and add `Proportion::step`, which adds or subtracts 1/10, clamps to `[1/10, 1]` and reduces by the greatest common divisor. Add `Column::step_width`, which counts full width as `Proportion::WHOLE` and turns it off. Verify with `crates/core/tests/layout.rs` tests for every width scenario of the layout delta: grow from 1/2 twice, grow from 1/3, shrink at 1/10, grow at 1 and shrink from full width
- [ ] 1.2 Add `SessionAction::StepWidth { pane, step }` and handle it in `Layout::apply` through `with_column`. Verify with a `crates/core/tests/events.rs` test that growing a 1/2 column emits one `ColumnWidthChanged` with width 3/5 and full width off, and that a step that changes nothing emits no event

## 2. Pane height weights

- [ ] 2.1 Add `weights: Vec<u8>` to `Column`, and route every change to `panes` in `layout.rs` through `Column` methods that keep both vectors the same length. A pane that opens, is consumed or is expelled takes weight 1. A pane that leaves removes its own weight. Verify that every existing `crates/core/tests/layout.rs` test passes, and add a check that `panes.len() == weights.len()` for every column after each operation of the consume and expel tests, plus the "Consumed pane takes weight 1" scenario
- [ ] 2.2 Change `geometry::tiles` to share a column's height by weight, with leftover rows going one each to the first panes. Verify that every existing `crates/core/tests/geometry.rs` test passes unchanged, and add the "Weighted stack" scenario: weights 2 and 1 on 80×25 give 17 and 8 rows
- [ ] 2.3 Add `SessionAction::StepHeight { pane, step }` and `SessionAction::ResetHeights(PaneId)` to `Layout::apply`, with the cap at 8, the shrink-at-1 rule and the GCD reduction after a grow or shrink. Add `LayoutEvent::PaneHeightsChanged { workspace, column, weights }` and drop `Copy` from `LayoutEvent`. Verify with `crates/core/tests/layout.rs` tests for every "Pane heights" scenario, and a `crates/core/tests/events.rs` test for "Grow a pane's height" that also checks that a change leaving every weight unchanged emits no event

## 3. Actions and the view

- [ ] 3.1 Add `SessionCommand::StepWidth(Step)`, `StepHeight(Step)` and `ResetHeights` to `crates/core/src/action.rs`, and map them in `View::resolve`. Verify with `crates/core/tests/view.rs` tests that each command resolves to the focused pane, and resolves to nothing when no pane is focused
- [ ] 3.2 Replace `Scene::viewport_cols` with `viewport: Size` in `crates/core/src/view.rs`, and update every `Scene` in `crates/core/tests/view.rs`, `crates/client/src/lib.rs` and `crates/client/tests/render.rs`. Verify that `cargo test -p gband-core -p gband-client` passes with no assertion changed
- [ ] 3.3 Add `View::shown(&self, scene) -> Vec<PaneId>`, returning the viewed workspace's panes whose tile has a cell inside the viewport, in layout order. Verify with `crates/core/tests/view.rs` tests for the four "Shown panes" scenarios of the layout-view delta

## 4. Protocol

- [ ] 4.1 Append `ClientMessage::Shown(Vec<PaneId>)` in `crates/protocol/src/message.rs` and set `PROTOCOL_VERSION` to 4. Verify with `crates/protocol/tests/messages.rs` round trips for shown naming panes 1 and 4, for `StepHeight` naming pane 2, and for a layout whose column holds weights 2 and 1. Update any pinned hello bytes to version 4

## 5. Server

- [ ] 5.1 Add `Command::Shown { client, panes }` to `crates/server/src/session.rs`, and keep each client's shown set in the session, dropping identifiers the layout does not hold. In `crates/server/src/connection.rs`, `dispatch` forwards `ClientMessage::Shown`, and `Attachment::drop` sends an empty set for its client. Verify that `cargo test -p gband-server` compiles and that the existing tests not about sizes pass
- [ ] 5.2 Add `SETTLE` (100 ms) and `settle_at: Option<Instant>` to `Session`. Set the deadline on every change of the area, the layout or a client's shown set. Remove the resize loop from `publish`, add `Session::settle`, which resizes only shown panes, and add a `sleep_until` arm to `drive` while a deadline is set. Verify with the tests in 6.2
- [ ] 5.3 Give `TestClient` in `crates/test-support/src/lib.rs` `show(&[PaneId])` and `show_all()`. Make the size-asserting tests in `crates/server/tests/panes.rs` and `crates/server/tests/sessions.rs` call `show_all()` after attaching and after opening panes. Verify that `cargo test -p gband-server` passes, including `latest_client_sets_the_area` and `opening_a_pane_keeps_other_sizes_and_width_changes_resize`

## 6. Settled and shown-only resizes

- [ ] 6.1 Add a SIGWINCH counter helper to `crates/server/tests/resize.rs`: a bash pane that runs `trap 'echo W >> <file>' WINCH` and loops on `read -t 1`, with a reader that counts the file's lines once the pane's size has settled. Verify that it counts exactly one after a single resize
- [ ] 6.2 Add tests to `crates/server/tests/resize.rs` for each "Pane resizes" scenario: a burst of three resizes within 100 ms gives one SIGWINCH and a 48×28 PTY; four grow-width actions within 100 ms give one layout per action and one SIGWINCH; an unshown third column keeps 38×22 with no SIGWINCH while the shown two become 48×28; showing it later resizes it; panes shown by different clients both resize; and a pane left alone in a stack while detached keeps its size until a client shows it. Verify that `cargo test -p gband-server --test resize` passes ten times in a row

## 7. Client

- [ ] 7.1 Bind `-`, `=`, `_`, `+` and `R` in `crates/client/src/bindings.rs` to `StepWidth(Shrink)`, `StepWidth(Grow)`, `StepHeight(Shrink)`, `StepHeight(Grow)` and `ResetHeights`. Verify by extending `table_keys_follow_the_spec` and `every_table_entry_names_an_action_of_its_kind`, and with a test that Shift+`+` and plain `+` both grow the height
- [ ] 7.2 Keep the last shown set in `Display` in `crates/client/src/lib.rs`. After each batch of server messages, each key event and each terminal resize, send `ClientMessage::Shown` when `View::shown` differs from it. Verify with a `crates/client/tests/actions.rs` test against a test server: the first shown message after attaching to three 1/2 columns on 80×24 names the first two, focusing the third sends the second and third, and focusing within an already shown column sends nothing
- [ ] 7.3 Add `crates/client/tests/render.rs` snapshot tests for "Grid smaller than its tile" (38×22 grid in a 50×30 tile) and "Grid larger than its tile" (48×28 grid in a 40×24 tile), checking that the border follows the tile and the cursor is hidden outside the interior. Verify that they pass against the existing renderer with no change to `render.rs`

## 8. Integration

- [ ] 8.1 Add a `tests/attach.rs` test for "Grow the column": in an 80×24 outer terminal, Ctrl+A then `=` makes the tile 48 columns wide, and `tput cols` then prints `46`. Verify that the existing "Resize reaches the program" test still passes
- [ ] 8.2 Verify that `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` pass
- [ ] 8.3 Manually verify in a real terminal: dragging the terminal edge with nvim open in a pane gives one redraw at the end; pressing Ctrl+A then `=` several times in quick succession grows the column at each press with one SIGWINCH at the end; a column scrolled offscreen keeps its old size until focused
