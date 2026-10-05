## MODIFIED Requirements

### Requirement: Column widths
A column's width SHALL be a proportion of the screen area's width, and a column SHALL also carry a full-width flag. The width presets SHALL be the configuration's `width_presets`, in ascending order, which are 1/3, 1/2 and 2/3 by default. A new column SHALL have the width the open pane action names, or the configuration's `default_column_width`, which is 1/2 by default, when it names none, and full width off.

Cycling a column's width SHALL turn full width off and set the width to the smallest preset larger than the current width, or to the smallest preset when no preset is larger. A column with full width on SHALL count as width 1 while cycling. Toggling full width SHALL flip the flag and keep the proportion.

Growing or shrinking a column's width SHALL turn full width off and add or subtract 1/10 to the current width, keeping the result between 0 and 10000. A width above 1 SHALL make the column wider than the screen area. A column with full width on SHALL count as width 1 while growing or shrinking. A width SHALL always be held in lowest terms. When the result equals the current width and full width was already off, nothing SHALL change.

Setting a column's width SHALL turn full width off and set the width to the given width, a number greater than 0 and at most 10000 held in lowest terms. When the given width equals the current width and full width was already off, nothing SHALL change.

#### Scenario: Cycle from the default
- **WHEN** a column of width 1/2 is cycled three times with the default presets
- **THEN** its width is 2/3, then 1/3, then 1/2

#### Scenario: Cycle from full width
- **WHEN** a column of width 2/3 has full width on and is cycled with the default presets
- **THEN** its full width is off and its width is 1/3

#### Scenario: Cycle through configured presets
- **WHEN** the width presets are 1/4, 1/2 and 3/4, and a column of width 1/2 is cycled twice
- **THEN** its width is 3/4, then 1/4

#### Scenario: Cycle from a width between presets
- **WHEN** the width presets are 1/4 and 3/4, and a column of width 1/2 is cycled
- **THEN** its width is 3/4

#### Scenario: Configured default width
- **WHEN** the default column width is 1/3 and a pane opens
- **THEN** its column has width 1/3 and full width off

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


#### Scenario: Set a width
- **WHEN** a column of width 1/2 has full width on and its width is set to 0.35
- **THEN** its full width is off and its width is 7/20

#### Scenario: Set the same width
- **WHEN** a column of width 1/3 with full width off has its width set to 1/3
- **THEN** the layout is unchanged

#### Scenario: Width named on open
- **WHEN** the default column width is 1/2 and a pane opens with the width 1/4
- **THEN** its column has width 1/4 and full width off

### Requirement: Open a pane
Opening a pane SHALL name a band and, optionally, a pane in that band and a column width. The new pane SHALL be placed alone in a new column, whose width is as "Column widths" defines. The new column SHALL be inserted immediately to the right of the named pane's column, or as the band's first column when no pane is named. No other column's width SHALL change.

#### Scenario: Open right of the focused column
- **WHEN** a band holds columns A and B, and a pane opens next to a pane in A
- **THEN** the band holds A, the new column and B, in that order
- **AND** A and B keep their widths

#### Scenario: Open in an empty band
- **WHEN** a pane opens in an empty band with no pane named
- **THEN** that band holds one column with the new pane

### Requirement: Close a pane
A pane SHALL leave the layout when its program exits, whether the program exited by itself or because the pane was closed. A plugin pane, as the session-server capability defines it, SHALL leave the layout when it is closed. The pane's column and band SHALL then follow the column rule and the dynamic band rule.

#### Scenario: Pane leaves a stack
- **WHEN** a column holds panes P1, P2 and P3, and P2's program exits
- **THEN** the column holds P1 and P3, in that order, and keeps its width

#### Scenario: Plugin pane closed
- **WHEN** a band holds columns A, B and C, and B holds only a plugin pane, which is closed
- **THEN** the band holds A and C, in that order

## ADDED Requirements

### Requirement: Set a pane's height
Setting a pane's height SHALL name either a number of rows or a weight.

Setting a number of rows SHALL give the pane a fixed height. When the pane has an automatic height, every pane in its column SHALL first take an automatic height whose weight is its current tile height divided by the column's median tile height, as growing does. The new fixed height SHALL be the number of rows, kept within the same limits that growing and shrinking keep: no less than 3 rows, and no more than the area's height less 3 rows for each other pane in the column, or no more than the area's height when the pane is alone. When the pane already had a fixed height and the result equals it, nothing SHALL change.

Setting a weight SHALL give the pane an automatic height of that weight, a number greater than 0 and at most 10000 held in lowest terms. The other panes of its column SHALL keep their heights. When the pane already had an automatic height of that weight, nothing SHALL change.

Setting a height SHALL measure tile heights for the session's current screen area.

#### Scenario: Fixed rows in a stack of two
- **WHEN** the screen area is 80×24, a column holds P1 and P2 with automatic heights of weight 1, and P1's height is set to 8 rows
- **THEN** P1 has a fixed height of 8 rows and P2 an automatic height of weight 1
- **AND** P1's tile is 8 rows high and P2's is 16

#### Scenario: Rows above the limit
- **WHEN** the screen area is 80×24, a column holds P1 and P2, and P1's height is set to 30 rows
- **THEN** P1 has a fixed height of 21 rows

#### Scenario: Rows below the limit
- **WHEN** P1's height is set to 1 row
- **THEN** P1 has a fixed height of 3 rows

#### Scenario: Fixed height moves to another pane
- **WHEN** the screen area is 80×24, a column holds P1 with a fixed height of 14 rows and P2 with an automatic height, and P2's height is set to 6 rows
- **THEN** P1 has an automatic height and P2 a fixed height of 6 rows

#### Scenario: Set a weight
- **WHEN** a column holds P1 with a fixed height of 14 rows and P2 with an automatic height of weight 1, and P1's weight is set to 2
- **THEN** P1 has an automatic height of weight 2 and P2 keeps weight 1
