## MODIFIED Requirements

### Requirement: Read the layout
`gband.layout()` SHALL return a new table describing the layout the client holds:

- `cols` and `rows`: the size of the screen area in the latest layout.
- `bands`: one table per band, in order. Each band table holds `id`, the band's number, `columns`, and `floating`.
- `columns`: one table per column, left to right. Each column table holds `width`, the column's width as a number, `full_width`, a boolean, and `windows`.
- `windows`: one table per window, top to bottom. Each window table holds `id`, the window's number, and either `rows`, the fixed height in rows, or `weight`, the weight of an automatic height as a number. It holds `plugin_window`, the plugin window's number, when the window is a drawn window of a plugin window this client opened. A window that runs a program also holds `name`, its shown name as the window-names capability defines it, and `manual_name`, its manual name, when it has one.
- `floating`: one table per floating window, in the band's floating list order. Each holds `id`, `width`, the box's width as a number, `full_width`, a boolean, `rows`, the box record's height, `col` and `row`, the box's top-left cell as placed in the screen area, and `plugin_window`, `name` and `manual_name` as a window table holds them.

Changing the returned table SHALL NOT change the layout. Calling `gband.layout()` while the configuration loads SHALL be an error at the line of the call.

#### Scenario: Two columns
- **WHEN** the screen area is 80×24, the first band holds a column of width 1/2 with window 1, and a full-width column of width 1/3 with window 2 at a fixed height of 10 rows above window 3 at weight 1, and a binding function calls `gband.layout()`
- **THEN** the result's `cols` is 80 and `rows` is 24
- **AND** its first band's columns are `{ width = 0.5, full_width = false, windows = { { id = 1, weight = 1 } } }` and a column with `width` 1/3 and `full_width` `true` whose windows are `{ id = 2, rows = 10 }` and `{ id = 3, weight = 1 }`
- **AND** its last band has no columns and no floating windows

#### Scenario: While loading
- **WHEN** line 4 of `user/init.lua` calls `gband.layout()`
- **THEN** loading fails with an error at `user/init.lua` line 4

#### Scenario: Floating window
- **WHEN** the screen area is 80×24 and band 1 holds window 1 in a column and floating window 2 with `col` 50, `row` 3, width 1/2, full width off and `rows` 10, and a binding function calls `gband.layout()`
- **THEN** its first band's `floating` is `{ { id = 2, width = 0.5, full_width = false, rows = 10, col = 40, row = 3 } }`

#### Scenario: Window names
- **WHEN** band 1 holds windows 1 and 2, both running `bash` with no title set, window 3 running `vim`, and window 3 is renamed `notes`, and a binding function calls `gband.layout()`
- **THEN** window 1's table holds `name = "bash #1"`, window 2's `name = "bash #2"`, and window 3's `name = "notes"` and `manual_name = "notes"`
- **AND** windows 1 and 2 hold no `manual_name`

#### Scenario: Drawn window has no name
- **WHEN** a client plugin opens a tiled plugin window and a binding function calls `gband.layout()`
- **THEN** the table of its drawn window holds `plugin_window` and no `name`

### Requirement: Dispatch order
`gband.window.focus`, `gband.window.set_width`, `gband.window.set_height`, `gband.window.set_position`, `gband.window.send_keys`, `gband.window.send_text`, `gband.window.paste`, `gband.window.rename` and `gband.band.view` SHALL be callable wherever an action value is, and an error elsewhere, as the configuration capability defines for action values. Each SHALL be dispatched like an action. It SHALL take effect after the callback returns, in the order dispatched together with the callback's actions.

#### Scenario: Order with actions
- **WHEN** a binding function calls `gband.window.set_width(1, 1/3)` and then `gband.action.cycle_column_width({ window = 1 })`
- **THEN** window 1's column has width 1/2

#### Scenario: During loading
- **WHEN** line 3 of `user/init.lua` calls `gband.window.focus(1)` at the top level
- **THEN** loading fails with an error at `user/init.lua` line 3

#### Scenario: Rename during loading
- **WHEN** line 3 of `user/init.lua` calls `gband.window.rename(1, "logs")` at the top level
- **THEN** loading fails with an error at `user/init.lua` line 3
