## MODIFIED Requirements

### Requirement: Window resizes
The server SHALL send every change of the screen area or the layout to the attached clients at once, as "State updates" defines, without waiting for the PTYs.

A window SHALL be shown when at least one client attached to its session reports it among its shown windows, as the wire-protocol capability defines. A client that detaches or disconnects SHALL show no window. A window SHALL be unshown until a client reports it.

A window's terminal size SHALL be its tile's, as the layout capability gives it, when the window is tiled, and its box's, as the floating-windows capability gives it, when the window floats.

The server SHALL resize PTYs only when the session has settled: 100 ms have passed since the last change of the screen area, the layout or the set of shown windows. It SHALL then resize every shown window, and every tab of a tabbed column one of whose tabs is shown, as the tabbed-columns capability defines tabs, whose PTY size differs from its terminal size, once, to that terminal size. Any other window SHALL keep its PTY size, whatever the screen area and the layout give it. A new window's PTY SHALL start at its terminal size.

#### Scenario: Burst of terminal resizes
- **WHEN** a client attached to an 80×24 session showing one window in a column of width 1/2 reports resizes to 90×25, 95×28 and 100×30, each less than 100 ms after the last
- **THEN** the window's program receives exactly one SIGWINCH
- **AND** its PTY becomes 48 columns by 28 rows

#### Scenario: Held resize key
- **WHEN** a client showing one window in a column of width 1/2 asks to grow that column's width four times, each less than 100 ms after the last
- **THEN** every attached client receives a layout after each action
- **AND** the window's program receives exactly one SIGWINCH, after the last action

#### Scenario: Offscreen window keeps its size
- **WHEN** a client with an 80×24 terminal views a band holding columns A, B and C, all of width 1/2, shows A and B, and resizes its terminal to 100×30
- **THEN** the PTYs of A and B become 48 columns by 28 rows
- **AND** C's PTY stays 38 columns by 22 rows, and C's program receives no SIGWINCH

#### Scenario: Offscreen window resized when shown
- **WHEN** C's PTY has kept 38 columns by 22 rows while offscreen in a 100×30 area, and the client focuses C so that it shows C
- **THEN** C's PTY becomes 48 columns by 28 rows

#### Scenario: Shown by another client
- **WHEN** two clients are attached, the first shows only window A, the second shows only window C, and the screen area changes
- **THEN** both A and C are resized

#### Scenario: Window leaves a stack while detached
- **WHEN** no client is attached, a column holds P1 and P2, and P2's program exits
- **THEN** P1's PTY keeps its size
- **AND** when a client attaches and shows P1, P1's PTY becomes the size of its tile, which now spans the column's height

#### Scenario: Hidden tabs sized with the shown tab
- **WHEN** a client with an 80×24 terminal views a band holding one tabbed column of width 1/2 with P1 and P2, shows only P2, and resizes its terminal to 100×30
- **THEN** the PTYs of P1 and P2 both become 48 columns by 28 rows

#### Scenario: Tabs sized when tabs turn on
- **WHEN** the screen area is 80×24, a client shows a column of width 1/2 holding P1 and P2 with automatic heights of weight 1, and tabbed display is toggled naming P2 so that the client shows only P2
- **THEN** the PTYs of P1 and P2 both become 38 columns by 22 rows

#### Scenario: Floating window takes its box size
- **WHEN** the screen area is 80×24, a shown floating window has width 1/3 and `rows` 12, and its height is grown
- **THEN** once the session settles, its PTY becomes 24 columns by 12 rows
