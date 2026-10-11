## 1. Cursor model and drawing

- [ ] 1.1 In `crates/lua/src/runtime/gband/prompt.lua`, give each kind a `cursor` character count, set to `utf8.len(line)` in `open` and reset to 0 in `on_close`. Rewrite `fitted` to return the text before the cursor cell, the cursor cell and the text after it, as design.md's "Fitting keeps the cursor cell and cuts both ends" defines, and make `show` draw them as `PluginWindow`, `PromptCursor` and `PluginWindow` spans. Verify that every existing case in `crates/lua/tests/prompt.rs` still passes, and add cases there for "Cursor on a character", "Long line follows the cursor" and "Wide character under the cursor"
- [ ] 1.2 Make `on_input` insert at the cursor and advance it by the inserted length, and make Backspace remove the character before the cursor, close the prompt on an empty line, and do nothing at cursor 0 on a line with text. Make Ctrl+U set the cursor to 0. Verify with `crates/lua/tests/prompt.rs` cases for "Insert in the middle", "Paste in the middle", "Backspace in the middle", "Backspace at the start of a line with text" and the changed "Clear the line"
- [ ] 1.3 Add `left` and `right` entries to the prompt window's `keys`, each moving the cursor one character and stopping at the line's ends. Verify with `crates/lua/tests/prompt.rs` cases for "Left at the start", "Right at the end", "Right after Left" and "Run with the cursor in the middle"
- [ ] 1.4 Verify the rename prompt in `crates/lua/tests/prompt.rs`: a case for "Edit the middle of the manual name", and a case that focuses the open rename prompt again with `prompt.rename` after Left and checks that the next typed character lands at the kept cursor

## 2. Screen tests

- [ ] 2.1 Following `.claude/skills/gband-test/SKILL.md`, look at a prompt with the cursor in the middle with `gband test - --show`, then add cases to `tests/lua/prompt_spec.lua` for the cursor cell on a character, insertion in the middle, and a long line following the cursor on a 21×8 terminal. Write their references under `tests/lua/screenshots/prompt_spec/` with `--update`, read them, and confirm the existing `typed` and `long` references are unchanged; verify `cargo test --test lua_specs`
- [ ] 2.2 Add a case to `tests/lua/window_names_spec.lua` that opens the rename prompt on a window named `logs`, presses Left twice, types `-x`, takes a screenshot, presses Enter and waits for `╭lo-xgs`. Write its reference under `tests/lua/screenshots/window_names_spec/` with `--update` and read it; verify `cargo test --test lua_specs`

## 3. Documentation

- [ ] 3.1 Update the Lua prompt section of `README.md`: Left and Right move the cursor, typing and Backspace act at it, and a long line keeps the cursor shown. Verify by reading it against the lua-prompt delta's "Editing the line" table
- [ ] 3.2 Update `docs/plugins.md`: the sentence naming the keys the prompt takes through `keys`, the description of the cursor cell and the fitting rule, and the editing table with the Left and Right rows. Verify every row of the delta's "Editing the line" table appears there
- [ ] 3.3 Quote the changed parts of `prompt.lua` again in `docs/internals/10-keylist-and-prompt.md`, in blocks tagged `lua defaults/lua/gband/prompt.lua`, and describe the cursor count and the fitting rule in its prose. Verify `cargo test -p gband-lua --test tutorial`

## 4. Final checks

- [ ] 4.1 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass
