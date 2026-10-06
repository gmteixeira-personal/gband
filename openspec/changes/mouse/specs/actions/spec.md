## MODIFIED Requirements

### Requirement: Kinds of action
Every action SHALL be exactly one of three kinds:

| kind | runs in | examples |
|---|---|---|
| view | the client alone, with no message to the server | focus a neighbouring window, view another band |
| session | the server, which applies it to the shared layout | open, close, consume or expel a window, change a column's width |
| client | the client alone, outside the view | detach, send the prefix key to the focused window |

The drag actions, `drag_window`, `drag_resize_window` and `drag_band`, are client actions that act only while the client handles a mouse press, and then send session actions and move the camera as the mouse capability defines. One view action sends a message: `center_column`, while the floating layer is active, sends the placing of the focused floating window, as the layout-view capability's "Center the focused column" defines.

A key binding SHALL name exactly one action or one Lua function. Pressing a key bound to an action SHALL have the same effect as dispatching that action from any other source.

#### Scenario: Every binding names an action
- **WHEN** the bindings of the default configuration are listed
- **THEN** every entry names one view, session or client action

#### Scenario: Center column on the floating layer
- **WHEN** a client focuses a floating window whose box is not centred and dispatches `center_column`
- **THEN** the client sends the placing of that window and changes no camera

#### Scenario: Action from another source
- **WHEN** a test dispatches the close-window session action to a client whose focused window is the second of two
- **THEN** the second window closes, as it does when the user presses Ctrl+Space then `q`

#### Scenario: Drag action sends session actions
- **WHEN** navigation mode binds `rightmouse` to `drag_resize_window` and the user drags a column's right edge
- **THEN** the client sends set width naming that column's window

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
| `drag_band` | slide the band with the mouse | client |

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
