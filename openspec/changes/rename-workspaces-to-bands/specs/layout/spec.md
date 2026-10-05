## RENAMED Requirements

- FROM: `### Requirement: Dynamic workspaces`
- TO: `### Requirement: Dynamic bands`


## MODIFIED Requirements

### Requirement: Layout structure
A session's layout SHALL be an ordered list of bands, stacked top to bottom. Each band SHALL hold an ordered list of columns, left to right. Each column SHALL hold an ordered list of one or more panes, top to bottom, and a width. Every pane and every band SHALL have an identifier that is unique within the session and is never reused. A column SHALL never be empty: a column whose last pane leaves it SHALL be removed.

#### Scenario: Column removed with its last pane
- **WHEN** a band holds columns A, B and C, B holds one pane, and that pane closes
- **THEN** the band holds columns A and C, in that order

#### Scenario: Identifiers not reused
- **WHEN** pane 2 closes and a new pane opens
- **THEN** the new pane's identifier is not 2

### Requirement: Dynamic bands
The layout SHALL always hold at least one band, and its last band SHALL always be empty. When a pane is placed in the last band, a new empty band SHALL be added below it. When any band other than the last becomes empty, it SHALL be removed.

#### Scenario: Initial layout
- **WHEN** a session starts with its first pane
- **THEN** the layout holds two bands
- **AND** the first holds one column with that pane, and the second is empty

#### Scenario: Pane opened in the empty workspace
- **WHEN** the layout holds bands B1, with panes, and B2, empty, and a pane opens in B2
- **THEN** the layout holds B1, B2 with the new pane, and a new empty band B3

#### Scenario: Middle workspace emptied
- **WHEN** the layout holds B1, B2 and an empty B3, and the last pane of B1 closes
- **THEN** the layout holds B2 and B3, in that order

### Requirement: Open a pane
Opening a pane SHALL name a band and, optionally, a pane in that band. The new pane SHALL be placed alone in a new column. The new column SHALL be inserted immediately to the right of the named pane's column, or as the band's first column when no pane is named. No other column's width SHALL change.

#### Scenario: Open right of the focused column
- **WHEN** a band holds columns A and B, and a pane opens next to a pane in A
- **THEN** the band holds A, the new column and B, in that order
- **AND** A and B keep their widths

#### Scenario: Open in an empty workspace
- **WHEN** a pane opens in an empty band with no pane named
- **THEN** that band holds one column with the new pane

### Requirement: Close a pane
A pane SHALL leave the layout when its program exits, whether the program exited by itself or because the pane was closed. The pane's column and band SHALL then follow the column rule and the dynamic band rule.

#### Scenario: Pane leaves a stack
- **WHEN** a column holds panes P1, P2 and P3, and P2's program exits
- **THEN** the column holds P1 and P3, in that order, and keeps its width

### Requirement: Consume or expel
Consume or expel SHALL name a pane and a direction, left or right.

When the pane shares its column with other panes, it SHALL be expelled: it leaves its column and is placed alone in a new column of width 1/2, immediately to the left or right of its old column.

When the pane is alone in its column, it SHALL be consumed: it is appended at the bottom of the adjacent column in that direction, and its own column is removed. When no column exists in that direction, nothing SHALL change.

A pane that is consumed or expelled SHALL take an automatic height of weight 1 in the column it enters. The other panes of both columns SHALL keep their heights, except as "Pane heights" defines for a pane left alone in its column.

#### Scenario: Expel to the left
- **WHEN** a band holds columns A and B, B holds panes P1 and P2, and P2 is expelled to the left
- **THEN** the band holds A, a column with P2, and B with P1, in that order

#### Scenario: Consume to the right
- **WHEN** a band holds columns A with P1 and B with P2 and P3, and P1 is consumed to the right
- **THEN** the band holds one column with P2, P3 and P1, in that order, and that column keeps B's width

#### Scenario: Consume at the edge
- **WHEN** P1 is alone in the band's first column and is consumed to the left
- **THEN** the layout is unchanged

#### Scenario: Consume next to a fixed height
- **WHEN** the area is 80×24, column B holds P2 with a fixed height of 16 rows and P3 with an automatic height of weight 1, and P1, alone in column A, is consumed to the right
- **THEN** P2 keeps its fixed height of 16 rows, and P1 takes an automatic height of weight 1
- **AND** the tiles of P2, P3 and P1 are 16, 4 and 4 rows high

### Requirement: Tile geometry
For a screen area of a given width and height, the layout SHALL give every pane of a band a tile on that band's strip:

- A column's width in cells SHALL be the area's width when full width is on, and otherwise the area's width multiplied by its proportion and rounded down. It SHALL be at least 3 cells and at most 65535 cells.
- The first column SHALL start at strip position 0, and each later column SHALL start where the previous one ends.
- A column's tiles SHALL be stacked from the area's top row, in the column's pane order, with no gap between them.
- A pane with a fixed height SHALL get that many rows. When its column holds other panes, it SHALL get no more than the area's height less 3 rows for each other pane. When it is alone in its column, it SHALL get no more than the area's height, and the rows below its tile SHALL be covered by no tile.
- The panes with an automatic height SHALL share the rows the fixed pane leaves, in proportion to their weights. Each gets those rows multiplied by its weight and divided by the sum of the automatic weights, rounded down, and the first automatic panes, one for each row left over, get one row more. While the shared rows allow every automatic pane 3 rows, a share below 3 rows SHALL be raised: the topmost automatic pane whose share is below 3 rows takes 3 rows, and the other automatic panes share the rest by the same rule.
- Each tile SHALL have a one-cell border. A pane's terminal size SHALL be its tile less 2 columns and 2 rows, and at least 1 column by 1 row.

#### Scenario: Default column on an 80×24 area
- **WHEN** a column of width 1/2 holds one pane and the area is 80×24
- **THEN** the pane's tile is 40×24 at strip position 0
- **AND** the pane's terminal size is 38×22

#### Scenario: Columns side by side
- **WHEN** a band holds a column of width 1/3 and then a column of width 2/3, and the area is 90×30
- **THEN** the first tile spans strip positions 0 to 29 and the second spans 30 to 89

#### Scenario: Stack with a leftover row
- **WHEN** a column holds two panes with automatic heights of weight 1 and the area is 80×25
- **THEN** the top tile is 13 rows high and the bottom tile is 12 rows high

#### Scenario: Fixed pane above an automatic pane
- **WHEN** a column holds P1 with a fixed height of 16 rows above P2 with an automatic height, and the area is 80×24
- **THEN** P1's tile is 16 rows high and P2's tile is 8 rows high

#### Scenario: Automatic panes share by weight
- **WHEN** the area is 80×24 and a column holds P1 with an automatic height of weight 10/7, P2 with an automatic height of weight 1, and P3 with a fixed height of 9 rows
- **THEN** the tiles of P1, P2 and P3 are 9, 6 and 9 rows high

#### Scenario: Fixed height leaves room for the others
- **WHEN** the area is 80×24 and a column holds P1 with a fixed height of 30 rows above two panes with automatic heights
- **THEN** the tiles are 18, 3 and 3 rows high

#### Scenario: Automatic pane raised to 3 rows
- **WHEN** the area is 80×24 and a column holds P1 with an automatic height of weight 1/20 above P2 with an automatic height of weight 1
- **THEN** P1's tile is 3 rows high and P2's tile is 21 rows high

#### Scenario: Lone pane with a fixed height
- **WHEN** the area is 80×24 and a pane alone in its column has a fixed height of 20 rows
- **THEN** its tile is 20 rows high, and rows 20 to 23 of that column are covered by no tile

#### Scenario: Full width
- **WHEN** a column has full width on and the area is 100×30
- **THEN** its tile is 100 columns wide

#### Scenario: Column wider than the area
- **WHEN** a column has width 11/10 and the area is 80×24
- **THEN** its tile is 88 columns wide

#### Scenario: Column of width zero
- **WHEN** a column has width 0 and the area is 80×24
- **THEN** its tile is 3 columns wide

#### Scenario: Width beyond the cell limit
- **WHEN** a column has width 10000 and the area is 80×24
- **THEN** its tile is 65535 columns wide

### Requirement: Pane identity
A pane SHALL keep its identifier from the moment it opens until it closes. Consuming or expelling it, changing its column's width, and adding or removing other columns and bands SHALL NOT change its identifier.

#### Scenario: Identifier survives moves
- **WHEN** a band holds panes 1 and 2 in separate columns, and pane 2 is consumed into pane 1's column, expelled to the right again, and its column's width is cycled
- **THEN** after every step the layout holds exactly panes 1 and 2
- **AND** pane 2 sits where the step placed it
