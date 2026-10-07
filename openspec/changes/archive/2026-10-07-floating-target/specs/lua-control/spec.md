## MODIFIED Requirements

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
