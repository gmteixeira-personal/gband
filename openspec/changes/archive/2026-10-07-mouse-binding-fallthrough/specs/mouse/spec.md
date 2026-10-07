## MODIFIED Requirements

### Requirement: Mouse names in key tables
A press of the left, middle or right button, and a wheel step up, down, left or right, SHALL be matched against the active key table as a key, under the mouse names the configuration capability defines, with the Ctrl, Alt and Shift the terminal reports. A release and a motion SHALL never match a binding. While a gesture runs, a wheel step SHALL match no binding.

The binding that a press or a wheel step matches SHALL either take the event or decline it. A binding to a function, whether a binding function or the function of a registered action, SHALL decline the event when the first value the function returns is the boolean `false`. Every other binding SHALL take the event. This includes a binding to a built-in action, a function that returns any other value, nil or no value included, a function that raises an error, and a function that does not run because its plugin is marked failed, as the plugins capability defines. A press or a wheel step that no binding takes is one that the active table does not bind, or one whose binding declines it.

- While a mode is active, a press that no binding takes SHALL be discarded. The mode SHALL stay active, as for a key.
- While a table that is neither `root` nor a mode is active, every press SHALL end the sequence, as for a key, and a press that no binding takes SHALL be discarded. A wheel step that a binding takes SHALL also end the sequence.
- While `root` is active, a binding that takes a press SHALL run in place of the default, and a press that no binding takes SHALL take the default that "Interactive mode defaults" defines.
- In any table, a wheel step that no binding takes SHALL go to its target, as "Wheel" defines, and SHALL leave the active table unchanged.

A declined event SHALL be handled exactly as it would be if the active table bound no mouse name that matches it. No other binding of the table SHALL run for it, even one made earlier that matches the same event. The fallback SHALL be that of the table that was active when the event arrived. The actions, spawns and table changes that the declining function dispatched SHALL stand, as for any callback, and SHALL take effect before the fallback. A table that the declining function enters with `gband.keymap.enter` SHALL therefore be active after the event, also when the event is a wheel step.

After a wheel step that a binding takes, each further wheel step of the same mouse name, with the same modifiers, that comes less than 150 ms after the step that a binding last took SHALL be discarded: it SHALL run no binding and go to no target. A step that comes 150 ms or more after it SHALL be matched again. A declined wheel step SHALL neither start nor extend this cooldown. This keeps one turn of a free-spinning wheel or a touchpad from running the binding many times.

The release and every motion that follow a press that a binding took SHALL go to the gesture that binding started, when it started one, and SHALL otherwise be discarded. The release and every motion that follow a declined press SHALL be handled as those that follow a press that the active table does not bind.

A binding function bound to a mouse name, and the function of a registered action bound to one, SHALL run with one argument: for a press, the payload of the event as the lua-events capability defines it for `MousePressed`, and for a wheel step, the payload it defines for `MouseScrolled`.

#### Scenario: Bound in root
- **WHEN** `root` binds `leftmouse` to a function and the user clicks a window that is not focused
- **THEN** the function runs with a table whose `button` is `"left"` and whose `window` names the clicked window
- **AND** focus does not change and nothing is sent to the window

#### Scenario: Unbound in a mode
- **WHEN** navigation mode is active, its table does not bind `ctrl+leftmouse`, and the user presses the left button with Ctrl held
- **THEN** nothing happens and navigation mode stays active

#### Scenario: Modifier with a mouse name
- **WHEN** a mode binds `alt+rightmouse` and the user presses the right button with Alt held
- **THEN** the binding runs

#### Scenario: Bound wheel step in root
- **WHEN** `root` binds `alt+wheeldown` to `focus_band_down`, the layout holds bands B1 and B2 with windows, the client views B1, and the user turns the wheel one step down with Alt held over a window whose program reports the mouse
- **THEN** the client views B2
- **AND** the program receives nothing

#### Scenario: Cooldown after a wheel binding
- **WHEN** `root` binds `alt+wheeldown` to `focus_band_down`, the layout holds bands B1, B2 and B3 with windows, the client views B1, and the user turns the wheel down with Alt held three steps within 100 ms
- **THEN** the client views B2, and no window receives a wheel step

#### Scenario: Binding runs again after the cooldown
- **WHEN** `root` binds `alt+wheeldown` to `focus_band_down`, the layout holds bands B1, B2 and B3 with windows, the client views B1, and the user turns the wheel down with Alt held, then again 200 ms later
- **THEN** the client views B3

#### Scenario: Unbound wheel step keeps the sequence
- **WHEN** the direct key style is in use, the second of two columns is focused, and the user presses Ctrl+Space, turns the wheel down over `htop` with mouse reporting, then presses `h`
- **THEN** `htop` receives the wheel step and the first column is focused

#### Scenario: Wheel during a gesture
- **WHEN** `root` binds `alt+leftmouse` to `drag_window` and `alt+wheeldown` to `focus_band_down`, two bands hold windows, and the user drags a floating window with the left button and Alt held and turns the wheel down with Alt held before the release
- **THEN** the viewed band does not change

#### Scenario: Wheel binding function
- **WHEN** `root` binds `ctrl+wheelup` to a function that records its argument, and the user turns the wheel up with Ctrl held over empty ribbon
- **THEN** the function runs once with a table whose `direction` is `"up"` and whose `ctrl` is true

#### Scenario: Declined click reaches the program
- **WHEN** `root` binds `leftmouse` to a function that returns `false`, a window's program enabled mode 1000 with SGR encoding, and the user clicks its content cell at column 4 and row 2 with the left button
- **THEN** the function runs once
- **AND** the window is focused and its program receives `\x1b[<0;5;3M` and then, on release, `\x1b[<0;5;3m`

#### Scenario: Border taken, content declined
- **WHEN** the copy buffer holds `ls`, `root` binds `rightmouse` to a function that returns `false` when its argument's `content_col` is not nil and otherwise records a call, and the user right-clicks the top border of an unfocused shell window whose program reports no mouse, then right-clicks its content
- **THEN** the first press records one call, and the window is not focused and receives nothing
- **AND** the second press focuses the window and pastes `ls` into it

#### Scenario: Actions of a declining function stand
- **WHEN** `prefix` is a mode, `root` binds `leftmouse` to a function that calls `gband.keymap.enter("prefix")` and returns `false`, and the user clicks the content of an unfocused window whose program reports no mouse
- **THEN** the window is focused and a selection starts at the clicked cell
- **AND** `prefix` is the active table

#### Scenario: Declined in a mode
- **WHEN** navigation mode is active, its table binds `leftmouse` to a function that returns `false`, and the user clicks an unfocused window
- **THEN** focus does not change, nothing is sent to the window, and navigation mode stays active

#### Scenario: Declined press ends the sequence
- **WHEN** `prefix` is not a mode and binds `leftmouse` to a function that returns `false`, and the user presses Ctrl+Space, clicks an unfocused window, then types `h`
- **THEN** focus does not change and the focused window receives `h`

#### Scenario: Declined wheel step keeps the sequence
- **WHEN** the direct key style is in use, `user/init.lua` also binds `prefix wheeldown` to a function that returns `false`, the second of two columns is focused, and the user presses Ctrl+Space, turns the wheel down over `htop` with mouse reporting, then presses `h`
- **THEN** `htop` receives the wheel step and the first column is focused

#### Scenario: Declined wheel step starts no cooldown
- **WHEN** `root` binds `alt+wheeldown` to a function that returns `false` on its first call and calls `gband.action.focus_band_down()` on later calls, the layout holds bands B1, B2 and B3 with windows, the client views B1, and the user turns the wheel down with Alt held twice within 100 ms
- **THEN** the function runs twice and the client views B2

#### Scenario: Declining skips an earlier binding
- **WHEN** `mouse_mod` is `"alt"`, `root` binds `alt+leftmouse` to `drag_window` and then `mod+leftmouse` to a function that returns `false`, and the user presses the left button with Alt held on the content of an unfocused window whose program reports no mouse, then drags 5 cells right
- **THEN** the window is focused, no window moves, and the dragged text is selected

#### Scenario: Error takes the press
- **WHEN** `root` binds `leftmouse` to a function that raises an error, and the user clicks an unfocused window
- **THEN** the client reports the error, focus does not change and nothing is sent to the window

### Requirement: Interactive mode defaults
While `root` is active, a press that no binding of `root` takes, as "Mouse names in key tables" defines, SHALL take this default, by its target:

| event | window target | plugin window target | empty ribbon or outside |
|---|---|---|---|
| left, middle or right press | focus the window, then forward the press when the program reports the mouse and Ctrl and Alt are not both held | focus the plugin window | nothing |
| left press, not forwarded | start a selection at the cell | move the cursor line to the line at the cell, with `cursorline` on | — |
| right press, not forwarded | paste the copy buffer into the window | nothing | — |

A press whose `root` binding declines it SHALL take this default exactly as a press that `root` does not bind.

Focusing a window SHALL dispatch focus of that window, as `gband.window.focus` does, which raises a floating window and switches the layer. Focusing a floating plugin window SHALL make it the focused floating plugin window. Focusing a tiled plugin window SHALL focus its drawn window.

A program SHALL report the mouse when its grid's mouse tracking mode, as the terminal-emulator capability defines, is any mode other than none. A press on a border cell SHALL focus and SHALL NOT be forwarded, start a selection or paste.

Pasting the copy buffer SHALL send it as a paste naming the window, as a terminal paste is sent. An empty copy buffer SHALL paste nothing.

#### Scenario: Click to focus
- **WHEN** two tiled windows are open with the second focused and the user clicks inside the first
- **THEN** the first window is focused, and typing `echo left` and Enter prints `left` in the first tile only

#### Scenario: Click a floating window
- **WHEN** the tiled layer is active and the user clicks a floating window
- **THEN** the floating layer is active and the clicked window is focused and drawn on top

#### Scenario: Forwarded click
- **WHEN** a window's program enabled mode 1000 with SGR encoding, and the user clicks its content cell at column 4 and row 2 with the left button
- **THEN** the window is focused and its program receives `\x1b[<0;5;3M` and then, on release, `\x1b[<0;5;3m`

#### Scenario: Alt selects in a mouse program
- **WHEN** a window's program reports the mouse and the user drags across it with the left button and Ctrl and Alt held
- **THEN** the program receives nothing and the dragged text is selected

#### Scenario: Unbound Alt click is forwarded
- **WHEN** `root` binds no `alt+leftmouse`, a window's program enabled mode 1000 with SGR encoding, and the user clicks its content cell at column 4 and row 2 with the left button and Alt held
- **THEN** the program receives `\x1b[<8;5;3M`

#### Scenario: Right click pastes
- **WHEN** the copy buffer holds `ls` and the user right-clicks a shell window whose program reports no mouse
- **THEN** the window is focused and receives `ls` as a paste

#### Scenario: Declined right click pastes
- **WHEN** the copy buffer holds `ls`, `root` binds `rightmouse` to a function that returns `false`, and the user right-clicks an unfocused shell window whose program reports no mouse
- **THEN** the window is focused and receives `ls` as a paste

#### Scenario: Click a plugin window line
- **WHEN** a floating plugin window with `cursorline` on shows lines 1 to 10 and the user clicks its fourth content row
- **THEN** the plugin window is focused and its cursor line is 4

### Requirement: Drag gestures
The client actions `drag_window`, `drag_resize_window` and `drag_band` SHALL start a gesture when they are dispatched while the client handles a mouse press, from that press's binding or from a binding function it runs, and that binding takes the press. Dispatched by a function that declines the press, or at any other time, they SHALL do nothing. A declined press therefore starts no gesture, and its release and motions take the fallback that "Mouse names in key tables" defines.

A gesture SHALL hold the press's button, target, cell, and the geometry drawn at the press. It SHALL take every motion until that button is released, and end at the release. While it runs, other presses SHALL be discarded, wheel steps SHALL be handled as "Wheel" defines, and keys SHALL be handled as usual. A gesture whose window leaves the layout SHALL end at once with no further change.

The changes a gesture sends to the server SHALL be sent at most once per frame, and only when the size or position they set differs from the last one sent.

#### Scenario: Drag action from a key
- **WHEN** navigation mode binds `m` to `drag_window` and the user presses `m`
- **THEN** nothing changes

#### Scenario: Drag action from a binding function
- **WHEN** navigation mode binds `leftmouse` to a function that calls `gband.action.drag_window()`, and the user drags a floating window 5 cells right
- **THEN** the floating window's box moves 5 cells right

#### Scenario: Declining function starts no gesture
- **WHEN** navigation mode binds `leftmouse` to a function that calls `gband.action.drag_window()` and returns `false`, and the user drags a floating window 5 cells right
- **THEN** the floating window's box does not move and nothing is sent to the server
- **AND** navigation mode stays active

#### Scenario: Declined press in root reaches the program
- **WHEN** `root` binds `leftmouse` to a function that calls `gband.action.drag_window()` and returns `false`, a floating window's program enabled mode 1002 with SGR encoding, and the user drags across its content from content column 2 to content column 6 on content row 1
- **THEN** the box does not move
- **AND** the program receives the press, a motion at content column 6, and the release

### Requirement: Wheel
gband SHALL make no use of the wheel for itself except through a binding, as "Mouse names in key tables" defines. A wheel step that a binding takes, or that the cooldown discards, SHALL go to no target. Every other wheel step, a declined one included, SHALL go to its target alone, whatever key table or mode is active, whatever modifiers are held, and while a gesture runs, and SHALL change no focus, view or camera.

- Over a window, the client SHALL send the step to that window as a mouse message, as the wire-protocol capability defines, with the content cell under the pointer, or the nearest content cell when the pointer is on the border. The server SHALL write it as the session-server capability defines, so the program receives it whenever its mouse tracking mode reports the wheel.
- Over a plugin window, the step SHALL go to the plugin window, as the plugin-windows capability's "Mouse in plugin windows" defines.
- Over empty ribbon or outside, the step SHALL do nothing.

#### Scenario: Wheel over an unfocused mouse program
- **WHEN** `htop` with mouse reporting is shown in an unfocused tile and the user turns the wheel down over it
- **THEN** `htop` receives the wheel step and focus does not change

#### Scenario: Wheel in navigation mode
- **WHEN** navigation mode is active and the user turns the wheel up over content column 2 and row 1 of a window whose program enabled modes 1000 and 1006
- **THEN** the program receives `\x1b[<64;3;2M`
- **AND** navigation mode stays active and the view is unchanged

#### Scenario: Wheel with Alt
- **WHEN** the active table binds no `alt+wheeldown`, a window's program enabled modes 1000 and 1006, and the user turns the wheel down with Alt held over its content cell at column 0 and row 0
- **THEN** the program receives `\x1b[<73;1;1M`

#### Scenario: Wheel with Ctrl
- **WHEN** the active table binds no `ctrl+wheeldown`, a window's program enabled modes 1000 and 1006, and the user turns the wheel down with Ctrl held over its content cell at column 0 and row 0
- **THEN** the program receives `\x1b[<81;1;1M`

#### Scenario: Declined wheel step reaches the program
- **WHEN** `root` binds `alt+wheeldown` to a function that returns `false`, the layout holds bands B1 and B2 with windows, the client views B1, a window's program enabled modes 1000 and 1006, and the user turns the wheel down with Alt held over its content cell at column 0 and row 0
- **THEN** the program receives `\x1b[<73;1;1M`
- **AND** the client still views B1

#### Scenario: Wheel over a program without mouse reporting
- **WHEN** a shell prompt with no mouse mode is under the pointer and the user turns the wheel
- **THEN** the shell receives nothing and nothing changes
