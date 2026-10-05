# session-events Specification

## Purpose
Defines the events a gband session emits when its layout, its panes or its clients change, so that later features such as Lua, agent awareness and persistence can react to changes without polling the layout.

## Requirements

### Requirement: Layout events
Every change to the layout SHALL produce the events that describe it, in the order the changes happened. A change that leaves the layout as it was SHALL produce no event. The layout events SHALL be:

| event | content |
|---|---|
| pane opened | the pane, and the workspace it was placed in |
| pane closed | the pane, and the workspace it left |
| pane moved | the pane, and the workspace, column and row it now occupies |
| column width changed | the workspace, the column, its new width and its full-width flag |
| pane heights changed | the workspace, the column, and the height of each of its panes, top to bottom: automatic with its weight, or fixed with its rows |
| workspace added | the workspace and its position |
| workspace removed | the workspace |

A pane opened or moved into a column SHALL take an automatic height of weight 1 there, and the last pane left in a column with an automatic height SHALL take weight 1, as the layout capability defines, without a pane heights changed event.

#### Scenario: Open in the empty workspace
- **WHEN** a pane opens in the last, empty workspace
- **THEN** the events are pane opened for that workspace, then workspace added for the new empty workspace below it

#### Scenario: Last pane of a middle workspace closes
- **WHEN** the only pane of a workspace that is not the last one closes
- **THEN** the events are pane closed, then workspace removed

#### Scenario: Consume into a neighbour
- **WHEN** a pane alone in its column is consumed into the column on its left
- **THEN** the events are one pane moved for that pane, naming the left column and its new row

#### Scenario: Consume at the edge
- **WHEN** a consume or expel leaves the layout unchanged
- **THEN** no event is produced

#### Scenario: Grow a pane's height
- **WHEN** the screen area is 80×24, the second column of a workspace holds P1 above P2, both with automatic heights of weight 1, and P1's height is grown
- **THEN** the events are one pane heights changed naming that workspace, column 1, P1 fixed at 14 rows and P2 automatic with weight 1

#### Scenario: Grow a column's width
- **WHEN** a column of width 1/2 is grown
- **THEN** the events are one column width changed naming the width 3/5 and full width off

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
