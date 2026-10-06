## ADDED Requirements

### Requirement: Center the focused column
While the tiled layer is active, the `center_column` view action SHALL move the viewed band's camera so that the focused window's column sits in the middle of the client's terminal, as niri's `center-column` does. The camera SHALL move to the column's start less half the difference between the terminal's width and the column's width, rounded down. When the column is at least as wide as the terminal, the camera SHALL move to the column's start. The action SHALL change neither the focused window nor the viewed band, and SHALL send nothing to the server. On a band with no focused window, it SHALL leave the view unchanged.

While the floating layer is active, the action SHALL instead centre the focused floating window in the session's screen area, as niri's `center-column` does for a floating window. The client SHALL send the server the placing of that window, as the floating-windows capability defines, at a column of half the difference between the area's width and the box's width, and a row of half the difference between the area's height and the box's height, each rounded down, where the box is as placed in the current screen area. The client SHALL send nothing when the box already sits there. The action SHALL change neither the camera, the focused window nor the viewed band.

After the action, every later change of focus, of the layout or of the client's terminal width SHALL move the camera by the `center_focused_column` policy, as the Camera requirement defines, starting from the centred camera.

#### Scenario: Center a column at the right edge
- **WHEN** the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the third is focused with the camera at 40, and the client runs `center_column`
- **THEN** the camera moves to 60
- **AND** the third column is still focused

#### Scenario: Center the first column
- **WHEN** the terminal is 80 columns wide, the focused column is 40 cells wide at strip position 0, the camera is at 0, and the client runs `center_column`
- **THEN** the camera moves to -20 and the 20 cells left of the column are drawn empty

#### Scenario: Column wider than the terminal
- **WHEN** the terminal is 80 columns wide, the focused column is 100 cells wide at strip position 40, and the client runs `center_column`
- **THEN** the camera moves to 40

#### Scenario: Centred column keeps the camera
- **WHEN** the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the second is focused, the client runs `center_column`, and the terminal then narrows to 78 columns
- **THEN** the camera stays at 20

#### Scenario: Policy resumes on the next focus change
- **WHEN** the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells at strip positions 0, 40 and 80, the first is focused, the client runs `center_column`, and focus then moves to the third column
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

## MODIFIED Requirements

### Requirement: Per-client view
Each attached client SHALL hold its own view: the viewed band, the focused window, an active layer for each band as the floating-windows capability defines, and a camera position for each band. The focused window SHALL be in the viewed band, tiled or floating, and there SHALL be no focused window while the viewed band is empty. View actions SHALL change only the view of the client that performs them, and SHALL NOT change the layout or the view of any other client. The one exception is `center_column` on the floating layer, which sends the placing of the focused floating window and so changes the layout, as "Center the focused column" defines.

#### Scenario: Two clients focus independently
- **WHEN** two clients view a band with columns A and B, both focus a window in B, and the first client focuses the column to the left
- **THEN** the first client focuses a window in A
- **AND** the second client still focuses the window in B

#### Scenario: Two clients in different layers
- **WHEN** two clients view a band with column A and floating window P3, both focus a window in A, and the first switches layers
- **THEN** the first client focuses P3
- **AND** the second client still focuses the window in A
