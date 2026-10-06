## MODIFIED Requirements

### Requirement: Key names
A key name SHALL be a key optionally preceded by modifiers, joined by `+`. The modifiers SHALL be `ctrl`, `alt` and `shift`, in any order, and, before a mouse name only, `mod`, as "Mouse modifier" defines. The key SHALL be one character, or one of `enter`, `tab`, `backtab`, `backspace`, `escape`, `esc`, `space`, `up`, `down`, `left`, `right`, `home`, `end`, `insert`, `delete`, `pageup`, `pagedown` and `f1` to `f12`, or a mouse name. The mouse names SHALL be `leftmouse`, `middlemouse` and `rightmouse`, which name a press of that mouse button, and `wheelup`, `wheeldown`, `wheelleft` and `wheelright`, which name one wheel step in that direction, as the mouse capability defines. Modifier and key names longer than one character SHALL be read without regard to case, and a one-character key SHALL be read as written, so `D` and `d` differ. A `+` that ends the name SHALL be the key, so `alt++` is Alt with `+`. `shift` with a lowercase letter SHALL name the uppercase letter. `shift` with any other character SHALL be an error, because the shifted character is written instead. `shift` with a mouse name SHALL name that mouse name with Shift. `mod` with a key that is not a mouse name SHALL be an error. A mouse name SHALL be valid only as the key of a binding in a key table, and SHALL be an error as the `prefix` option or as a key of a plugin window's `keys`. Any other name SHALL be an error.

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
- **WHEN** a binding names `Shift+RightMouse`
- **THEN** it names a press of the right button with Shift

#### Scenario: Wheel name
- **WHEN** a binding names `Alt+WheelDown`
- **THEN** it names one wheel step down with Alt

#### Scenario: Unknown wheel name
- **WHEN** line 4 of `init.lua` binds `prefix scrollup`
- **THEN** loading fails with an error at `init.lua` line 4 naming `scrollup`

#### Scenario: Mod with a key
- **WHEN** line 6 of `init.lua` binds `mod+h`
- **THEN** loading fails with an error at `init.lua` line 6 naming `mod+h`

#### Scenario: Mouse name as the prefix
- **WHEN** line 3 of `init.lua` sets `gband.opt.prefix = "leftmouse"`
- **THEN** loading reports a configuration error at `init.lua` line 3 naming `leftmouse`

## ADDED Requirements

### Requirement: Mouse modifier
The client SHALL have the built-in option `mouse_mod`, with the description `modifiers that mod stands for in a mouse name`. Its value SHALL be one or more of the modifiers `ctrl`, `alt` and `shift`, joined by `+`, in any order and case, and its default SHALL be `"alt"`. Any other value SHALL be rejected as "Options and their values" defines for a value of the wrong type. Reading `gband.opt.mouse_mod` SHALL return the modifiers as set, in lowercase, in the order `ctrl`, `alt`, `shift`, joined by `+`.

`mod` in the key name of a binding SHALL stand for the modifiers that `mouse_mod` names when the bindings are used, added to any other modifier the name gives, as `prefix` stands for the key the `prefix` option names. Setting `mouse_mod` before or after the binding is made, in the same load, SHALL give the same binding. `gband.keymap.list` SHALL give the key of such a binding as written, such as `mod+leftmouse`.

When more than one binding of one table matches the same mouse event, the binding made first SHALL run.

#### Scenario: Default mouse modifier
- **WHEN** `user/init.lua` binds `mod+leftmouse` in `root` to `gband.action.drag_window` and does not set `mouse_mod`, and the user drags a floating window 5 cells right with the left button and Alt held
- **THEN** the floating window's box moves 5 cells right

#### Scenario: Changed mouse modifier
- **WHEN** `user/init.lua` calls `gband.keystyle.use()` and then sets `gband.opt.mouse_mod = "ctrl+alt"`, and the user drags a floating window with the left button and Ctrl and Alt held
- **THEN** the floating window's box moves with the pointer
- **AND** a left drag with Alt alone over a program that reports the mouse reaches the program

#### Scenario: Read back
- **WHEN** `user/init.lua` sets `gband.opt.mouse_mod = "Alt+Ctrl"` and reads `gband.opt.mouse_mod`
- **THEN** it reads `ctrl+alt`

#### Scenario: Invalid mouse modifier
- **WHEN** line 2 of `user/init.lua` sets `gband.opt.mouse_mod = "super"`
- **THEN** a configuration error at `user/init.lua` line 2 naming `mouse_mod` is reported
- **AND** `gband.opt.mouse_mod` reads `alt`

#### Scenario: Listed as written
- **WHEN** `user/init.lua` binds `mod+rightmouse` in `root` and reads `gband.keymap.list("root")`
- **THEN** the entry's `key` is `mod+rightmouse`
