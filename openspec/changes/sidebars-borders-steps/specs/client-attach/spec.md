## MODIFIED Requirements

### Requirement: Ribbon area
The client's reported size SHALL be its terminal's size. The client's ribbon area SHALL be the cells of its terminal that its bars leave, as the bars capability places them: the terminal's full height, from the first column the left bars leave to the last column the right bars leave. Wherever the layout-view and animations capabilities speak of the client's terminal, its size, its top row or its first column, they SHALL mean the ribbon area, its size, its top row and its first column. The client's view SHALL use the ribbon area as its viewport, so the camera, the shown panes and the drawn bands follow the ribbon area, not the terminal. Tiles SHALL keep the sizes the layout gives them for the session's screen area, whatever the ribbon area's width.

The reported size SHALL change only when the terminal changes size. Every change of the reported size SHALL be handled as a change of the terminal's size: the client SHALL report the new size to the server, as "Send input" defines, and its drawn state SHALL snap, as the animations capability defines for a terminal resize. A change of the ribbon area that the bars cause SHALL be handled as the bars capability defines.

#### Scenario: Status line at the bottom
- **WHEN** the client's terminal is 80×24 and `user/init.lua` still sets `statusline_position = "bottom"`
- **THEN** an error naming `statusline_position` is reported, and the client reports the size 80×24

#### Scenario: Status line off
- **WHEN** the client's terminal is 80×24 and the client has no bar
- **THEN** the client reports the size 80×24 and the ribbon area is the whole terminal

#### Scenario: Turning the status line off
- **WHEN** the client's 80×24 terminal sets the screen area, the status line is a left bar 20 columns wide, the only pane sits in a column of width 1/2, and the user removes the setup of `gband.statusline` from `user/init.lua`
- **THEN** after the reload the client reports no resize, the ribbon area is the whole terminal, and the pane's tile is still 40×24
- **AND** `tput cols` in the pane still prints `38`

#### Scenario: Moving the status line
- **WHEN** the status line is a left bar 20 columns wide and the user sets it up with `side = "right"` in `user/init.lua`
- **THEN** after the reload the client reports no resize
- **AND** on an 80×24 terminal the ribbon is drawn from column 0 to column 59, and the status line from column 60

#### Scenario: Ribbon right of a left bar
- **WHEN** the client's 80×24 terminal sets the screen area, its only bar is a left bar of size 20, and the viewed band holds one column of width 1/2
- **THEN** the client reports 80×24
- **AND** the tile is 40 columns wide and is drawn on screen columns 20 to 59

### Requirement: Present the ribbon
After the handshake, the client SHALL take the terminal full screen in raw mode with bracketed paste enabled. It SHALL keep its own grid of every pane, as the wire-protocol capability defines, and its own view, as the layout-view capability defines. It SHALL draw only the viewed band, except during a band switch, when it SHALL draw the bands the animations capability places on screen. It SHALL draw the ribbon in the ribbon area and each shown bar where the bars capability places it. It SHALL draw the viewed band's floating panes over the tiles, in its stacking order, as the floating-panes capability defines. It SHALL draw the floats it opened over the floating panes, as the plugin-windows capability defines. While the configuration capability shows a configuration error and no status line is drawn, the client SHALL draw that error over the ribbon area's bottom row, after the tiles, the floating panes and the floats.

Each pane SHALL be drawn in its tile, as the layout capability's tile geometry gives it for the screen area in the latest layout. A tile SHALL be drawn at its strip position less the viewed band's camera position, counted from the ribbon area's left column, from the ribbon area's top row. While an animation runs, the tile's position and size, the camera and the band's top row SHALL be the drawn values the animations capability defines. At rest they equal the values above. Each tile SHALL show a one-cell border around the pane's grid, which is drawn from its top-left corner. The border SHALL be drawn with the client's tile border options, as the borders capability defines. The focused pane's border SHALL be drawn in a style distinct from the other borders.

Each floating pane SHALL be drawn in its box, as the floating-panes capability places it for the screen area in the latest layout, from the ribbon area's top-left cell, whatever the camera. Its box SHALL show a one-cell border drawn with the client's floating border options, as the borders capability defines, in the style of a tile's border and in the focused style when it is the focused pane, and its grid SHALL be drawn, cut and blanked inside the border as a tile's is. A box that crosses the ribbon area's edge SHALL be cut there, as a tile is.

A pane's grid MAY differ in size from its tile's interior while the server has not yet resized the pane. The border SHALL still follow the tile, at its drawn size while an animation runs. A grid larger than the interior SHALL be cut at the interior's right and bottom edges, and interior cells the grid does not cover SHALL be blank.

A tile that crosses the ribbon area's left, right, top or bottom edge SHALL be cut at that edge, and the part of the tile inside the ribbon area SHALL be drawn unchanged, except where a floating pane, a float or a configuration error covers it. No tile SHALL be resized to fit the ribbon area. A tile wholly outside the ribbon area SHALL NOT be drawn, no tile SHALL be drawn over a bar, and cells of the ribbon area that no tile, floating pane, float or configuration error covers SHALL be blank.

The terminal's cursor SHALL sit where the focused pane's cursor is. It SHALL be hidden when the focused pane hides its cursor, when that cell lies outside the ribbon area or outside the interior of the focused pane's tile or box, when a floating pane drawn over the focused pane covers that cell, when no pane is focused, while a float is focused, or while the animations capability hides it during motion.

#### Scenario: Two columns side by side
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, and the viewed band holds two columns of width 1/2 with the second focused
- **THEN** the first tile fills screen columns 0 to 39 and the second fills 40 to 79, each with a border
- **AND** only the second tile's border has the focused style
- **AND** the cursor sits in the second tile

#### Scenario: Column clipped at the left edge
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, the viewed band holds two columns of width 2/3, and the client focuses the second
- **THEN** the second tile fills screen columns 27 to 79
- **AND** screen columns 0 to 26 show the rightmost 27 columns of the first tile, without its left border

#### Scenario: Tile larger than the terminal
- **WHEN** the screen area is 120×40 because of another client, this client's terminal is 100×30, the client has no bar, and the viewed band holds one column of width 1/2
- **THEN** this client draws the top 30 rows of the 60-column tile
- **AND** the tile's bottom border is not drawn

#### Scenario: Tile taller than the ribbon area
- **WHEN** the screen area is 120×40 because of another client, this client's terminal is 100×30, the status line is a left bar 20 columns wide, and the viewed band holds one column of width 1/2
- **THEN** this client draws the top 30 rows of the 60-column tile on screen columns 20 to 79
- **AND** columns 0 to 19 show the status line

#### Scenario: Ribbon below a top status line
- **WHEN** the client's 80×24 terminal sets the screen area, the status line is a left bar 20 columns wide, and the viewed band holds two columns of width 1/2 with the first focused
- **THEN** the first tile spans screen columns 20 to 59, the second is cut after column 79, and columns 0 to 19 show the status line

#### Scenario: Empty band
- **WHEN** the client views an empty band
- **THEN** the ribbon area is blank and the cursor is hidden

#### Scenario: Grid smaller than its tile
- **WHEN** a pane's tile is 50×30 and its grid is still 38×22
- **THEN** the border is drawn around the 50×30 tile
- **AND** the grid fills the top-left 38×22 cells of the interior, and the rest of the interior is blank

#### Scenario: Grid larger than its tile
- **WHEN** a pane's tile is 40×24 and its grid is still 48×28
- **THEN** the interior shows the grid's top-left 38×22 cells
- **AND** the border is drawn unbroken around the 40×24 tile

#### Scenario: Mid-scroll frame
- **WHEN** the camera is drawn at 20 while it scrolls toward 40, and the viewed band holds three columns of 40 cells
- **THEN** the second tile fills screen columns 20 to 59
- **AND** the cursor is hidden

#### Scenario: Configuration error over the ribbon
- **WHEN** the client's 40×6 terminal sets the screen area, the client has no bar, the viewed band holds one column of width 1/2, and the client shows a configuration error
- **THEN** the bottom row shows the error, cut at the terminal's width
- **AND** the rows above it show the tile unchanged

#### Scenario: Float over two tiles
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, the viewed band holds two columns of width 1/2, and a float with a border spans columns 20 to 59 and rows 6 to 16
- **THEN** those cells show the float, and the tiles show around it
- **AND** the cursor is hidden while the float is focused

#### Scenario: Error banner over a float
- **WHEN** the client has no bar, a float covers the ribbon area's bottom row, and the client shows a configuration error
- **THEN** the bottom row shows the error


#### Scenario: Floating pane over two tiles
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, the viewed band holds two columns of width 1/2 and a floating pane whose box spans columns 20 to 59 and rows 6 to 17
- **THEN** those cells show the floating pane with its border, and the tiles show around it

#### Scenario: Floating pane ignores the camera
- **WHEN** the floating pane's box starts at column 20 and the camera scrolls from 0 to 40
- **THEN** the box is still drawn from screen column 20

#### Scenario: Float over a floating pane
- **WHEN** a float the client opened and a floating pane cover the same cell
- **THEN** that cell shows the float

#### Scenario: Cursor under a floating pane
- **WHEN** a tiled pane is focused and its cursor sits in a cell that a floating pane covers
- **THEN** the cursor is hidden

