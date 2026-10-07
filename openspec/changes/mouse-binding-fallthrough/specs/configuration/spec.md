## MODIFIED Requirements

### Requirement: Binding functions
A callback SHALL be a binding function, the function of a registered action, the function of a command, or an event handler. A callback SHALL run in the client, never in the server. A binding function SHALL run with no arguments when its keys are pressed, except that a binding function bound to a mouse name, and the function of a registered action bound to one, SHALL run with one argument, the mouse event's payload, as the mouse capability defines. The values that a binding function or the function of a registered action returns SHALL be ignored, with one exception: when a mouse name's binding runs the function for a press or a wheel step, a first return value of `false` SHALL decline that event, as the mouse capability's "Mouse names in key tables" defines. Calling an action value, `gband.spawn` or `gband.keymap.enter` outside a callback SHALL be a configuration error. Wherever this capability allows a call inside a binding function, the call SHALL be allowed inside any callback. An error raised while a callback runs SHALL be reported as "Reporting configuration errors" defines, and the actions the callback dispatched before the error SHALL stand.

#### Scenario: Action during evaluation
- **WHEN** line 7 of `init.lua` calls `gband.action.close_window()` at the top level
- **THEN** loading fails with an error at `init.lua` line 7

#### Scenario: Error in a binding function
- **WHEN** a binding function calls `gband.action.focus_column_left()` and then raises an error on line 9 of `init.lua`
- **THEN** the column to the left is focused
- **AND** the client shows an error at `init.lua` line 9

#### Scenario: Spawn from an event handler
- **WHEN** a `User` handler calls `gband.spawn({ cmd = "fish" })` and a binding function emits that event
- **THEN** a new window running `fish` opens

#### Scenario: Mouse binding function
- **WHEN** navigation mode binds `leftmouse` to a function that records its argument, and the user clicks terminal column 12 and row 3
- **THEN** the function runs with a table whose `button` is `"left"`, `col` is 12 and `row` is 3

#### Scenario: Key binding returns false
- **WHEN** `root` binds `alt+x` to a function that returns `false`, and the user presses Alt+X
- **THEN** the function runs once and the focused window receives nothing

#### Scenario: Mouse binding declines
- **WHEN** `root` binds `leftmouse` to a function that returns `false`, and the user clicks an unfocused window
- **THEN** the function runs once and the clicked window is focused

#### Scenario: Registered action declines
- **WHEN** `user/init.lua` registers the action `pass` with a function that records its argument and returns `false`, binds `leftmouse` in `root` to `gband.action.pass`, and the user clicks an unfocused window
- **THEN** the function runs with a table whose `button` is `"left"`
- **AND** the clicked window is focused

#### Scenario: Return value through keymap.run
- **WHEN** `root` binds `leftmouse` to a function that returns `false`, and a binding function calls `gband.keymap.run("root", "leftmouse")`
- **THEN** the function runs with no argument, `gband.keymap.run` returns `true`, and focus does not change
