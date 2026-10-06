## 0. Specs

- [x] 0.1 For each MODIFIED requirement in this change's delta specs, compare the block with the current `openspec/specs/<capability>/spec.md` block of the same requirement, as rename-panes-to-windows and the changes archived before this one left it, and fold in every requirement text and scenario this change did not write. Verify that the only differences left are this change's own and that `openspec validate sidebars-borders-steps --strict` passes

## 1. Resize steps

- [x] 1.1 Give `SessionCommand::StepWidth` and `SessionCommand::StepHeight` a `by: Proportion` in `crates/core/src/action.rs`, and their `SessionAction` forms the same in `crates/core/src/layout.rs`. Make `Column::step_width` add or subtract `by` and `Column::step_height` move by `round_half_up(rows × by).max(1)` rows in `crates/core/src/layout.rs`. Pass `by` to the floating-window width and height sizing. Leave `height_step` and `width_step` in `crates/core/src/geometry.rs` as the fixed tenth that floating moves and a new floating box use. Verify with `crates/core/tests/layout.rs` cases for "Grow by another step", "Height step from the request", "Grow a floating width by another step" and the existing 1/10 cases, and that "First float opens a new box" and the floating move cases still pass.
- [x] 1.2 Carry the step in the grow and shrink messages and raise `PROTOCOL_VERSION` in `crates/protocol/src/message.rs` to one above dev's. Verify the "Grow height round trip" and the version-mismatch cases in `crates/protocol/tests/messages.rs`.
- [x] 1.3 Apply the received step in `crates/server/src/session.rs`. Verify with a `crates/server/tests/resize.rs` case where a client grows a column of width 1/2 with the step 1/4 and every client receives width 3/4.
- [x] 1.4 Add the client options `width_step` and `height_step` to `crates/lua/src/options.rs`, with their ranges and fraction reading. Verify "Invalid step" in `crates/lua/tests/options.rs`.
- [x] 1.5 Accept `step` in the client's targets of the four grow and shrink actions in `crates/lua/src/control.rs`. Fill the step from the target or the client's options. Resolve bound grow and shrink with the client's options in `crates/client/src/bindings.rs`. Verify "Grow by a given step" and "Step out of range" in `crates/lua/tests/control.rs`, and "Step from the client's option" in `crates/client/tests/actions.rs`.
- [x] 1.6 Accept `step` in the server's session targets of the four grow and shrink actions in `crates/lua/src/actions.rs`, as server-runtime's "Server session actions" defines: the client's ranges, an error for a `step` out of range or on any other action, and `1/10` when the target names none. Verify "Grow by a step from the server", "Server default step", "Server step out of range" and "Step on another action" in `crates/lua/tests/server.rs`.

## 2. Borders

- [x] 2.1 Add `tile_border_sides`, `tile_border_chars`, `floating_border_sides` and `floating_border_chars` to `crates/lua/src/options.rs`. Sides are held in order with duplicates removed. Characters are the four named sets or eight one-cell strings. Verify "Sides are held in order", "Custom characters" read back, and "Wide character rejected" in `crates/lua/tests/options.rs`.
- [x] 2.2 Add `draw_border` to `crates/client/src/render.rs`, following the borders spec. Draw tiles with the tile border and floating windows with the floating border, always insetting the interior by one cell. Drop `Block::bordered()` for windows. Verify with snapshots in `crates/client/tests/render.rs` for "Only the left side", "Top and left sides", "No sides", "Tiles without side borders" and "Floating windows differ from tiles".
- [x] 2.3 Accept a border table in `gband.win.open` and `set_config` in `crates/lua/src/runtime/gband/win.lua`, carried as `Option<Border>` in `FloatingFrame` in `crates/lua/src/plugin_windows.rs`. Draw it with `draw_border`. Verify "Invalid border table" in `crates/lua/tests/plugin_windows.rs`, and "Floating plugin window with a rounded top only" and "Title over an undrawn top side" as snapshots in `crates/client/tests/render.rs`.
- [x] 2.4 Verify "Two clients with different borders" in `tests/plugin_windows.rs`: two attached clients with the same terminal size and different `tile_border_sides` show the same grid, and only one draws border characters.

## 3. Side bars

- [x] 3.1 Add `crates/lua/src/bars.rs` with the pure `place(bars, terminal)` function, plus `host.place_bars`, `host.present_bars` and the `bars_resized` hook. Verify unit tests for "Left bar", "Two bars on one side", "Bars on both sides" and "Bar too wide".
- [x] 3.2 Add `crates/lua/src/runtime/gband/bar.lua` with `gband.bar.add`, `set_lines`, `set_config`, `remove`, `info` and `list`. Cover validation, plugin namespacing, removal on reload and on a failed plugin, `on_resize` after loading, and the `Bar` group default. Register the module in `crates/lua/src/bundled.rs` and `crates/lua/src/lib.rs`. Verify every scenario of the bars spec that needs no drawing in a new `crates/lua/tests/bars.rs`.
- [x] 3.3 Add `bar` and `errors` to `CLIENT_ONLY` in `crates/lua/src/sides.rs`. Verify "Bars in the server" in `crates/lua/tests/plugins.rs`.
- [x] 3.4 Make `Display` in `crates/client/src/lib.rs` report the terminal size and take its ribbon from `bars::place`. Give the view the ribbon's size, and offset drawing by the ribbon's left column. On a bar change, snap and sync the view without sending a resize. Replace `crates/client/src/placement.rs` with the bar placement. Verify "Bar added by a key", "Camera follows the narrower ribbon", "Ribbon right of a left bar" and "Moving the status line" in a new `crates/client/tests/bars.rs`.
- [x] 3.5 Draw shown bars in `crates/client/src/render.rs`, styled as the bar contents rule defines. Verify "Styled bar" and "Line cut at the bar's edge" as snapshots in `crates/client/tests/render.rs`.
- [x] 3.6 Set `Bar` in `crates/lua/src/runtime/gband/colors/default.lua`. Verify in `crates/lua/tests/colorschemes.rs` that the default colorscheme sets every built-in group, `Bar` included.

## 4. Status line on a side bar

- [x] 4.1 Remove `statusline_position`, `statusline_height` and `statusline_separator` from `crates/lua/src/options.rs`. Remove `StatusLine`, `host.present` and `take_line` from `crates/lua/src/ui.rs` and `crates/lua/src/runtime.rs`, and `StatusArea` and `draw_status` from `crates/client/src/render.rs`. Verify "Invalid status line height" in `crates/lua/tests/options.rs` and a clean `cargo build --workspace`.
- [x] 4.2 Turn `crates/lua/src/runtime/gband/statusline.lua` into the `gband.statusline` plugin:
  - setup options `side`, `min_width`, `max_width` and `order`, and the bar `statusline`;
  - top, center and bottom alignment, `{ lines = ... }` output, and vertical layout with priority dropping;
  - the clamped width, the context's `total_height` and `height`, fill components that do not widen the bar, and triggers on height changes;
  - the error item `error`, reported through `host.error_item`.

  Verify every scenario of the status-line delta in `crates/lua/tests/statusline.rs`.
- [x] 4.3 Move `crates/lua/src/runtime/gband/statusline/band.lua`, `mode.lua`, `position.lua` and `clock.lua` to top and bottom alignment. Make `hints.lua` align to the top and wrap its hints into lines within `width` and `height`. Verify "Reorder a segment" and "Clock" in `crates/lua/tests/statusline.rs`, and "Component entry", "Prefix table with the defaults", "Some hints left out", "All hints fit", "Nothing fits", "No rows left" and "Registered action" in `crates/lua/tests/key_hints.rs`.
- [x] 4.4 Draw the banner in `crates/client/src/render.rs` and `crates/client/src/lib.rs` only when an error is reported and no error item is drawn. Verify "Plugin error on the banner" and "Banner without a status line" in `crates/client/tests/statusline.rs`.
- [x] 4.5 Rewrite `crates/client/tests/statusline.rs` and `tests/statusline.rs` for the side-bar status line. Verify "Default placement", "Right side", "Terminal too short", "Turning the status line off" and "Default segments" pass.
- [x] 4.6 Move the other tests off the removed options and the bottom-row status line:
  - drop `statusline_position` from `crates/client/tests/plugin_windows.rs`, `tests/floating.rs`, `tests/plugin_testing.rs` and `tests/lua/keylist_spec.lua`, leaving `gband.statusline` out where they turned the status line off;
  - make the "configuration of the case" case in `tests/plugin_testing.rs` set up the status line with `side = "right"` and find it on the last 20 columns, as "Configuration of the case" reads;
  - move the tile columns, `tput` sizes and bottom-row reads of `tests/attach.rs`, `tests/config.rs` and `tests/plugin_runtime.rs` to the 20-column left bar, with errors read as the error item and the agent count read from the bar, and "Height excludes the status line", "Grow the window's height", "Center the column", "Float the focused window" and "Move a floating window" as they now read.

  Verify that `rg 'statusline_(position|height|separator)' tests crates examples` finds only cases that check the options are refused, and that `cargo test --test attach --test config --test floating --test plugin_runtime --test plugin_testing --test plugin_windows` passes.

## 5. Error list

- [x] 5.1 Keep the ordered error list beside the latest error in `crates/client/src/lib.rs`. Push it to Lua through `crates/lua/src/ui.rs`, and expose `gband.errors()` in `crates/lua/src/api.rs`. Verify "Errors kept in order", "Syntax error" and "Plugin error in the status line" in `crates/lua/tests/config.rs`.
- [x] 5.2 Add the bundled plugin `crates/lua/src/runtime/gband/errors.lua`:
  - the `kind` option, the action and command `errors.open`, and the `kind` field of the command's arguments table;
  - the floating or tiled plugin window, entering `root`, and wrapping by display width, again on resize;
  - `no errors`, focusing an open list, and closing with `q`.

  Register it in `crates/lua/src/bundled.rs`. Verify every error-list scenario in a new `crates/lua/tests/errors.rs`.
- [x] 5.3 Set up `gband.errors` and `gband.statusline` in `crates/lua/src/defaults.lua`, in the order the configuration delta gives. Verify "Defaults reproduce the built-in behaviour" and "Copied defaults load unchanged" in `crates/lua/tests/config.rs`.

## 6. Docs and examples

- [x] 6.1 Update `README.md` and `docs/plugins.md`:
  - side bars and `gband.bar`;
  - the `gband.statusline` setup options in place of the removed options, and vertical alignment with `{ lines = ... }` output;
  - `gband.errors()` and `gband.errors`, with a sample binding for `errors.open`;
  - the border options and the step options.

  Verify `rg 'statusline_(position|height|separator)|align = "(left|right)"' README.md docs` prints nothing.
- [x] 6.2 Move `examples/plugins/agent-status/client.lua` and `examples/plugins/window/lua/window/init.lua` to top or bottom alignment. Verify both load in `crates/lua/tests/plugins.rs` or with their existing example tests.
- [x] 6.3 Following `.claude/skills/gband-test/SKILL.md`, move the screen tests to the side-bar status line:
  - set up `gband.statusline` in the cases of `tests/lua/statusline_spec.lua` and `examples/plugins/window/tests/window_spec.lua` that draw segments;
  - read the waiting count from the bar in `examples/plugins/agent-status/tests/agent_status_spec.lua`;
  - update the default-configuration cases of `tests/lua/keylist_spec.lua` and `tests/lua/loop_bands_spec.lua` for the 60-column ribbon.

  Rewrite the references under `tests/lua/screenshots/statusline_spec/`, `tests/lua/screenshots/keylist_spec/`, `examples/plugins/agent-status/tests/screenshots/agent_status_spec/`, `examples/plugins/hello/tests/screenshots/hello_spec/` and `examples/plugins/window/tests/screenshots/window_spec/` with `--update`, and read each one. Verify `cargo test --test lua_specs` passes.

## 7. Gate

- [x] 7.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass.
