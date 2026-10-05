## Context

See proposal.md for the motivation. This change assumes config-directory has been archived. After that change, the default configuration is the `include_str!` text in `gband-lua`, written to `defaults/init.lua` on first run. An evaluation starts from `Options::default()` and no bindings, so the `prefix` default lives in two places: the `gband.set` call of the default configuration, and `Options::default()` for a `user/init.lua` that never sets `prefix`. The "Defaults reproduce the built-in behaviour" test pins the two to the same value.

The key-name parser already reads `ctrl+space` as `KeyCode::Char(' ')` with Ctrl. The input encoder already writes Ctrl+Space as `\x00`. crossterm 0.29 decodes a `\x00` byte from the client terminal as `KeyCode::Char(' ')` with `KeyModifiers::CONTROL`. The client enables no extended keyboard protocol.

## Goals / Non-Goals

**Goals:**
- One edit to each of the two places that hold the default prefix, with no new code path.
- Every test that presses the default prefix presses Ctrl+Space.

**Non-Goals:**
- Changes to `matches`, the leader state machine, key names or the input encoding.

## Decisions

### Change the value, not the mechanism
`prefix = "ctrl+space"` in the default configuration and `Key::new(KeyCode::Char(' '), Modifiers::CTRL)` in `Options::default()`. The leader, `send_prefix` and the direct-binding check all read the prefix from the loaded options, so they need no change.

*Alternative:* a special `KeyCode::Null` for the NUL byte. The decoder and the parser already agree on `Char(' ')` with Ctrl, so a new code would add a translation and change nothing visible.

### Tests send `\x00` as the prefix byte
End-to-end tests write raw bytes into the client's terminal, so `b"\x01"` becomes `b"\x00"` wherever it stands for the prefix. Tests that set their own prefix, such as `ctrl+b`, stay as they are. Unit tests that build the default keymap press `key("ctrl+space")`. Tests that pin Ctrl+A as unbound, or as reaching the pane, keep Ctrl+A on purpose.

### The literal prefix is checked with `cat -v`
Bash binds `\x00` to `set-mark`, which changes nothing on screen. The "Literal Ctrl+Space" test runs `cat -v` and expects `^@`, so the one byte that reaches the pane is visible. The old bash cursor check stays under "Literal Ctrl+A", where it now shows that Ctrl+A is no longer captured.

## Risks / Trade-offs

- [Ctrl+@ and Ctrl+2 produce the same `\x00` byte, so they also start a prefix sequence] → Accepted. Without an extended keyboard protocol the terminal does not tell them apart, and tmux and screen have the same limitation.
- [Some desktops capture Ctrl+Space to switch input methods, so it never reaches the terminal] → A user in that position sets `prefix` in `user/init.lua`. The README shows how.
- [A user without a `user/init.lua` loses Ctrl+A as the prefix on upgrade] → Accepted as a breaking change of an early-development default. `defaults/init.lua` shows the new prefix after the first start.
- [Emacs binds Ctrl+Space to set-mark] → An Emacs user presses Ctrl+Space twice, the same cost Ctrl+A had for every shell user.

## Migration Plan

None in code. A user who wants Ctrl+A keeps or adds `gband.set { prefix = "ctrl+a" }` in `user/init.lua`.
