## MODIFIED Requirements

### Requirement: Column widths
A column's width SHALL be a proportion of the screen area's width, and a column SHALL also carry a full-width flag. The width presets SHALL be the configuration's `width_presets`, in ascending order, which are 1/3, 1/2 and 2/3 by default. A new column SHALL have the width the open window action names, or the configuration's `default_column_width`, which is 1/2 by default, when it names none, and full width off.

Cycling a column's width SHALL turn full width off and set the width to the smallest preset larger than the current width, or to the smallest preset when no preset is larger. A column with full width on SHALL count as width 1 while cycling. Toggling full width SHALL flip the flag and keep the proportion.

A request to grow or shrink a column's width or a window's height SHALL name a step, as the actions capability defines. In the scenarios of this capability, a grow or shrink that names no step names the step 1/10.

Growing or shrinking a column's width SHALL turn full width off and add or subtract the step to the current width, keeping the result between 0 and 10000. A width above 1 SHALL make the column wider than the screen area. A column with full width on SHALL count as width 1 while growing or shrinking. A width SHALL always be held in lowest terms. When the result equals the current width and full width was already off, nothing SHALL change.

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
- **WHEN** the default column width is 1/3 and a window opens
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
- **WHEN** the default column width is 1/2 and a window opens with the width 1/4
- **THEN** its column has width 1/4 and full width off

#### Scenario: Grow by another step
- **WHEN** a column of width 1/2 is grown with the step 1/4
- **THEN** its width is 3/4

### Requirement: Window heights
Each window in a column SHALL have either an automatic height with a weight, a positive fraction held in lowest terms, or a fixed height in rows. At most one window in a column SHALL have a fixed height. A window SHALL take an automatic height of weight 1 when it opens. When a window leaves a column and one window remains with an automatic height, that window's weight SHALL become 1. Otherwise the windows that remain SHALL keep their heights.

Growing or shrinking a window's height SHALL give it a fixed height. When the window has an automatic height, every window in its column SHALL first take an automatic height whose weight is its current tile height divided by the column's median tile height, so the other windows keep their apparent sizes relative to one another. The median SHALL be the tile height at index n/2, rounded down and counted from 0, of the column's n tile heights sorted in ascending order. The new fixed height SHALL be the window's current tile height plus, when growing, or less, when shrinking, one step in rows: the area's height multiplied by the step, rounded to the nearest whole number with halves rounded up, and at least 1. The result SHALL be no less than 3 rows. It SHALL be no more than the area's height less 3 rows for each other window in the column, or no more than the area's height when the window is alone. When the window already had a fixed height and the result equals it, nothing SHALL change.

Resetting a window's height SHALL give it an automatic height of weight 1. The other windows of its column SHALL keep their heights. When the window already had an automatic height of weight 1, nothing SHALL change.

Growing and shrinking SHALL measure tile heights, and the step in rows, for the session's current screen area.

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

#### Scenario: Resizing another window keeps the earlier one larger
- **WHEN** the area is 80×24, a column holds P1, P2 and P3, all with automatic heights of weight 1, P1's height is grown, and then P3's height is grown
- **THEN** after the first grow the tiles are 10, 7 and 7 rows high
- **AND** after the second grow P1 has an automatic height of weight 10/7, P2 an automatic height of weight 1, and P3 a fixed height of 9 rows
- **AND** the tiles are 9, 6 and 9 rows high

#### Scenario: Upper limit
- **WHEN** the area is 80×24, a column holds three windows, and the top window has a fixed height of 18 rows and is grown
- **THEN** the layout is unchanged

#### Scenario: Lower limit
- **WHEN** a window has a fixed height of 3 rows and is shrunk
- **THEN** the layout is unchanged

#### Scenario: Shrink a lone window
- **WHEN** the area is 80×24 and a window alone in its column with an automatic height is shrunk
- **THEN** it has a fixed height of 22 rows, and rows 22 and 23 of that column are covered by no tile

#### Scenario: Reset height
- **WHEN** the area is 80×24, a column holds P1 with a fixed height of 16 rows above P2 with an automatic height of weight 1, and P1's height is reset
- **THEN** both windows have automatic heights of weight 1, and both tiles are 12 rows high

#### Scenario: Last window in a column takes weight 1
- **WHEN** a column holds P1 with an automatic height of weight 10/7 and P2 with a fixed height, P2 closes, and then P3 is consumed into P1's column on an 80×24 area
- **THEN** P1 and P3 both have automatic heights of weight 1, and both tiles are 12 rows high

#### Scenario: Height step from the request
- **WHEN** the area is 80×24, a column holds P1 above P2, both with automatic heights of weight 1, and P1's height is grown with the step 1/4
- **THEN** P1 has a fixed height of 18 rows and P2's tile is 6 rows high
