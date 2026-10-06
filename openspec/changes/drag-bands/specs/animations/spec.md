## MODIFIED Requirements

### Requirement: Drawing during a gesture
While a drag gesture runs, as the mouse capability defines, the client SHALL draw the lifted tile, the floating box or floating plugin window being moved or resized, the slid camera, and the bands at the drawn vertical position a vertical band drag sets, at the positions the gesture sets, with no animation. A layout that a gesture's own changes cause SHALL draw the gesture's window at its target at once, and SHALL animate the other windows as any layout change does.

When a lifted tile drops, its tile SHALL move from where it was drawn at the release to its new place, as tile movement animates. When the release of a band slide moves the camera, the camera SHALL scroll as camera scroll animates. When a vertical band drag is released, the drawn vertical position SHALL move from where the drag left it to the viewed band's region, as the band switch animates, whether or not the release changed the viewed band. Every band drawn at the release SHALL keep the camera it was drawn with until it leaves the terminal.

#### Scenario: Floating box follows the pointer
- **WHEN** animations are on and the user drags a floating window 5 cells right with `drag_window`
- **THEN** the next frame draws its box 5 cells right of where it was, with no frame in between

#### Scenario: Drop animates
- **WHEN** animations are on and the user drops a lifted tile into a new column
- **THEN** the dropped tile moves from the release position to its new place over the following frames

#### Scenario: Bands follow the pointer
- **WHEN** animations are on, the terminal is 80×24, the client views B1 of B1, B2 and an empty B3, and the user drags with `drag_band` 6 rows up
- **THEN** the next frame draws B1 from row -6 and B2 from row 18, with no frame in between

#### Scenario: Release continues the switch
- **WHEN** animations are on, the terminal is 80×24, the client views B1 of B1, B2 and an empty B3, and the user drags with `drag_band` 16 rows up and releases
- **THEN** the first frame after the release draws B2 from row 8
- **AND** later frames draw B2 higher until it starts at row 0, with B1 keeping the camera it was drawn with

#### Scenario: Release slides back
- **WHEN** animations are on, the terminal is 80×24, the client views B1 of B1, B2 and an empty B3, and the user drags with `drag_band` 6 rows up and releases
- **THEN** later frames draw B1 lower until it starts at row 0 and B2 is no longer drawn

#### Scenario: Release with animations off
- **WHEN** animations are off and the user releases a vertical band drag that moved the bands 16 rows up
- **THEN** the next frame shows only the newly viewed band
