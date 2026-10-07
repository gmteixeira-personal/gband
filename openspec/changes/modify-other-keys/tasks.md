## 1. Decode the user's terminal

- [ ] 1.1 In `crates/core/src/terminal_input.rs`, read `\x1b[27;m;c~`, `\x1b[c;mu` and `\x1b[cu` as the key with code `c` (13 Enter, 9 Tab, 127 Backspace, 27 Escape, otherwise a printable character) and modifiers from `m`. Read a Shift-only Tab as `KeyCode::BackTab` with no modifiers. Any other code, and colon sub-parameters, give no key. Verify with new cases in `crates/core/tests/terminal_input.rs` for every "Keys from the terminal's bytes" scenario this change adds, plus a split-read case of `\x1b[27;5;13~` across two `push` calls.

## 2. Track modifyOtherKeys in the emulator

- [ ] 2.1 Add a `modify_other_keys: u8` field to `Modes` in `crates/core/src/input.rs`, set to 0 in `Modes::DEFAULT`. The only struct literal listing every field is `Vt100::modes()` in `crates/emulator/src/lib.rs`; the others spread `..Modes::DEFAULT`. Verify with `cargo build --workspace --all-targets`.
- [ ] 2.2 In `crates/emulator/src/callbacks.rs`, set the level from `\e[>4;n m` for `n` 0, 1 or 2, clear it on `\e[>4m` and `\e[>m`, and ignore any other `n` and other `\e[>…m`, all without logging them as unhandled. Report the level from `Vt100::modes()` in `crates/emulator/src/lib.rs`. Verify with unit tests in `callbacks.rs` for each terminal-emulator scenario except the full reset.
- [ ] 2.3 In `Vt100::process`, clear the level when ESC is directly followed by `c`, carrying a chunk-final ESC into the next call. Verify with tests: `\ec` in one chunk, `\e` and `c` split across two `process` calls, and `\ec` right after an unterminated OSC, each clearing level 1. Add a test that `\e[>4;1m` leaves a grid's cells, attributes and write-back unchanged.

## 3. Encode for the window

- [ ] 3.1 In `encode_key`, while `modes.modify_other_keys` is 1 or 2, encode Enter or Backspace with Shift or Ctrl, and Tab with Ctrl, as `\x1b[27;m;c~` with Alt in `m`. Every other key keeps its current bytes. Verify with cases in `crates/core/tests/input_encoding.rs` for every "Editing keys" and "Alt as an ESC prefix" scenario in the delta spec, at levels 0, 1 and 2.

## 4. End to end

- [ ] 4.1 Add a test in `tests/attach.rs`: the window runs `printf '\033[>4;1m'; cat -v`, the client's terminal sends `\x1b[27;5;13~`, and the window shows `^[[27;5;13~`. Add a second test where the window runs `cat -v` without the `printf`: the client sends `\x1b[27;5;13~` then `x`, and the window shows `x` on a new line. Verify with `cargo test --test attach`.
- [ ] 4.2 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets` and confirm both pass.
- [ ] 4.3 Manual check in Ghostty with fish: attach to gband, type the start of a command fish suggests, press Ctrl+Enter, and confirm the suggestion is accepted and run, as it is in Ghostty without gband.
