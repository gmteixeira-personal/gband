## 1. Emulator

- [ ] 1.1 In `crates/emulator/src/graphics.rs`, add sixel scanning beside the kitty APC scan: find complete `ESC P <params> q <payload> ESC \` runs across `process` calls where `<params>` is only digits and `;`, skip a non-sixel device control string (an intermediate such as `$` before the final byte) to its `ESC \`, and discard a pending sixel past 16 MiB. In `crates/emulator/src/lib.rs`, feed `process` in segments bounded by each sixel's start, read the grid cursor (1-based from the top-left) at that start, collect `(row, col, bytes)` into a `Vec`, add `Vt100::take_sixel`, and feed the whole chunk to `vte` unchanged so the screen is untouched. Verify `cargo build -p gband-emulator`
- [ ] 1.2 In `crates/emulator/src/lib.rs`, add a `sixel_supported` flag on `Vt100` with a setter, default false, and in `crates/emulator/src/callbacks.rs` answer Primary Device Attributes with `\x1b[?62;4;22c` when the flag is set and `\x1b[?62;22c` otherwise. Verify `cargo build -p gband-emulator`
- [ ] 1.3 Add `crates/emulator/tests/sixel.rs` with the terminal-emulator delta's scenarios: a sixel collected with its cursor position, a sequence split across calls, collected sequences cleared when read, a non-sixel device control string ignored, and the Primary Device Attributes reply with and without the sixel flag. Verify `cargo test -p gband-emulator`

## 2. Protocol

- [ ] 2.1 In `crates/protocol/src/message.rs`, add `sixel: bool` to `ClientMessage::Attach`, add `ServerMessage::Sixel { window, col, row, data }`, and raise `PROTOCOL_VERSION` to 17. In `crates/protocol/tests/messages.rs` add the attach round trip with sixel support, the sixel message round trip of window 2 at row 3 column 5 with `\x1bPq#0~~\x1b\\`, and the version 16 rejection. Verify `cargo test -p gband-protocol`

## 3. Server

- [ ] 3.1 In `crates/server/src/connection.rs`, store the attach `sixel` flag with the client, and in `crates/server/src/session.rs` and `crates/server/src/registry.rs` add a query for whether any attached client supports sixel and push it to each window grid's `sixel_supported` flag, updated on attach and detach. Verify `cargo build -p gband-server`
- [ ] 3.2 In `crates/server/src/window.rs`, after `grid.process` call `grid.take_sixel`; for each sequence broadcast `ServerMessage::Sixel { window, col, row, data }` onto the hub of each attached client that supports sixel, in order, through the session's event-broadcast path. Verify `cargo build -p gband-server`
- [ ] 3.3 Add `crates/server/tests/sixel.rs` with the window-sixel delta's delivery scenarios: two supporting clients each receive the sequence with its position, order is kept across two sequences, a non-supporting client receives none, a sequence written before attach never arrives, and another session does not receive it. Add the Primary Device Attributes reply with a sixel client to `crates/server/tests/queries.rs`. Verify `cargo test -p gband-server --test sixel`
- [ ] 3.4 Update the attach `sixel` field in `crates/server/tests/scripting.rs` and the version 16 rejection in `crates/server/tests/sync.rs`. Verify `cargo test -p gband-server`

## 4. Client

- [ ] 4.1 In `crates/client/src/lib.rs`, extend `graphics_probe` and `attributes_reply` to report sixel support when the Primary Device Attributes reply's parameters hold `4`, from the one probe with no second query, and report sixel support AND the `window_sixel` option in the attach request beside the graphics flag. Verify `cargo build -p gband-client`
- [ ] 4.2 In `crates/client/src/lib.rs`, route `ServerMessage::Sixel { window, col, row, data }` to a positioned write between frames, after the frame's cells are drawn, when `window_sixel` is true: save the cursor, move to the window's on-screen origin plus `(row, col)`, write the bytes, restore the cursor, and write nothing when the anchor cell lies outside the window's drawn area. Hold the option on the display state as `window_graphics` is held. Verify `cargo build -p gband-client`
- [ ] 4.3 In `crates/test-support/src/lib.rs`, send the attach `sixel` flag from `TestClient` and handle `ServerMessage::Sixel` in `apply`, capturing forwarded sequences with their positions for assertions. Add `crates/client/tests/sixel.rs`: the positioned write with the option on, no write with it off, and no write when the anchor is outside the window. Update the attach `sixel` field in `crates/client/tests/connect.rs`. Verify `cargo test -p gband-client --test sixel`

## 5. Lua

- [ ] 5.1 Add the client option `window_sixel` (boolean, default `true`, the spec's description) to `crates/lua/src/options.rs`, set it in `crates/lua/src/defaults.lua` after `window_graphics`, and add default, off and wrong-type tests to `crates/lua/tests/options.rs`, including it in the listed names. Verify `cargo test -p gband-lua --test options`

## 6. Documentation

- [ ] 6.1 In `README.md`, add `window_sixel` to the client options table, and at the terminal-features section say that sixel images from a window program, such as yazi in Windows Terminal, are forwarded to your terminal by default when it supports them, that a program is offered sixel only when an attached client's terminal supports it, and that a client attaching after an image was drawn, or a scroll, shows it only once the program re-emits. Show `gband.set { window_sixel = false }`. Verify by reading it against the window-sixel spec
- [ ] 6.2 In `docs/plugins.md`, add `window_sixel` to the client options list with its description, and note that the server adds Primary Device Attributes attribute `4` only when an attached client's terminal supports sixel. Verify every requirement of the window-sixel option appears there
- [ ] 6.3 Add `gband.opt.window_sixel = true` to the quoted `defaults/init.lua` block of `docs/internals/11-default-configs.md`, beside `window_graphics`. Verify `cargo test -p gband-lua --test tutorial`

## 7. Final checks

- [ ] 7.1 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass
