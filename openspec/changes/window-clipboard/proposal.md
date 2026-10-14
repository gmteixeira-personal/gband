## Why

A program inside a gband window cannot set the clipboard of the terminal gband runs in. Claude Code, neovim and other programs copy with OSC 52 (`ESC ] 52 ; c ; <base64> BEL`). The server's emulator decodes that sequence today and then drops it, so a copy made by an agent in a window never reaches the user's clipboard. tmux forwards the same sequence with `set-clipboard on`, and gband users working with agents expect the same.

## What Changes

- The server keeps the latest OSC 52 clipboard write of each session: the writing window and the decoded text, at most 1 MiB.
- A new server message, `clipboard`, carries that write to every client attached to the session. The protocol version goes from 14 to 15. **BREAKING** for mixed client and server versions, as every protocol bump is.
- The client emits a new built-in event, `WindowClipboard`, with `window` and `text`, so Lua can react to or filter the write.
- A new client option, `window_clipboard`, `true` by default, makes the client copy the text to its own terminal's clipboard with the same sequence `gband.clipboard` writes. `gband.set { window_clipboard = false }` stops it, and a `WindowClipboard` handler can then copy selectively with `gband.clipboard`.
- OSC 52 queries (`?`) are never answered, so a program cannot read the user's clipboard through gband.
- The README and the plugin guide document the option, the event and the risk: with the option on, any output written to a window, such as `cat` of an untrusted file or a remote host reached through nested ssh, can replace the local clipboard.

## Capabilities

### New Capabilities
- `window-clipboard`: forwarding a window program's OSC 52 clipboard write from the server to the attached clients, the `window_clipboard` option and the `WindowClipboard` event.

### Modified Capabilities
- `wire-protocol`: the handshake's current version becomes 15, and the server messages gain `clipboard`.
- `lua-events`: the built-in events table gains `WindowClipboard`.

## Impact

- `crates/emulator/src/callbacks.rs`, `crates/emulator/src/lib.rs`: keep the latest clipboard write outside the test-only records, and accept an empty selection field as well as one holding `c`.
- `crates/server/src/window.rs`, `crates/server/src/connection.rs`: hand the write to the session and send it to each attached client.
- `crates/protocol/src/message.rs`, `crates/protocol/tests/messages.rs`: the new message and version 15.
- `crates/client/src/lib.rs`, `crates/lua/src/events.rs`, `crates/lua/src/options.rs`, `crates/lua/src/defaults.lua`: the event, the option and its default.
- Tests in `crates/lua/tests/options.rs`, `crates/client/tests/events.rs` and a new `tests/lua/window_clipboard_spec.lua`.
- Docs: `README.md`, `docs/plugins.md`, `docs/internals/11-default-configs.md`.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/emulator/src/callbacks.rs
- crates/emulator/src/lib.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/src/window.rs
- crates/server/src/connection.rs
- crates/server/src/session.rs
- crates/server/tests/clipboard.rs
- crates/lua/src/options.rs
- crates/lua/src/defaults.lua
- crates/lua/src/events.rs
- crates/lua/tests/options.rs
- crates/client/src/lib.rs
- crates/client/tests/events.rs
- tests/lua/window_clipboard_spec.lua
- README.md
- docs/plugins.md
- docs/internals/11-default-configs.md
- openspec/changes/window-clipboard/
