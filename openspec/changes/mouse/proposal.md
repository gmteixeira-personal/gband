## Why

gband ignores the mouse. It never turns on mouse reporting, and the client drops every mouse event it reads. A user cannot click a window to focus it, select text to copy it, or scroll a program such as `htop` or `less --mouse` with the wheel. niri's mouse model is what gband follows elsewhere: holding the modifier and dragging moves a window, dragging with the right button resizes it from the nearest edge, and dragging empty space slides the strip. Navigation mode is gband's held modifier, so the same gestures belong there. Plugins also need to react to the mouse, as they already react to keys and focus.

## What Changes

- **Mouse capture**: the client turns on the terminal's mouse reporting (presses, releases, any motion, SGR encoding) when it takes the terminal, and turns it off whenever it gives the terminal back.
- **Pointer targets**: each mouse event resolves against the frame drawn last to a floating plugin window, a floating window, a tile, empty ribbon or the space outside the ribbon. Border cells belong to their window.
- **Mouse names in key tables**: `leftmouse`, `middlemouse`, `rightmouse`, `wheelup`, `wheeldown`, `wheelleft` and `wheelright`, with optional `ctrl`, `alt` and `shift`, become key names. A press or a wheel step matches the active key table as a key does. A binding function bound to a mouse name receives the event's payload. Hints and the key list leave mouse bindings out.
- **Interactive mode defaults**, for a mouse name `root` does not bind:
  - A left, middle or right press focuses the window under the pointer.
  - When that window's program asked for mouse reporting and Alt is not held, the event is forwarded to the program, encoded as the program's mode asks.
  - Alt is the override, not Shift: most terminals keep Shift with a mouse event for their own selection, so gband would never see it. Shift+drag stays the terminal's native selection.
  - Otherwise, a left drag selects text in that window, shown in reverse video in this client. Release copies the text to the host terminal's clipboard with OSC 52, and to the client's copy buffer.
  - A right press pastes the copy buffer into the window under the pointer as a paste.
  - The wheel goes to the window under the pointer when its program reports the mouse, and otherwise does nothing. gband keeps no scrollback.
  - In a plugin window, a press moves the cursor line and the wheel scrolls.
- **Drag gestures**: three new client actions, which start a gesture when a mouse press runs them and do nothing otherwise:
  - `drag_window` moves a window. A floating window or floating plugin window follows the pointer. A tiled window lifts, follows the pointer, and drops into the place under it: a new column left or right of a column, or above or below a window in a column. A press on empty ribbon slides the band instead.
  - `drag_resize_window` resizes from the edges nearest the press, picked by thirds of the box as niri does. It sets the column's width or the window's height for a tiled window, and the box's size and position for a floating window or floating plugin window.
  - `drag_band` slides the viewed band's camera with the pointer. On release, focus moves to the column at the ribbon's middle.
- **Default navigation mode mouse bindings**, as holding niri's modifier:
  - `leftmouse` is `drag_window`, `rightmouse` is `drag_resize_window`, and `middlemouse` is `drag_band`.
  - `wheeldown` and `wheelup` view the band below and above.
  - `wheelright`, `wheelleft`, `alt+wheeldown` and `alt+wheelup` focus the column right or left. niri uses Shift here, but the terminal would keep Shift with the wheel.
- **Move a window to a place**: a new session action moves a tiled window into a new column beside another window's column, or above or below another window in its column. Dropping a dragged tiled window sends it.
- **Mouse reports to programs**: a new client message carries a mouse event for a window. The server encodes it with the window's mouse tracking mode and encoding: X10, normal, button-event or any-event tracking, in the default, UTF-8 or SGR encoding. The server drops the event when the mode does not report it. The emulator exposes each grid's mouse modes.
- **Lua**:
  - New built-in client events `MousePressed`, `MouseReleased`, `MouseDragged` and `MouseScrolled`. Their payloads carry the button or direction, the modifiers, the terminal cell, the target and its content-relative cell.
  - Plugin windows take an `on_mouse` callback.
- **Plugin testing**: `g.mouse(...)` writes a mouse report to the client's terminal as xterm sends it.
- **BREAKING**: the protocol version goes up, because of the new client message and session action.
- **BREAKING**: while attached, the host terminal no longer selects text on a plain drag. Most terminals still select natively with Shift held.

Out of scope:
- Scrollback, history scrolling and selecting text above the visible screen.
- Double-click word selection, triple-click line selection, and rectangular selection.
- Scrolling the camera when a drag nears the ribbon's edge, and dropping a dragged window into another band.
- Reading the host clipboard with OSC 52, and middle-click paste.
- A cooldown between wheel steps.
- Forwarding a program's own OSC 52 to the host terminal.

## Capabilities

### New Capabilities

- `mouse`: mouse capture, pointer targets, mouse names in key tables, the interactive mode defaults, selection and copy, forwarding to programs, the drag gestures, and how plugin windows and floating plugin windows take them.

### Modified Capabilities

- `configuration`: mouse names in "Key names". "Binding functions" passes the mouse payload to a function bound to a mouse name.
- `actions`: `drag_window`, `drag_resize_window` and `drag_band`, and how the client kind sends session actions for them.
- `client-attach`: the default navigation mode mouse bindings.
- `key-hints`: mouse bindings are not hinted.
- `key-list`: mouse bindings are not listed.
- `input-encoding`: mouse events from the terminal, and the bytes of a mouse report for each mode and encoding.
- `terminal-emulator`: each grid reports its program's mouse tracking mode and encoding.
- `wire-protocol`: the mouse client message, the move-to-place action, and the protocol version.
- `session-server`: the move-to-place session action, and writing mouse reports to a window's program.
- `layout`: moving a tiled window to a place beside or inside another column.
- `lua-events`: the four mouse events.
- `plugin-windows`: the `on_mouse` callback and the mouse defaults in plugin windows.
- `plugin-testing`: `g.mouse`.
- `animations`: drawing during a gesture, and the drop of a dragged tile.

## Impact

- `crates/core`: mouse input types and report encoding in `input.rs`, the move-to-place action in `layout.rs` and `action.rs`, the drag actions, and hit-testing over drawn geometry in `geometry.rs`.
- `crates/emulator`: mouse modes in `Modes`.
- `crates/protocol`: the `Mouse` client message, the new session action, and the version bump.
- `crates/server`: the move-to-place action in `session.rs`, and writing mouse reports from the PTY writer in `window.rs` and `connection.rs`.
- `crates/client`:
  - mouse capture and event intake in `lib.rs` and `input.rs`;
  - the new `mouse.rs` for targets, selection, gestures and the copy buffer;
  - selection and lifted-tile drawing in `render.rs` and `animation.rs`;
  - plugin window mouse handling in `plugin_windows.rs`;
  - mouse names in `bindings.rs`.
- `crates/lua`:
  - mouse names in `keys.rs`;
  - the drag actions in `actions.rs`;
  - the mouse events in `events.rs`;
  - `on_mouse` and the mouse defaults in `runtime/gband/win.lua`;
  - mouse bindings skipped in `runtime/gband/statusline/hints.lua` and `runtime/gband/keylist.lua`;
  - the navigation mode mouse bindings in `defaults.lua`.
- `crates/harness`: `g.mouse` in `runner.rs` and `case.rs`.
- Tests in the crates' `tests/` directories, a new `tests/mouse.rs`, and Lua specs with screenshots under `tests/lua/`.
- `README.md` and `docs/plugins.md`.
- No new dependency. crossterm already parses mouse events.

## Coordination

### Author
- gmteixeira

### Depends On
- navigation-mode

### Expected Files
- openspec/changes/mouse/
- crates/core/src/input.rs
- crates/core/src/layout.rs
- crates/core/src/action.rs
- crates/core/tests/input_encoding.rs
- crates/core/tests/layout.rs
- crates/emulator/src/lib.rs
- crates/emulator/tests/contract.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/src/window.rs
- crates/server/src/connection.rs
- crates/server/src/session.rs
- crates/server/tests/windows.rs
- crates/server/tests/sync.rs
- crates/lua/src/keys.rs
- crates/lua/src/api.rs
- crates/lua/src/actions.rs
- crates/lua/src/events.rs
- crates/lua/src/runtime.rs
- crates/lua/src/plugin_windows.rs
- crates/lua/src/defaults.lua
- crates/lua/src/runtime/gband/win.lua
- crates/lua/src/runtime/gband/keyform.lua
- crates/lua/src/runtime/gband/keylist.lua
- crates/lua/src/runtime/gband/statusline/hints.lua
- crates/lua/tests/keymap.rs
- crates/lua/tests/config.rs
- crates/lua/tests/events.rs
- crates/lua/tests/plugin_windows.rs
- crates/lua/tests/key_hints.rs
- crates/client/src/lib.rs
- crates/client/src/input.rs
- crates/client/src/render.rs
- crates/client/src/mouse.rs
- crates/client/src/bindings.rs
- crates/client/src/animation.rs
- crates/client/src/plugin_windows.rs
- crates/client/tests/key_translation.rs
- crates/client/tests/render.rs
- crates/client/tests/actions.rs
- crates/client/tests/events.rs
- crates/client/tests/animation.rs
- crates/client/tests/snapshots/
- crates/harness/src/runner.rs
- crates/harness/src/case.rs
- tests/attach.rs
- tests/mouse.rs
- tests/plugin_testing.rs
- tests/lua_specs.rs
- tests/lua/mouse_spec.lua
- tests/lua/keylist_spec.lua
- tests/lua/screenshots/mouse_spec/
- README.md
- docs/plugins.md
