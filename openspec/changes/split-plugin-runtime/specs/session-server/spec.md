## MODIFIED Requirements

### Requirement: Session actions
The server SHALL own the session's layout, as the layout capability defines it, and change it only through session actions. It SHALL apply the session actions of every client, and those the server's Lua calls as the server-runtime capability defines, in the order it receives them, and send the resulting layout to every attached client. An action that names a pane or a band no longer in the layout SHALL be ignored. The session actions SHALL be:

| action | effect |
|---|---|
| open pane | start a pane program, as "Pane programs" defines, and place the pane as the layout capability's "Open a pane" defines |
| close pane | close the named pane, as "Close a pane" defines |
| consume or expel | as the layout capability defines, left or right |
| cycle width | cycle the width of the named pane's column |
| toggle full width | toggle full width of the named pane's column |
| grow width | grow the width of the named pane's column |
| shrink width | shrink the width of the named pane's column |
| grow height | grow the height of the named pane |
| shrink height | shrink the height of the named pane |
| reset height | reset the height of the named pane |

After placing a pane opened at a client's request, the server SHALL send that client, after the layout that holds the pane, a message telling it to focus that pane. A pane opened by the server's Lua SHALL NOT change any client's focus. When the program of a new pane cannot be started, the server SHALL record the reason in its log and leave the layout unchanged.

#### Scenario: Open a pane
- **WHEN** two clients are attached and the first asks to open a pane next to the pane it focuses
- **THEN** both clients receive a layout holding both panes
- **AND** only the first client is told to focus the new pane

#### Scenario: Concurrent actions
- **WHEN** two clients each ask to open a pane at the same moment
- **THEN** the layout holds three panes, and every client receives the same layout

#### Scenario: Action on a closed pane
- **WHEN** a client asks to cycle the width of a pane that has already left the layout
- **THEN** the layout is unchanged

#### Scenario: Grow a pane's height
- **WHEN** the screen area is 80×24 and a client asks to grow the height of the top pane of a column holding two panes with automatic heights of weight 1
- **THEN** every attached client receives a layout in which the top pane has a fixed height of 14 rows and the bottom pane an automatic height of weight 1

#### Scenario: Action from the server's Lua
- **WHEN** a server handler of `PaneOpened` calls `gband.action.grow_column_width` for the new pane
- **THEN** every attached client receives a layout in which that pane's column is wider than the default width
