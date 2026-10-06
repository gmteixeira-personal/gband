## MODIFIED Requirements

### Requirement: Input to the server
The client SHALL send each key press and repeat that the key bindings do not consume to the server as a key naming the focused window, each paste as a paste naming the focused window, and each change of its reported size, as "Ribbon area beside the bars" defines it, as a resize carrying the reported size. While a plugin window is focused, the plugin-windows capability SHALL take the keys and the pastes instead, whether or not a window is focused. Otherwise, keys and pastes SHALL be dropped while no window is focused. Keys the input-encoding capability cannot represent SHALL be dropped. On attach, the client SHALL send its reported size as the terminal size its hello carries, as the wire-protocol capability defines the hello. Its bars SHALL already be placed for that report, as the configuration loaded before the handshake adds them.

#### Scenario: Typing runs a command
- **WHEN** the user types `echo hi` and Enter
- **THEN** the focused window prints `hi`

#### Scenario: Typing reaches only the focused window
- **WHEN** two windows are open with the second focused and the user types `echo hi` and Enter
- **THEN** the second window prints `hi`
- **AND** the first window's screen is unchanged

#### Scenario: Resize reaches the program
- **WHEN** the client has no bar, the only window sits in a column of width 1/2, the user resizes the terminal to 70 columns and runs `tput cols`
- **THEN** the window prints `33`

#### Scenario: Resize beside the sidebar
- **WHEN** the sidebar is a left bar 1 column wide, the only window sits in a column of width 1/2, the user resizes the terminal to 71 columns and runs `tput cols`
- **THEN** the client reports the size 70×24 and the window prints `33`

#### Scenario: First report beside the sidebar
- **WHEN** the default configuration is in use and the user runs `gband attach -s fresh` in an 80×24 terminal, creating the session
- **THEN** the client's hello carries the size 79×24
- **AND** the session's first window starts with a PTY of 37 columns by 22 rows

#### Scenario: Height beside the sidebar
- **WHEN** the client's terminal is 80×24, the sidebar is a left bar 1 column wide, and the only window runs `tput lines`
- **THEN** the window prints `22`

#### Scenario: Typing into a focused floating plugin window
- **WHEN** a floating plugin window is focused and the user types `ls`
- **THEN** nothing is sent to the server

#### Scenario: Paste into a focused floating plugin window
- **WHEN** a floating plugin window without `on_input` is focused and the user pastes `hello`
- **THEN** the paste is discarded and nothing is sent to the server

#### Scenario: Paste into a plugin window that takes text
- **WHEN** a floating plugin window whose `on_input` records its arguments is focused and the user pastes `hello`
- **THEN** `on_input` runs with `hello` and nothing is sent to the server

#### Scenario: Paste on a band with no window
- **WHEN** the viewed band holds no window, a floating plugin window whose `on_input` records its arguments is focused, and the user pastes `hi`
- **THEN** `on_input` runs with `hi`

### Requirement: Ribbon area beside the bars
The client's ribbon area SHALL be the cells of its terminal that its bars leave, as the bars capability places them: the terminal's full height, from the first column the left bars leave to the last column the right bars leave. The client's reported size SHALL be the ribbon area's size, so the session's screen area, while this client sets it, is the space its bars leave. Wherever the layout-view and animations capabilities speak of the client's terminal, its size, its top row or its first column, they SHALL mean the ribbon area, its size, its top row and its first column. The client's view SHALL use the ribbon area as its viewport, so the camera, the shown windows and the drawn bands follow the ribbon area, not the terminal. Tiles SHALL keep the sizes the layout gives them for the session's screen area, so a tile measured for another client's screen area MAY be wider or narrower than this client's ribbon area.

The reported size SHALL change when the terminal changes size and when the bars change the ribbon area's size. Every change of the reported size SHALL be reported to the server, as "Input to the server" defines, and the client's drawn state SHALL snap, as the animations capability defines for a terminal resize. A change of the ribbon area that keeps its size SHALL be handled as the bars capability defines, and SHALL NOT be reported.

#### Scenario: Default sidebar
- **WHEN** the client's terminal is 80×24 and the default configuration is in use
- **THEN** the client reports the size 79×24
- **AND** column 0 shows the sidebar and the ribbon area spans columns 1 to 79

#### Scenario: Sidebar off
- **WHEN** the client's terminal is 80×24 and the client has no bar
- **THEN** the client reports the size 80×24 and the ribbon area is the whole terminal

#### Scenario: Full width beside the sidebar
- **WHEN** the client's terminal is 80×24, the default configuration is in use, the client sets the screen area, and the user toggles full width on the only window's column
- **THEN** the tile is 79 columns wide, drawn on screen columns 1 to 79, with its right border on column 79
- **AND** `tput cols` in the window prints `77`

#### Scenario: Turning the sidebar off
- **WHEN** the client's terminal is 80×24, the client sets the screen area, the sidebar is a left bar 1 column wide, the only window sits in a column of width 1/2, and the user removes the setup of `gband.sidebar` from `user/init.lua`
- **THEN** after the reload the client reports the size 80×24 once, the ribbon area is the whole terminal, and the window's tile is 40×24
- **AND** `tput cols` in the window prints `38`

#### Scenario: Moving the sidebar
- **WHEN** the sidebar is a left bar 1 column wide and the user sets it up with `gband.plugin("gband.sidebar", { side = "right" })` in `user/init.lua`
- **THEN** after the reload the client reports no resize
- **AND** on an 80×24 terminal the ribbon is drawn from column 0 to column 78, and the sidebar on column 79

#### Scenario: Ribbon right of a left bar
- **WHEN** the client's terminal is 80×24, the client sets the screen area, its only bar is a left bar of size 20, and the viewed band holds one column of width 1/2
- **THEN** the client reports 60×24
- **AND** the tile is 30 columns wide and is drawn on screen columns 20 to 49

#### Scenario: Another client's wider area
- **WHEN** this client has an 80×24 terminal and a left bar of size 20, a second client with an 80×24 terminal and no bar attaches after it, and the viewed band holds one column with full width on
- **THEN** this client draws the 80-column tile from screen column 20, cut after column 79

### Requirement: Ribbon presentation
After the handshake, the client SHALL take the terminal full screen in raw mode with bracketed paste enabled. It SHALL keep its own grid of every window, as the wire-protocol capability defines, and its own view, as the layout-view capability defines. It SHALL draw only the viewed band, except during a band switch, when it SHALL draw the bands the animations capability places on screen. It SHALL draw the ribbon in the ribbon area and each shown bar where the bars capability places it. It SHALL draw the viewed band's floating windows over the tiles, in its stacking order, as the floating-windows capability defines. It SHALL draw the floating plugin windows it opened over the floating windows, as the plugin-windows capability defines. While the configuration capability shows a configuration error and the sidebar's error marker, as the sidebar capability defines, is not drawn, the client SHALL draw that error over the ribbon area's bottom row, after the tiles, the floating windows and the floating plugin windows.

Each window SHALL be drawn in its tile, as the layout capability's tile geometry gives it for the screen area in the latest layout. A tile SHALL be drawn at its strip position less the viewed band's camera position, counted from the ribbon area's left column, from the ribbon area's top row. While an animation runs, the tile's position and size, the camera and the band's top row SHALL be the drawn values the animations capability defines. At rest they equal the values above. Each tile SHALL show a one-cell border around the window's grid, which is drawn from its top-left corner. The border SHALL be drawn with the client's tile border options, as the borders capability defines. The focused window's border SHALL be drawn in a style distinct from the other borders.

Each floating window SHALL be drawn in its box, as the floating-windows capability places it for the screen area in the latest layout, from the ribbon area's top-left cell, whatever the camera. Its box SHALL show a one-cell border drawn with the client's floating border options, as the borders capability defines, in the style of a tile's border and in the focused style when it is the focused window, and its grid SHALL be drawn, cut and blanked inside the border as a tile's is. A box that crosses the ribbon area's edge SHALL be cut there, as a tile is.

A window's grid MAY differ in size from its tile's interior while the server has not yet resized the window. The border SHALL still follow the tile, at its drawn size while an animation runs. A grid larger than the interior SHALL be cut at the interior's right and bottom edges, and interior cells the grid does not cover SHALL be blank.

A tile that crosses the ribbon area's left, right, top or bottom edge SHALL be cut at that edge, and the part of the tile inside the ribbon area SHALL be drawn unchanged, except where a floating window, a floating plugin window or a configuration error covers it. No tile SHALL be resized to fit the ribbon area. A tile wholly outside the ribbon area SHALL NOT be drawn, no tile SHALL be drawn over a bar, and cells of the ribbon area that no tile, floating window, floating plugin window or configuration error covers SHALL be blank.

The terminal's cursor SHALL sit where the focused window's cursor is. It SHALL be hidden when the focused window hides its cursor, when that cell lies outside the ribbon area or outside the interior of the focused window's tile or box, when a floating window drawn over the focused window covers that cell, when no window is focused, while a floating plugin window is focused, or while the animations capability hides it during motion.

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
- **WHEN** the screen area is 120×40 because of another client, this client's terminal is 100×30, the sidebar is a left bar 1 column wide, and the viewed band holds one column of width 1/2
- **THEN** this client draws the top 30 rows of the 60-column tile on screen columns 1 to 60
- **AND** column 0 shows the sidebar

#### Scenario: Ribbon beside the sidebar
- **WHEN** the client's terminal is 80×24, the client sets the screen area, the sidebar is a left bar 1 column wide, and the viewed band holds two columns of width 1/2 with the first focused
- **THEN** the screen area is 79×24, the first tile spans screen columns 1 to 39 and the second spans 40 to 78, both whole, and column 0 shows the sidebar

#### Scenario: Empty band
- **WHEN** the client views an empty band
- **THEN** the ribbon area is blank and the cursor is hidden

#### Scenario: Grid smaller than its tile
- **WHEN** a window's tile is 50×30 and its grid is still 38×22
- **THEN** the border is drawn around the 50×30 tile
- **AND** the grid fills the top-left 38×22 cells of the interior, and the rest of the interior is blank

#### Scenario: Grid larger than its tile
- **WHEN** a window's tile is 40×24 and its grid is still 48×28
- **THEN** the interior shows the grid's top-left 38×22 cells
- **AND** the border is drawn unbroken around the 40×24 tile

#### Scenario: Mid-scroll frame
- **WHEN** the client has no bar, the camera is drawn at 20 while it scrolls toward 40, and the viewed band holds three columns of 40 cells
- **THEN** the second tile fills screen columns 20 to 59
- **AND** the cursor is hidden

#### Scenario: Configuration error over the ribbon
- **WHEN** the client's 40×6 terminal sets the screen area, the client has no bar, the viewed band holds one column of width 1/2, and the client shows a configuration error
- **THEN** the bottom row shows the error, cut at the terminal's width
- **AND** the rows above it show the tile unchanged

#### Scenario: Floating plugin window over two tiles
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, the viewed band holds two columns of width 1/2, and a floating plugin window with a border spans columns 20 to 59 and rows 6 to 16
- **THEN** those cells show the floating plugin window, and the tiles show around it
- **AND** the cursor is hidden while the floating plugin window is focused

#### Scenario: Error banner over a floating plugin window
- **WHEN** the client has no bar, a floating plugin window covers the ribbon area's bottom row, and the client shows a configuration error
- **THEN** the bottom row shows the error

#### Scenario: Floating window over two tiles
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, the viewed band holds two columns of width 1/2 and a floating window whose box spans columns 20 to 59 and rows 6 to 17
- **THEN** those cells show the floating window with its border, and the tiles show around it

#### Scenario: Floating window ignores the camera
- **WHEN** the client has no bar, the floating window's box starts at column 20, and the camera scrolls from 0 to 40
- **THEN** the box is still drawn from screen column 20

#### Scenario: Floating plugin window over a floating window
- **WHEN** a floating plugin window the client opened and a floating window cover the same cell
- **THEN** that cell shows the floating plugin window

#### Scenario: Cursor under a floating window
- **WHEN** a tiled window is focused and its cursor sits in a cell that a floating window covers
- **THEN** the cursor is hidden
