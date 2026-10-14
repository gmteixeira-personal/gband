## Why

In gband 0.3.2 the terminal's cursor flashes away from the focused window on every screen change. Each frame, ratatui moves the visible cursor through every changed cell, sends Show with its own flush while the cursor sits on the last changed cell, and only then moves it back. Ghostty paints between those writes. gband never sends the synchronized-update markers of mode 2026, so the terminal cannot tell where a frame ends.

## What Changes

- The client wraps every frame it draws in `\e[?2026h` and `\e[?2026l`, so a terminal that supports mode 2026 shows each frame whole.
- The client hides the cursor at the start of every frame, before the first changed cell is written. A terminal without mode 2026 then never shows the cursor on a cell it passes through while drawing.
- A frame whose drawing fails still ends its synchronized update before the error is returned.
- Leaving the client ends any synchronized update before the terminal is restored, so a frame cut short by a panic does not leave the terminal holding back output.

## Capabilities

### New Capabilities

### Modified Capabilities
- `client-attach`: a new requirement "Whole frames" makes each frame one synchronized update with the cursor hidden until the frame places it. "Leaving the client" ends any synchronized update first.

## Impact

- `crates/client/src/lib.rs`: a public `synchronized` function sends the begin marker and Hide before `terminal.draw` and the end marker after it, whatever its result, and `draw` calls it. `TerminalGuard`'s `drop` sends the end marker.
- `crates/client/tests/frames.rs`: new tests that read the bytes of a frame written to a `Vec<u8>`.
- No protocol, server, Lua or configuration change. No new dependency: crossterm 0.29 already provides `BeginSynchronizedUpdate` and `EndSynchronizedUpdate`.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/client/src/lib.rs
- crates/client/tests/frames.rs
- openspec/changes/synchronized-frames/
