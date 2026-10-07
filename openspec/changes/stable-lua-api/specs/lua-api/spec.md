## Purpose

Defines the boundary between gband's executable and Lua: the one API the executable provides to every Lua script, documented in full and with nothing private, the bundled Lua that consumes it as examples users can read and copy, and the rule that keeps the API stable between releases.

## ADDED Requirements

### Requirement: One provided API
The executable SHALL provide the whole Lua API. A Lua chunk that gband runs, whether a configuration file, a plugin file, a module, a colorscheme, a test file or a bundled file, SHALL reach gband only through the `gband` global of its process, the module `gband.test` in a test file, and the modules a runtimepath entry or gband's bundled modules provide, as the plugins capability defines. gband SHALL pass no chunk an argument that only a bundled chunk receives. It SHALL set no global and no `package.loaded` entry that the API documentation does not describe. No table, function or value reachable from the API SHALL let a chunk call into gband other than as the documentation describes.

#### Scenario: Bundled chunk receives nothing
- **WHEN** gband loads the bundled module `gband.sidebar`
- **THEN** the chunk's `...` holds only the values `require` passes to any module found on the runtimepath: the module name and its path

#### Scenario: No private module
- **WHEN** `user/init.lua` calls `pcall(require, "gband.win")`
- **THEN** the call returns `false`

### Requirement: Documented API
With only the API installed, before any configuration file runs, every field of the client's and the server's `gband` table, every field of each table reachable from those through named fields, every built-in event of each side, and every field of the table `require("gband.test")` returns in a test file SHALL be described in gband's API documentation: `docs/plugins.md` for the client and the server, built-in actions included, and `docs/testing.md` for the test side. The documentation SHALL give each function's arguments, return values and errors, each event's payload, and each value's meaning. A field that the documentation does not describe SHALL NOT exist.

#### Scenario: Every client field documented
- **WHEN** a client has installed its API and run no configuration file, and a test walks its `gband` table and every table reachable from it through named fields, and collects each path such as `gband.win.open`
- **THEN** `docs/plugins.md` names every collected path

#### Scenario: Every server field documented
- **WHEN** a test walks the server's `gband` table the same way
- **THEN** `docs/plugins.md` names every collected path

#### Scenario: Every test field documented
- **WHEN** a test walks the table `require("gband.test")` returns
- **THEN** `docs/testing.md` names every collected field

### Requirement: Bundled Lua consumes the API
Every Lua file bundled with gband SHALL use only the documented API: the default client and server configurations, the key style presets, the bundled plugins `gband.sidebar`, `gband.keylist`, `gband.errors` and `gband.prompt`, the settings window module `gband.settings.window`, the modules `gband.theme`, `gband.theme.catppuccin` and `gband.keyform`, and every bundled colorscheme. A copy of any of these files, placed where the runtimepath finds it first, SHALL behave as the bundled file does: a module at `lua/<path>.lua` of a runtimepath entry, and a colorscheme at `colors/<name>.lua`.

#### Scenario: Copied sidebar
- **WHEN** `user/lua/gband/sidebar.lua` holds the bundled sidebar's text, the default configuration is in use, and an error is reported
- **THEN** the sidebar shows the mode letter, the band labels and `!`, exactly as the bundled sidebar does
- **AND** no banner is drawn

#### Scenario: Copied prompt
- **WHEN** `user/lua/gband/prompt.lua` holds the bundled prompt's text and the user runs `error("boom")` from the Lua prompt
- **THEN** the client reports the error `prompt:1: boom`

#### Scenario: Copied settings window
- **WHEN** `user/lua/gband/settings/window.lua` holds the bundled settings window's text and the user presses Enter on its `keys` line
- **THEN** `user/keystyle.lua` holds the other style, and after the reload the settings window is open on its third line

#### Scenario: Copied theme
- **WHEN** `user/colors/nord.lua` holds the bundled `nord` theme's text and `user/init.lua` calls `gband.colorscheme("nord")`
- **THEN** every group and the palette resolve as they do with the bundled `nord`

#### Scenario: Every bundled file copied
- **WHEN** gband's Lua spec suite runs with every bundled module copied into `user/lua/` and every bundled colorscheme into `user/colors/` of each case's configuration directory
- **THEN** every case passes, with the same screens as without the copies

### Requirement: Bundled sources on disk
Each process SHALL write the text of every bundled module, other than the key style presets, to `defaults/lua/gband/` of the configuration directory, at the module's path: a module `gband.a.b` at `defaults/lua/gband/a/b.lua`. It SHALL write every bundled colorscheme to `defaults/colors/<name>.lua`. It SHALL write them as the configuration capability's "Configuration directory" writes the files under `defaults`, and they SHALL be copies for the user to read: loading SHALL use the built-in text, never these files. `defaults` SHALL NOT be a runtimepath entry.

#### Scenario: Sources written
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `/tmp/cfg/gband` does not exist, and a client attaches
- **THEN** `/tmp/cfg/gband/defaults/lua/gband/sidebar.lua` holds the bundled sidebar's text
- **AND** `/tmp/cfg/gband/defaults/lua/gband/settings/window.lua` holds the bundled settings window's text
- **AND** `/tmp/cfg/gband/defaults/colors/nord.lua` holds the bundled `nord` theme's text

#### Scenario: Copies have no effect
- **WHEN** the user edits `defaults/lua/gband/sidebar.lua` and reloads
- **THEN** the sidebar behaves as the built-in text defines

#### Scenario: Copy to override
- **WHEN** the user copies `defaults/lua/gband/sidebar.lua` to `user/lua/gband/sidebar.lua` and changes its mode letter for `root` from `I` to `i`
- **THEN** after the reload the sidebar's first row shows `i` in interactive mode

### Requirement: API version rule
`gband.api_version` SHALL identify the documented API. A build that removes a documented function, table, field, event or payload field, or that changes the documented arguments, return values, errors or effect of one, SHALL raise `gband.api_version` by one. A build that only adds to the documented API, or that changes only what the documentation does not describe, SHALL keep it. The API documentation SHALL state this rule, and SHALL list for each value of `gband.api_version` above 1 what changed.

#### Scenario: Version after this change
- **WHEN** `user/init.lua` reads `gband.api_version`
- **THEN** it reads `1`

#### Scenario: Rule documented
- **WHEN** a reader opens the "API and stability" section of `docs/plugins.md`
- **THEN** it states when `gband.api_version` changes and lists no change for version 1
