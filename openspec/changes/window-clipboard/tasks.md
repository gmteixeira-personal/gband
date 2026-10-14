## 1. Emulator

- [ ] 1.1 In `crates/emulator/src/callbacks.rs`, accept a selection that is empty or holds `c`, drop data that is `?`, not base64 or over 1 MiB decoded, keep the latest accepted text in a `clipboard` field whatever the records hold, and add `Vt100::take_clipboard` in `crates/emulator/src/lib.rs`. Extend `osc_52_is_decoded` with the empty selection, `ST` ending and the 1 MiB plus one byte case, and add a test that `take_clipboard` returns the latest write with records off; verify `cargo test -p gband-emulator`

## 2. Protocol

- [ ] 2.1 Add `ServerMessage::Clipboard { window, text }` to `crates/protocol/src/message.rs`, raise `PROTOCOL_VERSION` to 15, and add a round trip of window 2 with `hi` plus the version 14 rejection to `crates/protocol/tests/messages.rs`; verify `cargo test -p gband-protocol`

## 3. Server

- [ ] 3.1 In `crates/server/src/session.rs` and `crates/server/src/window.rs`, give each window the session's shared clipboard counter, take the grid's clipboard write in `Window::process`, stamp it and keep `(stamp, text)`; leave `Window::replace` without a clipboard write. Verify `cargo build -p gband-server`
- [ ] 3.2 In `crates/server/src/connection.rs`, add `clipboard` to `Sent`, set it to the counter at attach, and after the window names send `ServerMessage::Clipboard` for the window holding the highest stamp above it. Add `crates/server/tests/clipboard.rs` with the specs' "Two clients", "Other session", "Write before attaching", "Latest write wins" and "Query is never answered" scenarios; verify `cargo test -p gband-server --test clipboard`

## 4. Client and Lua

- [ ] 4.1 Add the client option `window_clipboard` (boolean, default `true`, the spec's description) to `crates/lua/src/options.rs`, set it in `crates/lua/src/defaults.lua` after `notify_style`, and add default, off and wrong-type tests to `crates/lua/tests/options.rs`, including it in the listed names; verify `cargo test -p gband-lua --test options`
- [ ] 4.2 Add `WindowClipboard` with `window` and `text` to `crates/lua/src/events.rs`. In `crates/client/src/lib.rs`, route `ServerMessage::Clipboard` through `Controls::bridge` to the event, then push a write of `clipboard_sequence(text)` when `window_clipboard` is true. Add tests to `crates/client/tests/events.rs` for the event payload, the default write, no write with the option off, and a handler copying only window 2; verify `cargo test -p gband-client --test events`

## 5. End to end

- [ ] 5.1 Following `.claude/skills/gband-test/SKILL.md`, add `tests/lua/window_clipboard_spec.lua`: a case that runs `printf '\033]52;c;aGk=\007'` in a window and waits for `g.clipboard()` to equal `{ "hi" }`, and a case with `window_clipboard = false` in `user/init.lua` where the same command leaves `g.clipboard()` empty while a `WindowClipboard` handler records the text; verify `cargo test --test lua_specs`

## 6. Documentation

- [ ] 6.1 In `README.md`, add `window_clipboard` to the client options table, and next to the OSC 52 note at the clipboard section say that programs in windows, such as Claude Code or neovim, copy to your clipboard by default, name the risk of `cat` of an untrusted file and nested ssh, and show `gband.set { window_clipboard = false }`. Verify by reading it against the window-clipboard spec
- [ ] 6.2 In `docs/plugins.md`, add `window_clipboard` to the client options list with its description and the risk, add `WindowClipboard` to the built-in events with a handler that copies from one window only, and say queries are never answered. Verify every row of the lua-events delta and the "Window clipboard option" requirement appear there
- [ ] 6.3 Add `gband.opt.window_clipboard = true` to the quoted `defaults/init.lua` block of `docs/internals/11-default-configs.md` in the same place as `defaults.lua`; verify `cargo test -p gband-lua --test tutorial`

## 7. Final checks

- [ ] 7.1 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass
