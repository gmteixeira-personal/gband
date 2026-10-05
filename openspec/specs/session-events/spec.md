# session-events Specification

## Purpose
Defines the events a gband session emits when its layout, its panes or its clients change, so that later features such as Lua, agent awareness and persistence can react to changes without polling the layout.

## Requirements

### Requirement: Layout events
Every change to the layout SHALL produce the events that describe it, in the order the changes happened. A change that leaves the layout as it was SHALL produce no event. The layout events SHALL be:

| event | content |
|---|---|
| pane opened | the pane, and the band it was placed in |
| pane closed | the pane, and the band it left |
| pane moved | the pane, and the band, column and row it now occupies |
| column moved | the band, the column's old position, and its new position |
| column width changed | the band, the column, its new width and its full-width flag |
| pane heights changed | the band, the column, and the height of each of its panes, top to bottom: automatic with its weight, or fixed with its rows |
| pane floated | the pane, the band, and its box record: column, row, width, full-width flag and rows |
| pane tiled | the pane, the band, the column it now occupies, and that column's width and full-width flag |
| floating box changed | the pane, the band, and its new box record |
| band added | the band and its position |
| band removed | the band |

A pane opened or moved into a column SHALL take an automatic height of weight 1 there, and the last pane left in a column with an automatic height SHALL take weight 1, as the layout capability defines, without a pane heights changed event. A pane that floats or opens floating SHALL produce no pane heights changed event for the column it left. Moving a pane within its column SHALL produce one pane moved event for each of the two panes that swapped, in order from the pane named by the action, and no pane heights changed event. A pane that opens floating SHALL produce pane opened and then pane floated.

#### Scenario: Open in the empty band
- **WHEN** a pane opens in the last, empty band
- **THEN** the events are pane opened for that band, then band added for the new empty band below it

#### Scenario: Last pane of a middle band closes
- **WHEN** the only pane of a band that is not the last one closes
- **THEN** the events are pane closed, then band removed

#### Scenario: Consume into a neighbour
- **WHEN** a pane alone in its column is consumed into the column on its left
- **THEN** the events are one pane moved for that pane, naming the left column and its new row

#### Scenario: Consume at the edge
- **WHEN** a consume or expel leaves the layout unchanged
- **THEN** no event is produced

#### Scenario: Grow a pane's height
- **WHEN** the screen area is 80×24, the second column of a band holds P1 above P2, both with automatic heights of weight 1, and P1's height is grown
- **THEN** the events are one pane heights changed naming that band, column 1, P1 fixed at 14 rows and P2 automatic with weight 1

#### Scenario: Grow a column's width
- **WHEN** a column of width 1/2 is grown
- **THEN** the events are one column width changed naming the width 3/5 and full width off

#### Scenario: Move a column
- **WHEN** a band holds columns A, B and C, and A is moved right
- **THEN** the events are one column moved naming that band, position 0 and position 1

#### Scenario: Swap two panes
- **WHEN** a column at position 2 holds P1 above P2, and P1 is moved down
- **THEN** the events are pane moved for P1 naming column 2 and row 1, then pane moved for P2 naming column 2 and row 0

#### Scenario: Float a pane
- **WHEN** the screen area is 80×24, the default column width is 1/2, and P2, alone in a column, is floated
- **THEN** the events are one pane floated naming P2, its band, column 20, row 2, width 1/2, full width off and 20 rows

#### Scenario: Move a floating pane
- **WHEN** the area is 80×24 and a floating pane whose box starts at column 20 is moved right
- **THEN** the events are one floating box changed naming its new column 28

#### Scenario: Tile a pane
- **WHEN** a floating pane of width 1/3 is tiled as the band's second column
- **THEN** the events are one pane tiled naming column 1, width 1/3 and full width off

#### Scenario: Open floating in the empty band
- **WHEN** a floating pane opens in the last, empty band
- **THEN** the events are pane opened, pane floated, then band added

### Requirement: Session event bus
The server SHALL publish every layout event, every pane exit with its exit status, and every client attaching and detaching, as session events. It SHALL publish them on one bus, in the order the session applied the changes. Every subscriber SHALL receive the events in that order. A subscriber that falls behind SHALL be told how many events it missed, and SHALL NOT slow the session or other subscribers. The server SHALL record every session event in its log at debug level.

#### Scenario: Pane opened through the bus
- **WHEN** a subscriber is on the bus and a client opens a pane
- **THEN** the subscriber receives pane opened for the new pane before any later event

#### Scenario: Exit then close
- **WHEN** a pane's program exits
- **THEN** the subscriber receives the pane exit with its status, then pane closed

#### Scenario: Attach and detach
- **WHEN** a client attaches and then detaches
- **THEN** the subscriber receives client attached, then client detached, naming the same client
