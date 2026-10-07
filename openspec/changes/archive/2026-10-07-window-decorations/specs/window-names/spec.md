## MODIFIED Requirements

### Requirement: Border title
While titles are on, as "Titles turned off" defines, the client SHALL draw the shown name of every tiled and floating window that runs a program on the window's top border, after drawing the border. It SHALL start at the border's second column. It SHALL be cut, leaving out whole characters, as `gband.ui.width` measures them, to a room of:
- the border's width less 2 cells, when the client draws no decorations on that border;
- the border's width less 4 cells and less the width of the decorations, when the client draws them, as the window-decorations capability defines.

So at least one cell of the border SHALL stay between the title and the decorations. A room of 0 cells SHALL draw no title. The title SHALL be drawn in the window's border style, as the client-attach capability defines it, distinct for the focused window. When the window's border definition, as the borders capability defines it, does not draw the top side, the client SHALL draw no title. A drawn window SHALL have no title drawn.

#### Scenario: Title on a tile
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, the only window sits in a column of width 1/2, and its name is `vim`
- **THEN** the tile's top row shows `vim` from column 1 to column 3, with the border's top side on either side

#### Scenario: Long title cut
- **WHEN** a tile is 10 columns wide and its shown name is `cargo test --workspace`
- **THEN** its top row shows `cargo te` from column 1 to column 8

#### Scenario: Title cut before decorations
- **WHEN** a tile is 20 columns wide, its shown name is `cargo test --workspace`, and its decorations are the spans `[_]`, `[□]` and `[X]`
- **THEN** its top row shows `cargo t` from column 1 to column 7, `─` on column 8, and `[_][□][X]` from column 9 to column 17

#### Scenario: Short title beside decorations
- **WHEN** a tile is 40 columns wide, its shown name is `vim`, and its decorations are the spans `[_]`, `[□]` and `[X]`
- **THEN** its top row shows `vim` from column 1 to column 3, `─` from column 4 to column 28, and `[_][□][X]` from column 29 to column 37

#### Scenario: Focused title styled as the focused border
- **WHEN** two tiles are drawn with the first focused
- **THEN** the first tile's title is drawn in `WindowBorderFocused`'s resolved style and the second's in `WindowBorder`'s

#### Scenario: No top side
- **WHEN** `user/init.lua` sets `tile_border_sides` to `{ "left", "right" }`
- **THEN** no tile's top row shows its name

#### Scenario: Title follows the program
- **WHEN** a tile shows the title `bash` and the user runs `vim` in it
- **THEN** within one second the tile's top border shows `vim`
