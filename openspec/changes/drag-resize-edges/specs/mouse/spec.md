## MODIFIED Requirements

### Requirement: Resize by dragging
`drag_resize_window` pressed on a window or a plugin window SHALL focus it and move one or two edges of its box. Dispatched with a target holding `edges`, as the lua-control capability's "Action targets" defines, it SHALL move exactly the edges that `edges` names, wherever the press's cell lies in the box. Dispatched with no target, it SHALL pick the edges it moves from the press's cell in the box pressed on, including its border. Let `x` and `y` be the cell's column and row in the box, and `w` and `h` the box's width and height:

- The left edge when `x` is less than `w / 3`, rounded down; the right edge when `x` is at least `w` less `w / 3`, rounded down; no vertical edge otherwise.
- The top edge when `y` is less than `h / 3`, rounded down; the bottom edge when `y` is at least `h` less `h / 3`, rounded down; no horizontal edge otherwise.
- When neither rule picks an edge, the single edge nearest the cell, preferring right, then bottom, then left, then top among equals.

Let `dx` and `dy` be the pointer's movement in columns and rows since the press. Each moved edge SHALL follow the pointer, the far edge staying put. An edge that the gesture does not move SHALL stay put. These rules SHALL apply in the same way to edges that a target names and to edges picked from the press's cell:

- For a tile, a moved left or right edge SHALL set its column's width to the column's width in cells at the press, plus `dx` for the right edge or less `dx` for the left edge, at least 3 cells, as the session's screen area's width divided into that many cells. Moving the left edge SHALL also move the camera by the change in width, so the column's right edge stays at the same terminal column. A moved top or bottom edge SHALL set the window's height to its tile height at the press, plus `dy` for the bottom edge or less `dy` for the top edge, as a number of rows, as the layout capability's "Set a window's height" defines and limits. This SHALL hold for every window of a column, the first and the last included. The column's tiles SHALL then be placed as the layout capability defines for that height.
- For a floating window, the width and `rows` SHALL change the same way, each at least 3. Moving the left or top edge SHALL also place the box so that its right or bottom edge stays put.
- For a floating plugin window, `width` and `height` SHALL change the same way, each at least 3 with a border and at least 1 without, and `col` and `row` SHALL move with the left and top edges, as `gband.win.set_config` does. Its `on_resize` SHALL run for each new size.

#### Scenario: Pick the right edge
- **WHEN** a tile's box is 40×24 and the user presses `rightmouse` in navigation mode at column 35 and row 12 of that box
- **THEN** the gesture moves the right edge only

#### Scenario: Pick a corner
- **WHEN** a floating window's box is 30×12 and the press is at column 2 and row 10 of that box
- **THEN** the gesture moves the left and bottom edges

#### Scenario: Pick the nearest edge from the middle
- **WHEN** a box is 30×12 and the press is at column 14 and row 5 of that box
- **THEN** the gesture moves the top edge only

#### Scenario: Widen a column
- **WHEN** the screen area is 80×24, a column is 40 cells wide, and the user drags its right edge 8 cells right
- **THEN** the client sends set width naming that column's window and the width 3/5

#### Scenario: Grow a window down
- **WHEN** the screen area is 80×24, a column holds P1 above P2 with tiles of 12 rows each, and the user drags P1's bottom edge 4 rows down
- **THEN** P1 has a fixed height of 16 rows and P2's tile is 8 rows high

#### Scenario: Resize a floating window from its left edge
- **WHEN** the screen area is 80×24, a floating window's box is 40 cells wide at column 20, and the user drags its left edge 10 cells left
- **THEN** its box is 50 cells wide and starts at column 10

#### Scenario: Named bottom edge pressed near a corner
- **WHEN** the screen area is 80×24, a floating window's box is 30×12 at column 10 and row 4, `drag_resize_window` runs with the target `{ edges = { "bottom" } }` for a press at column 2 and row 11 of that box, and the pointer moves 4 columns left and 3 rows down
- **THEN** the box is 30 cells wide at column 10 and row 4, and 15 rows high
- **AND** the client sends no set width and no placing of that window

#### Scenario: Named corner
- **WHEN** the screen area is 80×24, a floating window's box is 30×12 at column 10 and row 4, `drag_resize_window` runs with the target `{ edges = { "left", "top" } }` for a press at column 1 and row 0 of that box, and the pointer moves 5 columns left and 2 rows up
- **THEN** the box is 35×14 at column 5 and row 2

#### Scenario: Named edge pressed away from it
- **WHEN** the screen area is 80×24, the camera is at 0, a band holds two columns of 40 cells, and `drag_resize_window` runs with the target `{ edges = { "right" } }` for a press at column 5 and row 12 of the first column's tile, and the pointer moves 8 columns right
- **THEN** the client sends set width naming the first column's window and the width 3/5
- **AND** the camera stays at 0

#### Scenario: Named left edge of a tile moves the camera
- **WHEN** the screen area is 80×24, the camera is at 0, a band holds three columns of 40 cells, and `drag_resize_window` runs with the target `{ edges = { "left" } }` for a press at column 35 and row 12 of the second column's tile, and the pointer moves 4 columns left
- **THEN** the client sends set width naming the second column's window and the width 11/20
- **AND** the camera is at 4

#### Scenario: Named top edge of a column's first window
- **WHEN** the screen area is 80×24, a column holds P1 above P2 with tiles of 12 rows each, and `drag_resize_window` runs with the target `{ edges = { "top" } }` for a press at column 20 and row 6 of P1's box, and the pointer moves 4 rows up
- **THEN** P1 has a fixed height of 16 rows and its tile starts at row 0
- **AND** P2's tile is 8 rows high

#### Scenario: Named edge of a floating plugin window
- **WHEN** a floating plugin window with a border has a 30×12 box at column 10 and row 3 of the ribbon area, `drag_resize_window` runs with the target `{ edges = { "right" } }` for a press at column 28 and row 10 of that box, and the pointer moves 6 columns right and 2 rows down
- **THEN** `gband.win.info(win)` reports `col` 10, `row` 3, `width` 36 and `height` 12
- **AND** its `on_resize` ran last with 34 columns and 10 rows
