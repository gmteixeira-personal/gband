## Context

The sidebar draws `!` on its last row while the client reports an error. Its `MousePressed` handler in `crates/lua/src/runtime/gband/sidebar.lua` returns early for any button but `left`, and acts only on the apps character, band labels and agent labels. A press on `!` falls through and does nothing.

The `gband.errors` plugin registers `errors.open` and `errors.clear`. The default configuration sets it up after `gband.keystyle.use()`, so the preset loads before either action exists. `gband.plugin` refuses a second setup of one plugin in a load, so a preset cannot set up `gband.errors` itself: a user calling `gband.plugin("gband.errors", { kind = "tiled" })` after the preset would then get a plugin error.

The tutorial's chapter 00 binds `prefix e` to `errors.open` by hand, and all 12 chapters' `user/init.lua` files carry that line. The README's "Errors" section says no key is bound and suggests `C` for clear and `E` for a tiled list.

## Goals / Non-Goals

**Goals:**
- A left press on the drawn `!` opens the error list, and a right press clears it, in every key table.
- `e` and `E` in `prefix` open and clear the list with the modal and direct styles.
- Both work whether `gband.errors` is set up before the preset, after it, or with options.

**Non-Goals:**
- The floating style gains no `e` or `E`. Its `prefix` opens the window list, and its bindings are a fixed short list in the floating-key-style spec.
- A press on the blank marker row stays a no-op.
- The error list's own `c` and `q` keys stay as they are.

## Decisions

1. Resolve the actions at press time. The sidebar and the presets read `gband.action["errors.open"]` and `gband.action["errors.clear"]` inside the handler or binding function, and do nothing when the entry is nil. The sidebar already does this for `desktop.list`. Binding `gband.action["errors.open"]` directly in the preset reads nil at load time, since the default configuration sets up `gband.errors` after the preset. Moving the setup into the presets breaks a later `gband.plugin("gband.errors", opts)` call with a duplicate-setup error.

2. Bind `e` and `E` as Lua functions with fixed descriptions, `list the errors` and `clear the errors`. These match the descriptions the plugin gives its two actions. A function binding takes its description from the client-attach table, as the key-style spec requires.

3. Place `e` and `E` after `m` and before `D` in both presets. Both presets get the same two functions, so "Preset parity" holds with no new allowed difference.

4. `e` returns to interactive mode through the error list itself. `errors.open` calls `gband.keymap.enter("root")` before it opens the window, as `?` and `:` do. The binding adds no `enter` of its own, so a missing plugin leaves navigation mode active. `E` keeps navigation mode, as other non-opening keys do.

5. In the sidebar handler, test the marker before the apps character. The marker cell is row `h − 1`, which is row 0 when the bar is 1 row high. The existing apps-character test already skips row 0 on a 1-row bar with an error. Testing the marker first makes that case explicit, and the right button reaches only the marker branch.

6. The right press reaches the sidebar in every style. The lua-events spec emits `MousePressed` for every press after a binding, a default or a gesture handled it. The modal and direct presets bind `rightmouse` in `prefix` to `drag_resize_window`, and the floating preset binds it in `root` to `desktop.menu`, which declines bar presses. The handler still runs in both cases, as it does for a left press on a band label in navigation mode. A test confirms that `drag_resize_window` starts no gesture on an `outside` press.

7. Remove the hand-made `prefix e` line from the 12 tutorial `user/init.lua` files and from chapter 00's prose. Chapter 00 then says the key style binds `e` and `E`. Chapter 01's `keys.e` check still passes, now through the preset.

## Risks / Trade-offs

- A user configuration that binds `prefix e` or `prefix E` keeps its own binding, since a later `gband.keymap.set` replaces the preset's. The README's old `E` example for a tiled list conflicts with the new `E`, so the README example changes to another key.
- The key list's `prefix` page grows by 2 lines, so its screenshot references change.
- A right press on `!` clears without confirmation. The errors stay in the logs, and the next load reports them again, so nothing is lost for good.
