## Context

A key crosses gband twice. The client decodes the user's terminal bytes into a `Key` (code plus Shift, Alt and Ctrl) in `crates/core/src/terminal_input.rs`. It sends that `Key` over the wire, and the server re-encodes it for the window's PTY in `encode_key` (`crates/core/src/input.rs`), using the window's `Modes`. `Modes` comes from the server's grid, `Vt100::modes()` in `crates/emulator/src/lib.rs`, which reads vt100's screen state.

vt100 0.16 sends `\e[>4;n m` to `Callbacks::unhandled_csi` as `(Some(b'>'), None, [[4], [n]], 'm')`. Our callbacks already answer queries and keep the title stack there. vt100 handles a full reset, `\ec`, inside `Screen::ris()`. That replaces the screen and calls no callback, so the callbacks cannot see it.

The wire `Key` already carries all three modifiers, so the wire protocol does not change.

## Goals / Non-Goals

**Goals:**

- Ctrl+Enter from Ghostty reaches fish under gband as `\e[27;5;13~`.
- Programs that never ask for modifyOtherKeys get exactly today's bytes.

**Non-Goals:**

- The kitty keyboard protocol, both answering `\e[?u` and the progressive enhancement flags.
- modifyOtherKeys for character keys. At level 2, xterm also encodes Ctrl and Alt with characters, such as Ctrl+`1`, in this form. That changes a large surface that no reported problem needs.
- Asking the user's terminal for modifyOtherKeys or kitty mode. gband reads whatever the terminal sends by default.
- Answering the `\e[?4m` query (XTQMODKEYS).

## Decisions

### Track the level in the emulator callbacks, not in vt100's screen

Keep a `modify_other_keys: u8` in `Callbacks`. Set it in `unhandled_csi` for `(Some(b'>'), None, 'm')`. `Vt100::modes()` reads it into a new `Modes` field.

The alternative was to fork or patch vt100 to model the mode. That is more code to own for one byte of state.

vte dispatches a CSI with no parameter as one parameter of 0, so `\e[>m` and `\e[>0m` reach the callbacks identically. Both clear the level. In xterm, `\e[>0m` resets only modifyKeyboard, but no program is known to write it while expecting modifyOtherKeys to survive, and telling the two apart would need a second byte-level scanner.

### Detect a full reset at the byte level in `Vt100::process`

A full reset never reaches the callbacks, so `Vt100::process` scans each chunk for ESC directly followed by `c`, which is exactly when vte dispatches RIS. On a match it clears the level. A flag carries a chunk-final ESC into the next chunk, so a reset split across PTY reads is still seen.

The alternative was to ignore resets. Then `reset` after a crashed fish would leave Ctrl+Enter sending `\e[27;5;13~` to bash, which prints it as garbage. xterm clears the mode on RIS, and gband should match.

### Treat levels 1 and 2 the same, for Enter, Tab and Backspace only

At level 1, xterm uses the `27;m;c~` form only for keys whose modified encoding would otherwise be ambiguous. Enter, Tab and Backspace with Shift or Ctrl are exactly those keys. At level 2, xterm extends the form to every modified key, characters included. Because character keys are a non-goal, both levels give the same bytes. The level stays a number, not a bool, so the character keys can be added later without changing the emulator.

Shift+Tab alone keeps `\e[Z`, as xterm does at both levels. Escape keeps its legacy bytes: fish and readline expect a lone ESC there, and no request needs a modified Escape.

### Decode both the modifyOtherKeys form and plain CSI u

Ghostty, foot and xterm send `\e[27;m;c~`. kitty, WezTerm with CSI u enabled, and some other terminals send `\e[c;mu`. Both name the same thing, so one mapping from a code to a key serves both. Kitty's colon sub-parameters, such as alternate keys and event types, stay unparsed. gband never asks for them, so a terminal that sends them anyway gets no key, as for any other unknown sequence.

A decoded Shift-only Tab becomes `KeyCode::BackTab` with no modifiers, the same `Key` that `\e[Z` produces today. Bindings and the encoder then treat it the same whichever way the terminal sent it.

### Keep `Modes::DEFAULT` the zero point

The new field defaults to 0 in `Modes::DEFAULT`. Struct literals that spread `..Modes::DEFAULT` keep compiling. `Vt100::modes()` is the only literal that lists every field, and it gains the new one. The harness encodes test keystrokes with its outer terminal's `Modes`, and that terminal never asks for modifyOtherKeys, so the harness keeps sending legacy bytes. An end-to-end test therefore writes the raw sequence with `type_text`.

## Risks / Trade-offs

- [A program enables modifyOtherKeys and then mis-parses `\e[27;5;13~`] → It asked for the mode, and xterm and Ghostty send the same bytes, so gband matches their behaviour.
- [A byte-level RIS detector could disagree with vte] → vte dispatches RIS for ESC followed by `c` from any state, string states included, because ESC ends the string. A unit test feeds `\ec` split across two `process` calls and inside an OSC.
- [The level is lost from a snapshot] → Keys are encoded only with the server's grid, which is never rebuilt from a snapshot. `Window::replace` rebuilds from raw output, which carries the request again.
- [Ctrl+Backspace changes from `\x08` to `\e[27;5;127~` for programs at level 1] → That is the point of the mode. Fish then reads it as `ctrl-backspace` rather than `ctrl-h`.
