## MODIFIED Requirements

### Requirement: Session actions
The server SHALL own the session's layout, as the layout capability defines it, and change it only through session actions. It SHALL apply the session actions of every client, and those the server's Lua calls as the server-runtime capability defines, in the order it receives them, and send the resulting layout to every attached client. An action that names a window or a band no longer in the layout SHALL be ignored. The session actions SHALL be:

| action | effect |
|---|---|
| open window | start a window program, as "Window program" defines, or open a drawn window, as "Drawn windows" defines, and place the window as the layout capability's "Open a window" defines, with the column width the action names, tiled or floating as the action names |
| close window | close the named window, as "Close a window" defines |
| consume or expel | as the layout capability defines, left or right |
| move column | move the named window's column, or its floating box, left or right, as the layout capability defines |
| move window | move the named window, or its floating box, down or up, as the layout capability defines |
| toggle floating | float the named tiled window, or tile the named floating window after the tiled window the action names, as the floating-windows capability defines; when the action names a layer, move the window only when it is in the other layer |
| set position | place the named floating window at the column and row the action names, as the floating-windows capability defines |
| move to place | move the named tiled window to the place beside or inside the reference window's column that the action names, as the layout capability's "Move a window to a place" defines |
| cycle width | cycle the width of the named window's column or floating box |
| toggle full width | toggle full width of the named window's column or floating box |
| grow width | grow the width of the named window's column or floating box |
| shrink width | shrink the width of the named window's column or floating box |
| grow height | grow the height of the named window |
| shrink height | shrink the height of the named window |
| reset height | reset the height of the named window |
| set width | set the width of the named window's column or floating box to the width the action names |
| set height | set the height of the named window to the rows or the weight the action names |

Set position naming a tiled window SHALL leave the layout unchanged.

Toggle floating SHALL name either no layer, the floating layer or the tiled layer. Naming no layer SHALL toggle the window. Naming the floating layer SHALL be a request to float the window, and naming the tiled layer a request to tile it, as the floating-windows capability's "Float a window" and "Tile a window" define. The server SHALL apply each request to the layout it holds when the request's turn comes, not to the layout the sender held. Requests from several clients and from the server's Lua for one window SHALL therefore leave the window in the layer the last of them names.

After placing an opened window whose action asks for focus, the server SHALL send the client that asked for it, after the layout that holds the window, a message telling it to focus that window. The server SHALL send that message only while the latest layout it has sent that client holds the window. When the window has left the layout before the message is sent, as when its program exits at once, the server SHALL send no focus message for it, so a client is never told to focus a window that the last layout it received does not hold. A window opened by the server's Lua SHALL NOT change any client's focus. When the program of a new window cannot be started, the server SHALL record the reason in its log and leave the layout unchanged.

#### Scenario: Open a window
- **WHEN** two clients are attached and the first asks to open a window next to the window it focuses
- **THEN** both clients receive a layout holding both windows
- **AND** only the first client is told to focus the new window

#### Scenario: Concurrent actions
- **WHEN** two clients each ask to open a window at the same moment
- **THEN** the layout holds three windows, and every client receives the same layout

#### Scenario: Action on a closed window
- **WHEN** a client asks to cycle the width of a window that has already left the layout
- **THEN** the layout is unchanged

#### Scenario: Grow a window's height
- **WHEN** the screen area is 80×24 and a client asks to grow the height of the top window of a column holding two windows with automatic heights of weight 1
- **THEN** every attached client receives a layout in which the top window has a fixed height of 14 rows and the bottom window an automatic height of weight 1

#### Scenario: Open without focus
- **WHEN** a client asks to open a window and the action does not ask for focus
- **THEN** every client receives a layout holding the new window
- **AND** no client is told to focus it

#### Scenario: Open with a width
- **WHEN** a client asks to open a window with the column width 1/4
- **THEN** every client receives a layout whose new column has width 1/4

#### Scenario: Set a column's width
- **WHEN** a client asks to set the width of window 1's column to 2/5
- **THEN** every attached client receives a layout in which window 1's column has width 2/5 and full width off

#### Scenario: Action from the server's Lua
- **WHEN** a server handler of `WindowOpened` calls `gband.action.grow_column_width` for the new window
- **THEN** every attached client receives a layout in which that window's column is wider than the default width

#### Scenario: Float a window for every client
- **WHEN** two clients are attached and the first asks to toggle floating on tiled window 2
- **THEN** both clients receive a layout in which window 2 is in its band's floating list

#### Scenario: Two clients float the same window
- **WHEN** the screen area is 80×24, the default column width is 1/2, two clients are attached, window 3 is alone in a column of band 1, and each client sends toggle floating naming window 3 and the floating layer
- **THEN** both clients receive a layout in which band 1's floating list holds window 3 once, with width 1/2, `rows` 20, `col` 20 and `row` 2
- **AND** no column of band 1 holds window 3

#### Scenario: Two clients toggle the same window
- **WHEN** two clients are attached, window 3 is alone in a column of band 1, and each client sends toggle floating naming window 3 and no layer
- **THEN** both clients receive a layout in which window 3 is in a column of band 1 and band 1 holds no floating window

#### Scenario: Last request wins
- **WHEN** two clients are attached, window 3 is in band 1's floating list, the first client sends toggle floating naming window 3 and the tiled layer, and the second client then sends toggle floating naming window 3 and the floating layer
- **THEN** both clients receive a layout in which window 3 is in band 1's floating list

#### Scenario: Open a floating window with focus
- **WHEN** a client asks to open a floating window that asks for focus
- **THEN** every client receives a layout whose band's floating list ends with the new window
- **AND** only that client is told to focus it

#### Scenario: Set position on a tiled window
- **WHEN** a client asks to set the position of tiled window 1
- **THEN** the layout is unchanged

#### Scenario: Move to place reaches every client
- **WHEN** two clients view a band holding columns A, B and C, and the first sends move to place naming A's window, B's window as the reference, and right of its column
- **THEN** both clients receive a layout holding B, A and C, in that order

#### Scenario: Window closed before its focus
- **WHEN** a client asks to open a window running `printf` with focus, and the program exits before the server sends the focus message
- **THEN** every focus message the client receives names a window that the last layout it received holds
- **AND** the client receives no focus message naming the closed window after a layout that does not hold it

