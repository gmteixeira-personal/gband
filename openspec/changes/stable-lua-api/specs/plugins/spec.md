## MODIFIED Requirements

### Requirement: Module lookup
`require(name)` SHALL look for the module in each runtimepath entry in order, before Lua's own search path: a name `a.b` SHALL be found at `lua/a/b.lua`, then at `lua/a/b/init.lua`, in each entry. When no entry holds the module, `require` SHALL look among the modules bundled with gband, and only then in Lua's own search path. The bundled modules are these:

- `gband.prelude` and the API modules `gband.hl`, `gband.palette`, `gband.colorscheme`, `gband.bar`, `gband.win`, `gband.settings` and `gband.keystyle`, as the lua-api capability defines;
- `gband.sidebar`, as the sidebar capability defines;
- `gband.errors`, as the error-list capability defines;
- `gband.prompt`, as the lua-prompt capability defines;
- the key style presets `gband.keystyle.modal` and `gband.keystyle.direct`, as the key-style capability defines;
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
