## RENAMED Requirements

- FROM: `### Requirement: Workspace switch`
- TO: `### Requirement: Band switch`


## MODIFIED Requirements

### Requirement: Animated presentation
The client SHALL separate what it draws from what the layout and its view decide. The layout, the tile geometry, the focused pane, the viewed band and each camera SHALL change at once, as the layout, layout-view and client-attach capabilities define. Only the drawn camera, the drawn band and each tile's drawn position and size SHALL move toward those targets over time. Key input, pastes and session actions SHALL go to the newly focused pane from the moment focus changes, whether or not an animation is running. When no animation is running, the drawn state SHALL equal the target state.

#### Scenario: Typing during a scroll
- **WHEN** the client focuses the column to the right, the camera starts to scroll, and the user types `echo right` and Enter before the scroll ends
- **THEN** `right` appears in the newly focused pane only

#### Scenario: At rest
- **WHEN** no layout or view change has happened for 400 ms
- **THEN** the client draws exactly what it would draw with animations off

### Requirement: Camera scroll
When the camera of the viewed band changes, the client SHALL animate the drawn camera from its drawn position to the new camera position. Tiles SHALL be drawn at their strip position less the drawn camera.

#### Scenario: Scroll right
- **WHEN** the terminal is 80 columns wide, a band holds three columns of 40 cells, the camera is at 0 at rest, and focus moves from the second column to the third
- **THEN** the first frame draws the camera at 0
- **AND** later frames draw it between 0 and 40 until it reaches 40

#### Scenario: Scroll reversed before it ends
- **WHEN** the camera is scrolling from 0 toward 40 and focus moves back to the first column
- **THEN** the drawn camera turns back without a jump and reaches 0

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
- **WHEN** the client switches from B1 to B2 and the last pane of B1 exits before the slide ends
- **THEN** the client shows B2 at rest from the next frame

### Requirement: Tile movement
When the layout changes, each pane of the viewed band that it held both before and after the change SHALL have its drawn strip position and its drawn row animated from where it was drawn to its new tile position. A pane new to the band SHALL be drawn at its tile position from the first frame. A pane that left the band SHALL no longer be drawn. Where tiles overlap while they move, the focused tile SHALL be drawn over the others, and the others SHALL be drawn in layout order. Panes of bands that are not drawn SHALL NOT animate.

#### Scenario: Open a pane between two columns
- **WHEN** the viewed band holds columns A and B of width 1/3 on a 90×30 area, A is focused, and a pane opens right of A
- **THEN** B's tile is drawn starting at strip position 30 in the first frame
- **AND** it slides right until it starts at strip position 75, after the new 45-cell column

#### Scenario: Close a column
- **WHEN** the viewed band holds columns A, B and C, and B's only pane exits
- **THEN** C's tile slides left from B's end to where B started

#### Scenario: Expel to the right
- **WHEN** a column holds P1 and P2, and P2 is expelled to the right
- **THEN** P2's tile moves from the lower half of the old column toward the top of the new column to its right

### Requirement: Cases that snap
The drawn state SHALL jump to the target state, ending every running animation, on the first frame after attach, when the screen area in a layout differs from the previous one, and when the client terminal changes size. A band switch whose target is the band that the old band's removal made the viewed one SHALL jump too.

#### Scenario: Terminal resize
- **WHEN** the camera is scrolling and the client terminal is resized
- **THEN** the next frame draws the camera, the band and every tile at their targets

#### Scenario: Attach
- **WHEN** a client attaches to a session whose focused column needs the camera at 40
- **THEN** the first frame draws the camera at 40
