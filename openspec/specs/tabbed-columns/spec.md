# tabbed-columns Specification

## Purpose
Defines tabbed columns, as niri has them: a column that shows one of its windows at a time over the whole column, the action that turns it on and off, the height its tabs share, which tab each client shows, and the tab indicator drawn on the column's left border.

## Requirements

### Requirement: Tabbed display
Every column SHALL carry a tabbed flag. A column whose flag is on is a tabbed column, and each of its windows is a tab, in the column's window order.

Toggling tabbed display SHALL name a window. When the window is tiled, the tabbed flag of its column SHALL flip. The windows' own heights and the column's tab height, as "Tab height" defines it, SHALL stay as they were, so turning tabs off gives back the stack the column had. When the window floats, nothing SHALL change.

A column holding one window SHALL be tabbed like any other. A window that enters a tabbed column, by consume or by moving to a place above or below one of its windows, SHALL become a tab at its place in the column's window order, and its own height SHALL be as the layout capability defines for the column it enters. A tabbed column SHALL stay tabbed while windows leave it, and SHALL be removed with its last window, as the layout capability's column rule defines.

A column made by opening a window, expelling a window, moving a window to a new column or tiling a floating window SHALL have the tabbed flag off and an automatic tab height.

#### Scenario: Turn tabs on
- **WHEN** the screen area is 80×24, a column of width 1/2 holds P1, P2 and P3 with automatic heights of weight 1, and tabbed display is toggled naming P2
- **THEN** the column is tabbed
- **AND** the tiles of P1, P2 and P3 are each 40×24 at row 0, and each window's terminal size is 38×22

#### Scenario: Turn tabs off gives the stack back
- **WHEN** the screen area is 80×24, a column holds P1 with a fixed height of 16 rows above P2 with an automatic height, and tabbed display is toggled twice naming P1
- **THEN** the column is not tabbed, and the tiles of P1 and P2 are 16 and 8 rows high

#### Scenario: Floating window
- **WHEN** tabbed display is toggled naming a floating window
- **THEN** the layout is unchanged

#### Scenario: Lone window
- **WHEN** the screen area is 80×24 and tabbed display is toggled naming a window alone in a column of width 1/2
- **THEN** the column is tabbed and the window's tile is still 40×24

#### Scenario: Consume into a tabbed column
- **WHEN** a band holds column A with P1 and tabbed column B with P2 and P3, and P1 is consumed to the right
- **THEN** the band holds one tabbed column with P2, P3 and P1, in that order

#### Scenario: Expel from a tabbed column
- **WHEN** a band holds tabbed column B with P2 and P3, and P3 is expelled to the right
- **THEN** the band holds tabbed column B with P2, and a column with P3 that is not tabbed

#### Scenario: New column is not tabbed
- **WHEN** a band holds tabbed column A, and a window opens next to a window in A
- **THEN** the new window's column is not tabbed

### Requirement: Tab height
Every column SHALL carry a tab height, shared by all its tabs: automatic, which fills the screen area's height, or fixed, a number of rows. A new column's tab height SHALL be automatic.

While a column is tabbed, a request to change the height of one of its windows SHALL change the column's tab height instead, and SHALL leave every window's own height as it was:

- Growing or shrinking SHALL give the column a fixed tab height: the height of its tabs' tiles, plus, when growing, or less, when shrinking, one step in rows, as the layout capability's "Window heights" measures a step. The result SHALL be no less than 3 rows and no more than the screen area's height.
- Setting a number of rows SHALL give the column a fixed tab height of that many rows, kept within the same limits.
- Resetting, or setting a weight, SHALL give the column an automatic tab height.

When the result equals the column's tab height, nothing SHALL change. While a column is not tabbed, its tab height SHALL not change and SHALL not affect any tile.

#### Scenario: Shrink a tab
- **WHEN** the screen area is 80×24, a tabbed column with an automatic tab height holds P1 and P2 with automatic heights of weight 1, and P1's height is shrunk
- **THEN** the column has a fixed tab height of 22 rows, and the tiles of P1 and P2 are each 22 rows high
- **AND** P1 and P2 still have automatic heights of weight 1

#### Scenario: Grow above the screen area
- **WHEN** the screen area is 80×24, a tabbed column has a fixed tab height of 23 rows, and the height of one of its windows is grown
- **THEN** the column has a fixed tab height of 24 rows

#### Scenario: Reset the tab height
- **WHEN** the screen area is 80×24, a tabbed column has a fixed tab height of 16 rows, and the height of one of its windows is reset
- **THEN** the column has an automatic tab height, and every tile of the column is 24 rows high

#### Scenario: Set rows above the limit
- **WHEN** the screen area is 80×24 and the height of a window in a tabbed column is set to 30 rows
- **THEN** the column has a fixed tab height of 24 rows

#### Scenario: Set a weight
- **WHEN** a tabbed column has a fixed tab height of 16 rows, and the weight of one of its windows is set to 2
- **THEN** the column has an automatic tab height, and the window keeps its own height

#### Scenario: Stack heights survive tab resizing
- **WHEN** the screen area is 80×24, a column holds P1 with a fixed height of 16 rows above P2 with an automatic height, tabbed display is turned on, P2's height is shrunk, and tabbed display is turned off
- **THEN** the tiles of P1 and P2 are 16 and 8 rows high

### Requirement: Shown tab
Each client's view SHALL give every tabbed column of the viewed band one shown tab:

1. the focused window, a peeked window included, when it is a tab of the column;
2. otherwise, the tab this client focused most recently, as the layout-view capability's "Focus order" defines;
3. otherwise, the column's top window.

The client SHALL draw a tabbed column's shown tab only, and no other of its tabs. Focusing the window below or above SHALL move between tabs, as the layout-view capability's "Focus within a column" moves within a stack, and so change the shown tab. Every tab of a column SHALL have the same tile, so changing the shown tab SHALL resize no window. Each client SHALL have its own shown tabs, as each holds its own focus.

#### Scenario: Focused tab shown
- **WHEN** the screen area and the client's terminal are both 80×24, the viewed band holds one tabbed column with P1, P2 and P3, and the client focuses P2
- **THEN** the client draws P2's tile and neither P1 nor P3

#### Scenario: Next tab
- **WHEN** the client of "Focused tab shown" focuses the window below
- **THEN** the client focuses and draws P3

#### Scenario: Remembered tab
- **WHEN** the viewed band holds tabbed column A with P1, P2 and P3 and column B with P4, and the client focuses P3 and then the column to the right
- **THEN** the client draws P3 in column A

#### Scenario: Never focused
- **WHEN** a client attaches to a session whose first band holds column B with P4 and then tabbed column A with P1, P2 and P3, so it focuses P4
- **THEN** the client draws P1 in column A

#### Scenario: Two clients, two tabs
- **WHEN** two clients view a band with one tabbed column holding P1 and P2, the first focuses P1 and the second focuses P2
- **THEN** the first client draws P1 and the second draws P2

#### Scenario: Switching tabs resizes nothing
- **WHEN** the screen area is 80×24, a tabbed column of width 1/2 holds two shells, and the client focuses the second and then the first
- **THEN** `tput cols` and `tput lines` print `38` and `22` in both shells, before and after

### Requirement: Tab indicator
The client SHALL draw a tab indicator on the left border column of a tabbed column's shown tab, whatever sides the tile's border draws. It SHALL take the tile's column 0 from row 1 to row `h − 2`, where `h` is the tile's drawn height, and SHALL leave rows 0 and `h − 1` as the border draws them. It SHALL take no other cell, so no window's terminal size depends on it. A tiled window that is not in a tabbed column SHALL be drawn with no indicator.

Let `r` be `h − 2` and `n` the number of tabs. The indicator SHALL hold one segment per tab, from the top in the column's window order:

- When `r` is at least `2n − 1`, one blank row SHALL separate each segment from the next. The other `r − (n − 1)` rows SHALL be shared: each segment takes that number divided by `n`, rounded down, and the first segments, one for each row left over, take one row more.
- When `r` is less than `2n − 1` and at least `n`, the segments SHALL have no row between them, and SHALL share the `r` rows by the same rule.
- When `r` is less than `n`, the first `r` tabs SHALL each take one row, and the other tabs SHALL have no segment.

The shown tab's segment SHALL be drawn with `█`, in the style of `TabIndicatorActive` when the shown tab is this client's focused window and of `TabIndicator` otherwise. Every other segment SHALL be drawn with `░` in the style of `TabIndicator`. A blank row between segments SHALL be drawn in the tile's border style. The highlights capability defines both groups.

A floating window or the error banner that covers an indicator cell SHALL stay over it, as it stays over the border. While an animation runs, `h` SHALL be the tile's drawn height. A lifted tile, as the mouse capability defines it, SHALL be drawn with no indicator.

#### Scenario: Three tabs
- **WHEN** the screen area and the client's terminal are both 80×24, the client has no bar, the viewed band holds one tabbed column of width 1/2 with P1, P2 and P3, and the client focuses P2
- **THEN** column 0 shows `╭` on row 0, `░` on rows 1 to 7, a blank cell on row 8, `█` on rows 9 to 15, a blank cell on row 16, `░` on rows 17 to 22 and `╰` on row 23
- **AND** rows 9 to 15 of column 0 are drawn in `TabIndicatorActive`'s resolved style and rows 1 to 7 and 17 to 22 in `TabIndicator`'s

#### Scenario: One tab
- **WHEN** the screen area and the client's terminal are both 80×24, the client has no bar, and the viewed band holds one tabbed column of width 1/2 with P1 focused
- **THEN** column 0 shows `█` on rows 1 to 22

#### Scenario: No room for gaps
- **WHEN** a tabbed column with P1, P2 and P3 has a fixed tab height of 6 rows, and the client focuses P1
- **THEN** the tile's column 0 shows `█` on rows 1 and 2, and `░` on rows 3 and 4

#### Scenario: Fewer rows than tabs
- **WHEN** a tabbed column with P1, P2 and P3 has a fixed tab height of 3 rows, and the client focuses P3
- **THEN** the tile's column 0 shows `░` on row 1, P1's segment, and no segment of P2 or P3

#### Scenario: Unfocused tabbed column
- **WHEN** the viewed band holds tabbed column A with P1 and P2 and column B with P3, and the client focused P2 and then P3
- **THEN** A's tile shows P2's segment with `█` in `TabIndicator`'s resolved style

#### Scenario: Left side not drawn
- **WHEN** `user/init.lua` sets `tile_border_sides` to `{ "top", "bottom" }`, the screen area and the client's terminal are both 80×24, the client has no bar, and the viewed band holds one tabbed column of width 1/2 with P1 focused
- **THEN** column 0 shows `█` on rows 1 to 22, and `tput cols` in P1 prints `38`

#### Scenario: Stacked column has no indicator
- **WHEN** a column that is not tabbed holds P1 and P2 with all four border sides drawn
- **THEN** both tiles' column 0 shows the border's left side character between their corners

### Requirement: Tab indicator as a pointer target
A cell of a tab's segment SHALL be a pointer target for that tab's window, as a border cell with no content cell, in place of the shown tab. Every other cell of the shown tab's tile, the blank rows between segments included, SHALL target the shown tab as the mouse capability defines.

A press on a segment SHALL therefore take its default, or run the binding it matches, for that tab's window. In interactive mode a left press on it SHALL focus that window, which then becomes the shown tab, and SHALL NOT be forwarded, start a selection or paste, as for any border cell.

#### Scenario: Click a tab
- **WHEN** the screen area and the client's terminal are both 80×24, the client has no bar, the viewed band holds one tabbed column of width 1/2 with P1, P2 and P3, the client focuses P1, `root` is active, and the user clicks column 0 at row 12
- **THEN** the client focuses and draws P2
- **AND** P1's and P2's programs receive no mouse input

#### Scenario: Click between segments
- **WHEN** the client of "Click a tab" clicks column 0 at row 8 with P1 focused
- **THEN** the client still focuses and draws P1

#### Scenario: Drag a tab out of its column
- **WHEN** the terminal is 80 columns wide, the camera is at 0, the viewed band holds tabbed column A of 40 cells with P1, P2 and P3 and nothing right of it, P1 is shown, navigation mode is active, and the user presses `leftmouse` on column 0 at row 20 and releases at column 70
- **THEN** the band holds tabbed column A with P1 and P2, and a column with P3 that is not tabbed, right of A
