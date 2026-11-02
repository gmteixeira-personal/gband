## MODIFIED Requirements

### Requirement: Focus across columns
While the tiled layer is active, focusing the column to the left or right SHALL move focus to the adjacent column in that direction. Within that column, focus SHALL go to the window this client focused most recently, or to the top window when this client has focused none of its windows. When no column exists in that direction and the configuration's `band_ends` is `"loop"` or `"bounce"`, focus SHALL go round the band: right of the last column to the first column, and left of the first column to the last. Under `"loop"` it SHALL do so whether or not the band's strip loops, as the Camera requirement defines. Under `"bounce"` the strip never loops, so the camera SHALL scroll back across the band to the newly focused column. When no column exists in that direction and `band_ends` is `"stop"`, or the band holds one column, the view SHALL not change. While the floating layer is active, these actions SHALL move focus between floating windows, as the floating-windows capability defines.

#### Scenario: Return to the remembered window
- **WHEN** column A holds P1 and P2, the client focuses P2, focuses the column to the right, then the column to the left
- **THEN** the client focuses P2

#### Scenario: Right of the last column
- **WHEN** `band_ends` is `"loop"`, a band holds columns A, B and C, and the client focuses a window in C and focuses the column to the right
- **THEN** the client focuses a window in A

#### Scenario: Left of the first column
- **WHEN** `band_ends` is `"loop"`, a band holds columns A, B and C, A holds P1 and P2 and C holds P3 and P4, the client has focused P4 most recently in C, and it focuses P1 and then the column to the left
- **THEN** the client focuses P4

#### Scenario: Short band still goes round
- **WHEN** `band_ends` is `"loop"`, the terminal is 80 columns wide, a band holds two columns of 40 cells, and the client focuses the second and then the column to the right
- **THEN** the client focuses a window in the first column

#### Scenario: One column
- **WHEN** `band_ends` is `"loop"`, a band holds one column, and the client focuses the column to the right
- **THEN** the view is unchanged

#### Scenario: No column to the right
- **WHEN** `band_ends` is `"stop"`, and the client focuses a window in the last column and focuses the column to the right
- **THEN** the focus is unchanged

#### Scenario: Bounce from the last column
- **WHEN** `band_ends` is `"bounce"`, a band holds columns A, B and C, and the client focuses a window in C and focuses the column to the right
- **THEN** the client focuses a window in A

#### Scenario: Bounce from the first column
- **WHEN** `band_ends` is `"bounce"`, a band holds columns A, B and C, A holds P1 and C holds P3 and P4, the client has focused P4 most recently in C, and it focuses P1 and then the column to the left
- **THEN** the client focuses P4

#### Scenario: Bounce on one column
- **WHEN** `band_ends` is `"bounce"`, a band holds one column, and the client focuses the column to the right
- **THEN** the view is unchanged

#### Scenario: Floating layer stays in its layer
- **WHEN** the floating layer is active on the band's only floating window, and the band holds columns A and B
- **THEN** focusing the column to the left or right leaves the view unchanged

### Requirement: Camera
Each band's camera SHALL be the strip position shown at the client terminal's left edge. A camera below 0 SHALL be allowed, and strip positions left of 0 SHALL be drawn empty while the band's strip does not loop. After every change of focus, of the layout or of the client's terminal width, the camera of the viewed band SHALL move by the configuration's `center_focused_column` policy, where the view spans from the camera to the camera plus the terminal's width. While the floating layer is active, the camera SHALL move as though the tiled window this client focused most recently in the band were focused, and SHALL NOT move when there is none.

A band's strip SHALL loop while the configuration's `band_ends` is `"loop"` and the strip's width, the end of its last column, less the width of its widest column is at least the terminal's width less one. A column and its copy are the strip's width less the column's width apart, so a view at most one cell wider than that gap never shows both. A strip that does not loop SHALL be drawn once, so no window is ever drawn twice. While a strip loops:

- Each column SHALL also stand at its strip position plus every whole multiple of the strip's width. Each of these positions is a copy of the column. The last column's copy ends at strip position 0 and the first column's copy starts at the strip's width, so the strip closes into a loop.
- A tile's drawn copy SHALL be the copy of its column that has at least one cell inside the view. The strip's length ensures that a tile has at most one.
- The policies below SHALL place the focused column at one of its copies. When focus moved to the column to the left or right, it SHALL be the copy next to the previously focused column's drawn copy, on the side of the move. When focus moved to a column the layout opened beside the previously focused column, it SHALL be the copy next to the previously focused column's drawn copy, on the side the column opened. Otherwise, or when that column has no drawn copy, it SHALL be the copy that needs the smallest camera move, the one with the smaller start on a tie.
- After every move, the camera SHALL be held at least 0 and below the strip's width, by adding or subtracting a whole multiple of the strip's width. Holding it SHALL NOT change what the view shows.
- When the strip's width changes, by a change of the layout, of the screen area or of the terminal's width, each tile SHALL be drawn at its drawn copy for the camera held within the new width, however many times the camera crossed the seam before.

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
- **WHEN** `band_ends` is `"loop"`, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused with the camera at 40, and the client focuses the column to the right
- **THEN** the client focuses the first column and the camera moves to 80
- **AND** the terminal shows the third column in cells 0 to 39 and the first column in cells 40 to 79

#### Scenario: Keep going right
- **WHEN** the client then focuses the column to the right again
- **THEN** the client focuses the second column and the camera moves to 120, which is held at 0

#### Scenario: Scroll left across the seam
- **WHEN** `band_ends` is `"loop"`, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the first is focused with the camera at 0, and the client focuses the column to the left
- **THEN** the client focuses the third column and the camera moves to -40, which is held at 80
- **AND** the terminal shows the third column in cells 0 to 39 and the first column in cells 40 to 79

#### Scenario: Always centre across the seam
- **WHEN** `band_ends` is `"loop"`, the policy is `"always"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused, and the client focuses the column to the right
- **THEN** the camera moves to 100
- **AND** the terminal shows the last 20 cells of the third column, then the first column, then the first 20 cells of the second column

#### Scenario: Strip one cell short of the terminal loops
- **WHEN** `band_ends` is `"loop"`, the policy is `"never"`, the terminal is 79 columns wide, a band holds three columns of 39 cells at strip positions 0, 39 and 78, the third is focused with the camera at 38, and the client focuses the column to the right
- **THEN** the client focuses the first column and the camera moves to 77
- **AND** the terminal shows the last cell of the second column in cell 0, the third column in cells 1 to 39 and the first column in cells 40 to 78

#### Scenario: Short strip does not loop
- **WHEN** `band_ends` is `"loop"`, the policy is `"always"`, the terminal is 80 columns wide, a band holds two columns of 40 cells at strip positions 0 and 40, and focus moves from the second column to the first by the column to the right
- **THEN** the camera moves to -20 and the 20 cells left of the first column are drawn empty
- **AND** each window is drawn once

#### Scenario: Looping off
- **WHEN** `band_ends` is `"stop"`, the policy is `"always"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, and the first column is focused
- **THEN** the camera is at -20 and the 20 cells left of the first column are drawn empty

#### Scenario: Bounce scrolls back to the first column
- **WHEN** `band_ends` is `"bounce"`, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused with the camera at 40, and the client focuses the column to the right
- **THEN** the client focuses the first column and the camera moves to 0
- **AND** each window is drawn once

#### Scenario: Bounce scrolls forward to the last column
- **WHEN** `band_ends` is `"bounce"`, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the first is focused with the camera at 0, and the client focuses the column to the left
- **THEN** the client focuses the third column and the camera moves to 40

#### Scenario: Bounce never loops the strip
- **WHEN** `band_ends` is `"bounce"`, the policy is `"always"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, and the first column is focused
- **THEN** the camera is at -20 and the 20 cells left of the first column are drawn empty

#### Scenario: Opening across the seam
- **WHEN** `band_ends` is `"loop"`, the policy is `"never"`, the terminal is 80 columns wide, a band holds two columns of 40 cells at strip positions 0 and 40, the second is focused with the camera at 0, and the layout opens a column of 40 cells right of the second, which the client focuses
- **THEN** the camera moves to 40
- **AND** the terminal shows the second column in cells 0 to 39 and the new column in cells 40 to 79

#### Scenario: Closing the last column pulls the camera back
- **WHEN** `band_ends` is `"stop"`, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused with the camera at 40, and the third column's only window exits
- **THEN** the client focuses the second column and the camera moves to 0
- **AND** the terminal shows the first column in cells 0 to 39 and the second in cells 40 to 79

#### Scenario: Shrinking the last column pulls the camera back
- **WHEN** `band_ends` is `"stop"`, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused with the camera at 40, and the third column's width becomes 20 cells
- **THEN** the camera moves to 20 and the third column is drawn in cells 60 to 79

#### Scenario: Narrower area pulls the camera back
- **WHEN** the policy is `"never"`, the terminal is 60 columns wide, a band holds two columns of 40 cells at strip positions 0 and 40, the second is focused with the camera at 20, and the screen area changes so the columns are 30 cells wide at strip positions 0 and 30
- **THEN** the camera moves to 0
- **AND** the terminal shows the first column in cells 0 to 29 and the second in cells 30 to 59

#### Scenario: Pulled back no further than 0
- **WHEN** the policy is `"never"`, the terminal is 80 columns wide, a band holds two columns of 35 cells at strip positions 0 and 35, the second is focused with the camera at 10, and the screen area changes so the columns are 30 cells wide at strip positions 0 and 30
- **THEN** the camera moves to 0 and cells 60 to 79 are drawn empty

#### Scenario: Centred camera keeps its blank strip
- **WHEN** `band_ends` is `"stop"`, the policy is `"always"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused, and the screen area changes so the columns are 30 cells wide at strip positions 0, 30 and 60
- **THEN** the camera moves to 35 and cells 55 to 79 are drawn empty

#### Scenario: Wider terminal after crossing the seam
- **WHEN** `band_ends` is `"loop"`, the policy is `"never"`, the terminal is 80 columns wide, a band holds four columns of 40 cells at strip positions 0, 40, 80 and 120, the first is focused with the camera at 0, the client focuses the column to the left, which focuses the fourth column with the camera held at 120, and the terminal and the screen area then widen to 100 columns, so the columns are 50 cells wide at strip positions 0, 50, 100 and 150
- **THEN** the camera stays at 120
- **AND** the terminal shows the last 30 cells of the third column in cells 0 to 29, the fourth column in cells 30 to 79 and the first 20 cells of the first column in cells 80 to 99

#### Scenario: Narrower column after crossing the seam
- **WHEN** `band_ends` is `"loop"`, the policy is `"never"`, the terminal is 80 columns wide, a band holds four columns of 40 cells at strip positions 0, 40, 80 and 120, the first is focused with the camera at 0, the client focuses the column to the left, which focuses the fourth column with the camera held at 120, and the first column's width then becomes 26 cells, so the columns stand at strip positions 0, 26, 66 and 106
- **THEN** the camera moves to 106
- **AND** the terminal shows the fourth column in cells 0 to 39, the first column in cells 40 to 65 and the first 14 cells of the second column in cells 66 to 79

### Requirement: Center the focused column
While the tiled layer is active, the `center_column` view action SHALL move the viewed band's camera so that the focused window's column sits in the middle of the client's terminal, as niri's `center-column` does. The camera SHALL move to the column's start less half the difference between the terminal's width and the column's width, rounded down. When the column is at least as wide as the terminal, the camera SHALL move to the column's start. While the band's strip loops, as the Camera requirement defines, the column's start SHALL be that of the copy needing the smallest camera move, and the camera SHALL then be held within the strip. The action SHALL change neither the focused window nor the viewed band, and SHALL send nothing to the server. On a band with no focused window, it SHALL leave the view unchanged.

While the floating layer is active, the action SHALL instead centre the focused floating window in the session's screen area, as niri's `center-column` does for a floating window. The client SHALL send the server the placing of that window, as the floating-windows capability defines, at a column of half the difference between the area's width and the box's width, and a row of half the difference between the area's height and the box's height, each rounded down, where the box is as placed in the current screen area. The client SHALL send nothing when the box already sits there. The action SHALL change neither the camera, the focused window nor the viewed band.

After the action, every later change of focus, of the layout or of the client's terminal width SHALL move the camera by the `center_focused_column` policy, as the Camera requirement defines, starting from the centred camera.

#### Scenario: Center a column at the right edge
- **WHEN** the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused with the camera at 40, and the client runs `center_column`
- **THEN** the camera moves to 60
- **AND** the third column is still focused

#### Scenario: Center the first column
- **WHEN** the terminal is 80 columns wide, the band holds only the focused column, 40 cells wide at strip position 0, the camera is at 0, and the client runs `center_column`
- **THEN** the camera moves to -20 and the 20 cells left of the column are drawn empty

#### Scenario: Column wider than the terminal
- **WHEN** the terminal is 80 columns wide, the focused column is 100 cells wide at strip position 40, and the client runs `center_column`
- **THEN** the camera moves to 40

#### Scenario: Centred column keeps the camera
- **WHEN** the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the second is focused, the client runs `center_column`, and the terminal then narrows to 78 columns
- **THEN** the camera stays at 20

#### Scenario: Policy resumes on the next focus change
- **WHEN** `band_ends` is `"stop"`, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the first is focused, the client runs `center_column`, and focus then moves to the third column
- **THEN** the camera moves to 40

#### Scenario: Center a floating window
- **WHEN** the screen area is 80×24, the camera is at 40, the focused floating window's box is 40 columns wide and 4 rows high at column 0 and row 0, and the client runs `center_column`
- **THEN** the client sends the placing of that window at column 20 and row 10
- **AND** the camera stays at 40 and the floating window is still focused

#### Scenario: Floating window already centred
- **WHEN** the screen area is 80×24, the focused floating window's box is 40 columns wide and 20 rows high at column 20 and row 2, and the client runs `center_column`
- **THEN** the client sends nothing

#### Scenario: Empty band
- **WHEN** the client views an empty band with its camera at 0 and runs `center_column`
- **THEN** the camera stays at 0

#### Scenario: Center the first column of a looping strip
- **WHEN** `band_ends` is `"loop"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the first is focused with the camera at 0, and the client runs `center_column`
- **THEN** the camera moves to -20, which is held at 100
- **AND** the terminal shows the last 20 cells of the third column, then the first column, then the first 20 cells of the second column

### Requirement: Shown windows
A view's shown windows SHALL be the windows of its viewed band whose tile, as the layout capability's tile geometry gives it for the screen area, or whose box, as the floating-windows capability places it, has at least one cell inside the client's terminal. A floating window this client has minimized SHALL NOT be shown, wherever its box is, unless this client peeks it. A tile's cells SHALL be placed at its strip position less the viewed band's camera position, from the terminal's top row. While the band's strip loops, as the Camera requirement defines, a tile's cells SHALL be placed at its drawn copy's position instead, and a tile with no drawn copy SHALL NOT be shown. Wherever the client-attach and animations capabilities draw a tile at its strip position less a camera, they SHALL use its drawn copy while the strip loops. A box's cells SHALL be placed at its column and row, from the terminal's top-left cell, whatever the camera. A tile that a box covers SHALL still be shown. A tab of a tabbed column that is not this client's shown tab, as the tabbed-columns capability defines it, SHALL NOT be shown, wherever its tile is. A view of an empty band SHALL show no window.

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
- **WHEN** `band_ends` is `"loop"`, the screen area and the client's terminal are both 80×24, the viewed band holds three columns of width 1/2, and the camera is at strip position 80
- **THEN** the shown windows are those of the third and first columns
- **AND** the first column's tile is placed at terminal column 40

#### Scenario: Floating window stays in place across the seam
- **WHEN** `band_ends` is `"loop"`, the screen area and the client's terminal are both 80×24, the viewed band holds three columns of width 1/2 and floating window P4 with its box at column 10, and the camera moves from 40 to 80
- **THEN** P4's box stays at terminal column 10

#### Scenario: Minimized window not shown
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds one column of width 1/2 with P1 and floating window P4, and the client has minimized P4
- **THEN** the only shown window is P1

#### Scenario: Peeked minimized window shown
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds one column of width 1/2 with P1 and floating window P4, and the client has minimized P4 and peeks it
- **THEN** the shown windows are P1 and P4
- **AND** when the client then focuses P1, the only shown window is P1

#### Scenario: Hidden tab not shown
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds one tabbed column of width 1/2 with P1 and P2, and the client focuses P2
- **THEN** the only shown window is P2
- **AND** when the client then focuses the window above, the only shown window is P1

### Requirement: Peek a window
Peeking a named window SHALL focus it as "Focus a named window" focuses it: the client views the window's band, focuses the window, shows the window's layer as active, and the camera follows the new focus. A peek SHALL record nothing in this client's view. It SHALL NOT change the focus order, the window that each band remembers as last focused in each layer, the layer that each band remembers as active, the stacking order or the set of minimized windows. A peek SHALL NOT restore a window this client has minimized. For every rule that names the window this client focused most recently, last focused or never focused, a peek SHALL NOT count as a focus. Peeking a window not in the layout SHALL leave the view unchanged.

While a peek lasts, the peeked window SHALL be this client's focused window. View actions, session actions resolved against the view, and the focus that the client reports to Lua and in its events SHALL use it as they use any focused window. A peeked floating window SHALL be drawn above every other floating window of its band, as the floating-windows capability's "Stacking" defines, and SHALL be shown and a pointer target, whether or not this client has minimized it. A peeked tiled window SHALL be drawn as the focused tiled window is, under the band's floating windows.

The peek SHALL end, and a focus SHALL take its place, when one of these happens:

- The view focuses a window by any means other than a peek, the peeked window included. That focus SHALL be recorded as usual.
- The client peeks another window. The new peek SHALL take the place of the old one.
- The viewed band changes.

A focus of the peeked window by other means SHALL restore it when this client has minimized it, put it on top of the stacking order, and record the focus. The focused window does not change, so it SHALL emit no `FocusChanged` and no `BandChanged`.

The peek SHALL end, and no window SHALL take the focus in its place, when one of these happens:

- The client views the band it already views by name, as "View a named band" defines.
- This client minimizes the peeked window, as the floating-windows capability defines.
- The peeked window leaves the layout.
- A reload of the configuration loads the new configuration, and so closes every plugin window, as the plugin-windows capability defines. A reload that fails keeps the previous configuration, its plugin windows and the peek.

In these four cases, focus SHALL go to the window this client last focused in the viewed band, in the layer that window is in, when that window is still there and this client has not minimized it, and this SHALL record no focus. Otherwise focus SHALL go where "Switch band" sends it on viewing that band.

When the peek ends, the window SHALL return to its place in the stacking order, and SHALL be hidden again when this client has minimized it and no focus restored it. A peeked window that moves to the other layer of its band SHALL stay peeked, and the shown layer SHALL follow it. A layout change that keeps the peeked window in the layout SHALL NOT end the peek. A drag gesture pressed on the peeked window focuses it, as the mouse capability defines, and so ends the peek as any focus does. Each time a client attaches, it SHALL peek no window.

#### Scenario: Peek a minimized window
- **WHEN** band 1 holds floating windows 3 and 4 with overlapping boxes, a client focused 3 and then 4, minimized 3, and peeks 3
- **THEN** the client focuses 3 with the floating layer active and draws 3 over 4
- **AND** 3 is still minimized, and the last focus of 3 and of 4 is unchanged

#### Scenario: Focus elsewhere hides it again
- **WHEN** the client of "Peek a minimized window" then focuses 4 by number
- **THEN** the client focuses 4 and does not draw 3

#### Scenario: Focus the peeked window
- **WHEN** band 1 holds floating windows 3 and 4 with overlapping boxes, a client focused 3 and then 4, minimized 3, peeks 3, and then focuses 3 by number
- **THEN** 3 is no longer minimized, the client draws 3 over 4, and 3 has the highest last focus
- **AND** the focus by number emits no `FocusChanged`

#### Scenario: Peek in another band
- **WHEN** a client views band 1 with window 1 focused, band 2 holds tiled windows 2 and 5, the client last focused 5 there, and it peeks 2
- **THEN** the client views band 2 with window 2 focused
- **AND** when it then views band 1 by name and band 2 again, it focuses window 5

#### Scenario: Moving focus from a peek
- **WHEN** band 1 holds the columns of windows 1, 2 and 3, a client focuses window 1, peeks window 2, and focuses the column to the right
- **THEN** the client focuses window 3
- **AND** window 2's last focus is unchanged

#### Scenario: View the viewed band
- **WHEN** band 1 holds only floating window 3, a client that views band 1 has minimized 3 and so focuses no window, peeks 3, and then views band 1 by name
- **THEN** the client focuses no window and does not draw 3

#### Scenario: Peeked window closes
- **WHEN** a client focuses tiled window 1 of band 1, which also holds floating window 3, peeks 3, and the program of 3 exits
- **THEN** the client focuses window 1 with the tiled layer active
- **AND** window 1's last focus is unchanged

#### Scenario: Reload ends the peek
- **WHEN** band 1 holds floating windows 3 and 4 with overlapping boxes, a client focused 3 and then 4, minimized 3, peeks 3, and the user saves `user/init.lua`
- **THEN** after the reload the client focuses 4 and does not draw 3
- **AND** the last focus of 3 and of 4 is unchanged

#### Scenario: Failed reload keeps the peek
- **WHEN** a client focuses tiled window 1 of band 1, which also holds floating window 3, peeks 3, and the user saves a `user/init.lua` that raises an error
- **THEN** the client still focuses 3

#### Scenario: Peek a tiled window
- **WHEN** `band_ends` is `"stop"`, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells, the first is focused with the camera at 0, and the client peeks the window of the third column
- **THEN** the client focuses that window and the camera moves to 40
- **AND** when it then views the band it views by name, it focuses the window of the first column and the camera moves to 0
