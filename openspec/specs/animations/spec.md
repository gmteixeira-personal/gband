# animations Specification

## Purpose

Defines how the client moves its drawing from one state of the layout and view to the next: the camera scroll, the band switch, tile movement and the resize morph, the motion they share, the cases that do not animate, and how to turn animations off.

## Requirements

### Requirement: Animated presentation
The client SHALL separate what it draws from what the layout and its view decide. The layout, the tile geometry, the focused pane, the viewed band and each camera SHALL change at once, as the layout, layout-view and client-attach capabilities define. Only the drawn camera, the drawn band and each tile's drawn position and size SHALL move toward those targets over time. Key input, pastes and session actions SHALL go to the newly focused pane from the moment focus changes, whether or not an animation is running. When no animation is running, the drawn state SHALL equal the target state.

#### Scenario: Typing during a scroll
- **WHEN** the client focuses the column to the right, the camera starts to scroll, and the user types `echo right` and Enter before the scroll ends
- **THEN** `right` appears in the newly focused pane only

#### Scenario: At rest
- **WHEN** no layout or view change has happened for 400 ms
- **THEN** the client draws exactly what it would draw with animations off

### Requirement: Motion
Every animated quantity SHALL move from its drawn value toward its target as a critically damped spring with a stiffness of 800 and a damping ratio of 1, the defaults niri uses. A quantity at rest SHALL move toward its target without passing it. The drawn value SHALL be rounded to the nearest whole cell. A quantity SHALL reach its target exactly no later than 400 ms after its animation starts or after it was last retargeted, and SHALL then be at rest. When a target changes while the quantity is moving, the animation SHALL continue from the quantity's current drawn value and velocity, without a jump.

#### Scenario: Reaches the target
- **WHEN** a quantity at rest at 0 is given the target 40
- **THEN** it is drawn at 0 at the instant the animation starts
- **AND** it is drawn at a value strictly between 0 and 40 50 ms later
- **AND** it is drawn at 40 and is at rest 400 ms later

#### Scenario: Never passes the target from rest
- **WHEN** a quantity at rest at 0 is given the target 40
- **THEN** every drawn value until it is at rest lies between 0 and 40, and no value is smaller than one drawn before it

#### Scenario: Retarget mid-flight
- **WHEN** a quantity moving from 0 toward 40 is drawn at 20, and its target changes to 0
- **THEN** the next frame draws it within one cell of where it would have been drawn without the change
- **AND** it later reaches 0

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

### Requirement: Resize morph
When a pane's tile width or height changes, the client SHALL animate the tile's drawn width and height from its drawn size to its new size. The tile's border SHALL be drawn at the drawn size. The pane's grid SHALL be drawn from the inside of the tile's top-left corner and cut at the drawn border. Cells inside the border that the grid does not cover SHALL be blank. The morph SHALL NOT change when or how the server resizes the pane.

#### Scenario: Cycle a width
- **WHEN** the focused column is 40 cells wide on an 80×24 area and its width is cycled to 2/3
- **THEN** the first frame draws its tile 40 cells wide
- **AND** later frames draw it wider, with the right border moving, until it is 53 cells wide

#### Scenario: Shrinking cuts the grid
- **WHEN** a tile shrinks from 53 to 26 cells wide while the pane's grid is still 51 columns wide
- **THEN** each frame shows only the grid's columns that fit inside the drawn border

### Requirement: Cases that snap
The drawn state SHALL jump to the target state, ending every running animation, on the first frame after attach, when the screen area in a layout differs from the previous one, and when the client terminal changes size. A band switch whose target is the band that the old band's removal made the viewed one SHALL jump too.

#### Scenario: Terminal resize
- **WHEN** the camera is scrolling and the client terminal is resized
- **THEN** the next frame draws the camera, the band and every tile at their targets

#### Scenario: Attach
- **WHEN** a client attaches to a session whose focused column needs the camera at 40
- **THEN** the first frame draws the camera at 40

### Requirement: Frames while animating
While any animation runs, the client SHALL draw a frame about every 16 ms. When every animation has reached its target, the client SHALL stop drawing frames on a timer and SHALL draw only in response to messages from the server and to input.

#### Scenario: Idle client
- **WHEN** no animation is running and nothing is received from the server or the terminal
- **THEN** the client draws no frame

### Requirement: Cursor during motion
While the drawn camera, the drawn vertical position, or the focused tile's drawn position or size differs from its target, the terminal's cursor SHALL be hidden. When they all reach their targets, the cursor SHALL follow the client-attach capability's rules again.

#### Scenario: Cursor returns after a scroll
- **WHEN** focus moves to a column that needs the camera to scroll
- **THEN** the cursor is hidden while the camera moves
- **AND** it sits at the focused pane's cursor once the camera reaches its target

### Requirement: Turning animations off
The client SHALL read the environment variable `GBAND_ANIMATIONS` when it starts. When the value is `off`, every change SHALL snap as if every case snapped, and the client SHALL draw exactly as the client-attach capability defines without animations. When the variable is unset or `on`, animations SHALL be on. Any other value SHALL be logged as a warning in the client's log, and animations SHALL be on.

#### Scenario: Animations off
- **WHEN** a client starts with `GBAND_ANIMATIONS=off` and focus moves to a column that needs the camera to scroll
- **THEN** the next frame draws the camera at its new position

#### Scenario: Unknown value
- **WHEN** a client starts with `GBAND_ANIMATIONS=fast`
- **THEN** the client's log holds a warning naming the value
- **AND** animations are on
