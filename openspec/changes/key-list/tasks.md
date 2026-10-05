## 1. Specs

- [ ] 1.1 For each MODIFIED requirement in this change's delta specs, compare the block with the current `openspec/specs/<capability>/spec.md` block of the same requirement, and fold in every difference this change did not make, such as center-column's `c` binding. Verify that the only differences left are this change's own and that `openspec validate key-list --strict` passes

## 2. Closing the focused floating plugin window

- [ ] 2.1 Add `q` to the default keys in `crates/lua/src/runtime/gband/win.lua`, closing a focused floating plugin window with no `keys` entry for `q` as Escape does. Verify with tests for the plugin-windows scenarios that cover `q`
- [ ] 2.2 Add a runtime entry point in `crates/lua/src/runtime.rs` that closes the focused floating plugin window as `gband.win.close` does, runs its `on_close`, and reports whether it closed one. In `crates/client/src/lib.rs`, call it when close window is resolved against the view, before the check for a focused window, and send nothing when it closed one. Verify with tests for the actions scenarios this change adds, including the one on the empty band

## 3. Key list plugin

- [ ] 3.1 Move `key_form` out of `crates/lua/src/runtime/gband/statusline/hints.lua` into a new bundled module `gband.keyform`, listed in `crates/lua/src/bundled.rs`, and require it from the hints segment. Verify that `cargo test -p gband-lua --locked key_hints` passes unchanged
- [ ] 3.2 Write `crates/lua/src/runtime/gband/keylist.lua` and list it in `crates/lua/src/bundled.rs`: `setup` with no options, the action `keylist.open`, a second open that focuses the open list, the lines in the key form padded to the widest key plus two, the width and height rules, the groups and the status line group defaults, and Enter with its muted lines. Verify with Lua tests for every scenario of the key-list spec, using `gband test`
- [ ] 3.3 In `crates/lua/src/defaults.lua`, set up `gband.keylist` before the key bindings and bind `prefix ?` to `keylist.open` with its description, after the floating window bindings and before `prefix D`. Update `default_keys_follow_the_spec` and `every_default_binding_names_an_action` in `crates/client/src/bindings.rs` to accept a binding to a registered action. Verify that `cargo test --workspace --locked` passes and that the configuration scenario "Key list set up by the defaults" holds

## 4. Documentation

- [ ] 4.1 Add `prefix ?` to the README's default keys and the key list to its configuration section, and describe `gband.keylist` in `docs/plugins.md` beside the bundled segments. Verify by reading both sections

## 5. Check

- [ ] 5.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo test --workspace --locked`, and check that no comment breaks the rules in `CLAUDE.md`. Verify that all three pass
- [ ] 5.2 Attach to a fresh server on an 80×24 terminal, press Ctrl+Space then `?`, and check that no line is cut and that the prefix key shows as `C-space`. Press Enter on `h` and check that the list stays focused, Enter on `q` and check that the list closes and no window does, and open it again and press Escape. Verify each step on screen
