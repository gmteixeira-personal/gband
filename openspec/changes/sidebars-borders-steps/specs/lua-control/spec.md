## MODIFIED Requirements

### Requirement: Action targets
A built-in session action value SHALL accept one optional table, its target. A target SHALL take effect only when the action value is called. For `close_pane`, `consume_or_expel_left`, `consume_or_expel_right`, `move_column_left`, `move_column_right`, `move_pane_down`, `move_pane_up`, `cycle_column_width`, `toggle_full_width`, `grow_column_width`, `shrink_column_width`, `grow_pane_height`, `shrink_pane_height` and `reset_pane_height`, the target SHALL hold `pane`. The action SHALL then name that pane instead of the focused pane. For `open_pane`, the target SHALL hold `band`, `after`, `floating`, or any of them together:
- With both `band` and `after`, the new column follows `after`'s column in `band`.
- With only `band`, the new column is that band's first column.
- With only `after`, the band is the band that holds `after`.
- `floating`, a boolean, opens the pane floating when `true`, in `band`, or in the viewed band when the target holds no `band`. A target holding `after` with `floating = true` SHALL be an error. `floating = false` SHALL open the pane tiled, as a target without `floating` does.

For `toggle_pane_floating`, the target SHALL hold `pane`, and optionally `after`, a tiled pane of the same band. Tiling a floating pane SHALL then place it after `after`. Without `after`, it SHALL be placed after the tiled pane this client focused most recently in that band, when it is still tiled there, and as the band's first column otherwise. `after` on a pane that is tiled SHALL be ignored.

The target of `grow_column_width`, `shrink_column_width`, `grow_pane_height` and `shrink_pane_height` MAY also hold `step`: a number greater than 0 and at most 10000 for the two width actions, and greater than 0 and at most 1 for the two height actions, read as the configuration capability reads a step. The action SHALL then use that step, as the actions capability defines. A target of these actions that holds `step` and no `pane` SHALL name the pane the view resolves.

`send_prefix` SHALL accept a target holding `pane`, and send the prefix key to that pane.

A target passed to a view action or to `detach` SHALL be an error. Each of these SHALL also be an error at the line of the call:
- A target that is not a table.
- A field the action does not take.
- A `step` out of its range.
- A pane or band number not in the client's layout.
- An `after` pane not in `band`, or, for `toggle_pane_floating`, not a tiled pane of `pane`'s band.

Calling an action value with no target SHALL resolve it against the view, as the actions capability defines.

#### Scenario: Close a named pane
- **WHEN** panes 1 and 2 are open with pane 2 focused and a binding function calls `gband.action.close_pane({ pane = 1 })`
- **THEN** the client sends close pane naming pane 1
- **AND** pane 2 stays focused

#### Scenario: Grow a named column
- **WHEN** the only column of band 1 holds pane 1 at width 1/2, the client views band 2, and a binding function calls `gband.action.grow_column_width({ pane = 1 })`
- **THEN** pane 1's column has width 3/5

#### Scenario: Open in another band
- **WHEN** the client views band 1 and a binding function calls `gband.action.open_pane({ band = 2 })`, where band 2 is the empty last band
- **THEN** band 2 holds a new pane running the user's shell
- **AND** the client views band 2 with the new pane focused

#### Scenario: Unknown pane
- **WHEN** line 6 of `user/init.lua` holds a binding function that calls `gband.action.close_pane({ pane = 99 })` and no pane 99 exists, and the user presses its key
- **THEN** the client shows an error at `user/init.lua` line 6 naming pane 99
- **AND** nothing is sent to the server

#### Scenario: Target on a view action
- **WHEN** a binding function calls `gband.action.focus_column_left({ pane = 1 })`
- **THEN** the call raises an error and the view is unchanged

#### Scenario: Open a floating pane
- **WHEN** the client views band 1 and a binding function calls `gband.action.open_pane({ floating = true })`
- **THEN** band 1's floating list ends with a new pane running the user's shell
- **AND** the client focuses it with the floating layer active

#### Scenario: Floating with after
- **WHEN** a binding function calls `gband.action.open_pane({ after = 1, floating = true })`
- **THEN** the call raises an error naming `after`

#### Scenario: Tile a named pane after a named pane
- **WHEN** band 1 holds columns with panes 1 and 2 and floating pane 3, and a binding function calls `gband.action.toggle_pane_floating({ pane = 3, after = 1 })`
- **THEN** band 1 holds the columns of pane 1, pane 3 and pane 2, in that order

#### Scenario: Move a named pane
- **WHEN** band 1 holds columns with panes 1 and 2, pane 2 is focused, and a binding function calls `gband.action.move_column_right({ pane = 1 })`
- **THEN** band 1 holds the columns of pane 2 and pane 1, in that order, and pane 2 stays focused

#### Scenario: Grow by a given step
- **WHEN** pane 1 is focused alone in a column of width 1/2 and a binding function calls `gband.action.grow_column_width({ step = 1/4 })`
- **THEN** pane 1's column has width 3/4

#### Scenario: Step out of range
- **WHEN** line 4 of a binding function's file calls `gband.action.grow_pane_height({ step = 2 })`
- **THEN** the call raises an error at line 4 naming `step`, and nothing is sent
