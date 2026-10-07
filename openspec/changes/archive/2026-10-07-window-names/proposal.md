## Why

A gband window has no name, so the only way to tell windows apart is to read their screens. zellij names each pane after what runs in it and keeps the name current, and lets the user rename a pane by hand. gband should do the same, with the name on each window's top border, so a strip of several shells, editors and builds can be read at a glance. Screen tests should show titles only where a test is about titles, so that drawing them does not rewrite every screen reference, and users who prefer bare borders should be able to turn them off.

## What Changes

- **Automatic name**: every window that runs a program has one, kept by the server and shared by every client:
  - The window's title, as the program last set it with OSC 0 or OSC 2. Titles holding `;` are kept whole. The title stack of `CSI 22 t` and `CSI 23 t` is honoured, so a program that pushes the title and pops it on exit restores the earlier one. An empty title clears it.
  - While no title is set, the name of the window's foreground command: the base name of the first argument of the terminal's foreground process group leader, with a login shell's leading `-` removed. It follows the foreground process within one second. Where it cannot be read, the window program's own command name is used.
  - As in zellij, a title the shell sets at each prompt, such as `user@host:~`, wins over the command. The command is shown only while no title is set, or when the shell itself sets the title to the running command, as fish does by default.
- **Manual name**: `N` in navigation mode, after the prefix key Ctrl+Space, opens a one-line `rename` prompt for the focused window. Enter saves the typed name, which overrides the automatic name until it is cleared. Enter on an empty line clears it, and the window takes its automatic name again. Escape cancels. The prompt starts holding the window's current manual name.
- **Duplicate names**: when two or more windows of one band share a name, each shows ` #n` after it, numbered from 1 in layout order: columns left to right, windows top to bottom within a column, then floating windows in their list order. A name used once shows no number. So `zsh #1` and `zsh #2`, but `vim` alone.
- **Border title**: the top border of every tiled and floating window that runs a program shows its name, from the border's second column, cut to the border's width less 2, in the border's style. A window whose top side is not drawn shows no title.
- **Lua**:
  - `gband.window.rename(window, name)` sets a window's manual name, and clears it when `name` is nil or empty. It is dispatched like an action.
  - The window tables of `gband.layout()` gain `name`, the name with its number, and `manual_name`.
  - `gband.prompt`'s setup registers `prompt.rename`, described `rename the window`, beside `prompt.open`.
- **Key bindings**: both key style presets bind `prefix N` to `prompt.rename`. With the modal style, `N` returns to interactive mode, as `:` does. The key list shows the new line by itself.
- **Turning titles off**:
  - A client option `window_titles`, a boolean, `true` by default. While it is `false`, the client draws no border title. Names are still kept, sent, read by Lua and renamed.
  - The client reads `GBAND_WINDOW_TITLES` when it starts, as it reads `GBAND_ANIMATIONS`. `off` turns titles off whatever the option holds. Unset or `on` leaves the option in charge. Any other value is logged as a warning and read as unset.
  - `gband test` cases start with titles off. The runner sets `GBAND_WINDOW_TITLES=off` unless `g.start`'s new `window_titles` field is `true`. gband's own end-to-end tests start with titles off too, and only the tests about titles turn them on.
  - So no existing screen reference gains a title. Only the key list references change, for the `N` line.
- **Protocol**: a client `rename` message and a server `window name` message, sent for each window after the snapshots of an attach and whenever a name changes, with a protocol version bump. **BREAKING** for a client and server of different builds, which the handshake already refuses.

Out of scope:
- An event for a name change, and names in the server's Lua.
- Title highlight groups apart from the border's.
- A name for a drawn window of a plugin window, and naming bands.
- Options to turn the command fallback or the numbering off, and hiding titles per window or per band.

## Capabilities

### New Capabilities
- `window-names`: the automatic name from the title and the foreground command, the manual name, the numbering of duplicates in a band, the border title, the rename prompt, and `gband.window.rename`.

### Modified Capabilities
- `client-attach`: the key binding tables gain `N`, renaming the focused window.
- `lua-control`: `gband.layout()` window tables hold `name` and `manual_name`, and `gband.window.rename` is dispatched like an action.
- `configuration`: the client option `window_titles` in the options table, and the default `prefix N` binding between `:` and `s`.
- `plugin-testing`: `g.start`'s `window_titles` field and `GBAND_WINDOW_TITLES=off` in the case environment.
- `wire-protocol`: the `rename` client message, the `window name` server message and its place in the attach order, and the protocol version.

## Impact

- Emulator: `crates/emulator/src/callbacks.rs` and `lib.rs` keep the title and its stack.
- Core: a new `crates/core/src/names.rs` numbers duplicate names in layout order.
- Protocol: `crates/protocol/src/message.rs`, with its round-trip tests.
- Server: `crates/server/src/window.rs`, `session.rs`, `lib.rs`, and a new `process.rs` that reads the foreground command from `/proc` on Linux.
- Client: `crates/client/src/lib.rs` keeps the names, sends renames and reads `GBAND_WINDOW_TITLES`, and `render.rs` draws the titles. `bindings.rs` tests the new default key.
- Options: `crates/lua/src/options.rs` declares `window_titles`.
- Harness: `crates/harness/src/case.rs` (the `window_titles` field and the variable) and `crates/harness/src/env.rs` (gband's own end-to-end environment).
- Lua: `crates/lua/src/control.rs`, `crates/lua/src/runtime/gband/prompt.lua` and the two key style presets.
- Tests: unit, Lua, end-to-end and screen tests. Of the existing references, only the key list's change, gaining the `N` line.
- `README.md`, `docs/plugins.md` and `docs/testing.md`.
- No new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- settings-interactive-on-new

### Expected Files
- openspec/changes/window-names/
- README.md
- crates/client/src/bindings.rs
- crates/client/src/lib.rs
- crates/client/src/render.rs
- crates/client/tests/render.rs
- crates/core/src/lib.rs
- crates/core/src/names.rs
- crates/core/tests/names.rs
- crates/emulator/src/callbacks.rs
- crates/emulator/src/lib.rs
- crates/emulator/tests/contract.rs
- crates/harness/src/case.rs
- crates/harness/src/env.rs
- crates/harness/src/runner.rs
- crates/lua/src/api.rs
- crates/lua/src/control.rs
- crates/lua/src/defaults.lua
- crates/lua/src/lib.rs
- crates/lua/src/options.rs
- crates/lua/src/runtime/gband/keystyle/direct.lua
- crates/lua/src/runtime/gband/keystyle/modal.lua
- crates/lua/src/runtime/gband/prompt.lua
- crates/lua/src/ui.rs
- crates/lua/tests/config.rs
- crates/lua/tests/control.rs
- crates/lua/tests/options.rs
- crates/lua/tests/plugins.rs
- crates/lua/tests/prompt.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/src/connection.rs
- crates/server/src/lib.rs
- crates/server/src/process.rs
- crates/server/src/session.rs
- crates/server/src/window.rs
- crates/server/tests/scripting.rs
- crates/server/tests/sync.rs
- crates/test-support/src/lib.rs
- docs/plugins.md
- docs/testing.md
- tests/keystyle.rs
- tests/lua/prompt_spec.lua
- tests/lua/screenshots/keylist_spec/default-list--last-page.txt
- tests/lua/screenshots/prompt_spec/the-key-list-holds-the-prompt-between-the-key-list-and-detach--hint.txt
- tests/lua/screenshots/window_names_spec/
- tests/lua/window_names_spec.lua
- tests/lua_specs.rs
- tests/prompt.rs
- tests/window_names.rs
