## Why

gband forwards only the kitty graphics protocol (`openspec/changes/archive/2026-10-14-forward-kitty-graphics`). Windows Terminal supports sixel since v1.22 and advertises it through Primary Device Attributes attribute `4`, but does not implement the kitty graphics protocol. A program like yazi that shows images in Windows Terminal directly shows nothing inside a gband window: gband's emulator answers Primary Device Attributes with `\e[?62;22c`, no `4`, so the program never emits sixel, and even if it did the emulator drops the DCS and forwards nothing. The two protocols should coexist: a window program emits the one protocol gband advertises for the attached clients, and gband forwards each image only to the clients whose terminal renders it.

## What Changes

- The emulator collects a window program's complete sixel DCS sequences (`ESC P <params> q <data> ESC \`) while processing output, buffering a sequence split across writes, and records the window's cursor position when each sequence begins. The screen is unchanged; `vte` drops the DCS as it drops the kitty APC.
- The server forwards each collected sixel sequence, with its recorded position, in order, to every client whose terminal supports sixel, through the same per-client hub events use. A sequence written while no supporting client is attached is dropped. A client that attaches later receives none sent before it attached.
- The server includes attribute `4` in a window program's Primary Device Attributes reply (`\e[?62;4;22c`) only when an attached client's terminal supports sixel, mirroring how the kitty `a=q` query is answered. Without a supporting client the reply stays `\e[?62;22c`, so the program falls back to text or another protocol.
- The client detects sixel support at startup by reading attribute `4` from the Primary Device Attributes reply it already requests for the kitty graphics probe, so no second query is sent. The attach request gains a sixel flag beside the graphics flag. A new client option `window_sixel`, `true` by default, ANDs with the probe.
- The client draws a forwarded sixel at the recorded position: it saves the cursor, moves to the window's on-screen origin plus the recorded row and column, writes the sequence's bytes, and restores the cursor, between frames and after the frame's cells are drawn. It writes nothing when the target cell lies outside the window's visible area. A program re-emits the image on redraw after a scroll or resize, as it does on a bare terminal.
- The client measures its terminal's cell size in pixels, from `TIOCGWINSZ` or else the reply to `\e[16t` written in the startup probe, and reports it at attach and on resize. The session uses the most recently attached sixel client's cell size: the server sets each window PTY's pixel fields from it and answers a program's `\e[16t` and `\e[14t`, so yazi sizes images to the cells. Without a sixel client both pixel fields stay 0 and both queries stay unanswered.
- A new server message, `sixel`, carries one forwarded sequence with its window and position.
- The protocol version goes from 16 to 17. **BREAKING** for mixed client and server versions, as every protocol bump is.
- The README and the plugin guide document the option, the detection and the live-clients-only behaviour.

## Capabilities

### New Capabilities
- `window-sixel`: collecting a window program's sixel DCS sequences with their placement, forwarding them to attached clients whose terminal supports sixel, offering sixel to the program through Primary Device Attributes attribute `4`, drawing a forwarded sequence at its recorded position, startup detection, and the `window_sixel` option.

### Modified Capabilities
- `wire-protocol`: the handshake's current version becomes 17, the attach request gains a sixel flag, and the server messages gain `sixel`.
- `terminal-emulator`: the emulator collects sixel DCS sequences from processed output, with the cursor position at each sequence's start, and reports them in order, leaving the screen unchanged.
- `session-server`: the Primary Device Attributes reply gains attribute `4`, written only when an attached client's terminal supports sixel.

## Impact

- `crates/emulator/src/graphics.rs`, `crates/emulator/src/lib.rs`: scan processed output for complete sixel DCS sequences beside the kitty APC scan, buffer a split sequence, record the cursor position at each sequence's start, and `Vt100::take_sixel`.
- `crates/emulator/src/callbacks.rs`, `crates/emulator/tests/sixel.rs` (new): the Primary Device Attributes reply gains `4` under an emulator sixel-support flag, and the collection, split, placement and non-sixel-DCS scenarios.
- `crates/protocol/src/message.rs`, `crates/protocol/tests/messages.rs`: `ServerMessage::Sixel`, the attach sixel flag, version 17, and round trips.
- `crates/server/src/window.rs`, `crates/server/src/session.rs`, `crates/server/src/connection.rs`, `crates/server/src/registry.rs`: take the grid's sixel sequences, set the emulator's sixel-support flag from the attached clients, and broadcast forwarded sequences in order.
- `crates/server/tests/sixel.rs` (new), `crates/server/tests/scripting.rs`, `crates/server/tests/sync.rs`, `crates/server/tests/queries.rs`: the forwarding, the attach sixel field, the version 16 rejection, and the Primary Device Attributes reply with a sixel client.
- `crates/emulator/src/callbacks.rs`, `crates/server/src/window.rs`, `crates/server/src/session.rs`: the cell size replies, the PTY pixel fields and the session's cell size from the latest sixel client.
- `crates/client/src/lib.rs`: the probe parses attribute `4`, the attach flag, and writing a forwarded sixel at its position between frames.
- `crates/client/tests/sixel.rs` (new), `crates/client/tests/connect.rs`: the positioned write, the option off, and the attach field.
- `crates/test-support/src/lib.rs`: the attach sixel flag and handling `ServerMessage::Sixel`.
- `crates/lua/src/options.rs`, `crates/lua/src/defaults.lua`, `crates/lua/tests/options.rs`: the `window_sixel` option and its default.
- `README.md`, `docs/plugins.md`, `docs/internals/11-default-configs.md`: the option, detection and behaviour.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/emulator/src/graphics.rs
- crates/emulator/src/lib.rs
- crates/emulator/src/callbacks.rs
- crates/emulator/tests/sixel.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/src/window.rs
- crates/server/src/session.rs
- crates/server/src/connection.rs
- crates/server/src/registry.rs
- crates/server/tests/sixel.rs
- crates/server/tests/scripting.rs
- crates/server/tests/sync.rs
- crates/server/tests/queries.rs
- crates/client/src/lib.rs
- crates/client/tests/sixel.rs
- crates/client/tests/connect.rs
- crates/test-support/src/lib.rs
- crates/lua/src/options.rs
- crates/lua/src/defaults.lua
- crates/lua/tests/options.rs
- README.md
- docs/plugins.md
- docs/internals/11-default-configs.md
- openspec/changes/forward-sixel-graphics/
