## 1. Saved setting

- [x] 1.1 Add `interactive_on_new()` to `crates/lua/src/runtime/gband/settings.lua`, reading `user/interactive_on_new.lua` through the shared reader with a boolean check. Verify every "Saved I on new" scenario that reads the file ("Nothing saved", "Unknown saved value", "Broken file is not a configuration error") in `crates/lua/tests/settings.rs`

## 2. Modal `n`

- [x] 2.1 In `crates/lua/src/runtime/gband/keystyle/modal.lua`, read `gband.settings.interactive_on_new()` once while the preset evaluates, and call `interactive()` after `open_window()` in the `n` function unless it returned `false`. Leave `direct.lua` unchanged. Verify `default_keys_follow_the_spec` and `direct_keys_follow_the_spec` in `crates/client/src/bindings.rs` against the updated client-attach table, and the "Open a window and stay in navigation mode", "Direct key style ignores the setting" and "Own configuration with the modal preset" scenarios in `tests/settings.rs`

## 3. Settings window line

- [x] 3.1 In `open()`, add the fourth line `I on new` with `on` or `off` when the `keys` value is `modal`, and set the height from the line count. Verify "Window opens", "Window beside the default sidebar", "Window with the direct style" and "Saved values shown" in `crates/lua/tests/settings.rs`, the 80×24 sizes and positions included
- [x] 3.2 Dispatch Enter, `h`, `l`, Left and Right on the `I on new` line to save the other value through the shared writer, with the reopen line set to 4. Verify "Saved file", "Turn I on new off" and "Turn I on new back on" in `crates/lua/tests/settings.rs` and `tests/settings.rs`, and "Switch the key style" reopening with three lines
- [x] 3.3 Regenerate the settings window references under `tests/lua/screenshots/settings_spec/` with `GBAND_UPDATE_SCREENSHOTS`, and add one for the window under the direct style. Check that each diff holds only the new line and the moved border. Verify with `cargo test --test lua_specs`

## 4. Documentation and gates

- [x] 4.1 Update `README.md` (the `I on new` row, its file, that it applies to the modal style only, and that a user's own `n` binding can read `gband.settings.interactive_on_new()`) and `docs/plugins.md` (`gband.settings.interactive_on_new`). Verify with `rg 'interactive_on_new'` that every mention names the function or the file correctly
- [x] 4.2 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `openspec validate settings-interactive-on-new --strict`, and verify all pass
