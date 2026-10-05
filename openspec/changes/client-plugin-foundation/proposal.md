## Why

gband's Lua configuration can set options and bind keys, but nothing beyond one `init.lua` can extend the client: there is no way to ship reusable Lua code, add an action, react to what happens in the client, or declare an option. Later features (a status line, overlays, a command palette, notifications) each need an extension point, and they need a plugin model to hang it on. This change builds that model, client side only, keeping the Rust core small and making the default configuration use the same public API a plugin uses.

## What Changes

- A runtimepath, `gband.runtimepath`: the user directory `user/` in the configuration directory, then every directory under `$XDG_DATA_HOME/gband/plugins/` (`~/.local/share/gband/plugins/`) in name order. In each entry, `lua/<name>.lua` and `lua/<name>/init.lua` are found by `require("<name>")`, and `plugin/*.lua` then `plugin/client/*.lua` are sourced at startup. `plugin/server/` is reserved for a future server runtime and never sourced.
- Load order: the init file, then the `plugin/` files of each runtimepath entry in order. The init file is `user/init.lua` when it exists and the default configuration otherwise, exactly as today: there is no built-in layer under the user's file. The default configuration is the one Lua configuration gband ships.
- `gband.side` (`"client"`), `gband.api_version` (`1`), and `gband.plugin(name, opts)`, which requires a plugin module of the shape `{ name, api, setup }` and calls `setup(opts)`. A plugin declaring another `api` is set up with a warning.
- Error isolation: every plugin entry point and callback runs protected. A failing plugin file or setup is reported with the plugin's name, file and line, in the log and on the client's existing error banner. Its callbacks are disabled, and the client starts and keeps running. Errors in the init file itself still fail the load, as today.
- A VM instruction limit stops runaway Lua code in any entry point or callback, and disables the plugin that owns it.
- Actions: `gband.action.register(name, fn, { desc })` adds actions that bind and call like built-in ones. `gband.action.list()` lists built-in and registered actions with descriptions.
- Commands: `gband.cmd.register(name, fn, { desc, args })`, `gband.cmd.run(name, args)` and `gband.cmd.list()`. No palette or command-line integration.
- Names a plugin registers (actions, commands, options) are namespaced `<plugin>.<name>`. Names from the init file and the default configuration are not.
- Options: `gband.opt.<name>` reads and sets options with validation. `gband.opt.declare(name, { type, default, desc })` declares a plugin option. An invalid value set through `gband.opt` is reported at its file and line and falls back to the declared default without failing the load. `gband.set` keeps its current behaviour, and the options the server reads reach it exactly as today.
- Events: `gband.on(event, fn, { group, once, pattern })`, `gband.augroup(name, { clear })` and `gband.emit(name, data)` for `User` events. Built-in events cover only what the client already observes: `Attached`, `FocusChanged`, `BandChanged`, `PaneOpened`, `PaneClosed`, `TerminalResized`, `ConfigReloaded` and `KeyTableChanged`.
- Keymaps: `gband.keymap.set(table, key, action_or_fn, { desc })`, `gband.keymap.del(table, key)`, `gband.keymap.list(table)`, `gband.keymap.current_table()` and `gband.keymap.enter(table)`. Tables are `root`, `prefix` and any named table. `gband.bind` and `gband.unbind` keep working as shorthands for `root` and `prefix`.
- The default configuration binds every default key through `gband.keymap.set` with a description. Its bindings and options are unchanged.
- Changes to any `.lua` file under `user/` reload the configuration, so `user/plugin/` and `user/lua/` reload like `user/init.lua`.
- `print` writes to the process log instead of the terminal, so a plugin cannot corrupt the client's screen.
- A sample plugin in `examples/plugins/hello` and a "writing a plugin" guide in `docs/plugins.md`.

## Capabilities

### New Capabilities

- `plugins`: the runtimepath, module lookup, `plugin/` sourcing and its reserved subdirectories, `gband.side`, `gband.api_version`, `gband.plugin` and the module shape, ownership and namespacing, error isolation, the instruction limit, and `print`.
- `lua-commands`: the named command registry, `gband.cmd`.
- `lua-events`: `gband.on`, `gband.augroup`, `gband.emit`, the built-in client events and their payloads.

### Modified Capabilities

- `configuration`: the load sequence after the init file, the default configuration's use of `gband.keymap.set` with descriptions, `gband.opt` and declared options, the keymap API beside `gband.bind`, registered actions, callbacks beyond binding functions, non-fatal plugin errors, and reload on any `.lua` change under `user/`.
- `client-attach`: key bindings resolve through key tables, of which the prefix table is one, and the client tracks the active table.
- `actions`: every built-in action has a description, and `gband.action.list()` lists built-in and registered actions.

## Impact

- `crates/lua`: the bulk of the work. The API module splits into runtime, plugin ownership, guard, actions, commands, events, keymap and options modules. `Config` gains the keymap tables, the callback registry and the non-fatal errors of a load.
- `crates/client`: `Keymap` and `Leader` generalise to named key tables. `Controls` runs callbacks through the guarded runtime and emits the built-in events from changes it already observes in `Display`.
- `src/main.rs`: passes the plugin directory to the loader. The server still evaluates the configuration and reads only the column width options, through the same `watch` channel.
- The server process evaluates the full client load, plugins included, to read those options, so a plugin's top-level code runs there too. No callback ever runs in the server.
- No protocol, server-side or dependency change. `mlua` already provides the hook API.
- New `examples/plugins/hello/` and `docs/plugins.md`. `README.md` points at the guide and shows `gband.keymap.set`.

## Coordination

### Author
- gmteixeira

### Depends On
- rename-workspaces-to-bands

### Expected Files
- crates/lua/
- crates/client/src/lib.rs
- crates/client/src/bindings.rs
- crates/client/tests/actions.rs
- crates/client/tests/events.rs
- src/main.rs
- tests/common/mod.rs
- tests/config.rs
- examples/plugins/hello/
- docs/plugins.md
- README.md
- openspec/changes/client-plugin-foundation/
