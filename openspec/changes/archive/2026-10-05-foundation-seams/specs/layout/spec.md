## ADDED Requirements

### Requirement: Pane identity
A pane SHALL keep its identifier from the moment it opens until it closes. Consuming or expelling it, changing its column's width, and adding or removing other columns and workspaces SHALL NOT change its identifier.

#### Scenario: Identifier survives moves
- **WHEN** a workspace holds panes 1 and 2 in separate columns, and pane 2 is consumed into pane 1's column, expelled to the right again, and its column's width is cycled
- **THEN** after every step the layout holds exactly panes 1 and 2
- **AND** pane 2 sits where the step placed it
