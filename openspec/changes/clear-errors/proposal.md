## Why

An error stays on screen until the configuration loads with none of the client's own. A user who has read an error, or whose error comes from a plugin or from the server's Lua that cannot be fixed at once, has no way to dismiss it. The error list that sidebars-borders-steps adds is where the user reads errors, so it is the natural place to clear them.

## What Changes

- **Clear the errors by hand**: a new client function, `gband.clear_errors()`, empties the client's error list and drops the shown error.
  - `gband.errors()` returns an empty list right after the call, in the same callback.
  - Once the callback returns, the status line's error item disappears. Without a status line, the banner disappears.
  - It is callable wherever an action value is, so bindings, commands, event handlers and a later Lua prompt can call it. Calling it while the configuration loads is an error.
  - An error reported after the clear is listed and shown as before.
- **What clearing does not do**:
  - It does not reload the configuration.
  - It does not enable a status line component that an error disabled, and it does not clear a plugin's failed mark. Both still wait for the next load.
  - It does not remove anything from the logs.
  - It clears only this client's errors. A server error that this client shows is removed from this client only. Other clients keep their errors, and the server still sends its latest error to each client that attaches.
- **Error list plugin**: `gband.errors` registers the action and the command `errors.clear`, beside `errors.open`. Both clear the errors. An open error list then shows `no errors`.
- **`c` in the error list**: `c` in the error list's plugin window clears the errors, in either kind. The plugin window then shows `no errors`, and keeps showing it when it is wrapped again after a resize.
- **Hint**: while a floating error list holds errors, its title is `errors  c clear`, the key and its label in the hints segment's form. Once it shows `no errors`, the title is `errors`.

Out of scope:
- A default key binding for `errors.clear`, for the same reason sidebars-borders-steps gives for `errors.open`.
- Live refresh of an open error list. A clear made by calling `gband.clear_errors()` directly leaves an open error list unchanged until it is closed and opened again, as a newly reported error does.
- Clearing one error at a time, and an event that reports a change of the error list.
- Clearing the errors of another client or of the server.

## Capabilities

### New Capabilities

None.

### Modified Capabilities
- `configuration`: `gband.clear_errors()` empties the client's error list and the shown error, without reloading and without reviving disabled components or failed plugins.
- `error-list`: the `gband.errors` plugin registers `errors.clear`, `c` clears the errors from the error list's plugin window, and a floating error list's title shows the `c` hint while it holds errors. sidebars-borders-steps adds this capability.
- `status-line`: the error item disappears when the errors are cleared by hand.
- `plugins`: `clear_errors` is a client-only field of `gband`.

## Impact

- Lua host (Rust): `gband.clear_errors()` and a dispatch that tells the client to clear, in `crates/lua/src/api.rs`. The Lua copy of the error list is emptied in `crates/lua/src/ui.rs`. `clear_errors` joins the client-only fields in `crates/lua/src/sides.rs`.
- Client: `crates/client/src/lib.rs` empties its error list and the shown error when the dispatch arrives.
- Lua runtime: `crates/lua/src/runtime/gband/errors.lua` gains `errors.clear`, the `c` key and the title hint.
- Docs: `README.md` and `docs/plugins.md`.
- Tests: `crates/lua/tests/`, `crates/client/tests/statusline.rs`, a two-client test in `tests/errors.rs`, and a screen test in `tests/lua/`.
- No protocol change and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- sidebars-borders-steps

### Expected Files
- openspec/changes/clear-errors/
- crates/lua/src/api.rs
- crates/lua/src/sides.rs
- crates/lua/src/ui.rs
- crates/lua/src/runtime/gband/errors.lua
- crates/lua/tests/config.rs
- crates/lua/tests/errors.rs
- crates/lua/tests/plugins.rs
- crates/client/src/lib.rs
- crates/client/tests/statusline.rs
- tests/errors.rs
- tests/lua_specs.rs
- tests/lua/errors_spec.lua
- tests/lua/screenshots/errors_spec/
- README.md
- docs/plugins.md
