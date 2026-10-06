## MODIFIED Requirements

### Requirement: Slide the band by dragging
`drag_band` SHALL slide the viewed band's camera or switch bands, never both in one gesture. `drag_window` pressed on empty ribbon SHALL act as `drag_band`. Let `dx` and `dy` be the pointer's movement in columns and rows since the press.

The gesture SHALL change nothing until a motion has `dx` of at least 2 either way or `dy` of at least 1 either way. That motion SHALL lock the gesture's axis: horizontal when the size of `dx` is at least twice the size of `dy`, and vertical otherwise, since a cell is about twice as tall as it is wide. The axis SHALL stay locked until the release, and movement along the other axis SHALL change nothing. A release before the axis locks SHALL be handled as the release of a horizontal slide.

Horizontal: each motion SHALL set the camera to its value at the press less `dx`, and the client SHALL draw it at once. Sliding SHALL send nothing to the server and change no focus until the release.

At the release of a horizontal slide, let C be the column of the viewed band whose span holds the strip position at the middle of the ribbon area, the camera plus half the ribbon area's width rounded down, or, when no column holds it, the column whose span lies nearest to it. Focus SHALL move to the tiled window this client focused most recently in C, or to C's first window, as focusing a window does. When C lies wholly inside the view, the camera SHALL stay where the slide left it, except that a strip that does not loop SHALL then be pulled back so it leaves no blank cells right of its end, as the layout-view capability's camera rule `"never"` pulls it back. Otherwise it SHALL move as that rule moves it. On a band with no column, the release SHALL return the camera to where it was at the press.

Vertical: the bands SHALL follow the pointer row for row. Let `H` be the client terminal's height, the height of each band's region as the animations capability's band switch draws it, and `v` the viewed band's index in the layout. Each motion SHALL set the drawn vertical position to `v` times `H` less `dy`, kept between 0 and the last band's index times `H`, and the client SHALL draw the bands there at once. Moving the pointer up SHALL therefore bring the band below into view, and moving it down the band above. A band other than the viewed one SHALL be drawn with the camera that viewing it would give this client. A vertical drag SHALL send nothing to the server, change no camera and change neither focus nor the viewed band until the release. When bands are added or removed above the viewed band during the drag, the drawn vertical position SHALL shift with them, so that no band jumps on screen. When the viewed band leaves the layout, the gesture SHALL end at once with no further change.

At the release of a vertical drag, let B be the band whose region holds the terminal's middle row: the band whose index is the drawn vertical position plus half of `H` rounded down, divided by `H` and rounded down. When B is the band below or above the viewed one, the client SHALL view it, as the layout-view capability's "Switch band" defines. When B is the viewed band, the view SHALL not change.

#### Scenario: Slide right and settle
- **WHEN** the terminal is 80 columns wide, a band holds four columns of 40 cells, the first is focused with the camera at 0, and the user drags with `drag_band` from column 70 to column 10, then releases
- **THEN** the camera is at 60 while dragging and stays at 60 after the release
- **AND** the third column is focused

#### Scenario: Partly shown column snaps
- **WHEN** the terminal is 80 columns wide, a band holds four columns of 60 cells, the first is focused with the camera at 0, and the user slides the band 70 cells left and releases
- **THEN** the second column is focused and the camera moves to 60

#### Scenario: Slide past the strip's end pulls back
- **WHEN** `loop_bands` is on, the terminal is 80 columns wide, a band holds two columns of 40 cells, the first is focused with the camera at 0, and the user drags with `drag_band` from column 70 to column 40, then releases
- **THEN** the camera is at 30 while dragging
- **AND** after the release the second column is focused and the camera moves to 0

#### Scenario: Middle button anywhere in navigation mode
- **WHEN** navigation mode is active with the default bindings and the user drags with the middle button over a tile
- **THEN** the band slides and no window moves

#### Scenario: Horizontal axis ignores vertical movement
- **WHEN** the terminal is 80×24, the layout holds B1 with four columns of 40 cells and B2 with a window, the client views B1 with the camera at 0, and the user drags with `drag_band` from column 70 and row 5 to column 40 and row 8, then to column 40 and row 20
- **THEN** the camera is at 30 after both motions and B2 is never drawn
- **AND** after the release B1 is still viewed

#### Scenario: Small movement locks no axis
- **WHEN** the user presses with `drag_band` at column 40 and row 10 and the pointer moves to column 41 and row 10
- **THEN** the camera and the drawn bands are unchanged

#### Scenario: Drag up to the band below
- **WHEN** the terminal is 80×24, the layout holds B1 and B2 with windows and an empty B3, the client views B1 at rest, and the user drags with `drag_band` from column 40 and row 20 to column 40 and row 4
- **THEN** rows 0 to 7 show rows 16 to 23 of B1 and rows 8 to 23 show rows 0 to 15 of B2
- **AND** no message is sent and B1 is still viewed
- **AND** after the release the client views B2 and focuses the window it last focused there

#### Scenario: Short drag returns
- **WHEN** the terminal is 80×24, the layout holds B1 and B2 with windows and an empty B3, the client views B1 at rest, and the user drags with `drag_band` from row 12 to row 6 and releases
- **THEN** B1 is still viewed with the same focus and camera

#### Scenario: Drag down to the band above
- **WHEN** the terminal is 80×24, the client views B2 of B1, B2 and an empty B3, and the user drags with `drag_band` from row 2 to row 20 and releases
- **THEN** the client views B1

#### Scenario: Vertical axis ignores sideways movement
- **WHEN** the terminal is 80×24, the client views B1 of four 40-cell columns with the camera at 0, and the user drags with `drag_band` from column 40 and row 10 to column 41 and row 5, then to column 0 and row 5
- **THEN** the camera stays at 0 and the drawn vertical position is 5 after both motions

#### Scenario: First band stops the drag
- **WHEN** the client views the first band and the user drags with `drag_band` 10 rows down and releases
- **THEN** the bands are drawn as before the press throughout, and the view is unchanged

#### Scenario: Drag from empty ribbon switches bands
- **WHEN** navigation mode is active with the default bindings, the terminal is 80×24, the client views B1 of B1, B2 and an empty B3, and the user drags with the left button on empty ribbon from row 22 to row 2 and releases
- **THEN** the client views B2 and no window moves
