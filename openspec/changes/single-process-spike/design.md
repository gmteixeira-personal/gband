## Context

`workspace-skeleton` left four empty library crates, a `gband` binary whose `server` and `attach` subcommands only log that they started, and file logging in the root package's `src/logging.rs`. `gband-core` depends only on `serde` and `thiserror`. `gband-client` already depends on `crossterm`, `ratatui`, `tui-term` and `vt100`, and `portable-pty` belongs to `gband-server`.

The pinned versions fit together: `tui-term` 0.3.4 depends on `vt100` 0.16.2 and on `ratatui-core` 0.1, which `ratatui` 0.30.2 uses too. `vt100` 0.16 reports every escape sequence it does not handle through its `Callbacks` trait, and exposes `application_cursor()` and `bracketed_paste()` on its `Screen`.

`crossterm`'s legacy input parser decides which key events the encoder sees. In raw mode it reports `\x08` as Ctrl+`h`, `\x0a` as Ctrl+`j`, `\x00` as Ctrl+space and `\x1c` to `\x1f` as Ctrl+`4` to Ctrl+`7`. It reports ESC followed by a key as Alt with that key. It reports `\x09` as Tab, so Ctrl+`i` and Tab arrive as the same event unless the outer terminal runs the kitty keyboard protocol.

## Goals / Non-Goals

**Goals:**
- An encoder that the server and the client can both call, so build step 1 can encode on either side without rewriting it.
- A spike whose failures can be traced: the escape sequences `vt100` ignores are in the log, not guessed at.
- A Findings list concrete enough to name each follow-up change.

**Non-Goals:**
- Fixing anything the spike finds. Each fix is its own change.
- The kitty keyboard protocol, on the outer terminal or toward the program in the pane. The encoder emits legacy xterm encodings only.
- Application keypad mode. The legacy input parser cannot tell keypad keys from the main keys, so the mode would change nothing until the kitty keyboard protocol arrives.
- Mouse, focus events, cursor shape (DECSCUSR), OSC 52 clipboard, and replies to terminal queries such as device attributes or cursor position.
- A server, a socket, a protocol or more than one pane.
- Any change to the `gband` binary or its command line.

## Decisions

### The encoder lives in `gband-core` with its own key type

`crates/core/src/input.rs` defines `Key`, its `KeyCode`, `Modifiers` with `shift`, `alt` and `ctrl`, and `Modes` with `application_cursor` and `bracketed_paste`. It exposes `encode_key(Key, Modes) -> Vec<u8>` and `encode_paste(&str, Modes) -> Vec<u8>`. `KeyCode` holds exactly the keys the `input-encoding` spec encodes: characters, Enter, Tab, Shift+Tab, Backspace, Escape, the four arrows, Home, End, Insert, Delete, Page Up, Page Down and function keys by number.

The encoder takes plain values, not `vt100::Screen` or `crossterm::event::KeyEvent`. That keeps `gband-core` free of terminal crates, and lets the server encode in build step 1. The server owns the authoritative grid, so it sees a mode change before any key typed after it. A client encoding from its own grid, which trails the server by one diff, could encode a key in a mode the program has already left. The kitty keyboard protocol makes this stronger: the program in the pane pushes its keyboard flags to whatever parses its output, which is the server.

Alternative: put the encoder in `gband-client` and take `crossterm`'s `KeyEvent` directly. That saves one translation function, but it ties the encoder to the client and to `crossterm`'s type, and moving it later means rewriting its signature and its tests.

### The translation from `crossterm` lives in `gband-client`

`crates/client/src/input.rs` exposes `key_from_event(&KeyEvent) -> Option<Key>`. It returns `None` for a release, and for any key code outside `KeyCode`, and drops Super, Hyper and Meta. It cannot be a `From` implementation, because both types are foreign to `gband-client`. `gband-client` gains `gband-core` as a direct dependency. It already reached `gband-core` only through `gband-protocol`.

### Legacy xterm encodings only

Every key encodes as xterm does with its defaults, so the bytes match what nvim and Claude Code expect when `TERM` is `xterm-256color` and no extended keyboard mode is on. A modifier that the legacy encoding cannot express is dropped rather than invented, as the spec states. Shift+Enter writes `\r` and Ctrl+`i` writes `\x09`. The spike will show what that costs, and the fix is the kitty keyboard protocol, not a private encoding.

### Paste filters control characters

`encode_paste` removes C0 controls other than tab, line feed and carriage return, and all C1 controls. Without that, pasted text holding `\x1b[201~` ends the bracket early, and the rest runs as typed input, which is a known paste-injection attack. Removing ESC also removes every other escape sequence a paste could carry into a program that has not enabled bracketed paste.

### The spike is an example of the root package

`examples/spike.rs` runs with `cargo run --example spike`. The root package holds `gband::logging`, which the spike needs to write to a file while the terminal is in raw mode. An example under `crates/client/` cannot reach it, because `gband-client` cannot depend on the root package that depends on it. The example's crates are dev-dependencies of the root package, so the `gband` binary links nothing new. It logs as the `client` role, so its log is `client.<date>.log`, and the `logging` spec is unchanged.

The spike runs on threads, not `tokio`:

- **Terminal**: `ratatui::init()` sets raw mode and the alternate screen and installs a panic hook that restores the terminal. The spike also enables bracketed paste on the outer terminal, so a paste arrives as one `Event::Paste`, and disables it on exit. It pushes no kitty keyboard flags.
- **PTY**: `portable_pty::native_pty_system()` opens a PTY at the outer terminal's size. `CommandBuilder::new_default_prog()` runs the user's `$SHELL` in the current directory with `TERM=xterm-256color` and `COLORTERM=truecolor`.
- **Output**: a reader thread reads the PTY master and feeds a `vt100::Parser` behind an `Arc<Mutex<_>>`, then wakes the main loop through a channel. End of file on the PTY means the shell exited, and the spike exits.
- **Unhandled sequences**: the parser is built with `new_with_callbacks` and a callbacks value that logs each `unhandled_csi`, `unhandled_osc`, `unhandled_escape` and `unhandled_control` at `debug`. Run with `GBAND_LOG=debug`, the log lists every sequence a program sent that `vt100` ignored.
- **Input**: `Event::Key` goes through `key_from_event`, then `encode_key` with `Modes` read from the parser's screen at that moment, then to the PTY writer. `Event::Paste` goes through `encode_paste`. `Event::Resize` resizes the PTY and the parser's screen.
- **Render**: `tui_term::widget::PseudoTerminal` draws the screen over the whole frame, with its own cursor hidden. The frame then places the outer terminal's real cursor at the screen's cursor position, or hides it when the program hid it. The real cursor is the one a later change gives a shape.
- **Keys**: no key is reserved. Every key reaches the shell, so the spike ends only when the shell exits. A hung program is killed from another terminal.

Alternative: a hidden `gband spike` subcommand. It would reuse logging directly, but it adds a subcommand to the binary and a requirement to the `command-line` spec for code this change throws away.

### How the findings are gathered

The spike is driven inside a `tmux` session. `tmux send-keys` types into it and `tmux capture-pane -p -e` reads the screen back, so an agent can run the byte-level and screen-level checks. Running under `tmux` puts `tmux`'s own key encoding between the keyboard and the spike, so the visual checks, such as cursor position, undercurl and colours, and anything key-related that looks wrong under `tmux`, are confirmed by the operator in their own terminal.

## Risks / Trade-offs

- [`vt100` lacks features nvim or Claude Code depend on, such as undercurl or synchronized output] → That is a finding, not a failure of this change. `summary.txt` names `alacritty_terminal` as the upgrade path, and the Findings say whether it is needed.
- [Programs wait for replies to terminal queries that nothing sends, and start slowly] → The unhandled-sequence log names each query. Replying is a follow-up change.
- [Locking the parser on every PTY read and every frame contends under heavy output] → Acceptable for one pane in a spike. Build step 1 replaces this loop with the server's.
- [Legacy encoding loses Shift+Enter, Ctrl+`i` against Tab, and similar chords] → The spec states each loss, so a test fails if a later change alters it by accident. The kitty keyboard change restores them.
- [The spike's code is copied into the server or client as is] → The Findings name what to keep. Only `input.rs` in `gband-core` and `gband-client` is meant to last.

## Migration Plan

None. Nothing has been released.

## Findings

The spike fills this section. Each row names the check, what happened, the evidence, such as a captured screen or a log line, and the follow-up change it needs, or `none`.

### nvim

| check | result | evidence | follow-up |
|---|---|---|---|

### Claude Code

| check | result | evidence | follow-up |
|---|---|---|---|

### Follow-up changes

Each follow-up change named above, with one line on its scope.
