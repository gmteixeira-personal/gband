## MODIFIED Requirements

### Requirement: Key names
A key name SHALL be a key optionally preceded by modifiers, joined by `+`. The modifiers SHALL be `ctrl`, `alt` and `shift`, in any order. The key SHALL be one character, or one of `enter`, `tab`, `backtab`, `backspace`, `escape`, `esc`, `space`, `up`, `down`, `left`, `right`, `home`, `end`, `insert`, `delete`, `pageup`, `pagedown` and `f1` to `f12`, or a mouse name: `leftmouse`, `middlemouse`, `rightmouse`, `wheelup`, `wheeldown`, `wheelleft` and `wheelright`, which name a press of a mouse button or a step of the wheel, as the mouse capability defines. Modifier and key names longer than one character SHALL be read without regard to case, and a one-character key SHALL be read as written, so `D` and `d` differ. A `+` that ends the name SHALL be the key, so `alt++` is Alt with `+`. `shift` with a lowercase letter SHALL name the uppercase letter. `shift` with any other character SHALL be an error, because the shifted character is written instead. `shift` with a mouse name SHALL name that mouse name with Shift. A mouse name SHALL be valid only as the key of a binding in a key table, and SHALL be an error as the `prefix` option or as a key of a plugin window's `keys`. Any other name SHALL be an error.

#### Scenario: Modifier and character
- **WHEN** a binding names `alt+h`
- **THEN** it names `h` with Alt

#### Scenario: Shift with a letter
- **WHEN** a binding names `shift+d`
- **THEN** it names the same key as `D`

#### Scenario: Plus as the key
- **WHEN** a binding names `alt++`
- **THEN** it names `+` with Alt

#### Scenario: Named key in any case
- **WHEN** a binding names `Ctrl+PageUp`
- **THEN** it names Page Up with Ctrl

#### Scenario: Unknown key
- **WHEN** line 5 of `init.lua` binds `alt+hyper`
- **THEN** loading fails with an error at `init.lua` line 5 naming `alt+hyper`

#### Scenario: Mouse name with modifiers
- **WHEN** a binding names `Shift+WheelDown`
- **THEN** it names a step of the wheel down with Shift

#### Scenario: Mouse name as the prefix
- **WHEN** line 3 of `init.lua` sets `gband.opt.prefix = "leftmouse"`
- **THEN** loading reports a configuration error at `init.lua` line 3 naming `leftmouse`

### Requirement: Binding functions
A callback SHALL be a binding function, the function of a registered action, the function of a command, or an event handler. A callback SHALL run in the client, never in the server. A binding function SHALL run with no arguments when its keys are pressed, except that a binding function bound to a mouse name SHALL run with one argument, the mouse event's payload, as the mouse capability defines. Calling an action value, `gband.spawn` or `gband.keymap.enter` outside a callback SHALL be a configuration error. Wherever this capability allows a call inside a binding function, the call SHALL be allowed inside any callback. An error raised while a callback runs SHALL be reported as "Configuration errors" defines, and the actions the callback dispatched before the error SHALL stand.

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
