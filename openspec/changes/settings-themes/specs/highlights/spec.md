## ADDED Requirements

### Requirement: Client groups
The client SHALL draw these parts of its terminal with groups, in place of fixed styles:

| group | draws | default |
|---|---|---|
| `WindowBorder` | the border of each tiled or floating window that is not focused | `{ dim = true }` |
| `WindowBorderFocused` | the border of the focused tiled or floating window | `{ bold = true }` |
| `ErrorBanner` | the error banner on the bottom row of the ribbon area | `{ fg = "red", reverse = true }` |

The client SHALL define each default in the defaults layer, before the init file runs. With the defaults, borders and the banner SHALL look as the client-attach and configuration capabilities define them. A change of a group's resolved style SHALL redraw the client. Floating plugin windows SHALL keep their own groups, as the plugin-windows capability defines.

#### Scenario: Defaults keep today's look
- **WHEN** `user/colors/blank.lua` is empty, `user/init.lua` calls only `gband.colorscheme("blank")`, and two windows are open with the second focused
- **THEN** the second tile's border is drawn bold and the first tile's border is drawn dim

#### Scenario: Colored focused border
- **WHEN** a binding function calls `gband.hl.set("WindowBorderFocused", { fg = 2, bold = true })`
- **THEN** the focused tile's border is drawn bold with foreground 2
- **AND** every other tile's border keeps the style of `WindowBorder`

#### Scenario: Banner group
- **WHEN** `user/init.lua` does not set up `gband.sidebar`, sets `ErrorBanner` to `{ fg = "#ffffff", bg = "#aa0000" }`, and a plugin raises an error
- **THEN** the banner is drawn with the foreground `#ffffff` on the background `#aa0000`
