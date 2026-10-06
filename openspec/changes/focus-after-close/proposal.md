## Why

When a client opens a window and asks to focus it, the server queues the focus message as soon as the window is placed. If the window's program exits at once, the window can leave the session before that message goes out, so the client is told to focus a window that the last layout it received no longer holds, or one it has not yet received any layout for. The session-server spec requires the focus message to follow the layout that holds the window. The real client ignores a focus for an unknown window, so users see nothing, but the test client checks the order and fails: `argument_list_program_runs_without_a_shell` in `crates/server/tests/programs.rs`, whose `printf` exits at once, failed twice in today's gate runs with "focus before a snapshot".

## What Changes

- The server sends a client the focus message for a window it opened only after sending that client its latest layout, and only when that layout holds the window. A focus message for a window that has already left the layout is dropped and logged at debug level.
- This applies wherever the connection delivers queued replies, including the flush the test channel's settle uses, which today delivers replies before syncing the layout.
- A regression test opens many windows whose program exits at once and checks that every focus message names a window the last received layout holds.
- `argument_list_program_runs_without_a_shell` runs an argument list that prints `a-b` and then waits for input, `awk` reading its PTY, instead of `printf`. A program that exits at once can leave the layout before any client receives a layout holding it, so no test can reliably read its output. With the focus fix alone, the test times out waiting for a focus the server rightly drops: 4 of 46 runs did.
- Out of scope: the opened reply that answers a plugin window request. The client needs it to resolve its request even when the window has already closed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `session-server`: "Session actions" states that the focus message for an opened window is sent only while the window is in the layout and after the layout that holds it, and is dropped once the window has left the layout. "Window program" names an argument list that stays open in its "Argument list program" scenario.

## Impact

- `crates/server/src/connection.rs`: the delivery of focus replies in the connection loop.
- `crates/server/tests/programs.rs`: the regression test, and the argument list test's program.
- No protocol change: the message stays the same, and a client already copes with receiving none.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- openspec/changes/focus-after-close/
- crates/server/src/connection.rs
- crates/server/tests/programs.rs
