## Why

A program inside a gband window cannot show an image in the terminal gband runs in. The kitty graphics protocol carries an image in APC sequences (`ESC _ G <control> ; <payload> ESC \`), which `vte` drops in `SosPmApcString` state, so the sequence never reaches a callback and gband discards it. yazi, chafa, timg, `kitten icat` and neovim image plugins all render through kitty graphics, and a user running one in a gband window sees nothing. tmux forwards the same protocol with its Unicode-placeholder passthrough.

## What Changes

- The emulator collects a window program's complete kitty graphics APC sequences while processing output, buffering a sequence split across writes, and reports them in order. The screen is unchanged; `vte` still drops them.
- The server forwards each collected sequence, in order, to every client attached to the session, through the same per-client queue events use. A sequence written while no client is attached is dropped. A client that attaches later does not receive sequences sent before it attached, so a placeholder cell shows blank until the program re-sends the image.
- Image placement uses Unicode placeholders. The program writes U+10EEEE cells carrying the image id in their foreground colour, which ride the existing grid snapshot and diff to each client unchanged. `apply_palette` stops remapping the foreground of a placeholder cell, so a colorscheme cannot corrupt an id carried as a 256-colour index below 16.
- The server answers a kitty graphics support query (`a=q`) with `OK`, echoing the query's image id, only when at least one attached client reported its terminal supports kitty graphics. It stays silent otherwise, so a program falls back to text.
- The client probes its outer terminal for kitty graphics support at startup with a graphics query followed by a primary device attributes request, reads the reply before the key decoder starts, and reports support in its attach request. The attach request gains a graphics flag. The protocol version goes from 15 to 16. **BREAKING** for mixed client and server versions, as every protocol bump is.
- A new client option, `window_graphics`, `true` by default, ANDs with the probe result: with it `false` the client reports no support, so the server forwards no sequence to it and answers no query for it. `gband.set { window_graphics = false }` at startup turns it off.
- A new server message, `graphics`, carries one forwarded sequence with its window.
- The README and the plugin guide document the option, the detection, and the placeholder requirement that the program carry the image id as a 24-bit RGB foreground or a 256-colour index of 16 or above.

## Capabilities

### New Capabilities
- `window-graphics`: collecting a window program's kitty graphics APC sequences, forwarding them to attached clients, answering the support query by the clients' terminals, Unicode-placeholder passthrough, startup detection, and the `window_graphics` option.

### Modified Capabilities
- `wire-protocol`: the handshake's current version becomes 16, the attach request gains a graphics flag, and the server messages gain `graphics`.
- `terminal-emulator`: the emulator collects kitty graphics APC sequences from processed output and reports them in order, leaving the screen unchanged.

## Impact

- `crates/emulator/src/graphics.rs` (new), `crates/emulator/src/lib.rs`: scan processed output for complete graphics APC sequences, buffer a split sequence, `Vt100::take_graphics`, and a classifier for the support query and its echo id.
- `crates/emulator/tests/graphics.rs` (new): collection, split sequences, and the query classifier.
- `crates/protocol/src/message.rs`, `crates/protocol/tests/messages.rs`: `ServerMessage::Graphics`, the attach graphics flag, version 16, and round trips.
- `crates/server/src/window.rs`, `crates/server/src/session.rs`, `crates/server/src/connection.rs`: take the grid's graphics, answer `a=q` when a client supports it, and broadcast forwarded sequences in order.
- `crates/server/tests/graphics.rs` (new): the forwarding, query and detection scenarios.
- `crates/client/src/connect.rs`, `crates/client/src/lib.rs`, `crates/client/src/render.rs`: the startup probe, the attach flag, writing a forwarded sequence to the terminal, and the placeholder palette skip.
- `crates/client/tests/graphics.rs` (new): the forwarded write, the option off, and the placeholder passthrough.
- `crates/test-support/src/lib.rs`: the attach graphics flag and handling `ServerMessage::Graphics`.
- `crates/lua/src/options.rs`, `crates/lua/src/defaults.lua`, `crates/lua/tests/options.rs`: the `window_graphics` option and its default.
- `README.md`, `docs/plugins.md`, `docs/internals/11-default-configs.md`: the option, detection and the placeholder requirement.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/emulator/src/graphics.rs
- crates/emulator/src/lib.rs
- crates/emulator/tests/graphics.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/src/window.rs
- crates/server/src/session.rs
- crates/server/src/connection.rs
- crates/server/tests/graphics.rs
- crates/client/src/connect.rs
- crates/client/src/lib.rs
- crates/client/src/render.rs
- crates/client/tests/graphics.rs
- crates/test-support/src/lib.rs
- crates/lua/src/options.rs
- crates/lua/src/defaults.lua
- crates/lua/tests/options.rs
- README.md
- docs/plugins.md
- docs/internals/11-default-configs.md
- openspec/changes/forward-kitty-graphics/
