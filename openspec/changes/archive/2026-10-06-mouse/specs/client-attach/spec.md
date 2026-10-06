## ADDED Requirements

### Requirement: Default mouse bindings
With no configuration file, `root` SHALL bind no mouse name, so every mouse event in interactive mode takes the mouse capability's defaults. With the modal key style, navigation mode, the `prefix` table, SHALL also hold these bindings, after its key bindings, in this order, each keeping navigation mode active. The direct key style SHALL bind no mouse name:

| mouse name in navigation mode | Lua binding | action | kind |
|---|---|---|---|
| `leftmouse` | `prefix leftmouse` | move the window with the mouse, or slide the band from empty ribbon | client |
| `rightmouse` | `prefix rightmouse` | resize the window with the mouse | client |
| `middlemouse` | `prefix middlemouse` | slide the band with the mouse | client |
| any other mouse name | — | discard the press; navigation mode stays active | — |

#### Scenario: Drag a floating window in navigation mode
- **WHEN** the default configuration is in use, navigation mode is active, and the user drags a floating window 6 cells right with the left button
- **THEN** the floating window's box moves 6 cells right and navigation mode stays active

#### Scenario: Wheel in navigation mode reaches the program
- **WHEN** the default configuration is in use, two bands hold windows, the first is viewed, navigation mode is active, and the user turns the wheel down over `htop` with mouse reporting
- **THEN** `htop` receives the wheel step, the first band is still viewed, and navigation mode stays active

#### Scenario: Left drag on a tile in interactive mode selects
- **WHEN** the default configuration is in use, interactive mode is active, and the user drags across a shell window with the left button
- **THEN** no window moves and the dragged text is selected
