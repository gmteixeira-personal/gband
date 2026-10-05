## 1. Loader foundations

- [ ] 1.1 Confirm `rename-workspaces-to-bands` is archived on `<base>`, and that `BandId`, `View::band` and `focus_band_down`/`focus_band_up` exist. Verify with `rg -n "focus_band_down|fn band" crates/`.
- [ ] 1.2 Add `Locations { config, plugins }` and a `plugins_dir_from(xdg_data_home, home)` beside `config_dir_from` in `crates/lua/src/directory.rs`. Change `load` and `watch` to take `Locations` and a load-options value carrying the instruction budget. Verify with unit tests for the `XDG_DATA_HOME` absolute, relative and unset cases.
- [ ] 1.3 Build `Locations` in `src/main.rs` for both the server and the client paths. The server still sends only `config.options.layout` through its `watch` channel. Verify with `cargo build` and the existing `tests/config.rs` width scenarios passing unchanged.
- [ ] 1.4 Make `tests/common/mod.rs` set `XDG_DATA_HOME` to a scratch directory for every spawned process, so tests never read the real plugins directory. Verify that `cargo test --test config` passes.

## 2. Ownership, guard and instruction budget

- [ ] 2.1 Add `owner.rs`, with an owner stack and a failed-plugin set as app data, and add `plugin: Option<String>` to `ConfigError`, prefixing `Display` with `<plugin>: `. Verify with a unit test of the display form `hello: /p/x.lua:3: boom`.
- [ ] 2.2 Add `guard.rs`: a protected run that pushes the owner, converts errors to `ConfigError` with the plugin, and installs the `every_nth_instruction(10_000)` hook. The budget resets at the outermost run, the hook keeps raising after exhaustion, nested runs re-raise the limit error, and the plugin blamed is the owner on top when the budget ran out. Verify with unit tests: `while true do end` stops under a small budget, a `pcall`-wrapped loop still stops, and a nested stop blames the inner owner.
- [ ] 2.3 Replace the global `print` with a function that logs tab-joined `tostring` values at info level, with the owner as a field. Verify with a unit test that `print` writes nothing to stdout, and that a captured tracing subscriber records the text and the plugin.

## 3. Runtimepath, modules and plugin files

- [ ] 3.1 Add `runtime.rs`. Set `gband.runtimepath` from `Locations` (user dir, then plugin dirs in byte order), `gband.side = "client"` and `gband.api_version = 1`. Verify with `crates/lua/tests/plugins.rs` covering the "Default runtimepath" and "Side and version" scenarios.
- [ ] 3.2 Insert the runtimepath searcher at `package.searchers[2]`, resolving `lua/a/b.lua` then `lua/a/b/init.lua` per entry, with chunks named `@<path>`. Verify with tests for "Module from a plugin directory", "Earlier entry wins", "Submodule", and an error inside a module reporting its path and line.
- [ ] 3.3 After the init file, source `plugin/*.lua` then `plugin/client/*.lua` per entry in byte order, using `gband.runtimepath` as it stands. Each file is guarded and owned by the entry's last path component, or by no plugin for the user entry. A failing file marks its plugin failed and skips its remaining files. Verify with tests for "Load order", "Server plugin files are ignored", "Init file adds an entry" and "Error in a plugin file".
- [ ] 3.4 Implement `gband.plugin(name, opts)`: guarded `require`, shape check, `api` mismatch warning, guarded `setup(opts or {})`, the boolean result, and rejecting a second setup. Verify with tests for "Setup receives the options", "API mismatch", "Missing module", "Wrong shape" and "Setup error", the last checking that its registered callbacks are disabled.
- [ ] 3.5 Return non-fatal errors in `Config::errors`, while init-file errors stay `Err`. Verify with a test for "Plugin error keeps the rest" and the existing "Broken file at start" test unchanged.

## 4. Actions and commands

- [ ] 4.1 Add the `Callbacks` registry (`CallbackId`, function, owner) and change `Binding::Function` to `Binding::Callback`. Check the failed set before every call. Verify that the existing binding-function tests in `crates/lua/tests/config.rs` pass after migration.
- [ ] 4.2 Move the built-in action table to `actions.rs` with descriptions from the actions spec. Add `gband.action.register` (namespacing, duplicate, built-in name and `register`/`list` rejection, load-only), `gband.action.list()`, and `LuaAction::Registered`, whose call runs its callback at once, nested and guarded. Verify with tests for "Every action is named", "Built-in descriptions", "Plugin action is namespaced", "Foreign namespace", "User names are not namespaced", "Built-in name taken" and "Registered action called from another callback".
- [ ] 4.3 Add `commands.rs` with `gband.cmd.register`, `run` and `list`, as the lua-commands spec defines. Verify with `crates/lua/tests/commands.rs` covering every lua-commands scenario, plus "Already namespaced" and a disabled command returning `false`.

## 5. Options

- [ ] 5.1 Add declarations to `options.rs`: built-in option metadata (type, default, desc), `OptValue`, and `gband.opt.declare` with types `boolean`/`integer`/`number`/`string`, an optional `values` list and plugin namespacing. Verify with unit tests for valid and invalid declarations.
- [ ] 5.2 Implement `gband.opt` reads and writes through `__index`/`__newindex`. Built-in writes use the existing `OptionsPatch` parsing. An invalid value pushes a located error and resets the option to its default. Undeclared names are held as pending, validated on declare, and reported at the end of the load. Writes and declares are load-only. Add `gband.opt.list()`. Verify with `crates/lua/tests/options.rs` covering "Option read and set through gband.opt", "Invalid value falls back to the default", "Declared plugin option", "Set before declared", "Never declared" and "Value outside the allowed list".
- [ ] 5.3 Keep `gband.set` raising on invalid input. Verify that the existing option tests in `crates/lua/src/options.rs` and `crates/lua/tests/config.rs` pass unchanged.

## 6. Keymap

- [ ] 6.1 Replace `Keys` with `(table, Chord)` and add `keymap.rs` with `gband.keymap.set`, `del`, `list`, `current_table` and `enter`. `gband.bind`/`unbind` map onto it. Add `Dispatch::Enter`. Errors cover a non-string table, `prefix` in `root`, load-only edits, `enter` outside a callback, and `enter` of an empty table. Verify with `crates/lua/tests/keymap.rs` covering "Keymap binding with a description", "Old and new forms agree", "Prefix in the root table", "Binding after the load", and `list` order and replacement.
- [ ] 6.2 Generalise `crates/client/src/bindings.rs`: `Keymap` holds named tables, and `Leader` tracks the active table name and returns transitions. Prefix pass-through applies when `prefix` is empty. Verify that every existing test in `bindings.rs` passes, plus new tests for "Named key table", "Unbound key in a named table" and a binding in `root` entering a table.

## 7. Events

- [ ] 7.1 Add `events.rs`: `gband.on` (event-name validation, `group`, `once`, `pattern` for `User` only), `gband.augroup` (stable ids per load, `clear` defaulting to true), `gband.emit` for `User`, and `Runtime::emit` delivering a fresh payload table to each guarded handler in registration order. Verify with `crates/lua/tests/events.rs` covering "Once", "Payload copies are separate", "Unknown event", "Clear on redefinition", "Keep without clear", "Group by name", "Pattern match" and "Error in a callback".
- [ ] 7.2 Add the `Runtime` handle (`call`, `emit`, `set_active_table`) returning `Outcome { dispatched, errors, disabled }`, and replace `Config::lua` and `bindings` with `runtime` and `keymap`. Verify with `cargo build --workspace` and the `crates/lua` tests.
- [ ] 7.3 In `crates/client/src/lib.rs`, snapshot `Observed { focused, band, panes, size }` around each message batch, key, dispatch and resize, and emit `FocusChanged`, `BandChanged`, `PaneOpened`, `PaneClosed` and `TerminalResized`. Nothing is emitted for the first view and first layout. Apply handler dispatches, re-diff, and stop at depth 10 with a warning. Verify with `crates/client/tests/events.rs`, driving `Display`/`Controls` with layouts and keys, covering "Focus change", "Band change", "Pane opened and closed", "Nothing at attach", "Handler dispatches an action" and "Event loop is cut".
- [ ] 7.4 Emit `Attached` once in `attach()`. Emit `ConfigReloaded` into the new runtime after `Controls::reload`. Emit `KeyTableChanged` from `Leader` transitions and `Dispatch::Enter`. A reload makes `root` active. Verify with client tests for "Key table change events", "Current table", "Reload", and an `Attached` handler running once.
- [ ] 7.5 Route callback and handler errors and `Config::errors` to the banner and the log, the banner showing the latest. A disabled callback's key does nothing. Verify with client tests for "Error in a binding function", "Plugin error on the banner" and "Infinite loop in a callback", the last using a small budget and checking that the next key is handled and the second press does nothing.

## 8. Default configuration and reload

- [ ] 8.1 Rewrite `crates/lua/src/defaults.lua` to set options through `gband.opt` and to bind every default key with `gband.keymap.set("prefix", …, { desc = <action description> })`. Verify that `default_keys_follow_the_spec` and `every_default_binding_names_an_action` pass, and add a test for "Every default binding is described".
- [ ] 8.2 Make `watch` follow `user/` recursively for `.lua` files, keeping the parent fallback. Verify with a watch test for "User plugin file edited" and the existing `defaults_file_is_not_watched` and parent-fallback tests.
- [ ] 8.3 Add integration tests in `tests/config.rs` that run real client and server processes: "Plugin sets a width option" (a plugin-set `default_column_width` reaches the server), "Default configuration with a plugin", and "Infinite loop in setup" with the real 100,000,000 budget. Verify with `cargo test --test config`.

## 9. Sample plugin and docs

- [ ] 9.1 Add `examples/plugins/hello/lua/hello/init.lua`, returning `{ name = "hello", api = 1, setup = … }`. Its `setup` declares the option `greeting`, registers the action `greet` (spawning a pane that prints the greeting and then runs the shell), registers the command `say` with `args = { "who" }`, and adds a `FocusChanged` handler in augroup `hello` that `print`s the pane. Add `examples/plugins/hello/README.md`, showing installation by symlink into the plugins directory and an `init.lua` that calls `gband.plugin("hello", { greeting = "hi" })` and binds `gband.keymap.set("prefix", "g", gband.action["hello.greet"], { desc = "Greet" })`. Verify with a `crates/lua` test that loads that `init.lua` with the example on the runtimepath and checks the action, command, option and handler.
- [ ] 9.2 Write `docs/plugins.md`: the runtimepath and directory layout (`lua/`, `plugin/`, `plugin/client/`, reserved `plugin/server/`), load order, the module shape and `gband.plugin`, namespacing, error isolation and the instruction limit and its limits, and the APIs `gband.action`, `gband.cmd`, `gband.opt`, `gband.on`/`augroup`/`emit` with the event table, `gband.keymap` and `print`. Add advice to only register at top level, because the server evaluates plugins too, and a section naming the extension points planned for later changes (server runtime, status line, overlays and palette, notifications, highlight groups). Verify that every API in the lua-events, lua-commands and plugins specs appears in the guide.
- [ ] 9.3 Update the configuration section of `README.md`: show `gband.keymap.set` beside `gband.bind`, mention `gband.opt`, and link `docs/plugins.md`. Verify by reading the section against the configuration spec.

## 10. Gate

- [ ] 10.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, and confirm they pass.
- [ ] 10.2 Manual check: with an empty data directory and no `user/init.lua`, attach, and confirm that every default prefix binding and Ctrl+A pass-through behave as before. Then install `examples/plugins/hello` and bind `prefix g` from `user/init.lua`, and confirm that the greeting pane opens and the focus handler logs to the client log.
