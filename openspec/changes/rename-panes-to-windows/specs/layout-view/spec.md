## RENAMED Requirements

- FROM: `### Requirement: Shown panes`
- TO: `### Requirement: Shown windows`

- FROM: `### Requirement: Focus a named pane`
- TO: `### Requirement: Focus a named window`

## MODIFIED Requirements

### Requirement: Per-client view
Each attached client SHALL hold its own view: the viewed band, the focused window, an active layer for each band as the floating-windows capability defines, and a camera position for each band. The focused window SHALL be in the viewed band, tiled or floating, and there SHALL be no focused window while the viewed band is empty. View actions SHALL change only the view of the client that performs them, and SHALL NOT change the layout or the view of any other client.

#### Scenario: Two clients focus independently
- **WHEN** two clients view a band with columns A and B, both focus a window in B, and the first client focuses the column to the left
- **THEN** the first client focuses a window in A
- **AND** the second client still focuses the window in B

#### Scenario: Two clients in different layers
- **WHEN** two clients view a band with column A and floating window P3, both focus a window in A, and the first switches layers
- **THEN** the first client focuses P3
- **AND** the second client still focuses the window in A

### Requirement: Initial view
A client SHALL start by viewing the first band, with focus on the top window of its first column and every camera at strip position 0.

#### Scenario: Attach to a session
- **WHEN** a client attaches to a session whose first band holds columns A and B
- **THEN** the client views the first band and focuses the top window of A

### Requirement: Focus across columns
While the tiled layer is active, focusing the column to the left or right SHALL move focus to the adjacent column in that direction. Within that column, focus SHALL go to the window this client focused most recently, or to the top window when this client has focused none of its windows. When no column exists in that direction, the view SHALL not change. While the floating layer is active, these actions SHALL move focus between floating windows, as the floating-windows capability defines.

#### Scenario: Return to the remembered pane
- **WHEN** column A holds P1 and P2, the client focuses P2, focuses the column to the right, then the column to the left
- **THEN** the client focuses P2

#### Scenario: No column to the right
- **WHEN** the client focuses a window in the last column and focuses the column to the right
- **THEN** the focus is unchanged

#### Scenario: Floating layer stays in its layer
- **WHEN** the floating layer is active on the band's only floating window, and the band holds columns A and B
- **THEN** focusing the column to the left or right leaves the view unchanged

### Requirement: Focus within a column
While the tiled layer is active, focusing the window below or above SHALL move focus to the adjacent window in that direction in the focused column. When no window exists in that direction, the view SHALL not change. While the floating layer is active, these actions SHALL move focus between floating windows, as the floating-windows capability defines.

#### Scenario: Move down a stack
- **WHEN** a column holds P1, P2 and P3, and the client focuses P1 and then the window below twice
- **THEN** the client focuses P3

#### Scenario: Bottom of the stack
- **WHEN** the client focuses the bottom window of a column and focuses the window below
- **THEN** the focus is unchanged

### Requirement: Switch band
Viewing the band below or above SHALL make the adjacent band in that direction the viewed band. Focus SHALL go to the window this client last focused in that band when it is still there, in the layer that window is in. Otherwise focus SHALL go to the top window of its first column, or, when the band holds no column, to the top floating window of this client's stacking order, or to no window when the band is empty. When no band exists in that direction, the view SHALL not change.

#### Scenario: Down to the empty band
- **WHEN** the layout holds B1 with windows and an empty B2, and a client viewing B1 views the band below
- **THEN** the client views B2 with no focused window

#### Scenario: Back up restores focus
- **WHEN** a client focuses the second column of B1, views the band below, then the band above
- **THEN** the client focuses the same window in the second column of B1

#### Scenario: Back up restores a floating focus
- **WHEN** a client focuses floating window P3 of B1, views the band below, then the band above
- **THEN** the client focuses P3 with the floating layer of B1 active

#### Scenario: Top band
- **WHEN** a client viewing the first band views the band above
- **THEN** the view is unchanged

### Requirement: Follow layout changes
When the layout changes, each client's view SHALL follow it:

- When the viewed band is removed, the client SHALL view the band that now holds the removed one's position, or the last band when none does.
- When the focused window moves within the viewed band, between columns, within a column or between layers, focus SHALL stay on that window.
- When the focused tiled window leaves the layout, focus SHALL go to the column at the same position, or to the last column when none is there. Within that column, focus SHALL go to the window at the same position from the top, or to the bottom window when none is there.
- When the focused floating window leaves the layout, focus SHALL go where the floating-windows capability sends it.
- When the server tells the client to focus a window, the client SHALL view that window's band and focus it.

#### Scenario: Focused column closed
- **WHEN** a client focuses column B in a band holding A, B and C, and B's only window exits
- **THEN** the client focuses the window in C

#### Scenario: Last column closed
- **WHEN** a client focuses column C, the last of A, B and C, and C's only window exits
- **THEN** the client focuses the window in B

#### Scenario: Focused pane expelled
- **WHEN** a client focuses P2 in a column holding P1 and P2, and P2 is expelled to the right
- **THEN** the client still focuses P2, now alone in its new column

#### Scenario: Focused column moved
- **WHEN** a client focuses a window of column A in a band holding A and B, and A is moved right
- **THEN** the client still focuses that window, now in the second column

#### Scenario: Viewed band removed
- **WHEN** a client views B1 of B1, B2 and an empty B3, and the last window of B1 exits
- **THEN** the client views B2

### Requirement: Camera
Each band's camera SHALL be the strip position shown at the client terminal's left edge. A camera below 0 SHALL be allowed, and strip positions left of 0 SHALL be drawn empty. After every change of focus, of the layout or of the client's terminal width, the camera of the viewed band SHALL move by the configuration's `center_focused_column` policy, where the view spans from the camera to the camera plus the terminal's width. While the floating layer is active, the camera SHALL move as though the tiled window this client focused most recently in the band were focused, and SHALL NOT move when there is none.

Under `"never"`, the default:

- When the focused window's column lies wholly inside the view, the camera SHALL not move.
- Otherwise, when the column is at least as wide as the terminal or starts left of the view, the camera SHALL move to the column's start.
- Otherwise, the camera SHALL move so that the column's end meets the terminal's right edge.

Under `"always"`, the camera SHALL centre the focused column: it SHALL move to the column's start less half the difference between the terminal's width and the column's width, rounded down. When the column is at least as wide as the terminal, the camera SHALL move to the column's start.

Under `"on-overflow"`, when focus moves from one column to another column C of the viewed band and the previously focused column is still in it, let S be the column next to C on the side the focus came from. When the span from the start of the leftmost of S and C to the end of the rightmost is no wider than the terminal, the camera SHALL move as under `"never"`. Otherwise it SHALL move as under `"always"`. Every other change SHALL move the camera as under `"never"`.

#### Scenario: Scroll right just enough
- **WHEN** the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the camera is at 0 and focus moves from the second column to the third
- **THEN** the camera moves to 40

#### Scenario: Scroll back left
- **WHEN** the policy is `"never"`, the camera is at 40 in that band and focus moves to the first column
- **THEN** the camera moves to 0

#### Scenario: Visible column keeps the camera
- **WHEN** the policy is `"never"`, the camera is at 40 in that band and focus moves from the third column to the second
- **THEN** the camera stays at 40

#### Scenario: Column wider than the terminal
- **WHEN** the policy is `"never"`, the focused column starts at strip position 40 and is wider than the terminal
- **THEN** the camera moves to 40

#### Scenario: Always centre
- **WHEN** the policy is `"always"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, and focus moves to the second column
- **THEN** the camera moves to 20

#### Scenario: Always centre the first column
- **WHEN** the policy is `"always"`, the terminal is 80 columns wide and the focused column is 40 cells wide at strip position 0
- **THEN** the camera is at -20 and the 20 cells left of the column are drawn empty

#### Scenario: On overflow, the pair fits
- **WHEN** the policy is `"on-overflow"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the camera is at 0 and focus moves from the second column to the third
- **THEN** the camera moves to 40

#### Scenario: On overflow, the pair does not fit
- **WHEN** the policy is `"on-overflow"`, the terminal is 80 columns wide, a band holds three columns of 53 cells at strip positions 0, 53 and 106, the camera is at 0 and focus moves from the first column to the second
- **THEN** the camera moves to 40

#### Scenario: On overflow after a width change
- **WHEN** the policy is `"on-overflow"`, the terminal is 80 columns wide, a band holds two columns of 40 cells at strip positions 0 and 40, the second is focused, the camera is at 0, and the second column grows to 48 cells
- **THEN** the camera moves to 8

#### Scenario: Floating focus keeps the camera
- **WHEN** the policy is `"never"`, the camera is at 40, the client last focused the third of three 40-cell columns, and it switches to the floating layer
- **THEN** the camera stays at 40

### Requirement: Shown windows
A view's shown windows SHALL be the windows of its viewed band whose tile, as the layout capability's tile geometry gives it for the screen area, or whose box, as the floating-windows capability places it, has at least one cell inside the client's terminal. A tile's cells SHALL be placed at its strip position less the viewed band's camera position, from the terminal's top row. A box's cells SHALL be placed at its column and row, from the terminal's top-left cell, whatever the camera. A tile that a box covers SHALL still be shown. A view of an empty band SHALL show no window.

#### Scenario: Column beyond the right edge
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds three columns of width 1/2, and the camera is at strip position 0
- **THEN** the shown windows are those of the first two columns

#### Scenario: Partly visible column
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds two columns of width 2/3, and the client focuses the second
- **THEN** the windows of both columns are shown

#### Scenario: Pane below the bottom edge
- **WHEN** the screen area is 120×60, the client's terminal is 100×30, and the viewed band holds one column of width 1/2 with two windows with automatic heights of weight 1
- **THEN** only the top window is shown

#### Scenario: Other bands
- **WHEN** the layout holds B1 with window P1 and B2 with window P2, and the client views B2
- **THEN** the only shown window is P2

#### Scenario: Floating pane shown
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds three columns of width 1/2 and floating window P4, and the camera is at 40
- **THEN** the shown windows are those of the second and third columns, and P4

#### Scenario: Box below a small terminal
- **WHEN** the screen area is 120×60, the client's terminal is 100×30, and the viewed band's floating window P4 has its box at row 40
- **THEN** P4 is not shown

### Requirement: Focus a named window
Focusing a named window SHALL make that window's band the viewed band, focus that window, and make its layer active, as a focus message from the server does. The camera SHALL follow the new focus as it follows any change of focus. Focusing a window not in the layout SHALL leave the view unchanged.

#### Scenario: Pane in the viewed band
- **WHEN** a client views a band holding columns A, B and C, focuses A, and focuses the named window in C
- **THEN** the client focuses that window
- **AND** focusing the column to the left then focuses B

#### Scenario: Pane in another band
- **WHEN** a client views B1 and focuses a named window in B2
- **THEN** the client views B2 with that window focused

#### Scenario: Floating pane by name
- **WHEN** a client focuses a tiled window of B1 and focuses floating window P3 of B1 by name
- **THEN** the client focuses P3 with the floating layer active

### Requirement: View a named band
Viewing a named band SHALL make it the viewed band. Focus SHALL go where "Switch band" sends it. Viewing the band already viewed, or a band not in the layout, SHALL leave the view unchanged.

#### Scenario: Jump two bands down
- **WHEN** the layout holds B1, B2 and an empty B3, and a client viewing B1 views B3 by name
- **THEN** the client views B3 with no focused window

#### Scenario: Remembered focus
- **WHEN** a client focuses the second column of B2, views B1, and then views B2 by name
- **THEN** the client focuses the same window in the second column of B2
