## MODIFIED Requirements

### Requirement: Pointer targets
Every mouse event SHALL point at the terminal cell it reports, at a column and a row counted from 0 at the terminal's top-left cell. The client SHALL resolve the cell to a target against the frame it drew last. The first of these that covers the cell SHALL be the target:

1. a floating plugin window's box, from the top of its stacking order down;
2. a floating window's box of the viewed band, from the top of this client's stacking order down, with the window this client peeks on top, as the floating-windows capability's "Stacking" draws them, except the box of a window this client has minimized and does not peek, as the floating-windows capability defines;
3. a tile of the viewed band, at the position drawn, where a tabbed column's tile is its shown tab's, and a cell of a tab indicator's segment is that segment's tab, as the tabbed-columns capability defines;
4. the ribbon area, as empty ribbon;
5. any other cell, as outside.

A tile whose window is a tiled plugin window's drawn window SHALL be that plugin window as the target. A target that is a window or a plugin window SHALL include its border cells. Its content cell SHALL be the cell's column and row less those of its content area's top-left cell, counted from 0. A border cell SHALL have no content cell.

#### Scenario: Click inside a tile
- **WHEN** the ribbon area starts at the terminal's top-left cell, a tile's box spans columns 40 to 79 and rows 0 to 23, and the user presses the left button at column 45 and row 3
- **THEN** the target is that tile's window, at content column 4 and content row 2

#### Scenario: Floating window over a tile
- **WHEN** a floating window's box covers column 45 and row 3 above a tile, and the user presses there
- **THEN** the target is the floating window

#### Scenario: Border cell
- **WHEN** the user presses on the top border of a tile
- **THEN** the target is that tile's window, with no content cell

#### Scenario: Empty ribbon
- **WHEN** the camera is at -20 and the user presses at column 5 of the ribbon area, left of the first column
- **THEN** the target is empty ribbon

#### Scenario: Tab indicator segment
- **WHEN** the ribbon area starts at the terminal's top-left cell, the viewed band holds one tabbed column whose tile spans columns 0 to 39 and rows 0 to 23 with tabs P1, P2 and P3, P1 is shown, and the user presses at column 0 and row 20
- **THEN** the target is P3, as a border cell with no content cell

#### Scenario: Minimized floating window over a tile
- **WHEN** this client has minimized a floating window whose box covers column 45 and row 3 above a tile, and the user presses there
- **THEN** the target is the tile's window

#### Scenario: Peeked minimized window over a tile
- **WHEN** this client has minimized floating window P3, whose box covers column 45 and row 3 above a tile, peeks P3, and the user presses there
- **THEN** the target is P3

#### Scenario: Peeked window over a higher window
- **WHEN** floating windows P3 and P4 both cover column 45 and row 3, this client focused P3 and then P4, peeks P3, and the user presses there
- **THEN** the target is P3

### Requirement: Move a tiled window by dragging
`drag_window` pressed on a tile SHALL focus its window. The first motion to another cell SHALL lift the window. While lifted, the client SHALL draw the window's tile at its position at the press moved by the pointer's movement since the press, above every tile and below floating windows, and SHALL draw the drop place as a one-cell outline in the focused border's style. The rest of the band SHALL be drawn as before the lift, except that a tabbed column whose tab is lifted SHALL show the tab that the tabbed-columns capability's "Shown tab" picks when the lifted window is left out of the column.

The drop place SHALL follow the pointer's cell on the viewed band's strip, ignoring floating windows and the lifted window:

- In the left quarter of a column's span, rounded down: a new column left of that column.
- In the right quarter of a column's span, rounded down: a new column right of that column.
- Elsewhere in a column's span: inside that column, above the window whose tile holds the pointer's row when the row lies in that tile's upper half, rounded down, and below it otherwise. A row under the column's last tile SHALL count as below its last window. In a tabbed column, the tile that holds the row SHALL be the shown tab's, picked as the column is drawn during the lift, and a row under it SHALL count as below the shown tab.
- Right of the last column: a new column right of the last column. Left of the first column: a new column left of the first column.

A column holding only the lifted window SHALL count as that column for the drop place, so dropping on it leaves the window where it was.

At the release, the client SHALL send moving the window to the drop place, as the layout capability's "Move a window to a place" defines, naming as the reference window the window of the place's column that this client focused most recently, or that column's first window, for a new column; and naming the window above or below which it drops, inside a column. It SHALL send nothing when moving there would leave the layout unchanged. A release without a lift SHALL only have focused the window.

#### Scenario: Drop between two columns
- **WHEN** the terminal is 80 columns wide, the camera is at 0, a band holds columns A, B and C of 40 cells with A's window focused, and the user drags A's window with `drag_window` and releases at column 75, in B's right quarter
- **THEN** the band holds B, a column with A's window, and C, in that order

#### Scenario: Drop into a column
- **WHEN** a band holds columns A and B of 40 cells, B holds one window with its tile on rows 0 to 23, and the user drags A's window and releases at column 60 and row 18
- **THEN** B holds its window above A's window, and A's column is removed

#### Scenario: Release without moving
- **WHEN** the user presses `leftmouse` in navigation mode on an unfocused tile and releases at the same cell
- **THEN** that window is focused and the layout is unchanged

#### Scenario: Drop into a tabbed column
- **WHEN** a band holds column A of 40 cells with P1 and tabbed column B of 40 cells with P2 and P3, B shows P3 with its tiles on rows 0 to 23, and the user drags P1 and releases at column 60 and row 18
- **THEN** B is tabbed and holds P2, P3 and P1, in that order, and A is removed
