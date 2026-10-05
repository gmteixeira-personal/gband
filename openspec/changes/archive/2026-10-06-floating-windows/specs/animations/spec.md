## ADDED Requirements

### Requirement: Floating panes at rest
The client SHALL draw every floating pane at its box, as the client-attach capability places it, in every frame, without animating its position or size. A pane that moves from a column to the floating layer SHALL be drawn at its box from the first frame. A pane that moves from the floating layer to a column SHALL be drawn at its tile position from the first frame, as a pane new to the band is. The tiles of the other panes SHALL move as "Tile movement" defines.

#### Scenario: Float a pane between two columns
- **WHEN** the viewed band holds columns A, B and C, and B's only pane is floated
- **THEN** the first frame draws that pane at its box
- **AND** C's tile slides left from B's end to where B started

#### Scenario: Move a floating pane
- **WHEN** a floating pane is moved right by one step
- **THEN** the next frame draws its box at the new column
