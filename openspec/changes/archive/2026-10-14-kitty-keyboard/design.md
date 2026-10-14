## Context

Keys travel in two hops. The client reads the user's terminal through `TerminalInputParser` in `crates/core/src/terminal_input.rs` and sends each `Key` (a `KeyCode` plus Shift, Alt and Ctrl) to the server. The server encodes it with `encode_key(key, window.modes())` in `crates/server/src/window.rs:488`, where `modes()` comes from the window's emulator grid in `crates/emulator/src/lib.rs:112`.

Hop one: `TerminalGuard::enter` in `crates/client/src/lib.rs:189` enables bracketed paste and mouse reporting, then `ratatui::init()` enters the alternate screen. Nothing asks the terminal for a keyboard protocol. The decoder already reads `\e[27;m;c~`, `\e[c;mu` and `\e[cu` through `sequence` and `coded_key`, but refuses `:` sub-fields and turns any non-control code, private-use codes included, into a `Char`.

Hop two: `Callbacks` in `crates/emulator/src/callbacks.rs` keeps `modify_other_keys` from `\e[>4;n m`, `Grid::process` zeroes it on `\ec` by splitting the chunk after each reset, and `encode_key` uses it for Enter, Backspace and Tab. vt100 0.16.2 sends every CSI it does not handle to `unhandled_csi`, including `\e[u` with no marker and the `?`, `>`, `<` and `=` forms ending in `u`. vt100 handles `\e[?1049h` itself, with no callback.

crossterm 0.29 provides `PushKeyboardEnhancementFlags`, writing `\e[>Nu`, and `PopKeyboardEnhancementFlags`, writing `\e[<1u`.

## Goals / Non-Goals

**Goals:**
- Shift+Enter and Ctrl+Enter reach a window program that uses either keyboard protocol, from a terminal that supports either.
- A window program's stuck kitty flags cannot outlive its alternate screen.

**Non-Goals:**
- Kitty's event types (flag 2), report-all-keys (flag 8) and associated text (flag 16), on either hop. `Key` has no press, repeat or release, and no Super, Hyper or Meta.
- Querying the user's terminal for either protocol.
- Detecting a stale kitty stack on the main screen after a program crashes. `reset` clears it, as in kitty.

## Decisions

### Ask the user's terminal for both, unconditionally

The client writes `\e[>4;1m` and pushes flags 5 (disambiguate and alternate keys) after `ratatui::init()`, and pops and writes `\e[>4m` in `TerminalGuard::drop` before `ratatui::restore()`. The push goes after `ratatui::init()` because kitty keeps one stack per screen: a push made on the main screen would not apply on the alternate screen gband draws on. The pop runs while the alternate screen is still current for the same reason.

A terminal with both protocols applies kitty's flags over modifyOtherKeys. A terminal with one applies that one. Both requests are no-ops elsewhere, so no query round trip is needed before the first frame.

Alternative: query `\e[?u` with a primary device attributes request and choose one protocol. Rejected: it delays the first frame by a round trip, and the decoder reads both forms anyway.

### Flag 4 on the user's terminal, flag 1 alone for windows

With disambiguate alone, kitty reports Alt+Shift+`1` as `\e[49;4u` and gband cannot tell it is `!` without knowing the layout. Alternate keys add the shifted key, `\e[49:33;4u`, so the decoder can turn it into Alt+`!`, the key a legacy terminal's `\e!` already gives. With no shifted key, a lowercase ASCII letter plus Shift becomes the uppercase letter, which covers terminals that ignore flag 4.

Windows get only flag 1. Programs such as neovim push flag 1, and `Key` cannot represent what the other flags report. The emulator clears unsupported bits on every store, so `\e[?u` tells a program what gband will send.

### Decoder changes

`numbers` splits the first field on `:` into code, shifted key and base-layout key, and keeps refusing `:` anywhere else, so an event-type sub-field still names no key. `coded_key` gets the private-use table for keypad keys and F13 to F35, and refuses every other code from 57344 to 63743, such as modifier keys. `modifiers` already drops bits above Ctrl, which covers Caps Lock and Num Lock. `starts_sequence` adds `E`, `P`, `Q` and `S`, and `sequence` maps bare `P`, `Q` and `S` to F1, F2 and F4. `E`, keypad Begin, names no key.

### Emulator state: a base value and a bounded stack per screen

`Callbacks` gains one `KittyFlags { base: u8, stack: Vec<u8> }` per screen. `unhandled_csi` handles `>`, `<`, `=` and `?` with final `u` and picks the screen from `screen.alternate_screen()`. The `\e[?u` reply goes to `write_back` like the other query replies. The stack holds 16 values, since the kitty spec names no size.

To clear the alternate screen's flags on entry, `Grid::process` splits each chunk after every `\e[?47h`, `\e[?1047h` and `\e[?1049h` it finds, the way it already splits after `\ec`. It compares `alternate_screen()` before and after each segment and clears the alternate flags on a change from main to alternate. A sequence split across two reads still clears, through the before-and-after comparison, unless the same segment also leaves and re-enters.

Alternative: clear on leaving the alternate screen instead. Same effect for programs that exit cleanly. Clearing on entry also covers a program that dies on the alternate screen and is followed by another, which is the Zellij bug class the proposal cites.

### Encoding order

`encode_key` checks `modes.keyboard_flags & 1` first, then the existing modifyOtherKeys path, then legacy. The kitty path returns the CSI u form for the cases "Kitty keyboard encoding" lists, and falls through to the legacy arms for text, cursor, tilde and F1 to F12 keys. Shift+Tab stays `\e[Z`: kitty's disambiguate list covers Escape and keys with Alt or Ctrl, and every reader of the protocol accepts `\e[Z`.

### Tests

- `crates/core/tests/input_encoding.rs` and `crates/core/tests/terminal_input.rs` take a scenario per spec line.
- `crates/emulator/tests/contract.rs` covers the emulator scenarios, written in separate chunks and in one chunk.
- `tests/attach.rs` adds the end-to-end Shift+Enter test, modelled on `ctrl_enter_reaches_a_program_that_asked_for_modify_other_keys`, plus a reattach variant. It also checks the outer requests. The `Attached` harness renders the client through gband's own emulator, so after this change its grid reports kitty flags 1 and modifyOtherKeys level 1 once the client attaches.
- `crates/client/tests/keyboard.rs` checks the exact bytes the client writes on attach and detach, through public functions over any `Write`, the way `frames.rs` tests `synchronized`.

## Risks / Trade-offs

- [Bindings on Tab, Enter or Escape stop matching Ctrl+`i`, Ctrl+`m` and Ctrl+`[` in kitty terminals] → The proposal marks it BREAKING. The default bindings use Ctrl+Space, which reads the same in both forms (`\x00` and `\e[32;5u`).
- [A program pushes kitty flags on the main screen and crashes, leaving the shell to receive `\e[99;5u` for Ctrl+C] → Unmodified Enter stays `\r`, so typing `reset` sends `\ec`, which clears the flags. kitty has the same failure and the same recovery.
- [A program re-enters the alternate screen and expects flags pushed before it left] → It loses them. Programs such as neovim push again after entering. Closing the stuck-flags bug class is worth it.
- [tmux or another multiplexer between gband and the terminal] → tmux applies modifyOtherKeys through its `extended-keys` option and ignores the kitty push, so gband reads the modifyOtherKeys form.
- [A terminal that sends kitty flag 4 fields gband does not expect] → The decoder refuses unknown `:` positions as no key, with no bytes reaching a window.
- [The client panics after ratatui's panic hook has left the alternate screen] → The pop then hits the main screen's stack, which gband never pushed. The pop empties it, and a shell running gband had flags 0 anyway.
