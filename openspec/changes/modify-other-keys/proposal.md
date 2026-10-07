## Why

Ctrl+Enter never reaches a program running in a gband window. Fish binds `ctrl-enter` to accept the autosuggestion and run it, and that works in Ghostty but does nothing under gband.

Fish asks for the kitty keyboard protocol with `\e[?u`. gband leaves that query unanswered, so fish falls back to xterm's modifyOtherKeys: it writes `\e[>4;1m` and expects Ctrl+Enter as `\e[27;5;13~`. gband ignores that request. Ghostty sends Ctrl+Enter as `\e[27;5;13~` by default, and gband's input decoder drops that sequence. If the decoder kept it, gband would still send the key on as plain `\r`, which fish reads as Enter.

## What Changes

- The decoder for the user's terminal reads xterm's modifyOtherKeys form `\x1b[27;m;code~` and the plain CSI u form `\x1b[code u` / `\x1b[code;m u` as keys with their modifiers. Codes 13, 9, 127 and 27 become Enter, Tab, Backspace and Escape. Any other code becomes that character.
- Every grid tracks its program's modifyOtherKeys level. `\x1b[>4;n m` sets it. `\x1b[>4m`, `\x1b[>4;0m` and `\x1b[>m` turn it off.
- While a window's modifyOtherKeys level is 1 or 2, gband encodes Enter, Tab and Backspace with Shift or Ctrl as `\x1b[27;m;code~`. Shift+Tab alone still encodes as `\x1b[Z`. With the level at 0, every key keeps today's bytes.
- The kitty keyboard protocol stays out of scope. The kitty query `\e[?u` stays unanswered.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `input-encoding`: the encoding depends on a third window input mode, modifyOtherKeys. Editing keys with Shift or Ctrl get the modifyOtherKeys encoding while that mode is on. The decoder for the user's terminal reads the `\x1b[27;m;code~` and CSI u forms.
- `terminal-emulator`: every grid reports its program's modifyOtherKeys level.

## Impact

- `crates/core/src/terminal_input.rs`: decodes the two new sequence forms.
- `crates/core/src/input.rs`: `Modes` gains the modifyOtherKeys level, and `encode_key` uses it for Enter, Tab and Backspace.
- `crates/emulator/src/callbacks.rs` and `crates/emulator/src/lib.rs`: track `\x1b[>4;n m` and report it in `modes()`.
- Other struct literals of `Modes`, in tests and in `crates/harness/src/case.rs`, spread `..Modes::DEFAULT` and need no change.
- Programs that never write `\x1b[>4;n m`, such as bash, see no change. No wire protocol change: the client already sends keys with their Shift, Alt and Ctrl modifiers, and the server encodes them.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- openspec/changes/modify-other-keys/
- crates/core/src/terminal_input.rs
- crates/core/src/input.rs
- crates/core/tests/terminal_input.rs
- crates/core/tests/input_encoding.rs
- crates/emulator/src/callbacks.rs
- crates/emulator/src/lib.rs
- tests/attach.rs
