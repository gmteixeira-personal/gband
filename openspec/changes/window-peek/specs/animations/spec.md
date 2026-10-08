## MODIFIED Requirements

### Requirement: Floating windows at rest
The client SHALL draw every floating window that it has not minimized, and the floating window it peeks, at its box, as the client-attach capability places it, in every frame, without animating its position or size. A window that the client minimizes SHALL NOT be drawn from the first frame after it is minimized. A window that the client restores SHALL be drawn at its box from the first frame after it is restored. A minimized window that the client peeks SHALL be drawn at its box from the first frame after the peek starts, and SHALL NOT be drawn from the first frame after the peek ends. A window that moves from a column to the floating layer SHALL be drawn at its box from the first frame. A window that moves from the floating layer to a column SHALL be drawn at its tile position from the first frame, as a window new to the band is. The tiles of the other windows SHALL move as "Tile movement" defines.

#### Scenario: Float a window between two columns
- **WHEN** the viewed band holds columns A, B and C, and B's only window is floated
- **THEN** the first frame draws that window at its box
- **AND** C's tile slides left from B's end to where B started

#### Scenario: Move a floating window
- **WHEN** a floating window is moved right by one step
- **THEN** the next frame draws its box at the new column

#### Scenario: Minimize and restore at once
- **WHEN** animations are on, the viewed band holds column A with P1 and floating window P3, the client focuses P3 and minimizes it
- **THEN** the next frame draws no part of P3, and A's tile does not move
- **AND** when the client focuses P3 again, the next frame draws P3 at its box

#### Scenario: Peek at once
- **WHEN** animations are on, the viewed band holds column A with P1 and floating window P3, the client focuses P1, has minimized P3, and peeks P3
- **THEN** the next frame draws P3 at its box, and A's tile does not move
- **AND** when the client then focuses P1 again, the next frame draws no part of P3
