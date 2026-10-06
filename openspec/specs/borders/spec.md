# borders Specification

## Purpose
Defines how the client draws a one-cell border: which of its sides are drawn and with which characters. It covers the border options of tiled and floating windows and the border table of a floating plugin window, all of which only change drawing and never a window's size.

## Requirements

### Requirement: Border definition
A border definition SHALL be a set of sides and a character set. The sides SHALL be any of `top`, `right`, `bottom` and `left`. The character set SHALL be one of the named sets below, or a list of exactly eight strings, each one cell wide as `gband.ui.width` measures it, giving in this order the top-left corner, the top side, the top-right corner, the right side, the bottom-right corner, the bottom side, the bottom-left corner and the left side.

| name | characters in that order |
|---|---|
| `plain` | `┌ ─ ┐ │ ┘ ─ └ │` |
| `rounded` | `╭ ─ ╮ │ ╯ ─ ╰ │` |
| `double` | `╔ ═ ╗ ║ ╝ ═ ╚ ║` |
| `thick` | `┏ ━ ┓ ┃ ┛ ━ ┗ ┃` |

A list of another length, or holding a string that is not one cell wide, SHALL be rejected naming the setting that received it.

#### Scenario: Custom characters
- **WHEN** `user/init.lua` sets `tile_border_chars` to `{ "+", "-", "+", "|", "+", "-", "+", "|" }`
- **THEN** every tile's border is drawn with `+` corners, `-` top and bottom sides and `|` left and right sides

#### Scenario: Wide character rejected
- **WHEN** line 2 of `user/init.lua` sets `tile_border_chars` to a list whose first string is `日`
- **THEN** an error at `user/init.lua` line 2 naming `tile_border_chars` is reported

### Requirement: Drawing a border
A border SHALL take the outermost cell on every side of its box, whichever sides are drawn. The box's interior SHALL be the box less one column on the left and on the right, and one row at the top and at the bottom, whatever the sides are.

- A drawn side SHALL fill its row or column with its side character, except at its corners.
- A corner SHALL be drawn with its corner character when both sides that meet there are drawn. When only one of them is drawn, the corner SHALL be drawn with that side's character, so the side reaches the box's edge. When neither is drawn, the corner SHALL be blank.
- Every cell of a side that is not drawn SHALL be blank.

Border cells SHALL be drawn in the border's style, blank cells included.

#### Scenario: Only the left side
- **WHEN** a 10×5 box is drawn with the sides `left` and the `plain` set
- **THEN** column 0 shows `│` on rows 0 to 4
- **AND** row 0, row 4 and column 9 are blank, and the interior is columns 1 to 8 and rows 1 to 3

#### Scenario: Top and left sides
- **WHEN** a 10×5 box is drawn with the sides `top` and `left` and the `rounded` set
- **THEN** cell (0, 0) shows `╭`, row 0 shows `─` from column 1 to column 9, and column 0 shows `│` from row 1 to row 4

#### Scenario: No sides
- **WHEN** a box is drawn with no sides
- **THEN** its outermost cells are blank and its interior keeps its size

### Requirement: Window border options
Tiled windows SHALL be drawn with the border definition of the client options `tile_border_sides` and `tile_border_chars`. Floating windows SHALL be drawn with the definition of `floating_border_sides` and `floating_border_chars`. Each SHALL apply only to the client that set it. A border's style SHALL stay as the client-attach capability defines it, distinct for the focused window. A window's terminal size SHALL NOT depend on these options.

#### Scenario: Tiles without side borders
- **WHEN** `user/init.lua` sets `tile_border_sides` to `{ "top", "bottom" }`, the client's 80×24 terminal sets the screen area, the status line is not set up, and one window sits in a column of width 1/2
- **THEN** the tile's top and bottom rows show `─` from column 0 to column 39, columns 0 and 39 are blank between them, and `tput cols` in the window prints `38`

#### Scenario: Floating windows differ from tiles
- **WHEN** `user/init.lua` sets `tile_border_chars` to `"plain"` and `floating_border_chars` to `"double"`, and a floating window is shown over a tile
- **THEN** the tile's corners are drawn with `┌ ┐ └ ┘` and the floating window's with `╔ ╗ ╚ ╝`

#### Scenario: Two clients with different borders
- **WHEN** two clients with 80×24 terminals view the same band, the first sets `tile_border_sides` to `{}` and the second keeps the default
- **THEN** both clients show each window's whole grid
- **AND** only the second client draws border characters around it

### Requirement: Floating plugin window border tables
A floating plugin window's `border`, as the plugin-windows capability defines it, SHALL be a boolean or a table holding `sides`, a list of side names, and `chars`, a character set, both optional. A table SHALL draw the border with `sides`, all four sides when `sides` is absent, and `chars`, `plain` when `chars` is absent. `true` SHALL mean all four sides and the `plain` set. A float with a border table SHALL have the content area of a float with `border` on.

#### Scenario: Floating plugin window with a rounded top only
- **WHEN** a binding function opens a floating plugin window of width 20 and height 5 with `border = { sides = { "top" }, chars = "rounded" }`
- **THEN** the floating plugin window's top row shows `─` across its width, its other edge cells are blank, and its content area is 18×3
