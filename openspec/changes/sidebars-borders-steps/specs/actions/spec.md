## MODIFIED Requirements

### Requirement: Session actions resolve against the view
A session action from a binding SHALL name no pane. Before sending it, the client SHALL resolve it against its view: the pane is the focused pane, tiled or floating, and open pane also takes the viewed band. A session action that needs a pane SHALL be dropped when no pane is focused.

Open pane SHALL be sent naming, as the pane to open after, the tiled pane this client focused most recently in the viewed band, when it is still tiled there, and no pane otherwise. Toggling floating SHALL be sent naming the same tiled pane, as the pane the toggled pane is tiled after.

A session action dispatched with a target, as the lua-control capability defines, SHALL NOT be resolved against the view: it SHALL name the pane, band or pane to open after that its target names, whichever pane is focused.

Grow and shrink of a column's width SHALL be sent naming a step: the target's `step` when the action was dispatched with one, and the client's `width_step` option otherwise. Grow and shrink of a pane's height SHALL be sent naming a step the same way, from `height_step`. A session action from the server's Lua SHALL name the target's `step`, or 1/10 when its target names none.

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

#### Scenario: Step from the client's option
- **WHEN** `user/init.lua` sets `width_step` to `1/20` and a view focusing pane 3 resolves grow width
- **THEN** the result is grow width naming pane 3 and the step 1/20

#### Scenario: Step from a target
- **WHEN** a binding function calls `gband.action.shrink_pane_height({ pane = 2, step = 1/4 })`
- **THEN** the result is shrink height naming pane 2 and the step 1/4
