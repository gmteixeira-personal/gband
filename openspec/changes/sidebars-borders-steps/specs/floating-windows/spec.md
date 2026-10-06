## MODIFIED Requirements

### Requirement: Float a window
Toggling floating on a tiled window SHALL remove it from its column, as closing removes a window from a column, and append it to its band's floating list. The column rule and the dynamic band rule SHALL then apply.

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

### Requirement: Size a floating window
The width actions SHALL change a floating window's box width and full-width flag exactly as the layout capability's "Column widths" changes a column's: cycling through the presets, toggling full width, growing or shrinking by the step the request names, and setting a width.

The height actions SHALL change a floating window's `rows`. Let `h` be its box height as placed in the session's current screen area, and the step in rows the area's height multiplied by the step the request names, rounded to the nearest whole number with halves rounded up, and at least 1:
- Growing SHALL set `rows` to `h` plus the step in rows, and shrinking to `h` less the step in rows.
- Resetting SHALL set `rows` to the height of a new box, as "Float a window" defines.
- Setting a number of rows SHALL set `rows` to that number.
- Setting a weight SHALL leave the window unchanged.

The new `rows` SHALL be kept between 3 and the area's height. When the result equals the current record, nothing SHALL change. Sizing SHALL keep `col` and `row`.

#### Scenario: Cycle a floating width
- **WHEN** a floating window has width 1/2 and its width is cycled with the default presets
- **THEN** its width is 2/3

#### Scenario: Grow a floating height
- **WHEN** the area is 80×24 and a floating window has `rows` 12 and is grown with the step 1/10
- **THEN** its `rows` is 14

#### Scenario: Height limit
- **WHEN** the area is 80×24 and a floating window has `rows` 24 and is grown
- **THEN** the layout is unchanged

#### Scenario: Reset a floating height
- **WHEN** the area is 80×25 and a floating window's height is reset
- **THEN** its `rows` is 19

#### Scenario: Consume or expel a floating window
- **WHEN** a floating window is consumed or expelled to the left
- **THEN** the layout is unchanged

#### Scenario: Grow a floating width by another step
- **WHEN** a floating window has width 1/2 and is grown with the step 1/5
- **THEN** its width is 7/10
