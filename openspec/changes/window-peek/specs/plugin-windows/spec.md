## MODIFIED Requirements

### Requirement: Focused plugin window
The client SHALL have at most one focused floating plugin window. The **focused plugin window** SHALL be the focused floating plugin window when there is one. Otherwise it SHALL be the plugin window whose drawn window is the focused window, when there is one, and otherwise no plugin window.

- Opening a floating plugin window with `focus` on SHALL make it the focused floating plugin window.
- `gband.win.focus(win)` on a floating plugin window SHALL make it the focused floating plugin window.
- `gband.win.focus(win)` on a tiled plugin window SHALL leave no focused floating plugin window, and dispatch focus of its drawn window as `gband.window.focus` does.
- A change of the focused window or of the viewed band SHALL leave no focused floating plugin window, except as the next rule allows.
- After the focused floating plugin window runs one of its `keys` functions, a change of the focused window or of the viewed band SHALL leave that floating plugin window focused until the user presses another key. This holds whether the change comes from the client at once or from the server later, as the focus of an opened window does. For a floating plugin window opened with `hover = true`, as "Mouse in plugin windows" defines, the same SHALL hold after it runs its `on_mouse`, for an event of any kind. A press that focuses a window or another plugin window, as the mouse capability's "Interactive mode defaults" defines, SHALL still move the focus there.
- Closing the focused floating plugin window SHALL leave no focused floating plugin window.

#### Scenario: Floating plugin window takes focus
- **WHEN** window 1 is focused and a binding function opens a floating plugin window
- **THEN** the floating plugin window is the focused plugin window and `gband.view().window` is still 1

#### Scenario: Moving focus leaves the floating plugin window
- **WHEN** a floating plugin window is focused and the user presses Ctrl+Space then `l`, with two columns open and the first focused
- **THEN** the second column is focused, the floating plugin window is still drawn, and no floating plugin window is focused

#### Scenario: Action from the floating plugin window's own key
- **WHEN** a focused floating plugin window has `keys = { enter = function() gband.action.focus_column_right() end }`, two columns are open with the first focused, and the user presses Enter
- **THEN** the second column is focused and the floating plugin window is still the focused plugin window
- **AND** when the user then presses Ctrl+Space then `h`, the first column is focused and no floating plugin window is focused

#### Scenario: Peek from a hover
- **WHEN** the columns of windows 1 and 2 are open with window 1 focused, a focused floating plugin window opened with `hover = true` has an `on_mouse` that calls `gband.window.focus(2, { peek = true })` for the kind `"move"`, and the user moves the pointer over it with no button held
- **THEN** window 2 is focused and the floating plugin window is still the focused plugin window
- **AND** when the user then presses Ctrl+Space then `h`, window 1 is focused and no floating plugin window is focused

#### Scenario: Wheel step of a hover window
- **WHEN** a focused floating plugin window opened with `hover = true` has an `on_mouse` that calls `gband.action.focus_column_right()` for the kind `"scroll"`, two columns are open with the first focused, and the user turns the wheel down over it
- **THEN** the second column is focused and the floating plugin window is still the focused plugin window

#### Scenario: Click a window after a hover
- **WHEN** a focused floating plugin window opened with `hover = true` has an `on_mouse` that calls `gband.window.focus(2, { peek = true })` for the kind `"move"`, the user moves the pointer over it, and then clicks the content of tiled window 1
- **THEN** window 1 is focused and no floating plugin window is focused

### Requirement: Mouse in plugin windows
`gband.win.open(opts)` SHALL also take the optional field `on_mouse`, a function, for both kinds, with no default, and the optional field `hover`, a boolean, for both kinds, `false` by default. A value of another type for either SHALL be an error at the line of the call. `hover` SHALL be fixed when the plugin window opens: `gband.win.set_config` SHALL NOT take it, and a `config` that holds `hover` SHALL be an error naming it, as for any field that `gband.win.set_config` does not take.

In this requirement, a wheel step over a plugin window means one that no binding takes and that the cooldown does not discard, as the mouse capability's "Mouse names in key tables" defines. A declined wheel step is one that no binding takes. A wheel step that a binding takes SHALL reach neither the plugin window's defaults nor `on_mouse`. A move over a plugin window means a motion with no button held to a cell other than the cell of the previous mouse event, whose target, as the mouse capability's "Pointer targets" defines, is that plugin window.

A plugin window without `on_mouse` SHALL take the mouse capability's interactive mode defaults: a left press moves the cursor line to the line under the pointer with `cursorline` on, and a wheel step up or down acts as Up or Down would, whether or not the plugin window is focused, in any key table or mode. A move SHALL change nothing in a plugin window without `on_mouse`.

A plugin window with `on_mouse` SHALL take none of those defaults. The client SHALL run `on_mouse` for every press on the plugin window that no binding of `root` takes while `root` is active, a press whose `root` binding declines it included, for the release and every motion with a button held that follow such a press, and for every wheel step over the plugin window in any key table or mode. When the plugin window was opened with `hover = true`, the client SHALL also run `on_mouse` for every move over it, in any key table or mode, whether or not it is focused. A plugin window opened without `hover = true` SHALL receive no move. `on_mouse` SHALL run with the plugin window's number and a table holding:

- `kind`: `"press"`, `"release"`, `"drag"`, `"scroll"`, or `"move"` for a move.
- `button`: `"left"`, `"middle"` or `"right"`, for `"press"`, `"release"` and `"drag"`.
- `direction`: `"up"`, `"down"`, `"left"` or `"right"`, for `"scroll"`.
- `content_col` and `content_row`: the content cell, counted from 0, and nil on the border.
- `line`: the number of the plugin window's line shown at `content_row`, counted from 1, and nil past its last line or on the border.
- `box_col` and `box_row`: the cell within the plugin window's box, border included, counted from 0 at the box's top-left cell.
- `box_width` and `box_height`: the plugin window's box as drawn in the frame the event was resolved against, border included.
- `ctrl`, `alt` and `shift`: booleans.

The four box fields SHALL be nil when the target of the cell under the pointer, as the mouse capability's "Pointer targets" defines, is not that plugin window, as for a drag or a release outside it. For a press and a wheel step, they SHALL equal the `box_col`, `box_row`, `box_width` and `box_height` of the event's `MousePressed` or `MouseScrolled` payload, as the lua-events capability defines them. For a move, they SHALL be computed as the lua-events capability defines them for the mouse fields. A move SHALL emit no built-in event, as the lua-events capability defines for a motion with no button held.

The client SHALL report a move to the windows provider's `mouse`, with the kind `"move"`, only for a plugin window whose latest frame, as `gband.core.present_window` sets it, holds `hover = true`. The bundled `gband.win` SHALL set `hover` in the frames of a plugin window opened with `hover = true`, and only of such a window. A provider whose frames never hold `hover = true` SHALL therefore never receive the kind `"move"`.

A press SHALL still focus the plugin window before `on_mouse` runs. A move SHALL NOT focus it. `on_mouse` SHALL run as a callback that belongs to the plugin window's plugin.

#### Scenario: Click a line
- **WHEN** a floating plugin window with `on_mouse` shows its lines from line 5, and the user clicks its third content row
- **THEN** `on_mouse` runs with the plugin window's number and a table whose `kind` is `"press"`, `button` is `"left"` and `line` is 7
- **AND** the cursor line does not move

#### Scenario: Wheel scrolls without on_mouse
- **WHEN** an unfocused floating plugin window without `cursorline` or `on_mouse` shows lines 1 to 10 of 25, and the user turns the wheel down over it
- **THEN** it shows lines 2 to 11 and focus does not change

#### Scenario: Bound wheel step over a plugin window
- **WHEN** the default configuration is in use, two bands hold windows, the first is viewed, and the user turns the wheel down with Alt held over a floating plugin window with `on_mouse`
- **THEN** the second band is viewed and `on_mouse` does not run

#### Scenario: Declined press reaches on_mouse
- **WHEN** `root` binds `leftmouse` to a function that returns `false`, and the user clicks the third content row of an unfocused floating plugin window with `on_mouse`
- **THEN** the plugin window is focused and `on_mouse` runs once with a table whose `kind` is `"press"` and `button` is `"left"`
- **AND** `on_mouse` runs once more with `kind` `"release"` when the button is released

#### Scenario: Declined wheel step scrolls a plugin window
- **WHEN** `root` binds `wheeldown` to a function that returns `false`, and the user turns the wheel down over an unfocused floating plugin window without `cursorline` or `on_mouse` that shows lines 1 to 10 of 25
- **THEN** it shows lines 2 to 11 and focus does not change

#### Scenario: Box cell on the border
- **WHEN** a floating plugin window with a border and `on_mouse` has a box of 20×10, and the user presses the left button on the top-right cell of its box
- **THEN** `on_mouse` runs with a table whose `content_col` is nil, `box_col` is 19, `box_row` is 0, `box_width` is 20 and `box_height` is 10

#### Scenario: Drag off the plugin window
- **WHEN** a floating plugin window with `on_mouse` has its box at columns 5 to 24 of the ribbon area, and the user presses inside it and drags to column 40, outside every plugin window
- **THEN** `on_mouse` runs for the drag with a table whose `box_col`, `box_row`, `box_width` and `box_height` are nil

#### Scenario: Invalid on_mouse
- **WHEN** a binding function calls `gband.win.open({ on_mouse = 3 })`
- **THEN** the call raises an error naming `on_mouse`

#### Scenario: Hover a line
- **WHEN** a floating plugin window with a border, `hover = true` and `on_mouse` has its box of 20×10 at column 5 and row 3 of a ribbon area that starts at the terminal's top-left cell, shows its lines from line 1, and the user moves the pointer with no button held from column 40 and row 12 to column 7 and row 6, then to column 8 and row 6
- **THEN** `on_mouse` runs twice, first with a table whose `kind` is `"move"`, `button` is nil, `line` is 3, `content_col` is 1, `box_col` is 2 and `box_row` is 3, then with `content_col` 2 and `line` 3
- **AND** focus does not change and no `MouseDragged` handler runs

#### Scenario: No move without hover
- **WHEN** a floating plugin window with `on_mouse` and without `hover` has its box at columns 5 to 24 of the ribbon area, and the user moves the pointer with no button held across it
- **THEN** `on_mouse` does not run

#### Scenario: Hover in a mode
- **WHEN** navigation mode is active and the user moves the pointer with no button held over a floating plugin window with `hover = true` and `on_mouse`
- **THEN** `on_mouse` runs with `kind` `"move"`, and navigation mode stays active

#### Scenario: Hover off the plugin window
- **WHEN** a floating plugin window with `hover = true` and `on_mouse` has its box at columns 5 to 24 of the ribbon area, and the user moves the pointer with no button held from column 30 to column 40 of the same row
- **THEN** `on_mouse` does not run

#### Scenario: Hover without on_mouse
- **WHEN** a floating plugin window with `cursorline`, `hover = true` and no `on_mouse` has its cursor on line 1, and the user moves the pointer with no button held over its third content row
- **THEN** its cursor stays on line 1

#### Scenario: Invalid hover
- **WHEN** a binding function calls `gband.win.open({ hover = "yes" })`
- **THEN** the call raises an error naming `hover`

#### Scenario: Hover is fixed at open
- **WHEN** a floating plugin window is open and a binding function calls `gband.win.set_config(win, { hover = true })`
- **THEN** the call raises an error naming `hover`, and the plugin window is unchanged

### Requirement: Plugin window information
`gband.win.list()` SHALL return the numbers of the open plugin windows in ascending order. `gband.win.info(win)` SHALL return a new table holding:
- `id`, `kind` and `focused`.
- `window`: the drawn window's number, or nil.
- `top`, `cursor` and `line_count`.
- `cols` and `rows`: the content area's size, or nil while a tiled plugin window's size is unknown.
- `hover`: `true` when the plugin window was opened with `hover = true`, as "Mouse in plugin windows" defines, and `false` otherwise.
- For a floating plugin window, `row`, `col`, `width` and `height`: its box as last placed.

#### Scenario: Info of a floating plugin window
- **WHEN** the ribbon area is 80×23 and a binding function opens a floating plugin window with width 40, height 11 and three lines, then calls `gband.win.info` on it
- **THEN** the result holds `kind = "floating"`, `focused = true`, `top = 1`, `cursor = 1`, `line_count = 3`, `cols = 38`, `rows = 9`, `hover = false`, `row = 6`, `col = 20`, `width = 40` and `height = 11`

#### Scenario: Info of a hover window
- **WHEN** a binding function opens a tiled plugin window with `hover = true`, then calls `gband.win.info` on it
- **THEN** the result holds `kind = "tiled"` and `hover = true`
