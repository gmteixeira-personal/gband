## MODIFIED Requirements

### Requirement: Per-client view
Each attached client SHALL hold its own view: the viewed band, the focused window, an active layer for each band and a set of minimized windows, as the floating-windows capability defines them, and a camera position for each band. The focused window SHALL be in the viewed band, tiled or floating, and SHALL NOT be a window this client has minimized. There SHALL be no focused window while the viewed band is empty, or while every window it holds is a floating window this client has minimized. View actions SHALL change only the view of the client that performs them, and SHALL NOT change the layout or the view of any other client. The one exception is `center_column` on the floating layer, which sends the placing of the focused floating window and so changes the layout, as "Center the focused column" defines.

#### Scenario: Two clients focus independently
- **WHEN** two clients view a band with columns A and B, both focus a window in B, and the first client focuses the column to the left
- **THEN** the first client focuses a window in A
- **AND** the second client still focuses the window in B

#### Scenario: Two clients in different layers
- **WHEN** two clients view a band with column A and floating window P3, both focus a window in A, and the first switches layers
- **THEN** the first client focuses P3
- **AND** the second client still focuses the window in A

#### Scenario: Two clients minimize independently
- **WHEN** two clients view a band with column A and floating window P3, both focus P3, and the first client runs `minimize_window`
- **THEN** the first client focuses a window in A and does not draw P3
- **AND** the second client still focuses and draws P3

### Requirement: Initial view
A client SHALL start by viewing the first band, with focus on the top window of its first column, every camera at strip position 0, and no minimized window.

#### Scenario: Attach to a session
- **WHEN** a client attaches to a session whose first band holds columns A and B
- **THEN** the client views the first band and focuses the top window of A

#### Scenario: Attach again after minimizing
- **WHEN** a client minimizes floating window P3 of the first band, detaches, and attaches again
- **THEN** the client draws P3

### Requirement: Switch band
Viewing the band below or above SHALL make the adjacent band in that direction the viewed band. Focus SHALL go to the window this client last focused in that band when it is still there and this client has not minimized it, in the layer that window is in. Otherwise focus SHALL go to the top window of its first column, or, when the band holds no column, to the top floating window of this client's stacking order, or to no window when the band holds no window that this client has not minimized. When no band exists in that direction, the view SHALL not change.

#### Scenario: Down to the empty band
- **WHEN** the layout holds B1 with windows and an empty B2, and a client viewing B1 views the band below
- **THEN** the client views B2 with no focused window

#### Scenario: Back up restores focus
- **WHEN** a client focuses the second column of B1, views the band below, then the band above
- **THEN** the client focuses the same window in the second column of B1

#### Scenario: Back up restores a floating focus
- **WHEN** a client focuses floating window P3 of B1, views the band below, then the band above
- **THEN** the client focuses P3 with the floating layer of B1 active

#### Scenario: Top band
- **WHEN** a client viewing the first band views the band above
- **THEN** the view is unchanged

#### Scenario: Back up past a minimized focus
- **WHEN** B1 holds column A with P1 and floating window P3, a client focuses P3, views the band below, minimizes P3 with `gband.window.minimize`, and views the band above
- **THEN** the client focuses P1 with the tiled layer of B1 active

#### Scenario: Down to a band of minimized windows
- **WHEN** the layout holds B1 with windows, B2 with only floating window P3, and an empty B3, and a client viewing B1 that has minimized P3 views the band below
- **THEN** the client views B2 with no focused window

### Requirement: Shown windows
A view's shown windows SHALL be the windows of its viewed band whose tile, as the layout capability's tile geometry gives it for the screen area, or whose box, as the floating-windows capability places it, has at least one cell inside the client's terminal. A floating window this client has minimized SHALL NOT be shown, wherever its box is. A tile's cells SHALL be placed at its strip position less the viewed band's camera position, from the terminal's top row. While the band's strip loops, as the Camera requirement defines, a tile's cells SHALL be placed at its drawn copy's position instead, and a tile with no drawn copy SHALL NOT be shown. Wherever the client-attach and animations capabilities draw a tile at its strip position less a camera, they SHALL use its drawn copy while the strip loops. A box's cells SHALL be placed at its column and row, from the terminal's top-left cell, whatever the camera. A tile that a box covers SHALL still be shown. A view of an empty band SHALL show no window.

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

#### Scenario: Minimized window not shown
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds one column of width 1/2 with P1 and floating window P4, and the client has minimized P4
- **THEN** the only shown window is P1

### Requirement: Focus a named window
Focusing a named window SHALL make that window's band the viewed band, focus that window, and make its layer active, as a focus message from the server does. When this client has minimized the window, focusing it SHALL restore it, as the floating-windows capability defines, and put it on top of this client's stacking order. The camera SHALL follow the new focus as it follows any change of focus. Focusing a window not in the layout SHALL leave the view unchanged.

#### Scenario: Window in the viewed band
- **WHEN** a client views a band holding columns A, B and C, focuses A, and focuses the named window in C
- **THEN** the client focuses that window
- **AND** focusing the column to the left then focuses B

#### Scenario: Window in another band
- **WHEN** a client views B1 and focuses a named window in B2
- **THEN** the client views B2 with that window focused

#### Scenario: Floating window by name
- **WHEN** a client focuses a tiled window of B1 and focuses floating window P3 of B1 by name
- **THEN** the client focuses P3 with the floating layer active

#### Scenario: Minimized window by name
- **WHEN** B2 holds floating windows P3 and P4 with overlapping boxes, a client focused P3 and then P4, minimized P3, views B1, and focuses P3 by name
- **THEN** the client views B2, focuses P3 with the floating layer active, and draws P3 over P4
