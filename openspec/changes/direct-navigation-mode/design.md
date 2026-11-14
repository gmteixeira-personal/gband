## Context

Navigation mode is a key table declared sticky with `gband.keymap.mode`. The client's `Leader` (`crates/client/src/bindings.rs:133`) keeps any declared mode active after a key, and returns to `root` after a key in any other table. The modal preset makes `prefix` that mode. The direct preset binds the same keys in `prefix` with no mode, so every key ends the sequence.

Mouse, wheel, `KeyTableChanged` and the sidebar's mode letter already work for any declared mode by name and label. A second sticky table needs no Rust change.

Three Lua modules name `prefix` directly:

- `gband.jump.setup()` binds `a` and `b` in `prefix` only.
- `gband.keylist` lists, titles and runs `prefix` only.
- `gband.settings` shows the `I on new` line only for the modal style.

`docs/internals/08-key-styles.md` and `docs/internals/07-settings.md` hold the bundled Lua as literate copies, so they change with the code.

## Goals / Non-Goals

**Goals:**
- One Lua source for navigation mode's bindings, used by both presets.
- Direct navigation mode equal to modal navigation mode, key for key, as "Preset parity" in the key-style delta requires.

**Non-Goals:**
- No change to the floating style.
- No change to the client's `Leader`, the keymap API or the wire protocol.
- No new user setting. The key that enters navigation mode stays `space`, rebindable with `gband.keymap.set` as any preset binding is.

## Decisions

### A bundled `gband.navigation` module with `bind(table)`

`crates/lua/src/runtime/gband/navigation.lua` moves modal.lua's `prefix` bindings into `bind(table)`, keyed by the table argument. It is registered in `bundled.rs` `MODULES`, loaded by the prelude as `gband.jump` is, and typed in `meta/gband.lua`. `bind` raises an error for `root` or a non-string, and uses `require_loading` semantics through `gband.keymap.set`.

The interactive-on-new read moves into `bind`, so both tables read it at load.

modal.lua becomes: plugins, `gband.keymap.mode("prefix", { label = "navigation" })`, `gband.navigation.bind("prefix")`, then the `root` mouse names.

direct.lua keeps its one-shot `prefix` bindings, adds `space` after `D` where modal has `escape` and `enter`, declares `navigation` a mode, and calls `gband.navigation.bind("navigation")`.

Alternative: copy modal's 50 lines into direct.lua under `navigation`. Rejected because two copies drift, and "Preset parity" would then rest on a test alone.

Alternative: make direct's `prefix` itself switch between sticky and one-shot. Rejected because the keymap has no per-entry stickiness, and that needs a Rust change.

### `gband.jump.setup(table)` takes an optional table

`setup(table)` declares both modes and binds `a` and `b` in `table or "prefix"`. `bind` calls it after `i`, matching the current order. direct.lua calls `setup()` for its own `prefix` and `bind` calls `setup("navigation")`. Re-declaring a mode and re-setting a binding replace the earlier entry, so two calls leave one set of modes.

### The key list lists the active table

`open` reads `gband.keymap.current_table()` before `gband.keymap.enter("root")`. A non-`root` result is the listed table, and `root` falls back to `prefix`. `entries`, the title and `gband.keymap.run` all use it. From a one-shot `prefix` binding, the client has already reset to `root` before the callback, so the fallback picks `prefix`. Task 3.2 confirms that reading.

Alternative: hard-code `navigation` in the key list. Rejected because a user mode that binds `keylist.open` should list itself too.

### `I on new` shown for direct

`settings.lua` shows the `I on new` line when the `keys` line shows `modal` or `direct`. The direct style's settings window grows from 5 to 6 lines.

## Risks / Trade-offs

- [The `prefix` table's key list from direct shows `space` with a description and no action] → It renders as other function bindings do. The key-list delta gives a scenario.
- [A user module shadowing `gband.navigation` changes both presets] → The behaviour matches every other bundled module. `docs/plugins.md` already says bundled modules can be shadowed.
- [`current_table()` during a one-shot callback might report `prefix`] → Both readings list `prefix`, so the outcome holds either way.

## Migration Plan

None. A user configuration that calls `gband.keystyle.use("direct")` gains `prefix space` and the `navigation` table on the next load. A user binding of `prefix space` made after the call replaces it.
