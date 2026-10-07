# mouse Specification

## Purpose
Defines how the client takes the mouse: what a mouse event points at, how mouse buttons and wheel steps take part in key tables, what interactive mode does with a click, a drag and a right click, how a wheel step that runs no binding reaches the program under the pointer while a bound one runs its binding, how text is selected and copied, and the niri-like gestures that move windows, resize them and slide the band.

## Requirements

### Requirement: Mouse capture
When the client takes the terminal, it SHALL enable the terminal's reporting of mouse presses, releases and motion with any button or none, with the SGR encoding: modes 1000, 1002, 1003 and 1006. Whenever the client gives the terminal back, as the client-attach capability defines for leaving the client, it SHALL first disable those modes.

#### Scenario: Reporting on attach
- **WHEN** the client attaches
- **THEN** it writes `\x1b[?1000h`, `\x1b[?1002h`, `\x1b[?1003h` and `\x1b[?1006h` to its terminal before its first frame

#### Scenario: Reporting off on detach
- **WHEN** the client detaches
- **THEN** it writes `\x1b[?1006l`, `\x1b[?1003l`, `\x1b[?1002l` and `\x1b[?1000l` before leaving the alternate screen

### Requirement: Pointer targets
Every mouse event SHALL point at the terminal cell it reports, at a column and a row counted from 0 at the terminal's top-left cell. The client SHALL resolve the cell to a target against the frame it drew last. The first of these that covers the cell SHALL be the target:

1. a floating plugin window's box, from the top of its stacking order down;
2. a floating window's box of the viewed band, from the top of this client's stacking order down, except the box of a window this client has minimized, as the floating-windows capability defines;
3. a tile of the viewed band, at the position drawn;
4. the ribbon area, as empty ribbon;
5. any other cell, as outside.

A tile whose window is a tiled plugin window's drawn window SHALL be that plugin window as the target. A target that is a window or a plugin window SHALL include its border cells. Its content cell SHALL be the cell's column and row less those of its content area's top-left cell, counted from 0. A border cell SHALL have no content cell.

#### Scenario: Click inside a tile
- **WHEN** the ribbon area starts at the terminal's top-left cell, a tile's box spans columns 40 to 79 and rows 0 to 23, and the user presses the left button at column 45 and row 3
- **THEN** the target is that tile's window, at content column 4 and content row 2

#### Scenario: Floating window over a tile
- **WHEN** a floating window's box covers column 45 and row 3 above a tile, and the user presses there
- **THEN** the target is the floating window

#### Scenario: Border cell
- **WHEN** the user presses on the top border of a tile
- **THEN** the target is that tile's window, with no content cell

#### Scenario: Empty ribbon
- **WHEN** the camera is at -20 and the user presses at column 5 of the ribbon area, left of the first column
- **THEN** the target is empty ribbon

#### Scenario: Minimized floating window over a tile
- **WHEN** this client has minimized a floating window whose box covers column 45 and row 3 above a tile, and the user presses there
- **THEN** the target is the tile's window

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

### Requirement: Selection and copy
A selection SHALL belong to one window and run from the cell where it started, its anchor, to the cell under the pointer, its head. Each motion with the left button held SHALL move the head to the cell under the pointer, kept inside the window's content area. The client SHALL draw the selected cells of its own screen with their foreground and background swapped. Other clients SHALL draw no change.

The selected cells SHALL be read in reading order from the earlier of anchor and head to the later, both included: the rest of the first row, every row between, and the last row up to the later cell. When anchor and head are on one row, only the cells between them SHALL be selected.

When the left button is released over a selection that covers more than its anchor cell, the client SHALL copy the selection's text: each row's selected characters with trailing spaces removed, the rows joined by a newline, except that a row the program wrapped into the next SHALL join it with no newline. The client SHALL set its copy buffer to that text, and set the terminal's clipboard to it as the local-actions capability's "Clipboard" defines, when the text is at most 1 MiB.

A press that does not move SHALL make no selection and copy nothing. A selection SHALL stay drawn until the next mouse press, until a key or paste is sent to any window, or until its window leaves this client's screen.

#### Scenario: Select across rows
- **WHEN** a window shows `hello world` on its first content row and `second line` on its second, and the user drags from content column 6 of the first row to content column 5 of the second, then releases
- **THEN** the copy buffer holds `world\nsecond`
- **AND** the client writes `ESC ] 52 ; c ; <base64 of world\nsecond> BEL` to its terminal

#### Scenario: Drag back before the anchor
- **WHEN** a window shows `abcdef` and the user drags from content column 4 to content column 1 on that row
- **THEN** the copy buffer holds `bcde`

#### Scenario: Click copies nothing
- **WHEN** the copy buffer holds `ls` and the user clicks a window without moving
- **THEN** the copy buffer still holds `ls` and no selection is drawn

#### Scenario: Typing clears the selection
- **WHEN** a selection is drawn and the user types `x`
- **THEN** the selection is no longer drawn

### Requirement: Forwarding to a program
A forwarded press SHALL make its window the grabbing window until every button is released. The client SHALL send that window the press, every motion while a button is held, and the release, each with its content cell. A cell outside the content area SHALL be moved to the nearest content cell. A motion with no button held SHALL be sent to the focused window when the pointer is inside its content area and its program reports the mouse.

Each forwarded event SHALL be sent as a mouse message, as the wire-protocol capability defines, and the server SHALL write it as the session-server capability defines.

#### Scenario: Drag outside the window
- **WHEN** a program with mode 1002 is clicked at content column 2 and the pointer moves to a cell left of its content area on content row 3
- **THEN** the program receives a motion at content column 0 and content row 3

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

### Requirement: Move a floating window by dragging
`drag_window` pressed on a floating window SHALL focus it. Each motion SHALL place it, as the floating-windows capability's placing defines, at its box's column and row at the press plus the pointer's movement in columns and rows since the press.

`drag_window` pressed on a floating plugin window SHALL focus it. Each motion SHALL set its `col` and `row` the same way, kept so that its box ends inside the ribbon area, as `gband.win.set_config` does.

#### Scenario: Drag a floating window
- **WHEN** the screen area is 80×24, a floating window's box is 40×12 at column 10 and row 4, and the user drags it with `drag_window` from column 20 and row 6 to column 35 and row 8
- **THEN** the client sends the placing of that window at column 25 and row 6

#### Scenario: Drag against the edge
- **WHEN** the screen area is 80×24, a floating window's box is 40×12 at column 30 and row 4, and the user drags it 20 cells right
- **THEN** the client sends the placing of that window at column 40 and row 4

#### Scenario: Drag a floating plugin window
- **WHEN** a floating plugin window's box is 20×10 at column 5 and row 3 of the ribbon area, and the user drags it 4 cells right and 2 down with `drag_window`
- **THEN** `gband.win.info(win)` reports `col` 9 and `row` 5

### Requirement: Move a tiled window by dragging
`drag_window` pressed on a tile SHALL focus its window. The first motion to another cell SHALL lift the window. While lifted, the client SHALL draw the window's tile at its position at the press moved by the pointer's movement since the press, above every tile and below floating windows, and SHALL draw the drop place as a one-cell outline in the focused border's style. The rest of the band SHALL be drawn as before the lift.

The drop place SHALL follow the pointer's cell on the viewed band's strip, ignoring floating windows and the lifted window:

- In the left quarter of a column's span, rounded down: a new column left of that column.
- In the right quarter of a column's span, rounded down: a new column right of that column.
- Elsewhere in a column's span: inside that column, above the window whose tile holds the pointer's row when the row lies in that tile's upper half, rounded down, and below it otherwise. A row under the column's last tile SHALL count as below its last window.
- Right of the last column: a new column right of the last column. Left of the first column: a new column left of the first column.

A column holding only the lifted window SHALL count as that column for the drop place, so dropping on it leaves the window where it was.

At the release, the client SHALL send moving the window to the drop place, as the layout capability's "Move a window to a place" defines, naming as the reference window the window of the place's column that this client focused most recently, or that column's first window, for a new column; and naming the window above or below which it drops, inside a column. It SHALL send nothing when moving there would leave the layout unchanged. A release without a lift SHALL only have focused the window.

#### Scenario: Drop between two columns
- **WHEN** the terminal is 80 columns wide, the camera is at 0, a band holds columns A, B and C of 40 cells with A's window focused, and the user drags A's window with `drag_window` and releases at column 75, in B's right quarter
- **THEN** the band holds B, a column with A's window, and C, in that order

#### Scenario: Drop into a column
- **WHEN** a band holds columns A and B of 40 cells, B holds one window with its tile on rows 0 to 23, and the user drags A's window and releases at column 60 and row 18
- **THEN** B holds its window above A's window, and A's column is removed

#### Scenario: Release without moving
- **WHEN** the user presses `leftmouse` in navigation mode on an unfocused tile and releases at the same cell
- **THEN** that window is focused and the layout is unchanged

### Requirement: Resize by dragging
`drag_resize_window` pressed on a window or a plugin window SHALL focus it and move one or two edges of its box. Dispatched with a target holding `edges`, as the lua-control capability's "Action targets" defines, it SHALL move exactly the edges that `edges` names, wherever the press's cell lies in the box. Dispatched with no target, it SHALL pick the edges it moves from the press's cell in the box pressed on, including its border. Let `x` and `y` be the cell's column and row in the box, and `w` and `h` the box's width and height:

- The left edge when `x` is less than `w / 3`, rounded down; the right edge when `x` is at least `w` less `w / 3`, rounded down; no vertical edge otherwise.
- The top edge when `y` is less than `h / 3`, rounded down; the bottom edge when `y` is at least `h` less `h / 3`, rounded down; no horizontal edge otherwise.
- When neither rule picks an edge, the single edge nearest the cell, preferring right, then bottom, then left, then top among equals.

Let `dx` and `dy` be the pointer's movement in columns and rows since the press. Each moved edge SHALL follow the pointer, the far edge staying put. An edge that the gesture does not move SHALL stay put. These rules SHALL apply in the same way to edges that a target names and to edges picked from the press's cell:

- For a tile, a moved left or right edge SHALL set its column's width to the column's width in cells at the press, plus `dx` for the right edge or less `dx` for the left edge, at least 3 cells, as the session's screen area's width divided into that many cells. Moving the left edge SHALL also move the camera by the change in width, so the column's right edge stays at the same terminal column. A moved top or bottom edge SHALL set the window's height to its tile height at the press, plus `dy` for the bottom edge or less `dy` for the top edge, as a number of rows, as the layout capability's "Set a window's height" defines and limits. This SHALL hold for every window of a column, the first and the last included. The column's tiles SHALL then be placed as the layout capability defines for that height.
- For a floating window, the width and `rows` SHALL change the same way, each at least 3. Moving the left or top edge SHALL also place the box so that its right or bottom edge stays put.
- For a floating plugin window, `width` and `height` SHALL change the same way, each at least 3 with a border and at least 1 without, and `col` and `row` SHALL move with the left and top edges, as `gband.win.set_config` does. Its `on_resize` SHALL run for each new size.

#### Scenario: Pick the right edge
- **WHEN** a tile's box is 40×24 and the user presses `rightmouse` in navigation mode at column 35 and row 12 of that box
- **THEN** the gesture moves the right edge only

#### Scenario: Pick a corner
- **WHEN** a floating window's box is 30×12 and the press is at column 2 and row 10 of that box
- **THEN** the gesture moves the left and bottom edges

#### Scenario: Pick the nearest edge from the middle
- **WHEN** a box is 30×12 and the press is at column 14 and row 5 of that box
- **THEN** the gesture moves the top edge only

#### Scenario: Widen a column
- **WHEN** the screen area is 80×24, a column is 40 cells wide, and the user drags its right edge 8 cells right
- **THEN** the client sends set width naming that column's window and the width 3/5

#### Scenario: Grow a window down
- **WHEN** the screen area is 80×24, a column holds P1 above P2 with tiles of 12 rows each, and the user drags P1's bottom edge 4 rows down
- **THEN** P1 has a fixed height of 16 rows and P2's tile is 8 rows high

#### Scenario: Resize a floating window from its left edge
- **WHEN** the screen area is 80×24, a floating window's box is 40 cells wide at column 20, and the user drags its left edge 10 cells left
- **THEN** its box is 50 cells wide and starts at column 10

#### Scenario: Named bottom edge pressed near a corner
- **WHEN** the screen area is 80×24, a floating window's box is 30×12 at column 10 and row 4, `drag_resize_window` runs with the target `{ edges = { "bottom" } }` for a press at column 2 and row 11 of that box, and the pointer moves 4 columns left and 3 rows down
- **THEN** the box is 30 cells wide at column 10 and row 4, and 15 rows high
- **AND** the client sends no set width and no placing of that window

#### Scenario: Named corner
- **WHEN** the screen area is 80×24, a floating window's box is 30×12 at column 10 and row 4, `drag_resize_window` runs with the target `{ edges = { "left", "top" } }` for a press at column 1 and row 0 of that box, and the pointer moves 5 columns left and 2 rows up
- **THEN** the box is 35×14 at column 5 and row 2

#### Scenario: Named edge pressed away from it
- **WHEN** the screen area is 80×24, the camera is at 0, a band holds two columns of 40 cells, and `drag_resize_window` runs with the target `{ edges = { "right" } }` for a press at column 5 and row 12 of the first column's tile, and the pointer moves 8 columns right
- **THEN** the client sends set width naming the first column's window and the width 3/5
- **AND** the camera stays at 0

#### Scenario: Named left edge of a tile moves the camera
- **WHEN** the screen area is 80×24, the camera is at 0, a band holds three columns of 40 cells, and `drag_resize_window` runs with the target `{ edges = { "left" } }` for a press at column 35 and row 12 of the second column's tile, and the pointer moves 4 columns left
- **THEN** the client sends set width naming the second column's window and the width 11/20
- **AND** the camera is at 4

#### Scenario: Named top edge of a column's first window
- **WHEN** the screen area is 80×24, a column holds P1 above P2 with tiles of 12 rows each, and `drag_resize_window` runs with the target `{ edges = { "top" } }` for a press at column 20 and row 6 of P1's box, and the pointer moves 4 rows up
- **THEN** P1 has a fixed height of 16 rows and its tile starts at row 0
- **AND** P2's tile is 8 rows high

#### Scenario: Named edge of a floating plugin window
- **WHEN** a floating plugin window with a border has a 30×12 box at column 10 and row 3 of the ribbon area, `drag_resize_window` runs with the target `{ edges = { "right" } }` for a press at column 28 and row 10 of that box, and the pointer moves 6 columns right and 2 rows down
- **THEN** `gband.win.info(win)` reports `col` 10, `row` 3, `width` 36 and `height` 12
- **AND** its `on_resize` ran last with 34 columns and 10 rows

### Requirement: Slide the band by dragging
`drag_band` SHALL slide the viewed band's camera or switch bands, never both in one gesture. `drag_window` pressed on empty ribbon SHALL act as `drag_band`. Let `dx` and `dy` be the pointer's movement in columns and rows since the press.

The gesture SHALL change nothing until a motion has `dx` of at least 2 either way or `dy` of at least 1 either way. That motion SHALL lock the gesture's axis: horizontal when the size of `dx` is at least twice the size of `dy`, and vertical otherwise, since a cell is about twice as tall as it is wide. The axis SHALL stay locked until the release, and movement along the other axis SHALL change nothing. A release before the axis locks SHALL be handled as the release of a horizontal slide.

Horizontal: each motion SHALL set the camera to its value at the press less `dx`, and the client SHALL draw it at once. Sliding SHALL send nothing to the server and change no focus until the release.

At the release of a horizontal slide, let C be the column of the viewed band whose span holds the strip position at the middle of the ribbon area, the camera plus half the ribbon area's width rounded down, or, when no column holds it, the column whose span lies nearest to it. Focus SHALL move to the tiled window this client focused most recently in C, or to C's first window, as focusing a window does. When C lies wholly inside the view, the camera SHALL stay where the slide left it, except that a strip that does not loop SHALL then be pulled back so it leaves no blank cells right of its end, as the layout-view capability's camera rule `"never"` pulls it back. Otherwise it SHALL move as that rule moves it. On a band with no column, the release SHALL return the camera to where it was at the press.

Vertical: the bands SHALL follow the pointer row for row. Let `H` be the client terminal's height, the height of each band's region as the animations capability's band switch draws it, and `v` the viewed band's index in the layout. Each motion SHALL set the drawn vertical position to `v` times `H` less `dy`, kept between 0 and the last band's index times `H`, and the client SHALL draw the bands there at once. Moving the pointer up SHALL therefore bring the band below into view, and moving it down the band above. A band other than the viewed one SHALL be drawn with the camera that viewing it would give this client. A vertical drag SHALL send nothing to the server, change no camera and change neither focus nor the viewed band until the release. When bands are added or removed above the viewed band during the drag, the drawn vertical position SHALL shift with them, so that no band jumps on screen. When the viewed band leaves the layout, the gesture SHALL end at once with no further change.

At the release of a vertical drag, let B be the band whose region holds the terminal's middle row: the band whose index is the drawn vertical position plus half of `H` rounded down, divided by `H` and rounded down. When B is the band below or above the viewed one, the client SHALL view it, as the layout-view capability's "Switch band" defines. When B is the viewed band, the view SHALL not change.

#### Scenario: Slide right and settle
- **WHEN** the terminal is 80 columns wide, a band holds four columns of 40 cells, the first is focused with the camera at 0, and the user drags with `drag_band` from column 70 to column 10, then releases
- **THEN** the camera is at 60 while dragging and stays at 60 after the release
- **AND** the third column is focused

#### Scenario: Partly shown column snaps
- **WHEN** the terminal is 80 columns wide, a band holds four columns of 60 cells, the first is focused with the camera at 0, and the user slides the band 70 cells left and releases
- **THEN** the second column is focused and the camera moves to 60

#### Scenario: Slide past the strip's end pulls back
- **WHEN** `loop_bands` is on, the terminal is 80 columns wide, a band holds two columns of 40 cells, the first is focused with the camera at 0, and the user drags with `drag_band` from column 70 to column 40, then releases
- **THEN** the camera is at 30 while dragging
- **AND** after the release the second column is focused and the camera moves to 0

#### Scenario: Middle button anywhere in navigation mode
- **WHEN** navigation mode is active with the default bindings and the user drags with the middle button over a tile
- **THEN** the band slides and no window moves

#### Scenario: Horizontal axis ignores vertical movement
- **WHEN** the terminal is 80×24, the layout holds B1 with four columns of 40 cells and B2 with a window, the client views B1 with the camera at 0, and the user drags with `drag_band` from column 70 and row 5 to column 40 and row 8, then to column 40 and row 20
- **THEN** the camera is at 30 after both motions and B2 is never drawn
- **AND** after the release B1 is still viewed

#### Scenario: Small movement locks no axis
- **WHEN** the user presses with `drag_band` at column 40 and row 10 and the pointer moves to column 41 and row 10
- **THEN** the camera and the drawn bands are unchanged

#### Scenario: Drag up to the band below
- **WHEN** the terminal is 80×24, the layout holds B1 and B2 with windows and an empty B3, the client views B1 at rest, and the user drags with `drag_band` from column 40 and row 20 to column 40 and row 4
- **THEN** rows 0 to 7 show rows 16 to 23 of B1 and rows 8 to 23 show rows 0 to 15 of B2
- **AND** no message is sent and B1 is still viewed
- **AND** after the release the client views B2 and focuses the window it last focused there

#### Scenario: Short drag returns
- **WHEN** the terminal is 80×24, the layout holds B1 and B2 with windows and an empty B3, the client views B1 at rest, and the user drags with `drag_band` from row 12 to row 6 and releases
- **THEN** B1 is still viewed with the same focus and camera

#### Scenario: Drag down to the band above
- **WHEN** the terminal is 80×24, the client views B2 of B1, B2 and an empty B3, and the user drags with `drag_band` from row 2 to row 20 and releases
- **THEN** the client views B1

#### Scenario: Vertical axis ignores sideways movement
- **WHEN** the terminal is 80×24, the client views B1 of four 40-cell columns with the camera at 0, and the user drags with `drag_band` from column 40 and row 10 to column 41 and row 5, then to column 0 and row 5
- **THEN** the camera stays at 0 and the drawn vertical position is 5 after both motions

#### Scenario: First band stops the drag
- **WHEN** the client views the first band and the user drags with `drag_band` 10 rows down and releases
- **THEN** the bands are drawn as before the press throughout, and the view is unchanged

#### Scenario: Drag from empty ribbon switches bands
- **WHEN** navigation mode is active with the default bindings, the terminal is 80×24, the client views B1 of B1, B2 and an empty B3, and the user drags with the left button on empty ribbon from row 22 to row 2 and releases
- **THEN** the client views B2 and no window moves

### Requirement: Copy buffer
Each client SHALL keep one copy buffer, empty when the client starts. It SHALL hold the text the last selection copied. A reload SHALL keep it.

#### Scenario: Buffer survives a reload
- **WHEN** the user copies `abc` by selecting it and then saves `user/init.lua`
- **THEN** a right click on a shell window pastes `abc`
