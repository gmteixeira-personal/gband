## Context

The client reads its terminal through `crossterm::event::read` on a thread of its own (`spawn_events` in `crates/client/src/lib.rs`), and `crates/client/src/input.rs` converts crossterm's key and mouse events to `gband_core::input::Key` and `MouseEvent`. crossterm resolves a lone ESC from the size of each `read()`, with no timeout. It also handles `ESC ESC` by emitting Escape and clearing both bytes (`crossterm-0.29.0/src/event/sys/unix/parse.rs:77`). See proposal.md for what that breaks.

The test channel's settle starts by writing a focus-in report `\x1b[I` to the client's terminal. The client counts the focus gains it reads, and settle's first step completes once that count reaches the number of markers the runner wrote. `Case::keys` in `crates/harness/src/case.rs` waits after a key ending in ESC until FIONREAD on the pty slave reads 0. That wait never waits: Linux moves pty input to the reader asynchronously, so FIONREAD read 0 right after the write in 2000 of 2000 trials.

Two reference implementations solve the same problem:
- zellij: `zellij-client/src/stdin_handler.rs`, with its vendored termwiz parser `zellij-utils/src/vendored/termwiz/input.rs` and `keymap.rs`. A probe of that parser decoded every merged and split case in the specs correctly.
- herdr: `src/raw_input.rs`.

Both are MIT-licensed. Their behaviour and their test cases are the reference for this change. Their code is not copied.

## Goals / Non-Goals

**Goals:**
- A decoder that has no clock and no I/O, fully testable from bytes alone.
- One timing rule, the 25 ms flush, owned by the client loop.
- A settle and a key wait that depend on bytes read instead of on markers that the terminal's own input can corrupt.

**Non-Goals:**
- The kitty keyboard protocol, CSI u keys, key release events.
- Windows input.
- Parsing the terminal's replies to queries. The client sends none today.
- urxvt mouse encoding 1015. gband enables SGR 1006 and never asks for 1015.

## Decisions

### A pure decoder in gband-core
`crates/core/src/terminal_input.rs` defines `TerminalInput { Key(Key), Mouse(MouseEvent), Paste(String), Focus(bool) }` and a `Decoder`:
- `push(&[u8]) -> Vec<TerminalInput>` returns every complete input and keeps the incomplete tail.
- `holds() -> bool` reports whether a tail is held.
- `flush() -> Vec<TerminalInput>` resolves the tail as "Incomplete input from the terminal" defines.

The decoder sits beside `encode_key`, so its tests can feed it what `encode_key` and `encode_mouse` produce. *Alternative:* a decoder that takes timestamps. Rejected because tests would need a fake clock, and zellij's split between `parse(maybe_more = true)` and an idle flush shows the timer belongs to the caller.

### The ESC prefix state, as in termwiz
After an ESC whose next byte starts another sequence, the decoder decodes that sequence first:
- A key gives that key with Alt.
- A mouse report, focus report or paste start gives Escape and then that event.
- Another ESC with nothing after it, before the flush, gives Alt+Escape.

Held bytes resolve as zellij's longest-match key lookup resolves them, where `\x1b[` is itself the key Alt+`[` and `\x1bO` is Alt+Shift+`O`. A byte that no sequence can continue resolves them at once: Alt with `[` or `O`, then the rest read as keys, so Alt+`[` followed within 25 ms by `x` is Alt+`[` then `x`. The flush resolves whatever is still held the same way, so Alt+`[` and Alt+Shift+`O` pressed alone stay Alt keys. A complete sequence that started like one of gband's and named none is dropped, as zellij skips a complete CSI sequence it does not know. gband's table differs from zellij's: it adds the Linux console F1 to F5 and the default mouse encoding, which zellij lacks. It leaves out zellij's rxvt Shift and Ctrl arrows, `\x1b[a` to `\x1b[d` and `\x1bOa` to `\x1bOd`, so Alt+`[` then `a` is Alt+`[` then `a`, where zellij reads Shift+Up. zellij also waits for the timeout before typing a malformed sequence's bytes, where gband types them once the malformed byte arrives; the keys are the same. A partial UTF-8 character survives the flush, as in zellij and herdr.

This matches zellij and herdr's macOS policy, which keep Terminal.app's Option+arrow (`\x1b\x1b[A`) as Alt+Up. *Alternative:* herdr's Linux policy, which splits `ESC ESC` into Escape and the next sequence. Rejected by the user's choice, and because it breaks Option+arrow on macOS.

### Sequence table as data
Keys are matched by an explicit table: the inverse of `encode_key`, plus the xterm, rxvt and Linux console variants in "Keys from the terminal's bytes". Control bytes decode as crossterm decodes them today, for example `\x08` as Ctrl+`h` and `\n` as Ctrl+`j`, so existing bindings keep working. A complete CSI or SS3 sequence that matches nothing is dropped whole. A sequence is complete when a CSI final byte in `0x40..=0x7e` or an SS3 letter arrives. Mouse reports are SGR `\x1b[<b;x;yM` or `m`, and the default `\x1b[M` followed by three bytes.

### Client loop
- `spawn_events` becomes a thread that reads raw standard input into byte chunks and sends them on the existing unbounded channel.
- The main `select!` gains a branch: a sleep until 25 ms after the last chunk, enabled only while `decoder.holds()`. When it fires, the loop calls `decoder.flush()`.
- Each chunk goes through `decoder.push`, and the inputs go to the existing `controls.press`, `controls.mouse` and `controls.paste`, one per pass of the loop with a draw between them, as crossterm's events did, so the frames drawn do not depend on how reads split the input. Focus inputs are dropped.
- Resize comes from `tokio::signal::unix::signal(SignalKind::window_change())` followed by `crossterm::terminal::size()`.
- `crates/client/src/input.rs` and its crossterm conversions are deleted.
- crossterm stays for raw mode, the alternate screen, bracketed paste and mouse enabling, and `terminal::size`.

### Byte-count acknowledgement in place of focus markers
- The runner already writes everything, keys and its emulator's replies, through one shared writer in `crates/harness/src/terminal.rs`. It counts bytes there.
- `ToProcess::Settle` changes from `markers: Option<u64>` to `input: Option<u64>`.
- The client counts the bytes of each chunk once it has handled the inputs that chunk produced. It answers an input settle once that count reaches the requested number and `decoder.holds()` is false. A held lone ESC therefore delays the answer until its flush.
- Settle's first step sends the runner's current byte count. It no longer writes `\x1b[I` or nudges every 250 ms.
- `Case::keys` replaces `drained()` with the same first step after a key ending in ESC, so the next key is written only after the client has read Escape alone.

*Alternative:* keep focus markers and only fix the decoder. Rejected: markers are bytes that the terminal's own input can corrupt, and the byte count needs no marker at all.

## Risks / Trade-offs

- **[Risk]** A key sequence from a terminal no one tested decodes to nothing. → Mitigation: port the key vectors from zellij's vendored `input.rs` tests, herdr's `raw_input.rs` tests and crossterm's `parse.rs` tests, and drop a sequence only when it is complete.
- **[Risk]** A lone Escape now reaches gband 25 ms late, and a sequence split by more than 25 ms still reads as Escape plus keys. → Accepted. This is the same trade-off as tmux's `escape-time`, neovim's `ttimeoutlen`, and zellij's 50 ms. The interval is a constant that can become an option later.
- **[Risk]** Two Escapes pressed within 25 ms read as Alt+Escape. → Accepted. Terminals encode Alt+Escape the same way.
- **[Risk]** The reader thread stays blocked in `read` when the client leaves. → No change: the crossterm reader thread behaves the same today.
- **[Trade-off]** Tests that press Escape take 25 ms longer per Escape.

## Migration Plan

No user-facing configuration changes. The protocol change to `ToProcess::Settle` is internal to the test channel. The runner and the client are the same binary, so they cannot disagree.
