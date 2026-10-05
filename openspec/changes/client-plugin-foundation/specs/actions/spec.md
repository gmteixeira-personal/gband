## MODIFIED Requirements

### Requirement: Lua names of actions
Every built-in action SHALL have one Lua name, under which `gband.action` holds it, and one description, the text of its action column:

| Lua name | action | kind |
|---|---|---|
| `focus_column_left` | focus the column to the left | view |
| `focus_column_right` | focus the column to the right | view |
| `focus_pane_down` | focus the pane below | view |
| `focus_pane_up` | focus the pane above | view |
| `focus_band_down` | view the band below | view |
| `focus_band_up` | view the band above | view |
| `open_pane` | open a pane running the user's shell | session |
| `close_pane` | close the pane | session |
| `consume_or_expel_left` | consume or expel the pane to the left | session |
| `consume_or_expel_right` | consume or expel the pane to the right | session |
| `cycle_column_width` | cycle the width of the pane's column | session |
| `toggle_full_width` | toggle full width of the pane's column | session |
| `grow_column_width` | grow the width of the pane's column | session |
| `shrink_column_width` | shrink the width of the pane's column | session |
| `grow_pane_height` | grow the height of the pane | session |
| `shrink_pane_height` | shrink the height of the pane | session |
| `reset_pane_height` | reset the height of the pane | session |
| `detach` | detach | client |
| `send_prefix` | send the prefix key to the focused pane | client |

Actions that the configuration capability's `gband.action.register` adds SHALL sit beside the built-in actions in `gband.action` and in `gband.action.list()`, and SHALL NOT take a built-in action's name.

#### Scenario: Every action is named
- **WHEN** no action is registered and the names `gband.action.list()` returns are read
- **THEN** they are exactly the Lua names in the table

#### Scenario: Built-in descriptions
- **WHEN** `gband.action.list()` is read
- **THEN** the entry named `cycle_column_width` has the description `cycle the width of the pane's column`

#### Scenario: Name and action agree
- **WHEN** a binding names `gband.action.cycle_column_width` and its keys are pressed with pane 3 focused
- **THEN** the client sends cycle width naming pane 3
