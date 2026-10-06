## MODIFIED Requirements

### Requirement: Colorscheme files
A colorscheme named `name` SHALL be the file `colors/<name>.lua` in the first runtimepath entry that holds one. When no entry holds one, it SHALL be the colorscheme of that name bundled with gband, if any. A colorscheme name SHALL begin with an ASCII letter or digit and hold only ASCII letters, digits, `_` and `-`. gband SHALL bundle one colorscheme, `default`, which sets `Bar`, every group the sidebar capability defines, and the key list groups `KeyListKey` and `KeyListMuted`.

A colorscheme file SHALL be Lua that runs with the whole `gband` API available. It sets groups with `gband.hl.set`. Code it runs SHALL belong to no plugin.

#### Scenario: User colorscheme
- **WHEN** `user/colors/dusk.lua` exists and `user/init.lua` calls `gband.colorscheme("dusk")`
- **THEN** `user/colors/dusk.lua` runs

#### Scenario: Earlier entry wins
- **WHEN** both `user/colors/dusk.lua` and a plugin directory's `colors/dusk.lua` exist
- **THEN** `gband.colorscheme("dusk")` runs `user/colors/dusk.lua`

#### Scenario: Bundled fallback
- **WHEN** no runtimepath entry holds `colors/default.lua`
- **THEN** `gband.colorscheme("default")` runs the bundled `default` colorscheme

#### Scenario: Shadowing the bundled colorscheme
- **WHEN** `user/colors/default.lua` exists
- **THEN** `gband.colorscheme("default")` runs `user/colors/default.lua`

#### Scenario: Default sets the sidebar groups
- **WHEN** no file calls `gband.colorscheme` and a binding function calls `gband.hl.get` for `SidebarMode`, `SidebarBand`, `SidebarBandActive` and `SidebarError`
- **THEN** each call returns the setting the `default` colorscheme gives the group

### Requirement: Switch colorscheme
`gband.colorscheme(name)` SHALL load the colorscheme `name`. It SHALL first remove every group's explicit setting, keeping the default settings, and then run the file. `gband.colorscheme()`, with no argument, SHALL return the name of the active colorscheme. When loading starts, the bundled `default` colorscheme SHALL be loaded and active, before the init file runs. `gband.colorscheme` SHALL be callable while the configuration loads and in any callback.

Loading SHALL be atomic. When the file runs to completion, its name SHALL become the active colorscheme, and `gband.colorscheme` SHALL return `true`. When the colorscheme cannot be found, has an invalid name, or its file raises an error or exceeds the instruction limit the plugins capability defines, every group's explicit setting SHALL be restored to what it was before the call. The active colorscheme SHALL stay as it was, and `gband.colorscheme` SHALL return `false`. The failure SHALL be reported as a plugin error, preceded by `colors/<name>` in place of a plugin name. It SHALL NOT fail the load, and it SHALL NOT disable any plugin.

After the configuration has loaded, each call that loads a colorscheme SHALL emit `ColorschemeChanged` once, after the file has run, as the lua-events capability defines. It SHALL emit no `HighlightChanged` for the groups the colorscheme sets. A failed call SHALL emit nothing.

#### Scenario: Explicit settings replaced
- **WHEN** `user/init.lua` sets `Title` to `{ fg = 1 }`, and then calls `gband.colorscheme("dusk")`, whose file sets only `SidebarMode`
- **THEN** `gband.hl.get("Title")` returns nil
- **AND** `gband.colorscheme()` returns `"dusk"`

#### Scenario: User settings after the colorscheme
- **WHEN** `user/init.lua` calls `gband.colorscheme("dusk")` and then sets `SidebarBandActive` to `{ fg = 2 }`
- **THEN** `SidebarBandActive` resolves to `{ fg = 2 }`

#### Scenario: Plugin default kept
- **WHEN** the plugin `window` declares the default `WindowSegment` `{ fg = 4 }` and a callback calls `gband.colorscheme("dusk")`, whose file does not mention `WindowSegment`
- **THEN** `WindowSegment` resolves to `{ fg = 4 }`

#### Scenario: Colorscheme overrides a plugin default
- **WHEN** the plugin `window` declares the default `WindowSegment` `{ fg = 4 }` after `user/init.lua` called `gband.colorscheme("dusk")`, whose file sets `WindowSegment` to `{ fg = "#00ff00" }`
- **THEN** `WindowSegment` resolves to `{ fg = "#00ff00" }`

#### Scenario: Default at start
- **WHEN** no file calls `gband.colorscheme`
- **THEN** `gband.colorscheme()` returns `"default"`

#### Scenario: Failing colorscheme
- **WHEN** `SidebarMode` is set by the active colorscheme `default`, and line 3 of `user/colors/broken.lua` sets `SidebarMode` to `{ fg = 1 }` and line 4 raises `boom`, and `user/init.lua` calls `gband.colorscheme("broken")` and then binds `alt+h`
- **THEN** loading succeeds and Alt+H is bound
- **AND** `SidebarMode` resolves as the `default` colorscheme set it, and `gband.colorscheme()` returns `"default"`
- **AND** an error `colors/broken: <path>/user/colors/broken.lua:4: boom` is reported

#### Scenario: Missing colorscheme
- **WHEN** a binding function calls `gband.colorscheme("absent")`
- **THEN** it returns `false` and an error naming `colors/absent` is reported

#### Scenario: Switch at run time
- **WHEN** a `ColorschemeChanged` handler is registered and a binding function calls `gband.colorscheme("dusk")`
- **THEN** the handler runs once with `name` `dusk` and `previous` `default`
