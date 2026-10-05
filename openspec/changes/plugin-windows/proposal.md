## Why

A plugin can dispatch the built-in actions, but every action acts on the focused pane and nothing else. A plugin cannot read the layout, act on a pane it names, set an exact column width or pane height, focus a given pane or band, or type into a pane. It also has no surface of its own: the only thing it can draw is a status line segment. So a plugin cannot show a scrollable list of key bindings, or a list of files to pick from. This change gives plugins the operations a user has, aimed at any pane, and lets them open windows whose contents they write.

## What Changes

- **Reading state**: `gband.layout()` returns the client's layout: bands, columns with their widths, and panes with their heights. `gband.view()` returns the viewed band, the focused pane, the focused window, the active key table and the ribbon area's size. Both can be called from any code that runs after loading.
- **Targeted actions**: the built-in session action values, and `send_prefix`, take an optional target table: `gband.action.close_pane({ pane = 3 })`, `gband.action.open_pane({ band = 2, after = 5 })`. Without a target they resolve against the view as today. `gband.spawn` also takes `band` and `after`.
- **Pane control** in `gband.pane`: `focus(pane)` views the pane's band and focuses it. `set_width(pane, width)` sets the column's width to an exact proportion. `set_height(pane, { rows = n })` or `set_height(pane, { weight = w })` sets an exact fixed or automatic height. `send_keys(pane, keys)`, `send_text(pane, text)` and `paste(pane, text)` give a pane input as if the user typed or pasted it. `gband.band.view(band)` views a band.
- **New session actions** set width and set height, with layout rules that match the existing grow and shrink rules for fixed heights.
- **Plugin windows** in `gband.win`: `open`, `close`, `set_lines`, `scroll`, `set_cursor`, `focus`, `set_config`, `info` and `list`. A window shows lines of styled spans, written as strings or `{ text, hl }` lists and drawn with highlight groups. It keeps a scroll position and an optional cursor line. While focused, it takes the keys the key bindings do not consume: a per-window `keys` table first, then default scrolling keys. Callbacks `on_close` and `on_resize` run when the window closes or its size changes.
- Two kinds of window:
  - **float**: drawn by this client only, over the ribbon area, at a position and size in cells. It has a border, an optional title, and a stacking order. Its position and size change with `gband.win.set_config`.
  - **pane**: a **plugin pane** in the shared layout. It has no program and no PTY. It is placed, resized, consumed, expelled, focused and closed like any pane, so `gband.pane` and the actions resize it. The owning client writes its screen, and the server keeps that screen and sends it to every client like a program's screen. The server drops keys and pastes sent to a plugin pane. Its plugin panes close when the owning client detaches, disconnects or reloads.
- Highlight groups `Window`, `WindowBorder`, `WindowTitle` and `WindowCursorLine`, with defaults owned by the window API.
- The session ends when its last program pane leaves. Plugin panes close with it, so a plugin pane never keeps a session alive.
- Protocol version 5:
  - Open pane names either a program or plugin content, and also takes an optional column width and a focus flag.
  - New session actions set width and set height.
  - A new client message `content` writes a plugin pane's screen.
  - A new server message `opened` tells the client which pane its plugin pane request created.
- **BREAKING**: a client and server on protocol version 4 cannot attach to each other. The version check already refuses that pairing, and the client replaces a mismatched server as it does today.

Out of scope:
- A shortcut list, file picker or any other plugin built on this API.
- Mouse input in windows.
- Line wrapping.
- Windows anchored to a pane or cursor.
- Windows in the server's Lua.
- Reading another pane's screen contents.
- Showing another client's plugin windows as anything but a pane's screen.

## Capabilities

### New Capabilities

- `lua-control`: `gband.layout()`, `gband.view()`, action targets, `gband.pane` and `gband.band`, and the order those calls take effect in.
- `plugin-windows`: `gband.win`, window kinds, contents and their drawing, scrolling and the cursor line, focus and key handling, float geometry, plugin pane contents, callbacks, closing, and the window highlight groups.

### Modified Capabilities

- `actions`: a session action with a target names that pane or band, instead of resolving against the view.
- `configuration`: `gband.spawn` takes `band` and `after`.
- `layout`: open pane with an optional column width, set width, set height, and plugin panes leaving the layout when closed.
- `layout-view`: focusing a named pane and viewing a named band.
- `client-attach`: keys and pastes go to a focused window, floats are drawn over the tiles, and the cursor is hidden while a float has focus.
- `session-server`: plugin panes, the new session actions, input and close for plugin panes, and the session's end at its last program pane.
- `wire-protocol`: protocol version 5, the `content` and `opened` messages, and the new action fields.

## Impact

- `crates/core`: `SessionAction` gains set width and set height. Open pane gains the content kind, the width and the focus flag. `Layout` and `Column` gain exact setters. `View` gains band viewing by identifier.
- `crates/protocol`: the new messages and fields, and `PROTOCOL_VERSION` 5.
- `crates/server`: program-less panes with an owner, the `content` message, immediate close, closing a client's plugin panes when it leaves, and the session's end rule.
- `crates/lua`: new Rust modules for the state snapshot, `gband.layout`, `gband.view`, `gband.pane` and `gband.band`, and the new `Dispatch` entries. A bundled `win.lua` API chunk on the status line's host table, with new host primitives that present window frames.
- `crates/client`: key and paste routing to the focused window, float drawing in `render.rs`, plugin pane content encoding and sending, the `opened` reply, closing windows on reload, and the window callbacks after layout and snapshot changes.
- `docs/plugins.md` documents the new API.
- No new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- status-line

### Expected Files
- crates/core/src/layout.rs
- crates/core/src/view.rs
- crates/core/tests/layout.rs
- crates/core/tests/view.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/
- crates/server/src/pane.rs
- crates/server/src/session.rs
- crates/server/src/connection.rs
- crates/server/tests/
- crates/lua/
- crates/client/src/lib.rs
- crates/client/src/render.rs
- crates/client/src/windows.rs
- crates/client/tests/
- tests/windows.rs
- tests/common/mod.rs
- docs/plugins.md
- openspec/changes/plugin-windows/
