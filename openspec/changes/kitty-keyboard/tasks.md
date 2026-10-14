## 1. Emulator: kitty keyboard flags

- [ ] 1.1 Add `keyboard_flags: u8` to `Modes` in `crates/core/src/input.rs` and `Modes::DEFAULT`, and report it from `Grid::modes` in `crates/emulator/src/lib.rs`. Verify `cargo build --workspace`
- [ ] 1.2 Add a per-screen base value and 16-entry stack to `Callbacks` in `crates/emulator/src/callbacks.rs`, and handle `\e[>f u`, `\e[<n u`, `\e[=f;m u` and `\e[?u` in `unhandled_csi`, clearing every bit but 1 and leaving `\e[u` without a marker alone. Verify with unit tests for the push, pop, set, query, overflow and restore-cursor scenarios of the terminal-emulator delta
- [ ] 1.3 Clear both screens' flags on `\ec` next to the existing modifyOtherKeys reset, and split `Grid::process` chunks after `\e[?47h`, `\e[?1047h` and `\e[?1049h`, clearing the alternate screen's flags on every change from main to alternate. Verify with tests for "Screens keep their own flags", "A new alternate screen starts clear" and "Full reset clears the flags", each written as one chunk and as separate chunks
- [ ] 1.4 Add "Flags left out of a snapshot" and the write-back scenarios to `crates/emulator/tests/contract.rs`. Verify `cargo test -p gband-emulator`

## 2. Encoding keys for windows

- [ ] 2.1 Add the kitty path to `encode_key` in `crates/core/src/input.rs`, checked before `modified_other_key`, per "Kitty keyboard encoding". Verify with a test in `crates/core/tests/input_encoding.rs` for every scenario of that requirement and of the modified "Alt as an ESC prefix", "Modifier parameter" and "Tilde and function keys"
- [ ] 2.2 Verify the existing input-encoding tests still pass unchanged with `keyboard_flags` 0: `cargo test -p gband-core`

## 3. Reading keys from the user's terminal

- [ ] 3.1 In `crates/core/src/terminal_input.rs`, parse alternate keys in the first field of the `u` and `27;m;c~` forms, keep refusing `:` elsewhere, and apply the shifted-key and lowercase-letter-with-Shift rules. Verify with tests for "Kitty sub-parameters are no key", "Shifted key names the character", "Shifted letter keeps Shift", "Shifted letter without an alternate key" and "Base-layout key ignored" in `crates/core/tests/terminal_input.rs`
- [ ] 3.2 Map the private-use codes in `coded_key` per the code table, refusing the rest of 57344 to 63743. Verify with tests for "Keypad Enter", "Kitty F13", "Modifier key alone is no key" and "Caps Lock dropped"
- [ ] 3.3 Add `E`, `P`, `Q` and `S` to `starts_sequence` and read bare `\e[P`, `\e[Q`, `\e[S` as F1, F2, F4. Verify with "Kitty F1", "Kitty Escape", "Kitty Ctrl+I stays apart from Tab", and a test that `\e[E` then `a` reads `a` only

## 4. Client requests to the user's terminal

- [ ] 4.1 Add public functions in `crates/client/src/lib.rs` that write the attach requests (`\e[>4;1m`, then `PushKeyboardEnhancementFlags` with disambiguate and alternate keys) and the detach requests (`PopKeyboardEnhancementFlags`, then `\e[>4m`) to any `Write`. Call the first right after `ratatui::init()` and the second in `TerminalGuard::drop` before `ratatui::restore()`. Verify with `crates/client/tests/keyboard.rs` asserting the exact bytes `\x1b[>4;1m\x1b[>5u` and `\x1b[<1u\x1b[>4m`

## 5. End to end

- [ ] 5.1 Add tests to `tests/attach.rs`: "Shift+Enter reaches a program that pushed kitty flags" (`printf '\033[>1u'; cat -v`, send `\x1b[13;2u`, expect `^[[13;2u`), the same after a reattach, and "Requests on attach" (the `Attached` grid reports kitty flags 1 and modifyOtherKeys level 1 after attach). Verify `cargo test --test attach`

## 6. Manual check

- [ ] 6.1 Build with `cargo build --release`, attach in Ghostty or kitty, run Claude Code in a window, and press Shift+Enter and Ctrl+Enter in its prompt. Verify each inserts a new line without submitting, and that Alt+`!`, Ctrl+C and Escape behave as before in a shell window

## 7. Final checks

- [ ] 7.1 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass
