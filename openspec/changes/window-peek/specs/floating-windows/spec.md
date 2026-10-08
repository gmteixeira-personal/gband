## MODIFIED Requirements

### Requirement: Layers of a view
Each client's view SHALL hold, for each band, an active layer, either tiled or floating, and the window it last focused in each layer. A view SHALL enter a band it has not viewed before in the tiled layer. While the floating layer of the viewed band is active, the focused window SHALL be one of its floating windows that this client has not minimized, or the floating window this client peeks, as the layout-view capability's "Peek a window" defines. While the tiled layer is active, the focused window SHALL be a tiled window, or none when neither layer holds a window.

For the rules of this capability, a layer SHALL hold a window when it holds a window that this client has not minimized. A floating layer whose windows this client has all minimized SHALL therefore hold no window.

Focusing a floating window by any means other than a peek SHALL make the floating layer active. Focusing a tiled window by any means other than a peek SHALL make the tiled layer active. A peek SHALL make the peeked window's layer active only while the peek lasts, and SHALL NOT change the layer or the windows that the band remembers. Whenever no peek lasts, the active layer of the viewed band holds no window, and the other layer holds one, the other layer SHALL become active, with focus as "Switch layers" sends it. When neither layer holds a window and no peek lasts, the tiled layer SHALL be active and no window SHALL be focused.

#### Scenario: Remembered layer
- **WHEN** a client focuses floating window P3 in B1, views B2, and views B1 again
- **THEN** the floating layer of B1 is active and P3 is focused

#### Scenario: Last tiled window closes
- **WHEN** a client focuses P1, the only tiled window of a band that also holds floating window P3, and P1's program exits
- **THEN** the client focuses P3 in the floating layer

#### Scenario: Focus by number activates the layer
- **WHEN** a client focuses tiled window P1 and focuses floating window P3 by number
- **THEN** the floating layer is active and P3 is focused

#### Scenario: Band with only floating windows
- **WHEN** a band holds no column and one floating window P3, and a client views that band for the first time
- **THEN** the floating layer is active and P3 is focused

#### Scenario: Every floating window minimized
- **WHEN** a band holds no column and floating windows P3 and P4, and a client that focuses P4 minimizes P3 and then P4
- **THEN** the tiled layer is active and the client focuses no window

#### Scenario: Last tiled window closes beside minimized windows
- **WHEN** a client focuses P1, the only tiled window of a band that also holds floating window P3, has minimized P3, and P1's program exits
- **THEN** the client focuses no window and does not draw P3

#### Scenario: Peek keeps the remembered layer
- **WHEN** a client focuses tiled window P1 of B1, which also holds floating window P3, peeks P3, views B2, and views B1 again
- **THEN** the tiled layer of B1 is active and P1 is focused

#### Scenario: Peek a minimized window beside a tiled window
- **WHEN** a client focuses tiled window P1 of a band that also holds floating window P3, has minimized P3, and peeks P3
- **THEN** the floating layer is active and P3 is focused

### Requirement: Focus follows a window between layers
When a window this client focuses moves to the other layer of its band, by this client or by another, focus SHALL stay on that window, and the layer SHALL follow it. When the focused floating window leaves the layout, or this client minimizes it, focus SHALL go to the floating window this client focused most recently in that band among those it has not minimized, or to the last window of the floating list that it has not minimized when it focused none of them. When no such floating window remains, the tiled layer SHALL become active, with focus as "Switch layers" sends it, or with no focused window when the band holds no tiled window. These two rules SHALL NOT apply to a window this client peeks: when the peeked window leaves the layout, or this client minimizes it, focus SHALL go where the layout-view capability's "Peek a window" sends it.

#### Scenario: Float the focused window
- **WHEN** a client focuses P2 in the tiled layer and P2 is floated
- **THEN** the client focuses P2 in the floating layer

#### Scenario: Tile the focused window
- **WHEN** a client focuses floating window P3 and P3 is tiled
- **THEN** the client focuses P3 in the tiled layer, in its new column

#### Scenario: Last floating window closes
- **WHEN** a client focuses floating window P3, the only floating window of a band holding column A with P1, and P3's program exits
- **THEN** the client focuses P1 in the tiled layer

#### Scenario: Minimize the last floating window
- **WHEN** a client focuses floating window P3, the only floating window of a band holding column A with P1, and minimizes P3
- **THEN** the client focuses P1 in the tiled layer

#### Scenario: Minimize with an unfocused floating window left
- **WHEN** a band's floating list holds P3, P4 and P5, a client focused P4 and then P5, minimized P4, and then minimizes P5
- **THEN** the client focuses P3 in the floating layer

#### Scenario: Tile the peeked window
- **WHEN** a client focuses tiled window P1 of a band that also holds floating window P3, peeks P3, and P3 is tiled
- **THEN** the client still peeks P3 and focuses it in the tiled layer, in its new column

#### Scenario: Peeked floating window closes
- **WHEN** a band's floating list holds P3, P4 and P5, a client focused P4 and then P5, peeks P3, and P3's program exits
- **THEN** the client focuses P5 in the floating layer

### Requirement: Stacking
Each client SHALL draw the floating windows of the viewed band in its own stacking order, from bottom to top. A window this client has minimized SHALL NOT be in its stacking order. The floating windows this client has never focused SHALL come first, in the order of the floating list. The floating windows it has focused SHALL follow, in the order of their last focus, as the layout-view capability's "Focus order" defines, the most recently focused on top. Minimizing a window SHALL NOT change the order of the others. Stacking SHALL NOT change the layout or another client's drawing.

While this client peeks a floating window of the viewed band, as the layout-view capability's "Peek a window" defines, it SHALL draw that window above every other floating window of the band, whether or not it has minimized the window. A peek SHALL NOT change the stacking order. When the peek ends, the window SHALL be drawn at its place in the order again, or not at all when this client has minimized it.

#### Scenario: Focus raises
- **WHEN** a band's floating list holds P3 and then P4, both boxes overlap, and a client focuses P4 and then P3
- **THEN** that client draws P3 over P4

#### Scenario: Never focused
- **WHEN** a band's floating list holds P3 and then P4, and a client has focused neither
- **THEN** that client draws P4 over P3

#### Scenario: Other clients keep their order
- **WHEN** two clients view the same band, the first focuses P3 and the second focuses P4
- **THEN** the first draws P3 on top and the second draws P4 on top

#### Scenario: Minimized window leaves the order
- **WHEN** a band's floating list holds P3, P4 and P5 with overlapping boxes, a client focused P5, P4 and then P3, and it minimizes P3
- **THEN** that client draws P4 over P5 and does not draw P3

#### Scenario: Peeked window on top
- **WHEN** a band's floating list holds P3, P4 and P5 with overlapping boxes, a client focused P5, P4 and then P3, and it peeks P5
- **THEN** that client draws P5 over P3, and P3 over P4

#### Scenario: Next peek returns the window to its place
- **WHEN** that client then peeks P4
- **THEN** it draws P4 over P3, and P3 over P5

### Requirement: Minimize a floating window
Each client's view SHALL hold a set of minimized windows. Minimizing a floating window SHALL add it to the set of the client that minimizes it. Only a floating window SHALL be in the set. Minimizing a tiled window, or a window that is already in the set and that this client does not peek, SHALL leave the view unchanged.

Minimizing SHALL change only the view of the client that minimizes. It SHALL NOT change the layout, the screen area, a window's terminal size or a window's PTY size. It SHALL NOT change the view or the drawing of another client. The client SHALL send the server no action for it.

A window in a client's set SHALL NOT be drawn by that client. It SHALL NOT be one of that client's shown windows, as the layout-view capability defines them, and SHALL NOT be a pointer target, as the mouse capability defines them. These three rules SHALL NOT apply while that client peeks the window, as the layout-view capability's "Peek a window" defines. A window in the set SHALL keep its place in its band's floating list and its box record. A session action that changes its box, such as a move, a resize or a placing, SHALL change the box as for any floating window, and the window SHALL stay in the set.

When a client minimizes its focused window, focus SHALL move as "Focus follows a window between layers" defines. When a client minimizes the window that its drag gesture moves or resizes, as the mouse capability defines, the gesture SHALL end at once with no further change. A selection in a window that a client minimizes SHALL no longer be drawn. Minimizing the window that a client peeks SHALL end the peek, and SHALL add the window to the set when the set does not hold it yet.

A window SHALL leave the set in these cases:

- The client focuses the window by any means other than a peek, such as `gband.window.focus` without `peek` or a focus message from the server. Focusing it SHALL make the floating layer of its band active and put the window on top of the client's stacking order, as focusing any floating window does. This SHALL restore the window. A peek SHALL NOT restore it.
- The window leaves its band's floating list, because a client tiles it or because it leaves the layout. When the window floats again, it SHALL NOT be minimized.

A reload of the configuration SHALL keep the set. Each time a client attaches, its set SHALL start empty.

#### Scenario: Minimize the focused floating window
- **WHEN** a band holds column A with P1 and floating windows P3 and P4, a client focused P3 and then P4, and the client runs `minimize_window`
- **THEN** the client focuses P3 in the floating layer and does not draw P4
- **AND** the layout is unchanged and the client sends the server no action

#### Scenario: Other clients still draw it
- **WHEN** two clients view a band with floating window P3, and the first client minimizes P3
- **THEN** the second client still draws P3

#### Scenario: Minimize a tiled window
- **WHEN** a client focuses tiled window P1 and runs `minimize_window`
- **THEN** the view is unchanged

#### Scenario: Hidden window keeps its PTY size
- **WHEN** the screen area is 80×24, floating window P3 has width 1/2 and `rows` 20, so its PTY is 38×18, the only attached client minimizes P3, and the screen area then grows to 100×30
- **THEN** P3's PTY stays 38×18 and its program receives no SIGWINCH
- **AND** when the client focuses P3, P3's PTY becomes 48×18 once the session settles

#### Scenario: Focus restores
- **WHEN** a band's floating list holds P3 and P4 with overlapping boxes, a client focused P3 and then P4, minimized P3, and then focuses P3 by number
- **THEN** the client draws P3 over P4 and focuses P3 with the floating layer active

#### Scenario: Tiling restores
- **WHEN** a client has minimized floating window P3, and another client tiles P3 and later floats it again
- **THEN** the first client draws P3 in its new column, and then in its box

#### Scenario: Box moves while minimized
- **WHEN** the screen area is 80×24, a client has minimized floating window P3 whose box starts at column 20, another client moves P3 right, and the first client then focuses P3
- **THEN** the first client draws P3's box from column 28

#### Scenario: Reload keeps the set
- **WHEN** a client minimizes floating window P3 and then saves `user/init.lua`
- **THEN** after the reload the client still does not draw P3

#### Scenario: Peek does not restore
- **WHEN** a band's floating list holds P3 and P4 with overlapping boxes, a client focused P3 and then P4, minimized P3, and peeks P3
- **THEN** the client draws P3 over P4, and P3 is still in the set

#### Scenario: Minimize the peeked window
- **WHEN** a client focuses tiled window P1 of a band that also holds floating window P3, peeks P3, and runs `minimize_window`
- **THEN** P3 is in the set, the client does not draw P3, and it focuses P1 in the tiled layer

#### Scenario: Minimize a peeked minimized window
- **WHEN** a client focuses tiled window P1 of a band that also holds floating window P3, has minimized P3, peeks P3, and runs `minimize_window`
- **THEN** the client does not draw P3, and it focuses P1 in the tiled layer
