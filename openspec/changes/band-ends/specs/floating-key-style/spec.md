## MODIFIED Requirements

### Requirement: Windows float
A window SHALL run a program when its table in `gband.layout()` holds `name`. The desktop plugin SHALL float a tiled window that runs a program by dispatching `toggle_window_floating` with the target `{ window = <number>, floating = true }`, which floats a tiled window and leaves a floating window as it is, as the lua-control capability's "Action targets" defines. It SHALL dispatch it in these cases:

- When `WindowOpened` runs, for the opened window, when the client's layout holds it tiled, the layout does not mark it with `plugin_window`, and no tiled plugin window that this client opened still awaits its window, as `gband.win.info` gives it with `window` nil.
- When `Attached` runs, for every tiled window of the layout that runs a program, in every band, in layout order.
- When `ConfigReloaded` runs, for every tiled window of the layout that runs a program, in every band, in layout order.

A window's `name` reaches the client after the `WindowOpened` that reports the window, so the first case cannot read it. It SHALL therefore also float the drawn window of a tiled plugin window that another client opened, which runs no program and shows no buttons. The `Attached` and `ConfigReloaded` sweeps SHALL NOT float a tiled window that runs no program, and no case SHALL float the drawn window of a tiled plugin window that this client opened. It SHALL NOT float a window in any other case. A window that a binding or another client tiles after it floated SHALL therefore stay tiled until the next attach or reload of a client that uses the floating style. The window list's `New window` and the floating preset's `n` SHALL open their window floating.

Every client that uses the floating style SHALL send these requests. The server floats each window once, whichever request arrives first, as the floating-windows capability's "Float a window" defines. Floating a window changes the shared layout, so every client attached to the session SHALL see it float, whatever key style that client uses. A window that a client with another key style opens SHALL float too, while a client with the floating style is attached.

The plugin SHALL place each window that it saw open with `WindowOpened`, or that its `Attached` or `ConfigReloaded` sweep named, once, in the first layout in which that window floats. Let `col` and `row` be the window's box's top-left cell, as `gband.layout()` gives it, `w` its width in cells and `r` its `rows`. When another floating window of its band has its top-left cell at `col` and `row`, the plugin SHALL dispatch `gband.window.set_position` with the first position `col + 2k`, `row + k`, for `k` from 1 up, that no other floating window of the band has as its top-left cell and that keeps the box inside the screen area, `col + 2k + w` at most the screen area's `cols` and `row + k + r` at most its `rows`. When no such position exists, the window SHALL keep its box. The plugin SHALL place the windows of one layout in the order of their band's floating list, and SHALL count a position it has just dispatched as taken. A window whose top-left cell no other floating window shares SHALL keep its box.

#### Scenario: First window of a new session
- **WHEN** no `user/init.lua` exists, `user/keystyle.lua` holds `return "floating"`, and a client with an 80×24 terminal attaches to a new session
- **THEN** the session's first window floats with the box record width 1/2, full width off, `rows` 20, `col` 20 and `row` 2
- **AND** the band holds no column

#### Scenario: Spawned window floats
- **WHEN** the floating style is in use and a binding function calls `gband.spawn({ cmd = "sh" })`
- **THEN** the new window floats and is focused

#### Scenario: Windows float after switching the style
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds two tiled windows, the settings window is open on its fourth line, and the user presses `h`
- **THEN** `user/keystyle.lua` holds `return "floating"`
- **AND** after the reload both windows float and the band holds no column

#### Scenario: Drawn window stays tiled
- **WHEN** the floating style is in use and a binding function opens a tiled plugin window with `gband.win.open({ kind = "tiled", lines = { "hi" } })`
- **THEN** its drawn window stays in a column of the viewed band

#### Scenario: Another client's tiled plugin window
- **WHEN** a first client uses the floating style, and a second client attached to the same session uses the modal style and opens a tiled plugin window
- **THEN** the plugin window's drawn window floats, and its top border shows no button

#### Scenario: Window of a client with another style
- **WHEN** a first client uses the floating style, and a second client attached to the same session uses the modal style and presses Ctrl+Space then `n`
- **THEN** the new window floats in both clients' layouts

#### Scenario: Two floating clients float a window once
- **WHEN** two clients attached to the same session use the floating style, and a binding function of the first calls `gband.spawn({ cmd = "sh" })`
- **THEN** the new window floats in both clients' layouts, and its band's floating list holds it once

#### Scenario: A tiled window stays tiled
- **WHEN** the floating style is in use, window 1 floats, and a binding function calls `gband.action.toggle_window_floating({ window = 1 })`
- **THEN** window 1 is tiled, and it is still tiled after the next layout the client receives

#### Scenario: New windows cascade
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")`, the client's 80×24 terminal sets the screen area, the only window floats with a 40×20 box at column 20 and row 2, and the user picks `New window` in the window list twice
- **THEN** the first new window's box is at column 22 and row 3, and the second's at column 24 and row 4

#### Scenario: Cascade stops at the edge
- **WHEN** three floating windows of the band have 40×20 boxes at column 20 and row 2, column 22 and row 3, and column 24 and row 4, and the user picks `New window`
- **THEN** the new window keeps its box at column 20 and row 2
