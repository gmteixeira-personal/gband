# plugins Specification

## Purpose

Defines how the client's Lua runtime finds, loads and isolates plugins: the runtimepath, module lookup, the files sourced at startup, the plugin module shape, which plugin owns each registered name and callback, and how a failing or runaway plugin is contained so the client keeps running.

## Requirements

### Requirement: Runtimepath
`gband.runtimepath` SHALL be a Lua list of directory paths. When loading starts, it SHALL hold the `user` directory of the configuration directory, followed by every directory directly under the plugins directory, in ascending byte order of their names. The plugins directory SHALL be `gband/plugins` under `$XDG_DATA_HOME` when that variable holds an absolute path, otherwise `.local/share/gband/plugins` under the user's home directory. A missing plugins directory SHALL leave only the `user` directory in the list, and SHALL NOT be an error. The init file MAY change the list. Module lookup and plugin sourcing SHALL use the list as it stands when they run. An entry that does not exist SHALL contribute nothing and SHALL NOT be an error.

#### Scenario: Default runtimepath
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `XDG_DATA_HOME` is `/tmp/data`, and `/tmp/data/gband/plugins` holds the directories `zeta` and `alpha`
- **THEN** `gband.runtimepath` is `{ "/tmp/cfg/gband/user", "/tmp/data/gband/plugins/alpha", "/tmp/data/gband/plugins/zeta" }`

#### Scenario: Data home fallback
- **WHEN** `XDG_DATA_HOME` is unset and the home directory is `/home/u`
- **THEN** the plugins directory is `/home/u/.local/share/gband/plugins`

#### Scenario: Init file adds an entry
- **WHEN** `user/init.lua` appends `/opt/hello` to `gband.runtimepath` and `/opt/hello/plugin/hello.lua` exists
- **THEN** `/opt/hello/plugin/hello.lua` is sourced after `user/init.lua`

### Requirement: Module lookup
`require(name)` SHALL look for the module in each runtimepath entry in order, before Lua's own search path: a name `a.b` SHALL be found at `lua/a/b.lua`, then at `lua/a/b/init.lua`, in each entry. The first file found SHALL be loaded, and errors in it SHALL name its path and line.

#### Scenario: Module from a plugin directory
- **WHEN** `/tmp/data/gband/plugins/hello/lua/hello/init.lua` exists and `user/init.lua` calls `require("hello")`
- **THEN** that file is loaded

#### Scenario: Earlier entry wins
- **WHEN** both `user/lua/util.lua` and a plugin directory's `lua/util.lua` exist
- **THEN** `require("util")` loads `user/lua/util.lua`

#### Scenario: Submodule
- **WHEN** a runtimepath entry holds `lua/hello/keys.lua`
- **THEN** `require("hello.keys")` loads it

### Requirement: Plugin files
After the init file returns, loading SHALL source, for each runtimepath entry in order, every file matching `plugin/*.lua` in ascending byte order of their names, then every file matching `plugin/client/*.lua` in the same order. Files in `plugin/server/`, in any other subdirectory of `plugin`, or not ending in `.lua` SHALL NOT be sourced. `plugin/server/` SHALL be reserved for a server runtime. Each file SHALL be sourced once per load.

#### Scenario: Load order
- **WHEN** the runtimepath holds `user` then the plugin directory `hello`, and `user/plugin/b.lua`, `user/plugin/a.lua`, `user/plugin/client/c.lua` and `hello/plugin/d.lua` each append their name to a shared list
- **THEN** the list after loading is `init`, `a`, `b`, `c`, `d`, where `init` is appended by `user/init.lua`

#### Scenario: Server plugin files are ignored
- **WHEN** a plugin directory holds `plugin/server/s.lua`, which raises an error
- **THEN** loading reports no error and `s.lua` is never run

#### Scenario: Default configuration with a plugin
- **WHEN** no `user/init.lua` exists and a plugin directory's `plugin/keys.lua` binds `alt+g` with `gband.keymap.set`
- **THEN** the default bindings apply and Alt+G runs the plugin's binding

### Requirement: Side and API version
`gband.side` SHALL be the string `"client"`. `gband.api_version` SHALL be the integer `1`. Both SHALL be set before the init file runs.

#### Scenario: Side and version
- **WHEN** `user/init.lua` reads `gband.side` and `gband.api_version`
- **THEN** they are `"client"` and `1`

### Requirement: Plugin modules
A plugin module SHALL be a table with a `setup` function, an optional string `name` and an optional integer `api`. `gband.plugin(name, opts)` SHALL require the module `name`, check its shape, and call its `setup` with `opts`, or with an empty table when `opts` is nil. The plugin's name SHALL be the module's `name` field when it has one, and `name` otherwise. When the module's `api` differs from `gband.api_version`, the process SHALL record a warning naming the plugin and both versions in its log, and SHALL still set the plugin up. `gband.plugin` SHALL return `true` when `setup` returns, and `false` when the module cannot be found, has the wrong shape, or its `setup` raises an error, each reported as a plugin error. Calling `gband.plugin` again for a plugin already set up in this load SHALL be a plugin error, and SHALL NOT call `setup` again.

#### Scenario: Setup receives the options
- **WHEN** `user/init.lua` calls `gband.plugin("hello", { greeting = "hi" })` and `hello`'s `setup` stores `opts.greeting`
- **THEN** the stored value is `"hi"`
- **AND** `gband.plugin` returns `true`

#### Scenario: API mismatch
- **WHEN** a plugin module declares `api = 2`
- **THEN** the log records a warning naming the plugin, version 2 and version 1
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
Sourcing a plugin's file, setting a plugin up and running a callback that belongs to a plugin SHALL each run protected, so that an error raised there SHALL NOT escape to the code around it. Such an error is a plugin error. It SHALL be recorded in the process's log with the plugin's name, the file and the line, and reported as the configuration capability defines.

A plugin error while loading SHALL NOT fail the load. It SHALL mark the plugin failed for the rest of that load: its remaining plugin files SHALL NOT be sourced, and every callback that belongs to it SHALL be disabled, whether registered before or after the error. A key bound to a disabled callback SHALL do nothing when pressed, a disabled event handler SHALL NOT run, running a disabled command SHALL return `false`, and calling a disabled registered action SHALL do nothing. Options the plugin declared SHALL keep their values.

A plugin error while a callback runs after loading SHALL be reported, and SHALL leave the callback enabled. The actions it dispatched before the error SHALL stand, and other callbacks SHALL be unaffected.

#### Scenario: Setup error
- **WHEN** the plugin `broken` registers the action `broken.go`, binds `alt+b` to it, then raises an error on line 6 of its file in `setup`
- **THEN** the client starts with the configuration's other bindings and options
- **AND** the banner shows a plugin error naming `broken`, its file and line 6
- **AND** Alt+B does nothing

#### Scenario: Error in a plugin file
- **WHEN** a plugin directory's `plugin/a.lua` raises an error and its `plugin/b.lua` exists
- **THEN** `b.lua` is not sourced and the load succeeds

#### Scenario: Error in a callback
- **WHEN** a plugin's handler for `FocusChanged` raises an error, and a second handler for `FocusChanged` belongs to another plugin
- **THEN** the second handler runs, the error is reported, and the first handler runs again on the next focus change

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
`print` SHALL write its arguments, converted as Lua's `tostring` converts them and separated by tabs, to the process's log at the info level, naming the plugin the calling code belongs to when it belongs to one. It SHALL NOT write to the terminal.

#### Scenario: Print from a plugin
- **WHEN** the plugin `hello` calls `print("ready", 3)` while the client runs
- **THEN** the client log records `ready	3` naming `hello`
- **AND** the client's screen is unchanged
