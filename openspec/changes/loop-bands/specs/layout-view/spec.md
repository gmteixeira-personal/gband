## MODIFIED Requirements

### Requirement: Focus across columns
While the tiled layer is active, focusing the column to the left or right SHALL move focus to the adjacent column in that direction. Within that column, focus SHALL go to the window this client focused most recently, or to the top window when this client has focused none of its windows. When no column exists in that direction and the configuration's `loop_bands` is on, focus SHALL go round the band: right of the last column to the first column, and left of the first column to the last. It SHALL do so whether or not the band's strip loops, as the Camera requirement defines. When no column exists in that direction and `loop_bands` is off, or the band holds one column, the view SHALL not change. While the floating layer is active, these actions SHALL move focus between floating windows, as the floating-windows capability defines.

#### Scenario: Return to the remembered window
- **WHEN** column A holds P1 and P2, the client focuses P2, focuses the column to the right, then the column to the left
- **THEN** the client focuses P2

#### Scenario: Right of the last column
- **WHEN** `loop_bands` is on, a band holds columns A, B and C, and the client focuses a window in C and focuses the column to the right
- **THEN** the client focuses a window in A

#### Scenario: Left of the first column
- **WHEN** `loop_bands` is on, a band holds columns A, B and C, A holds P1 and P2 and C holds P3 and P4, the client has focused P4 most recently in C, and it focuses P1 and then the column to the left
- **THEN** the client focuses P4

#### Scenario: Short band still goes round
- **WHEN** `loop_bands` is on, the terminal is 80 columns wide, a band holds two columns of 40 cells, and the client focuses the second and then the column to the right
- **THEN** the client focuses a window in the first column

#### Scenario: One column
- **WHEN** `loop_bands` is on, a band holds one column, and the client focuses the column to the right
- **THEN** the view is unchanged

#### Scenario: No column to the right
- **WHEN** `loop_bands` is off, and the client focuses a window in the last column and focuses the column to the right
- **THEN** the focus is unchanged

#### Scenario: Floating layer stays in its layer
- **WHEN** the floating layer is active on the band's only floating window, and the band holds columns A and B
- **THEN** focusing the column to the left or right leaves the view unchanged

### Requirement: Camera
Each band's camera SHALL be the strip position shown at the client terminal's left edge. A camera below 0 SHALL be allowed, and strip positions left of 0 SHALL be drawn empty while the band's strip does not loop. After every change of focus, of the layout or of the client's terminal width, the camera of the viewed band SHALL move by the configuration's `center_focused_column` policy, where the view spans from the camera to the camera plus the terminal's width. While the floating layer is active, the camera SHALL move as though the tiled window this client focused most recently in the band were focused, and SHALL NOT move when there is none.

A band's strip SHALL loop while the configuration's `loop_bands` is on and the strip's width, the end of its last column, less the width of its widest column is at least the terminal's width. A strip that does not loop SHALL be drawn once, so no window is ever drawn twice. While a strip loops:

- Each column SHALL also stand at its strip position plus every whole multiple of the strip's width. Each of these positions is a copy of the column. The last column's copy ends at strip position 0 and the first column's copy starts at the strip's width, so the strip closes into a loop.
- A tile's drawn copy SHALL be the copy of its column that has at least one cell inside the view. The strip's length ensures that a tile has at most one.
- The policies below SHALL place the focused column at one of its copies. When focus moved to the column to the left or right, it SHALL be the copy next to the previously focused column's drawn copy, on the side of the move. Otherwise, or when that column has no drawn copy, it SHALL be the copy that needs the smallest camera move, the one with the smaller start on a tie.
- After every move, the camera SHALL be held at least 0 and below the strip's width, by adding or subtracting a whole multiple of the strip's width. Holding it SHALL NOT change what the view shows.

Under `"never"`, the default:

- When the focused window's column lies wholly inside the view, the camera SHALL not move.
- Otherwise, when the column is at least as wide as the terminal or starts left of the view, the camera SHALL move to the column's start.
- Otherwise, the camera SHALL move so that the column's end meets the terminal's right edge.

Under `"always"`, the camera SHALL centre the focused column: it SHALL move to the column's start less half the difference between the terminal's width and the column's width, rounded down. When the column is at least as wide as the terminal, the camera SHALL move to the column's start.

Under `"on-overflow"`, when focus moves from one column to another column C of the viewed band and the previously focused column is still in it, let S be the column next to C on the side the focus came from. While the strip loops, S and C SHALL be the copies next to each other, and the side the focus came from SHALL be the side opposite the move when focus moved left or right. When the span from the start of the leftmost of S and C to the end of the rightmost is no wider than the terminal, the camera SHALL move as under `"never"`. Otherwise it SHALL move as under `"always"`. Every other change SHALL move the camera as under `"never"`.

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
- **WHEN** the policy is `"always"`, the terminal is 80 columns wide and the band holds only the focused column, 40 cells wide at strip position 0
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

#### Scenario: Scroll right across the seam
- **WHEN** `loop_bands` is on, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused with the camera at 40, and the client focuses the column to the right
- **THEN** the client focuses the first column and the camera moves to 80
- **AND** the terminal shows the third column in cells 0 to 39 and the first column in cells 40 to 79

#### Scenario: Keep going right
- **WHEN** the client then focuses the column to the right again
- **THEN** the client focuses the second column and the camera moves to 120, which is held at 0

#### Scenario: Scroll left across the seam
- **WHEN** `loop_bands` is on, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the first is focused with the camera at 0, and the client focuses the column to the left
- **THEN** the client focuses the third column and the camera moves to -40, which is held at 80
- **AND** the terminal shows the third column in cells 0 to 39 and the first column in cells 40 to 79

#### Scenario: Always centre across the seam
- **WHEN** `loop_bands` is on, the policy is `"always"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused, and the client focuses the column to the right
- **THEN** the camera moves to 100
- **AND** the terminal shows the last 20 cells of the third column, then the first column, then the first 20 cells of the second column

#### Scenario: Short strip does not loop
- **WHEN** `loop_bands` is on, the policy is `"always"`, the terminal is 80 columns wide, a band holds two columns of 40 cells at strip positions 0 and 40, and focus moves from the second column to the first by the column to the right
- **THEN** the camera moves to -20 and the 20 cells left of the first column are drawn empty
- **AND** each window is drawn once

#### Scenario: Looping off
- **WHEN** `loop_bands` is off, the policy is `"always"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, and the first column is focused
- **THEN** the camera is at -20 and the 20 cells left of the first column are drawn empty

### Requirement: Shown windows
A view's shown windows SHALL be the windows of its viewed band whose tile, as the layout capability's tile geometry gives it for the screen area, or whose box, as the floating-windows capability places it, has at least one cell inside the client's terminal. A tile's cells SHALL be placed at its strip position less the viewed band's camera position, from the terminal's top row. While the band's strip loops, as the Camera requirement defines, a tile's cells SHALL be placed at its drawn copy's position instead, and a tile with no drawn copy SHALL NOT be shown. Wherever the client-attach and animations capabilities draw a tile at its strip position less a camera, they SHALL use its drawn copy while the strip loops. A box's cells SHALL be placed at its column and row, from the terminal's top-left cell, whatever the camera. A tile that a box covers SHALL still be shown. A view of an empty band SHALL show no window.

#### Scenario: Column beyond the right edge
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds three columns of width 1/2, and the camera is at strip position 0
- **THEN** the shown windows are those of the first two columns

#### Scenario: Partly visible column
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds two columns of width 2/3, and the client focuses the second
- **THEN** the windows of both columns are shown

#### Scenario: Window below the bottom edge
- **WHEN** the screen area is 120×60, the client's terminal is 100×30, and the viewed band holds one column of width 1/2 with two windows with automatic heights of weight 1
- **THEN** only the top window is shown

#### Scenario: Other bands
- **WHEN** the layout holds B1 with window P1 and B2 with window P2, and the client views B2
- **THEN** the only shown window is P2

#### Scenario: Floating window shown
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds three columns of width 1/2 and floating window P4, and the camera is at 40
- **THEN** the shown windows are those of the second and third columns, and P4

#### Scenario: Box below a small terminal
- **WHEN** the screen area is 120×60, the client's terminal is 100×30, and the viewed band's floating window P4 has its box at row 40
- **THEN** P4 is not shown

#### Scenario: First column shown after the last
- **WHEN** `loop_bands` is on, the screen area and the client's terminal are both 80×24, the viewed band holds three columns of width 1/2, and the camera is at strip position 80
- **THEN** the shown windows are those of the third and first columns
- **AND** the first column's tile is placed at terminal column 40

#### Scenario: Floating window stays in place across the seam
- **WHEN** `loop_bands` is on, the screen area and the client's terminal are both 80×24, the viewed band holds three columns of width 1/2 and floating window P4 with its box at column 10, and the camera moves from 40 to 80
- **THEN** P4's box stays at terminal column 10
