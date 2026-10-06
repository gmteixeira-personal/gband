## MODIFIED Requirements

### Requirement: Send input
The client SHALL send each key press and repeat that the key bindings do not consume to the server as a key naming the focused window, each paste as a paste naming the focused window, except while a plugin window is focused, when the plugin-windows capability takes them, and each change of its reported size, as "Ribbon area" defines it, as a resize carrying the reported size. Keys and pastes SHALL be dropped while no window is focused. Keys the input-encoding capability cannot represent SHALL be dropped. On attach, the client SHALL report its reported size as its terminal size.

#### Scenario: Typing runs a command
- **WHEN** the user types `echo hi` and Enter
- **THEN** the focused window prints `hi`

#### Scenario: Typing reaches only the focused window
- **WHEN** two windows are open with the second focused and the user types `echo hi` and Enter
- **THEN** the second window prints `hi`
- **AND** the first window's screen is unchanged

#### Scenario: Resize reaches the program
- **WHEN** the only window sits in a column of width 1/2, the user resizes the terminal to 70 columns and runs `tput cols`
- **THEN** the window prints `33`

#### Scenario: Height excludes the status line
- **WHEN** the client's terminal is 80×24, the status line is a left bar 20 columns wide, and the only window runs `tput lines`
- **THEN** the window prints `22`

#### Scenario: Typing into a focused floating plugin window
- **WHEN** a floating plugin window is focused and the user types `ls`
- **THEN** nothing is sent to the server

#### Scenario: Paste into a focused floating plugin window
- **WHEN** a floating plugin window is focused and the user pastes `hello`
- **THEN** the paste is discarded and nothing is sent to the server

### Requirement: Ribbon area
The client's reported size SHALL be its terminal's size. The client's ribbon area SHALL be the cells of its terminal that its bars leave, as the bars capability places them: the terminal's full height, from the first column the left bars leave to the last column the right bars leave. Wherever the layout-view and animations capabilities speak of the client's terminal, its size, its top row or its first column, they SHALL mean the ribbon area, its size, its top row and its first column. The client's view SHALL use the ribbon area as its viewport, so the camera, the shown windows and the drawn bands follow the ribbon area, not the terminal. Tiles SHALL keep the sizes the layout gives them for the session's screen area, whatever the ribbon area's width.

The reported size SHALL change only when the terminal changes size. Every change of the reported size SHALL be handled as a change of the terminal's size: the client SHALL report the new size to the server, as "Send input" defines, and its drawn state SHALL snap, as the animations capability defines for a terminal resize. A change of the ribbon area that the bars cause SHALL be handled as the bars capability defines.

#### Scenario: Status line at the bottom
- **WHEN** the client's terminal is 80×24 and `user/init.lua` still sets `statusline_position = "bottom"`
- **THEN** an error naming `statusline_position` is reported, and the client reports the size 80×24

#### Scenario: Status line off
- **WHEN** the client's terminal is 80×24 and the client has no bar
- **THEN** the client reports the size 80×24 and the ribbon area is the whole terminal

#### Scenario: Turning the status line off
- **WHEN** the client's 80×24 terminal sets the screen area, the status line is a left bar 20 columns wide, the only window sits in a column of width 1/2, and the user removes the setup of `gband.statusline` from `user/init.lua`
- **THEN** after the reload the client reports no resize, the ribbon area is the whole terminal, and the window's tile is still 40×24
- **AND** `tput cols` in the window still prints `38`

#### Scenario: Moving the status line
- **WHEN** the status line is a left bar 20 columns wide and the user sets it up with `side = "right"` in `user/init.lua`
- **THEN** after the reload the client reports no resize
- **AND** on an 80×24 terminal the ribbon is drawn from column 0 to column 59, and the status line from column 60

#### Scenario: Ribbon right of a left bar
- **WHEN** the client's 80×24 terminal sets the screen area, its only bar is a left bar of size 20, and the viewed band holds one column of width 1/2
- **THEN** the client reports 80×24
- **AND** the tile is 40 columns wide and is drawn on screen columns 20 to 59

### Requirement: Present the ribbon
After the handshake, the client SHALL take the terminal full screen in raw mode with bracketed paste enabled. It SHALL keep its own grid of every window, as the wire-protocol capability defines, and its own view, as the layout-view capability defines. It SHALL draw only the viewed band, except during a band switch, when it SHALL draw the bands the animations capability places on screen. It SHALL draw the ribbon in the ribbon area and each shown bar where the bars capability places it. It SHALL draw the viewed band's floating windows over the tiles, in its stacking order, as the floating-windows capability defines. It SHALL draw the floating plugin windows it opened over the floating windows, as the plugin-windows capability defines. While the configuration capability shows a configuration error and no status line is drawn, the client SHALL draw that error over the ribbon area's bottom row, after the tiles, the floating windows and the floating plugin windows.

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

### Requirement: Key bindings
The client SHALL take its key tables, its modes and its prefix key from the configuration, as the configuration capability defines them. The client SHALL keep one active key table, which SHALL be `root` outside a key sequence and outside a mode. While `root` is active, a key bound in `root` SHALL run its binding, and the prefix key SHALL make `prefix` the active table. The client SHALL send neither to the server. Any other key while `root` is active SHALL go to the focused plugin window when there is one, as the plugin-windows capability defines, and SHALL otherwise be sent to the focused window.

While a table that is not a mode is active, the next key SHALL end the sequence: a key bound in that table SHALL run its binding, and any other key SHALL be discarded together with the keys that began the sequence. When the sequence ends, the active table SHALL become the table its binding entered with `gband.keymap.enter`, or `root` when it entered none.

While a mode is active, a key bound in it SHALL run its binding, and any other key SHALL be discarded. Neither SHALL be sent to the server, to the focused window or to a plugin window. The mode SHALL stay active after each key, until a binding enters another table with `gband.keymap.enter`, which then becomes active. Entering `root` ends the mode, and the keys that follow go to the focused plugin window or window again. This is interactive mode.

A binding run while `root` is active MAY also enter a table, which then becomes active. Entering a table other than `root` that has no binding SHALL be an error raised by `gband.keymap.enter`. When `prefix` holds no binding, the prefix key SHALL be handled like any other key. A reload SHALL make `root` the active table.

`gband.keymap.current_table()` SHALL return the name of the active table, and `root` while the configuration loads. Each change of the active table SHALL emit `KeyTableChanged`, as the lua-events capability defines, naming the new and the previous table. A key after which the same mode stays active SHALL emit none.

With no configuration file, the bindings SHALL be those the default configuration makes: Ctrl+Space as the prefix, no binding in `root`, `prefix` declared a mode with the label `navigation`, and these bindings in `prefix`, in this order. A binding SHALL keep navigation mode active unless its row says it returns to interactive mode. A binding whose row gives a description is a Lua function with that description, and every other binding is bound to the action its row names:

| key in navigation mode | Lua binding | action | kind | description of a function |
|---|---|---|---|---|
| `h` | `prefix h` | focus the column to the left | view | |
| `l` | `prefix l` | focus the column to the right | view | |
| `j` | `prefix j` | focus the window below | view | |
| `k` | `prefix k` | focus the window above | view | |
| `u` | `prefix u` | view the band below | view | |
| `i` | `prefix i` | view the band above | view | |
| `c` | `prefix c` | center the focused column, or the focused floating window | view | |
| `n` | `prefix n` | open a window right of the focused window's column, then return to interactive mode | session | `open a window` |
| `q` | `prefix q` | close the focused floating plugin window when one is focused, otherwise the focused window | session | |
| `[` | `prefix [` | consume or expel the focused window to the left | session | |
| `]` | `prefix ]` | consume or expel the focused window to the right | session | |
| `r` | `prefix r` | cycle the width of the focused window's column | session | |
| `f` | `prefix f` | toggle full width of the focused window's column | session | |
| `-` | `prefix -` | shrink the width of the focused window's column | session | |
| `=` | `prefix =` | grow the width of the focused window's column | session | |
| `_` | `prefix _` | shrink the height of the focused window | session | |
| `+` | `prefix +` | grow the height of the focused window | session | |
| `R` | `prefix R` | reset the height of the focused window | session | |
| `v` | `prefix v` | float or tile the focused window | session | |
| `V` | `prefix V` | switch focus between floating and tiled windows | view | |
| Ctrl+H | `prefix ctrl+h` | move the focused window's column, or its floating box, to the left | session | |
| Ctrl+L | `prefix ctrl+l` | move the focused window's column, or its floating box, to the right | session | |
| Ctrl+J | `prefix ctrl+j` | move the focused window, or its floating box, down | session | |
| Ctrl+K | `prefix ctrl+k` | move the focused window, or its floating box, up | session | |
| Ctrl+Left | `prefix ctrl+left` | move the focused window's column, or its floating box, to the left | session | |
| Ctrl+Right | `prefix ctrl+right` | move the focused window's column, or its floating box, to the right | session | |
| Ctrl+Down | `prefix ctrl+down` | move the focused window, or its floating box, down | session | |
| Ctrl+Up | `prefix ctrl+up` | move the focused window, or its floating box, up | session | |
| `?` | `prefix ?` | open the key list, as the key-list capability defines, which returns to interactive mode | client | |
| `D` | `prefix D` | detach | client | |
| Escape | `prefix escape` | return to interactive mode | — | `interactive mode` |
| Enter | `prefix enter` | return to interactive mode | — | `interactive mode` |
| Left | `prefix left` | focus the column to the left | view | |
| Right | `prefix right` | focus the column to the right | view | |
| Down | `prefix down` | focus the window below | view | |
| Up | `prefix up` | focus the window above | view | |
| Ctrl+Space | `prefix prefix` | send the prefix key to the focused window, then return to interactive mode | client | `send the prefix key` |
| any other key | — | discard the key; navigation mode stays active | — | |

A binding to an action SHALL dispatch it. A binding to a Lua function SHALL run the function, as the configuration capability defines. A view action SHALL change this client's view as the layout-view capability defines, and SHALL send nothing to the server, except `center_column` on the floating layer, which sends the placing the layout-view capability defines. A session action SHALL be sent to the server as an action naming the focused window, resolved as the actions capability defines, except close window while a floating plugin window is focused, which closes that floating plugin window in the client. Open window SHALL name the viewed band and the tiled window this client focused most recently there, or no window when there is none. Any other session action, and sending the prefix key, SHALL do nothing when no window is focused, except close window while a floating plugin window is focused. A character key SHALL match a binding by its character, Ctrl and Alt, so a `D` matches whether or not the terminal reports Shift with it.

#### Scenario: Detach
- **WHEN** the user presses Ctrl+Space then Shift+D
- **THEN** the client detaches

#### Scenario: Lowercase d does not detach
- **WHEN** the user presses Ctrl+Space then `d`
- **THEN** the client stays attached, nothing is sent to the window, and navigation mode stays active

#### Scenario: Literal Ctrl+Space
- **WHEN** the user runs `cat -v`, presses Ctrl+Space twice, then presses Enter
- **THEN** the focused window receives `\x00` once
- **AND** `cat -v` prints `^@` on a line of its own

#### Scenario: Literal Ctrl+A
- **WHEN** no `user/init.lua` exists and the user presses Ctrl+A at a bash prompt with text typed
- **THEN** the focused window receives `\x01`
- **AND** bash moves the cursor to the start of the line

#### Scenario: Unbound key after the prefix
- **WHEN** the user presses Ctrl+Space, `x`, then `l` with the first of two columns focused
- **THEN** nothing is sent to either window
- **AND** the second column is focused

#### Scenario: Repeated focus moves
- **WHEN** the viewed band holds four columns with the first focused, and the user presses Ctrl+Space, `l`, `l`, `l`
- **THEN** the fourth column is focused and no window receives a key
- **AND** the status line's mode segment shows `navigation`

#### Scenario: Repeated resize
- **WHEN** the client's 80×24 terminal sets the screen area, the only window sits in a column of width 1/2, and the user presses Ctrl+Space, `=`, `=`, then Escape, waits, and runs `tput cols`
- **THEN** the column's width is 7/10 and the tile is 56 columns wide
- **AND** the window prints `54`, so the keys after Escape reached it

#### Scenario: Arrow keys move focus
- **WHEN** the second of two columns is focused and the user presses Ctrl+Space then Left
- **THEN** the first column is focused and no window receives a key

#### Scenario: Focus back to the left
- **WHEN** the user presses Ctrl+Space, `h`, Escape, then types `echo left` and Enter, after opening a second window
- **THEN** `left` appears in the first tile only
- **AND** the focused window receives no Escape

#### Scenario: Enter returns to interactive mode
- **WHEN** the user presses Ctrl+Space then Enter with one window open
- **THEN** no window opens and the focused window receives nothing
- **AND** `root` is the active table

#### Scenario: Open a window
- **WHEN** one window is focused and the user presses Ctrl+Space then `n`
- **THEN** a second tile with a shell prompt appears right of the first
- **AND** the new window is focused and `root` is active, so `echo $GBAND_WINDOW` runs in it

#### Scenario: Close the focused window
- **WHEN** two windows are open with the second focused and the user presses Ctrl+Space then `q`
- **THEN** the second tile disappears and the first window is focused
- **AND** navigation mode stays active

#### Scenario: Another band
- **WHEN** the user presses Ctrl+Space, `u`, `n`, then Ctrl+Space, `i`
- **THEN** the client shows the first band with its original window focused
- **AND** the second band holds the new window

#### Scenario: Grow the column
- **WHEN** the client's 80×24 terminal sets the screen area, the only window sits in a column of width 1/2, and the user presses Ctrl+Space, `=`, then Escape, waits, and runs `tput cols`
- **THEN** the tile is 48 columns wide
- **AND** the window prints `46`

#### Scenario: Grow the window's height
- **WHEN** the client's 80×24 terminal sets the screen area, a column holds two windows with automatic heights with the top one focused, and the user presses Ctrl+Space then `+`
- **THEN** the top tile is 14 rows high and the bottom tile is 10 rows high

#### Scenario: Center the column
- **WHEN** the client's terminal is 80 columns wide, the client has no bar, the only window sits in a column 40 cells wide at strip position 0, and the user presses Ctrl+Space then `c`
- **THEN** the tile is drawn from the terminal's column 20
- **AND** the 20 columns left of it are drawn empty

#### Scenario: Direct binding acts without the prefix
- **WHEN** `user/init.lua` binds `alt+h` to `gband.action.focus_column_left`, the second of two windows is focused, and the user presses Alt+H
- **THEN** the first window is focused
- **AND** neither window receives the key

#### Scenario: Unbound Alt key reaches the window
- **WHEN** `user/init.lua` binds `alt+h` and the user presses Alt+X
- **THEN** the focused window receives `\x1bx`

#### Scenario: Another prefix key
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua` that then sets `prefix` to `"ctrl+b"`, and the user presses Ctrl+B, `q`, then Escape with two windows open
- **THEN** the focused window closes
- **AND** pressing Ctrl+Space then sends `\x00` to the focused window

#### Scenario: No prefix binding left
- **WHEN** `user/init.lua` makes no prefix binding and the user presses Ctrl+Space then `h`
- **THEN** the focused window receives `\x00` and then `h`

#### Scenario: Prefix table that is not a mode
- **WHEN** `user/init.lua` binds `prefix h` to `gband.action.focus_column_left` and declares no mode, and the user presses Ctrl+Space, `h`, then `h` with the second of two columns focused
- **THEN** the first column is focused
- **AND** the second `h` reaches the focused window

#### Scenario: Named key table
- **WHEN** `user/init.lua` binds `h` and `l` in the table `move`, and binds `prefix m` to a function that calls `gband.keymap.enter("move")`, and the user presses Ctrl+Space, `m`, then `l`, with the first of two columns focused
- **THEN** the second column is focused
- **AND** a further `l` reaches the focused window

#### Scenario: Unbound key in a named table
- **WHEN** the user enters the table `move` and presses `x`
- **THEN** nothing is sent to the window and `root` is active again

#### Scenario: A user mode
- **WHEN** `user/init.lua` declares `resize` a mode, binds `=` in it to `gband.action.grow_column_width` and `escape` to a function that calls `gband.keymap.enter("root")`, binds `alt+r` in `root` to a function that calls `gband.keymap.enter("resize")`, and the user presses Alt+R, `=`, `=`, Escape, then `x`
- **THEN** the focused column grew twice
- **AND** only `x` reaches the focused window

#### Scenario: Key table change events
- **WHEN** a `KeyTableChanged` handler records each payload and the user presses Ctrl+Space, `h`, `l`, then Escape
- **THEN** the handler records `prefix` with previous `root`, then `root` with previous `prefix`, and nothing else

#### Scenario: Current table
- **WHEN** a binding in `prefix` calls `gband.keymap.current_table()`
- **THEN** it returns `prefix`

#### Scenario: Unbound key goes to the focused plugin window
- **WHEN** a floating plugin window with `keys = { j = fn }` is focused and the user presses `j`
- **THEN** `fn` runs and the focused window receives nothing

#### Scenario: Navigation mode before the plugin window
- **WHEN** the default configuration is in use, a floating plugin window with `keys = { x = fn }` is focused, and the user presses Ctrl+Space then `x`
- **THEN** `fn` does not run, nothing reaches the focused window, and navigation mode stays active

#### Scenario: Root binding before the plugin window
- **WHEN** `user/init.lua` binds `alt+h` in `root` to `gband.action.focus_column_left`, a floating plugin window binding `alt+h` in its `keys` is focused, and the user presses Alt+H
- **THEN** the root binding runs and the floating plugin window's function does not

#### Scenario: Prefix q closes the focused floating plugin window
- **WHEN** two windows are open, a floating plugin window is focused, and the user presses Ctrl+Space then `q`
- **THEN** the floating plugin window closes and both tiles stay

#### Scenario: Prefix q closes a floating plugin window on the empty band
- **WHEN** the viewed band holds no window, a floating plugin window is focused, and the user presses Ctrl+Space then `q`
- **THEN** the floating plugin window closes

#### Scenario: Open the key list
- **WHEN** no `user/init.lua` exists and the user presses Ctrl+Space then `?`
- **THEN** a floating plugin window titled `navigation keys` lists the prefix bindings and has focus
- **AND** `root` is active, so a following Down moves the list's cursor line

#### Scenario: Float the focused window
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, the only window sits in a column of width 1/2, and the user presses Ctrl+Space then `v`
- **THEN** the window is drawn in a box spanning columns 20 to 59 and rows 2 to 21, and stays focused

#### Scenario: Switch to the tiled layer and back
- **WHEN** two windows are open, the second floats and is focused, and the user presses Ctrl+Space, `V`, Escape, then types `echo tiled` and Enter
- **THEN** `tiled` appears in the first window only
- **AND** pressing Ctrl+Space then `V` again focuses the floating window

#### Scenario: Move a column with Ctrl+H and Ctrl+Right
- **WHEN** the viewed band holds columns A and B with B focused, and the user presses Ctrl+Space then Ctrl+H
- **THEN** the band holds B and A, in that order, and B stays focused, and neither window receives a key
- **AND** pressing Ctrl+Right next, still in navigation mode, restores A and B

#### Scenario: Move a window with Ctrl+J
- **WHEN** a column holds P1 above P2 with P1 focused, and the user presses Ctrl+Space then Ctrl+J
- **THEN** the column holds P2 above P1, and P1 stays focused

#### Scenario: Move a floating window
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, a floating window is focused with its box starting at column 20, and the user presses Ctrl+Space then Ctrl+L
- **THEN** the box is drawn from column 28
