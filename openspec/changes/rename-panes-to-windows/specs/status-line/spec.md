## MODIFIED Requirements

### Requirement: Components
`gband.ui.statusline.add(spec)` SHALL add a component and return its full id. `spec` SHALL be a table with these fields:

| field | value | default |
|---|---|---|
| `id` | a non-empty string | the plugin's name |
| `render` | a function | required |
| `align` | `"left"`, `"center"` or `"right"` | `"left"` |
| `priority` | a number | `0` |
| `order` | a number | `0` |
| `hl` | a group name | `"StatusLineSegment"` |
| `redraw_on` | a list of built-in event names, as the lua-events capability lists them, and `"User"` | empty |
| `redraw_interval` | an integer number of milliseconds, at least 100 | none |
| `fill` | a boolean | `false` |

A component whose `fill` is true is a fill component, which "Render triggers" renders again when the width left to it changes.

A component added by code that belongs to a plugin SHALL belong to that plugin, and its full id SHALL follow the plugins capability's namespacing of names. When such code omits `id`, the full id SHALL be the plugin's name. Code that belongs to no plugin SHALL give `id`, which SHALL be used as given. A missing `render`, a field of the wrong type or value, an unknown event name, an omitted `id` outside a plugin, or a full id that another component already has SHALL be an error at the line of the call, and SHALL add nothing.

`gband.ui.statusline.remove(id)` SHALL remove the component whose full id is `id` and return `true`, or return `false` when no component has that id. `gband.ui.statusline.list()` SHALL return one table per component, in ascending byte order of full ids, each holding `id`, `align`, `priority`, `order`, `hl`, `fill`, `plugin` (the owner's name, or nil) and `enabled`. Changing a returned table SHALL NOT change the component. All three functions SHALL be callable while the configuration loads and in any callback.

#### Scenario: Plugin component without an id
- **WHEN** the plugin `window` calls `gband.ui.statusline.add({ render = fn })`
- **THEN** the call returns `"window"`

#### Scenario: Plugin component with an id
- **WHEN** the plugin `window` calls `gband.ui.statusline.add({ id = "count", render = fn })`
- **THEN** the call returns `"window.count"`

#### Scenario: User component
- **WHEN** `user/init.lua` calls `gband.ui.statusline.add({ id = "host", render = fn })`
- **THEN** the call returns `"host"`

#### Scenario: Duplicate id
- **WHEN** line 5 of `user/init.lua` adds a component with id `host` a second time
- **THEN** loading fails with an error at `user/init.lua` line 5 naming `host`

#### Scenario: Unknown event
- **WHEN** line 3 of a plugin's `setup` adds a component with `redraw_on = { "FocusChange" }`
- **THEN** a plugin error at that line names `FocusChange`

#### Scenario: Fill of the wrong type
- **WHEN** line 4 of `user/init.lua` adds a component with `fill = "yes"`
- **THEN** loading fails with an error at `user/init.lua` line 4 naming `fill`

#### Scenario: Remove a bundled segment
- **WHEN** the default configuration's segments are set up and a binding function calls `gband.ui.statusline.remove("mode")`
- **THEN** it returns `true` and the mode segment is no longer drawn

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
| `column` | `{ index, count }`: the focused window's column position in the viewed band, counting from 1, and the number of columns in that band; nil when the viewed band is empty |
| `window` | the focused window's number, or nil when no window is focused |
| `windows` | a list of every window in the client's layout, bands from the top, columns from the left and windows from the top, each `{ window, band, state }`: its number, its band's number, and a copy of its state as the plugin-bridge capability defines |

#### Scenario: Context values
- **WHEN** the client's terminal is 100 columns wide, the viewed band is the second of three bands and holds five columns, the third column holds focused window 7, the `prefix` table is active, and a component renders
- **THEN** its context has `total_width` 100, `table` `"prefix"`, `band.index` 2, `band.count` 3, `column.index` 3, `column.count` 5 and `window` 7

#### Scenario: Available width
- **WHEN** the terminal is 80 columns wide, the separator is three cells wide, the left region shows one other component 10 cells wide, and a component in the same region renders
- **THEN** its context's `width` is 67

#### Scenario: Waiting agents counted
- **WHEN** windows 1, 2 and 3 are open, the server has set `agent` to `"waiting"` in the states of windows 1 and 3, and a component with `redraw_on = { "WindowStateChanged" }` counts the entries of `ctx.windows` whose `state.agent` is `"waiting"`
- **THEN** it counts 2

### Requirement: Render errors
Each call of a component's `render` SHALL run protected, as a callback that belongs to the component's plugin, with its own instruction budget of the size the plugins capability defines. A run that exceeds the budget SHALL stop only that call. The code that triggered the render and the other components SHALL continue. When a call raises an error, returns a value "Render output" does not allow, or is stopped by the instruction limit, the component SHALL be disabled and hidden until the configuration next loads, and the error SHALL be reported as a plugin error, as the configuration capability defines. A call stopped by the instruction limit SHALL also mark the component's plugin failed, as the plugins capability defines. Components of a failed plugin SHALL NOT render and SHALL be hidden.

#### Scenario: Failing component
- **WHEN** the plugin `window`'s component raises `boom` on line 9 of its file, and another component returns `ok`
- **THEN** the client keeps running and `ok` is drawn
- **AND** the error item shows `window: <path>:9: boom`
- **AND** `gband.ui.statusline.list()` gives the component `enabled` false

#### Scenario: Looping render
- **WHEN** a component's `render` runs `while true do end`
- **THEN** the client handles the next key
- **AND** the error item names the plugin and the instruction limit
