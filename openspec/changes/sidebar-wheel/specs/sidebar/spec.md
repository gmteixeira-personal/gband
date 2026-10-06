## ADDED Requirements

### Requirement: Band wheel
A wheel step down on any cell of a shown sidebar, with no Ctrl, Alt or Shift held, SHALL view the band below the viewed band, as `focus_band_down` does. A wheel step up there SHALL view the band above, as `focus_band_up` does. Each step SHALL move one band from the band viewed when the step is handled, whatever row is under the pointer and whatever key table is active. A step past the first or the last band SHALL change nothing. A step left or right, or a step while the sidebar is not shown, SHALL do nothing. The sidebar SHALL take no action on a step with Ctrl, Alt or Shift held. A key binding on that step SHALL act as it does elsewhere. With the default keys, an Alt step over the sidebar therefore views one band, as a step without Alt does. A step SHALL NOT change the active key table.

#### Scenario: Wheel down over the labels
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user turns the wheel one step down over column 0, row 2
- **THEN** the client views band 2, and the label `2` is drawn in `SidebarBandActive`

#### Scenario: Steps move one band each
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user turns the wheel two steps down over the sidebar
- **THEN** the client views band 3

#### Scenario: Wheel up
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 3, and the user turns the wheel one step up over the sidebar
- **THEN** the client views band 2

#### Scenario: Any row of the sidebar
- **WHEN** the layout holds bands 1 and 2, the client views band 1, and the user turns the wheel one step down over the sidebar's mode letter, its blank row 1, or its last row
- **THEN** the client views band 2

#### Scenario: Past the last band
- **WHEN** the layout holds bands 1 and 2, the client views band 2, and the user turns the wheel one step down over the sidebar
- **THEN** the client still views band 2

#### Scenario: Wheel on the right sidebar
- **WHEN** the client's terminal is 80×24, the sidebar is on the right, the layout holds bands 1 and 2, the client views band 1, and the user turns the wheel one step down over column 79
- **THEN** the client views band 2

#### Scenario: Wheel with Alt held
- **WHEN** the default configuration is in use, the layout holds bands 1, 2 and 3, the client views band 1, and the user turns the wheel one step down over the sidebar with Alt held
- **THEN** the client views band 2

#### Scenario: Wheel in navigation mode
- **WHEN** the default configuration is in use, the layout holds bands 1 and 2, the client views band 1, navigation mode is active, and the user turns the wheel one step down over the sidebar
- **THEN** the client views band 2 and row 0 of the sidebar still shows `N`
