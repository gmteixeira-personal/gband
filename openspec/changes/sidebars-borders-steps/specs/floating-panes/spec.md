## MODIFIED Requirements

### Requirement: Size a floating pane
The width actions SHALL change a floating pane's box width and full-width flag exactly as the layout capability's "Column widths" changes a column's: cycling through the presets, toggling full width, growing or shrinking by the step the request names, and setting a width.

The height actions SHALL change a floating pane's `rows`. Let `h` be its box height as placed in the session's current screen area, and the step in rows the area's height multiplied by the step the request names, rounded to the nearest whole number with halves rounded up, and at least 1:
- Growing SHALL set `rows` to `h` plus the step in rows, and shrinking to `h` less the step in rows.
- Resetting SHALL set `rows` to half the area's height, rounded down.
- Setting a number of rows SHALL set `rows` to that number.
- Setting a weight SHALL leave the pane unchanged.

The new `rows` SHALL be kept between 3 and the area's height. When the result equals the current record, nothing SHALL change. Sizing SHALL keep `col` and `row`.

#### Scenario: Cycle a floating width
- **WHEN** a floating pane has width 1/2 and its width is cycled with the default presets
- **THEN** its width is 2/3

#### Scenario: Grow a floating height
- **WHEN** the area is 80×24 and a floating pane has `rows` 12 and is grown with the step 1/10
- **THEN** its `rows` is 14

#### Scenario: Height limit
- **WHEN** the area is 80×24 and a floating pane has `rows` 24 and is grown
- **THEN** the layout is unchanged

#### Scenario: Reset a floating height
- **WHEN** the area is 80×25 and a floating pane's height is reset
- **THEN** its `rows` is 12

#### Scenario: Consume or expel a floating pane
- **WHEN** a floating pane is consumed or expelled to the left
- **THEN** the layout is unchanged

#### Scenario: Grow a floating width by another step
- **WHEN** a floating pane has width 1/2 and is grown with the step 1/5
- **THEN** its width is 7/10
