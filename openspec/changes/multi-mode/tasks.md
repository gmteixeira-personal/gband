## 1. Client state and fan-out

- [x] 1.1 Add `ClientAction::ToggleMulti` to `crates/core/src/action.rs` and the built-in `toggle_multi`, `toggle multi mode`, to `crates/lua/src/actions.rs`. Update the action-list tests in `crates/lua/tests/config.rs` and `crates/lua/tests/control.rs` for "Every action is named" and "Listed with its description". Verify with `cargo test -p gband-lua --test config --test control`
- [x] 1.2 In `crates/client/src/lib.rs`, add `multi: bool` to `Display`, off at attach, flipped by `ToggleMulti` in `run_action` with no step, and cleared in the successful branch of `Controls::reload`. Replace `key_to_focused` and the paste path with a target list: the focused window with multi mode off, every window of the viewed band in layout order with it on, nothing for an empty band. Route unbound keys, pastes, `SendPrefix` and `SendKey` through it, leaving plugin windows, mouse input and `gband.window.send_keys`/`send_text` unchanged. Add tests to `crates/client/tests/actions.rs` for "Toggle twice", "Reload turns it off", the layout order of the sent keys, a paste, the prefix key, an empty band, a focused plugin window and a mouse press. Verify with `cargo test -p gband-client`
- [x] 1.3 Add `multi` to `ViewState` in `crates/lua/src/ui.rs`, set by `Display::view_state`, compared in `drawn_differs` and exposed by `core.state()`. Set `multi = true` in `gband.view()` in `crates/lua/src/control.rs` only while it is on. Add tests to `crates/lua/tests/control.rs` for "Off at attach" and "Multi mode on", and confirm "Focused window" still holds the exact table. Verify with `cargo test -p gband-lua --test control`

## 2. Key styles

- [x] 2.1 In `crates/lua/src/runtime/gband/keystyle/modal.lua`, bind `prefix m` after `!` to a function that dispatches `action.toggle_multi` and then enters `root`, with the description `multi mode`. In `direct.lua`, bind `prefix m` to `action.toggle_multi` at the same position. Update the binding lists and "Same names in both presets" in `crates/lua/tests/keystyle.rs` and `tests/keystyle.rs`, and add "Multi mode from navigation mode", "Multi mode with the direct key style" and "Navigation mode keeps multi mode" to `tests/keystyle.rs`. Verify with `cargo test --test keystyle` and `cargo test -p gband-lua --test keystyle`
- [x] 2.2 Rewrite the key list references under `tests/lua/screenshots/keylist_spec/` that list the modal `prefix` bindings with `target/debug/gband test --update tests/lua/keylist_spec.lua`, read each against the new `m` line, then run without `--update`. Verify with `cargo test --test lua_specs keylist`

## 3. Sidebar

- [x] 3.1 In `crates/lua/src/runtime/gband/sidebar.lua`, make `mode_letter` return `M` for `root` while `state.multi` is set, after the floating style's return. Add "Multi mode" and "Multi mode with the floating style" to `tests/lua/sidebar_spec.lua`, update "Open the window list from the sidebar" for the `m Multi mode` row, with a screenshot reference written by `target/debug/gband test --update tests/lua/sidebar_spec.lua` and read against the scenario, and a unit case to `crates/lua/tests/sidebar.rs`. Verify with `cargo test -p gband-lua --test sidebar` and `cargo test --test lua_specs sidebar`

## 4. Window list

- [x] 4.1 In `crates/lua/src/runtime/gband/desktop.lua`, add the entry `Multi mode`, shortcut `m`, after `Settings`, labelled `Multi mode (on)` while `gband.view().multi` is set. Picking it cancels the list, then dispatches `gband.action.toggle_multi()`. Its preview returns to the band at opening. Add `m` to `LIST_KEYS`. Verify with `cargo test -p gband-lua --test desktop`
- [x] 4.2 Update `crates/lua/tests/desktop.rs` and `tests/lua/floating_spec.lua` for every changed scenario of the floating-key-style delta: the `j` counts, the list sizes and rows 8 to 15 and 7 to 15, the `m Multi mode` rows, and add "Multi mode from the list", "Multi mode off from the list" and "Multi mode from a preview". Rewrite the references under `tests/lua/screenshots/floating_spec/` that show the window list with `target/debug/gband test --update tests/lua/floating_spec.lua`, read each against its scenario, then run without `--update`. Verify with `cargo test -p gband-lua --test desktop` and `cargo test --test lua_specs floating`

## 5. End-to-end

- [x] 5.1 Add `tests/multi_mode.rs` with real servers and clients for "Typing reaches every window", "Other bands receive nothing", "Paste reaches every window", "Minimized window receives the input", "Band change moves the input", "Plugin window takes the keys", "Prefix key to every window", "Mouse reaches one window", "Another client is unaffected", "Bound by a user" and "Multi mode by name", waiting for each window's shell prompt before typing. Verify with `cargo test --test multi_mode`

## 6. Documentation

- [x] 6.1 Update `README.md`: the modal and direct key tables gain `m`, multi mode gets a short section on what it sends and where, and the floating style's window list names `Multi mode` and `m`. Verify with `rg -n "Multi mode|multi mode" README.md`
- [x] 6.2 Update `docs/plugins.md` (the `toggle_multi` action in the action list, `prefix m` in the key tables, `multi` in `gband.view()`) and `docs/tutorial/01-keys.md` where it names the window list's own keys `n`, `s` and the digits. Verify with `cargo test -p gband-lua --test api_docs` and `cargo test --test lua_specs tutorial`
- [x] 6.3 Update the quotes and prose of `docs/internals/08-key-styles.md` for `modal.lua`, `direct.lua` and `desktop.lua`, and of `docs/internals/09-sidebar-and-errors.md` for `sidebar.lua`. Verify with `cargo test -p gband-lua --test tutorial`

## 7. Gates

- [x] 7.1 Search the main specs with ``rg -n 's Settings`, a separator|24 columns wide and 7 rows high|gives it back rows 8 to 14|`n`, `s` and the' openspec/specs`` and confirm that each hit is in a requirement this change's deltas modify. Run `openspec validate multi-mode --strict`
- [x] 7.2 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify that all pass
