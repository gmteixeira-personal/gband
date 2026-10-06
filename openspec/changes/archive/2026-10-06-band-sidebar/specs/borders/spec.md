## MODIFIED Requirements

### Requirement: Window border options
Tiled windows SHALL be drawn with the border definition of the client options `tile_border_sides` and `tile_border_chars`. Floating windows SHALL be drawn with the definition of `floating_border_sides` and `floating_border_chars`. Each SHALL apply only to the client that set it. A border's style SHALL stay as the client-attach capability defines it, distinct for the focused window. A window's terminal size SHALL NOT depend on these options.

#### Scenario: Tiles without side borders
- **WHEN** `user/init.lua` sets `tile_border_sides` to `{ "top", "bottom" }`, the client's 80×24 terminal sets the screen area, the sidebar is not set up and the client has no bar, and one window sits in a column of width 1/2
- **THEN** the tile's top and bottom rows show `─` from column 0 to column 39, columns 0 and 39 are blank between them, and `tput cols` in the window prints `38`

#### Scenario: Floating windows differ from tiles
- **WHEN** `user/init.lua` sets `tile_border_chars` to `"plain"` and `floating_border_chars` to `"double"`, and a floating window is shown over a tile
- **THEN** the tile's corners are drawn with `┌ ┐ └ ┘` and the floating window's with `╔ ╗ ╚ ╝`

#### Scenario: Two clients with different borders
- **WHEN** two clients with 80×24 terminals view the same band, the first sets `tile_border_sides` to `{}` and the second keeps the default
- **THEN** both clients show each window's whole grid
- **AND** only the second client draws border characters around it
