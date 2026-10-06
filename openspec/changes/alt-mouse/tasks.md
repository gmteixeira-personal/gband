## 1. Wheel names and `mod`

- [ ] 1.1 In `crates/core/src/input.rs`, make `MouseKey` hold `MouseInput::Button(MouseButton)` or `MouseInput::Wheel(WheelDirection)` beside its modifiers, and update every construction site; verify with `cargo build --workspace`
- [ ] 1.2 In `crates/lua/src/keys.rs`, parse `wheelup`, `wheeldown`, `wheelleft` and `wheelright` in any case, write them back in `mouse_name`, and accept `mod` only before a mouse name; replace `mouse_names_are_not_keys`'s wheel rejections with round-trip tests and add rejections of `mod+h` and `scrollup`; verify with `cargo test -p gband-lua keys`
- [ ] 1.3 Give `Chord::Mouse` a `uses_mod` flag in `crates/lua/src/api.rs` and `crates/lua/src/keymap.rs`, keeping the key as written for `gband.keymap.list`; verify the "Listed as written" and "Mod with a key" scenarios with tests in `crates/lua/tests/keymap.rs`, and update `crates/lua/tests/config.rs` for the new `MouseKey`
- [ ] 1.4 Add the `mouse_mod` client option in `crates/lua/src/options.rs`, with its description, validation, lowercase read-back in `ctrl`, `alt`, `shift` order and the `"alt"` default; verify the "Read back" and "Invalid mouse modifier" scenarios with tests in `crates/lua/tests/options.rs`

## 2. Client matching

- [ ] 2.1 In `crates/client/src/bindings.rs`, give `Keymap` the resolved `mouse_mod` and OR it into a `uses_mod` chord's modifiers when matching, first binding made winning; verify with unit tests for "Default mouse modifier" and "Changed mouse modifier" (set after `gband.keystyle.use()`)
- [ ] 2.2 Add `Leader::handle_wheel`: run a bound step in a mode or `root` without changing the table, run a bound step in any other table and return to `root`, and pass an unbound step with the table unchanged; verify with unit tests beside `navigation_mode_repeats_keys` and `prefix_table_that_is_not_a_mode`
- [ ] 2.3 In `crates/client/src/mouse.rs`, make `Controls::wheel` ask `handle_wheel` unless a gesture is held, run an action, or a function through a `call_scrolled` beside `call_pressed` in `crates/lua/src/runtime.rs` with the `MouseScrolled` payload, drop a step matching the last wheel binding's `MouseKey` within 150 ms of its last run, and send the rest to their target as today; verify with tests in `crates/client/tests/actions.rs` for "Bound wheel step in root", "Cooldown after a wheel binding", "Binding runs again after the cooldown", "Unbound wheel step keeps the sequence", "Wheel during a gesture" and "Wheel binding function"
- [ ] 2.4 In `press_default`, forward unless Ctrl and Alt are both held; verify "Ctrl+Alt selects in a mouse program" and "Unbound Alt click is forwarded" with tests in `crates/client/tests/actions.rs`, and update the existing Alt selection test
- [ ] 2.5 Verify with a test in `crates/client/tests/actions.rs` that a wheel binding over a plugin window with `on_mouse` runs the binding and not `on_mouse`, and that an unbound wheel step still scrolls a plugin window without `on_mouse`

## 3. Presets and parity

- [ ] 3.1 In `crates/lua/src/runtime/gband/keystyle/modal.lua` and `direct.lua`, bind the five `mod` rows in `root`, then the three plain mouse rows and the five `mod` rows in `prefix`, in the client-attach order, with the actions' descriptions; verify with `cargo test -p gband-lua --test keystyle` and the "Only mouse names in root" scenario
- [ ] 3.2 Update `default_keys_follow_the_spec`, `direct_keys_follow_the_spec` and `every_default_binding_names_an_action` in `crates/client/src/bindings.rs` to the new rows; verify with `cargo test -p gband-client bindings`
- [ ] 3.3 Add `presets_keep_parity` to `crates/client/src/bindings.rs`, comparing `root` and `prefix` under both saved styles with `escape` and `enter` removed from the modal `prefix`, and actions compared wherever both entries give one; verify it passes, and that it fails when a row is removed from `direct.lua` alone
- [ ] 3.4 Add end-to-end tests to `tests/mouse.rs` and `tests/keystyle.rs` for "Alt drag moves a window in interactive mode" under both styles, "Alt drag in navigation mode", "Alt wheel switches bands", "Drag after the prefix with the direct key style", "Other press after the prefix with the direct key style" and "Remove a preset mouse binding"; verify with `cargo test --test mouse --test keystyle`

## 4. Docs and checks

- [ ] 4.1 Update `README.md`: the Alt-selects line becomes Ctrl+Alt, the wheel paragraph names Alt+wheel band switching, the mouse table gains the Alt rows for both styles, and a note shows `gband.opt.mouse_mod = "ctrl+alt"` for desktops that take Alt+drag; verify by reading it against the client-attach and configuration deltas
- [ ] 4.2 Update `docs/plugins.md`: mouse names include the wheel names and `mod`, the `mouse_mod` option, wheel binding functions get the `MouseScrolled` payload, and bound wheel steps skip `on_mouse`; verify by reading it against the mouse and plugin-windows deltas
- [ ] 4.3 Change the Purpose of `openspec/specs/mouse/spec.md` from "how every wheel step reaches the program under the pointer" to say that unbound wheel steps reach it and bound ones run their binding; verify `openspec validate --specs --strict` passes
- [ ] 4.4 Run `cargo clippy --workspace --all-targets` and `cargo test --workspace`, and check that every scenario of the change's delta specs maps to a passing test
