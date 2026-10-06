## Why

The Lua API is reachable only from files: to call a gband function once, the user edits `user/init.lua`, binds a key, saves and presses it. Trying an action, opening a chooser again, or calling a plugin's function by hand should not need a reload. Neovim answers this with its command line, where `:` takes one line and runs it. gband needs the same: one line of Lua, typed and run at once, with the whole API a binding function has.

## What Changes

- **Lua prompt**: `:` in navigation mode opens a one-line floating plugin window, titled `lua`, on the bottom rows of the ribbon area. It shows `:`, the typed text, and a block cursor drawn as a reversed cell.
  - Typed characters, `j`, `k` and `q` included, go into the line. Backspace deletes the last character, and on an empty line closes the prompt. Ctrl+U clears the line.
  - A paste goes into the line, with each line break turned into a space.
  - Enter closes the prompt and runs the line. Escape closes it and runs nothing.
  - A line too long for the window shows its end, so the cursor stays in view.
- **Running the line**: the line is compiled as Lua text and runs as code that belongs to no plugin, as a binding function of the configuration file does. It can call `gband.action.*`, `gband.win.*`, `gband.keymap.enter` and everything else a binding function can.
  - It runs with its own instruction budget. An endless loop stops only the line, and the prompt keeps working.
  - Syntax errors, runtime errors and a stop by the instruction limit are reported like any configuration error, as `prompt:1: <message>`, so the status line's error item shows `error` and the error list holds the message.
- **Bundled plugin**: `gband.prompt`, plugin `prompt`, registers the action `prompt.open` with the description `run Lua`. Opening the prompt enters `root`, so typed keys reach it rather than navigation mode. Opening it while it is open focuses it.
- **Text input for plugin windows**: `gband.win.open` takes a new option, `on_input`, a function of the plugin window's number and a text.
  - A key with no `keys` entry that types a character, without Ctrl or Alt, runs `on_input` with that character.
  - A paste runs `on_input` with the pasted text, unchanged.
  - `keys` entries come first, then `on_input`, then the default keys. With `on_input` set, `j`, `k` and `q` are text and no longer scroll or close.
  - A plugin window without `on_input` behaves as today, and its pastes are still discarded.
- **Highlight group**: `PromptCursor`, with the default `{ reverse = true }`, draws the prompt's cursor.
- **Default configuration**: sets up `gband.prompt` right after `gband.keylist` and before `gband.errors`, and binds `prefix :` to `prompt.open`, between `?` and `D`. `root` still binds nothing, so `:` typed in interactive mode reaches the focused window.

Out of scope:
- History, completion, moving the cursor inside the line, and multi-line input.
- Showing the line's return values. The only notification API, `gband.notify`, asks the terminal for a desktop notification, which is the wrong place for a result. A line can call `gband.notify` itself.
- A command `prompt.open`, a prompt opened with text already typed, and a prompt for the server's Lua.
- A Rust drawing change for a terminal cursor inside plugin windows.

## Capabilities

### New Capabilities
- `lua-prompt`: the bundled `gband.prompt` plugin, its action, its floating plugin window, editing the line, running it as code of no plugin with its own budget, its error reports, the `PromptCursor` group, and its default setup and binding.

### Modified Capabilities
- `plugin-windows`: `gband.win.open` takes `on_input`. A focused plugin window gives typed text and pastes to `on_input`, after its `keys` entries and before the default keys. `on_input` runs as a callback of the plugin window's plugin.
- `client-attach`: the default table of navigation mode gains `:`, and a paste reaches a focused plugin window's `on_input` instead of being discarded.
- `configuration`: the default configuration sets up `gband.prompt` and binds `:` to `prompt.open`.
- `plugins`: the bundled modules `require` finds include `gband.prompt`, and `gband.errors`, which the list had left out. A bundled module's chunk receives the client's host table, which `require` never returns.

## Impact

- `crates/lua/src/runtime/gband/win.lua`: the `on_input` option, the key order, and a paste hook.
- `crates/lua/src/runtime.rs`: the key's text passed to the plugin window key hook, and a new entry point for a paste into a plugin window.
- `crates/lua/src/bundled.rs` and `runtime.rs`: the new bundled module, and the host table passed to bundled modules in the client.
- `crates/lua/src/runtime/gband/prompt.lua`: the new plugin.
- `crates/lua/src/defaults.lua`: the plugin's setup and the `:` binding.
- `crates/client/src/lib.rs`: a paste into a focused plugin window goes to the runtime.
- Tests: new `crates/lua/tests/prompt.rs` and `tests/prompt.rs`, a screen test `tests/lua/prompt_spec.lua`, new cases in the plugin window tests of `crates/lua/tests/` and `crates/client/tests/`, and the default binding lists in `crates/lua/tests/` and `crates/client/src/bindings.rs`.
- `README.md` and `docs/plugins.md`: the `:` key, the prompt, and `on_input`.
- No protocol change and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- navigation-mode
- sidebars-borders-steps

### Expected Files
- crates/lua/src/runtime/gband/win.lua
- crates/lua/src/runtime/gband/prompt.lua
- crates/lua/src/runtime.rs
- crates/lua/src/bundled.rs
- crates/lua/src/defaults.lua
- crates/lua/tests/plugin_windows.rs
- crates/lua/tests/prompt.rs
- crates/lua/tests/config.rs
- crates/lua/tests/plugins.rs
- crates/lua/tests/key_hints.rs
- crates/client/src/lib.rs
- crates/client/src/bindings.rs
- crates/client/tests/plugin_windows.rs
- tests/prompt.rs
- tests/lua/prompt_spec.lua
- tests/lua/screenshots/prompt_spec/
- tests/lua_specs.rs
- README.md
- docs/plugins.md
- openspec/changes/lua-prompt/
