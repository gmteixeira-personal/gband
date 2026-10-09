## Context

The client routes input in `crates/client/src/lib.rs`. A key that `Leader::handle` returns as `Command::Send` and a paste that no plugin window takes become one `ClientMessage::Key` or `ClientMessage::Paste` naming the focused window, through `Display::key_to_focused` and `Controls::paste`. `send_prefix` and `SendKey` use the same `key_to_focused`. Each message already names its window, and the server encodes a key for the window it names, so the wire protocol can carry broadcast input unchanged.

The Lua side sees the client through `ViewState` in `crates/lua/src/ui.rs`. `core.state()` and `core.on_state` feed the sidebar, and `drawn_differs` decides when `on_state` runs. `gband.view()` in `crates/lua/src/control.rs` reads the same state.

A successful reload replaces `Controls` with `Self::new(config, display)` but keeps `Display`.

The modal preset binds no `m` today. The floating preset reaches every feature through the desktop plugin's window list, and the direct preset mirrors the modal one under "Preset parity".

## Goals / Non-Goals

**Goals:**
- One client-side flag that changes where unbound keys, pastes and the sent prefix key go, with no server or wire change.
- The same toggle reachable from all three key styles, and visible in the sidebar and in `gband.view()`.

**Non-Goals:**
- No new Lua event for multi mode. The sidebar redraws through `on_state`, and the window list closes when it toggles the state. A plugin that wants to react can poll `gband.view()` in its existing handlers.
- No choice of a subset of windows, no cross-band broadcast, no broadcast of mouse input.
- No border or colour change on the windows that receive the input.
- No persistence across a reload or a detach.

## Decisions

### The flag lives in `Display`, and a reload clears it

`Display` owns the view and builds `ViewState`, and `key_to_focused` and the paste path already read it, so a `multi: bool` there reaches every send path without threading through `Controls`. A successful `Controls::reload` sets it to false next to its other resets, so a reload behaves as the spec says, while the failed-reload branch leaves it alone, as it leaves the view alone.

Alternative: keep it in `Controls` beside the `Leader`. Rejected: `Controls` is rebuilt on reload, which would clear the flag implicitly. An explicit reset is clearer, and the send paths would still need it passed to `Display`.

### Fan-out replaces `key_to_focused` with a target list

A `Display` helper returns the windows that input goes to: the focused window when multi mode is off, and every window of the viewed band in layout order when it is on, read from `Layout`'s band, tiled columns first, then the floating list. Keys, pastes, `send_prefix` and `SendKey` all map that list to one message per window. A band with no window gives an empty list, so nothing is sent.

Alternative: a new wire message carrying a list of windows. Rejected: it needs a protocol bump, and per-window encoding already happens on the server for each `Key`.

### `toggle_multi` is a client action

`ClientAction::ToggleMulti` in `crates/core/src/action.rs`, registered as the built-in `toggle_multi` with the description `toggle multi mode` in `crates/lua/src/actions.rs`. The client handles it in `run_action` by flipping the flag and returning `Step::Nothing`. It is a client action, not a view action, because it changes no camera or focus.

### `ViewState` carries `multi`

`ViewState` gains `multi: bool`, set by `Display::view_state`. `drawn_differs` compares it, so the sidebar redraws when it flips. `core.state()` exposes `multi`, and `gband.view()` sets `multi = true` only while it is on, as it does for `peek`, so existing exact-table scenarios keep holding.

### Presets bind `m` beside `!`

The modal preset binds `prefix m` to a function that dispatches `toggle_multi` and then enters `root`, with the description `multi mode`, after `!` and before `D`. The direct preset binds `prefix m` to `action.toggle_multi` at the same position. This falls within "Preset parity": a modal binding that dispatches an action and returns to interactive mode matches a direct binding to that action. Leaving navigation mode by Escape or Enter does not touch the flag, so it returns to multi mode.

### The window list gets a third fixed entry

`desktop.lua` adds `Multi mode`, shortcut `m`, after `Settings`. Its label reads the flag through `gband.view().multi`. Picking it cancels the list, then dispatches `gband.action.toggle_multi()`. `m` joins `LIST_KEYS`, so a user's `prefix m` binding no longer runs from the list. The floating preset itself binds no `prefix m`, so the key list's `prefix keys` stay as they are.

### The sidebar shows `M` in `root`

`mode_letter` in `sidebar.lua` returns `M` for `root` while `state.multi` is set, after the floating style's early return, so the floating style keeps `∷`.

## Risks / Trade-offs

- [A user's `prefix m` binding in the floating style stops running from the window list] → The spec lists `m` among the list's own keys. The README and the keys tutorial chapter name it.
- [Broadcast to a band of many windows sends many messages per key] → Each message is small and the server already handles one per key. Bands rarely hold more than a few dozen windows.
- [Typing into a hidden or minimized window by surprise] → The sidebar's `M` shows the state in modal and direct styles, and the window list label shows it in the floating style.
- [Window entries move down one row, so every window list scenario and screenshot that counts `j` presses or rows shifts] → The delta spec renumbers each one, and the tasks regenerate the affected screenshots.

## Migration Plan

None. The flag is off at attach, and the wire protocol version stays as it is.
