## 1. Reconcile the specs

- [ ] 1.1 Compare each MODIFIED requirement in this change's delta specs with the same requirement in `openspec/specs/` on `<base>`, carry over any wording another change landed since this proposal, and verify with `openspec validate direct-navigation-mode --strict`

## 2. Shared navigation bindings

- [ ] 2.1 Give `gband.jump.setup` an optional `table` argument, defaulting to `prefix`, in `crates/lua/src/runtime/gband/jump.lua`, its literate copy in `docs/internals/08-key-styles.md`, and its type in `crates/lua/src/runtime/meta/gband.lua`. Verify with a test in `crates/lua/tests/keystyle.rs` that `setup("nav")` binds `a` and `b` in `nav`, and that two calls leave one `agents` and one `bands` mode
- [ ] 2.2 Create `crates/lua/src/runtime/gband/navigation.lua` with `bind(table)` holding modal.lua's `prefix` bindings and its `interactive_on_new` read, register it in `crates/lua/src/bundled.rs`, load it from `crates/lua/src/runtime/gband/prelude.lua`, and type it in `meta/gband.lua`. Verify with tests in `crates/lua/tests/keystyle.rs` for "Bind into a named table" and "Root refused"
- [ ] 2.3 Reduce `crates/lua/src/runtime/gband/keystyle/modal.lua` to the plugins, the `prefix` mode, `gband.navigation.bind("prefix")` and the `root` mouse names. Verify `gband.keymap.list("prefix")` under modal is unchanged against a list captured before the edit, and that `cargo test -p gband-lua` passes
- [ ] 2.4 Add the `navigation` mode, `prefix space` after `D`, and `gband.navigation.bind("navigation")` to `crates/lua/src/runtime/gband/keystyle/direct.lua`. Verify with tests in `crates/lua/tests/keystyle.rs` for "Direct navigation equals modal navigation" and the amended "Same names in both presets"
- [ ] 2.5 Update the modal and direct preset copies and the `gband.navigation` section in `docs/internals/08-key-styles.md`, and verify the bundled-copies test in `tests/lua/bundled_copies_spec.lua` passes

## 3. Key list

- [ ] 3.1 Make `crates/lua/src/runtime/gband/keylist.lua` read `gband.keymap.current_table()` before entering `root`, falling back to `prefix` for `root`, and use that table for the entries, the title and `gband.keymap.run`. Update its literate copy in the docs. Verify with `tests/lua/keylist_spec.lua` cases for "Listed table from direct navigation mode" and "Listed table after the direct prefix key"
- [ ] 3.2 Confirm in `crates/client/src/lib.rs` what `current_table()` reports during a one-shot `prefix` callback, and verify the direct `prefix ?` case lists `prefix` whichever table it reports

## 4. Settings

- [ ] 4.1 Show the `I on new` line for the `direct` style in `crates/lua/src/runtime/gband/settings.lua` and its copy in `docs/internals/07-settings.md`. Verify with `window_with_the_direct_style` in `crates/lua/tests/settings.rs` updated to six lines and 8 rows, and the `the-settings-window-with-the-direct-key-style--direct.txt` screenshot regenerated through `tests/lua/settings_spec.lua`
- [ ] 4.2 Add tests in `crates/lua/tests/settings.rs` or `tests/settings.rs` for "Turn I on new on with the direct style" and the direct case of "Nothing saved", and update the style-switch scenario that expects five lines after modal to direct. Verify they pass

## 5. End-to-end behaviour

- [ ] 5.1 Add cases in `tests/keystyle.rs` or `tests/lua/keystyle_spec.lua` for the key-style scenarios: "Navigation mode with the direct style", "Escape leaves direct navigation mode", "Prefix key leaves direct navigation mode", "Agents mode from direct navigation mode", "I on new in direct navigation mode", and the sidebar `N` letter. Verify they pass
- [ ] 5.2 Add cases for the client-attach scenarios "Navigation mode with the direct key style", "Open a window from direct navigation mode" and "Multi mode from direct navigation mode", and the jump-modes scenario "Bands mode from direct navigation mode". Verify they pass
- [ ] 5.3 Update the client binding tests in `crates/client/src/bindings.rs` that assert the direct style has no mode, so they expect the `navigation` mode. Verify `cargo test -p gband-client` passes

## 6. User documentation

- [ ] 6.1 Describe Ctrl+Space then Space in `README.md` (the direct style entry and the key table), `docs/plugins.md` (the modes paragraph, the key list title and `gband.jump.setup(table)`) and `docs/tutorial/00-setup.md`. Verify `rg -n "declare no mode|direct style.*no mode" README.md docs` finds no stale claim

## 7. Gate

- [ ] 7.1 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`, and verify both pass
- [ ] 7.2 Attach a client with the direct style saved, press Ctrl+Space, Space, then `l`, `l`, `?` and Escape, and verify the sidebar shows `N`, focus moves two columns, and the key list is titled `navigation keys`
