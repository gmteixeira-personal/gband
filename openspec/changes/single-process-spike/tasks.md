## 1. Encoder in gband-core

- [ ] 1.1 Add `crates/core/src/input.rs`, exposed from `crates/core/src/lib.rs`, with `Key`, `KeyCode`, `Modifiers`, `Modes`, `encode_key` and `encode_paste` as the design describes, and verify that `cargo build -p gband-core` succeeds and `cargo tree -p gband-core -e normal --depth 1` still shows only `serde` and `thiserror`
- [ ] 1.2 Encode printable characters, control characters, editing keys and the Alt prefix, and verify with table-driven tests in `crates/core/tests/input_encoding.rs` that cover every scenario of the spec's encoding inputs, printable character, control character, editing key and Alt requirements
- [ ] 1.3 Encode cursor keys under both application cursor modes, and tilde and function keys, with the modifier parameter, and verify with table-driven tests covering every scenario of the spec's modifier parameter, cursor key and tilde and function key requirements
- [ ] 1.4 Encode pastes with the control-character filter and both bracketed paste modes, and verify with tests covering every scenario of the spec's paste requirement
- [ ] 1.5 Run `cargo test -p gband-core` and verify that it passes

## 2. Key translation in gband-client

- [ ] 2.1 Add `gband-core.workspace = true` to `crates/client/Cargo.toml`, and verify that `cargo tree -p gband-client -e normal --depth 1` lists `gband-core`
- [ ] 2.2 Add `crates/client/src/input.rs`, exposed from `crates/client/src/lib.rs`, with `key_from_event` translating `crossterm` key events as the design describes, and verify with tests in `crates/client/tests/key_translation.rs` covering every scenario of the spec's key event requirement

## 3. Spike

- [ ] 3.1 Add `portable-pty`, `vt100`, `tui-term`, `ratatui`, `crossterm` and `gband-core` as dev-dependencies of the root package, all with `.workspace = true`, and verify that `cargo tree -d -e normal` still shows `crossterm`, `ratatui` and `vt100` in one version each
- [ ] 3.2 Add `examples/spike.rs`, which starts logging as the `client` role, enters the full-screen terminal with bracketed paste enabled, spawns `$SHELL` in a PTY with `TERM=xterm-256color` and `COLORTERM=truecolor`, feeds the PTY output into a `vt100` parser with logging callbacks on a reader thread, and renders it with `PseudoTerminal`, and verify that `cargo run --example spike` shows a shell prompt and that `exit` returns to a restored outer terminal
- [ ] 3.3 Forward key events through `key_from_event` and `encode_key` with the screen's current modes, pastes through `encode_paste`, and resizes to the PTY and the parser, and verify inside the spike that typing a command with Enter runs it, that Ctrl+C interrupts `sleep 100`, that Up recalls the previous command, and that `tput cols` prints the new width after a resize
- [ ] 3.4 Place the outer terminal's real cursor at the screen's cursor position and hide it when the program hides it, and verify that the cursor follows typing at the shell prompt and disappears while `less` or nvim hides it
- [ ] 3.5 Run the spike with `GBAND_LOG=debug`, run `printf '\e[999z'` in it, and verify that the client log holds a `debug` event naming the unhandled CSI sequence

## 4. Findings

- [ ] 4.1 Run nvim in the spike and record in the nvim table of `design.md`'s Findings each of: startup time and any wait for query replies, `:checkhealth`, colours and truecolor, insert-mode cursor position and shape, Ctrl+`i` against Tab, Shift and Ctrl arrows, undercurl on a diagnostic, mouse, yank to the system clipboard, a paste of 1000 lines, and a resize, each with its result, its evidence and its follow-up
- [ ] 4.2 Run Claude Code in the spike and record in the Claude Code table of `design.md`'s Findings each of: startup and first render, typing and Enter, Shift+Enter for a newline, Escape to interrupt, Ctrl+C, arrow-key history, a large paste, colours, long scrolling output, a resize, the notification or bell when it waits, and exit, each with its result, its evidence and its follow-up
- [ ] 4.3 List every follow-up change the two tables name under Follow-up changes, each with one line on its scope, and verify that every row whose result is not `works` names a follow-up change or states why it needs none

## 5. Finish

- [ ] 5.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check`, and verify that all three exit with status 0
- [ ] 5.2 Check every new or changed Rust file against the repository's coding rules, which forbid narrative comments, and verify that no comment remains except one explaining an upstream API restriction
- [ ] 5.3 Run `cargo run -- --help` and verify that it still lists only the `server` and `attach` subcommands
