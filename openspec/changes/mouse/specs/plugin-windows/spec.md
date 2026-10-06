## ADDED Requirements

### Requirement: Mouse in plugin windows
`gband.win.open(opts)` SHALL also take the optional field `on_mouse`, a function, for both kinds, with no default. A value of another type SHALL be an error at the line of the call.

A plugin window without `on_mouse` SHALL take the mouse capability's interactive mode defaults: a left press moves the cursor line to the line under the pointer with `cursorline` on, and a wheel step up or down acts as Up or Down would, whether or not the plugin window is focused, in any key table or mode.

A plugin window with `on_mouse` SHALL take none of those defaults. The client SHALL run `on_mouse` for every press on the plugin window whose mouse name `root` does not bind while `root` is active, for the release and every motion with a button held that follow such a press, and for every wheel step over the plugin window in any key table or mode, with the plugin window's number and a table holding:

- `kind`: `"press"`, `"release"`, `"drag"` or `"scroll"`.
- `button`: `"left"`, `"middle"` or `"right"`, for every kind but `"scroll"`.
- `direction`: `"up"`, `"down"`, `"left"` or `"right"`, for `"scroll"`.
- `content_col` and `content_row`: the content cell, counted from 0, and nil on the border.
- `line`: the number of the plugin window's line shown at `content_row`, counted from 1, and nil past its last line or on the border.
- `ctrl`, `alt` and `shift`: booleans.

A press SHALL still focus the plugin window before `on_mouse` runs. `on_mouse` SHALL run as a callback that belongs to the plugin window's plugin.

#### Scenario: Click a line
- **WHEN** a floating plugin window with `on_mouse` shows its lines from line 5, and the user clicks its third content row
- **THEN** `on_mouse` runs with the plugin window's number and a table whose `kind` is `"press"`, `button` is `"left"` and `line` is 7
- **AND** the cursor line does not move

#### Scenario: Wheel scrolls without on_mouse
- **WHEN** an unfocused floating plugin window without `cursorline` or `on_mouse` shows lines 1 to 10 of 25, and the user turns the wheel down over it
- **THEN** it shows lines 2 to 11 and focus does not change

#### Scenario: Invalid on_mouse
- **WHEN** a binding function calls `gband.win.open({ on_mouse = 3 })`
- **THEN** the call raises an error naming `on_mouse`
