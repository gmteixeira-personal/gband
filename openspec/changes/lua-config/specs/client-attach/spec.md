## MODIFIED Requirements

### Requirement: Key bindings
The client SHALL take its key bindings and its prefix key from the configuration, as the configuration capability defines them. Outside a prefix sequence, a key that a direct binding names SHALL run that binding, and the prefix key SHALL start a prefix sequence. The client SHALL send neither to the server. Any other key outside a prefix sequence SHALL be sent to the focused pane. The key pressed next in a prefix sequence SHALL end it: a key that a prefix binding names SHALL run that binding, and any other key SHALL discard both keys. When no prefix binding exists, the prefix key SHALL be sent to the focused pane like any other key.

With no configuration file, the bindings SHALL be those `defaults.lua` makes: Ctrl+A as the prefix, no direct binding, and these prefix bindings:

| key after Ctrl+A | Lua binding | action | kind |
|---|---|---|---|
| `h` | `prefix h` | focus the column to the left | view |
| `l` | `prefix l` | focus the column to the right | view |
| `j` | `prefix j` | focus the pane below | view |
| `k` | `prefix k` | focus the pane above | view |
| `u` | `prefix u` | view the workspace below | view |
| `i` | `prefix i` | view the workspace above | view |
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
| `D` | `prefix D` | detach | client |
| Ctrl+A | `prefix prefix` | send the prefix key to the focused pane | client |
| any other key | — | discard both keys | — |

A binding to an action SHALL dispatch it. A binding to a Lua function SHALL run the function, as the configuration capability defines. A view action SHALL change this client's view as the layout-view capability defines, and SHALL send nothing to the server. A session action SHALL be sent to the server as an action naming the focused pane. Open pane SHALL name the viewed workspace and the focused pane, or no pane when none is focused. Any other session action, and sending the prefix key, SHALL do nothing when no pane is focused. A character key SHALL match a binding by its character, Ctrl and Alt, so a `D` matches whether or not the terminal reports Shift with it.

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

#### Scenario: Direct binding acts without the prefix
- **WHEN** `init.lua` binds `alt+h` to `gband.action.focus_column_left`, the second of two panes is focused, and the user presses Alt+H
- **THEN** the first pane is focused
- **AND** neither pane receives the key

#### Scenario: Unbound Alt key reaches the pane
- **WHEN** `init.lua` binds `alt+h` and the user presses Alt+X
- **THEN** the focused pane receives `\x1bx`

#### Scenario: Another prefix key
- **WHEN** `init.lua` sets `prefix` to `"ctrl+b"` and the user presses Ctrl+B then `q` with two panes open
- **THEN** the focused pane closes
- **AND** pressing Ctrl+A sends `\x01` to the focused pane

#### Scenario: No prefix binding left
- **WHEN** `init.lua` unbinds every prefix binding and the user presses Ctrl+A then `h`
- **THEN** the focused pane receives `\x01` and then `h`

### Requirement: Present the ribbon
After the handshake, the client SHALL take the terminal full screen in raw mode with bracketed paste enabled. It SHALL keep its own grid of every pane, as the wire-protocol capability defines, and its own view, as the layout-view capability defines. It SHALL draw only the viewed workspace, except during a workspace switch, when it SHALL draw the workspaces the animations capability places on screen. While the configuration capability shows a configuration error, the client SHALL draw that error over the bottom row, after the tiles.

Each pane SHALL be drawn in its tile, as the layout capability's tile geometry gives it for the screen area in the latest layout. A tile SHALL be drawn at its strip position less the viewed workspace's camera position, from the terminal's top row. While an animation runs, the tile's position and size, the camera and the workspace's top row SHALL be the drawn values the animations capability defines. At rest they equal the values above. Each tile SHALL show a one-cell border around the pane's grid, which is drawn from its top-left corner. The focused pane's border SHALL be drawn in a style distinct from the other borders.

A pane's grid MAY differ in size from its tile's interior while the server has not yet resized the pane. The border SHALL still follow the tile, at its drawn size while an animation runs. A grid larger than the interior SHALL be cut at the interior's right and bottom edges, and interior cells the grid does not cover SHALL be blank.

A tile that crosses the terminal's left, right or bottom edge SHALL be cut at that edge, and the part of the tile inside the terminal SHALL be drawn unchanged, except where a configuration error covers the bottom row. No tile SHALL be resized to fit the terminal. A tile wholly outside the terminal SHALL NOT be drawn, and cells no tile or configuration error covers SHALL be blank.

The terminal's cursor SHALL sit where the focused pane's cursor is. It SHALL be hidden when the focused pane hides its cursor, when that cell lies outside the terminal or outside the tile's interior, when no pane is focused, or while the animations capability hides it during motion.

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

#### Scenario: Mid-scroll frame
- **WHEN** the camera is drawn at 20 while it scrolls toward 40, and the viewed workspace holds three columns of 40 cells
- **THEN** the second tile fills screen columns 20 to 59
- **AND** the cursor is hidden

#### Scenario: Configuration error over the ribbon
- **WHEN** the client's 40×6 terminal sets the screen area, the viewed workspace holds one column of width 1/2, and the client shows a configuration error
- **THEN** the bottom row shows the error, cut at the terminal's width
- **AND** the rows above it show the tile unchanged
