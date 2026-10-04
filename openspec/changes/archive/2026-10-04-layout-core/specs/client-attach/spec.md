## ADDED Requirements

### Requirement: Present the ribbon
After the handshake, the client SHALL take the terminal full screen in raw mode with bracketed paste enabled. It SHALL keep its own grid of every pane, as the wire-protocol capability defines, and its own view, as the layout-view capability defines. It SHALL draw only the viewed workspace.

Each pane SHALL be drawn in its tile, as the layout capability's tile geometry gives it for the screen area in the latest layout. A tile SHALL be drawn at its strip position less the viewed workspace's camera position, from the terminal's top row. Each tile SHALL show a one-cell border around the pane's grid, which is drawn from its top-left corner. The focused pane's border SHALL be drawn in a style distinct from the other borders.

A tile that crosses the terminal's left, right or bottom edge SHALL be cut at that edge, and the part of the tile inside the terminal SHALL be drawn unchanged. No tile SHALL be resized to fit the terminal. A tile wholly outside the terminal SHALL NOT be drawn, and cells no tile covers SHALL be blank.

The terminal's cursor SHALL sit where the focused pane's cursor is. It SHALL be hidden when the focused pane hides its cursor, when that cell lies outside the terminal, or when no pane is focused.

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

## MODIFIED Requirements

### Requirement: Send input
The client SHALL send each key press and repeat that the key bindings do not consume to the server as a key naming the focused pane, each paste as a paste naming the focused pane, and each change of its terminal's size as a resize. Keys and pastes SHALL be dropped while no pane is focused. Keys the input-encoding capability cannot represent SHALL be dropped. On attach, the client SHALL report its terminal size.

#### Scenario: Typing runs a command
- **WHEN** the user types `echo hi` and Enter
- **THEN** the focused pane prints `hi`

#### Scenario: Typing reaches only the focused pane
- **WHEN** two panes are open with the second focused and the user types `echo hi` and Enter
- **THEN** the second pane prints `hi`
- **AND** the first pane's screen is unchanged

#### Scenario: Resize reaches the program
- **WHEN** the only pane sits in a column of width 1/2, the user resizes the terminal to 70 columns and runs `tput cols`
- **THEN** the pane prints `33`

### Requirement: Leaving the client
Whenever the client exits after taking the terminal, it SHALL first leave the alternate screen, disable raw mode and bracketed paste, and show the cursor. It SHALL then print one line to standard output, followed by the note for a server from a different build when one applies, and exit as follows:

| cause | line printed | exit status |
|---|---|---|
| the user detached | `[detached]` | 0 |
| the session ended | `[exited]` | 0 |
| the connection closed without the server saying why | `[lost server]` | 1 |

#### Scenario: Detach and reattach
- **WHEN** the user runs `sleep 100`, detaches with Ctrl+A then `D`, and runs `gband attach` again
- **THEN** the first client printed `[detached]` and exited with status 0
- **AND** the new client shows `sleep 100` still running

#### Scenario: Shell exits
- **WHEN** the user runs `exit` in the only pane
- **THEN** the client restores the terminal, prints `[exited]` and exits with status 0

#### Scenario: Server killed
- **WHEN** the server is killed with SIGKILL while a client is attached
- **THEN** the client restores the terminal, prints `[lost server]` and exits with status 1

## REMOVED Requirements

### Requirement: Present the pane
**Reason**: The client now draws a workspace of several panes in bordered tiles, not one pane full screen.
**Migration**: See "Present the ribbon".

### Requirement: Prefix key
**Reason**: The two-entry prefix table grows into the key-binding table, and detach moves from `d` to `D`.
**Migration**: See "Key bindings". Press Ctrl+A then `D` to detach.
