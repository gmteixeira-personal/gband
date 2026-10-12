## 1. Sidebar press on the error marker

- [x] 1.1 In `crates/lua/src/runtime/gband/sidebar.lua`, make the `MousePressed` handler accept the `left` and `right` buttons, and test the marker cell first: row `h − 1` while the marker is drawn, row 0 on a 1-row bar. A left press calls `gband.action["errors.open"]` and a right press calls `gband.action["errors.clear"]` when the entry exists. Every other right press returns before the label tests. Verify with `cargo test --test lua_specs bundled_sidebar`
- [x] 1.2 Add cases to `tests/lua/sidebar_spec.lua` for "Left click opens the error list", "Left click in navigation mode", "Right click clears the errors", "Right click with the error list open", "Error list plugin not set up", "One-row sidebar with the floating style" and "Click the blank error marker row". Write new screen references with `target/debug/gband test --update tests/lua/sidebar_spec.lua`, read each against its scenario, then run without `--update`. Verify with `cargo test --test lua_specs bundled_sidebar`

## 2. Key style presets

- [x] 2.1 In `crates/lua/src/runtime/gband/keystyle/modal.lua` and `direct.lua`, bind `prefix e` to a function described `list the errors` and `prefix E` to a function described `clear the errors`, right after `prefix m` and before `prefix D`. Each function looks up its action in `gband.action` when called and does nothing when the entry is nil. Verify with `cargo test -p gband-lua --test keystyle`
- [x] 2.2 Add a test to `crates/lua/tests/keystyle.rs` that checks, for both presets, that `e` and `E` sit between `m` and `D` with their descriptions. Confirm that `every_preset_binding_is_described` and the parity tests still pass. Verify with `cargo test -p gband-lua --test keystyle`
- [x] 2.3 Add cases to `tests/lua/keystyle_spec.lua` for "List the errors from navigation mode", "Clear the errors from navigation mode", "Error keys with the direct style", "Error keys without the error list plugin" and "Error list set up after the preset". Verify with `cargo test --test lua_specs key_styles`
- [x] 2.4 Rewrite the key list references under `tests/lua/screenshots/keylist_spec/` with `target/debug/gband test --update tests/lua/keylist_spec.lua`, read each against the new `e` and `E` lines, then run without `--update`. Verify with `cargo test --test lua_specs bundled_key_list`

## 3. Documentation

- [x] 3.1 Update `README.md`: add `e` and `E` rows to the modal and direct key table, add `e` to the keys that return to interactive mode, and rewrite the "Errors" section so it names the `!` left and right press and the `e` and `E` keys. Replace its `prefix E` tiled example with a key the presets leave free. Verify with `rg -n 'No key is bound|prefix", "E"' README.md` returning nothing
- [x] 3.2 Update `docs/plugins.md`: the sidebar section names the marker's left and right press, and the key styles section names `e` and `E` after `m` in both presets. Verify with `cargo test -p gband-lua --test api_docs`
- [x] 3.3 Update `docs/tutorial/00-setup.md` so it says the key style binds `e` and `E`, drops the `gband.keymap.set("prefix", "e", ...)` step and its quote of the full file, and mentions the click on `!`. Remove that line from each `examples/tutorial/*/user/init.lua` that holds it. Verify with `rg -n 'prefix", "e"' docs/tutorial examples/tutorial` returning nothing and `cargo test --test lua_specs tutorial`
- [x] 3.4 Update the quotes and prose of `docs/internals/08-key-styles.md` for `modal.lua` and `direct.lua`, and of `docs/internals/09-sidebar-and-errors.md` for `sidebar.lua`. Verify with `cargo test -p gband-lua --test tutorial` and `cargo test --test lua_specs bundled_copies`

## 4. Gates

- [x] 4.1 Search the main specs with `rg -n 'whose action no preset binds|on any other cell of the sidebar' openspec/specs` and confirm that each hit is in a requirement this change's deltas modify. Run `openspec validate error-list-shortcuts --strict`
- [x] 4.2 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify that all pass
- [x] 4.3 In a live server with the default configuration and a broken `user/init.lua`, confirm that a left click on `!` opens the list, a right click clears it, and Ctrl+Space then `e` and `E` do the same
