## MODIFIED Requirements

### Requirement: Read the layout
`gband.layout()` SHALL return a new table describing the layout the client holds:

- `cols` and `rows`: the size of the screen area in the latest layout.
- `bands`: one table per band, in order. Each band table holds `id`, the band's number, `columns`, and `floating`.
- `columns`: one table per column, left to right. Each column table holds `width`, the column's width as a number, `full_width`, a boolean, and `panes`.
- `panes`: one table per pane, top to bottom. Each pane table holds `id`, the pane's number, and either `rows`, the fixed height in rows, or `weight`, the weight of an automatic height as a number. It holds `window`, the window's number, when the pane is a plugin pane of a window this client opened.
- `floating`: one table per floating pane, in the band's floating list order. Each holds `id`, `width`, the box's width as a number, `full_width`, a boolean, `rows`, the box record's height, `col` and `row`, the box's top-left cell as placed in the screen area, and `window` as a pane table holds it.

Changing the returned table SHALL NOT change the layout. Calling `gband.layout()` while the configuration loads SHALL be an error at the line of the call.

#### Scenario: Two columns
- **WHEN** the screen area is 80×24, the first band holds a column of width 1/2 with pane 1, and a full-width column of width 1/3 with pane 2 at a fixed height of 10 rows above pane 3 at weight 1, and a binding function calls `gband.layout()`
- **THEN** the result's `cols` is 80 and `rows` is 24
- **AND** its first band's columns are `{ width = 0.5, full_width = false, panes = { { id = 1, weight = 1 } } }` and a column with `width` 1/3 and `full_width` `true` whose panes are `{ id = 2, rows = 10 }` and `{ id = 3, weight = 1 }`
- **AND** its last band has no columns and no floating panes

#### Scenario: While loading
- **WHEN** line 4 of `user/init.lua` calls `gband.layout()`
- **THEN** loading fails with an error at `user/init.lua` line 4

#### Scenario: Floating pane
- **WHEN** the screen area is 80×24 and band 1 holds pane 1 in a column and floating pane 2 with `col` 50, `row` 3, width 1/2, full width off and `rows` 10, and a binding function calls `gband.layout()`
- **THEN** its first band's `floating` is `{ { id = 2, width = 0.5, full_width = false, rows = 10, col = 40, row = 3 } }`

### Requirement: Read the view
`gband.view()` SHALL return a new table holding:
- `band`: the viewed band's number.
- `pane`: the focused pane's number, or nil when no pane is focused.
- `floating`: `true` when the floating layer of the viewed band is active, and `false` otherwise.
- `window`: the focused window's number, as the plugin-windows capability defines it, or nil.
- `table`: the active key table.
- `cols` and `rows`: the size of the ribbon area.

Calling it while the configuration loads SHALL be an error at the line of the call.

#### Scenario: Focused pane
- **WHEN** the client views band 1 with pane 2 focused, its ribbon area is 80×23, and a binding in `prefix` calls `gband.view()`
- **THEN** the result is `{ band = 1, pane = 2, floating = false, table = "prefix", cols = 80, rows = 23 }`

#### Scenario: Empty band
- **WHEN** the client views the empty band and a binding function calls `gband.view()`
- **THEN** the result's `pane` is nil

#### Scenario: Floating focus
- **WHEN** the client focuses floating pane 3 and a binding function calls `gband.view()`
- **THEN** the result's `pane` is 3 and its `floating` is `true`

### Requirement: Action targets
A built-in session action value SHALL accept one optional table, its target. A target SHALL take effect only when the action value is called. For `close_pane`, `consume_or_expel_left`, `consume_or_expel_right`, `move_column_left`, `move_column_right`, `move_pane_down`, `move_pane_up`, `cycle_column_width`, `toggle_full_width`, `grow_column_width`, `shrink_column_width`, `grow_pane_height`, `shrink_pane_height` and `reset_pane_height`, the target SHALL hold `pane`. The action SHALL then name that pane instead of the focused pane. For `open_pane`, the target SHALL hold `band`, `after`, `floating`, or any of them together:
- With both `band` and `after`, the new column follows `after`'s column in `band`.
- With only `band`, the new column is that band's first column.
- With only `after`, the band is the band that holds `after`.
- `floating`, a boolean, opens the pane floating when `true`, in `band`, or in the viewed band when the target holds no `band`. A target holding `after` with `floating = true` SHALL be an error. `floating = false` SHALL open the pane tiled, as a target without `floating` does.

For `toggle_pane_floating`, the target SHALL hold `pane`, and optionally `after`, a tiled pane of the same band. Tiling a floating pane SHALL then place it after `after`. Without `after`, it SHALL be placed after the tiled pane this client focused most recently in that band, when it is still tiled there, and as the band's first column otherwise. `after` on a pane that is tiled SHALL be ignored.

`send_prefix` SHALL accept a target holding `pane`, and send the prefix key to that pane.

A target passed to a view action or to `detach` SHALL be an error. Each of these SHALL also be an error at the line of the call:
- A target that is not a table.
- A field the action does not take.
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

### Requirement: Dispatch order
`gband.pane.focus`, `gband.pane.set_width`, `gband.pane.set_height`, `gband.pane.set_position`, `gband.pane.send_keys`, `gband.pane.send_text`, `gband.pane.paste` and `gband.band.view` SHALL be callable wherever an action value is, and an error elsewhere, as the configuration capability defines for action values. Each SHALL be dispatched like an action. It SHALL take effect after the callback returns, in the order dispatched together with the callback's actions.

#### Scenario: Order with actions
- **WHEN** a binding function calls `gband.pane.set_width(1, 1/3)` and then `gband.action.cycle_column_width({ pane = 1 })`
- **THEN** pane 1's column has width 1/2

#### Scenario: During loading
- **WHEN** line 3 of `user/init.lua` calls `gband.pane.focus(1)` at the top level
- **THEN** loading fails with an error at `user/init.lua` line 3

## ADDED Requirements

### Requirement: Place a floating pane
`gband.pane.set_position(pane, position)` SHALL dispatch set position naming the pane, as the session-server capability defines it. The `position` argument SHALL be a table holding `col` and `row`, each an integer of at least 0. The box SHALL be placed as the floating-panes capability defines. A pane number not in the client's layout, a pane that is not floating in the client's layout, a missing or unknown field, or a value of the wrong type or out of range SHALL be an error at the line of the call.

#### Scenario: Place a floating pane
- **WHEN** the screen area is 80×24, pane 3 floats with a 40×12 box, and a binding function calls `gband.pane.set_position(3, { col = 10, row = 2 })`
- **THEN** pane 3's box spans columns 10 to 49 and rows 2 to 13

#### Scenario: Place past the edge
- **WHEN** the same binding function calls `gband.pane.set_position(3, { col = 70, row = 2 })`
- **THEN** pane 3's box spans columns 40 to 79

#### Scenario: Tiled pane
- **WHEN** pane 1 is tiled and a binding function calls `gband.pane.set_position(1, { col = 0, row = 0 })`
- **THEN** the call raises an error naming pane 1

#### Scenario: Missing field
- **WHEN** pane 3 floats and a binding function calls `gband.pane.set_position(3, { col = 4 })`
- **THEN** the call raises an error naming `row`
