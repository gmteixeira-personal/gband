## 1. Record the layout entering floating

- [x] 1.1 In `crates/lua/src/runtime/gband/settings.lua`, add a serializer that turns a `gband.layout()` table into a `return <table>` string holding only each band's tiled columns (`width`, `full_width`, `tabbed`, and windows with `id` and `rows` or `weight`) and floating windows (`id`, `col`, `row`, `width`, `full_width`, `rows`); verify with a `cargo test --workspace --locked` unit or lua-spec round trip that serializing then reading a sample layout returns an equal table.
- [x] 1.2 Add `RESTORE_FILE = "user/floating_restore.lua"` and a `read` validator for the serialized shape, so a missing or malformed file reads as no record; verify the lua spec reads nil from an absent file.
- [x] 1.3 In `step_keys`, when the current style is not `floating` and the next style is `floating`, write the record from `gband.layout()` with `write` before `save`; verify by the "A record is kept entering floating" scenario that `user/floating_restore.lua` exists after the switch.

## 2. Restore the layout leaving floating

- [x] 2.1 Add a restore routine that, for each band in the record, tiles every recorded window from the last column's bottom window to the first column's top window with `gband.action.toggle_window_floating({ window = id, floating = false })` and no `after`, so each lands as the band's first column, then stacks each column's further windows with `gband.action.consume_or_expel_left`, then sets each column's `width` with `gband.window.set_width`, `full_width` with `gband.action.toggle_full_width` when on, each window's height with `gband.window.set_height`, and `tabbed` with `gband.action.toggle_column_tabbed` when on; verify the "Columns restored across a round trip" and "A stacked column restored" scenarios pass.
- [x] 2.2 Restore each recorded floating window to its box with `set_width`, `toggle_full_width` when on, `set_height({ rows = r })` then `set_position({ col, row })`, and skip a recorded window no longer in the layout; verify the "A previously floating window keeps its box" scenario passes.
- [x] 2.3 Tile each floating window not in the record before any recorded window, with no `after`, so the recorded windows tiled after it land to its left and it follows the restored tiled windows of its band; verify the "A window opened while floating tiles after the restored windows" scenario passes.
- [x] 2.4 In `step_keys`, when the current style is `floating` and the next style is not, read the record, restore it when present, otherwise tile every floating window with `toggle_window_floating({ window = id, floating = false })`, then delete the record file, then `save`; verify the "No record tiles every floating window" scenario passes.

## 3. Tests

- [x] 3.1 Add the six scenarios of `openspec/changes/leave-floating-restores-layout/specs/settings/spec.md` as cases in `tests/lua/settings_spec.lua`, driving the key presses and the reload wait as the existing settings cases do; verify `cargo test --workspace --locked` runs them green.

## 4. Documentation

- [x] 4.1 Update `docs/internals/07-settings.md` so the chapter still reads the module in full: add the serializer, the record file and validator, and the record and restore steps of `step_keys`, keeping the code blocks faithful to `crates/lua/src/runtime/gband/settings.lua`; verify `cargo test --workspace --locked` passes the `tutorial` internals tests.

## 5. Gate

- [x] 5.1 Run `./.claude/flow-gate` and verify `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo test --workspace --locked` all pass.
