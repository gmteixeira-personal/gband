## Context

See proposal.md for the motivation. The relevant pieces today:

- `MouseKey` in `crates/core/src/input.rs` holds a `MouseButton` and `Modifiers`. Wheel steps are `MouseKind::Wheel(WheelDirection)` and never reach the key tables.
- `Chord` in `crates/lua/src/api.rs` is `Key`, `Mouse(MouseKey)` or `Prefix`. `Prefix` is resolved by `Keymap::chord_key` in `crates/client/src/bindings.rs` against `Keymap::prefix`, the option's value when the bindings are used. `mod` follows the same pattern.
- `Leader::handle_mouse` looks a press up in the active table: a mode keeps it, any other non-`root` table falls back to `root` after one lookup, and `root` gives `MouseCommand::Default` when unbound.
- `Controls::wheel` in `crates/client/src/mouse.rs` sends every step to its target. `press_default` forwards a press when the program reports the mouse and Alt is not held.
- `call_pressed` runs a mouse binding function with the `MousePressed` payload.

This change depends on drag-bands (and so on band-sidebar), and its deltas are written against the specs as those two changes leave them.

## Goals / Non-Goals

**Goals:**
- Wheel steps take part in the key tables without changing anything for an unbound step.
- One option changes the modifier of every preset mouse binding, whatever the order of lines in `init.lua`.
- A test fails when the two presets drift apart.

**Non-Goals:**
- Changing which modifier gband's text selection uses, beyond Ctrl+Alt.
- Any change to the wire protocol or the server.

## Decisions

### `MouseKey` names a button or a wheel direction

`MouseKey` becomes `{ input: MouseInput, modifiers }`, with `MouseInput::Button(MouseButton)` or `MouseInput::Wheel(WheelDirection)`. `keys.rs` parses `wheelup`, `wheeldown`, `wheelleft` and `wheelright`, and `mouse_name` writes them back.

*Alternative:* a separate `Chord::Wheel`. Rejected: every place that handles mouse chords (parse, list, key list filtering, `find_mouse`) would need a second arm for the same idea.

### `mod` is a flag on the mouse chord, resolved at use

`Chord::Mouse` carries a `uses_mod` flag beside its `MouseKey`. `Keymap` gains `mouse_mod: Modifiers`, read from the `mouse_mod` option as `prefix` is. Matching ORs `mouse_mod` into the chord's own modifiers when the flag is set, then compares as today. `gband.keymap.list` already returns the key as written, so it shows `mod+leftmouse`.

*Alternative:* the presets read `gband.opt.mouse_mod` and bind `alt+leftmouse` directly. Rejected: a user who sets the option after `gband.keystyle.use()` would silently keep Alt, and the key list would show `alt+…` rather than the intent.

### Wheel steps go through `Leader`

`Leader::handle_wheel` returns `Run(binding)` or `Pass`. In a mode or `root` it looks the step up and leaves the table alone. In any other table it runs a bound step and returns to `root`, and passes an unbound step with the table unchanged, so Ctrl+Space then a stray wheel step does not eat the prefix. `Controls::wheel` asks `handle_wheel` first, unless a gesture is held, and runs the binding through the same path as a press binding, with the `MouseScrolled` payload for a function.

### Cooldown keyed by the matched name

`Pointer` keeps the last wheel binding run as `(MouseKey, Instant)`. A step whose `MouseKey` equals it within 150 ms is dropped before lookup: it runs nothing and goes nowhere. Each run of the binding updates the instant, so continuous scrolling runs it once every 150 ms, as niri's `cooldown-ms` does. The value is fixed. The `Instant` comes from the `now` that `Controls::mouse` already takes, so tests drive it.

*Alternative:* a `cooldown` field in `gband.keymap.set` opts. Deferred: no binding needs a different value yet.

### Selection override moves to Ctrl+Alt

`press_default` forwards unless `ctrl && alt`. Alt alone, unbound, is forwarded. With `mouse_mod = "ctrl+alt"`, the preset binding takes Ctrl+Alt+left first, so gband's text selection over a mouse program is then unreachable. That is accepted: a user who picks that modifier chooses the trade.

### "Key bindings" in client-attach is not modified

Four open changes (band-sidebar, window-names, settings-themes, settings-interactive-on-new) each replace that requirement. Replacing it a fifth time would discard one of them at sync. "Default mouse bindings" states instead that "no binding in `root`" there covers keys only.

### Parity test

A Rust test in `crates/client/src/bindings.rs` loads the configuration under each saved style, reads `root` and `prefix` through the same path as `default_prefix_entries`, removes `escape` and `enter` from the modal `prefix`, and asserts equal key lists and equal actions wherever both entries give one. The function entries (`n`, `prefix`) are checked by name only, since their bodies differ by the mode exit.

## Risks / Trade-offs

- [Desktops that take Alt+drag, such as Xfce, never pass it to the terminal] → `mouse_mod` changes it in one line, and the README says so.
- [Some terminals take Ctrl+Alt+drag for a rectangular selection when the program does not grab the mouse] → gband always enables mouse reporting, so those terminals pass it on. A terminal that still takes it leaves gband's selection to its own.
- [Alt+wheel no longer reaches programs with the default bindings] → `gband.keymap.del("root", "mod+wheeldown")` restores it. The proposal marks it BREAKING.
- [A touchpad sends many small steps] → the 150 ms cooldown limits a binding to one run per 150 ms of scrolling.
- [Wheel bindings over a plugin window take the step from `on_mouse`] → only for bound names, which the plugin-windows delta states.
