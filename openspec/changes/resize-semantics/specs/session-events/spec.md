## MODIFIED Requirements

### Requirement: Layout events
Every change to the layout SHALL produce the events that describe it, in the order the changes happened. A change that leaves the layout as it was SHALL produce no event. The layout events SHALL be:

| event | content |
|---|---|
| pane opened | the pane, and the workspace it was placed in |
| pane closed | the pane, and the workspace it left |
| pane moved | the pane, and the workspace, column and row it now occupies |
| column width changed | the workspace, the column, its new width and its full-width flag |
| pane heights changed | the workspace, the column, and the height weights of its panes, top to bottom |
| workspace added | the workspace and its position |
| workspace removed | the workspace |

A pane opened or moved into a column SHALL take weight 1 there, as the layout capability defines, without a pane heights changed event.

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
- **WHEN** the second column of a workspace holds P1 and P2, both of weight 1, and P1's height is grown
- **THEN** the events are one pane heights changed naming that workspace, column 1, and the weights 2 and 1

#### Scenario: Grow a column's width
- **WHEN** a column of width 1/2 is grown
- **THEN** the events are one column width changed naming the width 3/5 and full width off
