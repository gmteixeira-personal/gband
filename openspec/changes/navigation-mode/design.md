## Context

Key handling lives in two places. `crates/lua/src/keymap.rs` holds the bindings while the configuration loads and hands the client a `KeyTables` map (`table -> [(Chord, Binding)]`). `crates/client/src/bindings.rs` holds `Leader`, which tracks the active table. In a table other than `root`, `Leader::handle` resets the active table to `root` before it runs the binding, and a `Dispatch::Enter` queued by the binding's callback then sets it again. That callback-driven re-entry is the only mechanism for a table to outlive one key, and it works only for function bindings.

The key list (from the key-list change, a dependency) is a focused floating plugin window that already takes every key in `root`. It can run only action bindings, because `gband.keymap.list` exposes no handle to a function.

This change builds on floating-windows and key-list, both of which also edit `defaults.lua` and the default key table, so it starts after both are archived.

## Goals / Non-Goals

**Goals:**
- A key table can stay active across keys. Lua declares this, and Lua bindings decide when to leave.
- The default configuration expresses navigation mode with the public API only. A copy of it in `user/init.lua` behaves the same.
- Every default binding can be run from the key list.

**Non-Goals:**
- A Rust notion of "interactive" or "navigation". Rust knows modes as sticky tables and nothing more. Interactive mode is `root`.
- Changing how floating plugin windows receive keys in `root`.

## Decisions

### A mode is a sticky key table, declared from Lua
`gband.keymap.mode(table, { label })` records the table in a `modes` map in `Keymaps`, beside `bound`. `finish` returns the mode set with the key tables, and `Config` gains a `modes` field passed to the client's `Keymap`. In `Leader::handle`, a table in the mode set is not reset to `root`. A bound key runs and the table stays. An unbound key returns `Command::Discard` and the table stays. A `Dispatch::Enter` from the binding still overrides the active table, exactly as today.

Alternatives considered:
- *Every navigation binding is a function that runs its action and re-enters `prefix`.* This needs no Rust change, but every binding becomes a function. That loses the action names that the hints labels and the key list rely on, and it makes the defaults noisy.
- *A per-binding `exit` flag.* It moves the exit decision out of Lua code into a Rust-interpreted option. The user wants the configuration itself to do "open a window, then switch to interactive mode".
- *Rename the `prefix` table to `navigation`.* It breaks `gband.bind("prefix x")`, every user configuration, and the key-list and key-hints specs. A label gives the display name without the break.

### Leaving a mode is `gband.keymap.enter("root")`
`enter` currently refuses a table with no binding, and the defaults bind nothing in `root`. `root` is exempted from that check. The defaults bind Escape and Enter to `function() gband.keymap.enter("root") end`. They bind `n` to a function that calls `gband.action.open_window()` and then `gband.keymap.enter("root")`, and Ctrl+Space likewise with `send_prefix`. The dispatch queue runs entries in order, so the action is dispatched before the table changes.

### `gband.keymap.run(table, key)` runs a binding, and the key list uses it
`run` parses `key` with the same chord rules as `gband.keymap.set`, `prefix` included. It finds the binding in `Keymaps.bound` by `(table, chord)` and runs it in the calling callback's context:
- An action binding queues `Dispatch::Action`.
- A registered action or function binding calls its Lua function directly, through the callback registry, under that callback's owner. The function's own dispatches join the caller's queue.

Nothing goes through `Leader`, so the active table changes only through a queued `Enter`. The key list's Enter handler calls `gband.keymap.run("prefix", entry.key)`. Its muting rule shrinks to the `keylist.open` line.

Alternative considered: *`gband.keymap.list` returns a callable per entry.* Callables in plain data tables leak function identity to plugins and complicate the list's shape. `run` keeps `list` as plain data.

### The key list enters `root` when it opens
Without this, `?` pressed in navigation mode would open the list, and the following `j`, `k` and Enter would still be read by the `prefix` mode, never reaching the floating plugin window. `keylist.open` calls `gband.keymap.enter("root")` in its own callback. The decision stays in the plugin's Lua, consistent with the rest. A general rule such as "a focused floating plugin window pre-empts modes" was rejected: it would also stop navigation keys from working while any float is focused.

### Labels
`gband.keymap.label(table)` reads the `modes` map. `mode.lua` renders `gband.keymap.label(ctx.table)`, `hints.lua` labels the root prefix hint with `gband.keymap.label("prefix")`, and `keylist.lua` titles the floating plugin window `label .. " keys"`. `KeyTableChanged` and the render context keep the table name, so existing handlers keep comparing against `"prefix"`.

### Default binding order
The order is `h l j k u i n q [ ] r f - = _ + R`, then the floating-windows keys, then `? D escape enter left right down up prefix`. Escape, Enter and the arrow keys sit after the main keys. The arrows repeat hjkl, so placing them late keeps the hint bar's leading run informative. The key-list spec keeps `h` as the first line and `?` between `R` and `D`.

## Risks / Trade-offs

- [Users used to Ctrl+Space then a key, then typing] → The mode segment shows `navigation` while keys are captured, and Escape or Enter leaves. The breaking change is called out in the proposal and README.
- [A lone Escape byte can be read as the start of an Alt sequence] → crossterm reports a lone ESC as Esc when no byte follows in the same read. End-to-end tests that leave navigation mode send Enter (`\r`) instead of Escape where timing could merge bytes.
- [A copied `defaults/init.lua` from before this change] → It declares no mode, so `prefix` stays one-key and `prefix enter` still opens a window. Nothing breaks, and the user opts in by declaring the mode.
- [`gband.keymap.run` from a binding in the same table could recurse] → The instruction limit already bounds runaway Lua. `run` adds no guard of its own.

## Migration Plan

No data migration. Users with their own `user/init.lua` keep one-key prefix behaviour until they add `gband.keymap.mode("prefix", { label = "navigation" })` and the exit bindings. The README shows the snippet.
