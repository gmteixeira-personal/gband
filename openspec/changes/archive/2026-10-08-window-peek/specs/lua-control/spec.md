## MODIFIED Requirements

### Requirement: Read the layout
`gband.layout()` SHALL return a new table describing the layout the client holds:

- `cols` and `rows`: the size of the screen area in the latest layout.
- `bands`: one table per band, in order. Each band table holds `id`, the band's number, `columns`, and `floating`.
- `columns`: one table per column, left to right. Each column table holds `width`, the column's width as a number, `full_width`, a boolean, and `windows`.
- `windows`: one table per window, top to bottom. Each window table holds `id`, the window's number, and either `rows`, the fixed height in rows, or `weight`, the weight of an automatic height as a number. It holds `plugin_window`, the plugin window's number, when the window is a drawn window of a plugin window this client opened. A window that runs a program also holds `name`, its shown name as the window-names capability defines it, and `manual_name`, its manual name, when it has one. It holds `last_focus`, the window's last focus as the layout-view capability's "Focus order" defines it, an integer, when this client has focused the window, and no `last_focus` otherwise.
- `floating`: one table per floating window, in the band's floating list order. Each holds `id`, `width`, the box's width as a number, `full_width`, a boolean, `rows`, the box record's height, `col` and `row`, the box's top-left cell as placed in the screen area, and `plugin_window`, `name`, `manual_name` and `last_focus` as a window table holds them. It holds `minimized`, `true`, when this client has minimized the window, as the floating-windows capability defines, and no `minimized` otherwise.

A higher `last_focus` SHALL mean a more recent focus by this client. A peek SHALL change no `last_focus`. The floating windows of a band that this client has not minimized SHALL be in this client's stacking order, as the floating-windows capability defines, from bottom to top: those without `last_focus` in the order of `floating`, then the others by ascending `last_focus`.

Changing the returned table SHALL NOT change the layout. Calling `gband.layout()` while the configuration loads SHALL be an error at the line of the call.

#### Scenario: Two columns
- **WHEN** the screen area is 80×24, the first band holds a column of width 1/2 with window 1, and a full-width column of width 1/3 with window 2 at a fixed height of 10 rows above window 3 at weight 1, the client attached with window 1 focused and has focused no other window, and a binding function calls `gband.layout()`
- **THEN** the result's `cols` is 80 and `rows` is 24
- **AND** its first band's columns are `{ width = 0.5, full_width = false, windows = { { id = 1, weight = 1, last_focus = 1 } } }` and a column with `width` 1/3 and `full_width` `true` whose windows are `{ id = 2, rows = 10 }` and `{ id = 3, weight = 1 }`
- **AND** its last band has no columns and no floating windows

#### Scenario: While loading
- **WHEN** line 4 of `user/init.lua` calls `gband.layout()`
- **THEN** loading fails with an error at `user/init.lua` line 4

#### Scenario: Floating window
- **WHEN** the screen area is 80×24 and band 1 holds window 1 in a column and floating window 2 with `col` 50, `row` 3, width 1/2, full width off and `rows` 10, this client has never focused window 2, and a binding function calls `gband.layout()`
- **THEN** its first band's `floating` is `{ { id = 2, width = 0.5, full_width = false, rows = 10, col = 40, row = 3 } }`

#### Scenario: Window names
- **WHEN** band 1 holds windows 1 and 2, both running `bash` with no title set, window 3 running `vim`, and window 3 is renamed `notes`, and a binding function calls `gband.layout()`
- **THEN** window 1's table holds `name = "bash #1"`, window 2's `name = "bash #2"`, and window 3's `name = "notes"` and `manual_name = "notes"`
- **AND** windows 1 and 2 hold no `manual_name`

#### Scenario: Drawn window has no name
- **WHEN** a client plugin opens a tiled plugin window and a binding function calls `gband.layout()`
- **THEN** the table of its drawn window holds `plugin_window` and no `name`

#### Scenario: Minimized floating window
- **WHEN** band 1 holds window 1 in a column and floating windows 2 and 3, this client has minimized window 3, and a binding function calls `gband.layout()`
- **THEN** window 3's floating table holds `minimized = true`
- **AND** window 2's floating table holds no `minimized`

#### Scenario: Minimized by another client
- **WHEN** band 1 holds floating window 2, another client has minimized it, and a binding function of this client calls `gband.layout()`
- **THEN** window 2's floating table holds no `minimized`

#### Scenario: Last focus
- **WHEN** band 1 holds the columns of windows 1 and 2, the client attached with window 1 focused, then focused window 2 and window 1 again, and a binding function calls `gband.layout()`
- **THEN** window 1's table holds `last_focus = 3` and window 2's table holds `last_focus = 2`

#### Scenario: Last focus gives the stacking order
- **WHEN** band 1 holds window 1 in a column and floating windows 3, 4 and 5, the client attached with window 1 focused, focused 5 and then 3, and a binding function calls `gband.layout()`
- **THEN** window 4's floating table holds no `last_focus`, window 5's holds `last_focus = 2` and window 3's holds `last_focus = 3`
- **AND** the client draws 4 at the bottom, 5 over it and 3 on top

#### Scenario: Peek keeps the last focus
- **WHEN** band 1 holds floating windows 3 and 4, the client focused 3 and then 4, and a binding function calls `gband.window.focus(3, { peek = true })` and a later binding function calls `gband.layout()`
- **THEN** window 4's floating table holds a higher `last_focus` than window 3's

### Requirement: Read the view
`gband.view()` SHALL return a new table holding:
- `band`: the viewed band's number.
- `window`: the focused window's number, or nil when no window is focused.
- `floating`: `true` when the floating layer of the viewed band is active, and `false` otherwise.
- `plugin_window`: the focused plugin window's number, as the plugin-windows capability defines it, or nil.
- `peek`: `true` while the focused window is a window this client peeks, as the layout-view capability's "Peek a window" defines, and absent otherwise.
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

#### Scenario: Peeked focus
- **WHEN** band 1 holds window 1 in a column and floating window 3, window 1 is focused, a binding function calls `gband.window.focus(3, { peek = true })`, and a later binding function calls `gband.view()`
- **THEN** the result's `window` is 3, its `floating` is `true` and its `peek` is `true`
- **AND** after a binding function calls `gband.window.focus(1)`, `gband.view()` holds `window` 1 and no `peek`

### Requirement: Focus and view by number
`gband.window.focus(window, opts)` SHALL dispatch a view action that focuses the named window, as the layout-view capability's "Focus a named window" defines. `opts` SHALL be optional. When given, it SHALL be a table that holds at most one field, `peek`, a boolean. With `peek = true`, the view action SHALL peek the named window instead, as the layout-view capability's "Peek a window" defines. Without `peek`, or with `peek = false`, it SHALL focus the window. `gband.band.view(band)` SHALL dispatch a view action that views the named band, as the layout-view capability defines. Each of these SHALL be an error at the line of the call, and a call that raises one SHALL dispatch nothing:

- A number that names no window or band in the client's layout.
- An `opts` that is neither nil nor a table.
- A field of `opts` other than `peek`.
- A `peek` that is not a boolean.

#### Scenario: Focus a window in another band
- **WHEN** band 1 holds window 1, band 2 holds window 2, the client views band 1, and a binding function calls `gband.window.focus(2)`
- **THEN** the client views band 2 with window 2 focused

#### Scenario: View a band
- **WHEN** the client last focused window 4 in band 2, views band 1, and a binding function calls `gband.band.view(2)`
- **THEN** the client views band 2 with window 4 focused

#### Scenario: Peek a minimized window by number
- **WHEN** floating windows 3 and 4 of band 1 have overlapping boxes, the client focused 3 and then 4 and minimized 3, and a binding function calls `gband.window.focus(3, { peek = true })`
- **THEN** after the function returns, the client focuses window 3 and draws it over window 4
- **AND** a later `gband.layout()` gives window 3's floating table `minimized = true`

#### Scenario: Peek false focuses
- **WHEN** floating window 3 of band 1 is minimized by this client, and a binding function calls `gband.window.focus(3, { peek = false })`
- **THEN** after the function returns, the client focuses window 3, and a later `gband.layout()` gives window 3's floating table no `minimized`

#### Scenario: Unknown option
- **WHEN** line 5 of `user/init.lua` holds a binding function that calls `gband.window.focus(1, { raise = true })`, window 1 is open, and the user presses its key
- **THEN** the client shows an error at `user/init.lua` line 5 naming `raise`
- **AND** the focus does not change

#### Scenario: Peek that is not a boolean
- **WHEN** a binding function calls `gband.window.focus(1, { peek = "yes" })`
- **THEN** the call raises an error naming `peek`, and nothing is dispatched

#### Scenario: Options that are not a table
- **WHEN** a binding function calls `gband.window.focus(1, true)`
- **THEN** the call raises an error, and nothing is dispatched

### Requirement: Minimize a window by number
`gband.window.minimize(window)` SHALL dispatch a view action that minimizes the named window in this client's view, as the floating-windows capability defines. It SHALL send the server no action. A window number not in the client's layout, or a window that is not floating in the client's layout, SHALL be an error at the line of the call whose message names the window. An argument that is not a window number SHALL be an error at the line of the call. A window this client has already minimized SHALL stay minimized, and SHALL NOT be an error. Focusing the window with `gband.window.focus` without `peek = true` SHALL restore it. Peeking it SHALL NOT restore it.

#### Scenario: Minimize an unfocused floating window
- **WHEN** band 1 holds window 1 in a column and floating windows 2 and 3, window 3 is focused, and a binding function calls `gband.window.minimize(2)`
- **THEN** the client does not draw window 2, and window 3 stays focused
- **AND** the client sends the server no action

#### Scenario: Minimize the focused window
- **WHEN** band 1 holds window 1 in a column and floating window 2, window 2 is focused, and a binding function calls `gband.window.minimize(2)`
- **THEN** after the function returns, `gband.view()` has `window` 1 and `floating` false

#### Scenario: Minimize a window of another band
- **WHEN** the client views band 1, band 2 holds only floating window 4, and a binding function calls `gband.window.minimize(4)` and then `gband.band.view(2)`
- **THEN** the client views band 2 with no focused window

#### Scenario: Tiled window
- **WHEN** window 1 is tiled and line 5 of a binding function's file calls `gband.window.minimize(1)`
- **THEN** the call raises an error at line 5 naming window 1, and nothing is dispatched

#### Scenario: Unknown window
- **WHEN** no window 99 exists and a binding function calls `gband.window.minimize(99)`
- **THEN** the call raises an error naming window 99, and nothing is dispatched

#### Scenario: Restore with focus
- **WHEN** window 2 floats, this client has minimized it, and a binding function calls `gband.window.focus(2)`
- **THEN** the client draws window 2 on top and focuses it
- **AND** a later `gband.layout()` gives window 2's floating table no `minimized`

#### Scenario: Peek does not restore
- **WHEN** window 2 floats, this client has minimized it, and a binding function calls `gband.window.focus(2, { peek = true })`
- **THEN** the client draws window 2 on top and focuses it
- **AND** a later `gband.layout()` gives window 2's floating table `minimized = true`
