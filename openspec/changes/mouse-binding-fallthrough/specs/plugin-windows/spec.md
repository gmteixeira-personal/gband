## MODIFIED Requirements

### Requirement: Mouse in plugin windows
`gband.win.open(opts)` SHALL also take the optional field `on_mouse`, a function, for both kinds, with no default. A value of another type SHALL be an error at the line of the call.

In this requirement, a wheel step over a plugin window means one that no binding takes and that the cooldown does not discard, as the mouse capability's "Mouse names in key tables" defines. A declined wheel step is one that no binding takes. A wheel step that a binding takes SHALL reach neither the plugin window's defaults nor `on_mouse`.

A plugin window without `on_mouse` SHALL take the mouse capability's interactive mode defaults: a left press moves the cursor line to the line under the pointer with `cursorline` on, and a wheel step up or down acts as Up or Down would, whether or not the plugin window is focused, in any key table or mode.

A plugin window with `on_mouse` SHALL take none of those defaults. The client SHALL run `on_mouse` for every press on the plugin window that no binding of `root` takes while `root` is active, a press whose `root` binding declines it included, for the release and every motion with a button held that follow such a press, and for every wheel step over the plugin window in any key table or mode, with the plugin window's number and a table holding:

- `kind`: `"press"`, `"release"`, `"drag"` or `"scroll"`.
- `button`: `"left"`, `"middle"` or `"right"`, for every kind but `"scroll"`.
- `direction`: `"up"`, `"down"`, `"left"` or `"right"`, for `"scroll"`.
- `content_col` and `content_row`: the content cell, counted from 0, and nil on the border.
- `line`: the number of the plugin window's line shown at `content_row`, counted from 1, and nil past its last line or on the border.
- `box_col` and `box_row`: the cell within the plugin window's box, border included, counted from 0 at the box's top-left cell.
- `box_width` and `box_height`: the plugin window's box as drawn in the frame the event was resolved against, border included.
- `ctrl`, `alt` and `shift`: booleans.

The four box fields SHALL be nil when the target of the cell under the pointer, as the mouse capability's "Pointer targets" defines, is not that plugin window, as for a drag or a release outside it. For a press and a wheel step, they SHALL equal the `box_col`, `box_row`, `box_width` and `box_height` of the event's `MousePressed` or `MouseScrolled` payload, as the lua-events capability defines them.

A press SHALL still focus the plugin window before `on_mouse` runs. `on_mouse` SHALL run as a callback that belongs to the plugin window's plugin.

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
