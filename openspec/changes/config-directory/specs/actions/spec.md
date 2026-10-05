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
- **WHEN** the bindings of the default configuration are listed
- **THEN** every entry names one view, session or client action

#### Scenario: Action from another source
- **WHEN** a test dispatches the close-pane session action to a client whose focused pane is the second of two
- **THEN** the second pane closes, as it does when the user presses Ctrl+A then `q`
