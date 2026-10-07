# floating-windows Specification

## Purpose

Defines each band's floating layer, as niri's floating windows define it: windows placed in boxes over the band's strip. It covers floating and tiling a window, box geometry, size and move operations, exact placement, focus within the floating layer and between layers, and stacking.

## Requirements

### Requirement: Floating layer
Each band SHALL hold an ordered list of floating windows beside its columns. A floating window SHALL be a window like any other: it has a program or plugin content, an identifier that the layout capability's identity rules cover, and a terminal. A window SHALL be either in one column of its band or in its band's floating list, never both. A floating window SHALL have a box record:

- `col` and `row`: the box's top-left cell in the screen area, each an integer of at least 0.
- `width`: a proportion of the screen area's width, held in lowest terms as a column's width is, and a full-width flag.
- `rows`: the box's height in rows, at least 3.

A floating window SHALL NOT be placed on the band's strip, and SHALL NOT move with the camera.

#### Scenario: Float the only window of a band
- **WHEN** band B1 holds one column with window P1, and P1 is floated
- **THEN** B1 holds no column and one floating window, P1
- **AND** B1 is not removed, because it is not empty

#### Scenario: Window in one place only
- **WHEN** P1 is floated and then tiled again
- **THEN** after each step P1 appears exactly once in the layout

### Requirement: Box geometry
For a screen area of a given width and height, a floating window's box SHALL be placed as follows:

- Its width in cells SHALL be the area's width when full width is on, and otherwise the area's width multiplied by its proportion and rounded down. It SHALL be at least 3 cells and at most the area's width.
- Its height SHALL be its `rows`, at most the area's height.
- Its left column SHALL be its `col`, reduced so that the box ends inside the area. Its top row SHALL be its `row`, reduced in the same way.
- It SHALL have a one-cell border. The window's terminal size SHALL be its box less 2 columns and 2 rows, and at least 1 column by 1 row.

A box record SHALL keep its values when the area shrinks, so the box returns to them when the area grows again.

#### Scenario: Box in an 80×24 area
- **WHEN** the area is 80×24 and a floating window has `col` 10, `row` 4, width 1/2 and `rows` 12
- **THEN** its box spans columns 10 to 49 and rows 4 to 15
- **AND** its terminal size is 38×10

#### Scenario: Box pushed back inside a smaller area
- **WHEN** a floating window has `col` 50, `row` 4, width 1/2 and `rows` 12, and the area is 80×24
- **THEN** its box spans columns 40 to 79

#### Scenario: Box taller than the area
- **WHEN** a floating window has `row` 5 and `rows` 30, and the area is 80×24
- **THEN** its box spans rows 0 to 23

#### Scenario: Area grows back
- **WHEN** a floating window has `col` 50 and width 1/2, the area shrinks from 120×40 to 80×24, and then grows back to 120×40
- **THEN** the box starts at column 40 in the small area and at column 50 again in the large area

### Requirement: Float a window
Floating a tiled window SHALL remove it from its column, as closing removes a window from a column, and append it to its band's floating list. The column rule and the dynamic band rule SHALL then apply. A tiled window SHALL float when floating is toggled on it, and when a request to float it arrives. A request to float SHALL name the floating layer, as the session-server capability's "Session actions" defines. A request to float a window that already floats SHALL change nothing. That window SHALL keep its place in the floating list and its box record, and the request SHALL produce no layout event.

When the window has been floating before, it SHALL take the box record it had when it was last tiled. Otherwise it SHALL take a new box, whose record SHALL be:
- `width` and full width: the configuration's `default_column_width`, and full width off.
- `rows`: the screen area's height less twice its tenth, and at least 3, so that the box is two tenths shorter than the area. The tenth SHALL be the area's height divided by 10, rounded to the nearest whole number with halves rounded up, and at least 1, whatever step a grow or shrink request names.
- `col` and `row`: the box centred in the current screen area, each half the room left beside the box, rounded down, so that one tenth stays free above the box and one below.

#### Scenario: First float opens a new box
- **WHEN** the screen area is 80×24, the default column width is 1/2, window P2 is alone in a column, and P2 is floated
- **THEN** P2's box record has width 1/2, full width off, `rows` 20, `col` 20 and `row` 2
- **AND** its box spans columns 20 to 59 and rows 2 to 21

#### Scenario: New box takes the default width
- **WHEN** the screen area is 80×24, the default column width is 1/3, and a window in a full-width column is floated for the first time
- **THEN** its box record has width 1/3, full width off, `col` 27 and `row` 2

#### Scenario: Float from a stack
- **WHEN** the screen area is 80×24, the default column width is 1/2, a column of width 1/3 holds P1 and P2, both with automatic heights of weight 1, and P2 is floated
- **THEN** the column holds P1 alone, with weight 1
- **AND** P2's box record has width 1/2, `rows` 20, `col` 20 and `row` 2

#### Scenario: New box in a tall area
- **WHEN** the screen area is 120×67, the default column width is 1/2, and a window is floated for the first time
- **THEN** its box record has `rows` 53, `col` 30 and `row` 7

#### Scenario: Float again returns to the last box
- **WHEN** a floating window with `col` 5, `row` 3, width 1/3 and `rows` 10 is tiled and then floated again
- **THEN** its box record has `col` 5, `row` 3, width 1/3 and `rows` 10

#### Scenario: Request to float a tiled window
- **WHEN** the screen area is 80×24, the default column width is 1/2, window 3 is alone in a column, and a request to float window 3 arrives
- **THEN** window 3's box record has width 1/2, full width off, `rows` 20, `col` 20 and `row` 2

#### Scenario: Request to float a floating window
- **WHEN** a band's floating list holds window 3 and then window 4, window 3's box record has `col` 5, `row` 3, width 1/3 and `rows` 10, and a request to float window 3 arrives
- **THEN** the layout is unchanged, and the floating list still holds window 3 and then window 4
- **AND** no layout event is produced

#### Scenario: Two requests to float one window
- **WHEN** the screen area is 80×24, the default column width is 1/2, window 3 is alone in a column of band 1, and two requests to float window 3 arrive one after the other
- **THEN** band 1's floating list holds window 3 once, with width 1/2, `rows` 20, `col` 20 and `row` 2
- **AND** no column of band 1 holds window 3

### Requirement: Tile a window
Tiling a floating window SHALL remove it from the floating list and place it alone in a new column of its band. A floating window SHALL be tiled when floating is toggled on it, and when a request to tile it arrives. A request to tile SHALL name the tiled layer, as the session-server capability's "Session actions" defines. A request to tile a window that is already tiled SHALL change nothing, and SHALL produce no layout event. The new column SHALL take the window's box width and full-width flag, and the window SHALL take an automatic height of weight 1. The toggle or the request SHALL name optionally a tiled window of the same band. The new column SHALL be inserted right of that window's column, or as the band's first column when the toggle or the request names none, or names a window that is not a tiled window of that band. The window's box record SHALL be kept for the next time it floats.

#### Scenario: Tile right of the named window
- **WHEN** a band holds columns A and B and floating window P3 of width 1/3, and P3 is tiled naming a window of A
- **THEN** the band holds A, a column of width 1/3 with P3, and B, in that order, and no floating window

#### Scenario: Tile with no window named
- **WHEN** a band holds column A and floating window P3, and P3 is tiled naming no window
- **THEN** the band holds the column with P3, then A

#### Scenario: Request to tile a floating window
- **WHEN** band 1 holds the columns of windows 1 and 2 and floating window 3 of width 1/3, and a request to tile window 3 naming window 1 arrives
- **THEN** band 1 holds the column of window 1, a column of width 1/3 with window 3, and the column of window 2, in that order, and no floating window

#### Scenario: Request to tile a tiled window
- **WHEN** band 1 holds the columns of windows 1 and 2, and a request to tile window 2 naming window 1 arrives
- **THEN** the layout is unchanged
- **AND** no layout event is produced

### Requirement: Size a floating window
The width actions SHALL change a floating window's box width and full-width flag exactly as the layout capability's "Column widths" changes a column's: cycling through the presets, toggling full width, growing or shrinking by the step the request names, and setting a width.

The height actions SHALL change a floating window's `rows`. Let `h` be its box height as placed in the session's current screen area, and the step in rows the area's height multiplied by the step the request names, rounded to the nearest whole number with halves rounded up, and at least 1:
- Growing SHALL set `rows` to `h` plus the step in rows, and shrinking to `h` less the step in rows.
- Resetting SHALL set `rows` to the height of a new box, as "Float a window" defines.
- Setting a number of rows SHALL set `rows` to that number.
- Setting a weight SHALL leave the window unchanged.

The new `rows` SHALL be kept between 3 and the area's height. When the result equals the current record, nothing SHALL change. Sizing SHALL keep `col` and `row`.

#### Scenario: Cycle a floating width
- **WHEN** a floating window has width 1/2 and its width is cycled with the default presets
- **THEN** its width is 2/3

#### Scenario: Grow a floating height
- **WHEN** the area is 80×24 and a floating window has `rows` 12 and is grown with the step 1/10
- **THEN** its `rows` is 14

#### Scenario: Height limit
- **WHEN** the area is 80×24 and a floating window has `rows` 24 and is grown
- **THEN** the layout is unchanged

#### Scenario: Reset a floating height
- **WHEN** the area is 80×25 and a floating window's height is reset
- **THEN** its `rows` is 19

#### Scenario: Consume or expel a floating window
- **WHEN** a floating window is consumed or expelled to the left
- **THEN** the layout is unchanged

#### Scenario: Grow a floating width by another step
- **WHEN** a floating window has width 1/2 and is grown with the step 1/5
- **THEN** its width is 7/10

### Requirement: Move a floating window
Moving a floating window left, right, up or down SHALL move its box one step in that direction. Let `x` and `y` be the box's left column and top row as placed in the session's current screen area. A horizontal step SHALL be the area's width divided by 10, and a vertical step its height divided by 10, each rounded to the nearest whole number with halves rounded up, and at least 1. The new `col` SHALL be `x` plus or less the horizontal step, kept between 0 and the area's width less the box's width. The new `row` SHALL be found the same way. The other coordinate SHALL be set to its placed value. When the result equals the current record, nothing SHALL change.

Placing a floating window at a given column and row SHALL set `col` and `row` to them, each kept between 0 and the area's size less the box's size in that dimension.

#### Scenario: Move right
- **WHEN** the area is 80×24 and a floating window's box starts at column 20 and is moved right
- **THEN** its `col` is 28

#### Scenario: Move against the edge
- **WHEN** the area is 80×24 and a floating window's box is 40 cells wide, starts at column 36 and is moved right
- **THEN** its `col` is 40
- **AND** when it is moved right again, the layout is unchanged

#### Scenario: Move down
- **WHEN** the area is 80×25 and a floating window's box starts at row 2 and is moved down
- **THEN** its `row` is 5

#### Scenario: Place exactly
- **WHEN** the area is 80×24, a floating window's box is 40×12, and it is placed at column 70 and row 3
- **THEN** its `col` is 40 and its `row` is 3

### Requirement: Layers of a view
Each client's view SHALL hold, for each band, an active layer, either tiled or floating, and the window it last focused in each layer. A view SHALL enter a band it has not viewed before in the tiled layer. While the floating layer of the viewed band is active, the focused window SHALL be one of its floating windows. While the tiled layer is active, the focused window SHALL be a tiled window, or none when the band holds no window at all.

Focusing a floating window by any means SHALL make the floating layer active. Focusing a tiled window by any means SHALL make the tiled layer active. Whenever the active layer of the viewed band holds no window and the other layer holds one, the other layer SHALL become active, with focus as "Switch layers" sends it.

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

### Requirement: Switch layers
Switching focus between floating and tiled windows SHALL make the other layer of the viewed band active. In the floating layer, focus SHALL go to the floating window this client focused most recently in that band, or to the top floating window of this client's stacking order when it focused none. In the tiled layer, focus SHALL go to the tiled window this client focused most recently in that band, when it is still tiled there, and otherwise to the top window of the band's first column. When the other layer holds no window, the view SHALL NOT change.

#### Scenario: Into the floating layer
- **WHEN** a band holds column A with P1 and floating windows P3 and P4, a client focuses P1, focused P4 earlier, and switches layers
- **THEN** the client focuses P4

#### Scenario: Back to the tiled layer
- **WHEN** the client then switches layers again
- **THEN** the client focuses P1

#### Scenario: No floating window
- **WHEN** the viewed band holds no floating window and the client switches layers
- **THEN** the view is unchanged

### Requirement: Focus between floating windows
While the floating layer is active, focusing the column to the left or right, or the window below or above, SHALL focus the floating window of the viewed band nearest in that direction. Each box's centre is its left column plus half its width, and its top row plus half its height, in the placed box. A candidate SHALL be a floating window whose centre lies strictly in that direction from the focused window's centre. The nearest SHALL be the candidate with the smallest distance between centres along the direction's axis. A tie SHALL go to the smallest distance along the other axis, then to the candidate earlier in the floating list. When there is no candidate, the view SHALL NOT change.

#### Scenario: Focus right
- **WHEN** the floating layer is active on P3, whose box spans columns 0 to 19, and floating windows P4 at columns 30 to 49 and P5 at columns 60 to 79 sit on the same rows
- **THEN** focusing the column to the right focuses P4

#### Scenario: No floating window in that direction
- **WHEN** the floating layer is active on the leftmost floating window and the client focuses the column to the left
- **THEN** the view is unchanged

### Requirement: Focus follows a window between layers
When a window this client focuses moves to the other layer of its band, by this client or by another, focus SHALL stay on that window, and the layer SHALL follow it. When the focused floating window leaves the layout, focus SHALL go to the floating window this client focused most recently in that band, or to the last window of the floating list when it focused none. When no floating window remains, the tiled layer SHALL become active, with focus as "Switch layers" sends it.

#### Scenario: Float the focused window
- **WHEN** a client focuses P2 in the tiled layer and P2 is floated
- **THEN** the client focuses P2 in the floating layer

#### Scenario: Tile the focused window
- **WHEN** a client focuses floating window P3 and P3 is tiled
- **THEN** the client focuses P3 in the tiled layer, in its new column

#### Scenario: Last floating window closes
- **WHEN** a client focuses floating window P3, the only floating window of a band holding column A with P1, and P3's program exits
- **THEN** the client focuses P1 in the tiled layer

### Requirement: Stacking
Each client SHALL draw the floating windows of the viewed band in its own stacking order, from bottom to top. The floating windows this client has never focused SHALL come first, in the order of the floating list. The floating windows it has focused SHALL follow, in the order it last focused them, the most recently focused on top. Stacking SHALL NOT change the layout or another client's drawing.

#### Scenario: Focus raises
- **WHEN** a band's floating list holds P3 and then P4, both boxes overlap, and a client focuses P4 and then P3
- **THEN** that client draws P3 over P4

#### Scenario: Never focused
- **WHEN** a band's floating list holds P3 and then P4, and a client has focused neither
- **THEN** that client draws P4 over P3

#### Scenario: Other clients keep their order
- **WHEN** two clients view the same band, the first focuses P3 and the second focuses P4
- **THEN** the first draws P3 on top and the second draws P4 on top
