## Context

The sidebar is the bundled Lua plugin `gband.sidebar`. Its band click is a `MousePressed` handler: the client handles the press first, and a press outside the ribbon does nothing in core, then the event reaches the handler, which checks `target == "outside"`, the bar's column and the row, and calls `gband.band.view`. Wheel steps already emit `MouseScrolled` with `direction` and the mouse fields for every target, including `"outside"`, after the client has handled them.

## Goals / Non-Goals

**Goals:**
- Wheel over the sidebar moves through the bands with no change outside the plugin.

**Non-Goals:**
- Changing the mouse capability's "Wheel" requirement. Core still makes no use of the wheel and does nothing with a step outside the ribbon; the plugin acts on the reported event, as the click does.

## Decisions

- **A `MouseScrolled` handler in the plugin, not a core binding.** The click is built the same way, the event already carries everything needed, and the bar API has no mouse hook. A core rule would also have to modify the mouse "Wheel" requirement again, right after `alt-mouse` rewrites it for wheel bindings.
- **Dispatch `gband.action.focus_band_down()` and `focus_band_up()`.** They view relative to the viewed band, run the band switch animation, and stop at the first and last band. Computing a band from the pointer row with `gband.band.view` was rejected: the wheel names a direction, not a band, and the row under a resting pointer would pin the view to one band.
- **Hit test on the bar's column only.** The handler accepts `target == "outside"`, `info.shown` and `ev.col == info.col`, on any row, because the whole one-column bar is the wheel's surface.
- **Ignore steps with Ctrl, Alt or Shift.** `MouseScrolled` reports a step after a binding has taken it, and the handler cannot tell. This change builds on `alt-mouse`, where `alt+wheeldown` is bound to `focus_band_down`, so acting on it too would move two bands per step. Plain steps are never bound by the presets. A wheel step that `alt-mouse` discards in its 150 ms cooldown is a modified step, so the sidebar never sees it as its own. With the default keys, an Alt step over the sidebar therefore moves one band, through the binding alone, as a plain step does.
- **No cooldown.** Each step moves one band, as the user chose. Handlers dispatch actions in order, so two quick steps move two bands.

## Risks / Trade-offs

- [A user binds plain `wheeldown`, which `alt-mouse` allows] → over the sidebar both the binding and the handler act. The default presets bind no plain wheel name, so only a user binding hits this.
- [A trackpad flick sends many steps] → the view passes several bands. Accepted: the spec moves one band per step.
