## MODIFIED Requirements

### Requirement: Present the ribbon
After the handshake, the client SHALL take the terminal full screen in raw mode with bracketed paste enabled. It SHALL keep its own grid of every pane, as the wire-protocol capability defines, and its own view, as the layout-view capability defines. It SHALL draw only the viewed workspace.

Each pane SHALL be drawn in its tile, as the layout capability's tile geometry gives it for the screen area in the latest layout. A tile SHALL be drawn at its strip position less the viewed workspace's camera position, from the terminal's top row. Each tile SHALL show a one-cell border around the pane's grid, which is drawn from its top-left corner. The focused pane's border SHALL be drawn in a style distinct from the other borders.

A pane's grid MAY differ in size from its tile's interior while the server has not yet resized the pane. The border SHALL still follow the tile. A grid larger than the interior SHALL be cut at the interior's right and bottom edges, and interior cells the grid does not cover SHALL be blank.

A tile that crosses the terminal's left, right or bottom edge SHALL be cut at that edge, and the part of the tile inside the terminal SHALL be drawn unchanged. No tile SHALL be resized to fit the terminal. A tile wholly outside the terminal SHALL NOT be drawn, and cells no tile covers SHALL be blank.

The terminal's cursor SHALL sit where the focused pane's cursor is. It SHALL be hidden when the focused pane hides its cursor, when that cell lies outside the terminal or outside the tile's interior, or when no pane is focused.

#### Scenario: Two columns side by side
- **WHEN** the client's 80×24 terminal sets the screen area, and the viewed workspace holds two columns of width 1/2 with the second focused
- **THEN** the first tile fills screen columns 0 to 39 and the second fills 40 to 79, each with a border
- **AND** only the second tile's border has the focused style
- **AND** the cursor sits in the second tile

#### Scenario: Column clipped at the left edge
- **WHEN** the client's 80×24 terminal sets the screen area, the viewed workspace holds two columns of width 2/3, and the client focuses the second
- **THEN** the second tile fills screen columns 27 to 79
- **AND** screen columns 0 to 26 show the rightmost 27 columns of the first tile, without its left border

#### Scenario: Tile larger than the terminal
- **WHEN** the screen area is 120×40 because of another client, this client's terminal is 100×30, and the viewed workspace holds one column of width 1/2
- **THEN** this client draws the top 30 rows of the 60-column tile
- **AND** the tile's bottom border is not drawn

#### Scenario: Empty workspace
- **WHEN** the client views an empty workspace
- **THEN** the screen is blank and the cursor is hidden

#### Scenario: Grid smaller than its tile
- **WHEN** a pane's tile is 50×30 and its grid is still 38×22
- **THEN** the border is drawn around the 50×30 tile
- **AND** the grid fills the top-left 38×22 cells of the interior, and the rest of the interior is blank

#### Scenario: Grid larger than its tile
- **WHEN** a pane's tile is 40×24 and its grid is still 48×28
- **THEN** the interior shows the grid's top-left 38×22 cells
- **AND** the border is drawn unbroken around the 40×24 tile

### Requirement: Key bindings
Ctrl+A SHALL be the prefix key, and the client SHALL NOT send it to the server when it is pressed. The key pressed after the prefix SHALL act as the table below defines. Until a later change makes the bindings configurable, the table SHALL be fixed in the client.

| key after Ctrl+A | action | kind |
|---|---|---|
| `h` | focus the column to the left | view |
| `l` | focus the column to the right | view |
| `j` | focus the pane below | view |
| `k` | focus the pane above | view |
| `u` | view the workspace below | view |
| `i` | view the workspace above | view |
| Enter | open a pane right of the focused pane's column | session |
| `q` | close the focused pane | session |
| `[` | consume or expel the focused pane to the left | session |
| `]` | consume or expel the focused pane to the right | session |
| `r` | cycle the width of the focused pane's column | session |
| `f` | toggle full width of the focused pane's column | session |
| `-` | shrink the width of the focused pane's column | session |
| `=` | grow the width of the focused pane's column | session |
| `_` | shrink the height of the focused pane | session |
| `+` | grow the height of the focused pane | session |
| `R` | reset the height of the focused pane | session |
| `D` | detach | client |
| Ctrl+A | send one Ctrl+A to the focused pane | client |
| any other key | discard both keys | — |

A view action SHALL change this client's view as the layout-view capability defines, and SHALL send nothing to the server. A session action SHALL be sent to the server as an action naming the focused pane. Open pane SHALL name the viewed workspace and the focused pane, or no pane when none is focused. Any other session action, and sending Ctrl+A, SHALL do nothing when no pane is focused. A character key SHALL match a table entry by its character, Ctrl and Alt, so a `D` matches whether or not the terminal reports Shift with it.

#### Scenario: Detach
- **WHEN** the user presses Ctrl+A then Shift+D
- **THEN** the client detaches

#### Scenario: Lowercase d does not detach
- **WHEN** the user presses Ctrl+A then `d`
- **THEN** the client stays attached and nothing is sent to the pane

#### Scenario: Literal Ctrl+A
- **WHEN** the user presses Ctrl+A twice at a bash prompt with text typed
- **THEN** the focused pane receives `\x01` once
- **AND** bash moves the cursor to the start of the line

#### Scenario: Unbound key after the prefix
- **WHEN** the user presses Ctrl+A then `x`
- **THEN** nothing is sent to the pane

#### Scenario: Open a pane
- **WHEN** one pane is focused and the user presses Ctrl+A then Enter
- **THEN** a second tile with a shell prompt appears right of the first
- **AND** the new pane is focused, so `echo $GBAND_PANE` runs in it

#### Scenario: Focus back to the left
- **WHEN** the user has opened a second pane and presses Ctrl+A then `h`, then types `echo left` and Enter
- **THEN** `left` appears in the first tile only

#### Scenario: Close the focused pane
- **WHEN** two panes are open with the second focused and the user presses Ctrl+A then `q`
- **THEN** the second tile disappears and the first pane is focused

#### Scenario: Another workspace
- **WHEN** the user presses Ctrl+A then `u`, then Ctrl+A then Enter, then Ctrl+A then `i`
- **THEN** the client shows the first workspace with its original pane focused
- **AND** the second workspace holds the new pane

#### Scenario: Grow the column
- **WHEN** the client's 80×24 terminal sets the screen area, the only pane sits in a column of width 1/2, and the user presses Ctrl+A then `=`, waits, and runs `tput cols`
- **THEN** the tile is 48 columns wide
- **AND** the pane prints `46`

#### Scenario: Grow the pane's height
- **WHEN** the client's 80×24 terminal sets the screen area, a column holds two panes with automatic heights with the top one focused, and the user presses Ctrl+A then `+`
- **THEN** the top tile is 14 rows high and the bottom tile is 10 rows high

## ADDED Requirements

### Requirement: Report shown panes
The client SHALL send the server a shown message naming its shown panes, as the layout-view capability defines them, once it has received the first layout after attaching. It SHALL send a new shown message whenever its shown panes change, whether a layout, a focus message, a view action or a change of its terminal's size changed them. It SHALL NOT send a shown message that names the same panes as the last one it sent.

#### Scenario: Shown after attach
- **WHEN** a client with an 80×24 terminal attaches to a session whose first workspace holds three columns of width 1/2
- **THEN** the client's first shown message names the panes of the first two columns

#### Scenario: Shown follows the camera
- **WHEN** that client then focuses the third column
- **THEN** it sends a shown message naming the panes of the second and third columns

#### Scenario: No repeated report
- **WHEN** the client focuses the pane below within a column whose panes are all shown
- **THEN** it sends no shown message
