# lua-control Specification

## Purpose

Defines how Lua code reads the client's layout and view, and how it performs every operation a user can, on any pane or band it names: targeted actions, exact widths and heights, focus, and input to a pane.

## Requirements

### Requirement: Read the layout
`gband.layout()` SHALL return a new table describing the layout the client holds:

- `cols` and `rows`: the size of the screen area in the latest layout.
- `bands`: one table per band, in order. Each band table holds `id`, the band's number, and `columns`.
- `columns`: one table per column, left to right. Each column table holds `width`, the column's width as a number, `full_width`, a boolean, and `panes`.
- `panes`: one table per pane, top to bottom. Each pane table holds `id`, the pane's number, and either `rows`, the fixed height in rows, or `weight`, the weight of an automatic height as a number. It holds `window`, the window's number, when the pane is a plugin pane of a window this client opened.

Changing the returned table SHALL NOT change the layout. Calling `gband.layout()` while the configuration loads SHALL be an error at the line of the call.

#### Scenario: Two columns
- **WHEN** the screen area is 80×24, the first band holds a column of width 1/2 with pane 1, and a full-width column of width 1/3 with pane 2 at a fixed height of 10 rows above pane 3 at weight 1, and a binding function calls `gband.layout()`
- **THEN** the result's `cols` is 80 and `rows` is 24
- **AND** its first band's columns are `{ width = 0.5, full_width = false, panes = { { id = 1, weight = 1 } } }` and a column with `width` 1/3 and `full_width` `true` whose panes are `{ id = 2, rows = 10 }` and `{ id = 3, weight = 1 }`
- **AND** its last band has no columns

#### Scenario: While loading
- **WHEN** line 4 of `user/init.lua` calls `gband.layout()`
- **THEN** loading fails with an error at `user/init.lua` line 4

### Requirement: Read the view
`gband.view()` SHALL return a new table holding:
- `band`: the viewed band's number.
- `pane`: the focused pane's number, or nil when no pane is focused.
- `window`: the focused window's number, as the plugin-windows capability defines it, or nil.
- `table`: the active key table.
- `cols` and `rows`: the size of the ribbon area.

Calling it while the configuration loads SHALL be an error at the line of the call.

#### Scenario: Focused pane
- **WHEN** the client views band 1 with pane 2 focused, its ribbon area is 80×23, and a binding in `prefix` calls `gband.view()`
- **THEN** the result is `{ band = 1, pane = 2, table = "prefix", cols = 80, rows = 23 }`

#### Scenario: Empty band
- **WHEN** the client views the empty band and a binding function calls `gband.view()`
- **THEN** the result's `pane` is nil

### Requirement: State as of the call
`gband.layout()` and `gband.view()` SHALL describe the client's state when they are called. Actions and calls that the running callback dispatched SHALL NOT be reflected, because they take effect after it returns.

#### Scenario: Action not yet applied
- **WHEN** a binding function calls `gband.action.focus_column_right()` and then `gband.view()`, with the first of two columns focused
- **THEN** the view's `pane` names the pane of the first column
- **AND** after the function returns, the second column is focused

### Requirement: Action targets
A built-in session action value SHALL accept one optional table, its target. A target SHALL take effect only when the action value is called. For `close_pane`, `consume_or_expel_left`, `consume_or_expel_right`, `cycle_column_width`, `toggle_full_width`, `grow_column_width`, `shrink_column_width`, `grow_pane_height`, `shrink_pane_height` and `reset_pane_height`, the target SHALL hold `pane`. The action SHALL then name that pane instead of the focused pane. For `open_pane`, the target SHALL hold `band`, `after`, or both:
- With both, the new column follows `after`'s column in `band`.
- With only `band`, the new column is that band's first column.
- With only `after`, the band is the band that holds `after`.

`send_prefix` SHALL accept a target holding `pane`, and send the prefix key to that pane.

A target passed to a view action or to `detach` SHALL be an error. Each of these SHALL also be an error at the line of the call:
- A target that is not a table.
- A field the action does not take.
- A pane or band number not in the client's layout.
- An `after` pane not in `band`.

Calling an action value with no target SHALL resolve it against the view, as the actions capability defines.

#### Scenario: Close a named pane
- **WHEN** panes 1 and 2 are open with pane 2 focused and a binding function calls `gband.action.close_pane({ pane = 1 })`
- **THEN** the client sends close pane naming pane 1
- **AND** pane 2 stays focused

#### Scenario: Grow a named column
- **WHEN** the only column of band 1 holds pane 1 at width 1/2, the client views band 2, and a binding function calls `gband.action.grow_column_width({ pane = 1 })`
- **THEN** pane 1's column has width 3/5

#### Scenario: Open in another band
- **WHEN** the client views band 1 and a binding function calls `gband.action.open_pane({ band = 2 })`, where band 2 is the empty last band
- **THEN** band 2 holds a new pane running the user's shell
- **AND** the client views band 2 with the new pane focused

#### Scenario: Unknown pane
- **WHEN** line 6 of `user/init.lua` holds a binding function that calls `gband.action.close_pane({ pane = 99 })` and no pane 99 exists, and the user presses its key
- **THEN** the client shows an error at `user/init.lua` line 6 naming pane 99
- **AND** nothing is sent to the server

#### Scenario: Target on a view action
- **WHEN** a binding function calls `gband.action.focus_column_left({ pane = 1 })`
- **THEN** the call raises an error and the view is unchanged

### Requirement: Focus and view by number
`gband.pane.focus(pane)` SHALL dispatch a view action that focuses the named pane, as the layout-view capability defines. `gband.band.view(band)` SHALL dispatch a view action that views the named band, as the layout-view capability defines. A number that names no pane or band in the client's layout SHALL be an error at the line of the call.

#### Scenario: Focus a pane in another band
- **WHEN** band 1 holds pane 1, band 2 holds pane 2, the client views band 1, and a binding function calls `gband.pane.focus(2)`
- **THEN** the client views band 2 with pane 2 focused

#### Scenario: View a band
- **WHEN** the client last focused pane 4 in band 2, views band 1, and a binding function calls `gband.band.view(2)`
- **THEN** the client views band 2 with pane 4 focused

### Requirement: Exact sizes
`gband.pane.set_width(pane, width)` SHALL dispatch set width naming the pane and the width, as the layout capability defines it. The width SHALL be a number greater than 0 and at most 10000, read as the configuration capability reads a width option.

`gband.pane.set_height(pane, height)` SHALL dispatch set height naming the pane. The `height` argument SHALL be a table holding exactly one of two fields:
- `rows`: an integer of at least 1, which gives a fixed height.
- `weight`: a number greater than 0 and at most 10000, read as a width is read, which gives an automatic height.

A pane number not in the client's layout, or an argument of the wrong type or out of range, SHALL be an error at the line of the call.

#### Scenario: Set a width
- **WHEN** a binding function calls `gband.pane.set_width(1, 0.4)`
- **THEN** pane 1's column has width 2/5 and full width off

#### Scenario: Set a fixed height
- **WHEN** the screen area is 80×24, a column holds panes 1 and 2 at weight 1, and a binding function calls `gband.pane.set_height(1, { rows = 8 })`
- **THEN** pane 1's tile is 8 rows high and pane 2's is 16

#### Scenario: Both fields
- **WHEN** a binding function calls `gband.pane.set_height(1, { rows = 8, weight = 1 })`
- **THEN** the call raises an error

### Requirement: Input to a named pane
`gband.pane.send_keys(pane, keys)` SHALL send each key, given as one key name or a list of key names as the configuration capability defines them, to the named pane as a key press. `gband.pane.send_text(pane, text)` SHALL send each character of `text` to the named pane as a key press of that character: a line feed or carriage return as Enter, and a tab as Tab. Any other control character SHALL be an error. `gband.pane.paste(pane, text)` SHALL send `text` to the named pane as a paste. The server SHALL encode them as it encodes keys and pastes the user sends. These functions SHALL NOT change the active key table or run any binding. An invalid key name or a pane number not in the client's layout SHALL be an error at the line of the call.

#### Scenario: Run a command in another pane
- **WHEN** panes 1 and 2 run shells, pane 2 is focused, and a binding function calls `gband.pane.send_text(1, "echo hi\n")`
- **THEN** pane 1 prints `hi`
- **AND** pane 2's screen is unchanged and pane 2 stays focused

#### Scenario: Interrupt a program
- **WHEN** pane 1 runs `sleep 100` and a binding function calls `gband.pane.send_keys(1, "ctrl+c")`
- **THEN** `sleep` stops and pane 1 shows a prompt

#### Scenario: Paste
- **WHEN** pane 1 runs a program with bracketed paste enabled and a binding function calls `gband.pane.paste(1, "a b")`
- **THEN** the program receives `a b` between the bracketed paste markers

#### Scenario: Prefix key is not a binding
- **WHEN** a binding function calls `gband.pane.send_keys(1, "ctrl+space")`
- **THEN** pane 1 receives `\x00` and the active key table does not change

### Requirement: Dispatch order
`gband.pane.focus`, `gband.pane.set_width`, `gband.pane.set_height`, `gband.pane.send_keys`, `gband.pane.send_text`, `gband.pane.paste` and `gband.band.view` SHALL be callable wherever an action value is, and an error elsewhere, as the configuration capability defines for action values. Each SHALL be dispatched like an action. It SHALL take effect after the callback returns, in the order dispatched together with the callback's actions.

#### Scenario: Order with actions
- **WHEN** a binding function calls `gband.pane.set_width(1, 1/3)` and then `gband.action.cycle_column_width({ pane = 1 })`
- **THEN** pane 1's column has width 1/2

#### Scenario: During loading
- **WHEN** line 3 of `user/init.lua` calls `gband.pane.focus(1)` at the top level
- **THEN** loading fails with an error at `user/init.lua` line 3
