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

`tmux` is not installed on the machine that ran the spike, so a small Python driver stood in for it: it ran the spike in a PTY at 100×30, typed into it, resized it, and read the screen back through `pyte`. The spike ran with `GBAND_LOG=debug`. Log lines below come from `client.2026-10-04.log`. The visual rows, such as cursor shape, undercurl and colours, read the spike's rendered output through `pyte`, and still want the operator's confirmation in their own terminal. The pane's shell was the user's fish 4, which bash hands off to.

Result values: `works`, `degraded` (usable, with a visible loss) and `fails`.

### nvim

nvim 0.12.5, with the user's own configuration.

| check | result | evidence | follow-up |
|---|---|---|---|
| startup time and any wait for query replies | works | `--startuptime` reports the first screen update at 167 ms, and nothing waits. Eight queries go unanswered: `\e]11;?`, `\e[?0u`, `\e[>0q`, `\e[0c` and DECRQM `\e[?2026$p`, `\e[?2027$p`, `\e[?2031$p`, `\e[?2048$p`. nvim therefore guesses the background colour and turns off synchronized output | `terminal-query-replies` |
| `:checkhealth` | degraded | No terminal error. `vim.health` reports `Terminal: ghostty 1.3.1-4.fc44` and `$TERM_PROGRAM="ghostty"`, because the pane inherits the outer terminal's identity. The `vim.provider` warnings and the error concern missing language providers, not the terminal | `pane-environment` |
| colours and truecolor | works | `&termguicolors` is `1` with `$COLORTERM=truecolor`, and cells carry 24-bit colours such as `fg=957fb8 bg=1f1f28` for a keyword | none |
| insert-mode cursor position and shape | degraded | `3Gwi` puts the cursor at row 2, column 10, matching the status line's `3:5` behind a 6-column gutter. The shape is lost: `unhandled CSI \e[6 q` on entering insert, then `\e[2 q` and `\e[4 q` in other modes, and `unhandled OSC \e]112`. The outer cursor stays a block | `cursor-shape` |
| Ctrl+`i` against Tab | degraded | Both arrive as `\x09`: `crossterm` reports `\x09` as Tab, and the encoder writes `\x09` for Ctrl+`i`, as the spec states. In insert mode, Ctrl+V then Tab inserts a literal tab, so nvim cannot bind `<C-i>` apart from `<Tab>` | `kitty-keyboard` |
| Shift and Ctrl arrows | works | In insert mode, Ctrl+V followed by each key inserts `<S-Up> <C-Up> <S-Left> <C-Right> <M-Down>` | none |
| undercurl on a diagnostic | degraded | An error diagnostic on `std` renders as a straight underline (`u=True`) with no colour. nvim emits plain `\e[4m` for it. The one `\e[4:3m` nvim sent, at startup, vt100 dropped whole (`unhandled CSI \e[4:3m`). vt100 parses neither `4:N` nor the `58` underline colour | `terminal-state-upgrade` |
| mouse | fails | A click sent as `\e[<0;30;10M` leaves the cursor where it was. The spike enables no mouse capture on the outer terminal and drops `Event::Mouse`. nvim also enables focus events (`unhandled CSI \e[?1004h`), which the spike never sends | `mouse-and-focus-forwarding` |
| yank to the system clipboard | works | `"+yy` puts `use std::io::{Read, Write};` on the Wayland clipboard, read back with `wl-paste`. `:checkhealth` reports `Clipboard tool found: wl-copy`, so nvim never touches the terminal for this. OSC 52 would be needed only where no clipboard tool exists, such as over SSH | none |
| a paste of 1000 lines | works | A bracketed paste of 1000 lines into an empty buffer gives `line('$')` of 1001, the first and last lines intact and no auto-indent | none |
| a resize | works | Resizing to 70×20 and back to 100×30 redraws the status line to each width, with the cursor kept on line 1001 | none |

### Claude Code

Claude Code v2.1.289, in its vim input mode, running in the scratchpad directory.

| check | result | evidence | follow-up |
|---|---|---|---|
| startup and first render | works | The trust dialog draws about 0.34 s after `claude` is typed, and the banner and prompt follow. It queries `\e[?0u`, `\e[>0q`, `\e[0c` and `\e]11;?`, gets no reply, and does not wait. It also sets kitty keyboard flags (`\e[>5u`), modifyOtherKeys (`\e[>4;2m`) and synchronized output (`\e[?2026h`), all unhandled | `terminal-query-replies` |
| typing and Enter | works | `Reply with only the word PONG` and Enter sends the prompt, and `● PONG` comes back | none |
| Shift+Enter for a newline | fails | Under the legacy encoding a terminal sends Shift+Enter as `\r`, and the encoder writes `\r` for it too, so the prompt is sent instead of breaking the line. Ctrl+`j` (`\x0a`) does insert a newline | `kitty-keyboard` |
| Escape to interrupt | works | During `Bootstrapping… (8s · thinking with xhigh effort)`, the first Escape leaves vim INSERT mode and the second cancels the request, putting the prompt back into the input. A lone `\x1b` is delivered without delay | none |
| Ctrl+C | works | Ctrl+C clears a typed input, and two in a row exit to the shell | none |
| arrow-key history | works | Up recalls the last submitted prompt. Further Up presses do not reach older prompts in this nested session, whose banner warns that transcript saving is off. The bytes are the same `\e[A` that worked the first time, so this is not an encoding problem | none |
| a large paste | works | A bracketed paste of 1000 lines shows as `[Pasted text #1 +1000 lines]` | none |
| colours | works | Truecolor reaches the screen, such as `fg=ffc107` on the warning line, and the logo and status line keep their colours | none |
| long scrolling output | degraded | Printing 1 to 400 scrolls smoothly and the screen ends on `396` to `400`. The rest is gone: the spike's parser keeps no scrollback, and Claude Code prints its transcript into the normal screen, so it relies on the terminal for scrollback | `scrollback` |
| a resize | works | At 70×20 the transcript and the input box reflow to the new width, and they reflow again at 100×30 | none |
| the notification or bell when it waits | fails | About 60 s after a reply, Claude Code sends `unhandled OSC \e]777;notify;Claude Code;Claude is waiting for your input`. It chose OSC 777 because the inherited `TERM_PROGRAM` says ghostty. The spike shows nothing. A bell from the shell reaches vt100's callback (`bell` in the log), but nothing forwards it either | `notifications` |
| exit | works | `/exit` and two Ctrl+C presses each return to the shell prompt. On the way out Claude Code pops its keyboard flags (`\e[<0u`) and turns off focus events (`\e[?1004l`) | none |

The shell shows two more findings. fish 4 sends Primary Device Attributes (`\e[0c`), gets no reply, and blocks for 10 s at every start before printing `fish could not read response to Primary Device Attribute query after waiting for 10 seconds`. This is the slowest start in the spike, so `terminal-query-replies` should come first. The log also records `\e(B` hundreds of times per nvim redraw. That sequence selects the ASCII character set, which is already in use, so it is harmless, but it buries the useful lines.

### Follow-up changes

- `terminal-query-replies`: answer DA1, DSR (`\e[5n`, `\e[6n`), XTVERSION, the kitty keyboard query, DECRQM for the modes gband supports, OSC 10 and 11 colour queries and XTGETTCAP from the pane's state, so fish starts without its 10 s wait and nvim and Claude Code detect features instead of guessing.
- `pane-environment`: set `TERM_PROGRAM` and `TERM_PROGRAM_VERSION` to gband's own values in every pane, and remove outer-terminal identity variables, so programs stop sending sequences meant for the outer terminal.
- `cursor-shape`: track DECSCUSR and OSC 12 and 112 per pane, and apply them to the outer terminal's real cursor for the focused pane.
- `kitty-keyboard`: run the kitty keyboard protocol on the outer terminal and toward each pane according to the flags the program pushed, restoring Shift+Enter and Ctrl+`i` against Tab. The legacy encoder stays the fallback.
- `mouse-and-focus-forwarding`: enable mouse capture on the outer terminal, encode mouse events for the pane under the pointer in the mode it requested (vt100 already tracks it), and send focus in and out to panes that enable mode 1004.
- `terminal-state-upgrade`: decide between patching vt100 and moving to `alacritty_terminal`, for SGR `4:N` underline styles and the `58` and `59` underline colours, synchronized output (mode 2026) and OSC 8 hyperlinks. These are the gaps the spike hit that vt100 has no hook for.
- `scrollback`: keep scrollback per pane and add a way to scroll it, so the start of long output in the normal screen can still be read.
- `notifications`: turn BEL and OSC 9 and 777 notifications from a pane into a desktop notification through `notify-rust`, and mark the pane that raised it.
