## Why

A gband window has no name, so the only way to tell windows apart is to read their screens. zellij names each pane after what runs in it and keeps the name current, and lets the user rename a pane by hand. gband should do the same, with the name on each window's top border, so a strip of several shells, editors and builds can be read at a glance.

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
- **Protocol**: a client `rename` message and a server `window name` message, sent for each window after the snapshots of an attach and whenever a name changes, with a protocol version bump. **BREAKING** for a client and server of different builds, which the handshake already refuses.

Out of scope:
- An event for a name change, and names in the server's Lua.
- Title highlight groups apart from the border's.
- A name for a drawn window of a plugin window, and naming bands.
- Options to turn the titles, the command fallback or the numbering off.

## Capabilities

### New Capabilities
- `window-names`: the automatic name from the title and the foreground command, the manual name, the numbering of duplicates in a band, the border title, the rename prompt, and `gband.window.rename`.

### Modified Capabilities
- `client-attach`: the key binding tables gain `N`, renaming the focused window.
- `lua-control`: `gband.layout()` window tables hold `name` and `manual_name`, and `gband.window.rename` is dispatched like an action.
- `wire-protocol`: the `rename` client message, the `window name` server message and its place in the attach order, and the protocol version.

## Impact

- Emulator: `crates/emulator/src/callbacks.rs` and `lib.rs` keep the title and its stack.
- Core: a new `crates/core/src/names.rs` numbers duplicate names in layout order.
- Protocol: `crates/protocol/src/message.rs`, with its round-trip tests.
- Server: `crates/server/src/window.rs`, `session.rs`, `lib.rs`, and a new `process.rs` that reads the foreground command from `/proc` on Linux.
- Client: `crates/client/src/lib.rs` keeps the names and sends renames, and `render.rs` draws the titles. `bindings.rs` tests the new default key.
- Lua: `crates/lua/src/control.rs`, `crates/lua/src/runtime/gband/prompt.lua` and the two key style presets.
- Tests: unit, Lua, end-to-end and screen tests, including the key list references, which gain the `N` line.
- `README.md` and `docs/plugins.md`.
- No new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- settings-interactive-on-new

### Expected Files
- openspec/changes/window-names/
- crates/emulator/src/callbacks.rs
- crates/emulator/src/lib.rs
- crates/emulator/tests/contract.rs
- crates/core/src/names.rs
- crates/core/src/lib.rs
- crates/core/tests/names.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/src/process.rs
- crates/server/src/lib.rs
- crates/server/src/window.rs
- crates/server/src/session.rs
- crates/client/src/lib.rs
- crates/client/src/render.rs
- crates/client/src/bindings.rs
- crates/lua/src/control.rs
- crates/lua/src/runtime/gband/prompt.lua
- crates/lua/src/runtime/gband/keystyle/modal.lua
- crates/lua/src/runtime/gband/keystyle/direct.lua
- crates/lua/tests/control.rs
- crates/lua/tests/prompt.rs
- tests/window_names.rs
- tests/prompt.rs
- tests/keystyle.rs
- tests/lua_specs.rs
- tests/lua/window_names_spec.lua
- tests/lua/screenshots/
- README.md
- docs/plugins.md
