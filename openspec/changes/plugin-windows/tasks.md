## 1. Reconcile with the status line

- [ ] 1.1 Diff the archived `status-line` specs for `client-attach` ("Send input", "Present the ribbon", "Key bindings") and its delivered `host` primitives against this change's deltas and `design.md`. Carry any wording or name changes into them. Verify with `openspec validate plugin-windows --strict`.

## 2. Core layout and view

- [ ] 2.1 Add `SessionAction::SetWidth` and `SetHeight`, with `Column::set_width` and `Column::set_height` sharing `step_height`'s weight normalisation and limits. Verify with tests in `crates/core/tests/layout.rs` for every scenario of the layout delta's "Column widths" setter and "Set a pane's height".
- [ ] 2.2 Reshape `OpenPane` into `{ band, after, width, focus, content: PaneContent }`. Make `Layout::open` honor the width, and update every caller to send `focus: true` with `PaneContent::Program`. Verify with the "Width named on open" test, and confirm that `cargo test` passes unchanged elsewhere.
- [ ] 2.3 Add `ViewAction::FocusPane` and `ViewAction::ViewBand`, and `View::view_band` sharing the band-switch code. Verify with tests in `crates/core/tests/view.rs` for the layout-view delta's scenarios.

## 3. Protocol

- [ ] 3.1 Add `ClientMessage::Content`, `ServerMessage::Opened`, the new action fields, and `PROTOCOL_VERSION` 5. Verify with round-trip tests in `crates/protocol/tests/` for the open plugin pane, set height, content and opened scenarios, and the version 4 against version 5 handshake test.

## 4. Server plugin panes

- [ ] 4.1 Add `Pane::blank`, a grid-only `resize`, and `replace(output)`. Verify with unit tests: a replaced screen yields an update from an earlier checkpoint, and a resize keeps the cells that fit.
- [ ] 4.2 Add the plugin pane map, `Command::Action { client, .. }`, the `Reply` sender with `Focus` and `Opened`, the focus flag, and the opened-with-no-pane answer. Verify with `crates/server/tests/plugin_panes.rs` covering "Open a plugin pane", "Ignored request" and "Open without focus".
- [ ] 4.3 Handle `Command::Content` with the ownership checks. Verify with tests for "Content reaches every client", "Content replaces the screen" and "Content from another client".
- [ ] 4.4 Drop input to plugin panes, close them at once, send `Command::Leave` from `Attachment::drop`, end the session at its last program pane, and apply set width and set height. Verify with tests for "Key to a plugin pane", "Close a plugin pane", "Owner leaves", "Only a plugin pane remains" and "Set a column's width".
- [ ] 4.5 Resize plugin panes with the settle rule and send snapshots after each resize. Verify with the "Resized like a PTY" test.

## 5. Lua control API

- [ ] 5.1 Extend the state the client pushes with the layout, the screen area, the view, the ribbon size and the pane-to-window map. Add `control.rs` with `gband.layout()` and `gband.view()`, both refused while loading. Verify with `crates/lua/tests/control.rs` covering "Read the layout", "Read the view" and "State as of the call".
- [ ] 5.2 Accept target tables in `LuaAction`'s `__call` and queue `Dispatch::Session`, with errors for bad targets. Verify with tests for every "Action targets" scenario and the actions delta's target scenarios.
- [ ] 5.3 Add `gband.pane.focus`, `set_width`, `set_height`, `send_keys`, `send_text` and `paste`, `gband.band.view`, and `Dispatch::Input`. Verify with tests for "Focus and view by number", "Exact sizes", "Input to a named pane" and "Dispatch order".
- [ ] 5.4 Accept `band` and `after` in `gband.spawn`. Verify with the configuration delta's "Spawn after a named pane" and "Unknown field" tests.

## 6. Window store in Lua

- [ ] 6.1 Add the host primitives `next_window`, `present_window`, `forget_window`, `request`, `parse_key` and `window_hooks`, and make `after_event` a list. Verify with unit tests in `crates/lua` that the status line still renders after events.
- [ ] 6.2 Add the `win.lua` API chunk with `open` option validation, the store, `set_lines`, and line and span normalisation. Verify with `crates/lua/tests/windows.rs` covering "Window API", "Open a window" and "Window contents".
- [ ] 6.3 Add scrolling, the cursor line, the `keys` table and the default keys in the `key` hook. Verify with tests for "Scrolling and the cursor line" and "Keys in a focused window", driving the hook directly.
- [ ] 6.4 Add float placement against the ribbon size, `set_config`, `on_resize`, `info` and `list`. Verify with tests for "Floats" and "Window information" that read the presented frames.
- [ ] 6.5 Add pane windows: the open request, the `opened`, `pane_resized` and `pane_closed` hooks, and `close`. Verify with tests for "Plugin pane windows" and "Closing windows" that feed the hooks.
- [ ] 6.6 Add the focused-float rules on `FocusChanged` and `BandChanged`, closing on a failed plugin, the window groups' defaults, and redraws on highlight changes. Verify with tests for "Focused window", "Window callbacks" and "Window highlight groups".

## 7. Client

- [ ] 7.1 Add `crates/client/src/windows.rs` with the window counter, stacked floats, focused float, pending requests and pane map, and the content encoder. Verify with a unit test that feeds an encoded styled frame into an emulator grid and checks the cells, styles and hidden cursor.
- [ ] 7.2 Apply `Dispatch::Session`, `Dispatch::Input`, `Dispatch::Window`, `FocusPane` and `ViewBand` in `Controls::apply` and `dispatch`. Verify with tests in `crates/client/tests/actions.rs` for the messages each sends.
- [ ] 7.3 Route unconsumed keys and pastes to the focused window. Verify with `crates/client/tests/windows.rs` covering the client-attach delta's key and paste scenarios.
- [ ] 7.4 Handle `ServerMessage::Opened`, snapshots of owned plugin panes, and layouts without them. Send coalesced content after each runtime run, close everything on reload, and answer orphaned requests with close pane. Verify with tests for the reload scenario and an orphaned opened reply.
- [ ] 7.5 Draw floats in `render.rs` after the tiles and before the banner, and hide the cursor while a float is focused. Verify with snapshot tests in `crates/client/tests/render.rs` for "Float over two tiles", "Error banner over a float" and "Centered float".

## 8. Integration and documentation

- [ ] 8.1 Add `tests/windows.rs` with end-to-end runs: a pane window seen by two clients, a pane window closed with Ctrl+Space then `q`, a focused float taking keys, `send_text` running a command in another pane, and the owner detaching. Verify with `cargo test --test windows`.
- [ ] 8.2 Document `gband.layout`, `gband.view`, action targets, `gband.pane`, `gband.band`, `gband.win`, the window groups and the color note for plugin panes in `docs/plugins.md`. Verify that each example in the guide runs in a test configuration.
- [ ] 8.3 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`, and confirm all pass.
