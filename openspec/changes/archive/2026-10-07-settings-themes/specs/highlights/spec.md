## ADDED Requirements

### Requirement: Client groups
The client SHALL draw these parts of its terminal with groups, in place of fixed styles:

| group | draws | default |
|---|---|---|
| `WindowBorder` | the border of each tiled or floating window that is not focused | `{ dim = true }` |
| `WindowBorderFocused` | the border of the focused tiled or floating window, and the lifted tile's drop outline | `{ fg = "#b1b9f9", bold = true }` |
| `ErrorBanner` | the error banner on the bottom row of the ribbon area | `{ fg = "red", reverse = true }` |

The client SHALL define each default in the defaults layer, before the init file runs. With the defaults, the focused window's border SHALL stay distinct from the other borders, as the client-attach capability defines, and the banner SHALL look as the configuration capability defines it. A change of a group's resolved style SHALL redraw the client. Floating plugin windows SHALL keep their own groups, as the plugin-windows capability defines.

#### Scenario: Built-in border look
- **WHEN** the active colorscheme is `default`, the client supports 24-bit color, and two windows are open with the second focused
- **THEN** the second tile's border is drawn bold with the foreground `#b1b9f9`, and the first tile's border is drawn dim

#### Scenario: Colored focused border
- **WHEN** a binding function calls `gband.hl.set("WindowBorderFocused", { fg = 2, bold = true })`
- **THEN** the focused tile's border is drawn bold with foreground 2
- **AND** every other tile's border keeps the style of `WindowBorder`

#### Scenario: Banner group
- **WHEN** `user/init.lua` does not set up `gband.sidebar`, sets `ErrorBanner` to `{ fg = "#ffffff", bg = "#aa0000" }`, and a plugin raises an error
- **THEN** the banner is drawn with the foreground `#ffffff` on the background `#aa0000`
