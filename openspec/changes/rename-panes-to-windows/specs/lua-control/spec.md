## RENAMED Requirements

- FROM: `### Requirement: Input to a named pane`
- TO: `### Requirement: Input to a named window`

## MODIFIED Requirements

### Requirement: Read the layout
`gband.layout()` SHALL return a new table describing the layout the client holds:

- `cols` and `rows`: the size of the screen area in the latest layout.
- `bands`: one table per band, in order. Each band table holds `id`, the band's number, and `columns`.
- `columns`: one table per column, left to right. Each column table holds `width`, the column's width as a number, `full_width`, a boolean, and `windows`.
- `windows`: one table per window, top to bottom. Each window table holds `id`, the window's number, and either `rows`, the fixed height in rows, or `weight`, the weight of an automatic height as a number. It holds `plugin_window`, the plugin window's number, when the window is a drawn window of a plugin window this client opened.

Changing the returned table SHALL NOT change the layout. Calling `gband.layout()` while the configuration loads SHALL be an error at the line of the call.

#### Scenario: Two columns
- **WHEN** the screen area is 80×24, the first band holds a column of width 1/2 with window 1, and a full-width column of width 1/3 with window 2 at a fixed height of 10 rows above window 3 at weight 1, and a binding function calls `gband.layout()`
- **THEN** the result's `cols` is 80 and `rows` is 24
- **AND** its first band's columns are `{ width = 0.5, full_width = false, windows = { { id = 1, weight = 1 } } }` and a column with `width` 1/3 and `full_width` `true` whose windows are `{ id = 2, rows = 10 }` and `{ id = 3, weight = 1 }`
- **AND** its last band has no columns

#### Scenario: While loading
- **WHEN** line 4 of `user/init.lua` calls `gband.layout()`
- **THEN** loading fails with an error at `user/init.lua` line 4

### Requirement: Read the view
`gband.view()` SHALL return a new table holding:
- `band`: the viewed band's number.
- `window`: the focused window's number, or nil when no window is focused.
- `plugin_window`: the focused plugin window's number, as the plugin-windows capability defines it, or nil.
- `table`: the active key table.
- `cols` and `rows`: the size of the ribbon area.

Calling it while the configuration loads SHALL be an error at the line of the call.

#### Scenario: Focused pane
- **WHEN** the client views band 1 with window 2 focused, its ribbon area is 80×23, and a binding in `prefix` calls `gband.view()`
- **THEN** the result is `{ band = 1, window = 2, table = "prefix", cols = 80, rows = 23 }`

#### Scenario: Empty band
- **WHEN** the client views the empty band and a binding function calls `gband.view()`
- **THEN** the result's `window` is nil

### Requirement: State as of the call
`gband.layout()` and `gband.view()` SHALL describe the client's state when they are called. Actions and calls that the running callback dispatched SHALL NOT be reflected, because they take effect after it returns.

#### Scenario: Action not yet applied
- **WHEN** a binding function calls `gband.action.focus_column_right()` and then `gband.view()`, with the first of two columns focused
- **THEN** the view's `window` names the window of the first column
- **AND** after the function returns, the second column is focused

### Requirement: Action targets
A built-in session action value SHALL accept one optional table, its target. A target SHALL take effect only when the action value is called. For `close_window`, `consume_or_expel_left`, `consume_or_expel_right`, `cycle_column_width`, `toggle_full_width`, `grow_column_width`, `shrink_column_width`, `grow_window_height`, `shrink_window_height` and `reset_window_height`, the target SHALL hold `window`. The action SHALL then name that window instead of the focused window. For `open_window`, the target SHALL hold `band`, `after`, or both:
- With both, the new column follows `after`'s column in `band`.
- With only `band`, the new column is that band's first column.
- With only `after`, the band is the band that holds `after`.

`send_prefix` SHALL accept a target holding `window`, and send the prefix key to that window.

A target passed to a view action or to `detach` SHALL be an error. Each of these SHALL also be an error at the line of the call:
- A target that is not a table.
- A field the action does not take.
- A window or band number not in the client's layout.
- An `after` window not in `band`.

Calling an action value with no target SHALL resolve it against the view, as the actions capability defines.

#### Scenario: Close a named pane
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

#### Scenario: Unknown pane
- **WHEN** line 6 of `user/init.lua` holds a binding function that calls `gband.action.close_window({ window = 99 })` and no window 99 exists, and the user presses its key
- **THEN** the client shows an error at `user/init.lua` line 6 naming window 99
- **AND** nothing is sent to the server

#### Scenario: Target on a view action
- **WHEN** a binding function calls `gband.action.focus_column_left({ window = 1 })`
- **THEN** the call raises an error and the view is unchanged

### Requirement: Focus and view by number
`gband.window.focus(window)` SHALL dispatch a view action that focuses the named window, as the layout-view capability defines. `gband.band.view(band)` SHALL dispatch a view action that views the named band, as the layout-view capability defines. A number that names no window or band in the client's layout SHALL be an error at the line of the call.

#### Scenario: Focus a pane in another band
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

#### Scenario: Run a command in another pane
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
`gband.window.focus`, `gband.window.set_width`, `gband.window.set_height`, `gband.window.send_keys`, `gband.window.send_text`, `gband.window.paste` and `gband.band.view` SHALL be callable wherever an action value is, and an error elsewhere, as the configuration capability defines for action values. Each SHALL be dispatched like an action. It SHALL take effect after the callback returns, in the order dispatched together with the callback's actions.

#### Scenario: Order with actions
- **WHEN** a binding function calls `gband.window.set_width(1, 1/3)` and then `gband.action.cycle_column_width({ window = 1 })`
- **THEN** window 1's column has width 1/2

#### Scenario: During loading
- **WHEN** line 3 of `user/init.lua` calls `gband.window.focus(1)` at the top level
- **THEN** loading fails with an error at `user/init.lua` line 3

## ADDED Requirements

### Requirement: Removed pane names
The Lua names that gband 0.1.0 gave windows SHALL no longer work, in the client or the server. Using one SHALL be an error at the line of the use, whose message names the replacement, such as ``` `gband.pane` is now `gband.window` ```. The uses that SHALL raise it are:

- reading `gband.pane` or `gband.pane_state`
- reading `gband.action.open_pane`, `close_pane`, `focus_pane_down`, `focus_pane_up`, `grow_pane_height`, `shrink_pane_height` or `reset_pane_height`, which are now `open_window`, `close_window`, `focus_window_down`, `focus_window_up`, `grow_window_height`, `shrink_window_height` and `reset_window_height`
- naming `PaneOpened`, `PaneClosed`, `PaneExited`, `PaneOutput`, `PaneInput` or `PaneStateChanged` to `gband.on` or in a status line component's `redraw_on`, which are now `WindowOpened`, `WindowClosed`, `WindowExited`, `WindowOutput`, `WindowInput` and `WindowStateChanged`
- an action target holding the field `pane`, which is now `window`
- `gband.win.open` with `kind = "pane"`, which is now `kind = "tiled"`
- naming an old action in the hints segment's `labels`

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

#### Scenario: Old environment variable
- **WHEN** a window runs `echo "[$GBAND_PANE] $GBAND_WINDOW"` as window 2
- **THEN** it prints `[] 2`
