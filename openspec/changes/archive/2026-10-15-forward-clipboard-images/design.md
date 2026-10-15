## Context

A window program reads a pasted image from the OS clipboard, not from the terminal byte stream: Claude Code and others shell out to `wl-paste`, `xclip` or `pbpaste`. A program in a gband window reads the clipboard of the host it runs on. Over `gband -r <host>` that host is the remote one, whose clipboard is empty, so image paste fails. The client, by contrast, always runs next to the real clipboard, local or `-r`.

Three facts shape the change:

- `window-clipboard` already carries clipboard text from a window program to the client with OSC 52, capped at 1 MiB and text only. It is the wrong direction and the wrong payload for an image. This change adds the opposite direction, client to server, with raw bytes.
- The client already runs subprocesses on its own host: `gband.open` (`crates/lua/src/...` through the `local-actions` capability) starts `xdg-open` or `open` without a shell. A clipboard reader is the same shape, except it must wait for the reader's output.
- The server already turns a client `paste` message into bracketed-paste bytes for a window's PTY with `encode_paste` (`crates/core/src/input.rs:243`), honouring the window's modes. Pasting a file path reuses that path exactly.

The client has an error list: `Display::report_error` (`crates/client/src/lib.rs`) pushes a client-side error that the `error-list` capability and the `gband.errors` plugin show. A missing clipboard tool or an empty clipboard surfaces there.

## Goals / Non-Goals

**Goals:**
- A locally captured screenshot pastes into the focused window's program as a file path it can read, whether gband runs locally or with `gband -r <host>`.
- The program receives a file path through a bracketed paste, so it reads a file, as it does when a terminal pastes an image path.
- A missing clipboard reader, or a clipboard holding no image, surfaces a clear error in the client's error list, not a silent no-op.
- One default binding, `prefix` then `p`, in the modal and direct key styles.

**Non-Goals:**
- Pasting an image when the gband client itself runs on a host reached by a prior manual `ssh`. The client then reads that host's clipboard; use `gband -r <host>`, which keeps the client local.
- Carrying image bytes through the terminal byte stream or OSC 52. OSC 52 is text and the window-clipboard capability caps it at 1 MiB.
- Triggering the program's own clipboard image chip. The program receives a file path and reads it as a file.
- Reading an image with `xsel` on X11. `xsel` has no image MIME target, so `xclip` is the X11 reader.
- Cleaning up a pasted file the moment the program has read it. The program reads the path at an unknown later time, so the file persists and old files are pruned instead.

## Decisions

### A `paste_image` client action reads the clipboard and forwards one image
`crates/core/src/action.rs` gains `ClientAction::PasteImage`, registered in `crates/lua/src/actions.rs` as `builtin("paste_image", Action::Client(ClientAction::PasteImage), "paste an image from the clipboard")`, kind client. Dispatch in `crates/client/src/lib.rs` resolves the focused window, reads the clipboard, and returns `Step::Send(ClientMessage::PasteImage { window, mime, data })` on success. With no focused window it does nothing, as any window-targeted binding does.

Alternative: a session action resolved on the server. Rejected: the clipboard lives on the client's host, so the read must run in the client, which makes it a client action.

### Read the clipboard with the platform's image reader, prefer `image/png`
A new `crates/client/src/clipboard.rs` picks the reader from the environment: `wl-paste` when `WAYLAND_DISPLAY` is set, else `xclip` when `DISPLAY` is set, else `pbpaste` on macOS. It lists the clipboard's offered MIME types (`wl-paste --list-types`, `xclip -selection clipboard -t TARGETS -o`), chooses `image/png` when offered and else the first `image/*` type, and reads the bytes for that type. It returns the bytes and the chosen MIME type, or a typed error: the reader is not installed, the clipboard offers no image, or the reader failed. Each error becomes one `report_error` line.

Alternative: read the clipboard on every `paste` and detect an image. Rejected: text paste already works through the window program's own reader; a separate action keeps image paste explicit and bindable.

### Forward with `ClientMessage::PasteImage`, protocol 18
`crates/protocol/src/message.rs` gains `ClientMessage::PasteImage { window: WindowId, mime: String, data: Vec<u8> }` and raises `PROTOCOL_VERSION` to 18. `crates/protocol/tests/messages.rs` gains the round trip and the version 17 rejection. The payload is raw bytes, so no 1 MiB OSC 52 cap applies; the framing's own announced size bounds it.

### The server writes a temp file, prunes old ones, and pastes its absolute path
On `PasteImage` for a window of the client's session, `crates/server/src/window.rs` resolves the base directory: the `clipboard_image_dir` option when set, else `std::env::temp_dir()`, joined with `gband-images`, created if absent. It deletes files in that directory whose modification time is older than 1 hour, writes the bytes to a new file named by a random component and the MIME type's extension (`image/png` to `.png`, `image/jpeg` to `.jpg`, else `.bin`), and injects the file's absolute path into the window's PTY with `encode_paste(path, window.modes())`. The program reads a bracketed-pasted absolute path.

The file is written on the host where the server and the program run, so `gband -r <host>` writes it on the remote host and the program reads it there. Pruning on each paste bounds the directory without tracking each file's lifetime, which is unknown because the program reads the path whenever it chooses.

Alternative: write under the window's working directory. Rejected: it drops image files into the user's project tree; Claude Code reads an absolute path, so the OS temp directory is enough.

Alternative: delete the file after a fixed delay. Rejected: the program may read the path long after the paste, so a delay risks deleting the file first.

### Bind `prefix` then `p` in the modal and direct presets
`p` is unbound in both presets and reads as paste. `crates/lua/src/runtime/gband/keystyle/modal.lua` and `direct.lua` bind `prefix p` to `action.paste_image` with the description `paste an image from the clipboard`, after the `E` binding and before `D`. The action is not directional, so it takes no arrow binding.

### `clipboard_image_dir` is a server option
`crates/lua/src/options.rs` gains `clipboard_image_dir: Option<PathBuf>`, default `None`, which the server reads when it writes the file. `None` means the OS temp directory. It joins `OPTION_NAMES`, the patch, the merge, the reset and the `into_lua` listing, like the other options, and `crates/lua/src/runtime/meta/gband.lua` gains its type. It is a server option like `default_column_width`, so it is read from the server's configuration, which in `-r` is the remote host's.

## Risks / Trade-offs

- [The clipboard reader tool is not installed on the client's host] → The action reports a clear error naming the missing tool in the client's error list and sends nothing. No silent no-op.
- [The clipboard holds text, not an image] → The action reports that the clipboard holds no image and sends nothing.
- [A client of protocol 17 attaches to a server of protocol 18] → The handshake rejects it with version 18, as every earlier bump.
- [The program reads the pasted path after the file was pruned] → A paste prunes only files older than 1 hour, far longer than the gap between a paste and a program reading it; a program that waits over an hour sees the file gone.
- [Two pastes in the same second collide on a filename] → The random component, not the timestamp, names the file, so two pastes do not collide.
- [The clipboard image is large] → The bytes travel in one message bounded by the framing's announced size; no 1 MiB cap applies, unlike OSC 52 clipboard text.

## Migration Plan

None beyond the protocol bump. An old server keeps running until killed; a new client reports the version mismatch when it attaches to it.
