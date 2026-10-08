# plugins Specification

## Purpose

Defines how the client's Lua runtime finds, loads and isolates plugins: the runtimepath, module lookup, the files sourced at startup, the plugin module shape, which plugin owns each registered name and callback, and how a failing or runaway plugin is contained so the client keeps running.

## Requirements

### Requirement: Runtimepath
`gband.runtimepath` SHALL be a Lua list of directory paths. Each process SHALL build it from its own environment and the files of the machine it runs on: the server from the server's, and each client from its own. When loading starts, it SHALL hold the `user` directory of the configuration directory, followed by every directory directly under the plugins directory, in ascending byte order of their names. The plugins directory SHALL be `gband/plugins` under `$XDG_DATA_HOME` when that variable holds an absolute path, otherwise `.local/share/gband/plugins` under the user's home directory. A missing plugins directory SHALL leave only the `user` directory in the list, and SHALL NOT be an error. The init file MAY change the list. Module lookup, manifest evaluation and side file sourcing SHALL use the list as it stands when they run. An entry that does not exist SHALL contribute nothing and SHALL NOT be an error.

#### Scenario: Default runtimepath
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `XDG_DATA_HOME` is `/tmp/data`, and `/tmp/data/gband/plugins` holds the directories `zeta` and `alpha`
- **THEN** `gband.runtimepath` is `{ "/tmp/cfg/gband/user", "/tmp/data/gband/plugins/alpha", "/tmp/data/gband/plugins/zeta" }`

#### Scenario: Data home fallback
- **WHEN** `XDG_DATA_HOME` is unset and the home directory is `/home/u`
- **THEN** the plugins directory is `/home/u/.local/share/gband/plugins`

#### Scenario: Init file adds an entry
- **WHEN** `user/init.lua` appends `/opt/hello` to `gband.runtimepath`, and `/opt/hello` holds a valid manifest and `client.lua`
- **THEN** `/opt/hello/client.lua` is sourced after `user/init.lua`

#### Scenario: Each side reads its own machine
- **WHEN** the server runs with `XDG_DATA_HOME` `/srv/data` and a client with `XDG_DATA_HOME` `/home/u/data`
- **THEN** the server's runtimepath lists the directories under `/srv/data/gband/plugins`
- **AND** the client's lists those under `/home/u/data/gband/plugins`

### Requirement: Module lookup
`require(name)` SHALL look for the module in each runtimepath entry in order, before Lua's own search path: a name `a.b` SHALL be found at `lua/a/b.lua`, then at `lua/a/b/init.lua`, in each entry. When no entry holds the module, `require` SHALL look among the modules bundled with gband, and only then in Lua's own search path. The bundled modules are these:

- `gband.prelude` and the API modules `gband.hl`, `gband.palette`, `gband.colorscheme`, `gband.bar`, `gband.win`, `gband.settings` and `gband.keystyle`, as the lua-api capability defines;
- `gband.sidebar`, as the sidebar capability defines;
- `gband.errors`, as the error-list capability defines;
- `gband.prompt`, as the lua-prompt capability defines;
- the key style presets `gband.keystyle.modal`, `gband.keystyle.direct` and `gband.keystyle.floating`, as the key-style capability defines;
- `gband.desktop`, as the floating-key-style capability defines;
- `gband.keylist`, as the key-list capability defines;
- `gband.keyform`, which returns the function that turns a key name into the form the key-list capability's "Key form" defines;
- `gband.theme` and `gband.theme.catppuccin`, as the colorschemes capability's "Themes" defines.

The first file found SHALL be loaded, and errors in it SHALL name its path and line. Errors in a bundled module SHALL name its path under `gband/`, such as `gband/sidebar.lua`. A bundled module's chunk SHALL receive the same arguments as a module found on the runtimepath: the module name and its path, under `gband/` for a bundled module. `require` SHALL return the module's value and that path. A module in a runtimepath entry SHALL override the bundled module of the same name, the prelude and the API modules included.

#### Scenario: Module from a plugin directory
- **WHEN** `/tmp/data/gband/plugins/hello/lua/hello/init.lua` exists and `user/init.lua` calls `require("hello")`
- **THEN** that file is loaded

#### Scenario: Earlier entry wins
- **WHEN** both `user/lua/util.lua` and a plugin directory's `lua/util.lua` exist
- **THEN** `require("util")` loads `user/lua/util.lua`

#### Scenario: Submodule
- **WHEN** a runtimepath entry holds `lua/hello/keys.lua`
- **THEN** `require("hello.keys")` loads it

#### Scenario: Bundled module
- **WHEN** no runtimepath entry holds `lua/gband/sidebar.lua`
- **THEN** `require("gband.sidebar")` loads the bundled module

#### Scenario: Bundled key form
- **WHEN** `user/init.lua` calls `require("gband.keyform")("ctrl+space")`
- **THEN** it returns `C-space`

#### Scenario: Bundled prompt
- **WHEN** `user/init.lua` calls `local module, path = require("gband.prompt")`
- **THEN** `module.name` is `prompt` and `path` is `gband/prompt.lua`

#### Scenario: Bundled module shadowed
- **WHEN** `user/lua/gband/sidebar.lua` exists
- **THEN** `require("gband.sidebar")` loads `user/lua/gband/sidebar.lua`

#### Scenario: API module is a module
- **WHEN** no runtimepath entry holds `lua/gband/hl.lua` and `user/init.lua` calls `local hl = require("gband.hl")`
- **THEN** `hl` is the table `package.loaded["gband.hl"]` holds, and `hl.drawn` is a function defined in `gband/hl.lua`

#### Scenario: Bundled chunk arguments
- **WHEN** `user/lua/probe.lua` returns `{ ... }`, and `user/init.lua` compares what `require("probe")` returns with what the bundled `gband.keyform` chunk receives
- **THEN** both hold the module name and the module's path, and nothing else

#### Scenario: Bundled desktop plugin
- **WHEN** no runtimepath entry holds `lua/gband/desktop.lua` and `user/init.lua` calls `gband.plugin("gband.desktop")`
- **THEN** the bundled module loads and `gband.action.list()` holds `desktop.list`

### Requirement: Plugin files
After the init file returns, loading SHALL source, for each plugin in runtimepath order whose manifest is valid, its side file once: `client.lua` in a client, and `server.lua` in the server. A plugin MAY hold only one side file, and the other side SHALL then source nothing of it without an error. The `user` directory SHALL have no side file: its init file is `user/init.lua` in a client and `user/server.lua` in the server. A runtimepath entry without a manifest SHALL contribute its modules and colorschemes only; when it holds `client.lua` or `server.lua`, the process SHALL report a plugin error naming the missing manifest and source neither. No file under a `plugin` directory, in any entry, SHALL be sourced.

#### Scenario: Load order
- **WHEN** the runtimepath holds `user`, then the plugins `alpha` and `beta`, each with a valid manifest and a `client.lua`, and each file appends its name to a shared list
- **THEN** the client's list after loading is `init`, `alpha`, `beta`, where `init` is appended by `user/init.lua`

#### Scenario: One file per side
- **WHEN** a plugin holds a valid manifest, a `server.lua` that appends `s` to a window's state key `log`, and a `client.lua` that raises an error
- **THEN** the server sources only `server.lua` and reports no error
- **AND** the client sources only `client.lua` and reports its error

#### Scenario: Client-only plugin
- **WHEN** a plugin holds a valid manifest and only `client.lua`
- **THEN** the server loads it with no error and sources nothing of it

#### Scenario: Legacy plugin files are ignored
- **WHEN** a plugin directory holds `plugin/keys.lua` and `plugin/client/k.lua`, each raising an error
- **THEN** neither process sources them and neither reports an error

#### Scenario: Server plugin files are ignored
- **WHEN** a plugin directory holds `plugin/server/s.lua`, which raises an error
- **THEN** neither process reports an error and `s.lua` is never run

#### Scenario: Default configuration with a plugin
- **WHEN** no `user/init.lua` exists and a plugin's `client.lua` binds `alt+g` with `gband.keymap.set`
- **THEN** the default bindings apply and Alt+G runs the plugin's binding

### Requirement: Side and API version
`gband.side` SHALL be the string `"client"` in a client, `"server"` in the server, and `"test"` in a test file that `gband test` runs, as the plugin-testing capability defines. `gband.api_version` SHALL be the integer `2` on every side. Both SHALL be set before the init file or the test file runs.

#### Scenario: Side and version
- **WHEN** `user/init.lua` reads `gband.side` and `gband.api_version`
- **THEN** they are `"client"` and `2`

#### Scenario: Server side
- **WHEN** `user/server.lua` reads `gband.side`
- **THEN** it is `"server"`

#### Scenario: Test side
- **WHEN** a test file read by `gband test -` prints `gband.side`
- **THEN** standard output holds `test`

### Requirement: Plugin modules
A plugin module SHALL be a table with a `setup` function, an optional string `name` and an optional integer `api`. `gband.plugin(name, opts)` SHALL require the module `name`, check its shape, and call its `setup` with `opts`, or with an empty table when `opts` is nil. The plugin's name SHALL be the module's `name` field when it has one, and `name` otherwise. When the module's `api` differs from `gband.api_version`, the process SHALL record a warning naming the plugin and both versions in its log, and SHALL still set the plugin up. `gband.plugin` SHALL return `true` when `setup` returns, and `false` when the module cannot be found, has the wrong shape, or its `setup` raises an error, each reported as a plugin error. Calling `gband.plugin` again for a plugin already set up in this load SHALL be a plugin error, and SHALL NOT call `setup` again.

#### Scenario: Setup receives the options
- **WHEN** `user/init.lua` calls `gband.plugin("hello", { greeting = "hi" })` and `hello`'s `setup` stores `opts.greeting`
- **THEN** the stored value is `"hi"`
- **AND** `gband.plugin` returns `true`

#### Scenario: API mismatch
- **WHEN** a plugin module declares `api = 1`
- **THEN** the log records a warning naming the plugin, version 1 and version 2
- **AND** its `setup` runs

#### Scenario: Missing module
- **WHEN** `user/init.lua` calls `gband.plugin("absent")` and no runtimepath entry holds it
- **THEN** `gband.plugin` returns `false` and a plugin error names `absent`
- **AND** the rest of `user/init.lua` runs

#### Scenario: Wrong shape
- **WHEN** the module `hello` returns a table without `setup`
- **THEN** `gband.plugin("hello")` returns `false` and a plugin error names `hello`

### Requirement: Ownership and namespaces
Every plugin SHALL have a name. A file sourced from a runtimepath entry other than the `user` directory SHALL belong to the plugin named by that entry's last path component. Code run by `gband.plugin` SHALL belong to the plugin it sets up. A callback, a registered action, a registered command and a declared option SHALL belong to the plugin whose code registered it, and code run by a callback SHALL belong to the callback's plugin. The init file, the default configuration and files from the `user` directory SHALL belong to no plugin.

A name that a plugin registers for an action or a command, or declares for an option, SHALL be its full name `<plugin>.<name>`: a name that does not begin with `<plugin>.` SHALL be prefixed with it, and a name that contains a `.` but does not begin with `<plugin>.` SHALL be an error. A name registered by code that belongs to no plugin SHALL be used as given.

#### Scenario: Plugin action is namespaced
- **WHEN** the plugin `hello` calls `gband.action.register("greet", fn)`
- **THEN** `gband.action["hello.greet"]` holds the action and `gband.action.greet` is nil

#### Scenario: Already namespaced
- **WHEN** the plugin `hello` calls `gband.cmd.register("hello.say", fn)`
- **THEN** the command's full name is `hello.say`

#### Scenario: Foreign namespace
- **WHEN** the plugin `hello` calls `gband.action.register("other.greet", fn)` on line 4 of its file
- **THEN** a plugin error at that line names `other.greet`

#### Scenario: User names are not namespaced
- **WHEN** `user/init.lua` calls `gband.action.register("greet", fn)`
- **THEN** `gband.action.greet` holds the action

### Requirement: Plugin errors
Evaluating a plugin's manifest, sourcing a plugin's side file, setting a plugin up and running a callback that belongs to a plugin SHALL each run protected, so that an error raised there SHALL NOT escape to the code around it. Such an error is a plugin error. It SHALL be recorded in the process's log with the plugin's name, the file and the line, and reported as the configuration capability defines.

A plugin error while loading SHALL NOT fail the load. It SHALL mark the plugin failed for the rest of that load: its side file SHALL NOT be sourced when it is not yet, and every callback that belongs to it SHALL be disabled, whether registered before or after the error. A key bound to a disabled callback SHALL do nothing when pressed, a disabled event handler SHALL NOT run, running a disabled command SHALL return `false`, and calling a disabled registered action SHALL do nothing. Options the plugin declared SHALL keep their values.

A plugin error while a callback runs after loading SHALL be reported, and SHALL leave the callback enabled. The actions it dispatched before the error SHALL stand, and other callbacks SHALL be unaffected.

#### Scenario: Setup error
- **WHEN** the plugin `broken` registers the action `broken.go`, binds `alt+b` to it, then raises an error on line 6 of its file in `setup`
- **THEN** the client starts with the configuration's other bindings and options
- **AND** the client shows a plugin error naming `broken`, its file and line 6
- **AND** Alt+B does nothing

#### Scenario: Error in a plugin file
- **WHEN** a plugin's `client.lua` binds `alt+b` on line 1 and raises an error on line 2
- **THEN** the load succeeds, the plugin is failed, and Alt+B does nothing

#### Scenario: Error in a callback
- **WHEN** a plugin's handler for `FocusChanged` raises an error, and a second handler for `FocusChanged` belongs to another plugin
- **THEN** the second handler runs, the error is reported, and the first handler runs again on the next focus change

#### Scenario: Error in a server handler
- **WHEN** a plugin's `server.lua` handler of `WindowOpened` raises an error, and another plugin's handler of `WindowOpened` sets a window state key
- **THEN** the key is set, and the server keeps running its windows

### Requirement: Instruction limit
A run of Lua code started by the runtime, whether the init file, a plugin file, a plugin's setup or one call of a callback, SHALL be stopped with an error once it has executed 100,000,000 Lua VM instructions. A stopped plugin file, setup or callback SHALL be a plugin error that marks its plugin failed, as "Plugin errors" defines, also when the callback runs after loading. A stopped init file SHALL fail the load. A stopped callback that belongs to no plugin SHALL be reported as an error raised in it.

#### Scenario: Infinite loop in setup
- **WHEN** a plugin's `setup` runs `while true do end`
- **THEN** the client starts
- **AND** a plugin error names the plugin and the instruction limit

#### Scenario: Infinite loop in a callback
- **WHEN** a key is bound to a plugin's action whose function runs `while true do end`, and the user presses that key
- **THEN** the client keeps running and handles the next key
- **AND** pressing the key again does nothing

### Requirement: Print goes to the log
In a client and in the server, `print` SHALL write its arguments, converted as Lua's `tostring` converts them and separated by tabs, to the process's log at the info level, naming the plugin the calling code belongs to when it belongs to one. It SHALL NOT write to the terminal. In the test side, `print` SHALL write the same line, followed by a newline, to the standard output of `gband test`.

#### Scenario: Print from a plugin
- **WHEN** the plugin `hello` calls `print("ready", 3)` while the client runs
- **THEN** the client log records `ready	3` naming `hello`
- **AND** the client's screen is unchanged

#### Scenario: Print in a test
- **WHEN** a case calls `print("step", 2)`
- **THEN** the standard output of `gband test` holds the line `step	2`

### Requirement: Plugin manifest
A runtimepath entry other than the `user` directory SHALL be a plugin when it holds a file `plugin.lua`, its manifest. Loading SHALL evaluate each manifest, in runtimepath order, before sourcing any side file, as code that belongs to the plugin, in an environment holding no globals, so that it can only compute and return data. The manifest SHALL return a table holding `name`, a string equal to the entry's last path component, `version`, a version as the plugin-bridge capability defines, and optionally `client`, a requirement as the plugin-bridge capability defines, which only the server reads. A manifest that raises an error, returns anything else, or names another plugin SHALL be a plugin error that marks the plugin failed, and its side file SHALL NOT be sourced.

`gband.plugins()` SHALL return one table per plugin whose manifest the load evaluated, in runtimepath order, each holding `name`, `version`, `client` and `failed`, the last `true` when the plugin is failed. Changing a returned table SHALL change nothing else.

#### Scenario: Valid manifest
- **WHEN** `plugins/agent-status/plugin.lua` returns `{ name = "agent-status", version = "0.1.0" }`
- **THEN** `gband.plugins()` holds one entry with `name` `agent-status`, `version` `0.1.0` and `failed` `false`

#### Scenario: Name mismatch
- **WHEN** `plugins/agent-status/plugin.lua` returns `{ name = "agents", version = "1" }`, and `plugins/agent-status/client.lua` exists
- **THEN** a plugin error names `agent-status` and `agents`
- **AND** `client.lua` is not sourced

#### Scenario: Manifest cannot reach gband
- **WHEN** a manifest calls `gband.keymap.set("alt+x", fn)` on line 1
- **THEN** a plugin error at that line is reported and no binding is made

### Requirement: Side guard
Each process's `gband` table SHALL hold only the API of its side. Reading a field of `gband`, or of `gband.action`, that only another side provides SHALL raise an error naming the field and the side that provides it, at the line of the read. The fields only the client provides SHALL be `bind`, `unbind`, `spawn`, `keymap`, `keystyle`, `settings`, `ui`, `hl`, `colorscheme`, `palette`, `layout`, `view`, `window`, `band`, `win`, `bar`, `errors`, `clear_errors`, `rpc`, `notify`, `bell`, `clipboard`, `open`, `core`, and the view and client actions in `gband.action`. The fields only the server provides SHALL be `sessions` and `session`. Every other field the plugins, configuration, lua-events and lua-commands capabilities define SHALL exist on the client and the server, with each side's own behaviour where the server-runtime capability defines one. The test side's `gband` SHALL hold only `side`, `api_version` and `core`, whose fields are the test-side primitives the plugin-testing capability's API documentation gives: `register` and `wrap`. Reading any other field that the client or the server provides there SHALL raise an error naming the field and the sides that provide it.

#### Scenario: Client API in the server
- **WHEN** line 3 of a plugin's `server.lua` reads `gband.keymap`
- **THEN** a plugin error at that line names `gband.keymap` and the client

#### Scenario: Key style API in the server
- **WHEN** line 2 of `user/server.lua` calls `gband.keystyle.use()`
- **THEN** loading fails with an error at that line naming `gband.keystyle` and the client

#### Scenario: Settings API in the server
- **WHEN** line 2 of `user/server.lua` calls `gband.settings.theme()`
- **THEN** loading fails with an error at that line naming `gband.settings` and the client

#### Scenario: Palette in the server
- **WHEN** line 3 of a plugin's `server.lua` calls `gband.palette.get()`
- **THEN** a plugin error at that line names `gband.palette` and the client

#### Scenario: Server API in the client
- **WHEN** line 2 of `user/init.lua` calls `gband.sessions()`
- **THEN** loading fails with an error at that line naming `gband.sessions` and the server

#### Scenario: View action in the server
- **WHEN** a server handler calls `gband.action.focus_column_left()`
- **THEN** a plugin error names `focus_column_left` and the client

#### Scenario: Client API in a test file
- **WHEN** line 4 of a test file reads `gband.opt`
- **THEN** the file fails with an error at line 4 naming `gband.opt`, the client and the server

#### Scenario: Bars in the server
- **WHEN** line 2 of a plugin's `server.lua` reads `gband.bar`
- **THEN** a plugin error at that line names `gband.bar` and the client

#### Scenario: Clearing errors in the server
- **WHEN** line 4 of a plugin's `server.lua` calls `gband.clear_errors()`
- **THEN** a plugin error at that line names `gband.clear_errors` and the client

#### Scenario: Configuration directory in a test file
- **WHEN** line 2 of a test file reads `gband.config_dir`
- **THEN** the file fails with an error at line 2 naming `gband.config_dir`, the client and the server

#### Scenario: Core primitives in the server
- **WHEN** line 2 of `user/server.lua` calls `gband.core.owner()`
- **THEN** loading fails with an error at that line naming `gband.core` and the client

#### Scenario: Test primitives
- **WHEN** a test file reads `gband.core.register`
- **THEN** it is a function, and reading `gband.core.owner` there raises an error naming the client
