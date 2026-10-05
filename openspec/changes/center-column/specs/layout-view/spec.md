## ADDED Requirements

### Requirement: Center the focused column
The `center_column` view action SHALL move the viewed band's camera so that the focused pane's column sits in the middle of the client's terminal, as niri's `center-column` does. The camera SHALL move to the column's start less half the difference between the terminal's width and the column's width, rounded down. When the column is at least as wide as the terminal, the camera SHALL move to the column's start. The action SHALL change neither the focused pane nor the viewed band, and SHALL send nothing to the server. On a band with no focused pane, it SHALL leave the view unchanged.

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

#### Scenario: Empty band
- **WHEN** the client views an empty band with its camera at 0 and runs `center_column`
- **THEN** the camera stays at 0
