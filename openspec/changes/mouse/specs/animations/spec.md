## ADDED Requirements

### Requirement: Drawing during a gesture
While a drag gesture runs, as the mouse capability defines, the client SHALL draw the lifted tile, the floating box or floating plugin window being moved or resized, and the slid camera at the positions the gesture sets, with no animation. A layout that a gesture's own changes cause SHALL draw the gesture's window at its target at once, and SHALL animate the other windows as any layout change does.

When a lifted tile drops, its tile SHALL move from where it was drawn at the release to its new place, as tile movement animates. When the release of a band slide moves the camera, the camera SHALL scroll as camera scroll animates.

#### Scenario: Floating box follows the pointer
- **WHEN** animations are on and the user drags a floating window 5 cells right with `drag_window`
- **THEN** the next frame draws its box 5 cells right of where it was, with no frame in between

#### Scenario: Drop animates
- **WHEN** animations are on and the user drops a lifted tile into a new column
- **THEN** the dropped tile moves from the release position to its new place over the following frames
