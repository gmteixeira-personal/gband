## MODIFIED Requirements

### Requirement: Switch colorscheme
`gband.colorscheme(name)` SHALL load the colorscheme `name`. It SHALL first remove every group's explicit setting, keeping the default settings, and then run the file. `gband.colorscheme()`, with no argument, SHALL return the name of the active colorscheme. When loading starts, the bundled `default` colorscheme SHALL be loaded and active, before the init file runs. `gband.colorscheme` SHALL be callable while the configuration loads and in any callback.

Loading SHALL be atomic. When the file runs to completion, its name SHALL become the active colorscheme, and `gband.colorscheme` SHALL return `true`. When the colorscheme cannot be found, has an invalid name, or its file raises an error or exceeds the instruction limit the plugins capability defines, every group's explicit setting SHALL be restored to what it was before the call. The active colorscheme SHALL stay as it was, and `gband.colorscheme` SHALL return `false`. The failure SHALL be reported as a plugin error, preceded by `colors/<name>` in place of a plugin name. It SHALL NOT fail the load, and it SHALL NOT disable any plugin.

After the configuration has loaded, each call that loads a colorscheme SHALL emit `ColorschemeChanged` once, after the file has run, as the lua-events capability defines. It SHALL emit no `HighlightChanged` for the groups the colorscheme sets. A failed call SHALL emit nothing.

#### Scenario: Explicit settings replaced
- **WHEN** `user/init.lua` sets `Title` to `{ fg = 1 }`, and then calls `gband.colorscheme("dusk")`, whose file sets only `StatusLine`
- **THEN** `gband.hl.get("Title")` returns nil
- **AND** `gband.colorscheme()` returns `"dusk"`

#### Scenario: User settings after the colorscheme
- **WHEN** `user/init.lua` calls `gband.colorscheme("dusk")` and then sets `StatusLineAccent` to `{ fg = 2 }`
- **THEN** `StatusLineAccent` resolves to `{ fg = 2 }`

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
- **WHEN** `StatusLine` is set by the active colorscheme `default`, and line 3 of `user/colors/broken.lua` sets `StatusLine` to `{ fg = 1 }` and line 4 raises `boom`, and `user/init.lua` calls `gband.colorscheme("broken")` and then binds `alt+h`
- **THEN** loading succeeds and Alt+H is bound
- **AND** `StatusLine` resolves as the `default` colorscheme set it, and `gband.colorscheme()` returns `"default"`
- **AND** an error `colors/broken: <path>/user/colors/broken.lua:4: boom` is reported

#### Scenario: Missing colorscheme
- **WHEN** a binding function calls `gband.colorscheme("absent")`
- **THEN** it returns `false` and an error naming `colors/absent` is reported

#### Scenario: Switch at run time
- **WHEN** a `ColorschemeChanged` handler is registered and a binding function calls `gband.colorscheme("dusk")`
- **THEN** the handler runs once with `name` `dusk` and `previous` `default`
