## MODIFIED Requirements

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
