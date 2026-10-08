## Context

See proposal.md, "Why". The state this change builds on:

- `View` in `crates/core/src/view.rs` is each client's own state. It holds the viewed `band`, a `BandView { layer, tiled, floating, camera, .. }` per band, a `recency: HashMap<WindowId, u64>` with a `tick` counter, and the `minimized` set. `View::focus` places the focus and calls `touch`, which raises `tick` and stores it in `recency`. `View::stacked` derives the stacking order from `recency` and the floating list, so the stacking order and the focus order are one structure today. `View::settle` resolves focus after every change, and keeps a still-available remembered window with `place_focus`, without a touch.
- The client draws floating windows, and builds the regions that pointer targets resolve against, from one loop over `View::stacking` in `crates/client/src/render.rs`. `View::shown` feeds the shown message. `Display::hidden` in `crates/client/src/lib.rs` ends a gesture or drops a selection whose window is hidden, after every view action and layout.
- Lua reads the view through `ui::ViewState` (`crates/lua/src/ui.rs`), which `Display::view_state` fills. `gband.window.focus`, `gband.layout()` and `gband.view()` live in `crates/lua/src/control.rs`.
- The client sends a motion with no button held only to the focused window's program (`Controls::mouse_motion`, `Held::Free | Held::Ignored`). Plugin windows get mouse events through the windows provider's `mouse(id, event)`, built from `PluginMouse` in `crates/lua/src/plugin_windows.rs`. The client learns what to draw for a plugin window from the frame that `gband.core.present_window` sets: it keeps floating frames in `PluginWindows::floats` (`crates/client/src/plugin_windows.rs`) and turns tiled frames into content for the server.
- `gband.win` (`crates/lua/src/runtime/gband/win.lua`) keeps a focused floating plugin window focused across a `FocusChanged` or `BandChanged` when it is `held`. `hooks.key` and `hooks.paste` hold it, and `hooks.mouse` already holds it before `on_mouse` for every kind but `"scroll"`. A key press or a paste releases the hold. The spec documents only the `keys` case.
- A successful reload (`Controls::reload` in `crates/client/src/lib.rs`) closes every plugin window without running `on_close`, and keeps the `Display` and its `View`.
- `gband.api_version` is 2. The lua-api capability's "API version rule" keeps it for a build that only adds to the documented API.

## Goals / Non-Goals

**Goals:**
- A peek that a Lua caller can undo exactly: after the caller focuses the window that was focused before, or views the band it viewed, the band, the focus, every band's stacking order and the minimized set are as before the peek.
- One source of truth for what the client draws, targets and reports as shown: `View::stacking` and `View::shown`, as for minimize.
- Expose the focus order that already exists, rather than adding a second one.
- Keep every change an addition, so `gband.api_version` stays 2: a plugin window, a windows provider or an `on_mouse` written for today behaves exactly as today.

**Non-Goals:**
- Undoing camera moves. A peek moves the camera as any focus does.
- A built-in event for hover, or for the start and end of a peek.
- Routing a motion with no button held only to the target under the pointer.

## Decisions

### A peek is an overlay on the recorded view

`View` gains `peek: Option<WindowId>` and `ViewAction::Peek(WindowId)`. The recorded state, `recency`, `tick`, `minimized` and each band's `layer`, `tiled` and `floating`, is never written by a peek. While `peek` is set:

- `focused()` returns the peek, `layer()` returns the peek's layer, and `tiled_focus()` returns the peek when it is tiled, so the camera, `resolve` and `centred_box` act on it as on any focused window.
- `available(band, window)` accepts the peek even when it is minimized, so `settle` keeps it focused.
- `stacking()` returns the recorded order with the peek moved, or added, on top. `stacked()`, which `top_floating` and every focus fallback read, stays the recorded order, so a peek never steers where focus falls back to.
- `shown()` and a new public `hidden(window)` treat a peeked minimized window as shown.
- The viewed `band` moves to the peek's band, as a focus does. The client's existing `FocusChanged` and `BandChanged` checks then fire with no change.

`View::peek(window, scene)` sets `band` and `peek` and settles. It touches nothing else. `View::focus`, the one path that records a focus, clears `peek` first, and so does `View::enter`, which every band change runs. Every focus action, `focus_window`, a band change, `release_slide` and the fallback in `settle` therefore end the peek as the spec requires. `settle` keeps a peek only while the viewed band holds the peeked window.

*Alternative:* snapshot the recorded state at the first peek and restore it when the peek ends. Rejected: a minimize, a layout change or another client's focus message during the peek would be lost by the restore, or would need merging back.

*Alternative:* a `peek` flag on `FocusWindow`. Rejected: `FocusWindow` is matched in the client's mouse code and the `focus_window` primitive, which must keep recording. A new variant leaves every existing match unchanged.

### Ending a peek returns to what was recorded

When a peek ends without a new recorded focus, because the peeked window leaves the layout, the client views the band it already views, it minimizes the peeked window, or a reload succeeds, `View` clears `peek` and resolves focus for the viewed band as `enter` does, except that a remembered window that is still available is placed with `place_focus`, without a touch. The focus then equals the recorded one, and `last_focus` values do not move. Only when the remembered window is gone does focus fall back as "Switch band" defines, which records a focus.

- One rule covers every way a peek ends without a new focus, so `settle` needs one branch, not a peek case in each fallback.
- It is what an undo means: the peek was never recorded, so the recorded focus shows again.

A focus of the peeked window by other means goes through `focus_window`, which clears the peek, restores and touches the window. The focused window and the band are the same before and after, so the client's existing comparison emits no `FocusChanged` and no `BandChanged`. The spec states this, because a window list that refocuses its previewed window on Enter must not see a focus event it would take for a new selection.

*Alternative:* treat the peeked window as the focused one for the fallbacks of "Follow layout changes" and "Focus follows a window between layers". Rejected: the fallbacks read `position` and the remembered windows, which a peek must not write, and a peek is a preview, not a place to fall back from.

### Viewing the viewed band ends a peek

A band whose windows this client has all minimized has no focus. After a peek of one of them, no focus action and no band change returns to "no focus". `gband.band.view` of the viewed band, a no-op today, is the natural undo for that case, and for every case where the caller wants to drop the preview and keep the band. It changes nothing when no peek lasts, so no existing behaviour changes.

*Alternative:* a separate `gband.window.unpeek()`. Rejected: it adds a function outside the agreed contract, and `gband.band.view` already means "show this band as I left it".

### Minimizing the peeked window ends the peek

The title bar's `[_]` of a peeked window minimizes it. The user expects the window to go away, also when it was already minimized. `View::minimize` therefore treats the peek as the focused window: it ends the peek, inserts the window when the set does not hold it, and returns to the recorded focus.

### A successful reload ends a peek

A successful reload closes every plugin window, and `on_close` does not run, so the window list that started a peek cannot undo it. A peek that outlived its list would leave a minimized window drawn and focused with nothing on screen to explain it. `View` gains a public `end_peek(scene)`, the same return as above, and `Controls::reload` calls it through `Display::with_view` in its success branch, where it closes the plugin windows, before the events of the reload are emitted. A failed reload keeps the previous configuration and its plugin windows, so it keeps the peek too. The focus counter, the stacking order and the minimized set survive the reload as before.

*Alternative:* keep the peek across the reload, as the view keeps everything else. Rejected for the reason above: the peek belongs to the list, and the reload removes the list.

### A peeked tiled window is a focused tile

Tiles do not stack, and focusing a tile does not draw it above floating windows. A peek of a tile is therefore a focus that records nothing: the tiled layer shows as active, the camera moves to it by the `center_focused_column` policy, and floating windows still cover it. The window list of the floating key style lists tiled windows that run a program, so this case is reachable.

### The focus order is the existing recency

`last_focus` is `recency[window]` and the counter is `tick`. `View::last_focus()` returns the map. They already have every property the spec states: `tick` starts at 0 in `View::with_policy`, the first focus makes it 1, `settle` prunes windows that left the layout, `place_focus` without a touch is how focus stays on a window, and the reload keeps the `Display` and its `View`. A new attach builds a new `View`. The stacking order is derived from the same map, which is why `gband.layout()` can document it as "never focused in floating-list order, then ascending `last_focus`".

`ui::ViewState` gains `last_focus: BTreeMap<WindowId, u64>` and `peek: bool`, filled by `Display::view_state`. `control.rs` sets `last_focus` on window and floating tables, and `peek = true` on `gband.view()`'s table only while a peek lasts, so the existing exact-table scenarios of "Read the view" and "Floating window" keep their shape. Neither field joins `drawn_differs`, so `gband.core.on_state` keeps its documented triggers.

*Alternative:* report the stacking order as a list in `gband.layout()` or `gband.view()`. Rejected: the window list needs per-window recency for every window, tiled ones included, and the stacking order follows from it.

### The options of `gband.window.focus`

`focus(lua, (target, opts))` reads `opts` as nil or a table. It checks the fields with the same "takes no field" rule as action targets, and `peek` as a boolean, before it queues anything. `peek = true` queues `Action::View(ViewAction::Peek(window))`; nil, `{}` and `peek = false` queue `FocusWindow` as today. `gband.core.focus_window` keeps its one argument and always records.

### Hover is opt-in, through the frame

`gband.win.open` takes `hover`, checked like `cursorline` and kept in the window record. `gband.win.info` reports it, which only adds a field. `gband.win.set_config` does not take it: it changes only the geometry, border and title of a floating plugin window, it rejects every behaviour option, such as `cursorline`, `keys` or `on_mouse`, as an unknown field, and it is an error on a tiled plugin window, while `hover` applies to both kinds. A plugin that wants hover only some of the time can ignore `"move"` in its `on_mouse`.

The client must not call the provider for a move over a plugin window that did not ask for it, because the windows provider is documented API: a replaced `gband.win` written for today expects four kinds in `mouse`. The provider therefore tells the client which plugin windows take hover, through the frame it already presents: both frame kinds gain an optional boolean `hover`, read by `read_frame` into `FloatingFrame` and `TiledFrame`. `PluginWindows` keeps the set of plugin window numbers whose latest frame holds `hover = true`, updated in `present` for both kinds and cleared by `forget_window`. A provider that never sets `hover` never receives `"move"`, so the frame field and the kind are both additions.

`PluginMouseKind` gains `Move`, written as `kind = "move"` with no `button`. In `Controls::mouse_motion`, a motion with no button held, under `Held::Free` or `Held::Ignored`, whose hit target is a floating plugin window or a tiled plugin window's drawn window that is in the hover set, calls `plugin_mouse` with `Move`. The existing same-cell filter in `Controls::mouse` already drops a motion to the cell of the previous event. The forward to the focused window's program stays as it is. `gband.win` sets `hover = win.hover` in both frames it presents, so a window without `hover` costs no Lua call per cell.

*Alternative:* a new optional provider function, `move(id, event)`, that the client calls for every move over any plugin window. Rejected: it runs Lua on every cell the pointer crosses over every plugin window, also those that never asked, and it adds a provider function where a frame field does.

`lua_event` keeps returning no built-in event for a motion with no button held. A `MouseMoved` event would run every handler on every cell the pointer crosses, for every plugin and for the whole screen, while only hover windows need it. It can be added later as a pure addition.

### A hover window's `on_mouse` holds it for every kind

In `hooks.mouse`, the hold before `on_mouse` becomes `win.kind == "floating" and (win.hover or event.kind ~= "scroll")`, and the spec documents the hold for a hover window only. A window that opts into hover asks to treat the pointer as input, as keys are input: a move, a wheel step and a click over it all drive what it shows, and any of them may change focus on its behalf, as a preview does. Holding it for every kind gives its author one rule, the one `keys` functions already follow, instead of a rule per kind. A window without `hover` keeps today's behaviour exactly: the undocumented hold for a press, a release and a drag, and none for a wheel step.

`hold` only holds a plugin window that is focused, so a wheel step or a move over an unfocused one holds nothing. A key press still releases the hold. A press on a window still unfocuses every floating plugin window first, through `unfocus_plugin_windows`, so a click elsewhere is never swallowed by a hold.

### `gband.api_version` stays 2

The rule in the lua-api capability raises the version only when a build removes a documented part or changes its documented arguments, return values, errors or effect. Every change here is an addition:

- The options of `gband.window.focus` add an optional argument. A call with one argument does what it does today. A second argument that is not a table, which today is ignored and undocumented, becomes an error; the rule keeps the version for a change to what the documentation does not describe.
- `last_focus`, `peek`, `hover` in `gband.win.info` and `hover` in a frame are new fields.
- The kind `"move"` and the wider focus hold reach only a plugin window opened with `hover = true`, an option that does not exist today, and only a provider whose frames set `hover`. Every `on_mouse` and every provider written for version 2 receives exactly the events it receives today.
- The peek view action is new. A focus without `peek` records, raises and restores as today.

### Documentation

- `docs/plugins.md`: the options of `gband.window.focus` and the peek, `last_focus` on window and floating tables with the stacking order rule, `peek` in `gband.view()`, the `hover` option and `hover` in `gband.win.info`, the `"move"` kind and when it runs, the focus that a hover window's `on_mouse` keeps, `hover` in the frame table, and `"move"` in the provider's `mouse` row. "API and stability" does not change. The desktop section belongs to `desktop-list-tweaks`.
- `README.md`: one paragraph in "Floating windows" on peeking. The window list's own text belongs to `desktop-list-tweaks`.
- `docs/tutorial/05-layout.md`: `last_focus` in "Reading the layout", and the peek and `gband.view().peek` in "Acting on windows and bands", in prose, so the chapter's Lua blocks stay in step with its example files.
- `docs/tutorial/07-plugin-windows.md`: a prose paragraph after "Keeping it current" on previewing a mark with `{ peek = true }` and on `hover = true` with the `"move"` kind, pointing to the reference. The chapter's list and its example files do not change.
- `docs/internals/05-plugin-windows.md`: the quotes of `COMMON`, the window record in `api.open`, the two `core.present_window` calls and `api.info` follow `win.lua`, the prose names `hover`, and `held` is described as the floating window whose own key, input or mouse code is running. `docs/internals/06-window-provider.md`: the quote of `hooks.mouse` follows `win.lua`, and its prose names `"move"` and the hold of a hover window for every kind.
- `docs/tutorial/04-events.md`, `docs/testing.md` and the other internals chapters name nothing that changes.

## Risks / Trade-offs

- [A peek of a tiled window moves the camera, and the camera does not return when the peek ends] → The camera then follows the restored focus by the policy, so the restored window is shown. In the floating key style, every window that runs a program floats, and a peek of a floating window moves the camera only as viewing its band would, because the floating layer's camera follows the remembered tiled window.
- [A peek into another band runs the band switch animation for every selection move between bands] → It is the animation any focus into another band runs. A user who finds it slow can turn `animations` off.
- [A hover window runs Lua for every cell the pointer crosses over it] → Only windows that ask for it, and only while the pointer is over them. The same-cell filter drops repeats.
- [A motion with no button held over a plugin window still reaches the focused window's program, when the program asks for every motion] → Unchanged from today and out of scope. A preview focuses the peeked window, so its program may see hover motion while the pointer is over the list.
- [A hold from a hover window's `on_mouse` lasts until the next key, so a server focus that arrives later keeps the list focused] → The same as for a `keys` function today, which the spec states. A click elsewhere unfocuses plugin windows explicitly.
- [A replaced windows provider that wants hover must set `hover` in its frames] → Documented with the frame table and the provider's `mouse` row. A provider that does not is unaffected.

## Migration Plan

Nothing persists and no wire message changes, so client and server builds need no coordination. No plugin needs a change: hover reaches only plugin windows that pass `hover = true`. Rollback is reverting the change.
