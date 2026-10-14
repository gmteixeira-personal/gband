## Context

A window's program output is read in `read_output` (`crates/server/src/window.rs:465`), sent to `taps.output`, and fed to `Window::process` (`:325`), which runs `grid.process`, then `take_clipboard`, then `take_write_back`. Write-back goes as `Input::WriteBack` into the window's input channel and reaches the PTY in `spawn_input` (`:478`). Raw output is not stored; `Window::replace` (`:176`) rebuilds a grid from client-supplied plugin content only.

`vte` 0.15, under vt100 0.16.2, enters `SosPmApcString` on `ESC _` and ignores every byte but 0x18, 0x1A and 0x1B (`vte` `anywhere()`). The closing `ESC \` reaches vt100's `unhandled_escape`, which logs it (`crates/emulator/src/callbacks.rs:153`). An APC payload never reaches a callback, so a kitty graphics sequence is dropped with no screen effect today.

The clipboard path stamps the latest write per window with a session `AtomicU64` and the connection pulls the highest stamp at `sync` (last-wins). A separate ordered path exists: each client has an `UnboundedSender<ServerMessage>` hub (`crates/server/src/hub.rs:28`, `connection.rs:334`), and events, focus, reload and control messages are pushed onto it in order and drained after each `sync`.

The client sends nothing about its terminal today (`Hello { version, cols, rows }`, `Attach { session, cwd }`). It writes no startup query; the stdin thread (`crates/client/src/lib.rs:2260`) feeds bytes to a `Decoder` (`crates/core/src/terminal_input.rs`) that turns an APC reply into Alt-prefixed keys. `TerminalGuard::enter` (`:186`) writes only bracketed paste and mouse modes. The client already writes raw bytes to its terminal between frames through `Step::Write` (`:2064`), which carries clipboard OSC 52.

A vt100 cell holds 22 bytes and `append` stops at length 18 (`vt100-0.16.2/src/cell.rs`), so U+10EEEE (4 bytes) plus up to 4 three-byte combining marks fit; kitty uses at most 3. Snapshot and diff write the full `cell.contents()`. The client renders a cell through tui-term into a ratatui `Buffer` with `set_symbol(contents())`; a U+10EEEE symbol with zero-width marks counts as width 1. `apply_palette` (`crates/client/src/render.rs:204`) remaps `Indexed` below 16, named colours and `Reset` when a colorscheme palette is set, and passes RGB and indexes 16 and up unchanged. The server sets `TERM=xterm-256color` and `COLORTERM=truecolor` for window programs (`window.rs:371`).

## Goals / Non-Goals

**Goals:**
- A kitty graphics sequence a window program writes reaches every attached client's outer terminal, in order, with no configuration.
- A placeholder cell carrying an image id reaches each client through the ordinary screen path and renders at the right id.
- A program learns graphics work only when a client's terminal supports them.
- A user can turn forwarding off per client.

**Non-Goals:**
- Caching or replaying images to a client that attaches after the image was sent. The user chose live clients only.
- Translating cursor-relative placements (`a=p`, `a=T` without `U=1`) to pane coordinates, clipping them, or deleting them on scroll. Only Unicode placeholders are forwarded; placement is the program's job through the grid.
- Rendering images in the gband harness terminal, which is itself a gband grid and drops APC. End-to-end tests assert the server message and the client's write, not pixels.
- A server-side option. Support is a property of each client's terminal.

## Decisions

### Collect graphics in the emulator, drop nothing from the screen
`crates/emulator/src/graphics.rs` holds a scanner with a pending buffer. `Vt100::process` (`crates/emulator/src/lib.rs`) feeds each `process` chunk to the scanner before or alongside the existing `\x1bc` reset scan; the scanner finds `ESC _ G ... ESC \` runs, appends a partial run to its pending buffer, and on the closing `ESC \` pushes the complete sequence to a `Vec<Vec<u8>>`. The full chunk still goes to `vte`, which drops the APC, so the screen is unchanged and no stripping is needed. `Vt100::take_graphics` drains the vec, mirroring `take_write_back`.

The scanner discards a pending sequence that passes 16 MiB without ending, so a program cannot grow memory without bound. A sequence is recognised only when the byte after `ESC _` is `G`; any other APC string is skipped to its `ESC \`.

Alternative: scan raw output in `Window::process` before `grid.process`. Rejected: escape parsing already lives in the emulator, and `take_graphics` beside `take_write_back` and `take_clipboard` keeps one drain pattern.

### A shared classifier for the support query
`crates/emulator/src/graphics.rs` exposes `fn query_reply(sequence: &[u8]) -> Option<Vec<u8>>`: it parses the control keys of a sequence, and when `a=q` is present returns `ESC _ G i=<id> ; OK ESC \` with the `i` value or `0`. The window calls it to decide whether a sequence is a query and what to answer. Parsing is shallow: split the control section on `,`, read `a` and `i`.

### Forward on the ordered hub, broadcast like events
`Window::process` calls `grid.take_graphics()` after `grid.process`. For each sequence, `query_reply` decides:
- A query: the window asks the session whether any attached client supports graphics; when one does, it pushes the reply as `Input::WriteBack`, joining the program's other replies in order; otherwise it writes nothing. The query is never forwarded.
- Not a query: the window hands the sequence to the session, which pushes `ServerMessage::Graphics { window, data }` onto each attached client's hub in order.

The hub is the ordered per-client queue events use, so order is kept and nothing is cached. A sequence written with no client attached reaches no hub and is dropped. A client attaching later starts with an empty hub, so it receives no earlier sequence, which is the chosen live-clients-only behaviour.

Alternatives:
- The clipboard stamp-and-pull path. Rejected: it is last-wins, and every graphics chunk matters and must keep order.
- Forward only for windows the client shows. Rejected: a window can scroll into view later, and with no cache its image would be gone; kitty image ids are terminal-global, so forwarding for every attached client is simpler and correct.

### The session tracks each client's graphics support
`Attach` gains `graphics: bool`. The connection stores it when the client attaches (beside the client number), and the session answers "any client supports graphics" by scanning its attached connections. The flag changes only at attach, so detach and attach re-evaluate it naturally.

### Client detection at startup
`crates/client/src/connect.rs` builds the attach request, so detection runs there or in `lib.rs` before the stdin decoder starts. After `TerminalGuard::enter` sets raw mode, the client writes a kitty graphics query and `ESC [ c`, then reads the terminal's replies until the device attributes reply `ESC [ ? ... c` or a 1 second timeout. A graphics reply `ESC _ G ... ESC \` arriving before the device attributes reply means support. The query is a 1x1 RGB transmit with `a=q`, the sequence kitty documents for detection: `ESC _ G i=1,s=1,v=1,a=q,t=d,f=24;AAAA ESC \`. The read happens before the decoder thread owns stdin, so no reply leaks into keys. The attach request reports support AND the `window_graphics` option.

Alternative: trust `TERM` or an env var. Rejected: terminals that support kitty graphics carry varied `TERM` values (`xterm-kitty`, `xterm-ghostty`, WezTerm, Konsole), and the query is the only reliable test.

### Placeholder passthrough and the palette skip
Placeholder cells need no new path: U+10EEEE cells with their foreground id ride the snapshot and diff and render at width 1. The one correction is `apply_palette`: it must skip a cell whose `contents()` begins with U+10EEEE, so a colorscheme does not remap an id carried as a 256-colour index below 16 or a named colour. Ids carried as 24-bit RGB already pass unchanged; the README recommends RGB or an index of 16 and up, and the skip covers the rest.

### The client writes forwarded bytes between frames
The client turns `ServerMessage::Graphics { data, .. }` into a `Step::Write(data)` when `window_graphics` is true, the same mechanism clipboard OSC 52 uses, so the bytes reach the terminal between frames in arrival order. With the option false the client reported no support at attach, so it receives no graphics message; the write path is a second guard only.

### Protocol version 16
A new `ServerMessage` variant and a new attach field both change the encoding, so `PROTOCOL_VERSION` goes to 16 and `crates/protocol/tests/messages.rs` gains the graphics round trip, the attach graphics round trip, and the version 15 rejection.

### The `window_graphics` option
`crates/lua/src/options.rs` gains `window_graphics: bool`, default `true`, set in `defaults.lua` after `window_clipboard`. It is a client option like `window_clipboard`; it feeds the attach graphics flag at startup. No Lua event is added: raw image bytes are not useful to a handler.

## Risks / Trade-offs

- [A client that attaches after an image was sent shows blank placeholder cells until the program re-sends] → The user chose live clients only. The README names it. A program that redraws on resize or focus recovers.
- [The gband harness terminal drops APC, so an end-to-end lua spec cannot see a forwarded image] → Tests assert `ServerMessage::Graphics` and the client's `Step::Write` in the server and client crates. No lua spec is added for forwarding.
- [The startup probe could delay attach if the terminal never replies] → The 1 second timeout and the `ESC [ c` sentinel bound it; any vt-ish terminal answers device attributes.
- [A program carries an image id as a 256-colour index below 16] → `apply_palette` skips placeholder cells, so the id survives. The README still recommends RGB or an index of 16 and up as the robust form.
- [A client of protocol 15 cannot attach to a server of protocol 16] → The handshake rejects it with the version, as every earlier bump.
- [A burst of large images floods a slow client's unbounded hub] → Matches how events already queue. A 16 MiB cap bounds each sequence; an option-off client receives none.

## Migration Plan

None beyond the protocol bump. An old server keeps running until killed; a new client reports the version mismatch when it attaches to it.
