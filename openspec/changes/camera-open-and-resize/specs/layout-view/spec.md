## MODIFIED Requirements

### Requirement: Camera
Each band's camera SHALL be the strip position shown at the client terminal's left edge. A camera below 0 SHALL be allowed, and strip positions left of 0 SHALL be drawn empty while the band's strip does not loop. After every change of focus, of the layout or of the client's terminal width, the camera of the viewed band SHALL move by the configuration's `center_focused_column` policy, where the view spans from the camera to the camera plus the terminal's width. While the floating layer is active, the camera SHALL move as though the tiled window this client focused most recently in the band were focused, and SHALL NOT move when there is none.

A band's strip SHALL loop while the configuration's `loop_bands` is on and the strip's width, the end of its last column, less the width of its widest column is at least the terminal's width less one. A strip that does not loop SHALL be drawn once, so no window is ever drawn twice. While a strip loops:

- Each column SHALL also stand at its strip position plus every whole multiple of the strip's width. Each of these positions is a copy of the column. The last column's copy ends at strip position 0 and the first column's copy starts at the strip's width, so the strip closes into a loop.
- A tile's drawn copy SHALL be the copy of its column that has at least one cell inside the view. The strip's length ensures that a tile has at most one.
- The policies below SHALL place the focused column at one of its copies. When focus moved to the column to the left or right, it SHALL be the copy next to the previously focused column's drawn copy, on the side of the move. When focus moved to a column the layout opened beside the previously focused column, it SHALL be the copy next to the previously focused column's drawn copy, on the side the column opened. Otherwise, or when that column has no drawn copy, it SHALL be the copy that needs the smallest camera move, the one with the smaller start on a tie.
- After every move, the camera SHALL be held at least 0 and below the strip's width, by adding or subtracting a whole multiple of the strip's width. Holding it SHALL NOT change what the view shows.

Under `"never"`, the default:

- When the focused window's column lies wholly inside the view, the camera SHALL not move.
- Otherwise, when the column is at least as wide as the terminal or starts left of the view, the camera SHALL move to the column's start.
- Otherwise, the camera SHALL move so that the column's end meets the terminal's right edge.
- Once the camera has moved as above, when the strip does not loop, the camera is above 0 and the strip's end lies left of the terminal's right edge, the camera SHALL move left so that the strip's end meets the terminal's right edge, and SHALL NOT move below 0. This SHALL apply after every change that moves the camera by this policy, whether a change of focus, of the layout, of the screen area or of the terminal's width.

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

#### Scenario: Opening across the seam
- **WHEN** `loop_bands` is on, the policy is `"never"`, the terminal is 80 columns wide, a band holds two columns of 40 cells at strip positions 0 and 40, the second is focused with the camera at 0, and the layout opens a column of 40 cells right of the second, which the client focuses
- **THEN** the camera moves to 40
- **AND** the terminal shows the second column in cells 0 to 39 and the new column in cells 40 to 79

#### Scenario: Closing the last column pulls the camera back
- **WHEN** `loop_bands` is off, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused with the camera at 40, and the third column's only window exits
- **THEN** the client focuses the second column and the camera moves to 0
- **AND** the terminal shows the first column in cells 0 to 39 and the second in cells 40 to 79

#### Scenario: Shrinking the last column pulls the camera back
- **WHEN** `loop_bands` is off, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused with the camera at 40, and the third column's width becomes 20 cells
- **THEN** the camera moves to 20 and the third column is drawn in cells 60 to 79

#### Scenario: Narrower area pulls the camera back
- **WHEN** the policy is `"never"`, the terminal is 60 columns wide, a band holds two columns of 40 cells at strip positions 0 and 40, the second is focused with the camera at 20, and the screen area changes so the columns are 30 cells wide at strip positions 0 and 30
- **THEN** the camera moves to 0
- **AND** the terminal shows the first column in cells 0 to 29 and the second in cells 30 to 59

#### Scenario: Pulled back no further than 0
- **WHEN** the policy is `"never"`, the terminal is 80 columns wide, a band holds two columns of 35 cells at strip positions 0 and 35, the second is focused with the camera at 10, and the screen area changes so the columns are 30 cells wide at strip positions 0 and 30
- **THEN** the camera moves to 0 and cells 60 to 79 are drawn empty

#### Scenario: Centred camera keeps its blank strip
- **WHEN** `loop_bands` is off, the policy is `"always"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused, and the screen area changes so the columns are 30 cells wide at strip positions 0, 30 and 60
- **THEN** the camera moves to 35 and cells 55 to 79 are drawn empty
