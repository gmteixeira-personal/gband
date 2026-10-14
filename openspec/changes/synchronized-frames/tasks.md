## 1. Synchronized frames

- [x] 1.1 Add the public `synchronized` function to `crates/client/src/lib.rs`, generic over `W: Write`: queue `BeginSynchronizedUpdate` and `Hide`, call `terminal.draw` with the callback, then queue `EndSynchronizedUpdate` and flush on both the success and the error path, returning the draw's error first. Make `draw` call it. Verify `cargo build -p gband-client`
- [x] 1.2 Make `TerminalGuard::drop` write `EndSynchronizedUpdate` before turning mouse reporting off. Verify by reading `drop` against the client-attach delta's "Leaving the client" requirement

## 2. Tests

- [x] 2.1 Add `crates/client/tests/frames.rs` with a `Terminal` over `CrosstermBackend<Vec<u8>>` and a fixed 20×12 viewport. Cover "Frame between the markers", "Cursor hidden while cells are written" (cells on rows 2 and 10, cursor on row 5, last bytes `\e[?25h`, a `MoveTo` to row 5, `\e[?2026l`), and "Cursor stays hidden" (no frame cursor). Verify `cargo test -p gband-client --test frames`
- [x] 2.2 Add "Failed frame still ends" to `frames.rs` with a writer that fails after 64 bytes on its first error and accepts bytes again afterwards, and assert the recorded bytes end with `\e[?2026l` and the call returns the error. Verify `cargo test -p gband-client --test frames`

## 3. Manual check

- [x] 3.1 Build with `cargo build --release`, attach in Ghostty, run `while :; do date; sleep 0.05; done` in one window and focus a shell in another. Verify the focused shell's cursor stays on its prompt and does not flash on the clock's cells

## 4. Final checks

- [x] 4.1 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass
