## MODIFIED Requirements

### Requirement: Module lookup
`require(name)` SHALL look for the module in each runtimepath entry in order, before Lua's own search path: a name `a.b` SHALL be found at `lua/a/b.lua`, then at `lua/a/b/init.lua`, in each entry. When no entry holds the module, `require` SHALL look among the modules bundled with gband, which the status-line capability lists, and only then in Lua's own search path. The first file found SHALL be loaded, and errors in it SHALL name its path and line. Errors in a bundled module SHALL name its path under `gband/`, such as `gband/statusline/band.lua`.

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
- **WHEN** no runtimepath entry holds `lua/gband/statusline/band.lua`
- **THEN** `require("gband.statusline.band")` loads the bundled module

#### Scenario: Bundled module shadowed
- **WHEN** `user/lua/gband/statusline/band.lua` exists
- **THEN** `require("gband.statusline.band")` loads `user/lua/gband/statusline/band.lua`

### Requirement: Plugin errors
Sourcing a plugin's file, setting a plugin up and running a callback that belongs to a plugin SHALL each run protected, so that an error raised there SHALL NOT escape to the code around it. Such an error is a plugin error. It SHALL be recorded in the process's log with the plugin's name, the file and the line, and reported as the configuration capability defines.

A plugin error while loading SHALL NOT fail the load. It SHALL mark the plugin failed for the rest of that load: its remaining plugin files SHALL NOT be sourced, and every callback that belongs to it SHALL be disabled, whether registered before or after the error. A key bound to a disabled callback SHALL do nothing when pressed, a disabled event handler SHALL NOT run, running a disabled command SHALL return `false`, and calling a disabled registered action SHALL do nothing. Options the plugin declared SHALL keep their values.

A plugin error while a callback runs after loading SHALL be reported, and SHALL leave the callback enabled. The actions it dispatched before the error SHALL stand, and other callbacks SHALL be unaffected.

#### Scenario: Setup error
- **WHEN** the plugin `broken` registers the action `broken.go`, binds `alt+b` to it, then raises an error on line 6 of its file in `setup`
- **THEN** the client starts with the configuration's other bindings and options
- **AND** the client shows a plugin error naming `broken`, its file and line 6, in the status line's error item
- **AND** Alt+B does nothing

#### Scenario: Error in a plugin file
- **WHEN** a plugin directory's `plugin/a.lua` raises an error and its `plugin/b.lua` exists
- **THEN** `b.lua` is not sourced and the load succeeds

#### Scenario: Error in a callback
- **WHEN** a plugin's handler for `FocusChanged` raises an error, and a second handler for `FocusChanged` belongs to another plugin
- **THEN** the second handler runs, the error is reported, and the first handler runs again on the next focus change
