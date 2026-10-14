## Context

The server feeds each window's program output into a `Grid` (`crates/emulator`). vt100 already calls `Callbacks::copy_to_clipboard` for OSC 52, and the callback decodes it into `Record::Clipboard`. Records exist only when `Vt100::record` was called, which only the plugin-testing harness does, so the server drops every write today.

A connection (`crates/server/src/connection.rs`) sends state by comparing generations: it holds a `Sent` per client and sends a window's update or name only when its generation moved. Nothing in that loop is an event queue, so a burst of output collapses into one update.

The client already writes OSC 52 in two places: `gband.clipboard` (`crates/lua/src/bridge.rs`) and the mouse selection (`crates/client/src/mouse.rs`). Both go through `clipboard_sequence`, which caps the text at 1 MiB and writes between frames.

## Goals / Non-Goals

**Goals:**
- A copy made by a program in a window reaches each attached client's terminal clipboard with no configuration.
- Lua can turn the copy off, and can copy selectively from a `WindowClipboard` handler.

**Non-Goals:**
- Reading the clipboard: OSC 52 queries stay unanswered.
- The primary selection: a write naming only `p`, `s` or a cut buffer is ignored.
- A server-side option. The risk is to the user's local clipboard, so the choice belongs to the client that owns it.
- Replaying a write to a client that attaches later.

## Decisions

### Keep the latest write outside the test records
`Callbacks` gains a `clipboard: Option<String>` that `copy_to_clipboard` always sets, latest write wins, and `Vt100::take_clipboard` drains it. `Record::Clipboard` is still pushed when records are on, so the harness keeps working.

Alternative: turn records on in the server and drain them. Rejected: records also collect bells and notifications, which the server would then drain and discard on every read.

The selection test moves from `ty == b"c"` to "empty or holds `c`". Empty means xterm's default selection, which terminals map to the clipboard. The decoded length is checked against 1 MiB before the text is kept. vte with its `std` feature buffers an OSC without a limit, so the cap applies after parsing, the same as today.

### One latest write per session, ordered by a session counter
The session hands each window a shared `AtomicU64` counter, the same way it shares the `changed` watch. `Window::process` takes the grid's clipboard write after `grid.process`, stamps it with the next counter value and stores `(stamp, text)` on the window. The connection's `Sent` gains `clipboard: u64`, set to the counter's value when the client attaches. After the window names, the connection finds the window holding the highest stamp above `sent.clipboard`, sends `ServerMessage::Clipboard { window, text }` and stores the stamp.

Alternatives:
- A queue of every write per client. Rejected: the generation loop has no queue, and only the last copy matters for a clipboard.
- A per-window latest write sent in layout order. Rejected: two windows writing between deliveries would reach the client in layout order, so the older copy could win.

`Window::replace` builds a fresh grid from stored output and must not take its clipboard write, so a reload never copies again.

### Event first, then the option's write
The client turns `ServerMessage::Clipboard` into `Event::WindowClipboard { window, text }` in `Controls::bridge`, beside `ServerEvent`. After the handlers ran, it pushes a write step of `clipboard_sequence(text)` when `options.window_clipboard` is true. Handlers run in either case, so a configuration with the option off can still copy from chosen windows with `gband.clipboard`.

Alternatives (the user chose option plus event):
- Event alone, with the default configuration's handler copying. Turning it off would mean clearing a named handler group, which is harder to find than an option.
- Option alone. No way to copy from one window and not another.

### Protocol version 15
A new `ServerMessage` variant changes the encoding, so `PROTOCOL_VERSION` goes to 15 and `crates/protocol/tests/messages.rs` gains a round trip.

## Risks / Trade-offs

- [Any output in a window can replace the local clipboard: `cat` of an untrusted file, or a host reached through nested ssh] → The README and the plugin guide name the risk next to the option and show `gband.set { window_clipboard = false }`. The user chose on by default.
- [A burst of writes delivers only the last one, so `WindowClipboard` runs once for several writes] → The specs state it. A clipboard holds one value, so the earlier writes would be overwritten anyway.
- [A client of protocol 14 cannot attach to a server of protocol 15] → The handshake rejects it with the version, as for every earlier bump.
- [Several attached clients all receive the copy, including one on another machine] → Matches tmux `set-clipboard on`. A client that does not want it sets the option to `false`.

## Migration Plan

None beyond the protocol bump. An old server keeps running until killed; a new client reports the version mismatch when it attaches to it.
