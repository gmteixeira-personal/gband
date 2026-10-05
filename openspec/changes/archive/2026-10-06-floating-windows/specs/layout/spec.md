## MODIFIED Requirements

### Requirement: Layout structure
A session's layout SHALL be an ordered list of bands, stacked top to bottom. Each band SHALL hold an ordered list of columns, left to right, and an ordered list of floating panes, as the floating-panes capability defines. Each column SHALL hold an ordered list of one or more panes, top to bottom, and a width. Every pane and every band SHALL have an identifier that is unique within the session and is never reused. A column SHALL never be empty: a column whose last pane leaves it SHALL be removed.

#### Scenario: Column removed with its last pane
- **WHEN** a band holds columns A, B and C, B holds one pane, and that pane closes
- **THEN** the band holds columns A and C, in that order

#### Scenario: Identifiers not reused
- **WHEN** pane 2 closes and a new pane opens
- **THEN** the new pane's identifier is not 2

#### Scenario: Column removed when its last pane floats
- **WHEN** a band holds columns A, B and C, B holds one pane, and that pane is floated
- **THEN** the band holds columns A and C, in that order, and one floating pane

### Requirement: Dynamic bands
The layout SHALL always hold at least one band, and its last band SHALL always be empty. A band SHALL be empty when it holds no column and no floating pane. When a pane is placed in the last band, tiled or floating, a new empty band SHALL be added below it. When any band other than the last becomes empty, it SHALL be removed.

#### Scenario: Initial layout
- **WHEN** a session starts with its first pane
- **THEN** the layout holds two bands
- **AND** the first holds one column with that pane, and the second is empty

#### Scenario: Pane opened in the empty band
- **WHEN** the layout holds bands B1, with panes, and B2, empty, and a pane opens in B2
- **THEN** the layout holds B1, B2 with the new pane, and a new empty band B3

#### Scenario: Middle band emptied
- **WHEN** the layout holds B1, B2 and an empty B3, and the last pane of B1 closes
- **THEN** the layout holds B2 and B3, in that order

#### Scenario: Floating pane keeps its band
- **WHEN** the layout holds B1 with only floating pane P3, B2 with panes, and an empty B3
- **THEN** B1 is not removed
- **AND** when P3 closes, the layout holds B2 and B3

#### Scenario: Floating pane opened in the empty band
- **WHEN** the layout holds B1, with panes, and B2, empty, and a floating pane opens in B2
- **THEN** the layout holds B1, B2 with the floating pane, and a new empty band B3

### Requirement: Open a pane
Opening a pane SHALL name a band and, optionally, a pane in that band, a column width, and whether the pane floats.

When the pane does not float, it SHALL be placed alone in a new column, whose width is as "Column widths" defines. The new column SHALL be inserted immediately to the right of the named pane's column, or as the band's first column when no pane is named. No other column's width SHALL change.

When the pane floats, it SHALL be appended to the band's floating list, and the named pane SHALL be ignored. It SHALL take a new box, as the floating-panes capability's "Float a pane" defines, with the width the action names in place of the configuration's `default_column_width` when the action names one.

#### Scenario: Open right of the focused column
- **WHEN** a band holds columns A and B, and a pane opens next to a pane in A
- **THEN** the band holds A, the new column and B, in that order
- **AND** A and B keep their widths

#### Scenario: Open in an empty band
- **WHEN** a pane opens in an empty band with no pane named
- **THEN** that band holds one column with the new pane

#### Scenario: Open a floating pane
- **WHEN** the screen area is 80×24, the default column width is 1/2, and a floating pane opens in a band holding columns A and B
- **THEN** the band still holds A and B, and its floating list ends with the new pane
- **AND** the new pane's box record has width 1/2, `rows` 20, `col` 20 and `row` 2

#### Scenario: Open a floating pane with a width
- **WHEN** the screen area is 80×24 and a floating pane opens with the width 1/4
- **THEN** the new pane's box record has width 1/4, `rows` 20, `col` 30 and `row` 2


### Requirement: Close a pane
A pane SHALL leave the layout when its program exits, whether the program exited by itself or because the pane was closed. A plugin pane, as the session-server capability defines it, SHALL leave the layout when it is closed. A tiled pane's column and band SHALL then follow the column rule and the dynamic band rule. A floating pane SHALL leave its band's floating list, and its band SHALL then follow the dynamic band rule.

#### Scenario: Pane leaves a stack
- **WHEN** a column holds panes P1, P2 and P3, and P2's program exits
- **THEN** the column holds P1 and P3, in that order, and keeps its width

#### Scenario: Plugin pane closed
- **WHEN** a band holds columns A, B and C, and B holds only a plugin pane, which is closed
- **THEN** the band holds A and C, in that order

#### Scenario: Floating pane exits
- **WHEN** a band's floating list holds P3 and P4, and P3's program exits
- **THEN** the band's floating list holds only P4

### Requirement: Pane identity
A pane SHALL keep its identifier from the moment it opens until it closes. Consuming or expelling it, moving it or its column, floating or tiling it, changing its column's width or its box, and adding or removing other columns and bands SHALL NOT change its identifier.

#### Scenario: Identifier survives moves
- **WHEN** a band holds panes 1 and 2 in separate columns, and pane 2 is consumed into pane 1's column, expelled to the right again, and its column's width is cycled
- **THEN** after every step the layout holds exactly panes 1 and 2
- **AND** pane 2 sits where the step placed it

#### Scenario: Identifier survives floating
- **WHEN** pane 2 is floated, moved right, and tiled again
- **THEN** after every step the layout holds pane 2 under the same identifier

## ADDED Requirements

### Requirement: Move a column
Moving a tiled pane's column left or right SHALL swap that column with the adjacent column in that direction. Both columns SHALL keep their panes, widths, full-width flags and heights. When no column exists in that direction, nothing SHALL change. Moving a floating pane left or right SHALL move its box, as the floating-panes capability defines.

#### Scenario: Move a column right
- **WHEN** a band holds columns A, B and C, and a pane of A has its column moved right
- **THEN** the band holds B, A and C, in that order, and every column keeps its width

#### Scenario: Move at the edge
- **WHEN** a pane of the band's first column has its column moved left
- **THEN** the layout is unchanged

### Requirement: Move a pane within its column
Moving a tiled pane down or up SHALL swap it with the adjacent pane in that direction in its column. Each pane SHALL keep its own height, automatic with its weight or fixed with its rows, and the column SHALL keep its width. When no pane exists in that direction, nothing SHALL change. Moving a floating pane down or up SHALL move its box, as the floating-panes capability defines.

#### Scenario: Move a pane down
- **WHEN** a column holds P1 with a fixed height of 16 rows above P2 with an automatic height of weight 1, and P1 is moved down
- **THEN** the column holds P2 with an automatic height of weight 1 above P1 with a fixed height of 16 rows

#### Scenario: Move at the bottom
- **WHEN** a column holds P1 and P2, and P2 is moved down
- **THEN** the layout is unchanged

#### Scenario: Lone pane
- **WHEN** a pane alone in its column is moved up
- **THEN** the layout is unchanged
