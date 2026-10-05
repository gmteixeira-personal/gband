## 1. Specs

- [ ] 1.1 For each MODIFIED requirement in this change's delta specs, compare the block with the current `openspec/specs/<capability>/spec.md` block of the same requirement, and fold in every difference this change did not make, such as center-column's `center_column` row and `c` binding. Verify that the only differences left are this change's own and that `openspec validate key-list --strict` passes

## 2. Descriptions

- [ ] 2.1 In `crates/lua/src/actions.rs`, change every built-in action's description to the actions spec's table, so each one says window, and in `crates/lua/src/options.rs` change the `default_column_width` description to say window. Update `crates/lua/src/defaults.lua`'s `desc` strings to match. Verify that `cargo test -p gband-lua --locked` passes after updating the tests that compare descriptions, and that every `desc` in the default configuration equals the description `gband.action.list()` gives its action

## 3. Closing the focused float

- [ ] 3.1 Add `q` to the default keys in `crates/lua/src/runtime/gband/win.lua`, closing a focused float with no `keys` entry for `q` as Escape does. Verify with tests for the plugin-windows scenarios "q closes a float", "A keys entry for q wins" and "q in a pane window"
- [ ] 3.2 Add a runtime entry point in `crates/lua/src/runtime.rs` that closes the focused float as `gband.win.close` does, runs its `on_close`, and reports whether it closed one. In `crates/client/src/lib.rs`, call it when close pane is resolved against the view, before the check for a focused pane, and send nothing when it closed a float. Verify with tests for the actions scenarios "Close the focused float", "Close the focused float on the empty band", "Close a focused floating pane" and "Target names a pane behind a float"

## 4. Key list plugin

- [ ] 4.1 Move `key_form` out of `crates/lua/src/runtime/gband/statusline/hints.lua` into a new bundled module `gband.keyform`, listed in `crates/lua/src/bundled.rs`, and require it from the hints segment. Verify that `cargo test -p gband-lua --locked key_hints` passes unchanged
- [ ] 4.2 Write `crates/lua/src/runtime/gband/keylist.lua` and list it in `crates/lua/src/bundled.rs`: `setup` with no options, the action `keylist.open`, a second open that focuses the open list, the lines in the key form padded to the widest key plus two, the width and height rules, the groups and the status line group defaults, and Enter with its muted lines. Verify with Lua tests for every scenario of the key-list spec, using `gband test`
- [ ] 4.3 In `crates/lua/src/defaults.lua`, set up `gband.keylist` before the key bindings and bind `prefix ?` to `keylist.open` with its description, after the floating-panes bindings and before `prefix D`. Update `default_keys_follow_the_spec` and `every_default_binding_names_an_action` in `crates/client/src/bindings.rs` to accept a binding to a registered action. Verify that `cargo test --workspace --locked` passes and that the configuration scenario "Key list set up by the defaults" holds

## 5. Documentation

- [ ] 5.1 Add `prefix ?` to the README's default keys and the key list to its configuration section, and say that a `user/init.lua` copied from an older `defaults/init.lua` shows its old descriptions in full in the hints until it is copied again or its descriptions are updated. Describe `gband.keylist` in `docs/plugins.md` beside the bundled segments. Verify by reading both sections

## 6. Check

- [ ] 6.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo test --workspace --locked`, and check that no comment breaks the rules in `CLAUDE.md`. Verify that all three pass
- [ ] 6.2 Attach to a fresh server on an 80×24 terminal, press Ctrl+Space then `?`, and check that no line is cut and that the prefix key shows as `C-space`. Press Enter on `h` and check that the list stays focused, Enter on `q` and check that the list closes and no pane does, and open it again and press Escape. Verify each step on screen
