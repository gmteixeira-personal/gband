## MODIFIED Requirements

### Requirement: Present the ribbon
After the handshake, the client SHALL take the terminal full screen in raw mode with bracketed paste enabled. It SHALL keep its own grid of every pane, as the wire-protocol capability defines, and its own view, as the layout-view capability defines. It SHALL draw only the viewed band, except during a band switch, when it SHALL draw the bands the animations capability places on screen. It SHALL draw the ribbon in the ribbon area and the status line in the rows the status-line capability gives it. It SHALL draw the viewed band's floating panes over the tiles, in its stacking order, as the floating-panes capability defines. It SHALL draw the floats it opened over the floating panes, as the plugin-windows capability defines. While the configuration capability shows a configuration error and no status line is drawn, the client SHALL draw that error over the ribbon area's bottom row, after the tiles, the floating panes and the floats.

Each pane SHALL be drawn in its tile, as the layout capability's tile geometry gives it for the screen area in the latest layout. A tile SHALL be drawn at its strip position less the viewed band's camera position, from the ribbon area's top row. While an animation runs, the tile's position and size, the camera and the band's top row SHALL be the drawn values the animations capability defines. At rest they equal the values above. Each tile SHALL show a one-cell border around the pane's grid, which is drawn from its top-left corner. The focused pane's border SHALL be drawn in a style distinct from the other borders.

Each floating pane SHALL be drawn in its box, as the floating-panes capability places it for the screen area in the latest layout, from the ribbon area's top-left cell, whatever the camera. Its box SHALL show the same border as a tile, in the focused style when it is the focused pane, and its grid SHALL be drawn, cut and blanked inside the border as a tile's is. A box that crosses the ribbon area's edge SHALL be cut there, as a tile is.

A pane's grid MAY differ in size from its tile's interior while the server has not yet resized the pane. The border SHALL still follow the tile, at its drawn size while an animation runs. A grid larger than the interior SHALL be cut at the interior's right and bottom edges, and interior cells the grid does not cover SHALL be blank.

A tile that crosses the ribbon area's left, right, top or bottom edge SHALL be cut at that edge, and the part of the tile inside the ribbon area SHALL be drawn unchanged, except where a floating pane, a float or a configuration error covers it. No tile SHALL be resized to fit the ribbon area. A tile wholly outside the ribbon area SHALL NOT be drawn, no tile SHALL be drawn over the status line, and cells of the ribbon area that no tile, floating pane, float or configuration error covers SHALL be blank.

The terminal's cursor SHALL sit where the focused pane's cursor is. It SHALL be hidden when the focused pane hides its cursor, when that cell lies outside the ribbon area or outside the interior of the focused pane's tile or box, when a floating pane drawn over the focused pane covers that cell, when no pane is focused, while a float is focused, or while the animations capability hides it during motion.

#### Scenario: Two columns side by side
- **WHEN** the client's 80×24 terminal sets the screen area, the status line is off, and the viewed band holds two columns of width 1/2 with the second focused
- **THEN** the first tile fills screen columns 0 to 39 and the second fills 40 to 79, each with a border
- **AND** only the second tile's border has the focused style
- **AND** the cursor sits in the second tile

#### Scenario: Column clipped at the left edge
- **WHEN** the client's 80×24 terminal sets the screen area, the status line is off, the viewed band holds two columns of width 2/3, and the client focuses the second
- **THEN** the second tile fills screen columns 27 to 79
- **AND** screen columns 0 to 26 show the rightmost 27 columns of the first tile, without its left border

#### Scenario: Tile larger than the terminal
- **WHEN** the screen area is 120×40 because of another client, this client's terminal is 100×30, the status line is off, and the viewed band holds one column of width 1/2
- **THEN** this client draws the top 30 rows of the 60-column tile
- **AND** the tile's bottom border is not drawn

#### Scenario: Tile taller than the ribbon area
- **WHEN** the screen area is 120×40 because of another client, this client's terminal is 100×30, the status line takes the bottom row, and the viewed band holds one column of width 1/2
- **THEN** this client draws the top 29 rows of the 60-column tile on rows 0 to 28
- **AND** row 29 shows the status line

#### Scenario: Ribbon below a top status line
- **WHEN** the client's 80×24 terminal sets the screen area, the status line takes the top row, and the viewed band holds two columns of width 1/2
- **THEN** both tiles span rows 1 to 23, and row 0 shows the status line

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
- **WHEN** the client's 40×6 terminal sets the screen area, the status line is off, the viewed band holds one column of width 1/2, and the client shows a configuration error
- **THEN** the bottom row shows the error, cut at the terminal's width
- **AND** the rows above it show the tile unchanged

#### Scenario: Float over two tiles
- **WHEN** the client's 80×24 terminal sets the screen area, the status line is off, the viewed band holds two columns of width 1/2, and a float with a border spans columns 20 to 59 and rows 6 to 16
- **THEN** those cells show the float, and the tiles show around it
- **AND** the cursor is hidden while the float is focused

#### Scenario: Error banner over a float
- **WHEN** the status line is off, a float covers the ribbon area's bottom row, and the client shows a configuration error
- **THEN** the bottom row shows the error


#### Scenario: Floating pane over two tiles
- **WHEN** the client's 80×24 terminal sets the screen area, the status line is off, the viewed band holds two columns of width 1/2 and a floating pane whose box spans columns 20 to 59 and rows 6 to 17
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

### Requirement: Key bindings
The client SHALL take its key tables and its prefix key from the configuration, as the configuration capability defines them. The client SHALL keep one active key table, which SHALL be `root` outside a key sequence. While `root` is active, a key bound in `root` SHALL run its binding, and the prefix key SHALL make `prefix` the active table. The client SHALL send neither to the server. Any other key while `root` is active SHALL go to the focused window when there is one, as the plugin-windows capability defines, and SHALL otherwise be sent to the focused pane. While another table is active, the next key SHALL end the sequence: a key bound in that table SHALL run its binding, and any other key SHALL be discarded together with the keys that began the sequence. When the sequence ends, the active table SHALL become the table its binding entered with `gband.keymap.enter`, or `root` when it entered none. A binding run while `root` is active MAY also enter a table, which then becomes active. Entering a table with no binding SHALL be an error raised by `gband.keymap.enter`. When `prefix` holds no binding, the prefix key SHALL be handled like any other key. A reload SHALL make `root` the active table.

`gband.keymap.current_table()` SHALL return the name of the active table, and `root` while the configuration loads. Each change of the active table SHALL emit `KeyTableChanged`, as the lua-events capability defines, naming the new and the previous table.

With no configuration file, the bindings SHALL be those the default configuration makes: Ctrl+Space as the prefix, no binding in `root`, and these bindings in `prefix`:

| key after Ctrl+Space | Lua binding | action | kind |
|---|---|---|---|
| `h` | `prefix h` | focus the column to the left | view |
| `l` | `prefix l` | focus the column to the right | view |
| `j` | `prefix j` | focus the pane below | view |
| `k` | `prefix k` | focus the pane above | view |
| `u` | `prefix u` | view the band below | view |
| `i` | `prefix i` | view the band above | view |
| Enter | `prefix enter` | open a pane right of the focused pane's column | session |
| `q` | `prefix q` | close the focused pane | session |
| `[` | `prefix [` | consume or expel the focused pane to the left | session |
| `]` | `prefix ]` | consume or expel the focused pane to the right | session |
| `r` | `prefix r` | cycle the width of the focused pane's column | session |
| `f` | `prefix f` | toggle full width of the focused pane's column | session |
| `-` | `prefix -` | shrink the width of the focused pane's column | session |
| `=` | `prefix =` | grow the width of the focused pane's column | session |
| `_` | `prefix _` | shrink the height of the focused pane | session |
| `+` | `prefix +` | grow the height of the focused pane | session |
| `R` | `prefix R` | reset the height of the focused pane | session |
| `v` | `prefix v` | float or tile the focused pane | session |
| `V` | `prefix V` | switch focus between floating and tiled panes | view |
| Ctrl+H | `prefix ctrl+h` | move the focused pane's column, or its floating box, to the left | session |
| Ctrl+L | `prefix ctrl+l` | move the focused pane's column, or its floating box, to the right | session |
| Ctrl+J | `prefix ctrl+j` | move the focused pane, or its floating box, down | session |
| Ctrl+K | `prefix ctrl+k` | move the focused pane, or its floating box, up | session |
| Ctrl+Left | `prefix ctrl+left` | move the focused pane's column, or its floating box, to the left | session |
| Ctrl+Right | `prefix ctrl+right` | move the focused pane's column, or its floating box, to the right | session |
| Ctrl+Down | `prefix ctrl+down` | move the focused pane, or its floating box, down | session |
| Ctrl+Up | `prefix ctrl+up` | move the focused pane, or its floating box, up | session |
| `D` | `prefix D` | detach | client |
| Ctrl+Space | `prefix prefix` | send the prefix key to the focused pane | client |
| any other key | — | discard both keys | — |

A binding to an action SHALL dispatch it. A binding to a Lua function SHALL run the function, as the configuration capability defines. A view action SHALL change this client's view as the layout-view capability defines, and SHALL send nothing to the server. A session action SHALL be sent to the server as an action naming the focused pane, resolved as the actions capability defines. Open pane SHALL name the viewed band and the tiled pane this client focused most recently there, or no pane when there is none. Any other session action, and sending the prefix key, SHALL do nothing when no pane is focused. A character key SHALL match a binding by its character, Ctrl and Alt, so a `D` matches whether or not the terminal reports Shift with it.

#### Scenario: Detach
- **WHEN** the user presses Ctrl+Space then Shift+D
- **THEN** the client detaches

#### Scenario: Lowercase d does not detach
- **WHEN** the user presses Ctrl+Space then `d`
- **THEN** the client stays attached and nothing is sent to the pane

#### Scenario: Literal Ctrl+Space
- **WHEN** the user runs `cat -v`, presses Ctrl+Space twice, then presses Enter
- **THEN** the focused pane receives `\x00` once
- **AND** `cat -v` prints `^@` on a line of its own

#### Scenario: Literal Ctrl+A
- **WHEN** no `user/init.lua` exists and the user presses Ctrl+A at a bash prompt with text typed
- **THEN** the focused pane receives `\x01`
- **AND** bash moves the cursor to the start of the line

#### Scenario: Unbound key after the prefix
- **WHEN** the user presses Ctrl+Space then `x`
- **THEN** nothing is sent to the pane

#### Scenario: Open a pane
- **WHEN** one pane is focused and the user presses Ctrl+Space then Enter
- **THEN** a second tile with a shell prompt appears right of the first
- **AND** the new pane is focused, so `echo $GBAND_PANE` runs in it

#### Scenario: Focus back to the left
- **WHEN** the user has opened a second pane and presses Ctrl+Space then `h`, then types `echo left` and Enter
- **THEN** `left` appears in the first tile only

#### Scenario: Close the focused pane
- **WHEN** two panes are open with the second focused and the user presses Ctrl+Space then `q`
- **THEN** the second tile disappears and the first pane is focused

#### Scenario: Another band
- **WHEN** the user presses Ctrl+Space then `u`, then Ctrl+Space then Enter, then Ctrl+Space then `i`
- **THEN** the client shows the first band with its original pane focused
- **AND** the second band holds the new pane

#### Scenario: Grow the column
- **WHEN** the client's 80×24 terminal sets the screen area, the only pane sits in a column of width 1/2, and the user presses Ctrl+Space then `=`, waits, and runs `tput cols`
- **THEN** the tile is 48 columns wide
- **AND** the pane prints `46`

#### Scenario: Grow the pane's height
- **WHEN** the client's 80×25 terminal, whose status line takes one row, sets the screen area, a column holds two panes with automatic heights with the top one focused, and the user presses Ctrl+Space then `+`
- **THEN** the top tile is 14 rows high and the bottom tile is 10 rows high

#### Scenario: Direct binding acts without the prefix
- **WHEN** `user/init.lua` binds `alt+h` to `gband.action.focus_column_left`, the second of two panes is focused, and the user presses Alt+H
- **THEN** the first pane is focused
- **AND** neither pane receives the key

#### Scenario: Unbound Alt key reaches the pane
- **WHEN** `user/init.lua` binds `alt+h` and the user presses Alt+X
- **THEN** the focused pane receives `\x1bx`

#### Scenario: Another prefix key
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua` that then sets `prefix` to `"ctrl+b"`, and the user presses Ctrl+B then `q` with two panes open
- **THEN** the focused pane closes
- **AND** pressing Ctrl+Space sends `\x00` to the focused pane

#### Scenario: No prefix binding left
- **WHEN** `user/init.lua` makes no prefix binding and the user presses Ctrl+Space then `h`
- **THEN** the focused pane receives `\x00` and then `h`

#### Scenario: Named key table
- **WHEN** `user/init.lua` binds `h` and `l` in the table `move`, and binds `prefix m` to a function that calls `gband.keymap.enter("move")`, and the user presses Ctrl+Space, `m`, then `l`, with the first of two columns focused
- **THEN** the second column is focused
- **AND** a further `l` reaches the focused pane

#### Scenario: Unbound key in a named table
- **WHEN** the user enters the table `move` and presses `x`
- **THEN** nothing is sent to the pane and `root` is active again

#### Scenario: Key table change events
- **WHEN** a `KeyTableChanged` handler records each payload and the user presses Ctrl+Space then `h`
- **THEN** the handler records `prefix` with previous `root`, then `root` with previous `prefix`

#### Scenario: Current table
- **WHEN** a binding in `prefix` calls `gband.keymap.current_table()`
- **THEN** it returns `prefix`

#### Scenario: Unbound key goes to the focused window
- **WHEN** a float with `keys = { j = fn }` is focused and the user presses `j`
- **THEN** `fn` runs and the focused pane receives nothing

#### Scenario: Root binding before the window
- **WHEN** `user/init.lua` binds `alt+h` in `root` to `gband.action.focus_column_left`, a float binding `alt+h` in its `keys` is focused, and the user presses Alt+H
- **THEN** the root binding runs and the float's function does not


#### Scenario: Float the focused pane
- **WHEN** the client's 80×24 terminal sets the screen area, the status line is off, the only pane sits in a column of width 1/2, and the user presses Ctrl+Space then `v`
- **THEN** the pane is drawn in a box spanning columns 20 to 59 and rows 2 to 21, and stays focused

#### Scenario: Switch to the tiled layer and back
- **WHEN** two panes are open, the second floats and is focused, and the user presses Ctrl+Space then `V`, then types `echo tiled` and Enter
- **THEN** `tiled` appears in the first pane only
- **AND** pressing Ctrl+Space then `V` again focuses the floating pane

#### Scenario: Move a column with Ctrl+H and Ctrl+Right
- **WHEN** the viewed band holds columns A and B with B focused, and the user presses Ctrl+Space then Ctrl+H
- **THEN** the band holds B and A, in that order, and B stays focused, and neither pane receives a key
- **AND** pressing Ctrl+Space then Ctrl+Right restores A and B

#### Scenario: Move a pane with Ctrl+J
- **WHEN** a column holds P1 above P2 with P1 focused, and the user presses Ctrl+Space then Ctrl+J
- **THEN** the column holds P2 above P1, and P1 stays focused

#### Scenario: Move a floating pane
- **WHEN** the client's 80×24 terminal sets the screen area, a floating pane is focused with its box starting at column 20, and the user presses Ctrl+Space then Ctrl+L
- **THEN** the box is drawn from column 28
