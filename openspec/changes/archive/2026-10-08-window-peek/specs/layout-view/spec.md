## ADDED Requirements

### Requirement: Focus order
Each client's view SHALL hold a focus counter, and a last focus for each window of the layout that the client has focused. The counter SHALL be 0 when the client attaches. Each time the view records a focus of a window, the counter SHALL rise by one, and the window's last focus SHALL take the counter's new value.

The view SHALL record a focus each time it focuses a window, whether or not that window was focused before: by a focus action, by focusing a named window, by viewing a band, by a press or a gesture that focuses a window, when the client attaches, and when focus moves because the focused window left the layout or was minimized. Focus that stays on the focused window while the layout changes SHALL record nothing. A peek and the end of a peek SHALL record nothing, as "Peek a window" defines.

A window this client has never focused SHALL have no last focus. A window that leaves the layout SHALL lose its last focus. A reload of the configuration SHALL keep the counter and every last focus, as it keeps the rest of the view.

The focus order SHALL be the order of last focus: a higher last focus is a more recent focus. Every rule of this capability and of the floating-windows capability that names the window this client focused most recently, last focused or never focused SHALL read the focus order.

#### Scenario: Counter rises with each focus
- **WHEN** a client attaches to a session whose first band holds the columns of windows 1 and 2, focuses the column to the right, and then the column to the left
- **THEN** window 1's last focus is 3 and window 2's last focus is 2

#### Scenario: Never focused
- **WHEN** a client attaches to a session whose first band holds window 1 in a column and floating window 3, and focuses no other window
- **THEN** window 1's last focus is 1 and window 3 has no last focus

#### Scenario: Viewing a band records a focus
- **WHEN** a client attaches to a session whose first band holds window 1 and whose second band holds window 2, views the band below, and then the band above
- **THEN** window 1's last focus is 3 and window 2's last focus is 2

#### Scenario: Reload keeps the order
- **WHEN** a client has focused windows 1, 2 and then 1 again since it attached, and the user saves `user/init.lua`
- **THEN** after the reload window 1's last focus is 3 and window 2's last focus is 2

#### Scenario: Attach starts again
- **WHEN** a client focused windows 1, 2 and then 1 again, detaches, and attaches again with window 1 focused
- **THEN** window 1's last focus is 1 and window 2 has no last focus

### Requirement: Peek a window
Peeking a named window SHALL focus it as "Focus a named window" focuses it: the client views the window's band, focuses the window, shows the window's layer as active, and the camera follows the new focus. A peek SHALL record nothing in this client's view. It SHALL NOT change the focus order, the window that each band remembers as last focused in each layer, the layer that each band remembers as active, the stacking order or the set of minimized windows. A peek SHALL NOT restore a window this client has minimized. For every rule that names the window this client focused most recently, last focused or never focused, a peek SHALL NOT count as a focus. Peeking a window not in the layout SHALL leave the view unchanged.

While a peek lasts, the peeked window SHALL be this client's focused window. View actions, session actions resolved against the view, and the focus that the client reports to Lua and in its events SHALL use it as they use any focused window. A peeked floating window SHALL be drawn above every other floating window of its band, as the floating-windows capability's "Stacking" defines, and SHALL be shown and a pointer target, whether or not this client has minimized it. A peeked tiled window SHALL be drawn as the focused tiled window is, under the band's floating windows.

The peek SHALL end, and a focus SHALL take its place, when one of these happens:

- The view focuses a window by any means other than a peek, the peeked window included. That focus SHALL be recorded as usual.
- The client peeks another window. The new peek SHALL take the place of the old one.
- The viewed band changes.

A focus of the peeked window by other means SHALL restore it when this client has minimized it, put it on top of the stacking order, and record the focus. The focused window does not change, so it SHALL emit no `FocusChanged` and no `BandChanged`.

The peek SHALL end, and no window SHALL take the focus in its place, when one of these happens:

- The client views the band it already views by name, as "View a named band" defines.
- This client minimizes the peeked window, as the floating-windows capability defines.
- The peeked window leaves the layout.
- A reload of the configuration loads the new configuration, and so closes every plugin window, as the plugin-windows capability defines. A reload that fails keeps the previous configuration, its plugin windows and the peek.

In these four cases, focus SHALL go to the window this client last focused in the viewed band, in the layer that window is in, when that window is still there and this client has not minimized it, and this SHALL record no focus. Otherwise focus SHALL go where "Switch band" sends it on viewing that band.

When the peek ends, the window SHALL return to its place in the stacking order, and SHALL be hidden again when this client has minimized it and no focus restored it. A peeked window that moves to the other layer of its band SHALL stay peeked, and the shown layer SHALL follow it. A layout change that keeps the peeked window in the layout SHALL NOT end the peek. A drag gesture pressed on the peeked window focuses it, as the mouse capability defines, and so ends the peek as any focus does. Each time a client attaches, it SHALL peek no window.

#### Scenario: Peek a minimized window
- **WHEN** band 1 holds floating windows 3 and 4 with overlapping boxes, a client focused 3 and then 4, minimized 3, and peeks 3
- **THEN** the client focuses 3 with the floating layer active and draws 3 over 4
- **AND** 3 is still minimized, and the last focus of 3 and of 4 is unchanged

#### Scenario: Focus elsewhere hides it again
- **WHEN** the client of "Peek a minimized window" then focuses 4 by number
- **THEN** the client focuses 4 and does not draw 3

#### Scenario: Focus the peeked window
- **WHEN** band 1 holds floating windows 3 and 4 with overlapping boxes, a client focused 3 and then 4, minimized 3, peeks 3, and then focuses 3 by number
- **THEN** 3 is no longer minimized, the client draws 3 over 4, and 3 has the highest last focus
- **AND** the focus by number emits no `FocusChanged`

#### Scenario: Peek in another band
- **WHEN** a client views band 1 with window 1 focused, band 2 holds tiled windows 2 and 5, the client last focused 5 there, and it peeks 2
- **THEN** the client views band 2 with window 2 focused
- **AND** when it then views band 1 by name and band 2 again, it focuses window 5

#### Scenario: Moving focus from a peek
- **WHEN** band 1 holds the columns of windows 1, 2 and 3, a client focuses window 1, peeks window 2, and focuses the column to the right
- **THEN** the client focuses window 3
- **AND** window 2's last focus is unchanged

#### Scenario: View the viewed band
- **WHEN** band 1 holds only floating window 3, a client that views band 1 has minimized 3 and so focuses no window, peeks 3, and then views band 1 by name
- **THEN** the client focuses no window and does not draw 3

#### Scenario: Peeked window closes
- **WHEN** a client focuses tiled window 1 of band 1, which also holds floating window 3, peeks 3, and the program of 3 exits
- **THEN** the client focuses window 1 with the tiled layer active
- **AND** window 1's last focus is unchanged

#### Scenario: Reload ends the peek
- **WHEN** band 1 holds floating windows 3 and 4 with overlapping boxes, a client focused 3 and then 4, minimized 3, peeks 3, and the user saves `user/init.lua`
- **THEN** after the reload the client focuses 4 and does not draw 3
- **AND** the last focus of 3 and of 4 is unchanged

#### Scenario: Failed reload keeps the peek
- **WHEN** a client focuses tiled window 1 of band 1, which also holds floating window 3, peeks 3, and the user saves a `user/init.lua` that raises an error
- **THEN** the client still focuses 3

#### Scenario: Peek a tiled window
- **WHEN** `loop_bands` is off, the policy is `"never"`, the terminal is 80 columns wide, a band holds three columns of 40 cells, the first is focused with the camera at 0, and the client peeks the window of the third column
- **THEN** the client focuses that window and the camera moves to 40
- **AND** when it then views the band it views by name, it focuses the window of the first column and the camera moves to 0

## MODIFIED Requirements

### Requirement: Per-client view
Each attached client SHALL hold its own view: the viewed band, the focused window, an active layer for each band and a set of minimized windows, as the floating-windows capability defines them, the focus order and the peeked window, as "Focus order" and "Peek a window" define them, and a camera position for each band. The focused window SHALL be in the viewed band, tiled or floating, and SHALL NOT be a window this client has minimized, unless this client peeks it. There SHALL be no focused window while the viewed band is empty, or while every window it holds is a floating window this client has minimized and the client peeks none of them. View actions SHALL change only the view of the client that performs them, and SHALL NOT change the layout or the view of any other client. The one exception is `center_column` on the floating layer, which sends the placing of the focused floating window and so changes the layout, as "Center the focused column" defines.

#### Scenario: Two clients focus independently
- **WHEN** two clients view a band with columns A and B, both focus a window in B, and the first client focuses the column to the left
- **THEN** the first client focuses a window in A
- **AND** the second client still focuses the window in B

#### Scenario: Two clients in different layers
- **WHEN** two clients view a band with column A and floating window P3, both focus a window in A, and the first switches layers
- **THEN** the first client focuses P3
- **AND** the second client still focuses the window in A

#### Scenario: Two clients minimize independently
- **WHEN** two clients view a band with column A and floating window P3, both focus P3, and the first client runs `minimize_window`
- **THEN** the first client focuses a window in A and does not draw P3
- **AND** the second client still focuses and draws P3

#### Scenario: Two clients peek independently
- **WHEN** two clients view a band with floating windows P3 and P4 with overlapping boxes, both focused P3 and then P4, and the first client peeks P3
- **THEN** the first client focuses P3 and draws it over P4
- **AND** the second client still focuses P4 and draws it over P3

### Requirement: Follow layout changes
When the layout changes, each client's view SHALL follow it:

- When the viewed band is removed, the client SHALL view the band that now holds the removed one's position, or the last band when none does.
- When the focused window moves within the viewed band, between columns, within a column or between layers, focus SHALL stay on that window.
- When the focused tiled window leaves the layout, focus SHALL go to the column at the same position, or to the last column when none is there. Within that column, focus SHALL go to the window at the same position from the top, or to the bottom window when none is there.
- When the focused floating window leaves the layout, focus SHALL go where the floating-windows capability sends it.
- When the peeked window leaves the layout, focus SHALL go where "Peek a window" sends it, and the two rules above SHALL NOT apply to it.
- When the server tells the client to focus a window, the client SHALL view that window's band and focus it.

#### Scenario: Focused column closed
- **WHEN** a client focuses column B in a band holding A, B and C, and B's only window exits
- **THEN** the client focuses the window in C

#### Scenario: Last column closed
- **WHEN** a client focuses column C, the last of A, B and C, and C's only window exits
- **THEN** the client focuses the window in B

#### Scenario: Focused window expelled
- **WHEN** a client focuses P2 in a column holding P1 and P2, and P2 is expelled to the right
- **THEN** the client still focuses P2, now alone in its new column

#### Scenario: Focused column moved
- **WHEN** a client focuses a window of column A in a band holding A and B, and A is moved right
- **THEN** the client still focuses that window, now in the second column

#### Scenario: Viewed band removed
- **WHEN** a client views B1 of B1, B2 and an empty B3, and the last window of B1 exits
- **THEN** the client views B2

#### Scenario: Peeked column closed
- **WHEN** a client focuses the window of column A in a band holding A, B and C, peeks the window of B, and B's only window exits
- **THEN** the client focuses the window of A

### Requirement: Shown windows
A view's shown windows SHALL be the windows of its viewed band whose tile, as the layout capability's tile geometry gives it for the screen area, or whose box, as the floating-windows capability places it, has at least one cell inside the client's terminal. A floating window this client has minimized SHALL NOT be shown, wherever its box is, unless this client peeks it. A tile's cells SHALL be placed at its strip position less the viewed band's camera position, from the terminal's top row. While the band's strip loops, as the Camera requirement defines, a tile's cells SHALL be placed at its drawn copy's position instead, and a tile with no drawn copy SHALL NOT be shown. Wherever the client-attach and animations capabilities draw a tile at its strip position less a camera, they SHALL use its drawn copy while the strip loops. A box's cells SHALL be placed at its column and row, from the terminal's top-left cell, whatever the camera. A tile that a box covers SHALL still be shown. A view of an empty band SHALL show no window.

#### Scenario: Column beyond the right edge
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds three columns of width 1/2, and the camera is at strip position 0
- **THEN** the shown windows are those of the first two columns

#### Scenario: Partly visible column
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds two columns of width 2/3, and the client focuses the second
- **THEN** the windows of both columns are shown

#### Scenario: Window below the bottom edge
- **WHEN** the screen area is 120×60, the client's terminal is 100×30, and the viewed band holds one column of width 1/2 with two windows with automatic heights of weight 1
- **THEN** only the top window is shown

#### Scenario: Other bands
- **WHEN** the layout holds B1 with window P1 and B2 with window P2, and the client views B2
- **THEN** the only shown window is P2

#### Scenario: Floating window shown
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds three columns of width 1/2 and floating window P4, and the camera is at 40
- **THEN** the shown windows are those of the second and third columns, and P4

#### Scenario: Box below a small terminal
- **WHEN** the screen area is 120×60, the client's terminal is 100×30, and the viewed band's floating window P4 has its box at row 40
- **THEN** P4 is not shown

#### Scenario: First column shown after the last
- **WHEN** `loop_bands` is on, the screen area and the client's terminal are both 80×24, the viewed band holds three columns of width 1/2, and the camera is at strip position 80
- **THEN** the shown windows are those of the third and first columns
- **AND** the first column's tile is placed at terminal column 40

#### Scenario: Floating window stays in place across the seam
- **WHEN** `loop_bands` is on, the screen area and the client's terminal are both 80×24, the viewed band holds three columns of width 1/2 and floating window P4 with its box at column 10, and the camera moves from 40 to 80
- **THEN** P4's box stays at terminal column 10

#### Scenario: Minimized window not shown
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds one column of width 1/2 with P1 and floating window P4, and the client has minimized P4
- **THEN** the only shown window is P1

#### Scenario: Peeked minimized window shown
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds one column of width 1/2 with P1 and floating window P4, and the client has minimized P4 and peeks it
- **THEN** the shown windows are P1 and P4
- **AND** when the client then focuses P1, the only shown window is P1

### Requirement: View a named band
Viewing a named band SHALL make it the viewed band. Focus SHALL go where "Switch band" sends it. Viewing the band already viewed SHALL end a peek, as "Peek a window" defines, and SHALL otherwise leave the view unchanged. Viewing a band not in the layout SHALL leave the view unchanged.

#### Scenario: Jump two bands down
- **WHEN** the layout holds B1, B2 and an empty B3, and a client viewing B1 views B3 by name
- **THEN** the client views B3 with no focused window

#### Scenario: Remembered focus
- **WHEN** a client focuses the second column of B2, views B1, and then views B2 by name
- **THEN** the client focuses the same window in the second column of B2

#### Scenario: Viewed band ends a peek
- **WHEN** a client focuses window 1 of B1, peeks window 2 of B1, and then views B1 by name
- **THEN** the client focuses window 1
- **AND** window 1's last focus is unchanged
