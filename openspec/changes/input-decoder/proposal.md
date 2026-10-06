## Why

gband reads the user's terminal through crossterm 0.29, which decides what a lone ESC means from where `read()` happens to split the input. When Escape and the next escape sequence arrive in one read, crossterm emits Escape, drops the second ESC and passes the rest of that sequence on as typed characters (`\x1b\x1b[A` reads as Escape, `[`, `A`). When a sequence is split after its ESC, crossterm emits a false Escape and the rest is typed (crossterm issues #854 and #993). With any-motion mouse tracking on, mouse reports can therefore reach the focused program as text such as `[<35;10;5M`.

The test harness shows the same bug: after a key that ends in ESC it waits for the pty input queue to empty, but the kernel queues pty input asynchronously, so the queue reads empty right after every write and the wait never waits. Escape then merges with the settle marker `\x1b[I`, the marker is lost and the settle times out after 10 seconds. A loop of Escape and settle on `dev` hung in 6 of 6 runs, about once per 11 Escapes. zellij and herdr both replaced crossterm's input reader on Unix for the same reasons.

## What Changes

- gband decodes the terminal's input bytes itself, in place of `crossterm::event::read`. The decoder sends every complete key, mouse report, focus report and paste as soon as it reads it. It holds an incomplete sequence until more bytes arrive. If no byte arrives within 25 ms, it resolves what it holds, so a lone ESC becomes Escape.
- ESC before a key reads as Alt with that key, so `\x1b\x1b[A` is Alt+Up, as in zellij. ESC before a mouse report, a focus report or a paste start reads as Escape followed by that event. The decoder never drops input and never types the bytes of a sequence as text.
- The decoder recognises the sequences that common terminals send for gband's keys: xterm, rxvt and the Linux console variants of Home, End and F1 to F5. Their coverage comes from zellij's and herdr's parsers and from their test cases.
- The client reads raw standard input on its own thread and takes terminal resizes from SIGWINCH. crossterm stays for raw mode, output commands and the terminal size.
- The test runner counts every byte it writes to the client's terminal. The client reports how many input bytes it has read and whether it holds an incomplete sequence. Settle uses this count in place of the focus-in marker it writes today. `g.keys` waits after a key ending in ESC until the client has read that key, so the next key cannot merge with it.
- Key releases, Super, Hyper, Meta, Caps Lock and media keys can no longer reach gband: legacy terminal input cannot report them, and gband does not enable the kitty keyboard protocol.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `input-encoding`: "Key events from the terminal" is removed and replaced by "Keys from the terminal's bytes", which defines keys by the bytes the terminal sends instead of by the events crossterm reports. New requirements cover ESC timing, the ESC prefix rule, split and merged sequences, focus reports and pastes from the terminal.
- `test-channel`: a new requirement lets the runner wait until the client has read a given number of input bytes and holds no incomplete sequence. Settle's first step uses it.
- `plugin-testing`: `g.keys` waits after each key that ends in ESC until the client has read it.

## Impact

- `crates/core/src/input.rs`, or a new module beside it: the decoder.
- `crates/client/src/lib.rs`, `crates/client/src/input.rs`: the reader thread, the flush timer, SIGWINCH, and converting decoded events in place of crossterm events.
- `crates/client/src/channel.rs`, `crates/protocol/src/test.rs`, `crates/harness/src/case.rs`, `crates/harness/src/channel.rs`, `crates/harness/src/terminal.rs`: the input byte count and its acknowledgement.
- Tests: `crates/core/tests/`, `crates/client/tests/key_translation.rs` (replaced by decoder tests), and a new Lua regression spec under `tests/lua/`.
- Dependencies: no new crate. crossterm's `events` use ends on Unix.
- A lone Escape reaches gband 25 ms after it is pressed.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/core/src/terminal_input.rs
- crates/core/src/lib.rs
- crates/core/tests/terminal_input.rs
- crates/client/src/lib.rs
- crates/client/src/input.rs
- crates/client/src/channel.rs
- crates/client/tests/key_translation.rs
- crates/protocol/src/test.rs
- crates/harness/src/case.rs
- crates/harness/src/channel.rs
- crates/harness/src/terminal.rs
- tests/lua/escape_spec.lua
- tests/lua/sidebar_spec.lua
- docs/testing.md
- openspec/changes/input-decoder/
