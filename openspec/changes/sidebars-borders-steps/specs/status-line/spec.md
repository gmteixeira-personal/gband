## MODIFIED Requirements

### Requirement: Placement
The status line SHALL be a bar, as the bars capability defines, that the bundled plugin module `gband.statusline`, whose plugin is named `statusline`, adds when `gband.plugin` sets it up. Its `setup` SHALL take these options:

| option | value | default |
|---|---|---|
| `side` | `"left"` or `"right"` | `"left"` |
| `min_width` | an integer of at least 1 | `20` |
| `max_width` | an integer of at least `min_width` | `40` |
| `order` | a number | `0` |

An unknown option, or an option of the wrong type or value, SHALL make `setup` raise an error naming the option. The bar SHALL have the full id `statusline`, the side `side`, the order `order` and the group `StatusLine`. Its size SHALL be the display width of the widest line that the error item and the shown components other than fill components draw, by their latest outputs, kept between `min_width` and `max_width`. The status line SHALL be drawn exactly while its bar is shown, so at the terminal's full height. While the plugin is not set up, no component SHALL render. `gband.ui.statusline` SHALL be usable whether or not the plugin is set up.

#### Scenario: Default placement
- **WHEN** the default configuration is in use and the client's terminal is 80×24
- **THEN** the status line spans columns 0 to 19 and rows 0 to 23, and the ribbon area spans columns 20 to 79
- **AND** the client reports the size 80×24

#### Scenario: Top placement
- **WHEN** `user/init.lua` calls `gband.plugin("gband.statusline", { position = "top" })`
- **THEN** the call returns `false` and a plugin error names `position`

#### Scenario: Off
- **WHEN** `user/init.lua` does not set up `gband.statusline`
- **THEN** no status line is drawn and the ribbon area is the whole terminal

#### Scenario: Terminal too short
- **WHEN** the default configuration is in use and the client's terminal is 20 columns wide
- **THEN** no status line is drawn and the ribbon area is the whole terminal

#### Scenario: Grows with its content
- **WHEN** the status line is set up with `min_width = 10` and `max_width = 30`, and its widest component returns a line 25 cells wide
- **THEN** the status line is 25 columns wide

#### Scenario: Capped width
- **WHEN** the status line is set up with `min_width = 10` and `max_width = 30`, and a component returns a line 50 cells wide
- **THEN** the status line is 30 columns wide and that line is cut to 30 cells, ending with `…`

#### Scenario: Right side
- **WHEN** `user/init.lua` calls `gband.plugin("gband.statusline", { side = "right" })` and the client's terminal is 80×24
- **THEN** the status line spans columns 60 to 79 and the ribbon area spans columns 0 to 59

### Requirement: Drawing the line
Every cell of the status line SHALL be drawn with the resolved style of `StatusLine`. Each line of each shown component SHALL be drawn on the row "Layout" gives it, from the status line's first column. Each span SHALL be drawn with the resolved style of its group applied over the resolved style of `StatusLine`: a field the span's style sets SHALL replace that field of `StatusLine`, and the other fields of `StatusLine` SHALL stay. The error item SHALL be drawn the same way with `StatusLineError`.

#### Scenario: Span over the base
- **WHEN** `StatusLine` resolves to `{ fg = 7, bg = 236 }`, a span's group resolves to `{ fg = 3, bold = true }`, and the span is drawn
- **THEN** its cells have foreground 3, background 236 and bold
- **AND** cells no span covers have foreground 7 and background 236

### Requirement: Components
`gband.ui.statusline.add(spec)` SHALL add a component and return its full id. `spec` SHALL be a table with these fields:

| field | value | default |
|---|---|---|
| `id` | a non-empty string | the plugin's name |
| `render` | a function | required |
| `align` | `"top"`, `"center"` or `"bottom"` | `"top"` |
| `priority` | a number | `0` |
| `order` | a number | `0` |
| `hl` | a group name | `"StatusLineSegment"` |
| `redraw_on` | a list of built-in event names, as the lua-events capability lists them, and `"User"` | empty |
| `redraw_interval` | an integer number of milliseconds, at least 100 | none |
| `fill` | a boolean | `false` |

A component whose `fill` is true is a fill component. It SHALL NOT widen the status line, and "Render triggers" renders it again when the room left to it changes.

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

#### Scenario: Horizontal alignment rejected
- **WHEN** line 2 of `user/init.lua` adds a component with `align = "right"`
- **THEN** loading fails with an error at `user/init.lua` line 2 naming `align`

### Requirement: Render output
A component's `render` SHALL be called with one argument, the context, and SHALL return nil, a line, or a table `{ lines = <list> }` whose items are lines. A line SHALL be a string, or a list whose items are strings or tables `{ text = <string>, hl = <group name> }`. A string SHALL be one span in the component's `hl` group. A span table without `hl` SHALL use the component's `hl` group. Control characters, the code points U+0000 to U+001F, U+007F and U+0080 to U+009F, SHALL be removed from every span's text before it is measured or drawn. A component whose output is nil, holds no line, or holds only lines whose spans' text is empty SHALL be hidden: it SHALL take no row. Any other return value SHALL be a render error.

#### Scenario: Plain string
- **WHEN** a component with `hl = "StatusLineMuted"` returns `"hi"`
- **THEN** `hi` is drawn in `StatusLineMuted`

#### Scenario: Escape sequence removed
- **WHEN** a component returns `{ { text = "a\27[31mb" } }`
- **THEN** the drawn text is `a[31mb`

#### Scenario: Hidden component
- **WHEN** the top region holds A, B and C in that order, each returning one line, and B returns `""`
- **THEN** A and C are drawn on adjacent rows

#### Scenario: Several lines
- **WHEN** a component returns `{ lines = { "one", "two" } }`
- **THEN** it takes two rows, showing `one` and then `two`

### Requirement: Render context
The context SHALL be a new table for each call, holding:

| field | value |
|---|---|
| `id` | the component's full id |
| `side` | `"client"` |
| `total_width` | the status line's `max_width` |
| `total_height` | the status line's height in rows, which is the terminal's height |
| `width` | for a fill component, the status line's width as the error item and the other shown components set it, by their latest outputs; for any other component, `total_width` |
| `height` | for a fill component, `total_height` less the rows that the error item and the other shown components take, by their latest outputs, with the gaps "Layout" puts between regions, and at least 0; for any other component, `total_height` |
| `table` | the name of the active key table |
| `band` | `{ number, index, count }`: the viewed band's number, its position from the top, counting from 1, and the number of bands |
| `column` | `{ index, count }`: the focused window's column position in the viewed band, counting from 1, and the number of columns in that band; nil when the viewed band is empty or a floating window is focused |
| `window` | the focused window's number, or nil when no window is focused |
| `windows` | a list of every window in the client's layout, bands from the top, and within each band its columns from the left and their windows from the top, then the band's floating windows in its floating list order, each `{ window, band, state }`: its number, its band's number, and a copy of its state as the plugin-bridge capability defines |

#### Scenario: Context values
- **WHEN** the status line has `max_width` 40, the client's terminal is 100×30, the viewed band is the second of three bands and holds five columns, the third column holds focused window 7, the `prefix` table is active, and a component renders
- **THEN** its context has `total_width` 40, `total_height` 30, `table` `"prefix"`, `band.index` 2, `band.count` 3, `column.index` 3, `column.count` 5 and `window` 7

#### Scenario: Available width
- **WHEN** the status line has `min_width` 10 and `max_width` 40, the widest line of the other shown components is 14 cells wide, and a fill component renders
- **THEN** its context's `width` is 14

#### Scenario: Available height
- **WHEN** the terminal is 24 rows high, the top region shows one other component of one line, the bottom region shows one component of one line, and a fill component in the top region renders
- **THEN** its context's `height` is 21

#### Scenario: Waiting agents counted
- **WHEN** windows 1, 2 and 3 are open, the server has set `agent` to `"waiting"` in the states of windows 1 and 3, and a component with `redraw_on = { "WindowStateChanged" }` counts the entries of `ctx.windows` whose `state.agent` is `"waiting"`
- **THEN** it counts 2

#### Scenario: Floating focus has no column
- **WHEN** the client focuses a floating window of a band that also holds columns, and a component renders
- **THEN** `ctx.column` is nil
- **AND** `ctx.windows` lists the floating window after the windows of that band's columns

### Requirement: Render triggers
While the status line is set up, the client SHALL call an enabled component's `render`:
- once when the client attaches, and once after each successful reload;
- once when the component is added after the configuration has loaded;
- once each time an event its `redraw_on` names is emitted, after the handlers of that event have run;
- once each `redraw_interval` milliseconds, when the component has one;
- once when the terminal's height changes;
- once on each `HighlightChanged` and `ColorschemeChanged`;
- for a fill component, once more after any of the triggers above, when the line has been laid out and the component's available `width` or `height` differs from those of its latest call.

When one trigger renders several components, the components that are not fill components SHALL render first, and the fill components after them, in descending `priority`, then in the order they were added. Once those renders are done, the status line SHALL be laid out, and each enabled fill component whose available `width` or `height`, as the context defines them, then differs from those of its latest call SHALL render once more, in the same order. Those renders SHALL NOT cause further renders for the same trigger.

The client SHALL NOT call `render` at any other time. Each frame SHALL be drawn from the components' latest outputs, and drawing a frame, including every frame of an animation, SHALL NOT call `render`. A component's output SHALL stay in use until its next call.

#### Scenario: Redraw on an event
- **WHEN** a component has `redraw_on = { "FocusChanged" }` and the user focuses the column to the left
- **THEN** `render` runs once more, and the status line shows its new output

#### Scenario: Not per frame
- **WHEN** a component has rendered once and the camera animates over 20 frames with no event its `redraw_on` names
- **THEN** `render` is not called again

#### Scenario: Interval
- **WHEN** a component has `redraw_interval = 1000` and no other trigger occurs for 3.5 seconds
- **THEN** `render` runs 3 times in that period

#### Scenario: Width change
- **WHEN** the client's terminal changes from 80×24 to 80×30
- **THEN** every enabled component renders once with `total_height` 30

#### Scenario: Fill renders after the others
- **WHEN** the fill component F and the component M both have `redraw_on = { "KeyTableChanged" }`, M is in F's region, and the user presses Ctrl+Space, after which M's output grows from empty to `navigation`
- **THEN** F renders once, after M, and its context's `height` is one row less than in its latest call

#### Scenario: Fill follows another component's width
- **WHEN** the status line has `min_width` 10 and `max_width` 40, the fill component F has no `redraw_on`, and the component P, the widest shown component, with `redraw_on = { "FocusChanged" }`, changes its output from 20 cells wide to 22
- **THEN** F renders once more, with a context `width` two cells larger than in its latest call

#### Scenario: Fill width unchanged
- **WHEN** the fill component F has no `redraw_on`, and the component P, with `redraw_on = { "FocusChanged" }`, changes its output from `2/3` to `3/3` without changing the status line's width
- **THEN** F does not render

### Requirement: Layout
Each shown component SHALL be placed in the region its `align` names. Within a region, components SHALL be placed in ascending `order`, and in the order they were added when their `order` is equal. Each component SHALL take one row for each line of its output, and the components of a region SHALL take adjacent rows. The height of a region SHALL be the rows of its components. Two adjacent non-empty regions SHALL be at least one row apart.

When the regions with their gaps take more rows than the status line has, the shown component with the lowest `priority` SHALL be dropped, the one added last when several share it. Dropping SHALL repeat until the regions fit or one component is left. When one component is left and it still has more lines than the status line has rows, its lines after the last row SHALL NOT be drawn. Dropping SHALL change only what is drawn, and SHALL NOT disable a component.

Each line SHALL be drawn from the status line's first column. A line wider than the status line SHALL be cut to its width as `gband.ui.truncate` cuts it. All widths SHALL be display widths, as `gband.ui.width` measures them.

The top region SHALL start at the status line's first row, and the bottom region SHALL end at its last row. The center region SHALL start at half of the status line's height less half of its own height, rounded down, and SHALL then be moved the least distance that keeps it at least one row from the top and bottom regions.

#### Scenario: Default segments
- **WHEN** the default configuration is in use, the client's terminal is 80×24, the viewed band is the first, the second of three columns is focused, and `root` is active
- **THEN** the status line shows `band 1` on row 0, `C-space navigation` on row 1, and `2/3` on row 23

#### Scenario: Mode shown
- **WHEN** the default configuration is in use and the user presses Ctrl+Space
- **THEN** the status line shows `band 1` on row 0 and `navigation` on row 1, and the hints start on row 2

#### Scenario: Lowest priority dropped first
- **WHEN** the client's terminal is 1 row high and the top region holds A of priority 1 with output `aaa` and B of priority 2 with output `bbb`
- **THEN** only `bbb` is drawn

#### Scenario: Last component cut
- **WHEN** the status line is set up with `min_width = 1` and `max_width = 6`, and the only component returns `abcdefghij`
- **THEN** the status line is 6 columns wide and shows `abcde…`

#### Scenario: Wide characters
- **WHEN** the status line is set up with `min_width = 1` and `max_width = 5`, and the only component returns `日本語`
- **THEN** the status line shows `日本…`, which takes 5 cells

#### Scenario: Center region
- **WHEN** the client's terminal is 24 rows high and the center region holds one component returning `mid`
- **THEN** `mid` is drawn on row 11

### Requirement: Render errors
Each call of a component's `render` SHALL run protected, as a callback that belongs to the component's plugin, with its own instruction budget of the size the plugins capability defines. A run that exceeds the budget SHALL stop only that call. The code that triggered the render and the other components SHALL continue. When a call raises an error, returns a value "Render output" does not allow, or is stopped by the instruction limit, the component SHALL be disabled and hidden until the configuration next loads, and the error SHALL be reported as a plugin error, as the configuration capability defines. A call stopped by the instruction limit SHALL also mark the component's plugin failed, as the plugins capability defines. Components of a failed plugin SHALL NOT render and SHALL be hidden.

#### Scenario: Failing component
- **WHEN** the plugin `window`'s component raises `boom` on line 9 of its file, and another component returns `ok`
- **THEN** the client keeps running and `ok` is drawn
- **AND** the error item shows `error`, and the last entry of `gband.errors()` is `window: <path>:9: boom`
- **AND** `gband.ui.statusline.list()` gives the component `enabled` false

#### Scenario: Looping render
- **WHEN** a component's `render` runs `while true do end`
- **THEN** the client handles the next key
- **AND** the last entry of `gband.errors()` names the plugin and the instruction limit

### Requirement: Error item
While the status line is drawn and the configuration capability reports an error, including an error of the server's Lua, the status line SHALL show the error item: the word `error` in `StatusLineError`, as the first line of the top region, before every component. It SHALL have a priority above every component's, so it is dropped last, and it SHALL be cut like any line. The error item SHALL be shown until the configuration capability clears the error. While no error is reported, the error item SHALL take no row. The error's text SHALL be read through the error list, as the configuration and error-list capabilities define.

#### Scenario: Plugin error at start
- **WHEN** line 3 of the plugin `hello`'s `client.lua` raises `boom`, and a client attaches
- **THEN** row 0 of the status line shows `error`, and the ribbon area is unchanged
- **AND** `gband.errors()` holds `hello: <path>/client.lua:3: boom`

#### Scenario: Server plugin error
- **WHEN** line 5 of the plugin `agent-status`'s `server.lua` raises `boom`, and a client attaches
- **THEN** row 0 of the status line shows `error`
- **AND** `gband.errors()` holds `server: agent-status: <path>/server.lua:5: boom`

#### Scenario: Cleared by a good load
- **WHEN** the error item shows an error and the user fixes `user/init.lua`
- **THEN** the error item disappears once the configuration reloads

### Requirement: Bundled segment plugins
gband SHALL bundle these plugin modules, each set up with `gband.plugin` and each adding one component under its plugin's name with `gband.ui.statusline.add`:

| module | plugin | output | redraw on | align | priority | order | group |
|---|---|---|---|---|---|---|---|
| `gband.statusline.band` | `band` | `band ` and the viewed band's index | `BandChanged`, `LayoutChanged` | top | 20 | 10 | `StatusLineSegment` |
| `gband.statusline.mode` | `mode` | the active key table's label, as `gband.keymap.label` returns it; hidden while `root` is active | `KeyTableChanged` | top | 30 | 20 | `StatusLineAccent` |
| `gband.statusline.position` | `position` | the focused column's index, `/`, and the band's column count; hidden while the viewed band is empty or a floating window is focused | `FocusChanged`, `BandChanged`, `LayoutChanged` | bottom | 10 | 10 | `StatusLineMuted` |
| `gband.statusline.clock` | `clock` | the local time, formatted by `os.date` with `opts.format`, `"%H:%M"` by default | every `opts.interval` milliseconds, 1000 by default | bottom | 5 | 20 | `StatusLineMuted` |

Each SHALL take the options `align`, `priority`, `order` and `hl`, which replace the defaults in the table. An option of the wrong type or value SHALL make its `setup` raise an error. The default configuration SHALL set up `band`, `mode` and `position`, and SHALL NOT set up `clock`.

#### Scenario: Reorder a segment
- **WHEN** `user/init.lua` sets up `gband.statusline`, calls `gband.plugin("gband.statusline.mode", { align = "bottom", order = 1 })` and `gband.plugin("gband.statusline.position")`, and the user presses Ctrl+Space
- **THEN** the bottom region shows `navigation` on one row and the position on the next

#### Scenario: Label of a mode
- **WHEN** `user/init.lua` declares `resize` a mode with the label `RESIZE`, sets up `gband.statusline.mode`, and a binding enters `resize`
- **THEN** the mode segment shows `RESIZE`

#### Scenario: Clock
- **WHEN** `user/init.lua` calls `gband.plugin("gband.statusline.clock", { format = "%H:%M:%S" })`
- **THEN** the bottom region shows the local time, and it changes every second

#### Scenario: User file without segments
- **WHEN** `user/init.lua` sets up `gband.statusline`, sets up no segment plugin and adds no component
- **THEN** the status line is drawn in `StatusLine` with no text, 20 columns wide

#### Scenario: Position hidden on floating focus
- **WHEN** the default configuration is in use and the client focuses a floating window
- **THEN** the status line shows no position segment
