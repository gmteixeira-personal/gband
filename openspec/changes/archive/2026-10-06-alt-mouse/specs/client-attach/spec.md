## MODIFIED Requirements

### Requirement: Default mouse bindings
With no configuration file, both key styles SHALL make the same mouse bindings, in addition to the key bindings that "Key bindings" defines for each style. "No binding in `root`" there SHALL apply to keys only. Every mouse event that no binding takes in interactive mode SHALL take the mouse capability's defaults.

Both key styles SHALL bind these mouse names in `root`, in this order, each to the action its row names:

| mouse name | Lua binding | action | kind |
|---|---|---|---|
| `mod+leftmouse` | `mod+leftmouse` | move the window with the mouse, or slide the band or switch bands from empty ribbon | client |
| `mod+rightmouse` | `mod+rightmouse` | resize the window with the mouse | client |
| `mod+middlemouse` | `mod+middlemouse` | slide the band or switch bands with the mouse | client |
| `mod+wheeldown` | `mod+wheeldown` | view the band below | view |
| `mod+wheelup` | `mod+wheelup` | view the band above | view |

`mod` stands for the modifiers that the `mouse_mod` option names, Alt by default, as the configuration capability defines.

Both key styles SHALL also bind these mouse names in `prefix`, after its key bindings, in this order, each to the action its row names:

| mouse name after the prefix | Lua binding | action | kind |
|---|---|---|---|
| `leftmouse` | `prefix leftmouse` | move the window with the mouse, or slide the band or switch bands from empty ribbon | client |
| `rightmouse` | `prefix rightmouse` | resize the window with the mouse | client |
| `middlemouse` | `prefix middlemouse` | slide the band or switch bands with the mouse | client |
| `mod+leftmouse` | `prefix mod+leftmouse` | move the window with the mouse, or slide the band or switch bands from empty ribbon | client |
| `mod+rightmouse` | `prefix mod+rightmouse` | resize the window with the mouse | client |
| `mod+middlemouse` | `prefix mod+middlemouse` | slide the band or switch bands with the mouse | client |
| `mod+wheeldown` | `prefix mod+wheeldown` | view the band below | view |
| `mod+wheelup` | `prefix mod+wheelup` | view the band above | view |
| any other press | — | discard the press | — |

With the modal key style, each of these keeps navigation mode active, and a discarded press leaves it active. With the direct key style, each ends the key sequence, as any key after the prefix key does, and so does a discarded press. An unbound wheel step goes to its target in either style and leaves the active table unchanged, as the mouse capability defines.

#### Scenario: Drag a floating window in navigation mode
- **WHEN** the default configuration is in use, navigation mode is active, and the user drags a floating window 6 cells right with the left button
- **THEN** the floating window's box moves 6 cells right and navigation mode stays active

#### Scenario: Wheel in navigation mode reaches the program
- **WHEN** the default configuration is in use, two bands hold windows, the first is viewed, navigation mode is active, and the user turns the wheel down over `htop` with mouse reporting
- **THEN** `htop` receives the wheel step, the first band is still viewed, and navigation mode stays active

#### Scenario: Left drag on a tile in interactive mode selects
- **WHEN** the default configuration is in use, interactive mode is active, and the user drags across a shell window with the left button
- **THEN** no window moves and the dragged text is selected

#### Scenario: Middle drag up switches bands in navigation mode
- **WHEN** the default configuration is in use, two bands hold windows, the first is viewed, navigation mode is active, and the user drags with the middle button from the bottom row to the top row and releases
- **THEN** the second band is viewed and navigation mode stays active

#### Scenario: Alt drag moves a window in interactive mode
- **WHEN** the default configuration is in use with either key style, interactive mode is active, and the user drags a floating window 6 cells right with the left button and Alt held
- **THEN** the floating window's box moves 6 cells right and interactive mode stays active

#### Scenario: Alt drag in navigation mode
- **WHEN** the default configuration is in use with the modal key style, navigation mode is active, and the user drags a floating window 6 cells right with the left button and Alt held
- **THEN** the floating window's box moves 6 cells right and navigation mode stays active

#### Scenario: Alt wheel switches bands
- **WHEN** the default configuration is in use with either key style, interactive mode is active, two bands hold windows, the first is viewed, and the user turns the wheel down with Alt held over `htop` with mouse reporting
- **THEN** the second band is viewed and `htop` receives nothing

#### Scenario: Drag after the prefix with the direct key style
- **WHEN** the direct key style is saved, no `user/init.lua` exists, and the user presses Ctrl+Space and then drags a floating window 6 cells right with the left button
- **THEN** the floating window's box moves 6 cells right
- **AND** `root` is active after the press, so the keys that follow reach the focused window

#### Scenario: Other press after the prefix with the direct key style
- **WHEN** the direct key style is saved, no `user/init.lua` exists, and the user presses Ctrl+Space, clicks with the left button and Ctrl held on an unfocused window, then types `x`
- **THEN** focus does not change, and the focused window receives `x`
