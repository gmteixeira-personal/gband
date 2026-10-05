## Context

`crates/lua` holds all of gband's Lua today. `load(dir)` creates one `mlua::Lua` state, installs the `gband` table (`set`, `bind`, `unbind`, `spawn`, `action`), and evaluates exactly one init file: `user/init.lua` when it exists, otherwise the embedded `DEFAULTS` text, named `defaults/init.lua` so errors point at the on-disk mirror that `prepare` writes. While loading, a `Loading` app-data value collects options and bindings, and its presence is what makes `set`/`bind` legal. Binding functions run later through `gband_lua::call`, which installs a `Queue` app-data value that makes action calls and `gband.spawn` legal and collects the resulting `Dispatch`es.

Both processes call `load`. `src/main.rs` gives the server `config.options.layout` through a `tokio::sync::watch` channel, and the client gets the whole `Config` plus a reload channel. `gband_lua::watch` watches `user/init.lua` only, with a fallback to the configuration directory while `user/` is missing.

The client's `Controls` owns the Lua state, a `Keymap` (direct and prefixed bindings plus the prefix key) and a `Leader` (one `after_prefix` flag). `Display` owns the view, the layout and the grids, and already knows the focused pane, the viewed workspace (`View::workspace`), the panes of the last layout, and the terminal size. The server's `SessionEvent` bus never reaches the client. The only user-facing error surface is the bottom-row banner.

This change depends on `rename-workspaces-to-bands`. It is written against the band vocabulary that change introduces: `BandId`, `View::band` and `focus_band_down`/`focus_band_up`.

## Goals / Non-Goals

**Goals:**
- Exactly one init file per load, as today. The default configuration is the one Lua configuration gband ships, and the user's file replaces it.
- Every registration and callback has a known owner, so errors are attributed and a failed plugin is switched off as a unit.
- The client's behaviour with no `user/init.lua` and no plugin directory stays identical, binding for binding.
- The server keeps reading only the column width options, through the same `watch` channel.

**Non-Goals:**
- A server-side Lua runtime. `plugin/server/` is reserved and never read.
- Extension points for the status line, overlays, a command palette, notifications, highlight groups, persistence, agents or title/bell tracking. Each adds its own when it is built. `gband.cmd.list()` and `gband.action.list()` exist so a later palette has something to read.
- Sticky key-table modes. An entered table lasts one key, like `prefix`.
- Changing bindings, options or registrations after loading. Runtime keymap edits come later, if needed.
- Lazy loading, a plugin manager, health checks.
- Watching the plugins directory for reload.

## Decisions

### One init file, no bundled runtime layer
The loader keeps today's choice of init file: `user/init.lua` if present, otherwise the embedded default configuration. After it, plugin files are sourced from the runtimepath. There is no bundled runtime directory in the runtimepath. Anything auto-sourced from one would sit under the user's file and contradict "the user's file replaces the defaults".
- Alternative: a bundled `plugin/defaults.lua` sourced before `user/init.lua`, neovim style. Rejected by the user, because it turns the defaults into a layer that a user file must undo.
- Option declarations keep their defaults in Rust (`Options::default()`). A declared default is part of an option's type, not a configuration layer, and it is what makes an empty `user/init.lua` behave as today: Ctrl+Space as the prefix, no bindings.

### Runtimepath entries and locations
`gband.runtimepath` starts as `[<config>/user, <data>/gband/plugins/* in byte order]`. The user entry is `user/`, not the configuration directory itself, because `defaults/` is gband-owned and rewritten on start. `<data>` follows the existing `config_dir_from` pattern: `$XDG_DATA_HOME` when absolute, else `$HOME/.local/share`. `load` takes a `Locations { config: PathBuf, plugins: Option<PathBuf> }` built in `src/main.rs`, so tests can point both at scratch directories without touching the environment.

Module lookup is a searcher inserted at index 2 of `package.searchers`, after `preload`. It reads `gband.runtimepath` at call time and loads chunks named `@<path>`, so the existing `ConfigError::from_lua` location parsing works unchanged. Lua's own `package.path` searcher stays, so nothing that loads today stops loading.

### The server evaluates the same load
The server keeps calling `gband_lua::load` and keeps only `options.layout`. Plugin files run in the server too, so a width option set by a plugin reaches the server exactly as one set in `user/init.lua`. `gband.side` is `"client"` there as well, because what the server evaluates is the client configuration. No callback ever runs in the server: it never presses keys, emits events or runs commands.
- Alternative: the server evaluates only the init file. Rejected because a plugin's width option would then diverge between processes.
- Alternative: a server-only options pass. Rejected because it needs the server runtime that is out of scope.

### Module layout in `crates/lua`
`api.rs` is split by concern. Each module installs its part of the `gband` table:
- `runtime.rs`: `Locations`, runtimepath defaults, the searcher, plugin-file sourcing, `gband.plugin`, `gband.side`, `gband.api_version`, the `print` override.
- `owner.rs`: the owner stack (`Option<PluginName>` per running entry) and the failed-plugin set.
- `guard.rs`: protected execution of an entry point, the instruction budget hook, and attribution of errors to `ConfigError { plugin, location, message }`.
- `actions.rs`: built-in action table with descriptions, `register`, `list`, and the `LuaAction` userdata, now `Builtin(Action)` or `Registered { name, callback }`.
- `commands.rs`, `events.rs`, `keymap.rs`: the new APIs.
- `options.rs`: existing typed parsing, plus declarations, `gband.opt` metatable access and pending assignments.
- `api.rs`: `set`, `bind`, `unbind` and `spawn`, rewritten as thin layers over `options.rs` and `keymap.rs`.

### Callbacks are registry entries with owners
Every Lua function the runtime keeps (binding function, registered action, command, handler) goes into a `Callbacks` app-data vector of `{ function: RegistryKey, owner: Option<PluginName> }`, addressed by `CallbackId`. `Binding::Function(RegistryKey)` becomes `Binding::Callback(CallbackId)`. A binding to a registered action stores the callback plus the action's name for `keymap.list`. Disabling a plugin is then a set lookup at call time, so registrations made before the failure need no rewriting.

### The `Runtime` handle replaces the bare `Lua`
`Config { options, keymap, runtime, errors }`. `errors` holds the non-fatal errors of the load. `Runtime` wraps the `Lua` and offers `call(CallbackId)`, `emit(Event)` and `set_active_table(&str)`. Each call returns `Outcome { dispatched: Vec<Dispatch>, errors: Vec<ConfigError>, disabled: bool }`. `Dispatch` gains `Enter(String)` for `gband.keymap.enter`. Calling a registered action from Lua runs its function at once, nested and protected, so a registered action never appears as a `Dispatch`. Only built-in actions, spawns and table entries reach the client.

### Protection and the instruction budget
Plugin files, `setup` and callbacks run through `guard::run(owner, f)`. It pushes the owner, resets the budget when it is the outermost run, calls `f`, and turns an `mlua::Error` into a `ConfigError` carrying the plugin name. A hook via `Lua::set_hook(HookTriggers::new().every_nth_instruction(10_000), …)` subtracts from the budget and raises `instruction limit exceeded` once it reaches zero. It keeps raising every 10,000 instructions until the outermost run returns, so a plugin-level `pcall` cannot swallow the stop for long. Nested guarded runs, such as a registered action called from a binding function, re-raise the limit error instead of reporting it. The owner on top of the stack when the budget ran out is recorded and blamed once, at the outermost run. The budget is a field of the load options, defaulting to 100,000,000, so unit tests can use a small one. One integration test runs the real limit.
- Alternative: a wall-clock deadline checked in the hook. Rejected because results would vary with machine load and build profile. The request asks for an instruction count.

### Non-fatal versus fatal load errors
Errors from the init file's own code stay fatal (`Err(ConfigError)`), so "Last good configuration" is unchanged. `gband.plugin` and plugin-file sourcing catch through the guard and push to `errors`. `gband.opt` validation failures push to `errors` and reset the option to its declared default. `gband.set` keeps raising, as today.

### `gband.opt`
`gband.opt` is a table with `__index` and `__newindex` metamethods plus the plain fields `declare` and `list`. Reads of built-in options convert from `Options` back to Lua values. Writes of built-in options reuse the existing serde parsing of `OptionsPatch`. Declared options hold `OptValue::{Boolean, Integer, Number, String}`. Assignments to undeclared names go to a pending map with their location. `declare` validates any pending value, and loading finishes by reporting what is still pending.

### Key tables
`Keys`/`Chord` become `(table: String, key: Chord)`, with `Chord::{Key(Key), Prefix}` kept. `gband.bind` parses its string into that pair, and `keymap.set` takes it directly. Each table keeps a `Vec` in binding order, so `keymap.list` and replacement-in-place behave like today's `retain` and `push`. In the client, `Keymap` holds `BTreeMap<String, Vec<(Chord, Binding)>>`, and `Leader` holds the active table name instead of `after_prefix`. `Leader::handle` returns the command plus the table transition. `Controls` applies `Dispatch::Enter`, updates `Runtime::set_active_table`, and emits `KeyTableChanged` only when the name actually changes. "No prefix bindings means the prefix key passes through" stays a special case of the `prefix` table.

### Built-in events from observed state
`Controls` snapshots `Observed { focused, band, panes: BTreeMap<PaneId, BandId>, size }` from `Display` before and after each server message batch, key, dispatch and resize, and emits the differences. `Display` exposes a `first_view` flag, so the initial view and first layout emit nothing. Handlers' dispatches are applied, then the state is diffed again and the resulting events are emitted with depth + 1. Deliveries stop at depth 10 with a `tracing::warn!`. `Attached` is emitted once in `attach()` after `Controls::new`. `ConfigReloaded` is emitted into the new runtime right after `Controls::reload` swaps it in. No protocol message is added: everything comes from data the client already holds.

### Reload watching
`watch` registers `user/` recursively and keeps the existing fallback of watching the configuration directory while `user/` is missing. An event is relevant when any of its paths lies under `user/` and ends in `.lua`. `defaults/` stays unwatched, and so does the plugins directory.

### Default configuration text
`defaults.lua` keeps setting the options, through `gband.opt`, and binds every key with `gband.keymap.set("prefix", key, gband.action.<name>, { desc = <action description> })`. `prepare` rewrites the on-disk `defaults/init.lua` on the next start because its content differs. Users' copies of the old text keep loading, because `gband.set` and `gband.bind` are unchanged.

### `print`
The global `print` is replaced by a Rust function that joins `tostring` of its arguments with tabs and logs at info level, with the current owner as a `plugin` field. In the client, stdout is the terminal ratatui draws on, so today's `print` corrupts the screen.

## Risks / Trade-offs

- [Plugin top-level code runs in both processes] → The guide tells plugin authors to only register things at top level and do work in callbacks. `gband.side` stays `"client"` until a server runtime gives the server its own side.
- [The instruction limit cannot stop a blocking C call such as `os.execute("sleep 100")` or `io.read()`] → Documented in the guide. Sandboxing the standard library is out of scope.
- [A plugin's own `pcall` around a hot loop catches the limit error] → The hook re-raises every 10,000 instructions until the outermost run returns, so the loop cannot continue indefinitely.
- [Event feedback loops between handlers and actions] → The depth limit of 10 with a logged warning. The client keeps handling input.
- [Only the latest error shows on the banner when several plugins fail at start] → Every error is in the log with its plugin, file and line. A richer error surface arrives with later UI work.
- [The server pays for evaluating plugins on each reload] → Loads are rare, and the instruction budget bounds each one.
- [Conflicts with `rename-workspaces-to-bands` in `crates/lua`, `crates/client` and specs] → This change declares a dependency on it and starts after it is archived.

## Migration Plan

No user action is needed. Existing `user/init.lua` files keep their meaning: one init file, `gband.set`/`gband.bind`/`gband.unbind` unchanged. `prepare` replaces `defaults/init.lua` with the `gband.keymap.set` form on the first start of the new build. Rollback is reverting the change. Nothing persists beyond the mirror file, which the old build rewrites back.
