## MODIFIED Requirements

### Requirement: Layout events
Every change to the layout SHALL produce the events that describe it, in the order the changes happened. A change that leaves the layout as it was SHALL produce no event. The layout events SHALL be:

| event | content |
|---|---|
| window opened | the window, and the band it was placed in |
| window closed | the window, and the band it left |
| window moved | the window, and the band, column and row it now occupies |
| column width changed | the band, the column, its new width and its full-width flag |
| window heights changed | the band, the column, and the height of each of its windows, top to bottom: automatic with its weight, or fixed with its rows |
| band added | the band and its position |
| band removed | the band |

A window opened or moved into a column SHALL take an automatic height of weight 1 there, and the last window left in a column with an automatic height SHALL take weight 1, as the layout capability defines, without a window heights changed event.

#### Scenario: Open in the empty band
- **WHEN** a window opens in the last, empty band
- **THEN** the events are window opened for that band, then band added for the new empty band below it

#### Scenario: Last pane of a middle band closes
- **WHEN** the only window of a band that is not the last one closes
- **THEN** the events are window closed, then band removed

#### Scenario: Consume into a neighbour
- **WHEN** a window alone in its column is consumed into the column on its left
- **THEN** the events are one window moved for that window, naming the left column and its new row

#### Scenario: Consume at the edge
- **WHEN** a consume or expel leaves the layout unchanged
- **THEN** no event is produced

#### Scenario: Grow a pane's height
- **WHEN** the screen area is 80×24, the second column of a band holds P1 above P2, both with automatic heights of weight 1, and P1's height is grown
- **THEN** the events are one window heights changed naming that band, column 1, P1 fixed at 14 rows and P2 automatic with weight 1

#### Scenario: Grow a column's width
- **WHEN** a column of width 1/2 is grown
- **THEN** the events are one column width changed naming the width 3/5 and full width off

### Requirement: Session event bus
The server SHALL publish every layout event, every window exit with its exit status, and every client attaching and detaching, as session events. It SHALL publish them on one bus, in the order the session applied the changes. Every subscriber SHALL receive the events in that order. A subscriber that falls behind SHALL be told how many events it missed, and SHALL NOT slow the session or other subscribers. The server SHALL record every session event in its log at debug level.

#### Scenario: Pane opened through the bus
- **WHEN** a subscriber is on the bus and a client opens a window
- **THEN** the subscriber receives window opened for the new window before any later event

#### Scenario: Exit then close
- **WHEN** a window's program exits
- **THEN** the subscriber receives the window exit with its status, then window closed

#### Scenario: Attach and detach
- **WHEN** a client attaches and then detaches
- **THEN** the subscriber receives client attached, then client detached, naming the same client
