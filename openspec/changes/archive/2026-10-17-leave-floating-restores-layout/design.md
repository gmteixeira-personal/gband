## Context

See proposal.md - Why. The settings window's `keys` line saves the next key style, which triggers a reload. In the floating style the bundled `gband.desktop` plugin's sweep floats every tiled program window at each attach and reload (floating-key-style spec, "Windows float"). The modal and direct presets set up no desktop plugin, so nothing tiles the floating windows back when the style leaves `floating`. The pre-floating arrangement lives only in the server's session state, which the float sweep has already overwritten.

`gband.layout()` gives the full arrangement: per band, its tiled columns with `width`, `full_width`, `tabbed` and their windows' `rows` or `weight`, and its floating windows with `col`, `row`, `width`, `full_width` and `rows` (lua-control spec, "The layout table"). The session actions and window setters needed to rebuild it, `toggle_window_floating`, `consume_or_expel_left`, `toggle_full_width`, `toggle_column_tabbed`, `set_width`, `set_height` and `set_position`, all run from a key callback and take effect after it returns (lua-control spec, "Action targets" and "Dispatched like an action").

## Goals / Non-Goals

**Goals:**
- Record the layout when the `keys` line switches a non-floating style to `floating`, and restore it when the `keys` line switches `floating` to `modal` or `direct`.
- Restore bands, columns, order, stacking, column width, full-width and tabbed state, per-window height, and floating boxes.
- Tile every floating window when leaving `floating` with no record.

**Non-Goals:**
- Recording or restoring on a reload the user triggers any other way, or on `gband.keystyle.use` called from `user/init.lua`.
- Restoring across a change of the screen area between recording and restoring. Heights and widths are proportions or cell counts as recorded.
- Multi-client coordination beyond what the session actions already give.

## Decisions

**Record and restore run in `gband.settings`, in the `keys` line's step function.** It is the one module loaded in every key style, and the only code that runs in the floating Lua state at the moment the style leaves `floating`. The desktop plugin cannot do it: the preset that replaces `floating` never loads it. Alternative, a new bundled plugin in every preset, rejected: more surface for one behavior that belongs to the settings switch.

**The record is a file under `user/`, written and read through the module's `write` and `read` helpers.** A reload starts a new Lua state, so an in-memory record would not survive the switch into `floating`. The file, `user/floating_restore.lua`, holds `return <table>` serialized from `gband.layout()`, with a `read` validator that accepts only the recorded shape. Alternative, server-side state, rejected: no API for it, and it would outlive the process.

**Entering `floating` writes the record; leaving `floating` consumes and deletes it.** The step function compares the current saved style with the target. On non-floating to `floating` it writes the record from `gband.layout()` before `save` triggers the reload. On `floating` to `modal` or `direct` it reads the record, restores, deletes the file, then `save`. The record's presence is the sole signal that distinguishes "floating was entered through the switch" from "floating was already in use", so opening the settings window already in `floating` and switching away finds no record and tiles every floating window.

**Restore tiles windows in reverse order, each as the band's first column, then stacks with `consume_or_expel_left`.** The client checks `after` against the layout it holds when the call runs, where every window still floats, so an `after` naming a window tiled earlier in the same callback is an error. `toggle_window_floating({ window = id, floating = false })` with no `after` places the window as the band's first column when no tiled window of the band was focused, so tiling in reverse order rebuilds the order. For each band:
- Each floating window not in the record tiles first, so the recorded windows tiled after it land to its left and it follows them.
- Each recorded window tiles from the last column's bottom window to the first column's top window, leaving one column per window in recorded order.
- Each further window of a column, top to bottom, takes `consume_or_expel_left({ window = id })`, which appends it to the bottom of the column on its left.
- The column's `width` is set with `set_width` on its first window, which also turns full width off, then `full_width` with `toggle_full_width` when recorded on, each window's height with `set_height({ rows = r })` or `({ weight = w })`, and last `tabbed` with `toggle_column_tabbed` when recorded on. A height set on a tabbed column is not applied, so tabbed comes after the heights.

A recorded floating window already floats after the sweep; its box is restored with `set_width`, `toggle_full_width` when recorded on, `set_height({ rows = r })` then `set_position({ col, row })`, the order the desktop plugin's "Maximize, restore and tile" uses.

**The restore actions dispatch before the keystyle `save`.** They queue to send when the callback returns; `save`'s file write triggers the reload asynchronously through the watcher, so the server receives the tiling before the reload re-reads the configuration.

## Risks / Trade-offs

- A crash while `floating` is in use leaves a stale `user/floating_restore.lua`. → The next switch out of `floating` consumes and deletes it; the arrangement it restores is the one from before `floating` was entered, which is still what the user expects.
- The restore is a sequence of session actions, so a second client changing the layout between them could interleave. → The complaint and the common case are single-client; the session actions already converge when several clients send the same requests.
- A window closed while `floating` was in use is skipped on restore, and a column that held only closed windows does not reappear. → Expected: the window is gone, so its tile cannot return.
- The screen area changing between record and restore makes a recorded cell height or `col`/`row` no longer fit. → Out of scope; the box is restored as recorded and the layout clamps it as it clamps any box.

## Open Questions

None.
