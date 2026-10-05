## MODIFIED Requirements

### Requirement: Render context
The context SHALL be a new table for each call, holding:

| field | value |
|---|---|
| `id` | the component's full id |
| `side` | `"client"` |
| `total_width` | the width of the status line in cells, which is the terminal's width |
| `width` | `total_width` less the cells that the other shown components take, by their latest output, with the separators and region gaps "Layout" puts between components; at least 0 |
| `table` | the name of the active key table |
| `band` | `{ number, index, count }`: the viewed band's number, its position from the top, counting from 1, and the number of bands |
| `column` | `{ index, count }`: the focused pane's column position in the viewed band, counting from 1, and the number of columns in that band; nil when the viewed band is empty or a floating pane is focused |
| `pane` | the focused pane's number, or nil when no pane is focused |
| `panes` | a list of every pane in the client's layout, bands from the top, and within each band its columns from the left and their panes from the top, then the band's floating panes in its floating list order, each `{ pane, band, state }`: its number, its band's number, and a copy of its state as the plugin-bridge capability defines |

#### Scenario: Context values
- **WHEN** the client's terminal is 100 columns wide, the viewed band is the second of three bands and holds five columns, the third column holds focused pane 7, the `prefix` table is active, and a component renders
- **THEN** its context has `total_width` 100, `table` `"prefix"`, `band.index` 2, `band.count` 3, `column.index` 3, `column.count` 5 and `pane` 7

#### Scenario: Available width
- **WHEN** the terminal is 80 columns wide, the separator is three cells wide, the left region shows one other component 10 cells wide, and a component in the same region renders
- **THEN** its context's `width` is 67

#### Scenario: Waiting agents counted
- **WHEN** panes 1, 2 and 3 are open, the server has set `agent` to `"waiting"` in the states of panes 1 and 3, and a component with `redraw_on = { "PaneStateChanged" }` counts the entries of `ctx.panes` whose `state.agent` is `"waiting"`
- **THEN** it counts 2

#### Scenario: Floating focus has no column
- **WHEN** the client focuses a floating pane of a band that also holds columns, and a component renders
- **THEN** `ctx.column` is nil
- **AND** `ctx.panes` lists the floating pane after the panes of that band's columns

### Requirement: Bundled segment plugins
gband SHALL bundle these plugin modules, each set up with `gband.plugin` and each adding one component under its plugin's name with `gband.ui.statusline.add`:

| module | plugin | output | redraw on | align | priority | order | group |
|---|---|---|---|---|---|---|---|
| `gband.statusline.band` | `band` | `band ` and the viewed band's index | `BandChanged`, `LayoutChanged` | left | 20 | 10 | `StatusLineSegment` |
| `gband.statusline.mode` | `mode` | the active key table's name; hidden while `root` is active | `KeyTableChanged` | left | 30 | 20 | `StatusLineAccent` |
| `gband.statusline.position` | `position` | the focused column's index, `/`, and the band's column count; hidden while the viewed band is empty or a floating pane is focused | `FocusChanged`, `BandChanged`, `LayoutChanged` | right | 10 | 10 | `StatusLineMuted` |
| `gband.statusline.clock` | `clock` | the local time, formatted by `os.date` with `opts.format`, `"%H:%M"` by default | every `opts.interval` milliseconds, 1000 by default | right | 5 | 20 | `StatusLineMuted` |

Each SHALL take the options `align`, `priority`, `order` and `hl`, which replace the defaults in the table. An option of the wrong type or value SHALL make its `setup` raise an error. The default configuration SHALL set up `band`, `mode` and `position`, and SHALL NOT set up `clock`.

#### Scenario: Reorder a segment
- **WHEN** `user/init.lua` calls `gband.plugin("gband.statusline.mode", { align = "right", order = 1 })` and `gband.plugin("gband.statusline.position")`, and the user presses Ctrl+Space
- **THEN** the right region shows `prefix`, the separator, then the position

#### Scenario: Clock
- **WHEN** `user/init.lua` calls `gband.plugin("gband.statusline.clock", { format = "%H:%M:%S" })`
- **THEN** the right region shows the local time, and it changes every second

#### Scenario: User file without segments
- **WHEN** `user/init.lua` sets up no segment plugin and adds no component
- **THEN** the status line is drawn in `StatusLine` with no text

#### Scenario: Position hidden on floating focus
- **WHEN** the default configuration is in use and the client focuses a floating pane
- **THEN** the status line shows no position segment

