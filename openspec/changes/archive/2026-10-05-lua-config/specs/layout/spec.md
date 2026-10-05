## MODIFIED Requirements

### Requirement: Column widths
A column's width SHALL be a proportion of the screen area's width, and a column SHALL also carry a full-width flag. The width presets SHALL be the configuration's `width_presets`, in ascending order, which are 1/3, 1/2 and 2/3 by default. A new column SHALL have the configuration's `default_column_width`, which is 1/2 by default, and full width off.

Cycling a column's width SHALL turn full width off and set the width to the smallest preset larger than the current width, or to the smallest preset when no preset is larger. A column with full width on SHALL count as width 1 while cycling. Toggling full width SHALL flip the flag and keep the proportion.

Growing or shrinking a column's width SHALL turn full width off and add or subtract 1/10 to the current width, keeping the result between 0 and 10000. A width above 1 SHALL make the column wider than the screen area. A column with full width on SHALL count as width 1 while growing or shrinking. A width SHALL always be held in lowest terms. When the result equals the current width and full width was already off, nothing SHALL change.

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
