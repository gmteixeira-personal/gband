## MODIFIED Requirements

### Requirement: Bundled Lua consumes the API
Every Lua file bundled with gband SHALL use only the documented API, and none SHALL receive anything private. This covers:
- the default client and server configurations;
- the prelude and the API modules;
- the key style presets;
- the bundled plugins `gband.sidebar`, `gband.keylist`, `gband.errors`, `gband.prompt` and `gband.desktop`;
- the modules `gband.theme`, `gband.theme.catppuccin` and `gband.keyform`;
- every bundled colorscheme;
- on the test side, `gband.test`.

A copy of any of these files, placed where the runtimepath finds it first, SHALL behave as the bundled file does: a module at `lua/<path>.lua` of a runtimepath entry, and a colorscheme at `colors/<name>.lua`. The one difference is the settings capability's theme list, which names every colorscheme on the runtimepath, so a copy of the alias `catppuccin` is listed where the bundled alias is not.

#### Scenario: Copied sidebar
- **WHEN** `user/lua/gband/sidebar.lua` holds the bundled sidebar's text, the default configuration is in use, and an error is reported
- **THEN** the sidebar shows the mode letter, the band labels and `!`, exactly as the bundled sidebar does
- **AND** no banner is drawn

#### Scenario: Copied prompt
- **WHEN** `user/lua/gband/prompt.lua` holds the bundled prompt's text and the user runs `error("boom")` from the Lua prompt
- **THEN** the client reports the error `prompt:1: boom`

#### Scenario: Copied theme
- **WHEN** `user/colors/nord.lua` holds the bundled `nord` theme's text and `user/init.lua` calls `gband.colorscheme("nord")`
- **THEN** every group and the palette resolve as they do with the bundled `nord`

#### Scenario: Every bundled file copied
- **WHEN** gband's Lua spec suite runs with every bundled module copied into `user/lua/` and every bundled colorscheme other than the alias `catppuccin` into `user/colors/` of each case's configuration directory
- **THEN** every case passes, with the same screens as without the copies

#### Scenario: Copied desktop plugin
- **WHEN** `user/lua/gband/desktop.lua` holds the bundled desktop plugin's text, `user/keystyle.lua` holds `return "floating"`, no `user/init.lua` exists, and a floating window is drawn
- **THEN** its top border shows `[_][□][X]` and Ctrl+Space opens the window list, exactly as with the bundled plugin
