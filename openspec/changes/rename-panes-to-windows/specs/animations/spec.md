## RENAMED Requirements

- FROM: `### Requirement: Floating panes at rest`
- TO: `### Requirement: Floating windows at rest`

## MODIFIED Requirements

### Requirement: Animated presentation
The client SHALL separate what it draws from what the layout and its view decide. The layout, the tile geometry, the focused window, the viewed band and each camera SHALL change at once, as the layout, layout-view and client-attach capabilities define. Only the drawn camera, the drawn band and each tile's drawn position and size SHALL move toward those targets over time. Key input, pastes and session actions SHALL go to the newly focused window from the moment focus changes, whether or not an animation is running. When no animation is running, the drawn state SHALL equal the target state.

#### Scenario: Typing during a scroll
- **WHEN** the client focuses the column to the right, the camera starts to scroll, and the user types `echo right` and Enter before the scroll ends
- **THEN** `right` appears in the newly focused window only

#### Scenario: At rest
- **WHEN** no layout or view change has happened for 400 ms
- **THEN** the client draws exactly what it would draw with animations off

### Requirement: Band switch
When the viewed band changes to another band that is still in the layout, the client SHALL animate a vertical position from the old band to the new one. Band `i` in the layout SHALL be drawn in a region as tall as the client terminal, starting at row `i` times the terminal's height less the drawn vertical position. Each band's tiles SHALL be cut at the edges of its region. The band being left SHALL be drawn with the camera it was drawn with when the switch started. A switch made while another is running SHALL continue from the drawn vertical position. When bands are added or removed above the viewed one during a switch, the drawn vertical position SHALL shift with them, so that no band jumps on screen.

#### Scenario: Switch down
- **WHEN** the terminal is 80×24, the client views B1 at rest and views the band below, B2
- **THEN** the first frame shows B1 only
- **AND** a later frame shows the bottom part of B1 above the top part of B2
- **AND** within 400 ms only B2 is shown

#### Scenario: Two switches in a row
- **WHEN** the client views B1 of B1, B2 and B3, views the band below, and views the band below again before the first slide ends
- **THEN** the slide continues downward without a jump and ends showing B3

#### Scenario: Old band removed
- **WHEN** the client switches from B1 to B2 and the last window of B1 exits before the slide ends
- **THEN** the client shows B2 at rest from the next frame

### Requirement: Tile movement
When the layout changes, each window of the viewed band that it held both before and after the change SHALL have its drawn strip position and its drawn row animated from where it was drawn to its new tile position. A window new to the band SHALL be drawn at its tile position from the first frame. A window that left the band SHALL no longer be drawn. Where tiles overlap while they move, the focused tile SHALL be drawn over the others, and the others SHALL be drawn in layout order. Windows of bands that are not drawn SHALL NOT animate.

#### Scenario: Open a pane between two columns
- **WHEN** the viewed band holds columns A and B of width 1/3 on a 90×30 area, A is focused, and a window opens right of A
- **THEN** B's tile is drawn starting at strip position 30 in the first frame
- **AND** it slides right until it starts at strip position 75, after the new 45-cell column

#### Scenario: Close a column
- **WHEN** the viewed band holds columns A, B and C, and B's only window exits
- **THEN** C's tile slides left from B's end to where B started

#### Scenario: Expel to the right
- **WHEN** a column holds P1 and P2, and P2 is expelled to the right
- **THEN** P2's tile moves from the lower half of the old column toward the top of the new column to its right

### Requirement: Resize morph
When a window's tile width or height changes, the client SHALL animate the tile's drawn width and height from its drawn size to its new size. The tile's border SHALL be drawn at the drawn size. The window's grid SHALL be drawn from the inside of the tile's top-left corner and cut at the drawn border. Cells inside the border that the grid does not cover SHALL be blank. The morph SHALL NOT change when or how the server resizes the window.

#### Scenario: Cycle a width
- **WHEN** the focused column is 40 cells wide on an 80×24 area and its width is cycled to 2/3
- **THEN** the first frame draws its tile 40 cells wide
- **AND** later frames draw it wider, with the right border moving, until it is 53 cells wide

#### Scenario: Shrinking cuts the grid
- **WHEN** a tile shrinks from 53 to 26 cells wide while the window's grid is still 51 columns wide
- **THEN** each frame shows only the grid's columns that fit inside the drawn border

### Requirement: Cursor during motion
While the drawn camera, the drawn vertical position, or the focused tile's drawn position or size differs from its target, the terminal's cursor SHALL be hidden. When they all reach their targets, the cursor SHALL follow the client-attach capability's rules again.

#### Scenario: Cursor returns after a scroll
- **WHEN** focus moves to a column that needs the camera to scroll
- **THEN** the cursor is hidden while the camera moves
- **AND** it sits at the focused window's cursor once the camera reaches its target

### Requirement: Floating windows at rest
The client SHALL draw every floating window at its box, as the client-attach capability places it, in every frame, without animating its position or size. A window that moves from a column to the floating layer SHALL be drawn at its box from the first frame. A window that moves from the floating layer to a column SHALL be drawn at its tile position from the first frame, as a window new to the band is. The tiles of the other windows SHALL move as "Tile movement" defines.

#### Scenario: Float a pane between two columns
- **WHEN** the viewed band holds columns A, B and C, and B's only window is floated
- **THEN** the first frame draws that window at its box
- **AND** C's tile slides left from B's end to where B started

#### Scenario: Move a floating pane
- **WHEN** a floating window is moved right by one step
- **THEN** the next frame draws its box at the new column
