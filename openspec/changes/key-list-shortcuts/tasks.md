## 0. Specs

- [ ] 0.1 Compare each MODIFIED block in `specs/key-list/spec.md` with the same requirement in `openspec/specs/key-list/spec.md`, as navigation-mode and every change archived before this one left it, and fold in every requirement text and scenario this change did not write. Verify that the only differences left are this change's own: the Down and Up keys in "Moving through the list" and "Run a focus action", and the `q` rule and its scenarios in "Closing the key list". Verify that `openspec validate key-list-shortcuts --strict` passes

## 1. Direct keys in the key list

- [ ] 1.1 In `crates/lua/src/runtime/gband/keylist.lua`, move the Enter handler's body into one local function that runs a list entry by its index, and have the `enter` entry call it with the cursor line. Verify the existing "run a focus action", "run close from the list" and "function binding does nothing" or "the key list's own line does nothing" cases in `tests/lua/keylist_spec.lua` still pass with `cargo test --test lua_specs bundled_key_list`
- [ ] 1.2 Build the `keys` table when the list opens: for each entry whose binding key is neither `prefix` nor one of `up`, `down`, `pageup`, `pagedown`, `home`, `end`, `enter` and `escape`, as `host.parse_key` names them, add an entry under the binding's key that calls `gband.win.set_cursor(win, index)` and then the run function from 1.1. Verify the list opens with the default configuration and with a configuration binding `prefix home`, with no error from `gband.win.open`

## 2. Tests

- [ ] 2.1 Change the "moving through the list" case in `tests/lua/keylist_spec.lua` to press Down, Down, then Up, and the "run a focus action" case to move with Down after Enter. Verify with `cargo test --test lua_specs bundled_key_list`
- [ ] 2.2 Add cases to `tests/lua/keylist_spec.lua` for every scenario of "Running a line by its key": a line's key runs it, `j` runs its binding, the key list's own line by its key, arrows move the cursor line, a line under an own key runs only through Enter, and an unbound `j` moves the cursor line. Verify each case fails with the 1.2 entries removed and passes with them
- [ ] 2.3 Add the "q with no q binding" case and make the "q closes the list" case use the default configuration and check that every window stays open. Verify with `cargo test --test lua_specs bundled_key_list`
- [ ] 2.4 Find every other test that moves the key list's cursor line with `j` or `k`, such as navigation-mode's end-to-end key list case in `tests/navigation.rs`, with `rg -n 'keylist|key list|ctrl\+space \?|\\x00\?' tests crates`, and change it to Down or Up. Verify `cargo test --workspace` passes

## 3. Documentation and final checks

- [ ] 3.1 Update the "Key list" section of `README.md`: pressing a line's key runs it, Up and Down move through the list, `j` and `k` run their lines, and Enter runs the selected line. Verify the section names no key that moves the list other than the arrows, PageUp, PageDown, Home and End
- [ ] 3.2 Update the key list section of `docs/plugins.md` with the direct keys, the own keys, the prefix key's line running only through Enter, and `q` running its line. Verify every rule of "Running a line by its key" appears there
- [ ] 3.3 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass
- [ ] 3.4 Manual check: start gband with the default configuration and two columns, press Ctrl+Space then `?`, then `l`, Down, `?` and `q`. Verify the second column takes focus with the list still open, Down moves the cursor line, `?` moves it to its own line and does nothing else, and `q` closes the list
