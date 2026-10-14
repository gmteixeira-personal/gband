## Why

Shift+Enter and Ctrl+Enter reach Claude Code as plain Enter when it runs in a gband window. Claude Code detects modified Enter through the kitty keyboard protocol: it asks with `\e[?u`, and gband never answers, so Claude Code stays in legacy mode, where Enter, Shift+Enter and Ctrl+Enter are all `\r`. The client also never asks the user's terminal for a keyboard protocol, so most terminals send `\r` for all three before gband reads them. gband's existing modifyOtherKeys support for windows therefore only works in terminals that send `\e[27;m;13~` unasked, as Ghostty does for Ctrl+Enter.

## What Changes

- The client asks the user's terminal for both keyboard protocols on attach: kitty's disambiguate and alternate-key flags with `\e[>5u`, and modifyOtherKeys level 1 with `\e[>4;1m`. A terminal with kitty support uses kitty, one with only modifyOtherKeys uses that, and one with neither ignores both. Detach pops the kitty flags and resets modifyOtherKeys.
- gband reads the kitty forms the terminal then sends: alternate-key sub-fields, keypad keys, F13 to F35, and `\e[P`, `\e[Q`, `\e[S` for F1, F2 and F4. A shifted key the terminal names is read as that character, so Alt+`!` stays Alt+`!`.
- Each window's emulator implements the kitty keyboard protocol's flags: push, pop, set and the `\e[?u` query, with separate stacks for the main and alternate screens. Only the disambiguate flag is supported; the query reports it alone.
- A window whose program set the disambiguate flag receives kitty encodings: Shift+Enter as `\e[13;2u`, Ctrl+Enter as `\e[13;5u`, Escape as `\e[27u`, and Ctrl or Alt with a character as `\e[c;mu`. modifyOtherKeys keeps working for windows that set it, and kitty takes precedence when a window set both.
- **BREAKING** for key bindings in terminals with kitty support: Ctrl+`i`, Ctrl+`m` and Ctrl+`[` arrive as themselves, so they no longer match bindings on Tab, Enter and Escape. Ctrl+Shift with a letter arrives with its Shift.

## Capabilities

### New Capabilities

### Modified Capabilities
- `input-encoding`: a new requirement "Keyboard protocols of the user's terminal" covers the attach and detach sequences, a new requirement "Kitty keyboard encoding" covers keys for windows with the disambiguate flag, "Keys from the terminal's bytes" reads the kitty forms, and "Encoding inputs" adds the window's kitty flags to the modes an encoding depends on.
- `terminal-emulator`: a new requirement "Kitty keyboard flags" covers push, pop, set, query, the per-screen stacks, full reset and snapshots.
- `session-server`: "Terminal query replies" answers the kitty keyboard query `\e[?u` with the flags of the screen in use, and its unanswered example becomes the background color query.

## Impact

- `crates/core/src/input.rs`: `Modes` gains `keyboard_flags`; `encode_key` gains the kitty form, checked before modifyOtherKeys.
- `crates/core/src/terminal_input.rs`: the `u` form accepts alternate-key sub-fields, maps keypad and F13 to F35 codes, and `\e[P`, `\e[Q`, `\e[S` start sequences.
- `crates/emulator/src/callbacks.rs` and `crates/emulator/src/lib.rs`: per-screen flag stacks, the `\e[?u` reply, clearing on full reset and on entering the alternate screen, and `modes()` reporting the flags.
- `crates/client/src/lib.rs`: `TerminalGuard` and the alternate-screen setup push and pop the terminal's keyboard modes.
- Tests in `crates/core/tests/`, `crates/emulator/tests/contract.rs`, `crates/client/tests/keyboard.rs` and `tests/attach.rs`.
- No protocol, Lua or configuration change. No new dependency: crossterm 0.29 provides `PushKeyboardEnhancementFlags` and `PopKeyboardEnhancementFlags`.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/core/src/input.rs
- crates/core/src/terminal_input.rs
- crates/core/tests/input_encoding.rs
- crates/core/tests/terminal_input.rs
- crates/emulator/src/callbacks.rs
- crates/emulator/src/lib.rs
- crates/emulator/tests/contract.rs
- crates/client/src/lib.rs
- crates/client/tests/keyboard.rs
- crates/server/tests/queries.rs
- tests/attach.rs
- openspec/changes/kitty-keyboard/
