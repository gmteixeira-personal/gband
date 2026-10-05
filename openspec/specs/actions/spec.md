# actions Specification

## Purpose
Defines gband's actions: the one vocabulary that key bindings, and later Lua, mouse input, the command line and a command palette, use to change a client's view, the shared session or the client itself.

## Requirements

### Requirement: Kinds of action
Every action SHALL be exactly one of three kinds:

| kind | runs in | examples |
|---|---|---|
| view | the client alone, with no message to the server | focus a neighbouring pane, view another band |
| session | the server, which applies it to the shared layout | open, close, consume or expel a pane, change a column's width |
| client | the client alone, outside the view | detach, send the prefix key to the focused pane |

A key binding SHALL name exactly one action or one Lua function. Pressing a key bound to an action SHALL have the same effect as dispatching that action from any other source.

#### Scenario: Every binding names an action
- **WHEN** the bindings of the default configuration are listed
- **THEN** every entry names one view, session or client action

#### Scenario: Action from another source
- **WHEN** a test dispatches the close-pane session action to a client whose focused pane is the second of two
- **THEN** the second pane closes, as it does when the user presses Ctrl+Space then `q`

### Requirement: Session actions resolve against the view
A session action from a binding SHALL name no pane. Before sending it, the client SHALL resolve it against its view: the pane is the focused pane, and open pane also takes the viewed band. A session action that needs a pane SHALL be dropped when no pane is focused. Open pane SHALL be sent with no pane to open after when no pane is focused.

A session action dispatched with a target, as the lua-control capability defines, SHALL NOT be resolved against the view: it SHALL name the pane, band or pane to open after that its target names, whichever pane is focused.

#### Scenario: Resolve to the focused pane
- **WHEN** a view focuses pane 3 and resolves cycle width
- **THEN** the result is cycle width naming pane 3

#### Scenario: Nothing focused
- **WHEN** a view on the empty band resolves close pane
- **THEN** there is nothing to send

#### Scenario: Open pane on the empty band
- **WHEN** a view on the empty band resolves open pane
- **THEN** the result is open pane naming that band and no pane to open after

#### Scenario: Target overrides the view
- **WHEN** a view focuses pane 3 and cycle width is dispatched with the target pane 1
- **THEN** the result is cycle width naming pane 1

#### Scenario: Target on the empty band
- **WHEN** a view on the empty band dispatches close pane with the target pane 1
- **THEN** the result is close pane naming pane 1

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
