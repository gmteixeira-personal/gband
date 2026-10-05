## MODIFIED Requirements

### Requirement: Session actions
The server SHALL own the session's layout, as the layout capability defines it, and change it only through session actions. It SHALL apply the session actions of every client in the order it receives them, and send the resulting layout to every attached client. An action that names a pane or a band no longer in the layout SHALL be ignored. The session actions SHALL be:

| action | effect |
|---|---|
| open pane | start a pane program, as "Pane program" defines, or open a plugin pane, as "Plugin panes" defines, and place the pane as the layout capability's "Open a pane" defines, with the column width the action names, tiled or floating as the action names |
| close pane | close the named pane, as "Close a pane" defines |
| consume or expel | as the layout capability defines, left or right |
| move column | move the named pane's column, or its floating box, left or right, as the layout capability defines |
| move pane | move the named pane, or its floating box, down or up, as the layout capability defines |
| toggle floating | float the named tiled pane, or tile the named floating pane after the tiled pane the action names, as the floating-panes capability defines |
| set position | place the named floating pane at the column and row the action names, as the floating-panes capability defines |
| cycle width | cycle the width of the named pane's column or floating box |
| toggle full width | toggle full width of the named pane's column or floating box |
| grow width | grow the width of the named pane's column or floating box |
| shrink width | shrink the width of the named pane's column or floating box |
| grow height | grow the height of the named pane |
| shrink height | shrink the height of the named pane |
| reset height | reset the height of the named pane |
| set width | set the width of the named pane's column or floating box to the width the action names |
| set height | set the height of the named pane to the rows or the weight the action names |

Set position naming a tiled pane SHALL leave the layout unchanged.

After placing an opened pane whose action asks for focus, the server SHALL send the client that asked for it, after the layout that holds the pane, a message telling it to focus that pane. When the program of a new pane cannot be started, the server SHALL record the reason in its log and leave the layout unchanged.

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

#### Scenario: Open without focus
- **WHEN** a client asks to open a pane and the action does not ask for focus
- **THEN** every client receives a layout holding the new pane
- **AND** no client is told to focus it

#### Scenario: Open with a width
- **WHEN** a client asks to open a pane with the column width 1/4
- **THEN** every client receives a layout whose new column has width 1/4

#### Scenario: Set a column's width
- **WHEN** a client asks to set the width of pane 1's column to 2/5
- **THEN** every attached client receives a layout in which pane 1's column has width 2/5 and full width off

#### Scenario: Float a pane for every client
- **WHEN** two clients are attached and the first asks to toggle floating on tiled pane 2
- **THEN** both clients receive a layout in which pane 2 is in its band's floating list

#### Scenario: Open a floating pane with focus
- **WHEN** a client asks to open a floating pane that asks for focus
- **THEN** every client receives a layout whose band's floating list ends with the new pane
- **AND** only that client is told to focus it

#### Scenario: Set position on a tiled pane
- **WHEN** a client asks to set the position of tiled pane 1
- **THEN** the layout is unchanged

### Requirement: Pane resizes
The server SHALL send every change of the screen area or the layout to the attached clients at once, as "State updates" defines, without waiting for the PTYs.

A pane SHALL be shown when at least one client attached to its session reports it among its shown panes, as the wire-protocol capability defines. A client that detaches or disconnects SHALL show no pane. A pane SHALL be unshown until a client reports it.

A pane's terminal size SHALL be its tile's, as the layout capability gives it, when the pane is tiled, and its box's, as the floating-panes capability gives it, when the pane floats.

The server SHALL resize PTYs only when the session has settled: 100 ms have passed since the last change of the screen area, the layout or the set of shown panes. It SHALL then resize every shown pane whose PTY size differs from its terminal size, once, to that terminal size. A pane that is not shown SHALL keep its PTY size, whatever the screen area and the layout give it. A new pane's PTY SHALL start at its terminal size.

#### Scenario: Burst of terminal resizes
- **WHEN** a client attached to an 80×24 session showing one pane in a column of width 1/2 reports resizes to 90×25, 95×28 and 100×30, each less than 100 ms after the last
- **THEN** the pane's program receives exactly one SIGWINCH
- **AND** its PTY becomes 48 columns by 28 rows

#### Scenario: Held resize key
- **WHEN** a client showing one pane in a column of width 1/2 asks to grow that column's width four times, each less than 100 ms after the last
- **THEN** every attached client receives a layout after each action
- **AND** the pane's program receives exactly one SIGWINCH, after the last action

#### Scenario: Offscreen pane keeps its size
- **WHEN** a client with an 80×24 terminal views a band holding columns A, B and C, all of width 1/2, shows A and B, and resizes its terminal to 100×30
- **THEN** the PTYs of A and B become 48 columns by 28 rows
- **AND** C's PTY stays 38 columns by 22 rows, and C's program receives no SIGWINCH

#### Scenario: Offscreen pane resized when shown
- **WHEN** C's PTY has kept 38 columns by 22 rows while offscreen in a 100×30 area, and the client focuses C so that it shows C
- **THEN** C's PTY becomes 48 columns by 28 rows

#### Scenario: Shown by another client
- **WHEN** two clients are attached, the first shows only pane A, the second shows only pane C, and the screen area changes
- **THEN** both A and C are resized

#### Scenario: Pane leaves a stack while detached
- **WHEN** no client is attached, a column holds P1 and P2, and P2's program exits
- **THEN** P1's PTY keeps its size
- **AND** when a client attaches and shows P1, P1's PTY becomes the size of its tile, which now spans the column's height

#### Scenario: Floating pane takes its box size
- **WHEN** the screen area is 80×24, a shown floating pane has width 1/3 and `rows` 12, and its height is grown
- **THEN** once the session settles, its PTY becomes 24 columns by 12 rows
