## MODIFIED Requirements

### Requirement: Band click
A press of the left button on a cell of a shown sidebar that holds a band label SHALL view the band that label stands for, as `gband.band.view` does, whatever key table is active. A press of the left button on a cell that holds an agent label SHALL act as "Agent click" defines. A press of the left button on the apps character SHALL act as "Apps character" defines. A press of the left or the right button on the drawn error marker SHALL act as "Error marker press" defines. A press of another button, or on any other cell of the sidebar, SHALL do nothing. A press SHALL NOT change the active key table, except as "Error marker press" defines.

#### Scenario: Click a band label
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user presses the left button on column 0, row 4
- **THEN** the client views band 3, and the label `3` is drawn in `SidebarBandActive`

#### Scenario: Click on the right sidebar
- **WHEN** the client's terminal is 80×24, the sidebar is on the right, the layout holds bands 1 and 2, the client views band 1, and the user presses the left button on column 79, row 3
- **THEN** the client views band 2

#### Scenario: Click beside the labels
- **WHEN** the client views band 1 and the user presses the left button on the sidebar's mode letter, its blank row 1, the divider, or a row below the last label
- **THEN** the client still views band 1

#### Scenario: Click the mode letter's row with the modal style
- **WHEN** the modal key style is in use, the client views band 1, and the user presses the left button on column 0, row 0
- **THEN** the client still views band 1 and no floating plugin window opens

#### Scenario: Right click on a label
- **WHEN** the layout holds bands 1 and 2, the client views band 1, and the user presses the right button on the label `2`
- **THEN** the client still views band 1

#### Scenario: Click the blank error marker row
- **WHEN** the client's terminal is 80×24, the default configuration is in use, the client reports no error, and the user presses the left button on column 0, row 23
- **THEN** no plugin window opens

## ADDED Requirements

### Requirement: Error marker press
While the error marker is drawn, as "Error marker" defines, a press of the left button on its cell SHALL dispatch the action `errors.open` when `gband.action` holds it, as the error-list capability defines, whatever key table is active. Opening the error list SHALL make `root` the active table, as the error-list capability defines. A press of the right button on that cell SHALL dispatch the action `errors.clear` when `gband.action` holds it, whatever key table is active, and SHALL NOT change the active key table. Either press SHALL do nothing when `gband.action` does not hold its action. While the sidebar is 1 row high, its one row SHALL be the error marker's cell while the marker is drawn, and a press there SHALL NOT act as "Apps character" defines.

#### Scenario: Left click opens the error list
- **WHEN** the client's terminal is 80×24, the default configuration is in use, the user file holds a syntax error, and the user presses the left button on column 0, row 23
- **THEN** a focused floating plugin window titled `errors  c clear` opens and lists the error
- **AND** `root` is active

#### Scenario: Left click in navigation mode
- **WHEN** the default configuration is in use with the modal key style, the sidebar shows its error marker, navigation mode is active, and the user presses the left button on the marker
- **THEN** the error list opens and `root` is active

#### Scenario: Right click clears the errors
- **WHEN** the client's terminal is 80×24, the default configuration is in use, the sidebar shows its error marker, and the user presses the right button on column 0, row 23
- **THEN** `gband.errors()` returns an empty list and row 23 of column 0 is blank
- **AND** no plugin window opens and the active key table does not change

#### Scenario: Right click with the error list open
- **WHEN** the error list is open as a floating plugin window holding one error, and the user presses the right button on the sidebar's error marker
- **THEN** the error list shows `no errors` and is titled `errors`

#### Scenario: Error list plugin not set up
- **WHEN** `user/init.lua` calls `gband.keystyle.use()`, sets up `gband.sidebar` and not `gband.errors`, a binding function raises an error, and the user presses the left or the right button on the error marker
- **THEN** no plugin window opens and `gband.errors()` still returns the error

#### Scenario: One-row sidebar with the floating style
- **WHEN** the floating key style is in use, the sidebar is 1 row high, the client reports an error, and the user presses the left button on that row
- **THEN** the error list opens and the window list does not
