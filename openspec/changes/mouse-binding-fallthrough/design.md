## Context

See proposal.md for the motivation. The code paths this change touches today:

- `Leader::handle_mouse` (`crates/client/src/bindings.rs`) returns `MouseCommand::Run(binding)`, `Discard` or `Default`. In a table that is neither `root` nor a mode, it ends the sequence before it returns. `Leader::handle_wheel` returns `WheelCommand::Run(binding)` or `Pass`, and also ends a non-mode sequence before it returns.
- `Controls::mouse_press` (`crates/client/src/mouse.rs`) sets `Held::Ignored` and `Controls::pressing`, runs the binding, applies its `Outcome`, then clears `pressing`. A function's dispatches are queued while it runs and applied after it returns. A drag action applied while `pressing` is set starts the gesture in `Controls::run_action` (`crates/client/src/lib.rs`).
- `Controls::wheel` records `Pointer::wheel_binding` (the cooldown) before it runs the binding.
- `Runtime::call_pressed` and `call_scrolled` (`crates/lua/src/runtime.rs`) run the callback as `callbacks::run::<()>`, which drops every return value.
- The payload comes from `mouse::payload` and `events::Pointer::payload`. The client's `Hit` already holds the drawn `Region`: `x` and `y` (an `i64`, negative when the screen cuts the box), the full `width` and `height`, and `visible`.
- `gband.win`'s `hooks.mouse` (`crates/lua/src/runtime/gband/win.lua`) adds `line` to the provider event and passes that same table to `on_mouse`.

## Goals / Non-Goals

**Goals:**
- One rule for every table: a declined event behaves exactly as if the table did not bind its name.
- No change for any binding whose function does not return `false`.
- The box fields of a binding's payload, of a mouse event, and of `on_mouse` come from one computation, so they always agree.

**Non-Goals:**
- A way for `on_mouse`, event handlers or key bindings to decline.
- Passing an event on to an earlier binding of the same table.
- Changing which edges `drag_resize_window` picks. The box fields only give Lua the same numbers that the resize rule reads.

## Decisions

### `false` as the first return value declines
Only the boolean `false` declines. Nil, no value and every other value take the event. A function that ends without `return` returns nothing, so every existing binding keeps its behaviour unless it returns `false` on purpose or by the idiom `return cond and act()`.

- *Alternative*: a call such as `gband.mouse.pass()`. Rejected. It adds a field to the API and needs a per-event context. A return value is local to the call and needs no new API.
- *Alternative*: return `true` to take the event. Rejected. Every existing binding returns nothing and would start to decline.

The same rule covers a registered action bound directly to a mouse name. The keymap stores it as `Binding::Callback`, so the client already runs it with the payload, and telling the two apart would need a new binding kind for no gain. A built-in action value always takes the event. A function that raises an error, or does not run because its plugin is marked failed, takes the event, as a key bound to a disabled callback does nothing today.

### The fallback is the unbound path of the table active on arrival
A declined event takes exactly the path of an unbound event in the table that was active when it arrived. No other binding of that table runs, even one made earlier that matches the same event under the "made last wins" rule.

- *Alternative*: fall through to the next matching binding. Rejected. In practice the earlier binding is a preset's `mod+leftmouse` drag, which is exactly what a style that declines content clicks wants to avoid. A chain also makes the result depend on the binding order beyond the one rule that exists.

`MouseCommand::Run` gains the path an unbound press would take, `Default` in `root` and `Discard` elsewhere. The client then needs no second lookup, and a table change made by the function cannot change the fallback.

### The function's dispatches run first, then the fallback
The client applies the declining function's `Outcome` first, then the fallback. Dispatches are already applied after the function returns, and the client learns of the decline only then, so this order follows the order of events. It also lets a function prepare state for the default. For example, `gband.keymap.enter("prefix")` followed by `return false` leaves the press to the `root` default and the client in `prefix`.

- *Alternative*: the fallback first. Rejected. The outcome would have to wait while the default runs, and a table the function entered could steer the fallback.

### A declined press starts no gesture
Before the client applies a declining function's outcome, it clears `Controls::pressing`. `drag_window`, `drag_resize_window` and `drag_band` then find no press and do nothing, as they do when a key dispatches them. The reasons:

- The mouse capability gives a press's release and motions to one owner. The fallback can own them as a forwarded program press, a selection, or a plugin window's `on_mouse` drags, or it can discard them in a mode. A gesture would compete with that owner for the same motions.
- An unbound press never starts a gesture, and a declined press must behave as an unbound one.
- A function that wants the gesture takes the press. It returns nothing, which is today's behaviour.

### A declined wheel step starts no cooldown
`Controls::wheel` records `Pointer::wheel_binding` only after a binding takes the step. The cooldown keeps one flick from running a binding many times. A declined step ran no binding's effect. If it started the cooldown, it would discard the next steps for 150 ms, and those steps should reach the program. A function that declines only in some places, such as over one plugin window, would also swallow the steps that follow it elsewhere. A step inside a running cooldown is still discarded before any function runs, as today.

### A declined wheel step keeps the sequence
`Leader::handle_wheel` no longer ends the sequence. It returns `WheelCommand::Run { binding, ends }`, where `ends` is true in a table that is neither `root` nor a mode. When the binding takes the step and `ends` is true, the client calls `Leader::reset` before it applies the outcome. A `gband.keymap.enter` in the function therefore still wins, as it does today. When the binding declines, the client does not reset, and the step goes to its target, as an unbound step does. `Leader::handle_mouse` keeps ending the sequence before the call, because a press ends the sequence whether it is taken, declined or unbound.

### The runtime reports the decline in `Outcome`
`Outcome` gains `declined: bool`, beside `disabled`. `call_pressed` and `call_scrolled` run the callback as `callbacks::run::<mlua::Value>`, which keeps the first return value. They set `declined` only for `Ran::Returned(Value::Boolean(false))`. `Ran::Failed` and `Ran::Disabled` leave it false. Every other `Runtime` call leaves it false. `within_callback` reports the flag next to `disabled`.

- *Alternative*: return `(Outcome, bool)` from the two calls. Rejected. `Outcome` already carries `disabled`, a flag of the same kind, and the client already passes `Outcome` around whole.

### Box fields come from the hit region
`mouse::payload` fills the new `Pointer::boxed: Option<BoxCell>` from `hit.region` when the target is `Window`, `DrawnPlugin` or `PluginFloat`. `BoxCell` holds `col` (the event's column less `region.x`), `row` (its row less `region.y`), `width` (`region.width`) and `height` (`region.height`). `BoxCell` lives in `gband_lua::events` beside `Pointer`. `PluginMouse` gains the same `boxed` field, filtered for drags and releases off the plugin window in the same way that `content` is filtered today. `Pointer::payload` and `PluginMouse::to_lua` write `box_col`, `box_row`, `box_width` and `box_height`, or leave them nil.

`Region` is the one source that resolves the target, the content cell and the edges of `drag_resize_window`. The box fields therefore equal the `x`, `y`, `w` and `h` of the mouse capability's "Resize by dragging", and a Lua function can predict which edges a resize gesture will move. The fields count the whole box from its real top-left cell, so a tile that the camera cuts on the left gives its first shown cell a `box_col` above 0.

- *Alternative*: let Lua compute the box from `gband.layout()` and the view. Rejected. During animations, a lifted tile and camera slides, the drawn frame differs from the layout, and Lua cannot see the drawn positions.

`on_mouse` receives the fields with no change to `win.lua`, because `hooks.mouse` passes the provider's event table on. The provider event's documented fields grow. Adding fields keeps the API version.

### `gband.api_version` becomes 2
The rule in the lua-api capability is literal: a build raises the version when it changes the documented effect of a documented function. `docs/plugins.md` documents that in `root` "a bound mouse name replaces the interactive defaults". After this change, a binding whose function returns `false` stops replacing them. That is a change of documented effect, so the version rises. Returning `false` by accident is plausible, with `return e.target == "window" and act()`, which is the case the rule protects. The box fields alone are additions and would keep the version.

The version rise reaches every place that states `1`:
- `API_VERSION` in `crates/lua/src/runtime.rs`;
- the bundled plugins `gband.errors`, `gband.keylist` and `gband.prompt`, which would otherwise log a mismatch warning on every load;
- the example plugins and the tutorial plugins, which the tutorials capability's "Tutorial API version" test checks;
- the docs that state the version, the "API and stability" table, and the internals chapters that quote the bundled plugins verbatim;
- the tests in `crates/lua/tests/plugins.rs` that assert `1`, and the mismatch test that declares `api = 2`.

### Docs
- `docs/plugins.md` is the reference. It gets the decline rules under "Mouse names", the four fields in the mouse fields, in `on_mouse`'s table and in the provider's `mouse(id, event)`, version 2 everywhere, and the change list entry.
- `docs/tutorial/01-keys.md` teaches mouse names, so it gets a short example that marks a window from its top border and declines every other press. The example needs `box_row` and `return false`, and a case in the chapter's tests.
- `docs/tutorial/04-events.md` and `docs/tutorial/07-plugin-windows.md` name neither the mouse fields nor `on_mouse`, so they do not change.
- `docs/internals/06-window-provider.md` explains the provider's mouse event, so it gets one sentence on the box fields. Its quotes of `win.lua` stay valid because `win.lua` does not change.
- `docs/internals/09-sidebar-and-errors.md` and `docs/internals/10-keylist-and-prompt.md` quote `api = 1` from bundled files, so their quotes change with the files.
- `docs/internals/08-key-styles.md` describes the presets' mouse bindings, which do not change.
- `README.md` states that a mouse binding replaces what a click does, so it gets one sentence on returning `false`.
- `docs/testing.md` states the version, so it changes to 2. The test side gains nothing.

## Risks / Trade-offs

- [A configuration that returns `false` from a mouse binding by accident changes behaviour] → The version rises to 2, and the "API and stability" table names the change. A plugin that declares `api = 1` gets a log warning that points to it.
- [Another change in the same release also raises `gband.api_version`] → The docs say that a release raises the version by one. The integrator merges both changes into the one version 2 row, and the spec deltas that state the value agree on 2.
- [A function that declines and also dispatches focus loses to the default's focus] → The order is specified: the function's dispatches first, then the fallback. A test shows a dispatched table change surviving the default.
- [The box of a cut tile reports cells that are not on screen] → This is specified and tested. Code that wants only the shown part can compare `col` with the terminal's size.
- [Moving the sequence reset out of `handle_wheel` changes when a non-mode sequence ends] → The reset now runs after the function, so `gband.keymap.current_table()` inside a wheel function still reads the table before the step, as today. The existing "Unbound wheel step keeps the sequence" tests and new tests for declined steps cover the transition.

## Migration Plan

Users with a mouse binding function that returns `false` and should keep replacing the default: return nothing instead. Plugin authors: check every mouse binding, then set `api = 2`. Rollback is a revert of the change's commits. No protocol message, saved setting or file format changes.
