# lua-control Specification

## Purpose

Defines how Lua code reads the client's layout and view, and how it performs every operation a user can, on any window or band it names: targeted actions, exact widths and heights, focus, and input to a window.

## Requirements

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

### Requirement: Read the view
`gband.view()` SHALL return a new table holding:
- `band`: the viewed band's number.
- `window`: the focused window's number, or nil when no window is focused.
- `floating`: `true` when the floating layer of the viewed band is active, and `false` otherwise.
- `plugin_window`: the focused plugin window's number, as the plugin-windows capability defines it, or nil.
- `table`: the active key table.
- `cols` and `rows`: the size of the ribbon area.

Calling it while the configuration loads SHALL be an error at the line of the call.

#### Scenario: Focused window
- **WHEN** the client views band 1 with window 2 focused, its ribbon area is 80×23, and a binding in `prefix` calls `gband.view()`
- **THEN** the result is `{ band = 1, window = 2, floating = false, table = "prefix", cols = 80, rows = 23 }`

#### Scenario: Empty band
- **WHEN** the client views the empty band and a binding function calls `gband.view()`
- **THEN** the result's `window` is nil

#### Scenario: Floating focus
- **WHEN** the client focuses floating window 3 and a binding function calls `gband.view()`
- **THEN** the result's `window` is 3 and its `floating` is `true`

### Requirement: State as of the call
`gband.layout()` and `gband.view()` SHALL describe the client's state when they are called. Actions and calls that the running callback dispatched SHALL NOT be reflected, because they take effect after it returns.

#### Scenario: Action not yet applied
- **WHEN** a binding function calls `gband.action.focus_column_right()` and then `gband.view()`, with the first of two columns focused
- **THEN** the view's `window` names the window of the first column
- **AND** after the function returns, the second column is focused

### Requirement: Action targets
A built-in session action value SHALL accept one optional table, its target. A target SHALL take effect only when the action value is called. For `close_window`, `consume_or_expel_left`, `consume_or_expel_right`, `move_column_left`, `move_column_right`, `move_window_down`, `move_window_up`, `cycle_column_width`, `toggle_full_width`, `grow_column_width`, `shrink_column_width`, `grow_window_height`, `shrink_window_height` and `reset_window_height`, the target SHALL hold `window`. The action SHALL then name that window instead of the focused window. For `open_window`, the target SHALL hold `band`, `after`, `floating`, or any of them together:
- With both `band` and `after`, the new column follows `after`'s column in `band`.
- With only `band`, the new column is that band's first column.
- With only `after`, the band is the band that holds `after`.
- `floating`, a boolean, opens the window floating when `true`, in `band`, or in the viewed band when the target holds no `band`. A target holding `after` with `floating = true` SHALL be an error. `floating = false` SHALL open the window tiled, as a target without `floating` does.

For `toggle_window_floating`, the target SHALL hold `window`, and optionally `after`, a tiled window of the same band. Tiling a floating window SHALL then place it after `after`. Without `after`, it SHALL be placed after the tiled window this client focused most recently in that band, when it is still tiled there, and as the band's first column otherwise. `after` on a window that is tiled SHALL be ignored.

The target of `toggle_window_floating` MAY also hold `floating`, a boolean. With `floating = true`, the action SHALL float `window` when it is tiled, and SHALL change nothing when it already floats. With `floating = false`, the action SHALL tile `window` when it floats, placed as the previous paragraph defines, and SHALL change nothing when it is already tiled. Without `floating`, the action SHALL toggle `window`. The client SHALL send the action with the layer that `floating` names, whatever layer its own layout shows `window` in. The server SHALL decide whether the window moves, as the session-server capability's "Session actions" defines, so a request that another client's request has already satisfied SHALL change nothing.

The target of `grow_column_width`, `shrink_column_width`, `grow_window_height` and `shrink_window_height` MAY also hold `step`: a number greater than 0 and at most 10000 for the two width actions, and greater than 0 and at most 1 for the two height actions, read as the configuration capability reads a step. The action SHALL then use that step, as the actions capability defines. A target of these actions that holds `step` and no `window` SHALL name the window the view resolves.

`send_prefix` SHALL accept a target holding `window`, and send the prefix key to that window.

`drag_resize_window` SHALL accept a target holding `edges`, a list of edge names. Each name SHALL be `"left"`, `"right"`, `"top"` or `"bottom"`. The gesture the action starts SHALL move exactly the edges the list names, as the mouse capability's "Resize by dragging" defines. The order of the names SHALL NOT matter. The target SHALL NOT change when the action starts a gesture. It SHALL start one only while the client handles a mouse press, as the mouse capability's "Drag gestures" defines. Called with no target, `drag_resize_window` SHALL pick its edges from the press's cell, as "Resize by dragging" defines.

A target passed to a view action, or to a client action other than `send_prefix` and `drag_resize_window`, SHALL be an error. These client actions are `detach`, `drag_window` and `drag_band`. Each of these SHALL also be an error at the line of the call:
- A target that is not a table.
- A field the action does not take.
- A `step` out of its range.
- A window or band number not in the client's layout.
- An `after` window not in `band`, or a floating `after` window for `open_window` or `gband.spawn`, or, for `toggle_window_floating`, an `after` that is not a tiled window of `window`'s band.
- For `toggle_window_floating`, a target without `window`, a `floating` that is not a boolean, or `after` together with `floating = true`.
- For `drag_resize_window`, a target without `edges`, an `edges` that is not a list of strings, an empty list, a name that is not an edge name, a name given twice, both `"left"` and `"right"`, or both `"top"` and `"bottom"`.

A call that raises one of these errors SHALL dispatch nothing.

Calling an action value with no target SHALL resolve it against the view, as the actions capability defines.

#### Scenario: Close a named window
- **WHEN** windows 1 and 2 are open with window 2 focused and a binding function calls `gband.action.close_window({ window = 1 })`
- **THEN** the client sends close window naming window 1
- **AND** window 2 stays focused

#### Scenario: Grow a named column
- **WHEN** the only column of band 1 holds window 1 at width 1/2, the client views band 2, and a binding function calls `gband.action.grow_column_width({ window = 1 })`
- **THEN** window 1's column has width 3/5

#### Scenario: Open in another band
- **WHEN** the client views band 1 and a binding function calls `gband.action.open_window({ band = 2 })`, where band 2 is the empty last band
- **THEN** band 2 holds a new window running the user's shell
- **AND** the client views band 2 with the new window focused

#### Scenario: Unknown window
- **WHEN** line 6 of `user/init.lua` holds a binding function that calls `gband.action.close_window({ window = 99 })` and no window 99 exists, and the user presses its key
- **THEN** the client shows an error at `user/init.lua` line 6 naming window 99
- **AND** nothing is sent to the server

#### Scenario: Target on a view action
- **WHEN** a binding function calls `gband.action.focus_column_left({ window = 1 })`
- **THEN** the call raises an error and the view is unchanged

#### Scenario: Open a floating window
- **WHEN** the client views band 1 and a binding function calls `gband.action.open_window({ floating = true })`
- **THEN** band 1's floating list ends with a new window running the user's shell
- **AND** the client focuses it with the floating layer active

#### Scenario: Floating with after
- **WHEN** a binding function calls `gband.action.open_window({ after = 1, floating = true })`
- **THEN** the call raises an error naming `after`

#### Scenario: Tile a named window after a named window
- **WHEN** band 1 holds columns with windows 1 and 2 and floating window 3, and a binding function calls `gband.action.toggle_window_floating({ window = 3, after = 1 })`
- **THEN** band 1 holds the columns of window 1, window 3 and window 2, in that order

#### Scenario: Float a window that already floats
- **WHEN** band 1 holds a column with window 1 and floating window 3 with `col` 5, `row` 3, width 1/3 and `rows` 10, and a binding function calls `gband.action.toggle_window_floating({ window = 3, floating = true })`
- **THEN** the client sends toggle floating naming window 3 and the floating layer
- **AND** window 3 stays in band 1's floating list with `col` 5, `row` 3, width 1/3 and `rows` 10

#### Scenario: Float twice in one callback
- **WHEN** band 1 holds columns with windows 1 and 2, and a binding function calls `gband.action.toggle_window_floating({ window = 2, floating = true })` twice
- **THEN** band 1 holds the column of window 1 and one floating window, window 2

#### Scenario: Tile a named window with floating false
- **WHEN** band 1 holds columns with windows 1 and 2 and floating window 3, and a binding function calls `gband.action.toggle_window_floating({ window = 3, after = 1, floating = false })`
- **THEN** band 1 holds the columns of window 1, window 3 and window 2, in that order

#### Scenario: Tile a window that is already tiled
- **WHEN** band 1 holds columns with windows 1 and 2, and a binding function calls `gband.action.toggle_window_floating({ window = 2, floating = false })`
- **THEN** band 1 holds the columns of window 1 and window 2, in that order, and no floating window

#### Scenario: Floating that is not a boolean
- **WHEN** line 7 of `user/init.lua` holds a binding function that calls `gband.action.toggle_window_floating({ window = 3, floating = "yes" })`, window 3 is open, and the user presses its key
- **THEN** the client shows an error at `user/init.lua` line 7 naming `floating`
- **AND** nothing is sent to the server

#### Scenario: Float with after
- **WHEN** band 1 holds columns with windows 1 and 2, and a binding function calls `gband.action.toggle_window_floating({ window = 2, after = 1, floating = true })`
- **THEN** the call raises an error naming `after`, and nothing is dispatched

#### Scenario: Floating without a window
- **WHEN** a binding function calls `gband.action.toggle_window_floating({ floating = true })`
- **THEN** the call raises an error naming `window`, and nothing is dispatched

#### Scenario: Move a named window
- **WHEN** band 1 holds columns with windows 1 and 2, window 2 is focused, and a binding function calls `gband.action.move_column_right({ window = 1 })`
- **THEN** band 1 holds the columns of window 2 and window 1, in that order, and window 2 stays focused

#### Scenario: Grow by a given step
- **WHEN** window 1 is focused alone in a column of width 1/2 and a binding function calls `gband.action.grow_column_width({ step = 1/4 })`
- **THEN** window 1's column has width 3/4

#### Scenario: Step out of range
- **WHEN** line 4 of a binding function's file calls `gband.action.grow_window_height({ step = 2 })`
- **THEN** the call raises an error at line 4 naming `step`, and nothing is sent

#### Scenario: Resize by named edges
- **WHEN** navigation mode binds `rightmouse` to a function that calls `gband.action.drag_resize_window({ edges = { "bottom" } })`, the screen area is 80×24, a floating window's box is 30×12 at column 10 and row 4, and the user drags with the right button from column 12 and row 15 to column 8 and row 18
- **THEN** the box is 30 cells wide at column 10 and row 4, and 15 rows high

#### Scenario: Order of edge names
- **WHEN** a binding function calls `gband.action.drag_resize_window({ edges = { "top", "left" } })` while the client handles a mouse press
- **THEN** the gesture moves the left and top edges, as it does for `{ edges = { "left", "top" } }`

#### Scenario: Named edges from a key
- **WHEN** navigation mode binds `r` to a function that calls `gband.action.drag_resize_window({ edges = { "right" } })` and the user presses `r`
- **THEN** no error is shown and nothing changes

#### Scenario: Opposite edges
- **WHEN** line 5 of `user/init.lua` holds a binding function for `rightmouse` that calls `gband.action.drag_resize_window({ edges = { "left", "right" } })`, and the user presses the right button on a window
- **THEN** the client shows an error at `user/init.lua` line 5 naming `left` and `right`
- **AND** no gesture starts and nothing is sent to the server

#### Scenario: Empty edge list
- **WHEN** a binding function calls `gband.action.drag_resize_window({ edges = {} })`
- **THEN** the call raises an error naming `edges`, and nothing is dispatched

#### Scenario: Unknown edge name
- **WHEN** a binding function calls `gband.action.drag_resize_window({ edges = { "middle" } })`
- **THEN** the call raises an error naming `middle`, and nothing is dispatched

#### Scenario: Edge named twice
- **WHEN** a binding function calls `gband.action.drag_resize_window({ edges = { "top", "top" } })`
- **THEN** the call raises an error naming `top`, and nothing is dispatched

#### Scenario: Other field beside edges
- **WHEN** a binding function calls `gband.action.drag_resize_window({ edges = { "top" }, window = 1 })`
- **THEN** the call raises an error naming `window`, and nothing is dispatched

#### Scenario: Target on another drag action
- **WHEN** a binding function calls `gband.action.drag_window({ edges = { "left" } })`
- **THEN** the call raises an error naming `drag_window`, and nothing is dispatched

### Requirement: Focus and view by number
`gband.window.focus(window)` SHALL dispatch a view action that focuses the named window, as the layout-view capability defines. `gband.band.view(band)` SHALL dispatch a view action that views the named band, as the layout-view capability defines. A number that names no window or band in the client's layout SHALL be an error at the line of the call.

#### Scenario: Focus a window in another band
- **WHEN** band 1 holds window 1, band 2 holds window 2, the client views band 1, and a binding function calls `gband.window.focus(2)`
- **THEN** the client views band 2 with window 2 focused

#### Scenario: View a band
- **WHEN** the client last focused window 4 in band 2, views band 1, and a binding function calls `gband.band.view(2)`
- **THEN** the client views band 2 with window 4 focused

### Requirement: Exact sizes
`gband.window.set_width(window, width)` SHALL dispatch set width naming the window and the width, as the layout capability defines it. The width SHALL be a number greater than 0 and at most 10000, read as the configuration capability reads a width option.

`gband.window.set_height(window, height)` SHALL dispatch set height naming the window. The `height` argument SHALL be a table holding exactly one of two fields:
- `rows`: an integer of at least 1, which gives a fixed height.
- `weight`: a number greater than 0 and at most 10000, read as a width is read, which gives an automatic height.

A window number not in the client's layout, or an argument of the wrong type or out of range, SHALL be an error at the line of the call.

#### Scenario: Set a width
- **WHEN** a binding function calls `gband.window.set_width(1, 0.4)`
- **THEN** window 1's column has width 2/5 and full width off

#### Scenario: Set a fixed height
- **WHEN** the screen area is 80×24, a column holds windows 1 and 2 at weight 1, and a binding function calls `gband.window.set_height(1, { rows = 8 })`
- **THEN** window 1's tile is 8 rows high and window 2's is 16

#### Scenario: Both fields
- **WHEN** a binding function calls `gband.window.set_height(1, { rows = 8, weight = 1 })`
- **THEN** the call raises an error

### Requirement: Input to a named window
`gband.window.send_keys(window, keys)` SHALL send each key, given as one key name or a list of key names as the configuration capability defines them, to the named window as a key press. `gband.window.send_text(window, text)` SHALL send each character of `text` to the named window as a key press of that character: a line feed or carriage return as Enter, and a tab as Tab. Any other control character SHALL be an error. `gband.window.paste(window, text)` SHALL send `text` to the named window as a paste. The server SHALL encode them as it encodes keys and pastes the user sends. These functions SHALL NOT change the active key table or run any binding. An invalid key name or a window number not in the client's layout SHALL be an error at the line of the call.

#### Scenario: Run a command in another window
- **WHEN** windows 1 and 2 run shells, window 2 is focused, and a binding function calls `gband.window.send_text(1, "echo hi\n")`
- **THEN** window 1 prints `hi`
- **AND** window 2's screen is unchanged and window 2 stays focused

#### Scenario: Interrupt a program
- **WHEN** window 1 runs `sleep 100` and a binding function calls `gband.window.send_keys(1, "ctrl+c")`
- **THEN** `sleep` stops and window 1 shows a prompt

#### Scenario: Paste
- **WHEN** window 1 runs a program with bracketed paste enabled and a binding function calls `gband.window.paste(1, "a b")`
- **THEN** the program receives `a b` between the bracketed paste markers

#### Scenario: Prefix key is not a binding
- **WHEN** a binding function calls `gband.window.send_keys(1, "ctrl+space")`
- **THEN** window 1 receives `\x00` and the active key table does not change

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

### Requirement: Place a floating window
`gband.window.set_position(window, position)` SHALL dispatch set position naming the window, as the session-server capability defines it. The `position` argument SHALL be a table holding `col` and `row`, each an integer of at least 0. The box SHALL be placed as the floating-windows capability defines. A window number not in the client's layout, a window that is not floating in the client's layout, a missing or unknown field, or a value of the wrong type or out of range SHALL be an error at the line of the call.

#### Scenario: Place a floating window
- **WHEN** the screen area is 80×24, window 3 floats with a 40×12 box, and a binding function calls `gband.window.set_position(3, { col = 10, row = 2 })`
- **THEN** window 3's box spans columns 10 to 49 and rows 2 to 13

#### Scenario: Place past the edge
- **WHEN** the same binding function calls `gband.window.set_position(3, { col = 70, row = 2 })`
- **THEN** window 3's box spans columns 40 to 79

#### Scenario: Tiled window
- **WHEN** window 1 is tiled and a binding function calls `gband.window.set_position(1, { col = 0, row = 0 })`
- **THEN** the call raises an error naming window 1

#### Scenario: Missing field
- **WHEN** window 3 floats and a binding function calls `gband.window.set_position(3, { col = 4 })`
- **THEN** the call raises an error naming `row`

### Requirement: Removed pane names
The Lua names that gband 0.1.0 gave windows SHALL no longer work, in the client or the server. Using one SHALL be an error at the line of the use, whose message names the replacement, such as "`gband.pane` is now `gband.window`". The uses that SHALL raise it are:

- reading `gband.pane` or `gband.pane_state`
- reading `gband.action.open_pane`, `close_pane`, `focus_pane_down`, `focus_pane_up`, `move_pane_down`, `move_pane_up`, `toggle_pane_floating`, `grow_pane_height`, `shrink_pane_height` or `reset_pane_height`, which are now `open_window`, `close_window`, `focus_window_down`, `focus_window_up`, `move_window_down`, `move_window_up`, `toggle_window_floating`, `grow_window_height`, `shrink_window_height` and `reset_window_height`
- naming `PaneOpened`, `PaneClosed`, `PaneExited`, `PaneOutput`, `PaneInput` or `PaneStateChanged` to `gband.on`, which are now `WindowOpened`, `WindowClosed`, `WindowExited`, `WindowOutput`, `WindowInput` and `WindowStateChanged`
- an action target holding the field `pane`, which is now `window`
- `gband.win.open` with `kind = "pane"` or `kind = "float"`, which are now `kind = "tiled"` and `kind = "floating"`

Fields that gband fills in, such as `gband.view().pane`, the layout's `panes` and an event payload's `pane`, SHALL be absent, so reading one gives nil. The environment variable `GBAND_PANE` SHALL no longer be set; `GBAND_WINDOW` SHALL be set in its place.

#### Scenario: Old action name
- **WHEN** line 3 of `user/init.lua` calls `gband.bind("prefix x", gband.action.close_pane)`
- **THEN** the configuration fails to load with an error at line 3 whose message names `close_window`

#### Scenario: Old event name
- **WHEN** a plugin's `client.lua` calls `gband.on("PaneOpened", fn)` on line 5
- **THEN** the plugin fails with an error at line 5 whose message names `WindowOpened`

#### Scenario: Old target field
- **WHEN** a binding function calls `gband.action.close_window({ pane = 1 })`
- **THEN** the call raises an error whose message names the field `window`, and nothing is dispatched

#### Scenario: Old plugin window kind
- **WHEN** a binding function calls `gband.win.open({ kind = "pane" })`
- **THEN** the call raises an error whose message names `"tiled"`, and no plugin window opens

#### Scenario: Old floating kind
- **WHEN** a binding function calls `gband.win.open({ kind = "float" })`
- **THEN** the call raises an error whose message names `"floating"`, and no plugin window opens

#### Scenario: Old environment variable
- **WHEN** a window runs `echo "[$GBAND_PANE] $GBAND_WINDOW"` as window 2
- **THEN** it prints `[] 2`
