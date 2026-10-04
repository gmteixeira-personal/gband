# actions Specification

## Purpose
Defines gband's actions: the one vocabulary that key bindings, and later Lua, mouse input, the command line and a command palette, use to change a client's view, the shared session or the client itself.

## Requirements

### Requirement: Kinds of action
Every action SHALL be exactly one of three kinds:

| kind | runs in | examples |
|---|---|---|
| view | the client alone, with no message to the server | focus a neighbouring pane, view another workspace |
| session | the server, which applies it to the shared layout | open, close, consume or expel a pane, change a column's width |
| client | the client alone, outside the view | detach, send a key to the focused pane |

A key binding SHALL name exactly one action. Pressing a bound key SHALL have the same effect as dispatching that action from any other source.

#### Scenario: Every binding names an action
- **WHEN** the client's binding table is listed
- **THEN** every entry names one view, session or client action

#### Scenario: Action from another source
- **WHEN** a test dispatches the close-pane session action to a client whose focused pane is the second of two
- **THEN** the second pane closes, as it does when the user presses Ctrl+A then `q`

### Requirement: Session actions resolve against the view
A session action from a binding SHALL name no pane. Before sending it, the client SHALL resolve it against its view: the pane is the focused pane, and open pane also takes the viewed workspace. A session action that needs a pane SHALL be dropped when no pane is focused. Open pane SHALL be sent with no pane to open after when no pane is focused.

#### Scenario: Resolve to the focused pane
- **WHEN** a view focuses pane 3 and resolves cycle width
- **THEN** the result is cycle width naming pane 3

#### Scenario: Nothing focused
- **WHEN** a view on the empty workspace resolves close pane
- **THEN** there is nothing to send

#### Scenario: Open pane on the empty workspace
- **WHEN** a view on the empty workspace resolves open pane
- **THEN** the result is open pane naming that workspace and no pane to open after
