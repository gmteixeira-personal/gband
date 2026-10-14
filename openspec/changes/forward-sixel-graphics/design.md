## Context

`forward-kitty-graphics` built the forwarding spine this change reuses: the emulator's APC scanner in `crates/emulator/src/graphics.rs`, `Vt100::take_graphics`, the per-client hub broadcast in `crates/server/src/window.rs` and `session.rs`, the attach graphics flag, the startup probe in `crates/client/src/lib.rs`, and `ServerMessage::Graphics`. That change forwards kitty graphics; this one adds sixel, the only image protocol Windows Terminal speaks.

Two facts set sixel apart from kitty:

- Kitty placement rides the grid. A program writes U+10EEEE placeholder cells carrying the image id, and those cells reach every client through the snapshot and diff. The forwarded APC only transmits the image; the cells place it. Sixel has no placeholder: a sixel DCS paints pixels at the cursor the instant it is written, and nothing in the grid records where.
- `vte` 0.15 drops a sixel DCS exactly as it drops the kitty APC: `ESC P` enters `DcsPassthrough`/`DcsIgnore` and no byte reaches a vt100 callback, so the screen is unchanged and no stripping is needed, but the sequence and its position are lost unless collected.

The client answers a program's Primary Device Attributes query from the emulator callback (`crates/emulator/src/callbacks.rs:319`), a static `\e[?62;22c`. Sixel-capable terminals advertise sixel by adding attribute `4` to that reply, and yazi reads attribute `4` to choose sixel. So the reply must gain `4` when a sixel client is attached, which makes a previously static reply depend on the attached clients.

The client already writes raw bytes between frames through `Step::Write` (clipboard OSC 52, forwarded kitty graphics). The client renders through ratatui's double buffer and diff: cells it treats as unchanged are not redrawn, so pixels written over blank cells survive until those cells change.

## Goals / Non-Goals

**Goals:**
- A sixel image a window program writes reaches every attached client whose terminal renders sixel, at the position the program drew it, with no configuration.
- A program is offered sixel only when an attached client's terminal supports it.
- Kitty and sixel coexist: each client's terminal is probed for both, and a program emits whichever gband advertises.
- A user can turn sixel forwarding off per client.

**Non-Goals:**
- Caching or replaying an image to a client that attaches after it was sent. Live clients only, as with kitty.
- Clipping a sixel to a window's visible rectangle, or redrawing it when the window scrolls, resizes or is covered. A sixel cannot be partly drawn by cells; the client draws it only when its anchor cell is visible and relies on the program to re-emit on redraw, as a program does on a bare terminal.
- Answering XTSMGRAPHICS (`\e[?...S`). Windows Terminal advertises sixel through Primary Device Attributes attribute `4` and does not answer XTSMGRAPHICS; yazi reads attribute `4`. gband leaves XTSMGRAPHICS unanswered, as today.
- Transcoding between kitty and sixel. One window program emits one protocol per run, so two clients on different terminals, one kitty-only and one sixel-only, cannot both see an image.
- Rendering images in the gband harness terminal, which is a gband grid and drops the DCS. Tests assert the server message and the client's write, not pixels.

## Decisions

### Collect sixel in the emulator beside the kitty scan, record the cursor
`crates/emulator/src/graphics.rs` gains a sixel branch in the scanner: a DCS is a sixel when `ESC P` is followed by only digits and `;` and then `q`, which excludes DECRQSS (`ESC P $ q`). The scanner buffers a sixel across `process` calls and ends it at the first `ESC \`. The emulator feeds `process` in segments bounded by a sixel's start: at a sixel's start it reads the grid cursor, 1-based from the window's top-left, and stores it with the pending sixel, because `vte` drops the DCS and does not move the cursor, so the cursor at the DCS equals the cursor just before it. `Vt100::take_sixel` drains a `Vec<(u16, u16, Vec<u8>)>` of position and bytes, mirroring `take_graphics`. The 16 MiB per-sequence cap of the kitty scanner applies.

Alternative: one combined scanner state shared with the APC path. Rejected: the APC and DCS introducers and terminators differ enough that two branches read more clearly than one merged state.

### Offer sixel through a Primary Device Attributes flag on the emulator
`Vt100` gains a `sixel_supported` flag, default false. The Primary Device Attributes reply in `crates/emulator/src/callbacks.rs` becomes `\e[?62;4;22c` when the flag is set and stays `\e[?62;22c` otherwise. The session sets the flag on each window's grid to whether any attached client supports sixel, updated on attach and detach, the same moment the kitty-support answer re-evaluates. A program querying Primary Device Attributes after a sixel client attaches sees `4`.

Alternative: intercept and rewrite the program's Primary Device Attributes query in `window.rs`, as the kitty `a=q` query is intercepted. Rejected: Primary Device Attributes is a normal query the emulator already answers, so a flag on the reply is smaller than diverting the query out of the callback.

### Forward on the ordered hub with the position
`Window::process` calls `grid.take_sixel()` after `grid.process`, beside `take_graphics`. For each sixel, the session pushes `ServerMessage::Sixel { window, col, row, data }` onto the hub of every attached client whose terminal supports sixel, in order. A sixel collected with no supporting client attached reaches no hub and is dropped. This is the kitty forwarding path with a position added and a per-client support filter.

### Detect sixel in the existing probe, add the attach flag
`graphics_probe` in `crates/client/src/lib.rs` already finds the Primary Device Attributes reply (`attributes_reply`) and its parameter bytes. It gains sixel detection: split those parameters on `;` and report sixel when `4` is present. `probe_graphics` returns kitty support and sixel support from the one probe; no second query is written. The attach request reports sixel support AND the `window_sixel` option, beside the graphics flag.

### Draw a forwarded sixel at its position between frames
The client turns `ServerMessage::Sixel { col, row, data, window }` into a positioned write when `window_sixel` is true: save cursor (`\e7`), move to the window's on-screen origin plus `(row, col)`, write the bytes, restore cursor (`\e8`). It runs after the frame's cells are flushed and only when the anchor cell is within the window's drawn area; otherwise it writes nothing, so a sixel never lands at the wrong place. ratatui's diff leaves the pixels alone until the underlying cells change; the program re-emits on redraw, which recovers the image after a scroll or resize.

### Protocol version 17 and the `window_sixel` option
A new `ServerMessage` variant and a new attach field change the encoding, so `PROTOCOL_VERSION` goes to 17, and `crates/protocol/tests/messages.rs` gains the sixel message round trip, the attach sixel round trip, and the version 16 rejection. `crates/lua/src/options.rs` gains `window_sixel: bool`, default `true`, set in `defaults.lua` after `window_graphics`. It is a client option like `window_graphics` and feeds the attach sixel flag.

## Risks / Trade-offs

- [A program queries Primary Device Attributes before a sixel client attaches and never re-queries] → It sees no `4` and uses no sixel until it next probes. Live-clients-only, as with kitty. The README names it.
- [A sixel's anchor cell scrolls or is covered after it was drawn] → The pixels stay until the cells under them change, then the program re-emits on its own redraw. gband does not track or clip a drawn sixel. The README names it.
- [Two clients on different terminals, one kitty-only and one sixel-only, attach at once] → The program emits one protocol, so only the matching client sees the image. gband does not transcode. The README names it.
- [Windows Terminal 1.22 is Preview, and one report says its Primary Device Attributes reply can be malformed] → `attributes_reply` already scans leniently for `ESC [ ? ... c`; parameter parsing tolerates a missing or extra field.
- [A client of protocol 16 cannot attach to a server of protocol 17] → The handshake rejects it with the version, as every earlier bump.

## Migration Plan

None beyond the protocol bump. An old server keeps running until killed; a new client reports the version mismatch when it attaches to it.
