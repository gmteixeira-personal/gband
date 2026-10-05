## ADDED Requirements

### Requirement: Shown panes
A view's shown panes SHALL be the panes of its viewed workspace whose tile, as the layout capability's tile geometry gives it for the screen area, has at least one cell inside the client's terminal. A tile's cells SHALL be placed at its strip position less the viewed workspace's camera position, from the terminal's top row. A view of an empty workspace SHALL show no pane.

#### Scenario: Column beyond the right edge
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed workspace holds three columns of width 1/2, and the camera is at strip position 0
- **THEN** the shown panes are those of the first two columns

#### Scenario: Partly visible column
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed workspace holds two columns of width 2/3, and the client focuses the second
- **THEN** the panes of both columns are shown

#### Scenario: Pane below the bottom edge
- **WHEN** the screen area is 120×60, the client's terminal is 100×30, and the viewed workspace holds one column of width 1/2 with two panes with automatic heights of weight 1
- **THEN** only the top pane is shown

#### Scenario: Other workspaces
- **WHEN** the layout holds W1 with pane P1 and W2 with pane P2, and the client views W2
- **THEN** the only shown pane is P2
