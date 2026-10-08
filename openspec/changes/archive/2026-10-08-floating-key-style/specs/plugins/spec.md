## MODIFIED Requirements

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
