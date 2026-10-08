## MODIFIED Requirements

### Requirement: Focused plugin window
The client SHALL have at most one focused floating plugin window. The **focused plugin window** SHALL be the focused floating plugin window when there is one. Otherwise it SHALL be the plugin window whose drawn window is the focused window, when there is one, and otherwise no plugin window.

- Opening a floating plugin window with `focus` on SHALL make it the focused floating plugin window.
- `gband.win.focus(win)` on a floating plugin window SHALL make it the focused floating plugin window.
- `gband.win.focus(win)` on a tiled plugin window SHALL leave no focused floating plugin window, and dispatch focus of its drawn window as `gband.window.focus` does.
- A change of the focused window or of the viewed band SHALL leave no focused floating plugin window, except as the next two rules allow.
- After the focused floating plugin window runs one of its `keys` functions, a change of the focused window or of the viewed band SHALL leave that floating plugin window focused until the user presses another key. This holds whether the change comes from the client at once or from the server later, as the focus of an opened window does. For a floating plugin window opened with `hover = true`, as "Mouse in plugin windows" defines, the same SHALL hold after it runs its `on_mouse`, for an event of any kind. A press that focuses a window or another plugin window, as the mouse capability's "Interactive mode defaults" defines, SHALL still move the focus there.
- While a floating plugin window stays focused by the previous rule, another floating plugin window that becomes the focused floating plugin window, by opening with `focus` on or by `gband.win.focus`, while the first is still the focused floating plugin window or after it has closed, SHALL stay focused in its place through a change of the focused window or of the viewed band, until the user presses another key.
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

#### Scenario: Window opened from a held window's key
- **WHEN** a focused floating plugin window has a `keys` function for Enter that closes it with `gband.win.close`, calls `gband.action.focus_column_right()` and opens a second floating plugin window, two columns are open with the first focused, and the user presses Enter
- **THEN** the second column is focused and the second floating plugin window is the focused plugin window
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
