## MODIFIED Requirements

### Requirement: Session actions resolve against the view
A session action from a binding SHALL name no window. Before sending it, the client SHALL resolve it against its view: the window is the focused window, tiled or floating, and open window also takes the viewed band. A session action that needs a window SHALL be dropped when no window is focused.

Open window SHALL be sent naming, as the window to open after, the tiled window this client focused most recently in the viewed band, when it is still tiled there, and no window otherwise. Toggling floating SHALL be sent naming the same tiled window, as the window the toggled window is tiled after.

A session action dispatched with a target, as the lua-control capability defines, SHALL NOT be resolved against the view: it SHALL name the window, band or window to open after that its target names, whichever window is focused.

Grow and shrink of a column's width SHALL be sent naming a step: the target's `step` when the action was dispatched with one, and the client's `width_step` option otherwise. Grow and shrink of a window's height SHALL be sent naming a step the same way, from `height_step`. A session action from the server's Lua SHALL name the target's `step`, or 1/10 when its target names none.

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

#### Scenario: Step from the client's option
- **WHEN** `user/init.lua` sets `width_step` to `1/20` and a view focusing window 3 resolves grow width
- **THEN** the result is grow width naming window 3 and the step 1/20

#### Scenario: Step from a target
- **WHEN** a binding function calls `gband.action.shrink_window_height({ window = 2, step = 1/4 })`
- **THEN** the result is shrink height naming window 2 and the step 1/4
