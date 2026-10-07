## MODIFIED Requirements

### Requirement: Window border options
Tiled windows SHALL be drawn with the border definition of the client options `tile_border_sides` and `tile_border_chars`. Floating windows SHALL be drawn with the definition of `floating_border_sides` and `floating_border_chars`. The focused tiled window SHALL be drawn with the character set of `focused_tile_border_chars` in place of `tile_border_chars`, and the focused floating window with that of `focused_floating_border_chars` in place of `floating_border_chars`, each keeping its sides. A lifted tile's drop outline, as the mouse capability defines it, SHALL be drawn with all four sides and the character set of `focused_tile_border_chars`. Each SHALL apply only to the client that set it. A border's style SHALL stay as the client-attach capability defines it, distinct for the focused window. A window's terminal size SHALL NOT depend on these options.

#### Scenario: Tiles without side borders
- **WHEN** `user/init.lua` sets `tile_border_sides` to `{ "top", "bottom" }`, the client's 80×24 terminal sets the screen area, the sidebar is not set up and the client has no bar, and one window sits in a column of width 1/2
- **THEN** the tile's top and bottom rows show `─` from column 0 to column 39, columns 0 and 39 are blank between them, and `tput cols` in the window prints `38`

#### Scenario: Floating windows differ from tiles
- **WHEN** `user/init.lua` sets `tile_border_chars` to `"plain"` and `floating_border_chars` to `"double"`, and a floating window is shown over a tile
- **THEN** the tile's corners are drawn with `┌ ┐ └ ┘` and the floating window's with `╔ ╗ ╚ ╝`

#### Scenario: Rounded by default
- **WHEN** `user/init.lua` sets no border option, and two windows are open in two columns
- **THEN** both tiles' corners are drawn with `╭ ╮ ╰ ╯`

#### Scenario: Focused tile with its own characters
- **WHEN** `user/init.lua` sets `focused_tile_border_chars` to `"thick"`, and two windows are open in two columns with the second focused
- **THEN** the second tile's corners are drawn with `┏ ┓ ┗ ┛` and its sides with `━` and `┃`
- **AND** the first tile's corners are drawn with `╭ ╮ ╰ ╯`
- **AND** after the first window is focused, the first tile is drawn with `thick` and the second with `rounded`

#### Scenario: Focused floating window with its own characters
- **WHEN** `user/init.lua` sets `focused_floating_border_chars` to `"double"`, and two floating windows are shown with the later one focused
- **THEN** the focused floating window's corners are drawn with `╔ ╗ ╚ ╝` and the other's with `╭ ╮ ╰ ╯`

#### Scenario: Focused characters keep the sides
- **WHEN** `user/init.lua` sets `tile_border_sides` to `{ "left" }` and `focused_tile_border_chars` to `"thick"`, and one window is focused
- **THEN** the tile's column 0 shows `┃` on every row and its other edge cells are blank

#### Scenario: Drop outline uses the focused characters
- **WHEN** `user/init.lua` sets `focused_tile_border_chars` to `"double"`, and the user lifts a tile with `drag_window`
- **THEN** the drop place's outline is drawn with `╔ ╗ ╚ ╝`, in the style of `WindowBorderFocused`

#### Scenario: Two clients with different borders
- **WHEN** two clients with 80×24 terminals view the same band, the first sets `tile_border_sides` to `{}` and the second keeps the default
- **THEN** both clients show each window's whole grid
- **AND** only the second client draws border characters around it
