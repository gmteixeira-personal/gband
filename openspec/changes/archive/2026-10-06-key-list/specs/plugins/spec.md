## MODIFIED Requirements

### Requirement: Module lookup
`require(name)` SHALL look for the module in each runtimepath entry in order, before Lua's own search path: a name `a.b` SHALL be found at `lua/a/b.lua`, then at `lua/a/b/init.lua`, in each entry. When no entry holds the module, `require` SHALL look among the modules bundled with gband, and only then in Lua's own search path. The bundled modules SHALL be the segment modules the status-line capability lists, `gband.keylist`, as the key-list capability defines, and `gband.keyform`, which returns the function that turns a key name into the form the key-hints capability's "Key form" defines. The first file found SHALL be loaded, and errors in it SHALL name its path and line. Errors in a bundled module SHALL name its path under `gband/`, such as `gband/statusline/band.lua`.

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

#### Scenario: Bundled key form
- **WHEN** `user/init.lua` calls `require("gband.keyform")("ctrl+space")`
- **THEN** it returns `C-space`

#### Scenario: Bundled module shadowed
- **WHEN** `user/lua/gband/statusline/band.lua` exists
- **THEN** `require("gband.statusline.band")` loads `user/lua/gband/statusline/band.lua`
