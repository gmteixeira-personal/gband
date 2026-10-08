## Context

See proposal.md, "Why". The relevant state today:

- `View` in `crates/core/src/view.rs` is each client's own state. It holds a `BandView { layer, tiled, floating, camera, .. }` per band and a `recency` map of focused windows. The stacking order is not stored: `View::stacked` derives it from `recency` and the floating list. `View::shown` gives the shown windows, and `View::stacking` gives the floating windows to draw.
- The client draws floating windows, and builds the regions that mouse targets resolve against, from one loop over `view.stacking(scene)` in `crates/client/src/render.rs` (`layers`). Drawing, pointer targets and the "covered by a floating window" cursor check all follow that order.
- View actions never reach the server or the wire protocol. `ViewAction` is serialized nowhere outside the client and the Lua crate.
- Lua reads the client's state through `ui::ViewState` (`crates/lua/src/ui.rs`), which `Display::view_state` in `crates/client/src/lib.rs` fills. `gband.layout()` and `gband.window.*` live in `crates/lua/src/control.rs`; the built-in action table is `ACTIONS` in `crates/lua/src/actions.rs`.
- Every built-in action today has a default binding in both key style presets.

## Goals / Non-Goals

**Goals:**
- Hide a floating window from one client without touching the shared layout, the server or other clients.
- Keep one source of truth for what a client draws, what its mouse hits and what it reports as shown: the stacking order and `View::shown`.
- Give Lua what a window list needs: a way to minimize, a way to restore, and a way to read which windows are minimized.

**Non-Goals:**
- Minimizing tiled windows.
- Keeping the minimized set across a detach and a later attach.
- A built-in event for minimize and restore.
- Default key bindings in the modal and direct key styles.

## Decisions

### The minimized set is per-client view state, not a layout flag

`View` gains `minimized: BTreeSet<WindowId>`, beside `recency`. Minimizing changes only this set, so it needs no server action, no layout field and no protocol version.

- Another client may use a key style with no window list, such as modal or direct. A shared flag would hide the window on that client's screen too, with no built-in way to get it back there.
- Stacking is already per client, and minimizing is the same kind of choice: how this client presents the floating layer. Two people attached to one session can each keep their own desktop.
- The cost is that the set does not survive a detach and a later attach. No other part of the view survives either: focus, layers, cameras and stacking all start again on attach. A reload keeps the set, because the reload keeps the `Display` and its `View`.

*Alternative:* a `minimized` flag on `FloatingWindow` in the layout. Rejected for the reasons above, and because it changes the layout's serialized shape and so the protocol version.

*Alternative:* move the box off screen. Rejected: `set_position` and box geometry keep every box inside the screen area, and moving the box changes the shared layout.

### Only floating windows

A minimized tiled window would leave a hole in the strip that only one client sees, or need a per-client layout of the strip. The floating layer already draws boxes over the strip, so hiding a box changes nothing else. The `minimize_window` action on a tiled window does nothing, as consume or expel does on a floating window. `gband.window.minimize` on a tiled window is an error at the line of the call, as `gband.window.set_position` is, because a script that names a window by number expects it to be floating.

### One predicate for every focus rule

`View` gains a private `available(band, window)`: the band holds the window and the set does not. Every place that today asks `band.holds`, or picks a floating window, uses it:

- `settle`: keeps the focused window only when available, so minimizing the focused window falls through to the existing floating fallback, `top_floating`, then to `tiled_target`, then to `clear_focus`. It also prunes the set to windows still in a floating list, so tiling a window or closing it removes it, and drops a remembered floating focus that is no longer available.
- `enter` and `floating_target`: skip a minimized remembered window.
- `stacked`: leaves minimized windows out, so `top_floating`, `stacking`, drawing, regions and pointer targets all skip them.
- `neighbour_box`: leaves minimized candidates out.
- `shown`: leaves minimized boxes out.

`top_floating` is the top of the stacking order, which is the most recently focused available floating window, or the last available window of the floating list when none was focused. That is exactly the fallback the floating-windows delta states, so the existing code path needs no new rule.

`ViewAction` gains `Minimize(Option<WindowId>)`. `None` is the `minimize_window` built-in and names the focused window. `Some(window)` comes from `gband.window.minimize`. `View::apply` handles it before the band lookup, as it handles `FocusWindow`: it does nothing unless the window is floating in the layout and not yet in the set, then inserts it and settles with the previous focus, so the camera and `FocusChanged` follow as for any focus change.

`View::focus_window` removes the window from the set before focusing it. Every way to focus a window by number goes through it: `gband.window.focus`, the server's focus message and focusing a tiled plugin window's drawn window. Focusing touches `recency`, so the restored window lands on top of the stacking order without new code.

*Alternative:* a separate `restore` view action and `gband.window.restore(window)`. Rejected for now: restoring without focus would draw a window on top that has no focus, while stacking puts the focused window on top. The window list in `floating-key-style` restores by focusing. A restore function can be added later as a pure addition.

### Minimized windows are not shown windows

The first sketch of this change kept minimizing entirely off the server. This design departs from that in one place: a minimized window leaves this client's shown windows, so the client sends the server a new shown message, as it does whenever its shown windows change.

- The shown windows are defined as the windows this client draws. Keeping one rule, "drawn means shown", keeps `View::shown` and the drawing loop in step.
- The session-server capability keeps the PTY size of a window no client shows. So minimizing changes no PTY size, which is what the brief asked for, and a hidden program gets no SIGWINCH while the screen area changes, as a column scrolled off screen does.
- On restore, the window is shown again and the server resizes it once the session settles, as it does for a column scrolled back into view.

The shown message is per-client presentation data the server already receives on every camera move. It changes no layout and no other client.

*Alternative:* keep a minimized window among the shown windows, so the client sends nothing at all. Rejected: it splits the definition of shown windows from what is drawn, and resizes programs nobody can see.

### Reading the state from `gband.layout()`

Each table in a band's `floating` list holds `minimized = true` when this client has minimized that window, and no `minimized` otherwise. `ui::ViewState` gains `minimized: BTreeSet<WindowId>`, filled by `Display::view_state` from the view. `control.rs` sets the field while it builds the floating tables.

- A window list walks `gband.layout()` already, to read every window's `id` and `name`. A flag on the same entry needs no second lookup.
- The layout table already carries a per-client field, `plugin_window`, so a per-client `minimized` fits it.
- The field is absent rather than `false`, so every existing floating table, and the "Floating window" scenario of "Read the layout", stays exactly as it is.

`minimized` does not join `drawn_differs` in `ui.rs`, so `gband.core.on_state` keeps running only for the changes it documents.

*Alternative:* a `minimized` list in `gband.view()`. Rejected: `gband.view()` describes focus and the viewed band, and the list would repeat window numbers that the layout already holds.

### Lua surface

- `ACTIONS` gains `minimize_window`, `Action::View(ViewAction::Minimize(None))`, after `switch_focus_floating_tiled`, with the description `minimize the focused floating window`. It is a view action, so the existing rule makes a target on it an error, and the server's `gband.action` does not hold it.
- `gband.window.minimize(window)` in `control.rs` checks the window as `set_position` does, with the message `window N is not a floating window` for a tiled window, and queues `Dispatch::Action(Action::View(ViewAction::Minimize(Some(window))))`, so it follows the dispatch order of every other `gband.window` function.
- No default binding. The key-style presets belong to another change, and the modal and direct styles have no window list, so a key that minimizes would hide windows with no built-in way back. `floating-key-style` binds the action and draws the list.

### Gestures and selections

`Display::end_gesture_for_layout` in `crates/client/src/lib.rs` ends a gesture, and drops a selection, whose window has left the layout. It also checks the view's minimized set, and the client calls it after a view action as well as after a layout. A gesture whose window is minimized then ends with no further change, and its selection is no longer drawn.

### API version and events

`gband.api_version` does not rise. The change adds one action, one function and one optional field. No documented function, field or event changes its arguments, return values, errors or effect for any state that existed before: focus restores only a window that only the new function or action can minimize. Another open change, `mouse-binding-fallthrough`, raises the version for its own reason. This change neither raises it nor tests its value.

No built-in event is added. `FocusChanged` already fires when minimizing moves focus and when focusing restores a window. Minimizing an unfocused window fires no event; see Open Questions.

### Documentation

`README.md`, `docs/plugins.md` and `docs/tutorial/05-layout.md` describe the new action, function and field. `docs/tutorial/02-actions.md` links to the action reference and lists no built-in action, so it needs no change. The internals tutorial, `docs/internals/`, reads gband's bundled Lua, and this change adds no bundled Lua and changes none, so it needs no change either.

## Risks / Trade-offs

- [A window minimized from Lua in the modal or direct style has no built-in way back] → `gband.window.focus` restores it, `gband.layout()` marks it, and a detach and attach clears the set. No default key minimizes.
- [Another client moves or resizes a window this client has minimized] → the box changes as usual; this client draws the new box when it restores the window.
- [A restored window shows a stale grid size for one settle period after the screen area changed] → the same as a column scrolled back into view; the server resizes it once the session settles.
- [A band can hold windows but give no focus, when all its windows are minimized floating windows] → session actions that need a window are dropped, as on an empty band; `open_window` still works, and focusing any of them restores it.
- [A window list cannot learn that an unfocused window was minimized by other code] → `floating-key-style` minimizes through its own code, so it knows. An event can be added later.
- [Merge overlap with open changes that edit `crates/client/src/lib.rs`, `crates/lua/src/control.rs` and `docs/plugins.md`] → the edits here are local: one field in `view_state`, one check in `end_gesture_for_layout`, one function and one field in `control.rs`, and new rows and bullets in the docs.

## Migration Plan

Nothing persists and nothing crosses the wire, so client and server builds need no coordination. Rollback is reverting the change.

## Open Questions

- Should the built-in event list gain `WindowMinimized` and `WindowRestored`, with `window` and `band`, so a live window list can redraw when other code minimizes or restores a window? The built-in event list belongs to another change, and `floating-key-style` does not need it to start.
