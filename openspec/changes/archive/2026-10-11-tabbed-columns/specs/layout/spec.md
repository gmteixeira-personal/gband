## MODIFIED Requirements

### Requirement: Tile geometry
For a screen area of a given width and height, the layout SHALL give every window of a band a tile on that band's strip:

- A column's width in cells SHALL be the area's width when full width is on, and otherwise the area's width multiplied by its proportion and rounded down. It SHALL be at least 3 cells and at most 65535 cells.
- The first column SHALL start at strip position 0, and each later column SHALL start where the previous one ends.
- A column's tiles SHALL be stacked from the area's top row, in the column's window order, with no gap between them.
- A window with a fixed height SHALL get that many rows. When its column holds other windows, it SHALL get no more than the area's height less 3 rows for each other window. When it is alone in its column, it SHALL get no more than the area's height, and the rows below its tile SHALL be covered by no tile.
- The windows with an automatic height SHALL share the rows the fixed window leaves, in proportion to their weights. Each gets those rows multiplied by its weight and divided by the sum of the automatic weights, rounded down, and the first automatic windows, one for each row left over, get one row more. While the shared rows allow every automatic window 3 rows, a share below 3 rows SHALL be raised: the topmost automatic window whose share is below 3 rows takes 3 rows, and the other automatic windows share the rest by the same rule.
- In a tabbed column, as the tabbed-columns capability defines it, every window's tile SHALL start at the area's top row and SHALL be the column's tab height high: the area's height when the tab height is automatic, and its fixed rows, at most the area's height, otherwise. The rows below the tiles SHALL be covered by no tile. The windows' own heights SHALL not affect the tiles, and the two rules above SHALL not apply to the column.
- Each tile SHALL have a one-cell border. A window's terminal size SHALL be its tile less 2 columns and 2 rows, and at least 1 column by 1 row.

#### Scenario: Default column on an 80×24 area
- **WHEN** a column of width 1/2 holds one window and the area is 80×24
- **THEN** the window's tile is 40×24 at strip position 0
- **AND** the window's terminal size is 38×22

#### Scenario: Columns side by side
- **WHEN** a band holds a column of width 1/3 and then a column of width 2/3, and the area is 90×30
- **THEN** the first tile spans strip positions 0 to 29 and the second spans 30 to 89

#### Scenario: Stack with a leftover row
- **WHEN** a column holds two windows with automatic heights of weight 1 and the area is 80×25
- **THEN** the top tile is 13 rows high and the bottom tile is 12 rows high

#### Scenario: Fixed window above an automatic window
- **WHEN** a column holds P1 with a fixed height of 16 rows above P2 with an automatic height, and the area is 80×24
- **THEN** P1's tile is 16 rows high and P2's tile is 8 rows high

#### Scenario: Automatic windows share by weight
- **WHEN** the area is 80×24 and a column holds P1 with an automatic height of weight 10/7, P2 with an automatic height of weight 1, and P3 with a fixed height of 9 rows
- **THEN** the tiles of P1, P2 and P3 are 9, 6 and 9 rows high

#### Scenario: Fixed height leaves room for the others
- **WHEN** the area is 80×24 and a column holds P1 with a fixed height of 30 rows above two windows with automatic heights
- **THEN** the tiles are 18, 3 and 3 rows high

#### Scenario: Automatic window raised to 3 rows
- **WHEN** the area is 80×24 and a column holds P1 with an automatic height of weight 1/20 above P2 with an automatic height of weight 1
- **THEN** P1's tile is 3 rows high and P2's tile is 21 rows high

#### Scenario: Lone window with a fixed height
- **WHEN** the area is 80×24 and a window alone in its column has a fixed height of 20 rows
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

#### Scenario: Tabbed column
- **WHEN** a tabbed column of width 1/2 holds P1 with a fixed height of 16 rows above P2 with an automatic height, its tab height is automatic, and the area is 80×24
- **THEN** the tiles of P1 and P2 are both 40×24 at row 0

#### Scenario: Tabbed column with a fixed tab height
- **WHEN** a tabbed column holds P1 and P2, its tab height is fixed at 20 rows, and the area is 80×24
- **THEN** the tiles of P1 and P2 are both 20 rows high at row 0, and rows 20 to 23 of that column are covered by no tile

### Requirement: Window heights
Each window in a column SHALL have either an automatic height with a weight, a positive fraction held in lowest terms, or a fixed height in rows. At most one window in a column SHALL have a fixed height. A window SHALL take an automatic height of weight 1 when it opens. When a window leaves a column and one window remains with an automatic height, that window's weight SHALL become 1. Otherwise the windows that remain SHALL keep their heights.

Growing or shrinking a window's height SHALL give it a fixed height. When the window has an automatic height, every window in its column SHALL first take an automatic height whose weight is its current tile height divided by the column's median tile height, so the other windows keep their apparent sizes relative to one another. The median SHALL be the tile height at index n/2, rounded down and counted from 0, of the column's n tile heights sorted in ascending order. The new fixed height SHALL be the window's current tile height plus, when growing, or less, when shrinking, one step in rows: the area's height multiplied by the step, rounded to the nearest whole number with halves rounded up, and at least 1. The result SHALL be no less than 3 rows. It SHALL be no more than the area's height less 3 rows for each other window in the column, or no more than the area's height when the window is alone. When the window already had a fixed height and the result equals it, nothing SHALL change.

Resetting a window's height SHALL give it an automatic height of weight 1. The other windows of its column SHALL keep their heights. When the window already had an automatic height of weight 1, nothing SHALL change.

Growing and shrinking SHALL measure tile heights, and the step in rows, for the session's current screen area.

While a window's column is tabbed, growing, shrinking or resetting the window's height SHALL change the column's tab height instead, as the tabbed-columns capability's "Tab height" defines, and the rules above SHALL not apply to it.

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

#### Scenario: Grow in a tabbed column
- **WHEN** the area is 80×24, a tabbed column with a fixed tab height of 16 rows holds P1 above P2, both with automatic heights of weight 1, and P1's height is grown
- **THEN** the column's tab height is 18 rows, and P1 and P2 still have automatic heights of weight 1

### Requirement: Set a window's height
Setting a window's height SHALL name either a number of rows or a weight.

Setting a number of rows SHALL give the window a fixed height. When the window has an automatic height, every window in its column SHALL first take an automatic height whose weight is its current tile height divided by the column's median tile height, as growing does. The new fixed height SHALL be the number of rows, kept within the same limits that growing and shrinking keep: no less than 3 rows, and no more than the area's height less 3 rows for each other window in the column, or no more than the area's height when the window is alone. When the window already had a fixed height and the result equals it, nothing SHALL change.

Setting a weight SHALL give the window an automatic height of that weight, a number greater than 0 and at most 10000 held in lowest terms. The other windows of its column SHALL keep their heights. When the window already had an automatic height of that weight, nothing SHALL change.

Setting a height SHALL measure tile heights for the session's current screen area.

While a window's column is tabbed, setting the window's height SHALL change the column's tab height instead, as the tabbed-columns capability's "Tab height" defines, and the rules above SHALL not apply to it.

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

#### Scenario: Fixed height moves to another window
- **WHEN** the screen area is 80×24, a column holds P1 with a fixed height of 14 rows and P2 with an automatic height, and P2's height is set to 6 rows
- **THEN** P1 has an automatic height and P2 a fixed height of 6 rows

#### Scenario: Set a weight
- **WHEN** a column holds P1 with a fixed height of 14 rows and P2 with an automatic height of weight 1, and P1's weight is set to 2
- **THEN** P1 has an automatic height of weight 2 and P2 keeps weight 1

#### Scenario: Set rows in a tabbed column
- **WHEN** the screen area is 80×24, a tabbed column holds P1 with a fixed height of 14 rows and P2 with an automatic height, and P2's height is set to 10 rows
- **THEN** the column's tab height is 10 rows, P1 keeps its fixed height of 14 rows, and P2 keeps its automatic height

### Requirement: Move a column
Moving a tiled window's column left or right SHALL swap that column with the adjacent column in that direction. Both columns SHALL keep their windows, widths, full-width flags, heights, tabbed flags and tab heights. When no column exists in that direction, nothing SHALL change. Moving a floating window left or right SHALL move its box, as the floating-windows capability defines.

#### Scenario: Move a column right
- **WHEN** a band holds columns A, B and C, and a window of A has its column moved right
- **THEN** the band holds B, A and C, in that order, and every column keeps its width

#### Scenario: Move at the edge
- **WHEN** a window of the band's first column has its column moved left
- **THEN** the layout is unchanged

#### Scenario: Move a tabbed column
- **WHEN** a band holds tabbed column A with a fixed tab height of 16 rows and column B, and a window of A has its column moved right
- **THEN** the band holds B and A, in that order, and A is still tabbed with a tab height of 16 rows

