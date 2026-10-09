## MODIFIED Requirements

### Requirement: Lua names of actions
Every built-in action SHALL have one Lua name, under which `gband.action` holds it, and one description, the text of its action column:

| Lua name | action | kind |
|---|---|---|
| `focus_column_left` | focus the column to the left | view |
| `focus_column_right` | focus the column to the right | view |
| `focus_window_down` | focus the window below | view |
| `focus_window_up` | focus the window above | view |
| `focus_band_down` | view the band below | view |
| `focus_band_up` | view the band above | view |
| `center_column` | center the focused column | view |
| `switch_focus_floating_tiled` | switch focus between floating and tiled windows | view |
| `minimize_window` | minimize the focused floating window | view |
| `open_window` | open a window running the user's shell | session |
| `close_window` | close the window | session |
| `consume_or_expel_left` | consume or expel the window to the left | session |
| `consume_or_expel_right` | consume or expel the window to the right | session |
| `move_column_left` | move the column or floating window to the left | session |
| `move_column_right` | move the column or floating window to the right | session |
| `move_window_down` | move the window down | session |
| `move_window_up` | move the window up | session |
| `toggle_window_floating` | float or tile the window | session |
| `cycle_column_width` | cycle the width of the window's column | session |
| `toggle_full_width` | toggle full width of the window's column | session |
| `grow_column_width` | grow the width of the window's column | session |
| `shrink_column_width` | shrink the width of the window's column | session |
| `grow_window_height` | grow the height of the window | session |
| `shrink_window_height` | shrink the height of the window | session |
| `reset_window_height` | reset the height of the window | session |
| `detach` | detach | client |
| `send_prefix` | send the prefix key to the focused window | client |
| `reload` | reload the configuration | client |
| `toggle_multi` | toggle multi mode | client |
| `drag_window` | move the window with the mouse | client |
| `drag_resize_window` | resize the window with the mouse | client |
| `drag_band` | slide the band or switch bands with the mouse | client |

A width action named after a window's column SHALL act on the box of a floating window, as the floating-windows capability defines. `minimize_window` SHALL minimize the focused window in this client's view, as the floating-windows capability defines, and SHALL leave the view unchanged when the focused window is tiled or no window is focused. `toggle_multi` SHALL turn this client's multi mode on or off, as the multi-mode capability defines, and while multi mode is on `send_prefix` SHALL send the prefix key to every window of the viewed band instead of the focused window alone.

Actions that the configuration capability's `gband.action.register` adds SHALL sit beside the built-in actions in `gband.action` and in `gband.action.list()`, and SHALL NOT take a built-in action's name.

#### Scenario: Every action is named
- **WHEN** no action is registered and the names `gband.action.list()` returns are read
- **THEN** they are exactly the Lua names in the table

#### Scenario: Built-in descriptions
- **WHEN** `gband.action.list()` is read
- **THEN** the entry named `cycle_column_width` has the description `cycle the width of the window's column`

#### Scenario: Name and action agree
- **WHEN** a binding names `gband.action.cycle_column_width` and its keys are pressed with window 3 focused
- **THEN** the client sends cycle width naming window 3

#### Scenario: Toggle floating by name
- **WHEN** a binding names `gband.action.toggle_window_floating` and its keys are pressed with tiled window 3 focused
- **THEN** window 3 floats and stays focused

#### Scenario: Minimize by name
- **WHEN** a binding names `gband.action.minimize_window` and its keys are pressed with floating window 3 focused
- **THEN** the client does not draw window 3 and sends the server no action

#### Scenario: Minimize description
- **WHEN** `gband.action.list()` is read
- **THEN** the entry named `minimize_window` has the description `minimize the focused floating window`

#### Scenario: Reload by name
- **WHEN** a binding names `gband.action.reload` and its keys are pressed
- **THEN** the client loads its configuration again and asks the server to load its own, as the configuration capability's "Forced reload" defines

#### Scenario: Reload description
- **WHEN** `gband.action.list()` is read
- **THEN** the entry named `reload` has the description `reload the configuration`

#### Scenario: Multi mode by name
- **WHEN** a binding names `gband.action.toggle_multi` and its keys are pressed, with two windows in the viewed band, and the user then types `echo hi` and Enter
- **THEN** the client sends the server no action, and both windows print `hi`
