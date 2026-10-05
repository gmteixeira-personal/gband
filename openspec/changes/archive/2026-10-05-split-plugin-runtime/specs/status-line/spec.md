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
| `column` | `{ index, count }`: the focused pane's column position in the viewed band, counting from 1, and the number of columns in that band; nil when the viewed band is empty |
| `pane` | the focused pane's number, or nil when no pane is focused |
| `panes` | a list of every pane in the client's layout, bands from the top, columns from the left and panes from the top, each `{ pane, band, state }`: its number, its band's number, and a copy of its state as the plugin-bridge capability defines |

#### Scenario: Context values
- **WHEN** the client's terminal is 100 columns wide, the viewed band is the second of three bands and holds five columns, the third column holds focused pane 7, the `prefix` table is active, and a component renders
- **THEN** its context has `total_width` 100, `table` `"prefix"`, `band.index` 2, `band.count` 3, `column.index` 3, `column.count` 5 and `pane` 7

#### Scenario: Available width
- **WHEN** the terminal is 80 columns wide, the separator is three cells wide, the left region shows one other component 10 cells wide, and a component in the same region renders
- **THEN** its context's `width` is 67

#### Scenario: Waiting agents counted
- **WHEN** panes 1, 2 and 3 are open, the server has set `agent` to `"waiting"` in the states of panes 1 and 3, and a component with `redraw_on = { "PaneStateChanged" }` counts the entries of `ctx.panes` whose `state.agent` is `"waiting"`
- **THEN** it counts 2

### Requirement: Error item
While the status line is drawn, it SHALL show the error the configuration capability reports last, including an error of the server's Lua, as its first line, in `StatusLineError`, as the first item of the left region, before every component. It SHALL have a priority above every component's, so it is dropped last, and it SHALL be cut like any last component. The error item SHALL be shown until the configuration capability clears the error. While no error is reported, the error item SHALL take no cell and no separator.

#### Scenario: Plugin error at start
- **WHEN** line 3 of the plugin `hello`'s `client.lua` raises `boom`, and a client attaches
- **THEN** the status line starts with `hello: <path>/client.lua:3: boom`, and the ribbon area is unchanged

#### Scenario: Server plugin error
- **WHEN** line 5 of the plugin `agent-status`'s `server.lua` raises `boom`, and a client attaches
- **THEN** the status line starts with `server: agent-status: <path>/server.lua:5: boom`

#### Scenario: Cleared by a good load
- **WHEN** the error item shows an error and the user fixes `user/init.lua`
- **THEN** the error item disappears once the configuration reloads
