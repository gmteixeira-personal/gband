## Purpose

Defines the session's shared layout: workspaces, the strip of columns in each, the panes stacked in each column, column widths, the session actions that change them, and the size of every pane's tile for a given screen area.

## ADDED Requirements

### Requirement: Layout structure
A session's layout SHALL be an ordered list of workspaces, stacked top to bottom. Each workspace SHALL hold an ordered list of columns, left to right. Each column SHALL hold an ordered list of one or more panes, top to bottom, and a width. Every pane and every workspace SHALL have an identifier that is unique within the session and is never reused. A column SHALL never be empty: a column whose last pane leaves it SHALL be removed.

#### Scenario: Column removed with its last pane
- **WHEN** a workspace holds columns A, B and C, B holds one pane, and that pane closes
- **THEN** the workspace holds columns A and C, in that order

#### Scenario: Identifiers not reused
- **WHEN** pane 2 closes and a new pane opens
- **THEN** the new pane's identifier is not 2

### Requirement: Dynamic workspaces
The layout SHALL always hold at least one workspace, and its last workspace SHALL always be empty. When a pane is placed in the last workspace, a new empty workspace SHALL be added below it. When any workspace other than the last becomes empty, it SHALL be removed.

#### Scenario: Initial layout
- **WHEN** a session starts with its first pane
- **THEN** the layout holds two workspaces
- **AND** the first holds one column with that pane, and the second is empty

#### Scenario: Pane opened in the empty workspace
- **WHEN** the layout holds workspaces W1, with panes, and W2, empty, and a pane opens in W2
- **THEN** the layout holds W1, W2 with the new pane, and a new empty workspace W3

#### Scenario: Middle workspace emptied
- **WHEN** the layout holds W1, W2 and an empty W3, and the last pane of W1 closes
- **THEN** the layout holds W2 and W3, in that order

### Requirement: Column widths
A column's width SHALL be a proportion of the screen area's width, and a column SHALL also carry a full-width flag. The width presets SHALL be 1/3, 1/2 and 2/3. A new column SHALL have width 1/2 and full width off.

Cycling a column's width SHALL turn full width off and set the width to the smallest preset larger than the current width, or to 1/3 when no preset is larger. A column with full width on SHALL count as width 1 while cycling. Toggling full width SHALL flip the flag and keep the proportion.

#### Scenario: Cycle from the default
- **WHEN** a column of width 1/2 is cycled three times
- **THEN** its width is 2/3, then 1/3, then 1/2

#### Scenario: Cycle from full width
- **WHEN** a column of width 2/3 has full width on and is cycled
- **THEN** its full width is off and its width is 1/3

#### Scenario: Toggle full width keeps the proportion
- **WHEN** a column of width 1/3 has full width toggled twice
- **THEN** its full width is off and its width is 1/3

### Requirement: Open a pane
Opening a pane SHALL name a workspace and, optionally, a pane in that workspace. The new pane SHALL be placed alone in a new column. The new column SHALL be inserted immediately to the right of the named pane's column, or as the workspace's first column when no pane is named. No other column's width SHALL change.

#### Scenario: Open right of the focused column
- **WHEN** a workspace holds columns A and B, and a pane opens next to a pane in A
- **THEN** the workspace holds A, the new column and B, in that order
- **AND** A and B keep their widths

#### Scenario: Open in an empty workspace
- **WHEN** a pane opens in an empty workspace with no pane named
- **THEN** that workspace holds one column with the new pane

### Requirement: Close a pane
A pane SHALL leave the layout when its program exits, whether the program exited by itself or because the pane was closed. The pane's column and workspace SHALL then follow the column rule and the dynamic workspace rule.

#### Scenario: Pane leaves a stack
- **WHEN** a column holds panes P1, P2 and P3, and P2's program exits
- **THEN** the column holds P1 and P3, in that order, and keeps its width

### Requirement: Consume or expel
Consume or expel SHALL name a pane and a direction, left or right.

When the pane shares its column with other panes, it SHALL be expelled: it leaves its column and is placed alone in a new column of width 1/2, immediately to the left or right of its old column.

When the pane is alone in its column, it SHALL be consumed: it is appended at the bottom of the adjacent column in that direction, and its own column is removed. When no column exists in that direction, nothing SHALL change.

#### Scenario: Expel to the left
- **WHEN** a workspace holds columns A and B, B holds panes P1 and P2, and P2 is expelled to the left
- **THEN** the workspace holds A, a column with P2, and B with P1, in that order

#### Scenario: Consume to the right
- **WHEN** a workspace holds columns A with P1 and B with P2 and P3, and P1 is consumed to the right
- **THEN** the workspace holds one column with P2, P3 and P1, in that order, and that column keeps B's width

#### Scenario: Consume at the edge
- **WHEN** P1 is alone in the workspace's first column and is consumed to the left
- **THEN** the layout is unchanged

### Requirement: Tile geometry
For a screen area of a given width and height, the layout SHALL give every pane of a workspace a tile on that workspace's strip:

- A column's width in cells SHALL be the area's width when full width is on, and otherwise the area's width multiplied by its proportion and rounded down. It SHALL be at least 3 cells.
- The first column SHALL start at strip position 0, and each later column SHALL start where the previous one ends.
- A column's height SHALL be divided among its panes from the top. Each pane gets the area's height divided by the number of panes, rounded down. The first panes, one for each row left over, get one row more.
- Each tile SHALL have a one-cell border. A pane's terminal size SHALL be its tile less 2 columns and 2 rows, and at least 1 column by 1 row.

#### Scenario: Default column on an 80×24 area
- **WHEN** a column of width 1/2 holds one pane and the area is 80×24
- **THEN** the pane's tile is 40×24 at strip position 0
- **AND** the pane's terminal size is 38×22

#### Scenario: Columns side by side
- **WHEN** a workspace holds a column of width 1/3 and then a column of width 2/3, and the area is 90×30
- **THEN** the first tile spans strip positions 0 to 29 and the second spans 30 to 89

#### Scenario: Stack with a leftover row
- **WHEN** a column holds two panes and the area is 80×25
- **THEN** the top tile is 13 rows high and the bottom tile is 12 rows high

#### Scenario: Full width
- **WHEN** a column has full width on and the area is 100×30
- **THEN** its tile is 100 columns wide
