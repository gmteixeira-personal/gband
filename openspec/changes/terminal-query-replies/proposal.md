## Why

Programs ask their terminal questions and wait for the answer. In gband the program's terminal is the server's PTY, and nothing on the other side of it answers. fish 4 sends Primary Device Attributes (`\e[0c`) at every start and blocks for 10 s waiting for the reply, so every new session opens on a blank pane. The server log records it as `unhandled CSI \e[0c`. The single-process spike found this first and named `terminal-query-replies` as the fix. Manual task 6.3 of `server-client-split` hit it again with the user's own shell.

## What Changes

- The session server answers a fixed set of terminal queries that a pane's program writes to its PTY. It writes each reply back to the same PTY, whether or not a client is attached.
- The answered queries are:
  - Primary Device Attributes (DA1).
  - Device Status Report: operating status (`\e[5n`) and cursor position (`\e[6n`).
  - XTVERSION.
  - DECRQM, for both ANSI and DEC private modes.
- Cursor position and mode replies come from the server's own grid of the pane, the same state clients are shown.
- Replies reach the PTY in order with the keys and pastes clients send, through the one writer the server already has.
- A query the server answers is no longer logged as an unhandled sequence.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `session-server`: gains a requirement that the server answers terminal queries from the pane's program, and names which queries and what each reply holds.

## Impact

- Code: `crates/server/src/callbacks.rs` collects replies while the grid parses output, and `crates/server/src/pane.rs` writes them to the PTY.
- Tests: a new `crates/server/tests/queries.rs` runs programs that send each query and read the reply.
- Dependencies: none added.
- Wire protocol and client: unchanged. Replies go to the program only, never to clients.
- Out of scope:
  - **Kitty keyboard query** (`\e[?u`). gband does not implement that keyboard protocol, so it stays unanswered. A program treats it as unsupported once the DA1 reply arrives.
  - **OSC 10 and 11 colour queries.** The server does not know the colours of the terminals its clients run in, and a wrong answer would make programs pick the wrong light or dark theme. Answering these needs the client to report its terminal's colours, which changes the wire protocol.
  - **XTGETTCAP.** It is a DCS sequence, and the `vt100` grid drops DCS sequences without passing them to its callbacks.
  - **Secondary Device Attributes (DA2).** No program found so far waits on it.
  - **fish ignoring SIGHUP while it waits for DA1.** That wait no longer happens, and the server's existing SIGKILL fallback still covers any program that ignores SIGHUP.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/server/src/callbacks.rs
- crates/server/src/pane.rs
- crates/server/tests/lifecycle.rs
- crates/server/tests/panes.rs
- crates/server/tests/queries.rs
- openspec/changes/terminal-query-replies/
