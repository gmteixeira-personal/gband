## Why

Column widths can only be cycled through three presets or set to full width, and the panes in a stacked column always share its height equally. layout-core left interactive resizing out of scope. Every resize also reaches the program at once. Dragging the terminal's edge or holding a resize key sends a SIGWINCH for every step, and programs such as nvim and Claude Code redraw for each one. A pane that no client shows is resized along with the shown ones, although nobody sees its redraw. The design summary asks for the opposite: resize the PTY once at the end of a burst, and leave offscreen panes at their size.

## What Changes

- **Step a column's width.** New session actions grow and shrink the width of the focused pane's column by 1/10 of the screen area's width, kept between 1/10 and 1. They follow niri's `set-column-width "+10%"` and `"-10%"`.
- **Pane heights in a stack.** Each pane in a column carries a height weight, 1 by default. The column's height is shared among its panes in proportion to their weights. New session actions grow and shrink the focused pane's height and reset the heights of its column.
- **Key bindings.** After Ctrl+A, `-` and `=` shrink and grow the column width. `_` and `+` shrink and grow the pane height. `R` resets the column's pane heights.
- **Settled PTY resizes.** The server publishes every layout and screen-area change at once, so clients redraw tiles at the new size straight away. It resizes the PTYs only once 100 ms pass with no further change, so a burst of resizes reaches each program as one SIGWINCH at the final size. Until then, the client draws the pane's grid at its old size, cut or padded to the tile.
- **Offscreen panes keep their size.** Each client reports the panes it shows. The server resizes only panes that at least one attached client shows. A pane no client shows keeps its PTY size until a client shows it. A session with no client attached resizes nothing.
- **BREAKING** The wire protocol gains a shown message and pane height weights in the layout. The protocol version goes from 3 to 4, so older clients and servers detect each other and refuse.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `layout`: column widths gain grow and shrink steps. Columns gain per-pane height weights, with grow, shrink and reset. Tile geometry shares a column's height by weight.
- `layout-view`: a view defines the panes it shows: the panes whose tiles have a cell inside the client's terminal.
- `session-server`: the session actions gain the five resize actions. PTY resizes wait for a 100 ms quiet period and apply only to shown panes.
- `session-events`: a new layout event reports a column's changed pane heights.
- `wire-protocol`: a new shown client message, height weights in the layout message, and protocol version 4.
- `client-attach`: the new key bindings, reporting shown panes, and drawing a grid whose size differs from its tile.

## Impact

- Code: `gband-core` layout, geometry, actions, events and view. `gband-protocol` messages. `gband-server` session and connection. `gband-client` bindings and its main loop. The shared test harness in `gband-test-support`.
- Tests: new unit tests in `crates/core/tests/` and a new server integration test file for settled and shown-only resizes. Existing server tests that assert PTY sizes report their panes as shown and wait for the quiet period.
- Dependencies: none.
- Out of scope: fixed-cell column widths, which need configuration to be useful. Animating a resize. Making the 100 ms quiet period or the 1/10 step configurable, which waits for Lua. Mouse-driven resizing.

## Coordination

### Author
- gmteixeira

### Depends On
- core-coverage-gaps

### Expected Files
- crates/core/src/action.rs
- crates/core/src/event.rs
- crates/core/src/geometry.rs
- crates/core/src/layout.rs
- crates/core/src/view.rs
- crates/core/tests/events.rs
- crates/core/tests/geometry.rs
- crates/core/tests/layout.rs
- crates/core/tests/view.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/src/connection.rs
- crates/server/src/session.rs
- crates/server/tests/panes.rs
- crates/server/tests/resize.rs
- crates/server/tests/sessions.rs
- crates/test-support/src/lib.rs
- crates/client/src/bindings.rs
- crates/client/src/lib.rs
- crates/client/tests/actions.rs
- crates/client/tests/render.rs
- crates/client/tests/snapshots/
- tests/attach.rs
- openspec/changes/resize-semantics/
