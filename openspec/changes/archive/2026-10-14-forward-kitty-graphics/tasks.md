## 1. Emulator

- [x] 1.1 Add `crates/emulator/src/graphics.rs`: a scanner with a pending buffer that finds complete `ESC _ G ... ESC \` runs across `process` calls, pushes each to a `Vec<Vec<u8>>`, skips a non-`G` APC string to its `ESC \`, and discards a pending sequence past 16 MiB; and `pub fn query_reply(sequence: &[u8]) -> Option<Vec<u8>>` that returns `\x1b_Gi=<id>;OK\x1b\\` with the `i` value or `0` when the control holds `a=q`. Wire it into `crates/emulator/src/lib.rs` `Vt100::process` beside the `\x1bc` scan, add `Vt100::take_graphics`, and feed the whole chunk to `vte` unchanged so the screen is untouched. Verify `cargo build -p gband-emulator`
- [x] 1.2 Add `crates/emulator/tests/graphics.rs` with the terminal-emulator delta's scenarios: a transmit sequence collected with an empty screen, a sequence split across calls, collected sequences cleared when read, a non-graphics APC ignored, and `query_reply` returning the echo id for `a=q` and `None` otherwise. Verify `cargo test -p gband-emulator`

## 2. Protocol

- [x] 2.1 In `crates/protocol/src/message.rs`, add `graphics: bool` to `ClientMessage::Attach`, add `ServerMessage::Graphics { window, data }`, and raise `PROTOCOL_VERSION` to 16. In `crates/protocol/tests/messages.rs` add the attach round trip with graphics support, the graphics message round trip of window 2 with `\x1b_Ga=T;AAAA\x1b\\`, and the version 15 rejection. Verify `cargo test -p gband-protocol`

## 3. Server

- [x] 3.1 In `crates/server/src/connection.rs`, store the attach `graphics` flag with the client, and in `crates/server/src/session.rs` add a query for whether any attached client reported graphics support. Verify `cargo build -p gband-server`
- [x] 3.2 In `crates/server/src/window.rs`, after `grid.process` call `grid.take_graphics`; for a sequence that `query_reply` answers, push the reply as `Input::WriteBack` only when the session reports a supporting client; for any other sequence, broadcast `ServerMessage::Graphics { window, data }` onto each attached client's hub in order, through the session's event-broadcast path. Verify `cargo build -p gband-server`
- [x] 3.3 Add `crates/server/tests/graphics.rs` with the window-graphics delta's delivery and query scenarios: two supporting clients each receive the sequence, order is kept across two sequences, a sequence written before attach never arrives, another session does not receive it, a query is answered when a client supports graphics and silent otherwise, and a query without an id answers `i=0`. Verify `cargo test -p gband-server --test graphics`

## 4. Client

- [x] 4.1 In `crates/client/src/connect.rs` (or `crates/client/src/lib.rs` before the stdin decoder starts), after raw mode is set write the kitty graphics detection query and `\x1b[c`, read replies until the device attributes reply or a 1 second timeout, and set support from a graphics reply arriving first. Report support AND the `window_graphics` option in the attach request. Verify `cargo build -p gband-client`
- [x] 4.2 In `crates/client/src/lib.rs`, route `ServerMessage::Graphics { data, .. }` to a `Step::Write(data)` when `window_graphics` is true, in arrival order, and hold the option on the display state as `window_clipboard` is held. In `crates/client/src/render.rs`, make `apply_palette` skip the foreground of a cell whose `contents()` begins with U+10EEEE. Verify `cargo build -p gband-client`
- [x] 4.3 In `crates/test-support/src/lib.rs`, send the attach `graphics` flag from `TestClient` and handle `ServerMessage::Graphics` in `apply`, capturing forwarded sequences for assertions. Add `crates/client/tests/graphics.rs`: the forwarded write with the option on, no write with it off, and a placeholder cell with a 256-colour index below 16 drawn with that index under a colorscheme palette. Verify `cargo test -p gband-client --test graphics`

## 5. Lua

- [x] 5.1 Add the client option `window_graphics` (boolean, default `true`, the spec's description) to `crates/lua/src/options.rs`, set it in `crates/lua/src/defaults.lua` after `window_clipboard`, and add default, off and wrong-type tests to `crates/lua/tests/options.rs`, including it in the listed names. Verify `cargo test -p gband-lua --test options`

## 6. Documentation

- [x] 6.1 In `README.md`, add `window_graphics` to the client options table, and at the terminal-features section say that kitty graphics from a window program, such as yazi, chafa or an image preview, are forwarded to your terminal by default when it supports them, that placement uses Unicode placeholders so the program must carry the image id as a 24-bit RGB foreground or a 256-colour index of 16 or above, and that a client attaching after an image was sent shows it only once the program re-sends. Show `gband.set { window_graphics = false }`. Verify by reading it against the window-graphics spec
- [x] 6.2 In `docs/plugins.md`, add `window_graphics` to the client options list with its description, and note that the server answers the kitty graphics support query only when an attached client's terminal supports it. Verify every requirement of the window-graphics option appears there
- [x] 6.3 Add `gband.opt.window_graphics = true` to the quoted `defaults/init.lua` block of `docs/internals/11-default-configs.md`, beside `window_clipboard`. Verify `cargo test -p gband-lua --test tutorial`

## 7. Final checks

- [x] 7.1 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass
