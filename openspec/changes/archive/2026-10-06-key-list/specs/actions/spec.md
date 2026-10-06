## MODIFIED Requirements

### Requirement: Session actions resolve against the view
A session action from a binding SHALL name no window. Before sending it, the client SHALL resolve it against its view: the window is the focused window, tiled or floating, and open window also takes the viewed band. A session action that needs a window SHALL be dropped when no window is focused.

Open window SHALL be sent naming, as the window to open after, the tiled window this client focused most recently in the viewed band, when it is still tiled there, and no window otherwise. Toggling floating SHALL be sent naming the same tiled window, as the window the toggled window is tiled after.

Close window resolved against the view SHALL close the focused floating plugin window when one is focused, as the plugin-windows capability defines, even when no window is focused: it SHALL close that floating plugin window as `gband.win.close` does, and SHALL send nothing to the server. Otherwise it SHALL resolve to the focused window, tiled or floating. A floating window is a window, not a plugin window.

A session action dispatched with a target, as the lua-control capability defines, SHALL NOT be resolved against the view: it SHALL name the window, band or window to open after that its target names, whichever window or plugin window is focused.

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

#### Scenario: Close the focused floating plugin window
- **WHEN** window 3 is focused, a floating plugin window is focused, and close window is dispatched with no target
- **THEN** the floating plugin window closes, nothing is sent to the server, and window 3 stays open and focused

#### Scenario: Close the focused floating plugin window on the empty band
- **WHEN** a view on the empty band has a floating plugin window focused and resolves close window
- **THEN** the floating plugin window closes and there is nothing to send

#### Scenario: Close a focused floating window
- **WHEN** floating window 3 is focused, no floating plugin window is focused, and close window is dispatched with no target
- **THEN** the result is close window naming window 3

#### Scenario: Target names a window behind a floating plugin window
- **WHEN** a floating plugin window is focused and close window is dispatched with the target window 3
- **THEN** the client sends close window naming window 3 and the floating plugin window stays open
