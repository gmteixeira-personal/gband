## MODIFIED Requirements

### Requirement: Kinds of action
Every action SHALL be exactly one of three kinds:

| kind | runs in | examples |
|---|---|---|
| view | the client alone, with no message to the server | focus a neighbouring window, view another band |
| session | the server, which applies it to the shared layout | open, close, consume or expel a window, change a column's width |
| client | the client alone, outside the view | detach, send the prefix key to the focused window |

A key binding SHALL name exactly one action or one Lua function. Pressing a key bound to an action SHALL have the same effect as dispatching that action from any other source.

#### Scenario: Every binding names an action
- **WHEN** the bindings of the default configuration are listed
- **THEN** every entry names one view, session or client action

#### Scenario: Action from another source
- **WHEN** a test dispatches the close-window session action to a client whose focused window is the second of two
- **THEN** the second window closes, as it does when the user presses Ctrl+Space then `q`

### Requirement: Session actions resolve against the view
A session action from a binding SHALL name no window. Before sending it, the client SHALL resolve it against its view: the window is the focused window, tiled or floating, and open window also takes the viewed band. A session action that needs a window SHALL be dropped when no window is focused.

Open window SHALL be sent naming, as the window to open after, the tiled window this client focused most recently in the viewed band, when it is still tiled there, and no window otherwise. Toggling floating SHALL be sent naming the same tiled window, as the window the toggled window is tiled after.

A session action dispatched with a target, as the lua-control capability defines, SHALL NOT be resolved against the view: it SHALL name the window, band or window to open after that its target names, whichever window is focused.

#### Scenario: Resolve to the focused window
- **WHEN** a view focuses window 3 and resolves cycle width
- **THEN** the result is cycle width naming window 3

#### Scenario: Nothing focused
- **WHEN** a view on the empty band resolves close window
- **THEN** there is nothing to send

#### Scenario: Open window on the empty band
- **WHEN** a view on the empty band resolves open window
- **THEN** the result is open window naming that band and no window to open after

#### Scenario: Open window from the floating layer
- **WHEN** a view last focused tiled window 1, now focuses floating window 3 in the same band, and resolves open window
- **THEN** the result is open window naming that band and window 1 as the window to open after

#### Scenario: Tile the focused floating window
- **WHEN** a view last focused tiled window 1, now focuses floating window 3, and resolves toggle floating
- **THEN** the result is toggle floating naming window 3 and window 1 as the window to tile after

#### Scenario: Target overrides the view
- **WHEN** a view focuses window 3 and cycle width is dispatched with the target window 1
- **THEN** the result is cycle width naming window 1

#### Scenario: Target on the empty band
- **WHEN** a view on the empty band dispatches close window with the target window 1
- **THEN** the result is close window naming window 1

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
