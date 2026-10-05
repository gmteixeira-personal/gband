## MODIFIED Requirements

### Requirement: Session actions resolve against the view
A session action from a binding SHALL name no pane. Before sending it, the client SHALL resolve it against its view: the pane is the focused pane, tiled or floating, and open pane also takes the viewed band. A session action that needs a pane SHALL be dropped when no pane is focused.

Open pane SHALL be sent naming, as the pane to open after, the tiled pane this client focused most recently in the viewed band, when it is still tiled there, and no pane otherwise. Toggling floating SHALL be sent naming the same tiled pane, as the pane the toggled pane is tiled after.

Close pane resolved against the view SHALL close the focused float when one is focused, as the plugin-windows capability defines: it SHALL close that float as `gband.win.close` does, and SHALL send nothing to the server. Otherwise it SHALL resolve to the focused pane, tiled or floating. A floating pane is a pane, not a float.

A session action dispatched with a target, as the lua-control capability defines, SHALL NOT be resolved against the view: it SHALL name the pane, band or pane to open after that its target names, whichever pane or float is focused.

#### Scenario: Resolve to the focused pane
- **WHEN** a view focuses pane 3 and resolves cycle width
- **THEN** the result is cycle width naming pane 3

#### Scenario: Nothing focused
- **WHEN** a view on the empty band resolves close pane
- **THEN** there is nothing to send

#### Scenario: Open pane on the empty band
- **WHEN** a view on the empty band resolves open pane
- **THEN** the result is open pane naming that band and no pane to open after

#### Scenario: Open pane from the floating layer
- **WHEN** a view last focused tiled pane 1, now focuses floating pane 3 in the same band, and resolves open pane
- **THEN** the result is open pane naming that band and pane 1 as the pane to open after

#### Scenario: Tile the focused floating pane
- **WHEN** a view last focused tiled pane 1, now focuses floating pane 3, and resolves toggle floating
- **THEN** the result is toggle floating naming pane 3 and pane 1 as the pane to tile after

#### Scenario: Target overrides the view
- **WHEN** a view focuses pane 3 and cycle width is dispatched with the target pane 1
- **THEN** the result is cycle width naming pane 1

#### Scenario: Target on the empty band
- **WHEN** a view on the empty band dispatches close pane with the target pane 1
- **THEN** the result is close pane naming pane 1

#### Scenario: Close the focused float
- **WHEN** pane 3 is focused, a float is focused, and close pane is dispatched with no target
- **THEN** the float closes, nothing is sent to the server, and pane 3 stays open and focused

#### Scenario: Close a focused floating pane
- **WHEN** floating pane 3 is focused, no float is focused, and close pane is dispatched with no target
- **THEN** the result is close pane naming pane 3

#### Scenario: Target names a pane behind a float
- **WHEN** a float is focused and close pane is dispatched with the target pane 3
- **THEN** the client sends close pane naming pane 3 and the float stays open

### Requirement: Lua names of actions
Every built-in action SHALL have one Lua name, under which `gband.action` holds it, and one description, the text of its action column. A description is text gband shows the user, so it SHALL call a pane a window, as the README does:

| Lua name | action | kind |
|---|---|---|
| `focus_column_left` | focus the column to the left | view |
| `focus_column_right` | focus the column to the right | view |
| `focus_pane_down` | focus the window below | view |
| `focus_pane_up` | focus the window above | view |
| `focus_band_down` | view the band below | view |
| `focus_band_up` | view the band above | view |
| `switch_focus_floating_tiled` | switch focus between floating and tiled windows | view |
| `open_pane` | open a window running the user's shell | session |
| `close_pane` | close the window | session |
| `consume_or_expel_left` | consume or expel the window to the left | session |
| `consume_or_expel_right` | consume or expel the window to the right | session |
| `move_column_left` | move the column or floating window to the left | session |
| `move_column_right` | move the column or floating window to the right | session |
| `move_pane_down` | move the window down | session |
| `move_pane_up` | move the window up | session |
| `toggle_pane_floating` | float or tile the window | session |
| `cycle_column_width` | cycle the width of the window's column | session |
| `toggle_full_width` | toggle full width of the window's column | session |
| `grow_column_width` | grow the width of the window's column | session |
| `shrink_column_width` | shrink the width of the window's column | session |
| `grow_pane_height` | grow the height of the window | session |
| `shrink_pane_height` | shrink the height of the window | session |
| `reset_pane_height` | reset the height of the window | session |
| `detach` | detach | client |
| `send_prefix` | send the prefix key to the focused window | client |

A width action named after a pane's column SHALL act on the box of a floating pane, as the floating-panes capability defines.

Actions that the configuration capability's `gband.action.register` adds SHALL sit beside the built-in actions in `gband.action` and in `gband.action.list()`, and SHALL NOT take a built-in action's name.

#### Scenario: Every action is named
- **WHEN** no action is registered and the names `gband.action.list()` returns are read
- **THEN** they are exactly the Lua names in the table

#### Scenario: Built-in descriptions
- **WHEN** `gband.action.list()` is read
- **THEN** the entry named `cycle_column_width` has the description `cycle the width of the window's column`
- **AND** the entry named `close_pane` has the description `close the window`

#### Scenario: Name and action agree
- **WHEN** a binding names `gband.action.cycle_column_width` and its keys are pressed with pane 3 focused
- **THEN** the client sends cycle width naming pane 3

#### Scenario: Toggle floating by name
- **WHEN** a binding names `gband.action.toggle_pane_floating` and its keys are pressed with tiled pane 3 focused
- **THEN** pane 3 floats and stays focused
