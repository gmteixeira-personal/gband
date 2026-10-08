# animations Specification

## Purpose

Defines how the client moves its drawing from one state of the layout and view to the next: the camera scroll, the band switch, tile movement and the resize morph, the motion they share, the cases that do not animate, and how to turn animations off.

## Requirements

### Requirement: Animated presentation
The client SHALL separate what it draws from what the layout and its view decide. The layout, the tile geometry, the focused window, the viewed band and each camera SHALL change at once, as the layout, layout-view and client-attach capabilities define. Only the drawn camera, the drawn band and each tile's drawn position and size SHALL move toward those targets over time. Key input, pastes and session actions SHALL go to the newly focused window from the moment focus changes, whether or not an animation is running. When no animation is running, the drawn state SHALL equal the target state.

#### Scenario: Typing during a scroll
- **WHEN** the client focuses the column to the right, the camera starts to scroll, and the user types `echo right` and Enter before the scroll ends
- **THEN** `right` appears in the newly focused window only

#### Scenario: At rest
- **WHEN** no layout or view change has happened for `400 / s` ms, where `s` is the value of `animation_speed`
- **THEN** the client draws exactly what it would draw with animations off

### Requirement: Motion
Every animated quantity SHALL move from its drawn value toward its target as a critically damped spring with a stiffness of `800 × s²` and a damping ratio of 1, where `s` is the value of the client option `animation_speed`, as the configuration capability defines. At the default speed of 1 the stiffness is 800, the default niri uses. A quantity at rest SHALL move toward its target without passing it. The drawn value SHALL be rounded to the nearest whole cell. A quantity SHALL reach its target exactly no later than `400 / s` ms after its animation starts, after it was last retargeted, or after the speed last changed, and SHALL then be at rest. When a target changes while the quantity is moving, the animation SHALL continue from the quantity's current drawn value and velocity, without a jump. When a reload changes `animation_speed` while the quantity is moving, the animation SHALL continue from the quantity's current drawn value and velocity at the new speed, without a jump.

#### Scenario: Reaches the target
- **WHEN** `animation_speed` is 1 and a quantity at rest at 0 is given the target 40
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

#### Scenario: Twice as fast
- **WHEN** `animation_speed` is 2 and a quantity at rest at 0 is given the target 40
- **THEN** it is drawn at a value strictly between 0 and 40 25 ms later
- **AND** it is drawn at 40 and is at rest 200 ms later

#### Scenario: Half as fast
- **WHEN** `animation_speed` is 0.5 and a quantity at rest at 0 is given the target 40
- **THEN** it is not at rest 200 ms later
- **AND** it is drawn at 40 and is at rest 800 ms later

#### Scenario: Speed changed mid-flight
- **WHEN** a quantity moving from 0 toward 40 at speed 1 is drawn at 20, and a reload sets `animation_speed` to 2
- **THEN** at that instant it is drawn at the same value, with the same velocity
- **AND** it is drawn at 40 and is at rest no later than 200 ms after the reload

### Requirement: Camera scroll
When the camera of the viewed band changes, the client SHALL animate the drawn camera from its drawn position to the new camera position. Tiles SHALL be drawn at their strip position less the drawn camera. While the band's strip loops, as the layout-view capability defines, the drawn camera SHALL scroll in the direction and over the distance that the camera moved before it was held within the strip, and each tile SHALL be drawn at its drawn copy. Holding the camera within the strip SHALL NOT make the drawn camera jump or scroll the other way.

#### Scenario: Scroll right
- **WHEN** the terminal is 80 columns wide, a band holds three columns of 40 cells, the camera is at 0 at rest, and focus moves from the second column to the third
- **THEN** the first frame draws the camera at 0
- **AND** later frames draw it between 0 and 40 until it reaches 40

#### Scenario: Scroll reversed before it ends
- **WHEN** the camera is scrolling from 0 toward 40 and focus moves back to the first column
- **THEN** the drawn camera turns back without a jump and reaches 0

#### Scenario: Scroll right across the seam
- **WHEN** `loop_bands` is on, the terminal is 80 columns wide, a band holds three columns of 40 cells, the first is focused with the camera at 80 at rest, and focus moves to the column to the right
- **THEN** the camera moves to 120 and is held at 0
- **AND** every frame shows the strip moving left, with the second column entering from the right edge, until the first column is in cells 0 to 39 and the second in cells 40 to 79

#### Scenario: Scroll left across the seam
- **WHEN** `loop_bands` is on, the terminal is 80 columns wide, a band holds three columns of 40 cells, the first is focused with the camera at 0 at rest, and focus moves to the column to the left
- **THEN** every frame shows the strip moving right, with the third column entering from the left edge, until the third column is in cells 0 to 39 and the first in cells 40 to 79

### Requirement: Band switch
When the viewed band changes to another band that is still in the layout, the client SHALL animate a vertical position from the old band to the new one. Band `i` in the layout SHALL be drawn in a region as tall as the client terminal, starting at row `i` times the terminal's height less the drawn vertical position. Each band's tiles SHALL be cut at the edges of its region. The band being left SHALL be drawn with the camera it was drawn with when the switch started. A switch made while another is running SHALL continue from the drawn vertical position. When bands are added or removed above the viewed one during a switch, the drawn vertical position SHALL shift with them, so that no band jumps on screen.

#### Scenario: Switch down
- **WHEN** the terminal is 80×24, the client views B1 at rest and views the band below, B2
- **THEN** the first frame shows B1 only
- **AND** a later frame shows the bottom part of B1 above the top part of B2
- **AND** within `400 / s` ms, where `s` is the value of `animation_speed`, only B2 is shown

#### Scenario: Two switches in a row
- **WHEN** the client views B1 of B1, B2 and B3, views the band below, and views the band below again before the first slide ends
- **THEN** the slide continues downward without a jump and ends showing B3

#### Scenario: Old band removed
- **WHEN** the client switches from B1 to B2 and the last window of B1 exits before the slide ends
- **THEN** the client shows B2 at rest from the next frame

### Requirement: Tile movement
When the layout changes, each window of the viewed band that it held both before and after the change SHALL have its drawn strip position and its drawn row animated from where it was drawn to its new tile position. A window new to the band SHALL be drawn at its tile position from the first frame. A window that left the band SHALL no longer be drawn. Where tiles overlap while they move, the focused tile SHALL be drawn over the others, and the others SHALL be drawn in layout order. Windows of bands that are not drawn SHALL NOT animate.

#### Scenario: Open a window between two columns
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
- **AND** it sits at the focused window's cursor once the camera reaches its target

### Requirement: Turning animations off
The client SHALL animate only while the client option `animations` is `true` and the environment variable `GBAND_ANIMATIONS` does not turn animations off. While animations are off, every change SHALL snap as if every case snapped, and the client SHALL draw exactly as the client-attach capability defines without animations.

The client SHALL read `GBAND_ANIMATIONS` when it starts. When its value is `off`, animations SHALL be off whatever `animations` holds. When it is unset or `on`, the option SHALL decide. Any other value SHALL be logged as a warning in the client's log and read as unset.

A reload that turns animations off SHALL draw every animated quantity at its target from the next frame the client draws. A reload that turns them on SHALL animate the next change, and SHALL NOT move anything that is already at its target.

#### Scenario: Animations off
- **WHEN** a client starts with `GBAND_ANIMATIONS=off` and focus moves to a column that needs the camera to scroll
- **THEN** the next frame draws the camera at its new position

#### Scenario: Environment overrides the option
- **WHEN** a client starts with `GBAND_ANIMATIONS=off`, `user/init.lua` sets `gband.opt.animations = true`, and focus moves to a column that needs the camera to scroll
- **THEN** the next frame draws the camera at its new position

#### Scenario: Animations off by the option
- **WHEN** a client starts with `GBAND_ANIMATIONS` unset, `user/init.lua` sets `gband.opt.animations = false`, and focus moves to a column that needs the camera to scroll
- **THEN** the next frame draws the camera at its new position

#### Scenario: On by default
- **WHEN** a client starts with `GBAND_ANIMATIONS` unset and the default configuration, and focus moves to a column that needs the camera to scroll
- **THEN** the camera scrolls over several frames

#### Scenario: Unknown value
- **WHEN** a client starts with `GBAND_ANIMATIONS=fast` and `user/init.lua` sets `gband.opt.animations = false`
- **THEN** the client's log holds a warning naming the value
- **AND** animations are off

#### Scenario: Turned off by a reload mid-flight
- **WHEN** the camera is scrolling and a reload sets `gband.opt.animations = false`
- **THEN** the next frame the client draws shows the camera at its target

### Requirement: Floating windows at rest
The client SHALL draw every floating window that it has not minimized at its box, as the client-attach capability places it, in every frame, without animating its position or size. A window that the client minimizes SHALL NOT be drawn from the first frame after it is minimized. A window that the client restores SHALL be drawn at its box from the first frame after it is restored. A window that moves from a column to the floating layer SHALL be drawn at its box from the first frame. A window that moves from the floating layer to a column SHALL be drawn at its tile position from the first frame, as a window new to the band is. The tiles of the other windows SHALL move as "Tile movement" defines.

#### Scenario: Float a window between two columns
- **WHEN** the viewed band holds columns A, B and C, and B's only window is floated
- **THEN** the first frame draws that window at its box
- **AND** C's tile slides left from B's end to where B started

#### Scenario: Move a floating window
- **WHEN** a floating window is moved right by one step
- **THEN** the next frame draws its box at the new column

#### Scenario: Minimize and restore at once
- **WHEN** animations are on, the viewed band holds column A with P1 and floating window P3, the client focuses P3 and minimizes it
- **THEN** the next frame draws no part of P3, and A's tile does not move
- **AND** when the client focuses P3 again, the next frame draws P3 at its box

### Requirement: Drawing during a gesture
While a drag gesture runs, as the mouse capability defines, the client SHALL draw the lifted tile, the floating box or floating plugin window being moved or resized, the slid camera, and the bands at the drawn vertical position a vertical band drag sets, at the positions the gesture sets, with no animation. A layout that a gesture's own changes cause SHALL draw the gesture's window at its target at once, and SHALL animate the other windows as any layout change does.

When a lifted tile drops, its tile SHALL move from where it was drawn at the release to its new place, as tile movement animates. When the release of a band slide moves the camera, the camera SHALL scroll as camera scroll animates. When a vertical band drag is released, the drawn vertical position SHALL move from where the drag left it to the viewed band's region, as the band switch animates, whether or not the release changed the viewed band. Every band drawn at the release SHALL keep the camera it was drawn with until it leaves the terminal.

#### Scenario: Floating box follows the pointer
- **WHEN** animations are on and the user drags a floating window 5 cells right with `drag_window`
- **THEN** the next frame draws its box 5 cells right of where it was, with no frame in between

#### Scenario: Drop animates
- **WHEN** animations are on and the user drops a lifted tile into a new column
- **THEN** the dropped tile moves from the release position to its new place over the following frames

#### Scenario: Bands follow the pointer
- **WHEN** animations are on, the terminal is 80×24, the client views B1 of B1, B2 and an empty B3, and the user drags with `drag_band` 6 rows up
- **THEN** the next frame draws B1 from row -6 and B2 from row 18, with no frame in between

#### Scenario: Release continues the switch
- **WHEN** animations are on, the terminal is 80×24, the client views B1 of B1, B2 and an empty B3, and the user drags with `drag_band` 16 rows up and releases
- **THEN** the first frame after the release draws B2 from row 8
- **AND** later frames draw B2 higher until it starts at row 0, with B1 keeping the camera it was drawn with

#### Scenario: Release slides back
- **WHEN** animations are on, the terminal is 80×24, the client views B1 of B1, B2 and an empty B3, and the user drags with `drag_band` 6 rows up and releases
- **THEN** later frames draw B1 lower until it starts at row 0 and B2 is no longer drawn

#### Scenario: Release with animations off
- **WHEN** animations are off and the user releases a vertical band drag that moved the bands 16 rows up
- **THEN** the next frame shows only the newly viewed band
