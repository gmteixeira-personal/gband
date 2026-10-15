## 1. Protocol

- [x] 1.1 In `crates/protocol/src/message.rs`, add `ClientMessage::PasteImage { window: WindowId, mime: String, data: Vec<u8> }` and raise `PROTOCOL_VERSION` to 18. In `crates/protocol/tests/messages.rs`, update the version assertion to 18, add the paste image round trip of window 2 with the MIME type `image/png` and the bytes `\x89PNG\r\n`, and update the version mismatch test to a version 17 client meeting a version 18 server. Verify `cargo test -p gband-protocol`

## 2. Core

- [x] 2.1 In `crates/core/src/action.rs`, add `ClientAction::PasteImage`. Verify `cargo build -p gband-core`

## 3. Client

- [x] 3.1 Add `crates/client/src/clipboard.rs` reading an image from the clipboard: choose `wl-paste` when `WAYLAND_DISPLAY` is set, else `xclip` when `DISPLAY` is set, else `pbpaste`; list the offered MIME types, choose `image/png` when offered and else the first `image/*`, read that type's bytes, and return the bytes with the chosen MIME type or a typed error (reader missing, no image offered, reader failed). Put the subprocess runner behind a seam so a test can inject offered types and bytes without a real tool, and unit-test the MIME choice and the error mapping. Verify `cargo test -p gband-client --test clipboard` once 3.3 adds the file, else `cargo build -p gband-client`
- [x] 3.2 In `crates/client/src/lib.rs`, dispatch `ClientAction::PasteImage`: with a focused window, read the clipboard and return `Step::Send(ClientMessage::PasteImage { window, mime, data })`; on a reader error add one `report_error` line and return nothing; with no focused window return nothing. Verify `cargo build -p gband-client`
- [x] 3.3 In `crates/test-support/src/lib.rs`, add a `TestClient` helper that sends `ClientMessage::PasteImage`. Add `crates/client/tests/clipboard.rs` with the clipboard-image-paste delta's client scenarios through the injected reader: prefer PNG, any image when no PNG, reader not installed adds one error, no image adds one error, and nothing focused sends nothing. Verify `cargo test -p gband-client --test clipboard`

## 4. Server

- [x] 4.1 Route `ClientMessage::PasteImage` from `crates/server/src/connection.rs` and `crates/server/src/session.rs` to the window of the client's session, and in `crates/server/src/window.rs` write the image: resolve the base directory to the `clipboard_image_dir` option when set and else `std::env::temp_dir()`, join `gband-images`, create it, write the bytes to a new file named by a random component and the MIME extension (`.png` for `image/png`, `.jpg` for `image/jpeg`, else `.bin`), and inject the file's absolute path into the window with `encode_paste(path, window.modes())`. Verify `cargo build -p gband-server`
- [x] 4.2 Before each write, delete files in the `gband-images` directory whose modification time is more than 1 hour old, keeping the file about to be written, and ignore a delete failure. Verify `cargo build -p gband-server`
- [x] 4.3 Add `crates/server/tests/clipboard_images.rs` with the clipboard-image-paste delta's server scenarios: the absolute path is pasted into the window as a bracketed paste, a configured `clipboard_image_dir` is used with the right extension, a file older than 1 hour is pruned, and a recent file is kept. Update the version mismatch test to a version 17 client meeting a version 18 server in `crates/server/tests/sync.rs`. Verify `cargo test -p gband-server --test clipboard_images --test sync`

## 5. Lua

- [x] 5.1 In `crates/lua/src/actions.rs`, register `builtin("paste_image", Action::Client(ClientAction::PasteImage), "paste an image from the clipboard")` among the client actions, after `toggle_multi`. In `crates/lua/tests/config.rs`, add `paste_image` to the action-name list and assert its description. Verify `cargo test -p gband-lua --test config`
- [x] 5.2 In `crates/lua/src/options.rs`, add `clipboard_image_dir: Option<PathBuf>`, default `None`, with its patch field, merge, reset, `into_lua` and `OPTION_NAMES` entry, and add its type to `crates/lua/src/runtime/meta/gband.lua`. Add default, set-to-a-string and wrong-type tests to `crates/lua/tests/options.rs`, including it in the listed names. Verify `cargo test -p gband-lua --test options`
- [x] 5.3 Bind `prefix p` to `action.paste_image` with the description `paste an image from the clipboard` in `crates/lua/src/runtime/gband/keystyle/modal.lua` and `crates/lua/src/runtime/gband/keystyle/direct.lua`, after the `E` binding and before `D`, and update any keystyle or key-list test that lists the prefix bindings. Verify `cargo test -p gband-lua`

## 6. Documentation

- [x] 6.1 In `docs/plugins.md`, document the `paste_image` action, its `prefix p` binding, the `clipboard_image_dir` option, and the forwarding behavior: the client reads the local clipboard, the server writes a temp file on the program's host and pastes its absolute path, and it works over `gband -r <host>`. Verify by reading it against the clipboard-image-paste spec

## 7. Final checks

- [x] 7.1 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass
