## MODIFIED Requirements

### Requirement: Column widths
A column's width SHALL be a proportion of the screen area's width, and a column SHALL also carry a full-width flag. The width presets SHALL be 1/3, 1/2 and 2/3. A new column SHALL have width 1/2 and full width off.

Cycling a column's width SHALL turn full width off and set the width to the smallest preset larger than the current width, or to 1/3 when no preset is larger. A column with full width on SHALL count as width 1 while cycling. Toggling full width SHALL flip the flag and keep the proportion.

Growing or shrinking a column's width SHALL turn full width off and add or subtract 1/10 to the current width, keeping the result between 0 and 10000. A width above 1 SHALL make the column wider than the screen area. A column with full width on SHALL count as width 1 while growing or shrinking. A width SHALL always be held in lowest terms. When the result equals the current width and full width was already off, nothing SHALL change.

#### Scenario: Cycle from the default
- **WHEN** a column of width 1/2 is cycled three times
- **THEN** its width is 2/3, then 1/3, then 1/2

#### Scenario: Cycle from full width
- **WHEN** a column of width 2/3 has full width on and is cycled
- **THEN** its full width is off and its width is 1/3

#### Scenario: Toggle full width keeps the proportion
- **WHEN** a column of width 1/3 has full width toggled twice
- **THEN** its full width is off and its width is 1/3

#### Scenario: Grow from the default
- **WHEN** a column of width 1/2 is grown twice
- **THEN** its width is 3/5, then 7/10

#### Scenario: Grow from a preset of thirds
- **WHEN** a column of width 1/3 is grown
- **THEN** its width is 13/30

#### Scenario: Shrink to zero
- **WHEN** a column of width 1/2 is shrunk five times
- **THEN** its width is 2/5, then 3/10, then 1/5, then 1/10, then 0

#### Scenario: Shrink at zero
- **WHEN** a column of width 0 is shrunk
- **THEN** the layout is unchanged

#### Scenario: Grow past the screen width
- **WHEN** a column of width 1 with full width off is grown
- **THEN** its width is 11/10

#### Scenario: Grow at the limit
- **WHEN** a column of width 10000 is grown
- **THEN** the layout is unchanged

#### Scenario: Grow from full width
- **WHEN** a column of width 1/3 has full width on and is grown
- **THEN** its full width is off and its width is 11/10

#### Scenario: Shrink from full width
- **WHEN** a column of width 1/3 has full width on and is shrunk
- **THEN** its full width is off and its width is 9/10

### Requirement: Consume or expel
Consume or expel SHALL name a pane and a direction, left or right.

When the pane shares its column with other panes, it SHALL be expelled: it leaves its column and is placed alone in a new column of width 1/2, immediately to the left or right of its old column.

When the pane is alone in its column, it SHALL be consumed: it is appended at the bottom of the adjacent column in that direction, and its own column is removed. When no column exists in that direction, nothing SHALL change.

A pane that is consumed or expelled SHALL take an automatic height of weight 1 in the column it enters. The other panes of both columns SHALL keep their heights, except as "Pane heights" defines for a pane left alone in its column.

#### Scenario: Expel to the left
- **WHEN** a workspace holds columns A and B, B holds panes P1 and P2, and P2 is expelled to the left
- **THEN** the workspace holds A, a column with P2, and B with P1, in that order

#### Scenario: Consume to the right
- **WHEN** a workspace holds columns A with P1 and B with P2 and P3, and P1 is consumed to the right
- **THEN** the workspace holds one column with P2, P3 and P1, in that order, and that column keeps B's width

#### Scenario: Consume at the edge
- **WHEN** P1 is alone in the workspace's first column and is consumed to the left
- **THEN** the layout is unchanged

#### Scenario: Consume next to a fixed height
- **WHEN** the area is 80×24, column B holds P2 with a fixed height of 16 rows and P3 with an automatic height of weight 1, and P1, alone in column A, is consumed to the right
- **THEN** P2 keeps its fixed height of 16 rows, and P1 takes an automatic height of weight 1
- **AND** the tiles of P2, P3 and P1 are 16, 4 and 4 rows high

### Requirement: Tile geometry
For a screen area of a given width and height, the layout SHALL give every pane of a workspace a tile on that workspace's strip:

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
- **WHEN** a workspace holds a column of width 1/3 and then a column of width 2/3, and the area is 90×30
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

## ADDED Requirements

### Requirement: Pane heights
Each pane in a column SHALL have either an automatic height with a weight, a positive fraction held in lowest terms, or a fixed height in rows. At most one pane in a column SHALL have a fixed height. A pane SHALL take an automatic height of weight 1 when it opens. When a pane leaves a column and one pane remains with an automatic height, that pane's weight SHALL become 1. Otherwise the panes that remain SHALL keep their heights.

Growing or shrinking a pane's height SHALL give it a fixed height. When the pane has an automatic height, every pane in its column SHALL first take an automatic height whose weight is its current tile height divided by the column's median tile height, so the other panes keep their apparent sizes relative to one another. The median SHALL be the tile height at index n/2, rounded down and counted from 0, of the column's n tile heights sorted in ascending order. The new fixed height SHALL be the pane's current tile height plus, when growing, or less, when shrinking, one step: the area's height divided by 10, rounded to the nearest whole number with halves rounded up, and at least 1. The result SHALL be no less than 3 rows. It SHALL be no more than the area's height less 3 rows for each other pane in the column, or no more than the area's height when the pane is alone. When the pane already had a fixed height and the result equals it, nothing SHALL change.

Resetting a pane's height SHALL give it an automatic height of weight 1. The other panes of its column SHALL keep their heights. When the pane already had an automatic height of weight 1, nothing SHALL change.

Growing and shrinking SHALL measure tile heights, and the step, for the session's current screen area.

#### Scenario: Each step adds the same rows
- **WHEN** the area is 80×24, a column holds P1 above P2, both with automatic heights of weight 1, and P1's height is grown three times
- **THEN** P1's tile is 14, then 16, then 18 rows high
- **AND** P2's tile is 10, then 8, then 6 rows high

#### Scenario: Shrink in a stack of two
- **WHEN** the area is 80×24, a column holds P1 above P2, both with automatic heights of weight 1, and P1's height is shrunk
- **THEN** P1 has a fixed height of 10 rows and P2's tile is 14 rows high

#### Scenario: Step on an odd area
- **WHEN** the area is 80×25, a column holds P1 above P2, both with automatic heights of weight 1, and P1's height is grown
- **THEN** P1 has a fixed height of 16 rows and P2's tile is 9 rows high

#### Scenario: Resizing another pane keeps the earlier one larger
- **WHEN** the area is 80×24, a column holds P1, P2 and P3, all with automatic heights of weight 1, P1's height is grown, and then P3's height is grown
- **THEN** after the first grow the tiles are 10, 7 and 7 rows high
- **AND** after the second grow P1 has an automatic height of weight 10/7, P2 an automatic height of weight 1, and P3 a fixed height of 9 rows
- **AND** the tiles are 9, 6 and 9 rows high

#### Scenario: Upper limit
- **WHEN** the area is 80×24, a column holds three panes, and the top pane has a fixed height of 18 rows and is grown
- **THEN** the layout is unchanged

#### Scenario: Lower limit
- **WHEN** a pane has a fixed height of 3 rows and is shrunk
- **THEN** the layout is unchanged

#### Scenario: Shrink a lone pane
- **WHEN** the area is 80×24 and a pane alone in its column with an automatic height is shrunk
- **THEN** it has a fixed height of 22 rows, and rows 22 and 23 of that column are covered by no tile

#### Scenario: Reset height
- **WHEN** the area is 80×24, a column holds P1 with a fixed height of 16 rows above P2 with an automatic height of weight 1, and P1's height is reset
- **THEN** both panes have automatic heights of weight 1, and both tiles are 12 rows high

#### Scenario: Last pane in a column takes weight 1
- **WHEN** a column holds P1 with an automatic height of weight 10/7 and P2 with a fixed height, P2 closes, and then P3 is consumed into P1's column on an 80×24 area
- **THEN** P1 and P3 both have automatic heights of weight 1, and both tiles are 12 rows high
