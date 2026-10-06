## 1. Core types and encoding

- [ ] 1.1 Add `MouseKey`, its triggers, and a `MouseEvent` (kind, button or direction, content cell, modifiers) to `crates/core/src/input.rs`, and verify `cargo test -p gband-core` still passes
- [ ] 1.2 Add `mouse_tracking` and `mouse_encoding` to `Modes`, and `encode_mouse(event, modes)` for modes 9, 1000, 1002 and 1003 in the default, UTF-8 and SGR encodings, and verify new cases in `crates/core/tests/input_encoding.rs` cover every input-encoding "Mouse reports" scenario
- [ ] 1.3 Fill the mouse modes from the vt100 screen in `crates/emulator/src/lib.rs`, and add `text_between` to the `Emulator` trait, and verify `crates/emulator/tests/contract.rs` covers the terminal-emulator "Mouse modes" scenarios, a snapshot round trip, wrapped-row joining and trailing-space trimming

## 2. Layout and protocol

- [ ] 2.1 Add `SessionAction::MoveToPlace` and apply it in `Layout::apply` (`crates/core/src/layout.rs`), and verify `crates/core/tests/layout.rs` covers every layout "Move a window to a place" scenario
- [ ] 2.2 Add `ClientMessage::Mouse` to `crates/protocol/src/message.rs` and raise `PROTOCOL_VERSION` to one above `dev`'s value, and verify the mouse and move-to-place round trips in `crates/protocol/tests/messages.rs`
- [ ] 2.3 Write mouse messages to the PTY as `Input::Mouse` in `crates/server/src/window.rs` and `connection.rs`, dropping them for drawn windows and missing windows, and verify the session-server "Mouse input to the window" scenarios in `crates/server/tests/windows.rs`
- [ ] 2.4 Apply move to place in `crates/server/src/session.rs`, and verify in `crates/server/tests/sync.rs` that two clients receive the moved layout

## 3. Lua: names, actions, events, plugin windows

- [ ] 3.1 Parse mouse names in `crates/lua/src/keys.rs` and add `Chord::Mouse` in `crates/lua/src/api.rs`, rejecting them for the `prefix` option and plugin window `keys`, and verify the configuration "Key names" scenarios in `crates/lua/tests/keymap.rs` and `crates/lua/tests/config.rs`
- [ ] 3.2 Add `drag_window`, `drag_resize_window` and `drag_band` to `crates/lua/src/actions.rs` and `ClientAction` in `crates/core/src/action.rs`, and verify `gband.action.list()` in `crates/lua/tests/config.rs` holds exactly the actions "Lua names of actions" lists
- [ ] 3.3 Add `MousePressed`, `MouseReleased`, `MouseDragged` and `MouseScrolled` with their payloads to `crates/lua/src/events.rs`, and pass the payload to a binding function bound to a mouse name in `crates/lua/src/runtime.rs`, and verify in `crates/lua/tests/events.rs`
- [ ] 3.4 Add `on_mouse`, `hooks.mouse` and the mouse defaults to `crates/lua/src/runtime/gband/win.lua`, plus a host entry in `crates/lua/src/plugin_windows.rs` that sets a floating plugin window's box in one step, and verify the plugin-windows "Mouse in plugin windows" scenarios in `crates/lua/tests/plugin_windows.rs`
- [ ] 3.5 Add `is_mouse` to `crates/lua/src/runtime/gband/keyform.lua` and skip mouse bindings in `statusline/hints.lua` and `keylist.lua`, and verify in `crates/lua/tests/key_hints.rs` and `tests/lua/keylist_spec.lua`
- [ ] 3.6 Add the navigation mode mouse bindings to `crates/lua/src/defaults.lua`, and verify `gband.keymap.list("prefix")` in `crates/lua/tests/config.rs` ends with the client-attach "Default mouse bindings" rows in order

## 4. Client: intake, targets, bindings

- [ ] 4.1 Write the mouse capture modes on entering and their resets on leaving in `TerminalGuard` (`crates/client/src/lib.rs`), and verify the mouse "Mouse capture" scenarios in `tests/attach.rs`
- [ ] 4.2 Convert crossterm mouse events in `crates/client/src/input.rs`, dropping same-cell motion and unknown buttons, and verify the input-encoding "Mouse events from the terminal" scenarios in `crates/client/tests/key_translation.rs`
- [ ] 4.3 Extract `regions()` from `render()` in `crates/client/src/render.rs`, keep the last regions in `Display`, and add the hit test in the new `crates/client/src/mouse.rs`, and verify the `crates/client/tests/render.rs` snapshots are unchanged and new cases cover every "Pointer targets" scenario
- [ ] 4.4 Add `Leader::handle_mouse` to `crates/client/src/bindings.rs` with the root, mode and sequence rules, and verify the "Mouse names in key tables" scenarios in `crates/client/tests/actions.rs`
- [ ] 4.5 Route `Event::Mouse` through `Controls` to bindings, defaults, gestures and the Lua mouse events, and verify `MousePressed` payloads in `crates/client/tests/events.rs`

## 5. Client: interactive defaults

- [ ] 5.1 Click to focus windows, floating windows and plugin windows, and verify "Click to focus" and "Click a floating window" in `tests/mouse.rs`
- [ ] 5.2 Forward presses, drags, releases, motion and wheel to the grabbing or hovered window by its grid's mouse mode, with Alt as the override, and verify "Forwarded click", "Wheel over an unfocused mouse program", "Drag outside the window" and "Alt selects in a mouse program" in `tests/mouse.rs`
- [ ] 5.3 Track, draw and clear the selection, copy it to the copy buffer and with OSC 52, and verify the "Selection and copy" scenarios in `tests/mouse.rs` and a selection screenshot in `tests/lua/mouse_spec.lua`
- [ ] 5.4 Paste the copy buffer on a right press, keeping the buffer across reloads, and verify "Right click pastes" and "Buffer survives a reload" in `tests/mouse.rs`

## 6. Client: gestures

- [ ] 6.1 Add the current-press slot, the gesture state machine, and the per-frame flush of pending sends, and verify "Drag action from a key" and "Drag action from a binding function" in `crates/client/tests/actions.rs`
- [ ] 6.2 Move floating windows and floating plugin windows by dragging, and verify the "Move a floating window by dragging" scenarios in `crates/client/tests/actions.rs` and `tests/mouse.rs`
- [ ] 6.3 Lift, draw and drop tiled windows with the drop place from drawn regions, and verify the "Move a tiled window by dragging" scenarios in `tests/mouse.rs` and a lifted-tile screenshot with its drop outline in `tests/lua/mouse_spec.lua`
- [ ] 6.4 Resize tiles, floating windows and floating plugin windows from the picked edges, with the camera override for the left edge, and verify every "Resize by dragging" scenario in `crates/client/tests/actions.rs`
- [ ] 6.5 Slide the band with the camera override and the release rule, and verify the "Slide the band by dragging" scenarios in `crates/client/tests/actions.rs`
- [ ] 6.6 Draw gesture positions without animation and animate the drop and the slide's settling camera in `crates/client/src/animation.rs`, and verify the animations "Drawing during a gesture" scenarios in `crates/client/tests/animation.rs`

## 7. Test harness

- [ ] 7.1 Add `g.mouse` to `crates/harness/src/runner.rs` and `case.rs`, writing SGR reports, and verify the plugin-testing "Click a window" and "Invalid mouse kind" scenarios in `tests/plugin_testing.rs`
- [ ] 7.2 Add `tests/lua/mouse_spec.lua` to `tests/lua_specs.rs`, and verify `cargo test --test lua_specs` passes with its screenshots under `tests/lua/screenshots/mouse_spec/`

## 8. Docs and integration

- [ ] 8.1 Document the mouse in `README.md` and `on_mouse`, the mouse events, mouse names and drag actions in `docs/plugins.md`, including Shift for native selection and OSC 52 terminal settings, and verify every new Lua name appears in `docs/plugins.md`
- [ ] 8.2 Before `/ready`, rebuild the wire-protocol "Client messages" and "Handshake" MODIFIED blocks from the current main spec, re-apply this change's edits, and verify `openspec validate mouse --strict` passes
- [ ] 8.3 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass
- [ ] 8.4 By hand, in kitty or Alacritty: drag-select in a shell and paste it into another program, drag and resize windows in navigation mode, and slide the band, and verify each behaves as the mouse spec describes
