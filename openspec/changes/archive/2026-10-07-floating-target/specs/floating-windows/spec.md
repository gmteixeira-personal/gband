## MODIFIED Requirements

### Requirement: Float a window
Floating a tiled window SHALL remove it from its column, as closing removes a window from a column, and append it to its band's floating list. The column rule and the dynamic band rule SHALL then apply. A tiled window SHALL float when floating is toggled on it, and when a request to float it arrives. A request to float SHALL name the floating layer, as the session-server capability's "Session actions" defines. A request to float a window that already floats SHALL change nothing. That window SHALL keep its place in the floating list and its box record, and the request SHALL produce no layout event.

When the window has been floating before, it SHALL take the box record it had when it was last tiled. Otherwise it SHALL take a new box, whose record SHALL be:
- `width` and full width: the configuration's `default_column_width`, and full width off.
- `rows`: the screen area's height less twice its tenth, and at least 3, so that the box is two tenths shorter than the area. The tenth SHALL be the area's height divided by 10, rounded to the nearest whole number with halves rounded up, and at least 1, whatever step a grow or shrink request names.
- `col` and `row`: the box centred in the current screen area, each half the room left beside the box, rounded down, so that one tenth stays free above the box and one below.

#### Scenario: First float opens a new box
- **WHEN** the screen area is 80×24, the default column width is 1/2, window P2 is alone in a column, and P2 is floated
- **THEN** P2's box record has width 1/2, full width off, `rows` 20, `col` 20 and `row` 2
- **AND** its box spans columns 20 to 59 and rows 2 to 21

#### Scenario: New box takes the default width
- **WHEN** the screen area is 80×24, the default column width is 1/3, and a window in a full-width column is floated for the first time
- **THEN** its box record has width 1/3, full width off, `col` 27 and `row` 2

#### Scenario: Float from a stack
- **WHEN** the screen area is 80×24, the default column width is 1/2, a column of width 1/3 holds P1 and P2, both with automatic heights of weight 1, and P2 is floated
- **THEN** the column holds P1 alone, with weight 1
- **AND** P2's box record has width 1/2, `rows` 20, `col` 20 and `row` 2

#### Scenario: New box in a tall area
- **WHEN** the screen area is 120×67, the default column width is 1/2, and a window is floated for the first time
- **THEN** its box record has `rows` 53, `col` 30 and `row` 7

#### Scenario: Float again returns to the last box
- **WHEN** a floating window with `col` 5, `row` 3, width 1/3 and `rows` 10 is tiled and then floated again
- **THEN** its box record has `col` 5, `row` 3, width 1/3 and `rows` 10

#### Scenario: Request to float a tiled window
- **WHEN** the screen area is 80×24, the default column width is 1/2, window 3 is alone in a column, and a request to float window 3 arrives
- **THEN** window 3's box record has width 1/2, full width off, `rows` 20, `col` 20 and `row` 2

#### Scenario: Request to float a floating window
- **WHEN** a band's floating list holds window 3 and then window 4, window 3's box record has `col` 5, `row` 3, width 1/3 and `rows` 10, and a request to float window 3 arrives
- **THEN** the layout is unchanged, and the floating list still holds window 3 and then window 4
- **AND** no layout event is produced

#### Scenario: Two requests to float one window
- **WHEN** the screen area is 80×24, the default column width is 1/2, window 3 is alone in a column of band 1, and two requests to float window 3 arrive one after the other
- **THEN** band 1's floating list holds window 3 once, with width 1/2, `rows` 20, `col` 20 and `row` 2
- **AND** no column of band 1 holds window 3

### Requirement: Tile a window
Tiling a floating window SHALL remove it from the floating list and place it alone in a new column of its band. A floating window SHALL be tiled when floating is toggled on it, and when a request to tile it arrives. A request to tile SHALL name the tiled layer, as the session-server capability's "Session actions" defines. A request to tile a window that is already tiled SHALL change nothing, and SHALL produce no layout event. The new column SHALL take the window's box width and full-width flag, and the window SHALL take an automatic height of weight 1. The toggle or the request SHALL name optionally a tiled window of the same band. The new column SHALL be inserted right of that window's column, or as the band's first column when the toggle or the request names none, or names a window that is not a tiled window of that band. The window's box record SHALL be kept for the next time it floats.

#### Scenario: Tile right of the named window
- **WHEN** a band holds columns A and B and floating window P3 of width 1/3, and P3 is tiled naming a window of A
- **THEN** the band holds A, a column of width 1/3 with P3, and B, in that order, and no floating window

#### Scenario: Tile with no window named
- **WHEN** a band holds column A and floating window P3, and P3 is tiled naming no window
- **THEN** the band holds the column with P3, then A

#### Scenario: Request to tile a floating window
- **WHEN** band 1 holds the columns of windows 1 and 2 and floating window 3 of width 1/3, and a request to tile window 3 naming window 1 arrives
- **THEN** band 1 holds the column of window 1, a column of width 1/3 with window 3, and the column of window 2, in that order, and no floating window

#### Scenario: Request to tile a tiled window
- **WHEN** band 1 holds the columns of windows 1 and 2, and a request to tile window 2 naming window 1 arrives
- **THEN** the layout is unchanged
- **AND** no layout event is produced
