## Purpose

Defines one client's view of the session's layout: which workspace it shows, which pane has its focus, how focus moves and follows layout changes, and where its camera sits on the workspace's strip.

## ADDED Requirements

### Requirement: Per-client view
Each attached client SHALL hold its own view: the viewed workspace, the focused pane, and a camera position for each workspace. The focused pane SHALL be in the viewed workspace, and there SHALL be no focused pane while the viewed workspace is empty. View actions SHALL change only the view of the client that performs them, and SHALL NOT change the layout or the view of any other client.

#### Scenario: Two clients focus independently
- **WHEN** two clients view a workspace with columns A and B, both focus a pane in B, and the first client focuses the column to the left
- **THEN** the first client focuses a pane in A
- **AND** the second client still focuses the pane in B

### Requirement: Initial view
A client SHALL start by viewing the first workspace, with focus on the top pane of its first column and every camera at strip position 0.

#### Scenario: Attach to a session
- **WHEN** a client attaches to a session whose first workspace holds columns A and B
- **THEN** the client views the first workspace and focuses the top pane of A

### Requirement: Focus across columns
Focusing the column to the left or right SHALL move focus to the adjacent column in that direction. Within that column, focus SHALL go to the pane this client focused most recently, or to the top pane when this client has focused none of its panes. When no column exists in that direction, the view SHALL not change.

#### Scenario: Return to the remembered pane
- **WHEN** column A holds P1 and P2, the client focuses P2, focuses the column to the right, then the column to the left
- **THEN** the client focuses P2

#### Scenario: No column to the right
- **WHEN** the client focuses a pane in the last column and focuses the column to the right
- **THEN** the focus is unchanged

### Requirement: Focus within a column
Focusing the pane below or above SHALL move focus to the adjacent pane in that direction in the focused column. When no pane exists in that direction, the view SHALL not change.

#### Scenario: Move down a stack
- **WHEN** a column holds P1, P2 and P3, and the client focuses P1 and then the pane below twice
- **THEN** the client focuses P3

#### Scenario: Bottom of the stack
- **WHEN** the client focuses the bottom pane of a column and focuses the pane below
- **THEN** the focus is unchanged

### Requirement: Switch workspace
Viewing the workspace below or above SHALL make the adjacent workspace in that direction the viewed workspace. Focus SHALL go to the pane this client last focused in that workspace when it is still there, otherwise to the top pane of its first column, or to no pane when the workspace is empty. When no workspace exists in that direction, the view SHALL not change.

#### Scenario: Down to the empty workspace
- **WHEN** the layout holds W1 with panes and an empty W2, and a client viewing W1 views the workspace below
- **THEN** the client views W2 with no focused pane

#### Scenario: Back up restores focus
- **WHEN** a client focuses the second column of W1, views the workspace below, then the workspace above
- **THEN** the client focuses the same pane in the second column of W1

#### Scenario: Top workspace
- **WHEN** a client viewing the first workspace views the workspace above
- **THEN** the view is unchanged

### Requirement: Follow layout changes
When the layout changes, each client's view SHALL follow it:

- When the viewed workspace is removed, the client SHALL view the workspace that now holds the removed one's position, or the last workspace when none does.
- When the focused pane moves within the viewed workspace, focus SHALL stay on that pane.
- When the focused pane leaves the layout, focus SHALL go to the column at the same position, or to the last column when none is there. Within that column, focus SHALL go to the pane at the same position from the top, or to the bottom pane when none is there.
- When the server tells the client to focus a pane, the client SHALL view that pane's workspace and focus it.

#### Scenario: Focused column closed
- **WHEN** a client focuses column B in a workspace holding A, B and C, and B's only pane exits
- **THEN** the client focuses the pane in C

#### Scenario: Last column closed
- **WHEN** a client focuses column C, the last of A, B and C, and C's only pane exits
- **THEN** the client focuses the pane in B

#### Scenario: Focused pane expelled
- **WHEN** a client focuses P2 in a column holding P1 and P2, and P2 is expelled to the right
- **THEN** the client still focuses P2, now alone in its new column

#### Scenario: Viewed workspace removed
- **WHEN** a client views W1 of W1, W2 and an empty W3, and the last pane of W1 exits
- **THEN** the client views W2

### Requirement: Camera
Each workspace's camera SHALL be the strip position shown at the client terminal's left edge. After every change of focus, of the layout or of the client's terminal width, the camera of the viewed workspace SHALL move as follows, where the view spans from the camera to the camera plus the terminal's width:

- When the focused pane's column lies wholly inside the view, the camera SHALL not move.
- Otherwise, when the column is at least as wide as the terminal or starts left of the view, the camera SHALL move to the column's start.
- Otherwise, the camera SHALL move so that the column's end meets the terminal's right edge.

#### Scenario: Scroll right just enough
- **WHEN** the terminal is 80 columns wide, a workspace holds three columns of 40 cells at strip positions 0, 40 and 80, the camera is at 0 and focus moves from the second column to the third
- **THEN** the camera moves to 40

#### Scenario: Scroll back left
- **WHEN** the camera is at 40 in that workspace and focus moves to the first column
- **THEN** the camera moves to 0

#### Scenario: Visible column keeps the camera
- **WHEN** the camera is at 40 in that workspace and focus moves from the third column to the second
- **THEN** the camera stays at 40

#### Scenario: Column wider than the terminal
- **WHEN** the focused column starts at strip position 40 and is wider than the terminal
- **THEN** the camera moves to 40
