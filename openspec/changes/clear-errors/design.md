## Context

See proposal.md for the motivation. This change starts from the code sidebars-borders-steps leaves:

- **The error list.** The client keeps the latest error and an ordered list of error texts in `crates/client/src/lib.rs`. A load with none of its own errors empties both. The list reaches Lua in the `ViewState` that `host.state()` reads, kept by `crates/lua/src/ui.rs`, and `gband.errors()` in `crates/lua/src/api.rs` copies it.
- **Redrawing the status line.** `ui::set_state` runs the `on_state` hook only when the pushed state's errors or `drawn` differ from the stored copy. The hook lays the status line out again, which drops or adds the error item and reports it through `host.error_item`.
- **The banner.** The client draws the latest error over the ribbon's bottom row when an error is reported and no error item is drawn.
- **Dispatches.** Client functions such as `gband.win.open` queue a `Dispatch` with `api::queue`. The client applies the queue after the callback returns, then reports the callback's errors, then pushes the state again. `api::outside_callback` raises the error for a call made while loading.
- **The plugin.** `crates/lua/src/runtime/gband/errors.lua` opens the error list from a snapshot of `gband.errors()`, wraps it again in `on_resize`, and binds `q` in `keys`. The plugin window defaults are Up and `k`, Down and `j`, PageUp, PageDown, Home, End, and `q` and Escape for a floating window.
- **The client-only fields.** The list is `CLIENT_ONLY` in `crates/lua/src/sides.rs`, where sidebars-borders-steps adds `bar` and `errors`.

## Goals / Non-Goals

**Goals:**
- One function clears the errors. The plugin's key, action and command all go through it.
- Clearing reuses the paths a good load already takes: the error item and the banner disappear because the list and the latest error are empty.
- `gband.errors()` agrees with the clear inside the callback that made it.

**Non-Goals:**
- Live refresh of an open error list after a clear made elsewhere, or after a new error.
- A protocol message. The server never learns that a client cleared its errors.

## Decisions

### The function is `gband.clear_errors()`
`gband.errors` is a function in sidebars-borders-steps, so `gband.errors.clear` would need `errors` to become a callable table. That changes the dependency's API shape for one entry. A verb first matches the other top-level functions: `set`, `bind`, `spawn`, `notify`, `open`. The plugin's action and command follow the plugin naming, `errors.clear`, beside `errors.open`.

Alternative: `gband.errors_clear()`. Rejected because it reads as a noun and matches no other name.

### Clearing is a dispatch, and the Lua copy empties at once
`gband.clear_errors()` checks `in_callback`, then:
1. empties `error` and `errors` in the stored `ViewState`, so `gband.errors()` returns an empty list in the same callback;
2. marks the stored state changed, so the next `ui::set_state` runs `on_state` even though the pushed state equals the emptied copy;
3. queues `Dispatch::ClearErrors`.

The client applies `Dispatch::ClearErrors` by emptying its list and the latest error. It does this in order with the callback's other dispatches. The push of the state that follows runs `on_state`, so the status line drops its error item. The banner has no error left to draw.

Alternative: clear the client's list only, after the callback. Rejected because `gband.errors()` would still return the old errors in the same callback, which a Lua prompt user would read as a failed clear.

Alternative: run `on_state` from inside `gband.clear_errors()`. Rejected because the status line layout would then run inside the caller's callback, on the caller's instruction budget, and once more for every call in a loop. The flag lets the existing push run the hook once.

### Errors raised in the clearing callback survive the clear
The client reports a callback's errors after it applies the callback's dispatches. An error raised in the same callback, before or after the call, is therefore listed after the clear. The configuration spec states this so the order is a contract, not an accident. The alternative, recording the clear's position among the errors, needs a per-error sequence across Lua and the client for a case that only matters when a callback both clears and fails.

### Clearing is local to the client
Each client keeps its own list, so clearing touches no other client and sends nothing to the server. The server still holds its latest error and sends it to each client that attaches, as server-runtime defines. A server error therefore returns on a new attach until the server configuration loads with no error. This matches a client reload, which also leaves server errors with the server.

### Clearing dismisses the banner
The banner and the error item show the same latest error in two places, so emptying it removes both. No client-attach delta is needed. "Present the ribbon" draws the banner while the configuration capability shows an error, and the configuration delta says the client shows none after a clear.

### The plugin clears through one local function
`errors.lua` gains a local `clear()`:
- it calls `gband.clear_errors()`;
- when the error list is open, it empties the held texts, sets the lines to `no errors`, and sets the title of a floating window to `errors`.

`keys.c`, the action `errors.clear` and the command `errors.clear` all call it. Emptying the held texts keeps `on_resize` from wrapping the old errors back in.

### `c` and its hint
`c` clashes with no plugin window default and with no other `keys` entry of the error list. Ctrl+Space then `c` is still center column, because bindings win over a plugin window's keys. `c` is not a direction, so no arrow pair is needed.

The floating window's title shows `errors  c clear` while it holds errors. This is the hints segment's form: the key, one space, the label, and two spaces before it. The hint goes once the list is empty, because there is nothing left to clear. A tiled plugin window has no title, so the README documents `c` for both kinds. The key list shows no hint for Enter or `q`, but those are conventions, and `c` is not.

Alternative: a hint line inside the content area. Rejected because it would shift the wrapped errors and the `no errors` line that sidebars-borders-steps specifies.

## Risks / Trade-offs

- [A clear hides an error the user has not read] → `c` acts only in the focused error list, where the errors are shown. `errors.clear` has no default binding. The logs keep every error.
- [A disabled component stays hidden after a clear, with no error left to explain it] → The spec says so, `gband.ui.statusline.list()` still gives `enabled` false, and the README says a reload revives it.
- [A clear made by calling `gband.clear_errors()` directly leaves an open error list stale] → It matches how a new error leaves it, per sidebars-borders-steps' non-goal. The plugin's own key, action and command refresh it.
- [sidebars-borders-steps changes the error list's shape while it is implemented] → This change depends on it, so it starts from the merged code. Task 0.1 folds any change into these deltas.

## Migration Plan

Nothing to migrate. The change adds a function, an action, a command and a key. No option or binding changes, and the protocol version stays.
