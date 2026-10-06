## Purpose

Defines side bars: the columns that Lua code reserves at the left or right of a client's terminal and writes styled lines in. It covers the bar API, how bars are placed and narrow the ribbon area without changing any window's size, their contents and drawing, their callbacks, reloads, plugin ownership and their highlight group.

## ADDED Requirements

### Requirement: Bar API
`gband.bar` SHALL hold the functions `add`, `set_lines`, `set_config`, `remove`, `info` and `list`. Each SHALL be callable while the configuration loads and in any callback. Bars SHALL exist only in the client, and each client SHALL have its own bars.

A function that takes a bar SHALL take its full id. An id that names no bar SHALL be an error at the line of the call, except for `remove`. An argument of the wrong type, an unknown field, or a value out of range SHALL be an error at the line of the call, and SHALL change nothing.

#### Scenario: Unknown bar
- **WHEN** a binding function calls `gband.bar.set_lines("absent", {})` and no bar has the id `absent`
- **THEN** the call raises an error naming `absent`

#### Scenario: Added while loading
- **WHEN** `user/init.lua` calls `gband.bar.add({ id = "list", side = "left" })` at the top level
- **THEN** loading succeeds and the bar is placed when the client draws its first frame

### Requirement: Add a bar
`gband.bar.add(spec)` SHALL add a bar and return its full id. `spec` SHALL be a table with these fields:

| field | value | default |
|---|---|---|
| `id` | a non-empty string | the plugin's name |
| `side` | `"left"` or `"right"` | required |
| `size` | the bar's width in columns, an integer of at least 1 | `1` |
| `order` | a number | `0` |
| `lines` | the bar's lines, as "Bar contents" defines | no lines |
| `hl` | a group name | `"Bar"` |
| `on_resize` | a function | none |

A bar added by code that belongs to a plugin SHALL belong to that plugin, and its full id SHALL follow the plugins capability's namespacing of names. When such code omits `id`, the full id SHALL be the plugin's name. Code that belongs to no plugin SHALL give `id`, which SHALL be used as given. A missing `side`, an omitted `id` outside a plugin, or a full id that another bar already has SHALL be an error at the line of the call, and SHALL add nothing.

#### Scenario: Plugin bar without an id
- **WHEN** the plugin `tabs` calls `gband.bar.add({ side = "left" })`
- **THEN** the call returns `"tabs"`

#### Scenario: Plugin bar with an id
- **WHEN** the plugin `tabs` calls `gband.bar.add({ id = "list", side = "right", size = 20 })`
- **THEN** the call returns `"tabs.list"`

#### Scenario: Duplicate id
- **WHEN** `user/init.lua` adds a bar with `id = "info"` and then adds another with `id = "info"`
- **THEN** the second call raises an error naming `info`, and one bar exists

#### Scenario: Top is not a side
- **WHEN** `user/init.lua` calls `gband.bar.add({ id = "info", side = "top" })`
- **THEN** the call raises an error naming `side`

### Requirement: Change and remove a bar
`gband.bar.set_config(id, config)` SHALL change the bar's `side`, `size`, `order` and `hl` that `config` names, and keep the others. `gband.bar.set_lines(id, lines)` SHALL replace the bar's lines. `gband.bar.remove(id)` SHALL remove the bar and return `true`, or return `false` when no bar has that id.

#### Scenario: Widen a bar
- **WHEN** the client's terminal is 80×24, a left bar of size 10 is the only bar, and a binding function calls `gband.bar.set_config(id, { size = 20 })`
- **THEN** the bar spans columns 0 to 19 and the ribbon area spans columns 20 to 79

#### Scenario: Remove a bar
- **WHEN** a left bar of size 20 is the only bar and a binding function removes it
- **THEN** the call returns `true` and the ribbon area is the whole terminal

### Requirement: Placing bars
The client SHALL place its bars against its terminal each time it draws, one bar at a time, starting from the whole terminal. Bars SHALL be taken in ascending `order`, and in the order they were added when their `order` is equal. A left bar SHALL take the first `size` columns of the area left so far, at the terminal's full height, and a right bar its last `size` columns. A bar whose `size` is not less than the columns left SHALL NOT be shown: it SHALL take no cell, and placing SHALL go on with the next bar. A bar SHALL be shown exactly when it is placed and takes cells. The area left after the last bar SHALL be the ribbon area, as the client-attach capability uses it.

#### Scenario: Left bar
- **WHEN** the client's terminal is 80×24 and a left bar of size 20 is the only bar
- **THEN** the bar spans columns 0 to 19 and rows 0 to 23, and the ribbon area spans columns 20 to 79

#### Scenario: Two bars on one side
- **WHEN** the client's terminal is 80×24, a left bar A of size 10 has `order` 0 and a left bar B of size 5 has `order` -1
- **THEN** B spans columns 0 to 4, A spans columns 5 to 14, and the ribbon area starts at column 15

#### Scenario: Bars on both sides
- **WHEN** the client's terminal is 80×24 and it has a left bar of size 20 and a right bar of size 10
- **THEN** the ribbon area spans columns 20 to 69

#### Scenario: Bar too wide
- **WHEN** the client's terminal is 10×24 and a left bar of size 10 is the only bar
- **THEN** the bar is not shown and the ribbon area is the whole terminal

### Requirement: Bars keep window sizes
Bars SHALL change only the ribbon area. They SHALL NOT change the client's reported size, the session's screen area, or any window's terminal size. A change of the ribbon area that the bars cause SHALL be handled by the client's view and drawn state as the layout-view and animations capabilities handle a change of the client's terminal size, and SHALL NOT be reported to the server. Changing a bar's lines or `hl` SHALL NOT change the ribbon area. Several changes made by one callback or one load SHALL be handled as one change.

#### Scenario: Bar added by a key
- **WHEN** the client's 80×24 terminal sets the screen area, the only window sits in a column of width 1/2, and a binding function adds a left bar of size 20
- **THEN** the client reports no resize, `tput cols` in the window still prints `38`, and the tile is drawn on screen columns 20 to 59

#### Scenario: Camera follows the narrower ribbon
- **WHEN** the client's 80×24 terminal sets the screen area, the viewed band holds two columns of width 1/2 with the second focused and fully shown, and a binding function adds a left bar of size 20
- **THEN** the second column is still fully shown, on screen columns 40 to 79

### Requirement: Bar contents
A bar's lines SHALL have the form of a window's lines, as the plugin-windows capability defines, with the bar's `hl` group in place of `PluginWindow`. Row `r` of a bar, counted from 0, SHALL show line `r + 1`, and a bar SHALL NOT scroll. Each row SHALL be drawn from the bar's left column as the plugin-windows capability draws a row of a content area, and cut at the bar's right edge. Every cell of a shown bar that no span covers SHALL be blank in the resolved style of the bar's `hl` group. Lines beyond the bar's rows SHALL NOT be drawn.

#### Scenario: Styled bar
- **WHEN** a left bar of size 10 with `hl = "Bar"` holds the line `{ { text = "gband", hl = "Title" } }`, `Bar` resolves to `{ bg = 236 }` and `Title` to `{ bold = true }`
- **THEN** row 0 shows `gband` from column 0, bold on background 236
- **AND** the rest of the bar is blank on background 236

#### Scenario: Line cut at the bar's edge
- **WHEN** a left bar of size 4 holds the lines `"one"`, `"three"` and `"x"`
- **THEN** its first three rows show `one`, `thre` and `x`, and its other rows are blank

### Requirement: Bar callbacks
A bar's `on_resize` SHALL run as a callback that belongs to the bar's plugin, with the bar's full id and its shown width and height, each time that size changes. A bar that is not shown SHALL have the size 0×0. When a bar is added while the configuration loads, its first call SHALL come after loading finishes. An error raised by `on_resize` SHALL be reported as a plugin error, and SHALL leave the bar unchanged.

#### Scenario: Terminal grows
- **WHEN** a left bar of size 20 records the arguments of its `on_resize` and the client's terminal grows from 80×24 to 100×30
- **THEN** `on_resize` runs with the bar's id, 20 and 30

#### Scenario: Bar hidden
- **WHEN** a left bar of size 20 is shown and the terminal shrinks to 20 columns
- **THEN** its `on_resize` runs with 0 and 0

### Requirement: Bar information
`gband.bar.list()` SHALL return one table per bar, in ascending byte order of full ids, as `gband.bar.info` returns it. `gband.bar.info(id)` SHALL return a new table holding `id`, `side`, `size`, `order`, `hl`, `plugin` (the owner's name, or nil), `shown`, and `col`, `width` and `height`: the bar's first column and size in the terminal, or nil while it is not shown. Changing a returned table SHALL NOT change the bar.

#### Scenario: Info of a shown bar
- **WHEN** the client's terminal is 80×24, `user/init.lua` adds a right bar with `id = "info"` and size 12, and a binding function calls `gband.bar.info("info")`
- **THEN** the result holds `side = "right"`, `size = 12`, `shown = true`, `col = 68`, `width = 12` and `height = 24`

### Requirement: Bars and reloads
A reload SHALL remove every bar the previous configuration added, before the new configuration's `ConfigReloaded` handlers run. The bars the new configuration adds SHALL take their place, and the ribbon area SHALL change at most once for the reload. A failed reload SHALL keep the bars as they were. A bar SHALL be removed when the plugin it belongs to is marked failed.

#### Scenario: Reload keeps an unchanged bar
- **WHEN** `user/init.lua` adds a left bar of size 20, the user saves it unchanged, and the reload succeeds
- **THEN** the ribbon area does not change

#### Scenario: Failed plugin
- **WHEN** the plugin `tabs` adds a left bar in its `setup` and then raises an error
- **THEN** the bar is removed and the ribbon area is the whole terminal

### Requirement: Bar highlight group
The bar API SHALL own the group `Bar` with no fields as its default, set in the defaults layer the highlights capability defines. A change of any group's resolved style SHALL redraw every bar whose drawing uses it.

#### Scenario: Theme sets a bar's base
- **WHEN** `user/init.lua` calls `gband.hl.set("Bar", { bg = 236 })` and a bar with the default `hl` is shown
- **THEN** every cell of the bar has background 236
