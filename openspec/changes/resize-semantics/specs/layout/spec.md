## MODIFIED Requirements

### Requirement: Column widths
A column's width SHALL be a proportion of the screen area's width, and a column SHALL also carry a full-width flag. The width presets SHALL be 1/3, 1/2 and 2/3. A new column SHALL have width 1/2 and full width off.

Cycling a column's width SHALL turn full width off and set the width to the smallest preset larger than the current width, or to 1/3 when no preset is larger. A column with full width on SHALL count as width 1 while cycling. Toggling full width SHALL flip the flag and keep the proportion.

Growing or shrinking a column's width SHALL turn full width off and add or subtract 1/10 to the current width, keeping the result between 1/10 and 1. A column with full width on SHALL count as width 1 while growing or shrinking. A width SHALL always be held in lowest terms. When the result equals the current width and full width was already off, nothing SHALL change.

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

#### Scenario: Shrink at the minimum
- **WHEN** a column of width 1/10 is shrunk
- **THEN** its width stays 1/10 and the layout is unchanged

#### Scenario: Grow at the maximum
- **WHEN** a column of width 1 with full width off is grown
- **THEN** the layout is unchanged

#### Scenario: Shrink from full width
- **WHEN** a column of width 1/3 has full width on and is shrunk
- **THEN** its full width is off and its width is 9/10

### Requirement: Consume or expel
Consume or expel SHALL name a pane and a direction, left or right.

When the pane shares its column with other panes, it SHALL be expelled: it leaves its column and is placed alone in a new column of width 1/2, immediately to the left or right of its old column.

When the pane is alone in its column, it SHALL be consumed: it is appended at the bottom of the adjacent column in that direction, and its own column is removed. When no column exists in that direction, nothing SHALL change.

A consumed or expelled pane SHALL take height weight 1 in the column it enters. The other panes of both columns SHALL keep their height weights.

#### Scenario: Expel to the left
- **WHEN** a workspace holds columns A and B, B holds panes P1 and P2, and P2 is expelled to the left
- **THEN** the workspace holds A, a column with P2, and B with P1, in that order

#### Scenario: Consume to the right
- **WHEN** a workspace holds columns A with P1 and B with P2 and P3, and P1 is consumed to the right
- **THEN** the workspace holds one column with P2, P3 and P1, in that order, and that column keeps B's width

#### Scenario: Consume at the edge
- **WHEN** P1 is alone in the workspace's first column and is consumed to the left
- **THEN** the layout is unchanged

#### Scenario: Consumed pane takes weight 1
- **WHEN** column B holds P2 with weight 3 and P3 with weight 1, and P1, alone in column A, is consumed to the right
- **THEN** column B holds P2, P3 and P1 with weights 3, 1 and 1

### Requirement: Tile geometry
For a screen area of a given width and height, the layout SHALL give every pane of a workspace a tile on that workspace's strip:

- A column's width in cells SHALL be the area's width when full width is on, and otherwise the area's width multiplied by its proportion and rounded down. It SHALL be at least 3 cells.
- The first column SHALL start at strip position 0, and each later column SHALL start where the previous one ends.
- A column's height SHALL be divided among its panes from the top, in proportion to their height weights. Each pane gets the area's height multiplied by its weight and divided by the sum of the column's weights, rounded down. The first panes, one for each row left over, get one row more.
- Each tile SHALL have a one-cell border. A pane's terminal size SHALL be its tile less 2 columns and 2 rows, and at least 1 column by 1 row.

#### Scenario: Default column on an 80×24 area
- **WHEN** a column of width 1/2 holds one pane and the area is 80×24
- **THEN** the pane's tile is 40×24 at strip position 0
- **AND** the pane's terminal size is 38×22

#### Scenario: Columns side by side
- **WHEN** a workspace holds a column of width 1/3 and then a column of width 2/3, and the area is 90×30
- **THEN** the first tile spans strip positions 0 to 29 and the second spans 30 to 89

#### Scenario: Stack with a leftover row
- **WHEN** a column holds two panes of weight 1 and the area is 80×25
- **THEN** the top tile is 13 rows high and the bottom tile is 12 rows high

#### Scenario: Weighted stack
- **WHEN** a column holds a pane of weight 2 above a pane of weight 1 and the area is 80×25
- **THEN** the top tile is 17 rows high and the bottom tile is 8 rows high

#### Scenario: Full width
- **WHEN** a column has full width on and the area is 100×30
- **THEN** its tile is 100 columns wide

## ADDED Requirements

### Requirement: Pane heights
Each pane in a column SHALL carry a height weight, a whole number from 1 to 8. A pane SHALL take weight 1 when it opens. When a pane leaves a column, the other panes SHALL keep their weights.

Growing a pane's height SHALL add 1 to its weight. Shrinking a pane's height SHALL subtract 1 from its weight when its weight is above 1, and otherwise add 1 to the weight of every other pane in its column. No weight SHALL rise above 8. After a grow or shrink, every weight in the column SHALL be divided by the greatest common divisor of the column's weights. When the result leaves every weight as it was, nothing SHALL change. Resetting a column's heights SHALL set the weight of every pane in it to 1.

#### Scenario: Grow in a stack of two
- **WHEN** a column holds P1 and P2, both of weight 1, and P1's height is grown twice
- **THEN** the weights are 2 and 1, then 3 and 1

#### Scenario: Shrink at weight 1 grows the others
- **WHEN** a column holds P1, P2 and P3, all of weight 1, and P1's height is shrunk
- **THEN** the weights are 1, 2 and 2

#### Scenario: Weights kept in lowest terms
- **WHEN** a column holds P1 of weight 1 and P2 of weight 2, and P1's height is grown
- **THEN** the weights are 1 and 1

#### Scenario: Lone pane
- **WHEN** a pane is alone in its column and its height is grown
- **THEN** the layout is unchanged

#### Scenario: Weight limit
- **WHEN** a column holds P1 of weight 8 and P2 of weight 1, and P1's height is grown
- **THEN** the layout is unchanged

#### Scenario: Reset heights
- **WHEN** a column holds P1 of weight 3 and P2 of weight 1, and its heights are reset
- **THEN** both weights are 1
