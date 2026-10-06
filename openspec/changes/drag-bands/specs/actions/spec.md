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
| `drag_window` | move the window with the mouse | client |
| `drag_resize_window` | resize the window with the mouse | client |
| `drag_band` | slide the band or switch bands with the mouse | client |

A width action named after a window's column SHALL act on the box of a floating window, as the floating-windows capability defines.

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
