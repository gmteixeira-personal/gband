## Why

The camera only moves on its own, by the `center_focused_column` policy. Under the default `"never"` policy a focused column often sits at the view's edge, and there is no key to bring it to the middle. niri binds `center-column` to Mod+C for this, and gband follows niri where its specs are silent, so it should gain the same action on Ctrl+Space then `c`.

## What Changes

- **Centre the focused column**: a new view action, `center_column`, moves the viewed band's camera so that the focused column sits in the middle of the client's terminal, as niri's `center-column` does. It uses the same position as the `"always"` policy: the column's start less half the difference between the terminal's width and the column's width, rounded down, or the column's start when the column is at least as wide as the terminal. It changes neither focus nor the layout, sends nothing to the server, and does nothing on an empty band.
- **Floating focus**: while the floating layer is active, the action centres the focused floating window in the session's screen area instead, as niri's `center-column` does for a floating window. The client sends the server the placing of that window at the centred column and row, the same position a newly floated box takes, and sends nothing when the box already sits there. The camera does not move.
- **The camera stays where it was put**: later changes move the camera by the configured policy, as before. Under `"never"` and `"on-overflow"`, a centred column that stays inside the view keeps the camera centred.
- **Animation**: the camera scroll animates, as any other camera change does.
- **Default key**: Ctrl+Space then `c`, Lua binding `prefix c`.
- **Lua**: `gband.action.center_column`, with the description `center the focused column`, listed by `gband.action.list()`. It is a client action name, so the server's `gband.action` reports it as belonging to the client.
- **Key hints**: the short label `center`.

Out of scope:
- niri's `center-visible-columns` and `center-window`.
- Centring a column other than the focused one through a Lua target. View actions take no target.
- Any change to the `center_focused_column` policies.

## Capabilities

### New Capabilities

None.

### Modified Capabilities
- `layout-view`: a new requirement for centring the focused column, or the focused floating window, on demand. "Per-client view" names it as the one view action that changes the layout.
- `actions`: the Lua names table gains `center_column`, and the kinds of action name it as the one view action that sends a message.
- `client-attach`: the default prefix table gains `c`, and the view-action rule names the floating exception.
- `key-hints`: the short label table gains `center_column`.

## Impact

- `crates/core/src/view.rs`: `ViewAction::CenterColumn` and its camera move.
- `crates/lua/src/actions.rs`: the built-in action entry.
- `crates/lua/src/defaults.lua`: the `prefix c` binding.
- `crates/lua/src/runtime/gband/statusline/hints.lua`: the short label.
- `crates/client/src/bindings.rs`: the default key test.
- Tests in `crates/core/tests/view.rs`, `crates/lua/tests/config.rs`, `crates/lua/tests/key_hints.rs` and `crates/client/tests/actions.rs`.
- `README.md` and `docs/plugins.md`: the action and label tables.
- `crates/client/src/lib.rs`: `dispatch` sends the centred placing on the floating layer.
- No protocol change: the floating case reuses the existing set-position session action.

## Coordination

### Author
- gmteixeira

### Depends On
- rename-panes-to-windows

### Expected Files
- crates/core/src/view.rs
- crates/core/tests/view.rs
- crates/lua/src/actions.rs
- crates/lua/src/defaults.lua
- crates/lua/src/runtime/gband/statusline/hints.lua
- crates/lua/tests/config.rs
- crates/lua/tests/key_hints.rs
- crates/lua/tests/plugins.rs
- crates/client/src/bindings.rs
- crates/client/src/lib.rs
- crates/client/tests/actions.rs
- tests/attach.rs
- README.md
- docs/plugins.md
- openspec/changes/center-column/
