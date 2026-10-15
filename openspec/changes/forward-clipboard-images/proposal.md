## Why

Claude Code and similar programs read a pasted image from the OS clipboard with `wl-paste`, `xclip` or `pbpaste`, not from the terminal byte stream. A program running in a gband window over SSH reads the remote host's clipboard, which is empty, so image paste fails. Pasting a locally captured screenshot into a window program should work whether gband runs locally or with `gband -r <host>`.

## What Changes

- Add a client action `paste_image` that reads an image from the local clipboard and forwards it to the focused window.
- The client reads the clipboard with `wl-paste` on Wayland, `xclip` on X11, and `pbpaste` on macOS, preferring the `image/png` MIME type and else any `image/*` type.
- Add a new wire-protocol client message that carries the image bytes and MIME type from client to server, and raise `PROTOCOL_VERSION` from 17 to 18.
- The server writes the bytes to a file under the OS temp directory in a `gband-images` subdirectory, prunes files there older than 1 hour, and injects the file's absolute path into the window's PTY as a bracketed paste.
- Bind `paste_image` to `prefix` then `p` in the modal and direct presets, and add the server option `clipboard_image_dir` that overrides the base directory.
- The feature covers local gband and `gband -r <host>`, because the client runs next to the real clipboard in both and the server writes the file on the host where the program runs.

## Capabilities

### New Capabilities
- `clipboard-image-paste`: the client reads a local clipboard image on an action, forwards it to the server, and the server writes a temp file and pastes its path into the focused window.

### Modified Capabilities
- `wire-protocol`: add a client message carrying clipboard image bytes and a MIME type from client to server, and raise the protocol version to 18.
- `actions`: add the built-in client action `paste_image`.
- `client-attach`: bind `prefix` then `p` to `paste_image` in the modal and direct key styles.

## Non-goals

- Pasting an image when the gband client itself runs on a host reached by a prior manual `ssh`. The client then reads that host's clipboard, not the workstation's; use `gband -r <host>`, which keeps the client local.
- Carrying image bytes through the terminal byte stream or OSC 52. OSC 52 is text and the window-clipboard capability caps it at 1 MiB.
- Triggering the program's own clipboard image chip. The program receives a file path and reads it as a file.

## Impact

- New `ClientMessage::PasteImage` in the wire-protocol capability, encoded and decoded in `crates/protocol/src/message.rs`, with the version bump to 18.
- Client: a new `ClientAction::PasteImage` in `crates/core/src/action.rs`, a clipboard reader in `crates/client/src/clipboard.rs` calling `wl-paste`, `xclip` or `pbpaste` as a subprocess, and dispatch in `crates/client/src/lib.rs`.
- Server: temp file write, pruning and paste injection in `crates/server/src/window.rs`, reusing `encode_paste` in `crates/core/src/input.rs`.
- Lua: the built-in action `paste_image` in `crates/lua/src/actions.rs`, the option `clipboard_image_dir` in `crates/lua/src/options.rs`, and the `prefix p` binding in the modal and direct presets.
- New runtime dependency on a clipboard reader tool on the client host; its absence SHALL surface a clear error in the client's error list rather than a silent no-op.
- No README change: the in-flight `macos-support` change holds `README.md`, so this change documents the feature in `docs/plugins.md` only.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/core/src/action.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/client/src/lib.rs
- crates/client/src/clipboard.rs
- crates/client/tests/clipboard.rs
- crates/server/src/window.rs
- crates/server/src/session.rs
- crates/server/src/connection.rs
- crates/server/tests/clipboard_images.rs
- crates/server/tests/sync.rs
- crates/test-support/src/lib.rs
- crates/lua/src/actions.rs
- crates/lua/src/options.rs
- crates/lua/src/runtime/gband/keystyle/modal.lua
- crates/lua/src/runtime/gband/keystyle/direct.lua
- crates/lua/src/runtime/meta/gband.lua
- crates/lua/tests/options.rs
- crates/lua/tests/config.rs
- docs/plugins.md
- openspec/changes/forward-clipboard-images/
