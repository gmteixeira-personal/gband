## 1. Decoder

- [x] 1.1 Add `crates/core/src/terminal_input.rs` with `TerminalInput` and `Decoder` (`push`, `holds`, `flush`), exported from `crates/core/src/lib.rs`. Decode the key table of "Keys from the terminal's bytes", SGR and default mouse reports, focus reports and bracketed paste. Verify with `cargo test -p gband-core` on the scenarios of that requirement.
- [x] 1.2 Add the ESC prefix state and the incomplete-tail handling of "Incomplete input from the terminal" and "ESC before another input". Verify with tests for every scenario of both requirements in `crates/core/tests/terminal_input.rs`.
- [x] 1.3 Add tests that decode `encode_key` output back to the same key for every key code and modifier combination, except the documented control-byte aliases (Ctrl+Backspace, Ctrl+`i`, Ctrl+`m`, Ctrl+`[`), and `encode_mouse` SGR and default output back to the same event. Verify that `cargo test -p gband-core` passes.
- [x] 1.4 Add split and merge tests. Every encoded key, mouse report, focus report and paste must decode the same when pushed one byte at a time, split at every byte boundary, and merged in one push with a preceding Escape. Verify that `cargo test -p gband-core` passes.
- [x] 1.5 Port the key and escape vectors from zellij's vendored `zellij-utils/src/vendored/termwiz/input.rs` tests, herdr's `src/raw_input.rs` tests and crossterm 0.29's `src/event/sys/unix/parse.rs` tests, limited to inputs gband's `Key` and `MouseEvent` can hold. Port behaviour only, no code. Verify that `cargo test -p gband-core` passes.

## 2. Client

- [x] 2.1 Replace `spawn_events` in `crates/client/src/lib.rs` with a thread that sends raw standard input chunks. Feed them through the decoder into `controls.press`, `controls.mouse` and `controls.paste`. Verify with `cargo build` and the existing client tests.
- [x] 2.2 Add the 25 ms flush branch to the client loop, enabled only while the decoder holds bytes. Verify that a lone Escape in `cargo test` end-to-end tests still leaves the prefix table.
- [x] 2.3 Take resizes from SIGWINCH with `tokio::signal::unix` and `crossterm::terminal::size`. Verify that `g.resize` specs in `tests/lua/` and the resize end-to-end tests pass.
- [x] 2.4 Delete `crates/client/src/input.rs` and `crates/client/tests/key_translation.rs`, after confirming each of their cases is covered by a decoder test from section 1. Verify that `cargo test --workspace` passes and that `rg 'event::read' crates/client/src` finds nothing.

## 3. Test channel

- [x] 3.1 Change `ToProcess::Settle` in `crates/protocol/src/test.rs` from `markers` to `input`, a byte count. Make the client answer an input settle once it has read and handled that many bytes and its decoder holds nothing (`crates/client/src/channel.rs`). Remove the focus-marker counting. Verify with `cargo test -p gband-protocol` and `-p gband-client`.
- [x] 3.2 Count every byte the runner's terminal writes to the client's input, replies included, in `crates/harness/src/terminal.rs`. Send that count in settle's first step in `crates/harness/src/case.rs` and `crates/harness/src/channel.rs`, dropping the `\x1b[I` write and its 250 ms nudge loop. Verify that `cargo test -p gband-harness` passes.
- [x] 3.3 Replace `drained()` in `Case::keys` with the same input acknowledgement after a key whose bytes end in ESC. Verify the "Escape then another key" scenario.

## 4. Regression and docs

- [x] 4.1 Add `tests/lua/escape_spec.lua`. One case presses `escape` and settles 100 times. Another runs `cat -v` in a window, presses `escape up enter`, and checks that the window shows `^[^[[A` and no `[I`. Verify that `gband test` passes it 10 runs in a row.
- [x] 4.2 In `tests/lua/sidebar_spec.lua`, where a case presses `enter` to leave a mode only to avoid this bug, press `escape` instead. Verify that the spec passes 10 runs in a row.
- [x] 4.3 Update `docs/testing.md` where it describes settle's first step or `g.keys` timing. Verify that `rg -n 'focus' docs/testing.md` names no settle marker.
- [x] 4.4 Run `cargo clippy --workspace --all-targets`, `cargo test --workspace` and every Lua spec under `tests/lua/` and `examples/plugins/*/tests/`. Verify that all pass.
