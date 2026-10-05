## MODIFIED Requirements

### Requirement: Kinds of action
Every action SHALL be exactly one of three kinds:

| kind | runs in | examples |
|---|---|---|
| view | the client alone, with no message to the server | focus a neighbouring pane, view another workspace |
| session | the server, which applies it to the shared layout | open, close, consume or expel a pane, change a column's width |
| client | the client alone, outside the view | detach, send the prefix key to the focused pane |

A key binding SHALL name exactly one action or one Lua function. Pressing a key bound to an action SHALL have the same effect as dispatching that action from any other source.

#### Scenario: Every binding names an action
- **WHEN** the bindings of `defaults.lua` are listed
- **THEN** every entry names one view, session or client action

#### Scenario: Action from another source
- **WHEN** a test dispatches the close-pane session action to a client whose focused pane is the second of two
- **THEN** the second pane closes, as it does when the user presses Ctrl+A then `q`

## ADDED Requirements

### Requirement: Lua names of actions
Every action SHALL have one Lua name, under which `gband.action` holds it:

| Lua name | action | kind |
|---|---|---|
| `focus_column_left` | focus the column to the left | view |
| `focus_column_right` | focus the column to the right | view |
| `focus_pane_down` | focus the pane below | view |
| `focus_pane_up` | focus the pane above | view |
| `focus_workspace_down` | view the workspace below | view |
| `focus_workspace_up` | view the workspace above | view |
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

#### Scenario: Every action is named
- **WHEN** the keys of `gband.action` are listed
- **THEN** they are exactly the Lua names in the table

#### Scenario: Name and action agree
- **WHEN** a binding names `gband.action.cycle_column_width` and its keys are pressed with pane 3 focused
- **THEN** the client sends cycle width naming pane 3
