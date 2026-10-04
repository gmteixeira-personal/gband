## Context

See proposal.md for the motivation and specs/session-server/spec.md for the queries and replies.

`crates/server/src/pane.rs` owns one `vt100::Parser<LoggingCallbacks>` behind a `Mutex`. A reader thread feeds it every chunk the PTY yields. One input thread owns the PTY writer and writes encoded keys and pastes from an unbounded `mpsc` channel of `Input`. `crates/server/src/callbacks.rs` implements `vt100::Callbacks` and only logs.

Three `vt100` 0.16.2 facts shape the approach:

- Every query in scope reaches `Callbacks::unhandled_csi` with a `&mut Screen`: `c` for DA1, `n` for DSR, `q` with `>` for XTVERSION, and `p` with `$` for DECRQM. The callback can read the cursor and modes from that screen at the exact point in the stream where the query sits.
- `Parser::callbacks_mut()` gives the parser's owner access to its callbacks after `process` returns.
- DCS sequences are consumed by the parser and never reach any callback. This is why XTGETTCAP is out of scope.

## Goals / Non-Goals

**Goals:**
- Replies are computed from the grid state at the query's position in the output, not from the state after the whole chunk.
- One writer for the PTY, so a reply never splits a key's escape sequence or the other way round.

**Non-Goals:**
- A general terminal-response framework. The table in the spec is the whole set.
- Forwarding queries to a client's real terminal.

## Decisions

### Callbacks collect replies into a buffer

`LoggingCallbacks` becomes a struct with a `replies: Vec<u8>` field. Its renamed type, `PaneCallbacks`, recognises the queries in `unhandled_csi`, appends the reply bytes and returns. Anything it does not answer is logged as now. After `parser.process(bytes)`, `Pane::process` takes the buffer with `std::mem::take(&mut parser.callbacks_mut().replies)` while it still holds the lock.

The reply is built inside the callback, while the grid is at the query, so `\e[5;10H\e[6n` in one chunk answers `\e[5;10R` even when later bytes in that chunk move the cursor.

Alternative: scan the raw PTY bytes for queries before or after feeding the parser. Rejected. The scan would duplicate the parser's escape-sequence state machine, which has to survive a sequence split across reads. It would also read the cursor after the whole chunk, not at the query.

### Replies go through the input channel

`Input` gains a `Reply(Vec<u8>)` variant. The input thread writes it as raw bytes, without encoding. `Pane::process` sends a non-empty reply buffer on that channel, so the reader thread never touches the writer.

This needs the input channel to exist before the reader thread starts. `spawn` creates the channel first, gives the sender to the `Pane`, and starts the input thread from the receiver. The `Session::input` sender stays as the clients' entry point.

Alternative: a second `Mutex` around the writer, shared by the reader and input threads. Rejected. The channel already serialises writes, and a lock would let a slow PTY write block the reader thread, which must never stop draining output.

### Mode states read from public `Screen` accessors

DECRQM answers only modes with a public accessor in `vt100` 0.16.2:

- `application_cursor` for mode 1.
- `hide_cursor` for mode 25.
- `alternate_screen` for modes 47 and 1049.
- `mouse_protocol_mode` for modes 9, 1000, 1002 and 1003.
- `mouse_protocol_encoding` for modes 1005 and 1006.
- `bracketed_paste` for mode 2004.

`vt100` also tracks origin mode (6), but exposes no accessor for it, so mode 6 reports `0`. A wrong `0` makes a program assume the mode is unsupported and avoid it. A wrong `1` or `2` would misstate the screen.

### DA1 claims a VT220 with ANSI colour

`\e[?62;22c` declares a level-2 terminal with ANSI colour and nothing else. It claims no sixel (4), no rectangular editing (28) and no other optional feature gband lacks. fish only needs a reply to arrive. nvim and other programs read the parameters, and must not be told about features that do not exist.

### XTVERSION names gband

The reply carries `gband` and `env!("CARGO_PKG_VERSION")` of the server crate. The server and the binary share the workspace version, so this is the version the user installed.

## Risks / Trade-offs

- [A program that writes a query and never reads the reply leaves the reply in its input, where a shell may echo it] → Every real terminal does the same. Answering is what a terminal is expected to do.
- [Unanswered OSC 11 leaves nvim and Claude Code guessing the background as dark] → That is their behaviour without gband today. A later change can have the client report its terminal's colours.
- [`layout-core` reworks `pane.rs` into multiple panes] → The reply buffer lives in each pane's callbacks and the reply travels on that pane's input channel, so the approach carries over per pane. Whichever change lands second merges the other's `pane.rs` at `/ready`.
- [The fish scenario depends on fish being installed] → The automated tests drive each query with `sh` and `printf`. The fish scenario is verified by hand, in a manual task.

## Migration Plan

None. The server answers queries from the release that contains this change. Nothing is stored, and no client or wire-protocol change is needed. A server started from an older build keeps its old behaviour until it is replaced.
