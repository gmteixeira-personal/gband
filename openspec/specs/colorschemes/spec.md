# colorschemes Specification

## Purpose

Defines colorschemes: Lua files that set highlight groups as a unit, how they are found on the runtimepath, how one is switched in, and how a failing one is contained.

## Requirements

### Requirement: Colorscheme files
A colorscheme named `name` SHALL be the file `colors/<name>.lua` in the first runtimepath entry that holds one. When no entry holds one, it SHALL be the colorscheme of that name bundled with gband, if any. A colorscheme name SHALL begin with an ASCII letter or digit and hold only ASCII letters, digits, `_` and `-`. gband SHALL bundle the themes "Bundled themes" lists.

A colorscheme file SHALL be Lua that runs with the whole `gband` API available. It sets groups with `gband.hl.set` and the terminal palette with `gband.palette.set`. Code it runs SHALL belong to no plugin.

#### Scenario: User colorscheme
- **WHEN** `user/colors/dusk.lua` exists and `user/init.lua` calls `gband.colorscheme("dusk")`
- **THEN** `user/colors/dusk.lua` runs

#### Scenario: Earlier entry wins
- **WHEN** both `user/colors/dusk.lua` and a plugin directory's `colors/dusk.lua` exist
- **THEN** `gband.colorscheme("dusk")` runs `user/colors/dusk.lua`

#### Scenario: Bundled fallback
- **WHEN** no runtimepath entry holds `colors/nord.lua`
- **THEN** `gband.colorscheme("nord")` runs the bundled `nord` theme

#### Scenario: Shadowing the bundled colorscheme
- **WHEN** `user/colors/gruvbox.lua` exists
- **THEN** `gband.colorscheme("gruvbox")` runs `user/colors/gruvbox.lua`

#### Scenario: Default sets the sidebar groups
- **WHEN** no theme is saved, no file calls `gband.colorscheme`, and a binding function calls `gband.hl.get` for `SidebarMode`, `SidebarBand`, `SidebarBandActive` and `SidebarError`
- **THEN** each call returns the default setting the sidebar plugin gives the group, since the `default` colorscheme sets none of them

#### Scenario: Bars on the terminal's background
- **WHEN** no theme is saved, and no file calls `gband.colorscheme` or sets `Bar`
- **THEN** `gband.hl.get("Bar")` holds no field and `Bar` resolves to no field

#### Scenario: Default sets nothing
- **WHEN** no theme is saved, the default configuration is in use, and a binding function calls `gband.hl.get` for `SidebarMode` and `WindowBorderFocused`
- **THEN** the calls return `{ bold = true }` and `{ fg = "#b1b9f9", bold = true }`, the default settings of the sidebar plugin and the client
- **AND** `gband.palette.get()` returns an empty table

#### Scenario: Back to the defaults
- **WHEN** the active colorscheme is `gruvbox`, no runtimepath entry holds `colors/default.lua`, and a binding function calls `gband.colorscheme("default")`
- **THEN** it returns `true` and `gband.colorscheme()` returns `default`
- **AND** no group has an explicit setting, and `gband.palette.get()` returns an empty table

### Requirement: Switch colorscheme
`gband.colorscheme(name)` SHALL load the colorscheme `name`. It SHALL first remove every group's explicit setting, keeping the default settings, and empty the terminal palette, and then run the file. `gband.colorscheme()`, with no argument, SHALL return the name of the active colorscheme. `gband.colorscheme` SHALL be callable while the configuration loads and in any callback.

When loading starts, before the init file runs, the client SHALL load the start theme, as `gband.colorscheme` loads a colorscheme. The start theme SHALL be the theme `gband.settings.theme()` returns, as the settings capability defines. When it returns nil, or loading the saved theme fails, the start theme SHALL be `default`, loaded in the same way. A failure of the saved theme SHALL be reported as any failed colorscheme is. The init file MAY load another colorscheme, which then replaces the start theme.

Loading SHALL be atomic. When the file runs to completion, its name SHALL become the active colorscheme, and `gband.colorscheme` SHALL return `true`. When the colorscheme cannot be found, has an invalid name, or its file raises an error or exceeds the instruction limit the plugins capability defines, every group's explicit setting and the terminal palette SHALL be restored to what they were before the call. The active colorscheme SHALL stay as it was, and `gband.colorscheme` SHALL return `false`. The failure SHALL be reported as a plugin error, preceded by `colors/<name>` in place of a plugin name. It SHALL NOT fail the load, and it SHALL NOT disable any plugin.

After the configuration has loaded, each call that loads a colorscheme SHALL emit `ColorschemeChanged` once, after the file has run, as the lua-events capability defines. It SHALL emit no `HighlightChanged` for the groups the colorscheme sets. A failed call SHALL emit nothing.

#### Scenario: Explicit settings replaced
- **WHEN** `user/init.lua` sets `Title` to `{ fg = 1 }`, and then calls `gband.colorscheme("dusk")`, whose file sets only `SidebarMode`
- **THEN** `gband.hl.get("Title")` returns nil
- **AND** `gband.colorscheme()` returns `"dusk"`

#### Scenario: Palette replaced
- **WHEN** `user/theme.lua` holds `return "gruvbox"` and `user/init.lua` calls `gband.colorscheme("dusk")`, whose file sets no palette
- **THEN** `gband.palette.get()` returns an empty table

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
- **WHEN** no theme is saved and no file calls `gband.colorscheme`
- **THEN** `gband.colorscheme()` returns `"default"`

#### Scenario: Saved theme at start
- **WHEN** `user/theme.lua` holds `return "nord"` and `user/init.lua` binds only `alt+h`
- **THEN** `gband.colorscheme()` returns `"nord"`

#### Scenario: Init file replaces the start theme
- **WHEN** `user/theme.lua` holds `return "nord"` and `user/init.lua` calls `gband.colorscheme("dracula")`
- **THEN** `gband.colorscheme()` returns `"dracula"`

#### Scenario: Saved theme missing
- **WHEN** `user/theme.lua` holds `return "absent"` and a client attaches
- **THEN** `gband.colorscheme()` returns `"default"`
- **AND** an error naming `colors/absent` is reported

#### Scenario: Failing colorscheme
- **WHEN** `user/theme.lua` holds `return "gruvbox"`, so `SidebarMode` is set by the active colorscheme `gruvbox`, and line 3 of `user/colors/broken.lua` sets `SidebarMode` to `{ fg = 1 }`, line 4 calls `gband.palette.set({})` and line 5 raises `boom`, and `user/init.lua` calls `gband.colorscheme("broken")` and then binds `alt+h`
- **THEN** loading succeeds and Alt+H is bound
- **AND** `SidebarMode` resolves as `gruvbox` set it, `gband.palette.get()` returns the palette `gruvbox` set, and `gband.colorscheme()` returns `"gruvbox"`
- **AND** an error `colors/broken: <path>/user/colors/broken.lua:5: boom` is reported

#### Scenario: Missing colorscheme
- **WHEN** a binding function calls `gband.colorscheme("absent")`
- **THEN** it returns `false` and an error naming `colors/absent` is reported

#### Scenario: Switch at run time
- **WHEN** a `ColorschemeChanged` handler is registered and a binding function calls `gband.colorscheme("dusk")`
- **THEN** the handler runs once with `name` `dusk` and `previous` `default`

### Requirement: Bundled themes
gband SHALL bundle these colorschemes, called themes, in this order: `default`, `terminal`, `catppuccin-latte`, `catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`, `tokyo-night`, `dracula`, `nord`, `gruvbox`, `one-dark`, `solarized`, `kanagawa`, `rose-pine` and `vesper`. It SHALL also bundle `catppuccin`, which sets exactly what `catppuccin-mocha` sets.

`default` SHALL set no group and no field of the terminal palette. Loading it SHALL leave every group with only its default setting, as the highlights capability defines, and the terminal palette empty.

Each theme other than `default` SHALL give an explicit setting to every one of these theme groups: `Bar`, `SidebarMode`, `SidebarBand`, `SidebarBandActive`, `SidebarError`, `KeyListKey`, `KeyListMuted`, `PluginWindow`, `PluginWindowBorder`, `PluginWindowTitle`, `PluginWindowCursorLine`, `PromptCursor`, `SettingsLabel`, `WindowBorder`, `WindowBorderFocused` and `ErrorBanner`. The viewed band's label, a focused window's border and the cursor line SHALL each be told apart from their unfocused or unselected peers by color or by attribute.

Every theme except `default` and `terminal` SHALL set every field of the terminal palette, from the default foreground, default background and 16 ANSI colors that the theme's authors publish for terminals. `solarized` SHALL use Solarized's dark background, `rose-pine` the main variant, `kanagawa` the wave variant, and `tokyo-night` the night variant. Its groups MAY use hex colors.

`terminal` SHALL set no field of the terminal palette. Its groups SHALL use only the 16 named colors, the palette indexes 0 to 15, and the attributes, so the host terminal's own palette decides every color it draws.

#### Scenario: Gruvbox palette
- **WHEN** the active colorscheme is `gruvbox` and a callback calls `gband.palette.get()`
- **THEN** it returns `bg` `#282828`, `fg` `#ebdbb2`, `red` `#cc241d` and `bright_red` `#fb4934`

#### Scenario: Every theme sets every theme group
- **WHEN** each bundled theme other than `default` in turn is loaded with an empty `user/init.lua`
- **THEN** `gband.hl.get` returns a table for every theme group

#### Scenario: Terminal theme uses the host palette
- **WHEN** the active colorscheme is `terminal`
- **THEN** `gband.palette.get()` returns an empty table
- **AND** no theme group's resolved style holds a hex color or an index above 15

#### Scenario: Catppuccin alias
- **WHEN** a binding function calls `gband.colorscheme("catppuccin")`
- **THEN** `gband.colorscheme()` returns `catppuccin`
- **AND** every theme group and the palette equal what `catppuccin-mocha` sets

### Requirement: Terminal palette
The client SHALL keep one terminal palette: a set of up to 18 colors, each optional, with the fields `fg`, `bg` and the 16 color names of the highlights capability, from `black` to `bright_white`. Each color SHALL be a `#` followed by six hexadecimal digits of either case. The palette SHALL be empty when loading starts, before the start theme loads.

`gband.palette.set(spec)` SHALL replace the whole palette with `spec`, a table of those fields. `gband.palette.set({})` SHALL empty it. A `spec` that is not a table, a field not listed, or a value that is not such a hex color SHALL be an error at the line of the call naming the field, and SHALL leave the palette unchanged. `gband.palette.get()` SHALL return a new table of the palette's fields, with hex colors in lowercase. Changing a returned table SHALL NOT change the palette. Both functions SHALL be callable while the configuration loads and in any callback. After the configuration has loaded, a change of the palette SHALL redraw the client.

When the client draws its terminal, after every other step of drawing, it SHALL map each cell's foreground and background:
- the terminal's default foreground SHALL become `fg`;
- the terminal's default background SHALL become `bg`;
- palette index `n`, from 0 to 15, SHALL become the field of the `n`-th color name, `black` being index 0.

Each mapping SHALL apply only when the palette sets that field. It SHALL cover every cell of the terminal: windows' program output, borders, the empty ribbon, bars, plugin windows and the banner. A mapped color SHALL be drawn as a hex color, as the highlights capability defines, so a client without 24-bit color draws the nearest palette index from 16 to 255. Palette indexes 16 to 255 and hex colors SHALL be drawn unchanged. The palette SHALL change nothing the server stores or sends, and nothing a program in a window reads.

#### Scenario: Program color mapped
- **WHEN** the active colorscheme is `gruvbox`, the client supports 24-bit color, and a window runs `printf '\033[31mx\033[0m'`
- **THEN** the client draws `x` with the foreground `#cc241d` and the background `#282828`

#### Scenario: Empty ribbon takes the background
- **WHEN** the active colorscheme is `gruvbox` and part of the ribbon area holds no window
- **THEN** the client draws those cells with the background `#282828`

#### Scenario: Mapped color without 24-bit color
- **WHEN** the active colorscheme is `gruvbox`, the client does not support 24-bit color, and a window prints text in ANSI red
- **THEN** the client draws the text with palette index 160

#### Scenario: 256-color output unchanged
- **WHEN** the active colorscheme is `gruvbox` and a window runs `printf '\033[38;5;208mx'`
- **THEN** the client draws `x` with palette index 208

#### Scenario: Terminal theme leaves colors alone
- **WHEN** the active colorscheme is `terminal` and a window prints text in ANSI red on the default background
- **THEN** the client draws the text with palette index 1 and the terminal's default background

#### Scenario: Invalid palette field
- **WHEN** line 3 of `user/colors/dusk.lua` calls `gband.palette.set({ purple = "#800080" })`
- **THEN** an error at that line names `purple` and the palette is unchanged

#### Scenario: Invalid palette color
- **WHEN** line 2 of `user/init.lua` calls `gband.palette.set({ bg = 236 })`
- **THEN** loading fails with an error at `user/init.lua` line 2 naming `bg`

#### Scenario: Palette set by the init file
- **WHEN** `user/init.lua` calls `gband.palette.set({ bg = "#101010" })` and a window prints text with the default colors
- **THEN** the client draws the text on the background `#101010`, with the terminal's default foreground
