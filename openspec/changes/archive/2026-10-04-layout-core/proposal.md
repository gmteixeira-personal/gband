## Why

gband serves one pane today, so it is a remote shell rather than a multiplexer. The niri layout is the product: panes in columns on a strip that scrolls sideways, workspaces stacked vertically, and a view that follows focus. That model is pure logic, and every later feature builds on it: rules, agent badges, persistence and Lua all read or change the layout. It needs a home with no I/O and thorough unit tests before it is wired into the server and the client. This change covers build steps 2 and 3 of `summary.txt`.

## What Changes

- `gband-core` gains the layout model as pure code with unit tests:
  - Workspaces, each a strip of columns. Each column is a stack of panes, and each column has a width.
  - Width presets of 1/3, 1/2 and 2/3, plus full width.
  - Opening, closing, consuming and expelling panes.
  - Dynamic workspaces that always keep one empty workspace at the bottom.
  - Tile geometry: the size of every pane for a given screen area.
  - The per-client view: the viewed workspace, the focused pane and a camera that scrolls just enough to keep the focused column visible.
- The server hosts many panes, each with its own PTY, program and authoritative grid. It owns the layout and changes it only through session actions. It sizes every PTY from the layout and from the screen area of the client that most recently attached or resized. It sends each client the layout and per-pane snapshots and updates.
- Actions split between the client and the server:
  - **View actions** run in the client alone, with no round trip: focusing a pane and switching the viewed workspace. Each client has its own focus and camera.
  - **Session actions** go to the server: a new pane, closing a pane, consume or expel, and width changes. The server applies them to the one shared layout and sends the result to every client.
- The client draws the viewed workspace as a ribbon. Each pane is drawn in a bordered tile, and the focused pane's border is highlighted. A column that the camera cuts at the left or right edge of the terminal is clipped there, rather than resized.
- The key bindings are a hardcoded Rust table in the client, which Lua replaces later. Ctrl+A is the leader key. These keys follow it:

  | key | action | kind |
  |---|---|---|
  | `h` / `l` | focus the column to the left / right | view |
  | `j` / `k` | focus the pane below / above | view |
  | `u` / `i` | view the workspace below / above | view |
  | Enter | open a new pane in a new column to the right | session |
  | `q` | close the focused pane | session |
  | `[` / `]` | consume or expel the focused pane to the left / right | session |
  | `r` | cycle the column width through the presets | session |
  | `f` | toggle the column's full width | session |
  | `D` | detach | client |
  | Ctrl+A | send one Ctrl+A to the focused pane | client |

- **BREAKING**: leader `d` no longer detaches. Detaching takes leader `D`, which is Shift+D.
- **BREAKING**: the protocol version becomes 2. Keys and pastes name their target pane. Snapshots and updates name their pane. New messages carry the layout, session actions and focus. A version 1 client and a version 2 server reject each other through the existing handshake.
- The session ends when the last pane's program exits, not when the first program exits. SIGTERM hangs up every pane.
- Panes get `GBAND_PANE`, set to their pane id, in their environment.

## Capabilities

### New Capabilities

- `layout`: the session's shared layout. Covers workspaces and dynamic workspaces, columns and pane stacks, column widths and presets, opening, closing, consuming and expelling panes, and tile geometry for a screen area.
- `layout-view`: one client's view of the layout. Covers the viewed workspace, the focused pane and how focus moves, how the view follows layout changes, and the camera.

### Modified Capabilities

- `session-server`: many panes instead of one. Covers per-pane programs, grids, snapshots and updates, input routed by pane, the screen area replacing the PTY-size rule, session actions, closing a pane, and the session ending with its last pane.
- `client-attach`: the client renders the ribbon with clipping and borders, sends input to the focused pane, and resolves the bound keys into view actions and session actions. The prefix-key table changes, and detach moves to `D`.
- `wire-protocol`: protocol version 2, with pane-addressed input, snapshots and updates, and new layout, action and focus messages.
- `command-line`: `gband server` runs until the session ends, which is now when its last pane exits.

## Impact

- Code: new layout, geometry and view modules in `crates/core/src/`. Pane-addressed messages in `crates/protocol/src/message.rs`. A multi-pane session in `crates/server/src/`. A key-binding table, view state and ribbon rendering in `crates/client/src/`, where the binding table replaces `prefix.rs`.
- Tests: new unit tests for the layout and the view in `crates/core/tests/`. The server's test harness and its sync and lifecycle tests move to per-pane messages, and new tests cover several panes. `tests/attach.rs` detaches with `D` and gains end-to-end tests for opening, focusing and closing panes.
- Dependencies: none added.
- Compatibility: a version 1 client attaching to a version 2 server is rejected, or replaced by a debug build, as the existing handshake defines.
- Out of scope: Lua configuration and key bindings, animations, moving panes or columns between workspaces, fixed-cell widths and interactive resizing, new panes starting in the focused pane's directory, status bar and column titles, mouse input, and keeping each client's view across a detach.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/core/src/
- crates/core/tests/layout.rs
- crates/core/tests/geometry.rs
- crates/core/tests/view.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/protocol/tests/framing.rs
- crates/server/src/
- crates/server/tests/
- crates/client/src/
- crates/client/tests/connect.rs
- crates/client/tests/render.rs
- tests/attach.rs
- tests/common/mod.rs
- openspec/changes/layout-core/
