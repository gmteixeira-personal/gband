## ADDED Requirements

### Requirement: Focus a named pane
Focusing a named pane SHALL make that pane's band the viewed band and focus that pane, as a focus message from the server does. The camera SHALL follow the new focus as it follows any change of focus. Focusing a pane not in the layout SHALL leave the view unchanged.

#### Scenario: Pane in the viewed band
- **WHEN** a client views a band holding columns A, B and C, focuses A, and focuses the named pane in C
- **THEN** the client focuses that pane
- **AND** focusing the column to the left then focuses B

#### Scenario: Pane in another band
- **WHEN** a client views B1 and focuses a named pane in B2
- **THEN** the client views B2 with that pane focused

### Requirement: View a named band
Viewing a named band SHALL make it the viewed band. Focus SHALL go where "Switch band" sends it: to the pane this client last focused in that band when it is still there, otherwise to the top pane of its first column, or to no pane when the band is empty. Viewing the band already viewed, or a band not in the layout, SHALL leave the view unchanged.

#### Scenario: Jump two bands down
- **WHEN** the layout holds B1, B2 and an empty B3, and a client viewing B1 views B3 by name
- **THEN** the client views B3 with no focused pane

#### Scenario: Remembered focus
- **WHEN** a client focuses the second column of B2, views B1, and then views B2 by name
- **THEN** the client focuses the same pane in the second column of B2
