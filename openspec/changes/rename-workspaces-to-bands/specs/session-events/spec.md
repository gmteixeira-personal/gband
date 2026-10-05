## MODIFIED Requirements

### Requirement: Layout events
Every change to the layout SHALL produce the events that describe it, in the order the changes happened. A change that leaves the layout as it was SHALL produce no event. The layout events SHALL be:

| event | content |
|---|---|
| pane opened | the pane, and the band it was placed in |
| pane closed | the pane, and the band it left |
| pane moved | the pane, and the band, column and row it now occupies |
| column width changed | the band, the column, its new width and its full-width flag |
| pane heights changed | the band, the column, and the height of each of its panes, top to bottom: automatic with its weight, or fixed with its rows |
| band added | the band and its position |
| band removed | the band |

A pane opened or moved into a column SHALL take an automatic height of weight 1 there, and the last pane left in a column with an automatic height SHALL take weight 1, as the layout capability defines, without a pane heights changed event.

#### Scenario: Open in the empty workspace
- **WHEN** a pane opens in the last, empty band
- **THEN** the events are pane opened for that band, then band added for the new empty band below it

#### Scenario: Last pane of a middle workspace closes
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
