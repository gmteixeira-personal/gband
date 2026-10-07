## MODIFIED Requirements

### Requirement: Module lookup
`require(name)` SHALL look for the module in each runtimepath entry in order, before Lua's own search path: a name `a.b` SHALL be found at `lua/a/b.lua`, then at `lua/a/b/init.lua`, in each entry. When no entry holds the module, `require` SHALL look among the modules bundled with gband, and only then in Lua's own search path. The bundled modules SHALL be `gband.sidebar`, as the sidebar capability defines, `gband.errors`, as the error-list capability defines, `gband.prompt`, as the lua-prompt capability defines, the key style presets `gband.keystyle.modal` and `gband.keystyle.direct`, as the key-style capability defines, `gband.keylist`, as the key-list capability defines, `gband.keyform`, which returns the function that turns a key name into the form the key-list capability's "Key form" defines, `gband.theme` and `gband.theme.catppuccin`, as the colorschemes capability's "Themes" defines, and `gband.settings.window`, as the settings capability defines. The first file found SHALL be loaded, and errors in it SHALL name its path and line. Errors in a bundled module SHALL name its path under `gband/`, such as `gband/sidebar.lua`. A bundled module's chunk SHALL receive the same arguments as a module found on the runtimepath: the module name and its path, under `gband/` for a bundled module. `require` SHALL return the module's value and that path. No other module SHALL be bundled, and the parts of the API that the client installs SHALL NOT be `require`-able modules.

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

#### Scenario: Bundled chunk arguments
- **WHEN** `user/lua/probe.lua` returns `{ ... }`, and `user/init.lua` compares `require("probe")` with what the bundled `gband.keyform` chunk receives
- **THEN** both hold the module name and the module's path, and nothing else

#### Scenario: API tables are not modules
- **WHEN** `user/init.lua` calls `pcall(require, "gband.hl")`, `pcall(require, "gband.win")` and `pcall(require, "gband.bar")`
- **THEN** each call returns `false`

## ADDED Requirements

### Requirement: Evaluate source
`gband.eval(source, name)` SHALL compile `source`, a string, as Lua source text, never as a binary chunk, with `name` as its chunk name, and run it. It SHALL exist on the client and the server. The code SHALL belong to no plugin, as code of the configuration file does, whoever calls `gband.eval`. It SHALL be able to call everything the calling code can call. It SHALL run with its own instruction budget, of the size "Instruction limit" defines, so a run that exceeds it stops only the evaluated code, and the calling code continues with its own budget unchanged. A stop SHALL NOT mark the calling code's plugin failed.

A syntax error, an error raised while the code runs, and a stop by the instruction limit SHALL each be reported as a configuration error, as the configuration capability defines, with `name` in place of the file's path and the line within `source`. `gband.eval` SHALL then return `false` and the error's message. When the code returns, `gband.eval` SHALL return `true` followed by the code's return values. Dispatches the code made before an error SHALL stand. A `source` or `name` that is not a string SHALL raise an error at the line of the call.

#### Scenario: Return values
- **WHEN** a binding function calls `gband.eval("return 1 + 1", "calc")`
- **THEN** the call returns `true` and `2`, and no error is reported

#### Scenario: Runtime error reported at the name
- **WHEN** a binding function calls `gband.eval("\nerror('boom')", "probe")`
- **THEN** the client reports the error `probe:2: boom`
- **AND** the call returns `false` and the message

#### Scenario: Syntax error
- **WHEN** a binding function calls `gband.eval("gband.(", "probe")`
- **THEN** the client reports an error beginning `probe:1:` and the call returns `false`

#### Scenario: Endless loop stops only the code
- **WHEN** the action of plugin `p` calls `gband.eval("while true do end", "probe")` and then dispatches `focus_column_left`
- **THEN** the client reports the error `probe:1: instruction limit exceeded`
- **AND** the column to the left is focused, and `p` is not marked failed

#### Scenario: Evaluated code belongs to no plugin
- **WHEN** the action of plugin `p` calls `gband.eval("gband.action.register('greet', function() end)", "probe")`
- **THEN** `gband.action.greet` holds the action and `gband.action["p.greet"]` is nil

#### Scenario: Wrong argument
- **WHEN** line 3 of `user/init.lua` calls `gband.eval(42, "probe")`
- **THEN** loading fails with an error at `user/init.lua` line 3
