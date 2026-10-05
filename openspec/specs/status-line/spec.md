# status-line Specification

## Purpose

Defines the client's status line: the rows it takes from the terminal, the component API that fills it, when components render, how their output is laid out and cut to fit, how their errors are contained and reported, and the segment plugins bundled with gband.

## Requirements

### Requirement: Placement
The status line SHALL take `statusline_height` whole rows of the client's terminal: the bottom rows when `statusline_position` is `"bottom"`, and the top rows when it is `"top"`. It SHALL be drawn only when `statusline_position` is not `"off"` and the terminal has more rows than `statusline_height`. While it is not drawn, it SHALL take no row, and no component SHALL render. The rows the status line leaves form the ribbon area, as the client-attach capability defines.

#### Scenario: Default placement
- **WHEN** no option sets the placement and the client's terminal is 80×24
- **THEN** the status line is row 23 and the ribbon area is rows 0 to 22

#### Scenario: Top placement
- **WHEN** `statusline_position` is `"top"` and the client's terminal is 80×24
- **THEN** the status line is row 0 and the ribbon area is rows 1 to 23

#### Scenario: Off
- **WHEN** `statusline_position` is `"off"`
- **THEN** no status line is drawn and the ribbon area is the whole terminal

#### Scenario: Terminal too short
- **WHEN** `statusline_height` is 2 and the client's terminal is 80×2
- **THEN** no status line is drawn and the ribbon area is the whole terminal

### Requirement: Drawing the line
Every cell of the status line SHALL be drawn with the resolved style of `StatusLine`. The components SHALL be drawn on the status line's first row, as "Layout" places them. Each span SHALL be drawn with the resolved style of its group applied over the resolved style of `StatusLine`: a field the span's style sets SHALL replace that field of `StatusLine`, and the other fields of `StatusLine` SHALL stay. Separators SHALL be drawn the same way with `StatusLineSeparator`, and the error item with `StatusLineError`. Rows after the first SHALL hold no text.

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
- **WHEN** the plugin `pane` calls `gband.ui.statusline.add({ render = fn })`
- **THEN** the call returns `"pane"`

#### Scenario: Plugin component with an id
- **WHEN** the plugin `pane` calls `gband.ui.statusline.add({ id = "count", render = fn })`
- **THEN** the call returns `"pane.count"`

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

### Requirement: Render output
A component's `render` SHALL be called with one argument, the context, and SHALL return nil, a string, or a list whose items are strings or tables `{ text = <string>, hl = <group name> }`. A string SHALL be one span in the component's `hl` group. A span table without `hl` SHALL use the component's `hl` group. Control characters, the code points U+0000 to U+001F, U+007F and U+0080 to U+009F, SHALL be removed from every span's text before it is measured or drawn. A component whose output is nil, empty, or only spans whose text is empty SHALL be hidden: it SHALL take no cell and no separator. Any other return value SHALL be a render error.

#### Scenario: Plain string
- **WHEN** a component with `hl = "StatusLineMuted"` returns `"hi"`
- **THEN** `hi` is drawn in `StatusLineMuted`

#### Scenario: Escape sequence removed
- **WHEN** a component returns `{ { text = "a\27[31mb" } }`
- **THEN** the drawn text is `a[31mb`

#### Scenario: Hidden component
- **WHEN** the left region holds A, B and C in that order and B returns `""`
- **THEN** A and C are drawn with one separator between them

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

### Requirement: Render triggers
While the status line is drawn, the client SHALL call an enabled component's `render`:
- once when the client attaches, and once after each successful reload;
- once when the component is added after the configuration has loaded;
- once each time an event its `redraw_on` names is emitted, after the handlers of that event have run;
- once each `redraw_interval` milliseconds, when the component has one;
- once when the terminal's width changes, and once when the status line starts being drawn;
- once on each `HighlightChanged` and `ColorschemeChanged`;
- for a fill component, once more after any of the triggers above, when the line has been laid out and the component's available width differs from the `width` of its latest call.

When one trigger renders several components, the components that are not fill components SHALL render first, and the fill components after them, in descending `priority`, then in the order they were added. Once those renders are done, the line SHALL be laid out, and each enabled fill component whose available width, as the context defines it, then differs from the `width` of its latest call SHALL render once more, in the same order. Those renders SHALL NOT cause further renders for the same trigger.

The client SHALL NOT call `render` at any other time. Each frame SHALL be drawn from the components' latest outputs, and drawing a frame, including every frame of an animation, SHALL NOT call `render`. A component's output SHALL stay in use until its next call.

#### Scenario: Redraw on an event
- **WHEN** a component has `redraw_on = { "FocusChanged" }` and the user focuses the column to the left
- **THEN** `render` runs once more, and the line shows its new output

#### Scenario: Not per frame
- **WHEN** a component has rendered once and the camera animates over 20 frames with no event its `redraw_on` names
- **THEN** `render` is not called again

#### Scenario: Interval
- **WHEN** a component has `redraw_interval = 1000` and no other trigger occurs for 3.5 seconds
- **THEN** `render` runs 3 times in that period

#### Scenario: Width change
- **WHEN** the client's terminal changes from 80 to 60 columns
- **THEN** every enabled component renders once with `total_width` 60

#### Scenario: Fill renders after the others
- **WHEN** the fill component F and the component M both have `redraw_on = { "KeyTableChanged" }`, M is in F's region, and the user presses Ctrl+Space, after which M's output grows from empty to `prefix`
- **THEN** F renders once, after M, and its context's `width` counts `prefix` and the separator before it

#### Scenario: Fill follows another component's width
- **WHEN** the fill component F has no `redraw_on`, and the component P, with `redraw_on = { "FocusChanged" }`, changes its output from `2/3` to `10/12`
- **THEN** F renders once more, with a context `width` two cells smaller than in its latest call

#### Scenario: Fill width unchanged
- **WHEN** the fill component F has no `redraw_on`, and the component P, with `redraw_on = { "FocusChanged" }`, changes its output from `2/3` to `3/3`
- **THEN** F does not render

### Requirement: Layout
Each shown component SHALL be placed in the region its `align` names. Within a region, components SHALL be placed in ascending `order`, and in the order they were added when their `order` is equal. Adjacent components in a region SHALL be separated by the value of the `statusline_separator` option. The width of a region SHALL be the width of its components and separators. Two adjacent non-empty regions SHALL be at least one cell apart. All widths SHALL be display widths, as `gband.ui.width` measures them.

When the regions with their gaps are wider than the status line, the shown component with the lowest `priority` SHALL be dropped, the one added last when several share it. Dropping SHALL repeat until the line fits or one component is left. When one component is left and it is still wider than the status line, it SHALL be cut to the status line's width as `gband.ui.truncate` cuts it. Dropping SHALL change only what is drawn, and SHALL NOT disable a component.

The left region SHALL start at the line's first column, and the right region SHALL end at its last column. The center region SHALL start at half of the line's width less half of its own width, rounded down, and SHALL then be moved the least distance that keeps it at least one cell from the left and right regions.

#### Scenario: Default segments
- **WHEN** the default configuration is in use, the client's terminal is 80×24, the viewed band is the first, the second of three columns is focused, and `root` is active
- **THEN** row 23 shows `band 1` from column 0 and `2/3` ending at column 79

#### Scenario: Mode shown
- **WHEN** the default configuration is in use and the user presses Ctrl+Space
- **THEN** the left region shows `band 1`, the separator, then `prefix`

#### Scenario: Lowest priority dropped first
- **WHEN** the line is 18 cells wide, the separator is ` │ `, and the left region holds A of priority 1 with output `aaaaaaaa` and B of priority 2 with output `bbbbbbbb`
- **THEN** only `bbbbbbbb` is drawn

#### Scenario: Last component cut
- **WHEN** the line is 6 cells wide and the only component returns `abcdefghij`
- **THEN** the line shows `abcde…`

#### Scenario: Wide characters
- **WHEN** the line is 5 cells wide and the only component returns `日本語`
- **THEN** the line shows `日本…`, which takes 5 cells

#### Scenario: Center region
- **WHEN** the line is 40 cells wide and the center region holds one component returning `mid`
- **THEN** `mid` starts at column 18

### Requirement: Render errors
Each call of a component's `render` SHALL run protected, as a callback that belongs to the component's plugin, with its own instruction budget of the size the plugins capability defines. A run that exceeds the budget SHALL stop only that call. The code that triggered the render and the other components SHALL continue. When a call raises an error, returns a value "Render output" does not allow, or is stopped by the instruction limit, the component SHALL be disabled and hidden until the configuration next loads, and the error SHALL be reported as a plugin error, as the configuration capability defines. A call stopped by the instruction limit SHALL also mark the component's plugin failed, as the plugins capability defines. Components of a failed plugin SHALL NOT render and SHALL be hidden.

#### Scenario: Failing component
- **WHEN** the plugin `pane`'s component raises `boom` on line 9 of its file, and another component returns `ok`
- **THEN** the client keeps running and `ok` is drawn
- **AND** the error item shows `pane: <path>:9: boom`
- **AND** `gband.ui.statusline.list()` gives the component `enabled` false

#### Scenario: Looping render
- **WHEN** a component's `render` runs `while true do end`
- **THEN** the client handles the next key
- **AND** the error item names the plugin and the instruction limit

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

### Requirement: Built-in groups
The status line SHALL give these groups default settings, as the highlights capability defines them:

| group | default | use |
|---|---|---|
| `StatusLine` | `{ reverse = true }` | the base of every status line cell |
| `StatusLineSegment` | `{}` | components' default group |
| `StatusLineSeparator` | `{ dim = true }` | separators |
| `StatusLineMuted` | `{ dim = true }` | secondary text |
| `StatusLineAccent` | `{ bold = true }` | emphasised text |
| `StatusLineError` | `{ fg = "red", bold = true }` | the error item |

The defaults SHALL exist before the init file runs. The bundled `default` colorscheme SHALL set every group in the table.

#### Scenario: Defaults without a colorscheme setting
- **WHEN** the active colorscheme sets no group
- **THEN** `StatusLineAccent` resolves to `{ bold = true }`

### Requirement: Display width helpers
`gband.ui.width(text)` SHALL return the number of terminal cells `text` takes, with control characters removed: two for each wide character, zero for each zero-width character, and one for each other character. `gband.ui.truncate(text, width)` SHALL return `text`, with its control characters removed, when it takes at most `width` cells. Otherwise it SHALL return the longest prefix that takes at most `width − 1` cells, followed by `…`. When `width` is 0, it SHALL return the empty string.

#### Scenario: Width
- **WHEN** code calls `gband.ui.width("a日b")`
- **THEN** it returns 4

#### Scenario: Truncate at a wide character
- **WHEN** code calls `gband.ui.truncate("日本語", 4)`
- **THEN** it returns `日…`

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
