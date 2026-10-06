## MODIFIED Requirements

### Requirement: Mouse names in key tables
A press of the left, middle or right button, and a wheel step up, down, left or right, SHALL be matched against the active key table as a key, under the mouse names the configuration capability defines, with the Ctrl, Alt and Shift the terminal reports. A release and a motion SHALL never match a binding. While a gesture runs, a wheel step SHALL match no binding.

- While a mode is active, a bound mouse name SHALL run its binding, and an unbound press SHALL be discarded. The mode SHALL stay active, as for a key.
- While a table that is neither `root` nor a mode is active, a bound mouse name SHALL run its binding and end the sequence, and an unbound press SHALL be discarded and end the sequence, as for a key.
- While `root` is active, a bound mouse name SHALL run its binding in place of the default, and an unbound press SHALL take the default that "Interactive mode defaults" defines.
- In any table, an unbound wheel step SHALL go to its target, as "Wheel" defines, and SHALL leave the active table unchanged.

After a wheel step runs a binding, each further wheel step of the same mouse name, with the same modifiers, that comes less than 150 ms after the step that last ran that binding SHALL be discarded: it SHALL run no binding and go to no target. A step that comes 150 ms or more after it SHALL be matched again. This keeps one turn of a free-spinning wheel or a touchpad from running the binding many times.

The release and every motion that follow a press whose binding ran SHALL go to the gesture that binding started, when it started one, and SHALL otherwise be discarded.

A binding function bound to a mouse name SHALL run with one argument: for a press, the payload of the event as the lua-events capability defines it for `MousePressed`, and for a wheel step, the payload it defines for `MouseScrolled`.

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

### Requirement: Interactive mode defaults
While `root` is active, a mouse event whose mouse name `root` does not bind SHALL take this default, by its target:

| event | window target | plugin window target | empty ribbon or outside |
|---|---|---|---|
| left, middle or right press | focus the window, then forward the press when the program reports the mouse and Ctrl and Alt are not both held | focus the plugin window | nothing |
| left press, not forwarded | start a selection at the cell | move the cursor line to the line at the cell, with `cursorline` on | — |
| right press, not forwarded | paste the copy buffer into the window | nothing | — |

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

#### Scenario: Click a plugin window line
- **WHEN** a floating plugin window with `cursorline` on shows lines 1 to 10 and the user clicks its fourth content row
- **THEN** the plugin window is focused and its cursor line is 4

### Requirement: Wheel
gband SHALL make no use of the wheel for itself except through a binding, as "Mouse names in key tables" defines. A wheel step that runs a binding, or that the cooldown discards, SHALL go to no target. Every other wheel step SHALL go to its target alone, whatever key table or mode is active, whatever modifiers are held, and while a gesture runs, and SHALL change no focus, view or camera.

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

#### Scenario: Wheel over a program without mouse reporting
- **WHEN** a shell prompt with no mouse mode is under the pointer and the user turns the wheel
- **THEN** the shell receives nothing and nothing changes
