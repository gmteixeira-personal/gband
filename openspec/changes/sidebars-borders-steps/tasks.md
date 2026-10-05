## 1. Resize steps

- [ ] 1.1 Give `Action::StepWidth`, `Action::StepHeight` and their `SessionAction` forms a `by: Proportion` in `crates/core/src/action.rs`. Make `Column::step_width` add or subtract `by` and `Column::step_height` move by `round_half_up(rows × by).max(1)` rows in `crates/core/src/layout.rs`. Pass `by` to the floating-pane width and height sizing. Verify with `crates/core/tests/layout.rs` cases for "Grow by another step", "Height step from the request", "Grow a floating width by another step" and the existing 1/10 cases.
- [ ] 1.2 Carry the step in the grow and shrink messages and raise `PROTOCOL_VERSION` in `crates/protocol/src/message.rs` to one above dev's. Verify the "Grow height round trip" and the version-mismatch cases in `crates/protocol/tests/messages.rs`.
- [ ] 1.3 Apply the received step in `crates/server/src/session.rs`. Verify with a `crates/server/tests/resize.rs` case where a client grows a column of width 1/2 with the step 1/4 and every client receives width 3/4.
- [ ] 1.4 Add the client options `width_step` and `height_step` to `crates/lua/src/options.rs`, with their ranges and fraction reading. Verify "Invalid step" in `crates/lua/tests/options.rs`.
- [ ] 1.5 Accept `step` in the targets of the four grow and shrink actions in `crates/lua/src/actions.rs`. Fill the step from the target, the client's options, or `1/10` on the server. Resolve bound grow and shrink with the client's options in `crates/client/src/bindings.rs`. Verify "Grow by a given step" and "Step out of range" in `crates/lua/tests/control.rs`, a 1/10 server default in `crates/lua/tests/server.rs`, and "Step from the client's option" in `crates/client/tests/actions.rs`.

## 2. Borders

- [ ] 2.1 Add `tile_border_sides`, `tile_border_chars`, `floating_border_sides` and `floating_border_chars` to `crates/lua/src/options.rs`. Sides are held in order with duplicates removed. Characters are the four named sets or eight one-cell strings. Verify "Sides are held in order", "Custom characters" read back, and "Wide character rejected" in `crates/lua/tests/options.rs`.
- [ ] 2.2 Add `draw_border` to `crates/client/src/render.rs`, following the borders spec. Draw tiles with the tile border and floating panes with the floating border, always insetting the interior by one cell. Drop `Block::bordered()` for panes. Verify with snapshots in `crates/client/tests/render.rs` for "Only the left side", "Top and left sides", "No sides", "Tiles without side borders" and "Floating panes differ from tiles".
- [ ] 2.3 Accept a border table in `gband.win.open` and `set_config` in `crates/lua/src/runtime/gband/win.lua`, carried as `Option<Border>` in `FloatFrame` in `crates/lua/src/windows.rs`. Draw it with `draw_border`. Verify "Invalid border table" in `crates/lua/tests/windows.rs`, and "Float with a rounded top only" and "Title over an undrawn top side" as snapshots in `crates/client/tests/render.rs`.
- [ ] 2.4 Verify "Two clients with different borders" in `tests/windows.rs`: two attached clients with the same terminal size and different `tile_border_sides` show the same grid, and only one draws border characters.

## 3. Side bars

- [ ] 3.1 Add `crates/lua/src/bars.rs` with the pure `place(bars, terminal)` function, plus `host.place_bars`, `host.present_bars` and the `bars_resized` hook. Verify unit tests for "Left bar", "Two bars on one side", "Bars on both sides" and "Bar too wide".
- [ ] 3.2 Add `crates/lua/src/runtime/gband/bar.lua` with `gband.bar.add`, `set_lines`, `set_config`, `remove`, `info` and `list`. Cover validation, plugin namespacing, removal on reload and on a failed plugin, `on_resize` after loading, and the `Bar` group default. Register the module in `crates/lua/src/bundled.rs` and `crates/lua/src/lib.rs`. Verify every scenario of the bars spec that needs no drawing in a new `crates/lua/tests/bars.rs`.
- [ ] 3.3 Add `bar` and `errors` to the client-only fields in `crates/lua/src/guard.rs`. Verify "Bars in the server" in `crates/lua/tests/plugins.rs`.
- [ ] 3.4 Make `Display` in `crates/client/src/lib.rs` report the terminal size and take its ribbon from `bars::place`. Give the view the ribbon's size, and offset drawing by the ribbon's left column. On a bar change, snap and sync the view without sending a resize. Replace `crates/client/src/placement.rs` with the bar placement. Verify "Bar added by a key", "Camera follows the narrower ribbon", "Ribbon right of a left bar" and "Moving the status line" in a new `crates/client/tests/bars.rs`.
- [ ] 3.5 Draw shown bars in `crates/client/src/render.rs`, styled as the bar contents rule defines. Verify "Styled bar" and "Line cut at the bar's edge" as snapshots in `crates/client/tests/render.rs`.
- [ ] 3.6 Set `Bar` in `crates/lua/src/runtime/gband/colors/default.lua`. Verify in `crates/lua/tests/colorschemes.rs` that the default colorscheme sets every built-in group, `Bar` included.

## 4. Status line on a side bar

- [ ] 4.1 Remove `statusline_position`, `statusline_height` and `statusline_separator` from `crates/lua/src/options.rs`. Remove `StatusLine`, `host.present` and `take_line` from `crates/lua/src/ui.rs` and `crates/lua/src/runtime.rs`, and `StatusArea` and `draw_status` from `crates/client/src/render.rs`. Verify "Invalid status line height" in `crates/lua/tests/options.rs` and a clean `cargo build --workspace`.
- [ ] 4.2 Turn `crates/lua/src/runtime/gband/statusline.lua` into the `gband.statusline` plugin:
  - setup options `side`, `min_width`, `max_width` and `order`, and the bar `statusline`;
  - top, center and bottom alignment, `{ lines = ... }` output, and vertical layout with priority dropping;
  - the clamped width, the context's `total_height` and `height`, fill components that do not widen the bar, and triggers on height changes;
  - the error item `error`, reported through `host.error_item`.

  Verify every scenario of the status-line delta in `crates/lua/tests/statusline.rs`.
- [ ] 4.3 Move `crates/lua/src/runtime/gband/statusline/band.lua`, `mode.lua`, `position.lua` and `clock.lua` to top and bottom alignment. Make `hints.lua` align to the top and wrap its hints into lines within `width` and `height`. Verify "Reorder a segment" and "Clock" in `crates/lua/tests/statusline.rs`, and "Component entry", "Prefix table with the defaults", "Some hints left out", "All hints fit", "Nothing fits" and "No rows left" in `crates/lua/tests/key_hints.rs`.
- [ ] 4.4 Draw the banner in `crates/client/src/render.rs` and `crates/client/src/lib.rs` only when an error is reported and no error item is drawn. Verify "Plugin error on the banner" and "Banner without a status line" in `crates/client/tests/statusline.rs`.
- [ ] 4.5 Rewrite `crates/client/tests/statusline.rs` and `tests/statusline.rs` for the side-bar status line. Verify "Default placement", "Right side", "Terminal too short", "Turning the status line off" and "Default segments" pass.

## 5. Error list

- [ ] 5.1 Keep the ordered error list beside the latest error in `crates/client/src/lib.rs`. Push it to Lua through `crates/lua/src/ui.rs`, and expose `gband.errors()` in `crates/lua/src/api.rs`. Verify "Errors kept in order", "Syntax error" and "Plugin error in the status line" in `crates/lua/tests/config.rs`.
- [ ] 5.2 Add the bundled plugin `crates/lua/src/runtime/gband/errors.lua`:
  - the `kind` option, the action and command `errors.open`, and the command's `kind` argument;
  - the float or pane window, entering `root`, and wrapping by display width, again on resize;
  - `no errors`, focusing an open list, and closing with `q`.

  Register it in `crates/lua/src/bundled.rs`. Verify every error-list scenario in a new `crates/lua/tests/errors.rs`.
- [ ] 5.3 Set up `gband.errors` and `gband.statusline` in `crates/lua/src/defaults.lua`, in the order the configuration delta gives. Verify "Defaults reproduce the built-in behaviour" and "Copied defaults load unchanged" in `crates/lua/tests/config.rs`.

## 6. Docs and examples

- [ ] 6.1 Update `README.md` and `docs/plugins.md`:
  - side bars and `gband.bar`;
  - the `gband.statusline` setup options in place of the removed options, and vertical alignment with `{ lines = ... }` output;
  - `gband.errors()` and `gband.errors`, with a sample binding for `errors.open`;
  - the border options and the step options.

  Verify `rg 'statusline_(position|height|separator)|align = "(left|right)"' README.md docs` prints nothing.
- [ ] 6.2 Move `examples/plugins/agent-status/client.lua` and `examples/plugins/pane/lua/pane/init.lua` to top or bottom alignment. Verify both load in `crates/lua/tests/plugins.rs` or with their existing example tests.

## 7. Gate

- [ ] 7.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass.
