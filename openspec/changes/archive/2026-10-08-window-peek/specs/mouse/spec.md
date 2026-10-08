## MODIFIED Requirements

### Requirement: Pointer targets
Every mouse event SHALL point at the terminal cell it reports, at a column and a row counted from 0 at the terminal's top-left cell. The client SHALL resolve the cell to a target against the frame it drew last. The first of these that covers the cell SHALL be the target:

1. a floating plugin window's box, from the top of its stacking order down;
2. a floating window's box of the viewed band, from the top of this client's stacking order down, with the window this client peeks on top, as the floating-windows capability's "Stacking" draws them, except the box of a window this client has minimized and does not peek, as the floating-windows capability defines;
3. a tile of the viewed band, at the position drawn;
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

#### Scenario: Minimized floating window over a tile
- **WHEN** this client has minimized a floating window whose box covers column 45 and row 3 above a tile, and the user presses there
- **THEN** the target is the tile's window

#### Scenario: Peeked minimized window over a tile
- **WHEN** this client has minimized floating window P3, whose box covers column 45 and row 3 above a tile, peeks P3, and the user presses there
- **THEN** the target is P3

#### Scenario: Peeked window over a higher window
- **WHEN** floating windows P3 and P4 both cover column 45 and row 3, this client focused P3 and then P4, peeks P3, and the user presses there
- **THEN** the target is P3
