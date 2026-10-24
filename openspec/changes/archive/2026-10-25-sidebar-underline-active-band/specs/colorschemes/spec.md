## MODIFIED Requirements

### Requirement: Bundled themes
gband SHALL bundle these colorschemes, called themes, in this order: `default`, `terminal`, `catppuccin-latte`, `catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`, `tokyo-night`, `dracula`, `nord`, `gruvbox`, `one-dark`, `solarized`, `kanagawa`, `rose-pine` and `vesper`. It SHALL also bundle `catppuccin`, which sets exactly what `catppuccin-mocha` sets.

`default` SHALL set no group and no field of the terminal palette. Loading it SHALL leave every group with only its default setting, as the highlights capability defines, and the terminal palette empty.

Each theme other than `default` SHALL give an explicit setting to every one of these theme groups: `Bar`, `SidebarMode`, `SidebarBand`, `SidebarBandActive`, `SidebarError`, `KeyListKey`, `KeyListMuted`, `PluginWindow`, `PluginWindowBorder`, `PluginWindowTitle`, `PluginWindowCursorLine`, `PromptCursor`, `SettingsLabel`, `WindowBorder`, `WindowBorderFocused` and `ErrorBanner`. The viewed band's label, a focused window's border and the cursor line SHALL each be told apart from their unfocused or unselected peers by color or by attribute. Each such theme SHALL set `SidebarBandActive` to `{ link = "SidebarBand", underline = true }`, so the viewed band's label differs from the other band labels only by its underline.

Every theme except `default` and `terminal` SHALL set every field of the terminal palette, from the default foreground, default background and 16 ANSI colors that the theme's authors publish for terminals. `solarized` SHALL use Solarized's dark background, `rose-pine` the main variant, `kanagawa` the wave variant, and `tokyo-night` the night variant. Its groups MAY use hex colors.

`terminal` SHALL set no field of the terminal palette. Its groups SHALL use only the 16 named colors, the palette indexes 0 to 15, and the attributes, so the host terminal's own palette decides every color it draws.

#### Scenario: Gruvbox palette
- **WHEN** the active colorscheme is `gruvbox` and a callback calls `gband.palette.get()`
- **THEN** it returns `bg` `#282828`, `fg` `#ebdbb2`, `red` `#cc241d` and `bright_red` `#fb4934`

#### Scenario: Every theme sets every theme group
- **WHEN** each bundled theme other than `default` in turn is loaded with an empty `user/init.lua`
- **THEN** `gband.hl.get` returns a table for every theme group

#### Scenario: Viewed band underlined under every theme
- **WHEN** each bundled theme other than `default` in turn is loaded with an empty `user/init.lua`
- **THEN** `gband.hl.get("SidebarBandActive")` returns `{ link = "SidebarBand", underline = true }`
- **AND** `SidebarBandActive` resolves to the resolved style of `SidebarBand` with `underline = true` added

#### Scenario: Tokyo Night viewed band
- **WHEN** the active colorscheme is `tokyo-night` and no group is set
- **THEN** `SidebarBandActive` resolves to `{ fg = "#565f89", underline = true }`

#### Scenario: Terminal theme uses the host palette
- **WHEN** the active colorscheme is `terminal`
- **THEN** `gband.palette.get()` returns an empty table
- **AND** no theme group's resolved style holds a hex color or an index above 15

#### Scenario: Catppuccin alias
- **WHEN** a binding function calls `gband.colorscheme("catppuccin")`
- **THEN** `gband.colorscheme()` returns `catppuccin`
- **AND** every theme group and the palette equal what `catppuccin-mocha` sets
