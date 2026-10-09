# floating-key-style Specification

## Purpose
Defines the floating key style, a desktop like MS Windows built in Lua: the bundled `gband.desktop` plugin and the floating preset's bindings, the rules that float windows, the title bar buttons, the presses on a window's border, maximize, restore and tile, the window list, the window menu, the leader and the desktop highlight groups.

## Requirements

### Requirement: Desktop plugin
gband SHALL bundle the client plugin module `gband.desktop`, whose plugin name is `desktop`. Its `setup` SHALL take no options. An options table that holds any field SHALL make `setup` raise an error that names the field. `setup` SHALL register these actions, with these descriptions:

| action | description |
|---|---|
| `desktop.list` | `list the windows` |
| `desktop.leader` | `open the window list, or send the prefix key from it` |
| `desktop.press` | `move, resize or press a button of a floating window` |
| `desktop.menu` | `open the menu for the cell under the pointer` |

`setup` SHALL register the decorations function of "Title bar buttons" with `gband.core.provide("decorations", fn)`. It SHALL register the event handlers that "Windows float", "Title bar buttons", "Maximize, restore and tile" and "Desktop menus" need. The plugin SHALL use only the documented API.

Dispatching `desktop.list` SHALL open the window list, centred in the ribbon area, as "Window list" defines. When the window list is open, it SHALL focus the window list and open no second one. `desktop.leader` SHALL act as "Leader" defines. `desktop.press` and `desktop.menu` SHALL act as "Left press on a border" and "Right press" define when a mouse binding runs them. Run without a mouse payload, as from a key or from `gband.keymap.run`, they SHALL do nothing.

#### Scenario: Actions registered
- **WHEN** a configuration calls `gband.plugin("gband.desktop")` and reads `gband.action.list()`
- **THEN** it holds `desktop.list`, `desktop.leader`, `desktop.press` and `desktop.menu`, with the descriptions of the table

#### Scenario: Unknown option
- **WHEN** a configuration calls `gband.plugin("gband.desktop", { buttons = false })`
- **THEN** the plugin `desktop` reports an error that names `buttons`

#### Scenario: Press action from a key
- **WHEN** `user/init.lua` calls `gband.keystyle.use("floating")` and binds `prefix x` to `gband.action["desktop.press"]`, and a binding function calls `gband.keymap.run("prefix", "x")`
- **THEN** no error is reported and the layout does not change

### Requirement: Floating bindings
The floating preset, `gband.keystyle.floating`, SHALL first set up, with `gband.plugin` and no options, the key list plugin `gband.keylist`, then the Lua prompt plugin `gband.prompt`, then the desktop plugin `gband.desktop`. It SHALL declare no mode and set no option. It SHALL then make these bindings in `prefix`, in this order. A row with a function binds a Lua function with the row's description. Every other row binds the action it names, with that action's description:

| key after the prefix | Lua binding | binds | description |
|---|---|---|---|
| `n` | `prefix n` | a function that opens a window running the user's shell, floating, in the viewed band, as `gband.action.open_window({ floating = true })` does | `open a floating window` |
| `?` | `prefix ?` | `keylist.open` | `list the keys` |
| `:` | `prefix :` | `prompt.open` | `run Lua` |
| `N` | `prefix N` | `prompt.rename` | `rename the window` |
| `s` | `prefix s` | the function `gband.settings.open` | `settings` |
| `!` | `prefix !` | `reload` | `reload the configuration` |
| `D` | `prefix D` | `detach` | `detach` |
| Ctrl+Space | `prefix prefix` | `send_prefix` | `send the prefix key to the focused window` |

It SHALL then make these bindings in `root`, in this order, each to the action it names, with that action's description:

| mouse name | action |
|---|---|
| `mod+leftmouse` | `drag_window` |
| `mod+rightmouse` | `drag_resize_window` |
| `mod+middlemouse` | `drag_band` |
| `mod+wheeldown` | `focus_band_down` |
| `mod+wheelup` | `focus_band_up` |
| `leftmouse` | `desktop.press` |
| `rightmouse` | `desktop.menu` |

It SHALL bind no key in `root` and no mouse name in `prefix`. Last, it SHALL register the handler of `KeyTableChanged` that "Leader" defines.

#### Scenario: Prefix bindings in order
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")` and reads `gband.keymap.list("prefix")`
- **THEN** the keys are `n`, `?`, `:`, `N`, `s`, `!`, `D` and `prefix`, in that order
- **AND** the entry for `n` has no action and the description `open a floating window`

#### Scenario: Root bindings in order
- **WHEN** the floating style is in use and `gband.keymap.list("root")` is read
- **THEN** it holds `mod+leftmouse`, `mod+rightmouse`, `mod+middlemouse`, `mod+wheeldown`, `mod+wheelup`, `leftmouse` and `rightmouse`, in that order, and nothing else
- **AND** the entry for `leftmouse` has the action `desktop.press` and the entry for `rightmouse` the action `desktop.menu`

#### Scenario: Plugins set up by the preset
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")` and reads `gband.action.list()`
- **THEN** it holds `keylist.open`, `prompt.open` and `desktop.list`, and no `errors.open`
- **AND** no sidebar is drawn

#### Scenario: Alt drag still moves a window
- **WHEN** the floating style is in use, the screen area is 80×24, a floating window's box is 40×12 at column 20 and row 4, and the user drags from its content cell at column 30 and row 8 to column 36 and row 8 with the left button and Alt held
- **THEN** the box is at column 26 and row 4

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
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds two tiled windows, the settings window is open on its third line, and the user presses `h`
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

### Requirement: Title bar buttons
The desktop plugin's decorations function SHALL return, for a floating window that runs a program, three spans: `[_]`, the minimize button; `[□]`, the maximize button, or `[❐]` while the window counts as maximized, as "Maximize, restore and tile" defines; and `[X]`, the close button. It SHALL return nil for every other window. The first two spans SHALL take the style `require("gband.hl").drawn("DesktopButton")` returns, and `[X]` the style it returns for `DesktopClose`.

The client places the spans as the window-decorations capability defines. Each span is 3 cells wide, so in a box of width `w` of at least 13 cells, `[_]` SHALL take the box's columns `w − 11` to `w − 9`, the maximize button `w − 8` to `w − 6`, and `[X]` `w − 5` to `w − 3`. A box narrower than 13 cells SHALL show no button.

A left press on a button and the release that follows act as "Left press on a border" defines. A button SHALL act on the window whose box shows it:

- `[_]` SHALL minimize the window, as `gband.window.minimize` does.
- `[□]` SHALL maximize the window and `[❐]` SHALL restore it, as "Maximize, restore and tile" defines. Both SHALL then focus the window.
- `[X]` SHALL close the window, as `close_window` with a target that names the window does.

#### Scenario: Buttons on a floating window
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")`, the client's 80×24 terminal sets the screen area, and window 1 floats with a 40×12 box at column 20 and row 4
- **THEN** row 4 shows `[_][□][X]` from column 49 to column 57, `─` on column 58 and `╮` on column 59

#### Scenario: Restore glyph while maximized
- **WHEN** that window is maximized
- **THEN** row 0 shows `[_][❐][X]` from column 69 to column 77

#### Scenario: No buttons on a tiled window
- **WHEN** the floating style is in use and a tiled plugin window's drawn window is drawn in a tile
- **THEN** the tile's top row shows no span

#### Scenario: Narrow box
- **WHEN** the floating style is in use and a floating window's box is 12 columns wide
- **THEN** its top row shows no button

#### Scenario: Close button style
- **WHEN** the active colorscheme is `default`, the floating style is in use, and a floating window is drawn
- **THEN** each cell of its `[X]` has the style of the cells of its `[_]`: the window's border style with bold on
- **AND** no cell of its `[X]` has the foreground palette index 1

### Requirement: Left press on a border
The floating preset binds `leftmouse` in `root` to `desktop.press`, as "Floating bindings" defines. `desktop.press` SHALL take a press only when the press's `target` is `"window"`, its `window` floats and runs a program, and the pressed cell is a border cell, with `content_col` nil. It SHALL decline every other press by returning `false`, as the mouse capability's "Mouse names in key tables" defines. A declined press SHALL take the mouse capability's interactive mode defaults: a press on content focuses the window and is forwarded to its program or starts a selection, and a press on a tiled window, a plugin window, empty ribbon or a bar takes its own default.

For a press that it takes, let `x` and `y` be the press's `box_col` and `box_row`, and `w` and `h` its `box_width` and `box_height`. The first of these rules that matches SHALL apply:

1. **Button**: `y` is 0 and `x` lies on a button, as "Title bar buttons" places them for `w`. The press SHALL do nothing else, and SHALL start no gesture. When the left button is then released on a cell of the same window whose `box_row` is 0 and whose `box_col` lies on the same button, the button SHALL act. A release on any other cell SHALL do nothing.
2. **Corner**: the cell lies in a corner zone. The zones SHALL be checked in this order, and each SHALL resize the two edges that meet at its corner, with `drag_resize_window` and the target `{ edges = ... }`:
   - top-left: the cells (0, 0), (1, 0) and (0, 1), with the edges `left` and `top`;
   - top-right: the cells (`w − 1`, 0), (`w − 2`, 0) and (`w − 1`, 1), with `right` and `top`;
   - bottom-left: the cells (0, `h − 1`), (1, `h − 1`) and (0, `h − 2`), with `left` and `bottom`;
   - bottom-right: the cells (`w − 1`, `h − 1`), (`w − 2`, `h − 1`) and (`w − 1`, `h − 2`), with `right` and `bottom`.
3. **Title bar**: `y` is 0. The press SHALL move the window with `drag_window`.
4. **Side**: `x` is 0, `x` is `w − 1`, or `y` is `h − 1`. The press SHALL resize the one edge `left`, `right` or `bottom`, in that order of checks, with `drag_resize_window` and the target `{ edges = { <edge> } }`.

#### Scenario: Close by the button
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")`, the client's 80×24 terminal sets the screen area, window 1 floats with a 40×12 box at column 20 and row 4, and the user presses and releases the left button at column 56 and row 4
- **THEN** window 1 leaves the layout

#### Scenario: Release off the button
- **WHEN** the user presses the left button at column 56 and row 4 of that box, moves to column 40 and row 10, and releases
- **THEN** window 1 stays open and its box does not change

#### Scenario: Minimize by the button
- **WHEN** the user presses and releases the left button at column 50 and row 4 of that box
- **THEN** the client does not draw window 1 and the layout does not change

#### Scenario: Maximize by the button
- **WHEN** the user presses and releases the left button at column 53 and row 4 of that box
- **THEN** window 1's box is 80×24 at column 0 and row 0, and window 1 is focused

#### Scenario: Resize the top-left corner
- **WHEN** the user presses the left button at column 21 and row 4 of that box and drags to column 16 and row 2
- **THEN** window 1's box is 45×14 at column 15 and row 2

#### Scenario: Cell beside the bottom-right corner
- **WHEN** the user presses the left button at column 58 and row 15 of that box and drags 4 columns right and 3 rows down
- **THEN** window 1's box is 44×15 at column 20 and row 4

#### Scenario: Bottom edge near a corner
- **WHEN** the user presses the left button at column 22 and row 15 of that box and drags 4 columns left and 3 rows down
- **THEN** window 1's box is 40×15 at column 20 and row 4

#### Scenario: Left edge
- **WHEN** the user presses the left button at column 20 and row 10 of that box and drags 5 columns left
- **THEN** window 1's box is 45×12 at column 15 and row 4

#### Scenario: Move by the title bar
- **WHEN** the user presses the left button at column 30 and row 4 of that box and drags to column 35 and row 6
- **THEN** window 1's box is 40×12 at column 25 and row 6, and window 1 is focused

#### Scenario: Content press reaches the program
- **WHEN** window 1's program enabled mode 1000 with SGR encoding, and the user clicks at column 25 and row 7, its content cell at column 4 and row 2
- **THEN** window 1 is focused and its program receives `\x1b[<0;5;3M` and then, on release, `\x1b[<0;5;3m`
- **AND** the box does not change

#### Scenario: Content press selects
- **WHEN** window 1 runs a shell that reports no mouse, and the user drags across its content from column 22 to column 30 on row 6
- **THEN** window 1 is focused, the dragged text is selected and the box does not change

### Requirement: Right press
The floating preset binds `rightmouse` in `root` to `desktop.menu`, as "Floating bindings" defines. `desktop.menu` SHALL take these presses:

- A press whose `target` is `"window"`, whose `window` floats and runs a program, and whose cell is a border cell SHALL open the window menu for that window, as "Window menu" defines.
- A press whose `target` is `"ribbon"` SHALL open the window list, as "Window list" defines.

It SHALL decline every other press by returning `false`, so a right press on content pastes the copy buffer and a press on a tiled window, a plugin window or a bar takes its own default. Opening a menu from a right press SHALL NOT change the focused window.

A menu opened by a right press SHALL have its top-left cell at the pointer's cell in the ribbon area: the press's column less the columns that the shown bars on the left side take, as `gband.bar.list()` gives them, and the press's row. The box SHALL then be cut so that it ends inside the ribbon area, as the plugin-windows capability places a floating plugin window at a given row and column.

#### Scenario: Window menu at the pointer
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")`, the client's 80×24 terminal sets the screen area, window 1 is named `notes` and floats with a 40×12 box at column 20 and row 4, and the user presses the right button at column 30 and row 4
- **THEN** a focused floating plugin window 16 columns wide and 7 rows high opens at column 30 and row 4
- **AND** the focused window does not change

#### Scenario: Window list at the pointer, cut to the ribbon area
- **WHEN** the floating style is in use with no bar, the client's terminal is 80×24, the only window floats with a box that does not cover column 70 and row 20, and the user presses the right button at column 70 and row 20
- **THEN** the window list opens 24 columns wide and 7 rows high at column 56 and row 17

#### Scenario: Right press in content pastes
- **WHEN** the floating style is in use, the copy buffer holds `ls`, and the user right-clicks the content of a floating shell window whose program reports no mouse
- **THEN** the window is focused and receives `ls` as a paste, and no menu opens

#### Scenario: Right press on the sidebar
- **WHEN** no `user/init.lua` exists, `user/keystyle.lua` holds `return "floating"`, and the user presses the right button on column 0 and row 5
- **THEN** no floating plugin window opens

### Requirement: Maximize, restore and tile
Let `cols` and `rows` be the size of the screen area, as `gband.layout()` gives it. The desktop plugin SHALL give a floating window these boxes, with `gband.window.set_width`, `gband.window.set_height` with `rows`, and `gband.window.set_position`, dispatched in that order:

| state | width | `rows` | `col` | `row` |
|---|---|---|---|---|
| maximized | 1 | `rows` | 0 | 0 |
| tiled left | 1/2 | `rows` | 0 | 0 |
| tiled right | 1/2 | `rows` | `cols` less `cols / 2` rounded down | 0 |

A floating window SHALL count as in a state while its table in `gband.layout()` has the `col` and `row` of that state's row, `rows` at least the screen area's `rows`, and the state's width with `full_width` off. A window whose table has `full_width` on, `col` 0, `row` 0 and `rows` at least the screen area's `rows` SHALL also count as maximized. The state SHALL come from the box alone, so it survives a reload and a change made by another client.

When the plugin maximizes or tiles a window that counts as in none of these states, it SHALL first remember the window's box: `col`, `row`, `width`, `full_width` and `rows`, as `gband.layout()` gives them. It SHALL keep at most one remembered box per window, in this client only. It SHALL forget it when the window leaves the layout, when it restores the window, and when the configuration reloads.

Restoring a window SHALL give it its remembered box: the width with `gband.window.set_width`, full width with `toggle_full_width` when the remembered box had it on, `rows` with `gband.window.set_height`, then `col` and `row` with `gband.window.set_position`. Without a remembered box, restoring SHALL give the window the width 1/2, `rows` half the screen area's rows rounded down and at least 3, and the box centred in the screen area, each offset rounded down.

The plugin SHALL keep the screen area's size of the last layout it saw, from `gband.layout()` when the configuration loads and at each `LayoutChanged`. When `LayoutChanged` runs and the layout's `cols` or `rows` differ from that size, each floating window that counted as maximized, tiled left or tiled right against the previous size SHALL get the box of that state for the new size. This SHALL hold whatever changed the screen area: this client's terminal, another client's terminal, or another client attaching. Every client that uses the floating style sends the same boxes, so their requests agree.

#### Scenario: Maximize and restore
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")`, the client's 80×24 terminal sets the screen area, window 1 floats with a 40×12 box at column 20 and row 4, and the user maximizes it and then restores it
- **THEN** after maximizing, its box is 80×24 at column 0 and row 0
- **AND** after restoring, its box is 40×12 at column 20 and row 4

#### Scenario: Tile left
- **WHEN** the user picks `Tile left` in window 1's window menu
- **THEN** window 1's box is 40×24 at column 0 and row 0, and window 1 is focused

#### Scenario: Tile right in an odd width
- **WHEN** no `user/init.lua` exists, `user/keystyle.lua` holds `return "floating"`, the client's terminal is 80×24, so the screen area is 79×24, and the user picks `Tile right` in a floating window's window menu
- **THEN** that window's box is 39×24 at column 40 and row 0

#### Scenario: Restore goes back past a tile
- **WHEN** window 1's box is 40×12 at column 20 and row 4, and the user tiles it left, then maximizes it, then restores it
- **THEN** its box is 40×12 at column 20 and row 4

#### Scenario: Restore after a reload
- **WHEN** window 1 is maximized in an 80×24 screen area, the user saves `user/init.lua` unchanged, and then restores window 1
- **THEN** its title bar showed `[❐]` after the reload
- **AND** after restoring, its box is 40×12 at column 20 and row 6

#### Scenario: Terminal grows while maximized
- **WHEN** window 1 is maximized in an 80×24 screen area and the client's terminal becomes 100×30
- **THEN** once the layout with the 100×30 screen area arrives, window 1's box is 100×30 at column 0 and row 0

#### Scenario: Another client grows the screen area
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")`, a client with an 80×24 terminal has window 1 tiled right, and a second client with a 100×30 terminal attaches to the same session
- **THEN** window 1's box is 50×30 at column 50 and row 0

#### Scenario: Smaller screen area keeps the state
- **WHEN** window 1 is maximized in a 100×30 screen area and the screen area becomes 80×24
- **THEN** window 1's box is drawn 80×24 at column 0 and row 0, and its title bar shows `[❐]`

### Requirement: Desktop menus
The window list and the window menu SHALL be desktop menus. A desktop menu SHALL be a floating plugin window of the plugin `desktop`, with no border, `cursorline` off, `on_mouse`, and focus when it opens. The window list SHALL be opened with `hover = true`, as the plugin-windows capability's "Mouse in plugin windows" defines. The window menu SHALL be opened without `hover`. A desktop menu SHALL make `root` the active table when it opens. At most one desktop menu SHALL be open: opening one SHALL close the other first.

A desktop menu SHALL draw its own frame in its lines. Let `w` and `h` be its width and height:

- Its first line SHALL be its top border: `┌` on column 0; its title from column 1, in `PluginWindowTitle`; `─` on every other cell; its buttons, when it has any, placed on columns `w − 2 − d` to `w − 3`, where `d` is their total width, as the window-decorations capability places a window's decorations; and `┐` on column `w − 1`. The title SHALL be cut, as `gband.ui.truncate` cuts, to `w − 2` cells without buttons and `w − 4 − d` cells with buttons.
- Its last line SHALL be its bottom border: `└` on column 0, `─` on every other cell, its hint when it has one, placed as buttons are placed on the top border, and `┘` on column `w − 1`.
- Each line between them SHALL show one row of the menu: an entry, as `│`, a space, the entry's `w − 4` cells, a space and `│`; or a separator, as `├`, `─` on columns 1 to `w − 2`, and `┤`. An entry's cells SHALL hold the entry's text cut by `gband.ui.truncate` to `w − 4` cells and padded with spaces to `w − 4` cells, unless the menu lays them out as "Window list" does.
- The frame characters SHALL be drawn in `PluginWindowBorder`. An entry's cells SHALL be drawn in `PluginWindow`, unless the menu gives some or all of them another group. The selected entry's row SHALL be drawn from column 1 to column `w − 2` in `PluginWindowCursorLine`.

A desktop menu SHALL have one selected entry, the first entry when it opens. A separator SHALL never be selected. When the rows do not fit in `h − 2` lines, the menu SHALL show `h − 2` consecutive rows. The first shown row SHALL change only in these cases:

- When the selected entry changes, or moves to another row, it SHALL change by the least amount that shows the selected entry.
- On a wheel step over the window list, it SHALL change as "Window list" defines.
- When the number of rows or `h` changes, it SHALL change by the least amount that still shows `h − 2` consecutive rows, or become the first row when every row fits.

These keys SHALL act in a focused desktop menu:

| key | effect |
|---|---|
| `j`, Down | select the next entry; nothing on the last |
| `k`, Up | select the previous entry; nothing on the first |
| PageDown, PageUp | select the entry `h − 2` rows further down or up, or the last or first entry when there is none |
| Home, End | select the first or the last entry |
| Enter | pick the selected entry |
| Escape, `q` | close the menu |

A left press on an entry SHALL select it, and a release of the left button on the same entry SHALL pick it. A left press on a button SHALL act when the left button is released on the same button. A wheel step down or up over the window menu SHALL act as `j` or `k`. A wheel step over the window list, and a move of the pointer over it, SHALL act as "Window list" defines. Any other press on the menu SHALL do nothing. Picking an entry SHALL close the menu before the entry acts.

A press of any button whose `target` is not the open desktop menu SHALL close it, except the press that opened it. That press SHALL still take its usual effect, from its binding or the interactive mode defaults. Closing the window menu SHALL leave the focused window as it was. Closing the window list SHALL keep or cancel, as "Window list" defines.

#### Scenario: Move past the separator
- **WHEN** the window list shows `n New window`, `s Settings`, `m Multi mode`, a separator and `1 notes`, with `New window` selected, and the user presses `j` three times
- **THEN** the entry `1 notes` is selected

#### Scenario: Escape closes
- **WHEN** a desktop menu is open and focused and the user presses Escape, then types `echo hi` and Enter
- **THEN** the menu closes and the focused window prints `hi`

#### Scenario: Press outside closes and takes its effect
- **WHEN** the window list is open and the user clicks the content of an unfocused floating shell window
- **THEN** the window list closes, the window is focused, and a selection starts at the clicked cell

#### Scenario: Wheel moves the selection
- **WHEN** the window menu of window 1 is open with `Close` selected and the user turns the wheel one step down over it
- **THEN** `Maximize` is selected

#### Scenario: One menu at a time
- **WHEN** the window menu of window 1 is open and the user presses the right button on empty ribbon
- **THEN** the window menu is closed and the window list is open

#### Scenario: Hover only in the window list
- **WHEN** the floating style is in use, and a binding function reads `gband.win.info` of the open window list, and later of the open window menu of window 1
- **THEN** the window list's information holds `hover = true`, and the window menu's holds `hover = false`

### Requirement: Window list
The window list SHALL be a desktop menu, as "Desktop menus" defines, titled `windows`. Its buttons SHALL be `[□]`, or `[❐]` while its height is maximized, then `[X]`, in the groups `DesktopButton` and `DesktopClose`. Its hint SHALL be `? keys`, shown only while the `prefix` table binds `?`.

Its rows SHALL be, in this order:

1. the entry `New window`, with the shortcut `n`;
2. the entry `Settings`, with the shortcut `s`;
3. the entry `Multi mode`, with the shortcut `m`;
4. a separator, when at least one window entry follows;
5. one window entry for each window of the layout that runs a program, as "Windows float" defines.

The windows that this client has focused SHALL come first among the window entries, from the highest `last_focus` down, as `gband.layout()` gives `last_focus`. The windows that this client has never focused SHALL follow, in layout order: the bands in order, and in each band its columns from left to right with their windows from top to bottom, then its floating windows in the order of its floating list. The first nine window entries SHALL have the shortcuts `1` to `9`, in order. The tenth SHALL have the shortcut `0`. A window entry after the tenth SHALL have no shortcut.

The label of `New window` and of `Settings` SHALL be its name. The label of `Multi mode` SHALL be `Multi mode` while this client's multi mode is off, and `Multi mode (on)` while it is on, as the multi-mode capability defines. A window entry's label SHALL be the window's shown name, as `gband.layout()` gives `name`. When this client has minimized the window, ` (minimized)` SHALL follow the name in the label. While the windows that run a program are in two or more bands, each window entry SHALL have a band part: `band `, with its space, followed by the band label of the window's band. The band label SHALL be the band's position counted from 1, as the sidebar capability's "Band labels" defines.

An entry's `w − 4` cells, as "Desktop menus" defines them, SHALL hold, from left to right: its shortcut, or a space when it has none; a space; its label; spaces; and its band part when it has one, ending on the last of the `w − 4` cells. The label SHALL be cut, as `gband.ui.truncate` cuts, so that at least one space stays between it and the band part. Outside the selected entry, the shortcut SHALL be drawn in `DesktopShortcut`. The other cells of the entry of a window that this client has minimized SHALL be drawn in `DesktopMinimized`.

An entry's width SHALL be 2, plus the width of its label, plus 1 and the width of its band part when it has one. The window list SHALL be the same whatever opens it: `desktop.list`, `desktop.leader`, a right press on empty ribbon, or a press on the sidebar's apps character. Its width SHALL be the smallest of the ribbon area's width and the larger of 24 and the widest entry's width plus 4. Its height SHALL be the smallest of the ribbon area's height, 15, and its number of rows plus 2. Opened by a right press, it SHALL be placed as "Right press" defines. Otherwise it SHALL be centred in the ribbon area, each offset rounded down. Opening the list SHALL change neither the viewed band nor the focused window.

The band that the client views when the list opens SHALL be the list's band at opening. To return to the band at opening, the list SHALL make the client view that band, as `gband.band.view` does, also when the client views it already. When that band has left the layout, it SHALL view the band now at its position, or the last band when none is. Viewing a band ends a peek, as the layout-view capability's "Peek a window" defines. Focus then goes to the window that this client focused when the list opened, when that window is still in the layout, because previews record no focus.

Each time the selection lands on an entry, by a key, a press, a shortcut or the pointer, the list SHALL preview that entry:

- A window entry SHALL peek its window, as `gband.window.focus(window, { peek = true })` does, as the lua-control capability defines. The client then views the window's band, focuses the window and draws it above the other floating windows of its band, also when this client has minimized it.
- `New window`, `Settings` and `Multi mode` SHALL return to the band at opening.

A preview records nothing in this client's view: the focus order, the stacking order of every band, the window that each band remembers and the set of minimized windows stay as they were. The window list SHALL stay focused through its previews, as the plugin-windows capability's "Focused plugin window" defines for a floating plugin window's `keys` functions, and for the `on_mouse` of a floating plugin window opened with `hover = true`.

The window list SHALL close in one of two ways:

- To keep, it SHALL close and then, while the client peeks a window, as `gband.view()` gives `peek`, focus that window as `gband.window.focus` does. This records the focus, puts the window on top of the stacking order and restores it when this client has minimized it. It emits no `FocusChanged` and no `BandChanged`, as the layout-view capability's "Peek a window" defines for a focus of the peeked window.
- To cancel, it SHALL close and then return to the band at opening. The client then views the band, the focused window, the stacking order of every band and the minimized windows that it had when the list opened, except for the windows that have left the layout since.

Picking an entry SHALL first select it. Then it SHALL act as follows:

- A window entry SHALL keep. Its window is then focused, as `gband.window.focus` does: the client views its band, and restores the window when this client has minimized it.
- `New window` SHALL cancel, then open a window running the user's shell, floating, in the band that the client then views, as `gband.action.open_window({ floating = true, band = <that band> })` does.
- `Settings` SHALL cancel, then open the settings window, as `gband.settings.open()` does.
- `Multi mode` SHALL cancel, then toggle this client's multi mode, as `gband.action.toggle_multi()` does.

`[□]` SHALL give the list the ribbon area's height at row 0 and show `[❐]`. `[❐]` SHALL give it back the height and the row it had before. `[X]` SHALL cancel. Escape and `q` SHALL cancel. When code other than the desktop plugin closes the window list, as `gband.win.close` does, the list SHALL cancel. A reload that loads a new configuration SHALL close the list without keeping or cancelling, as the plugin-windows capability closes every plugin window. It ends a peek, as the layout-view capability's "Peek a window" defines: the client keeps the band it views and focuses the window it last focused there, and records nothing.

These keys SHALL also act while the window list is focused:

- `n`, `s`, `m` and the shortcut of each window entry SHALL act as a left press and release on that entry: they select it, then pick it.
- A digit from `0` to `9` that no entry has as its shortcut SHALL do nothing.
- A key that the `prefix` table binds SHALL keep, and then run that binding with `gband.keymap.run("prefix", key)`. This SHALL hold for every binding of `prefix` except the binding of the prefix key itself, the bindings of mouse names, the keys of the table in "Desktop menus", `n`, `s`, `m` and the digits `0` to `9`. Those keys SHALL keep their meaning in the list, whatever `prefix` binds. With the floating preset's bindings, `D` therefore detaches, `:` opens the Lua prompt, `?` opens the key list, and `N` renames the focused window, which is the window that the list previews.

When the pointer moves, with no button held, to a cell of an entry, as `on_mouse` reports with the kind `"move"`, that entry SHALL be selected. A move to a separator or to a cell of the frame SHALL change nothing. While the rows do not fit, a wheel step down over the list SHALL make the next row the first shown row, and a wheel step up the previous row, while `h − 2` rows stay shown. A wheel step SHALL NOT change the selected entry.

A right press that opens a desktop menu, as "Right press" defines, SHALL make the window list keep before the new menu opens. Every other press whose target is not the window list SHALL close it, as "Desktop menus" defines, once the press has taken its effect:

- When the press's payload names a `window`, as the lua-events capability defines it for `MousePressed`, the list SHALL focus that window as `gband.window.focus` does.
- Otherwise, when the press changed the viewed band or the focused window, the list SHALL leave the view as the press left it.
- Otherwise the list SHALL cancel.

While the window list is open, it SHALL build its rows again when `WindowOpened`, `WindowClosed`, `LayoutChanged` or `FocusChanged` runs, and when the desktop plugin minimizes a window. The order of the window entries SHALL be set when the list opens. An entry whose window has left the layout SHALL be removed. The windows that opened since the list opened SHALL follow the others, in layout order. The shortcuts, labels and band parts SHALL follow the new rows and the layout. The list SHALL keep the selected entry when that entry is still in the list. Otherwise it SHALL select the entry on the same row, or the last entry when no entry is on that row. Building the rows again SHALL preview nothing.

#### Scenario: List of two windows
- **WHEN** no `user/init.lua` exists, `user/keystyle.lua` holds `return "floating"`, the client's terminal is 80×24, the viewed band holds floating windows 1 and 2 named `notes` and `logs`, this client focused window 1 and then window 2, and a binding function dispatches `desktop.list`
- **THEN** a focused floating plugin window 24 columns wide and 8 rows high spans columns 27 to 50 and rows 8 to 15 of the 79-column ribbon area
- **AND** its rows read `┌windows────────[□][X]─┐`, `│ n New window         │`, `│ s Settings           │`, `│ m Multi mode         │`, `├──────────────────────┤`, `│ 1 logs               │`, `│ 2 notes              │` and `└───────────────? keys─┘`
- **AND** `root` is the active table and window 2 is still focused

#### Scenario: Most recent first
- **WHEN** the floating style is in use, the viewed band holds floating windows 1, 2 and 3 named `notes`, `logs` and `mail` with overlapping boxes, this client focused windows 1, 2 and 3 in that order, and the user opens the window list
- **THEN** its window entries read `1 mail`, `2 logs` and `3 notes`

#### Scenario: Windows never focused come last
- **WHEN** band 1's floating list holds windows 1, 2 and 3 named `notes`, `logs` and `mail`, a second client with the floating style attaches to the session, so that its first view focuses window 3, and the user of the second client opens the window list
- **THEN** its window entries read `1 mail`, `2 notes` and `3 logs`

#### Scenario: Tenth window and after
- **WHEN** the floating style is in use, the viewed band holds 11 floating windows named `w1` to `w11`, this client focused them in that order, and the user opens the window list and presses End
- **THEN** the window entries `w11` to `w3` have the shortcuts `1` to `9`, `w2` has `0`, and the row of `w1` reads `│   w1                 │`

#### Scenario: Shortcut picks a window
- **WHEN** the window list of "Most recent first" is open and the user presses `2`
- **THEN** the window list is closed, window 2 is focused and drawn over window 3, and window 2 has the highest `last_focus`

#### Scenario: Digits before a prefix binding
- **WHEN** `user/init.lua` calls `gband.keystyle.use("floating")` and binds `prefix 1` to `gband.action.open_window`, the window list of "Most recent first" is open, and the user presses `1`
- **THEN** the window list is closed, window 3 is focused, and no window opens

#### Scenario: Digit without an entry
- **WHEN** the window list of "List of two windows" is open and the user presses `5`
- **THEN** the window list is still open with `New window` selected, and window 2 is still focused

#### Scenario: Preview by keys
- **WHEN** the window list of "Most recent first" is open and the user presses `j` four times
- **THEN** the entry `2 logs` is selected, window 2 is focused and drawn over window 3, and the window list is still open and focused
- **AND** window 3's `last_focus` is still higher than window 2's

#### Scenario: Preview a minimized window
- **WHEN** the floating style is in use, the viewed band holds floating windows 1, 2 and 3 named `notes`, `logs` and `mail` with overlapping boxes, this client focused windows 1, 2 and 3 in that order and minimized window 1, and the user opens the window list and presses `j` five times
- **THEN** the entry `3 notes (minimized)` is selected, and window 1 is focused and drawn over windows 2 and 3
- **AND** `gband.layout()` still marks window 1 `minimized`

#### Scenario: Preview a window in another band
- **WHEN** the floating style is in use, band 1 holds floating windows 1 and 2 named `notes` and `logs`, band 2 holds floating window 3 named `mail`, this client focused windows 3, 1 and 2 in that order and views band 1, and the user opens the window list and presses End
- **THEN** the window entries read `1 logs        band 1`, `2 notes       band 1` and `3 mail        band 2`, and `3 mail        band 2` is selected
- **AND** the client views band 2 with window 3 focused, and the window list is still open and focused

#### Scenario: Back to the opening state
- **WHEN** the window list of "Preview by keys" previews window 2 and the user presses Home
- **THEN** `New window` is selected, and window 3 is focused and drawn over window 2

#### Scenario: Preview by the pointer
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")`, the client's terminal is 80×24, the window list of "Most recent first" is open at columns 28 to 51 and rows 7 to 15, and the user moves the pointer with no button held to column 35 and row 13
- **THEN** the entry `2 logs` is selected, window 2 is focused and drawn over window 3, and the window list is still focused

#### Scenario: Wheel scrolls the list
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")`, the client's terminal is 80×24, the viewed band holds 20 floating windows, and the user opens the window list and turns the wheel one step down over it
- **THEN** the first row between the list's borders is the entry `s Settings`, `New window` is still selected, and the focused window has not changed

#### Scenario: Escape returns to the state at opening
- **WHEN** the floating style is in use, band 1 holds floating windows 1, 2 and 3 named `notes`, `logs` and `mail` with overlapping boxes, band 2 holds floating windows 4 and 5 named `top` and `low` with overlapping boxes, this client focused windows 5, 4, 1, 2 and 3 in that order, minimized window 1 and views band 1, and the user opens the window list, presses `j` seven times and then presses Escape
- **THEN** after the seventh `j` the client views band 2 with window 5 focused and drawn over window 4
- **AND** after Escape the window list is closed, the client views band 1 with window 3 focused, draws window 3 over window 2 and does not draw window 1, and `gband.layout()` marks window 1 `minimized`
- **AND** when the user then views band 2, the client focuses window 4 and draws it over window 5

#### Scenario: Press on empty ribbon cancels
- **WHEN** the window list of "Preview by keys" previews window 2 and the user presses the left button on a cell of empty ribbon
- **THEN** the window list is closed, and window 3 is focused and drawn over window 2

#### Scenario: Press on the previewed window keeps it
- **WHEN** the window list of "Preview a minimized window" previews window 1 and the user clicks the content of window 1
- **THEN** the window list is closed, window 1 is focused, no longer minimized, and drawn over windows 2 and 3

#### Scenario: Pick a window
- **WHEN** the window list of "List of two windows" is open and the user presses `j`, `j`, `j`, `j`, then Enter
- **THEN** the list closes and window 1 is focused with the highest `last_focus`

#### Scenario: Restore a minimized window
- **WHEN** the window list of "Preview a minimized window" previews window 1 and the user presses Enter
- **THEN** the window list is closed, window 1 is no longer minimized, and it is focused and drawn over windows 2 and 3

#### Scenario: Windows in two bands
- **WHEN** band 1 holds window 1 named `notes`, band 2 holds window 2 named `logs`, the client views band 1 with window 1 focused, and the user opens the window list
- **THEN** its window entries read `1 notes       band 1` and `2 logs        band 2`
- **AND** when the user picks `2 logs        band 2`, the client views band 2 with window 2 focused

#### Scenario: New window from a preview in another band
- **WHEN** the window list of "Preview a window in another band" previews window 3 in band 2 and the user presses `n`
- **THEN** the window list is closed, the client views band 1, and band 1's floating list ends with a new window running the user's shell, which is focused

#### Scenario: Open a new window
- **WHEN** the window list is open with `New window` selected and the user presses Enter
- **THEN** the viewed band's floating list ends with a new window running the user's shell, and it is focused

#### Scenario: Open the settings
- **WHEN** the window list is open and the user presses `j` then Enter
- **THEN** the window list is closed and the settings window is open and focused

#### Scenario: Multi mode from the list
- **WHEN** the floating style is in use, the viewed band holds floating shell windows named `notes` and `logs`, and the user presses Ctrl+Space, `m`, then types `echo hi` and Enter
- **THEN** after `m` the window list is closed and `root` is active
- **AND** both windows print `hi`
- **AND** when the user opens the window list again, its rows between its borders are `n New window`, `s Settings`, `m Multi mode (on)`, a separator and the two window entries

#### Scenario: Multi mode off from the list
- **WHEN** the floating style is in use, multi mode is on, the viewed band holds two floating shell windows, and the user opens the window list, presses `j` twice, then Enter, and types `echo solo` and Enter
- **THEN** only the focused window prints `solo`

#### Scenario: Multi mode from a preview
- **WHEN** the window list of "Preview by keys" previews window 2 and the user presses `m`
- **THEN** the window list is closed, multi mode is on, and window 3 is focused and drawn over window 2

#### Scenario: Settings from a preview
- **WHEN** the window list of "Preview by keys" previews window 2 and the user presses `s`
- **THEN** the window list is closed, the settings window is open and focused, and window 3 is focused and drawn over window 2

#### Scenario: Scroll a long list
- **WHEN** the client's terminal is 80×24 with no bar, the viewed band holds 20 floating windows named `window 1` to `window 20`, which this client opened and focused in that order, and the user opens the window list and presses End
- **THEN** the list is 15 rows high and shows 13 rows between its borders
- **AND** its last window entry, `window 1`, is selected and shown on the row right above its bottom border

#### Scenario: Maximize the list's height
- **WHEN** the window list of "List of two windows" is open and the user clicks its `[□]` at terminal column 45 and row 8
- **THEN** the list spans rows 0 to 23 of the ribbon area and its top border shows `[❐]`
- **AND** a click on `[❐]` gives it back rows 8 to 15

#### Scenario: Close by the button
- **WHEN** the window list of "List of two windows" is open, the user presses `j` four times, and then clicks its `[X]` at terminal column 48 and row 8
- **THEN** the window list is closed and window 2 is focused and drawn over window 1

#### Scenario: Closed by other code
- **WHEN** the window list of "Preview by keys" previews window 2 and a plugin closes it with `gband.win.close`
- **THEN** window 3 is focused and drawn over window 2

#### Scenario: Detach from the list
- **WHEN** the floating style is in use, the window list is open, and the user presses `D`
- **THEN** the client detaches

#### Scenario: Lua prompt from the list
- **WHEN** the floating style is in use, the window list is open, and the user presses `:`
- **THEN** the window list is closed and the Lua prompt is open and focused

#### Scenario: Key list from the list
- **WHEN** the floating style is in use, the window list is open, and the user presses `?`
- **THEN** a floating plugin window titled `prefix keys` is open and focused, with a line for each of `n`, `?`, `:`, `N`, `s`, `!`, `D` and `C-space`

#### Scenario: Prefix key keeps the preview
- **WHEN** the window list of "Preview by keys" previews window 2 and the user presses `:`
- **THEN** the window list is closed, window 2 is focused with the highest `last_focus`, and the Lua prompt is open and focused

#### Scenario: Same list from a right press
- **WHEN** the floating style is in use, the viewed band holds floating windows named `notes` and `logs`, which this client focused in that order, and the user presses the right button on empty ribbon
- **THEN** the window list's rows between its borders are `n New window`, `s Settings`, `m Multi mode`, a separator, `1 logs` and `2 notes`

#### Scenario: Right press keeps the preview
- **WHEN** the window list of "Preview by keys" previews window 2 and the user presses the right button on empty ribbon
- **THEN** a new window list is open and focused at the pointer, and window 2 is focused with the highest `last_focus`

#### Scenario: Window closed under the list
- **WHEN** the window list of "Preview by keys" previews window 2 and the program of window 1 exits
- **THEN** the window list is still open and focused, its window entries read `1 mail` and `2 logs`, and `2 logs` is still selected

#### Scenario: Window focused at opening closes
- **WHEN** the window list of "Most recent first" is open, the user presses `j` five times, so that the list previews window 1, the program of window 3 exits, and the user presses Escape
- **THEN** the window list is closed, and window 2 is focused and drawn over window 1

#### Scenario: Reload from the key list
- **WHEN** the floating style is in use, the window list is open, the user presses `?`, then `!`
- **THEN** the configuration reloads as the configuration capability's "Forced reload" defines, the key list is closed, and `root` is active

### Requirement: Window menu
The window menu SHALL be a desktop menu, as "Desktop menus" defines, for one floating window. Its title SHALL be the window's shown name. It SHALL have no buttons and no hint. Its entries SHALL be, in this order: `Close`; `Maximize`, or `Restore` while the window counts as maximized, as "Maximize, restore and tile" defines; `Minimize`; `Tile left`; and `Tile right`. Its width SHALL be the smallest of the ribbon area's width and the largest of 16, the widest entry plus 4, and the title's width plus 2. Its height SHALL be the smaller of 7 and the ribbon area's height.

Picking an entry SHALL act on the menu's window:

- `Close` SHALL close it, as `close_window` with a target that names the window does.
- `Maximize` and `Restore` SHALL maximize or restore it, then focus it.
- `Minimize` SHALL minimize it, as `gband.window.minimize` does.
- `Tile left` and `Tile right` SHALL tile it, then focus it.

The window menu SHALL close when its window leaves the layout.

#### Scenario: Menu rows
- **WHEN** the window menu of a floating window named `notes` opens
- **THEN** its rows read `┌notes─────────┐`, `│ Close        │`, `│ Maximize     │`, `│ Minimize     │`, `│ Tile left    │`, `│ Tile right   │` and `└──────────────┘`

#### Scenario: Restore entry while maximized
- **WHEN** the window menu of a maximized floating window opens
- **THEN** its second entry is `Restore`

#### Scenario: Close from the menu
- **WHEN** the window menu of window 1 is open and the user presses Enter
- **THEN** the menu closes and window 1 leaves the layout

#### Scenario: Minimize from the menu
- **WHEN** the window menu of window 1 is open and the user presses `j`, `j`, then Enter
- **THEN** the client does not draw window 1

#### Scenario: Window closed under the menu
- **WHEN** the window menu of window 1 is open and window 1's program exits
- **THEN** the window menu is closed

### Requirement: Leader
The floating preset SHALL register one handler of `KeyTableChanged`. When the event's `table` is `prefix`, the handler SHALL call `gband.keymap.enter("root")` and then dispatch `desktop.leader`. `prefix` is not a mode with the floating style, and it holds bindings, so the prefix key makes it active, as the client-attach capability's "Key bindings" defines, and the handler then returns to `root` before the next key.

`desktop.leader` SHALL act as follows:

- When the window list is open and is the focused plugin window, it SHALL close the window list by keeping, as "Window list" defines, then run the binding of the prefix key in `prefix` with `gband.keymap.run("prefix", "prefix")`. With the floating preset's bindings, this sends the prefix key to the focused window, which is the window that the list previewed.
- Otherwise it SHALL act as `desktop.list` does: open the window list centred in the ribbon area, or focus it when it is open.

#### Scenario: Leader opens the list
- **WHEN** no `user/init.lua` exists, `user/keystyle.lua` holds `return "floating"`, and the user presses Ctrl+Space
- **THEN** the window list is open and focused, and `root` is the active table
- **AND** a following `j` selects `Settings` and reaches no window

#### Scenario: Leader twice sends the prefix key
- **WHEN** the floating style is in use, the focused window runs `cat -v`, and the user presses Ctrl+Space twice, then Enter
- **THEN** the window list is closed and `cat -v` prints `^@` on a line of its own

#### Scenario: Leader after a preview
- **WHEN** the floating style is in use, the viewed band holds floating windows 1 and 2, window 1 runs `cat -v`, this client focused window 1 and then window 2, and the user presses Ctrl+Space, `j` four times, Ctrl+Space and Enter
- **THEN** the window list is closed, window 1 is focused, and `cat -v` prints `^@` on a line of its own

#### Scenario: Table changes of the leader
- **WHEN** the floating style is in use, a `KeyTableChanged` handler registered after the preset records each payload, and the user presses Ctrl+Space
- **THEN** it records `prefix` with previous `root`, then `root` with previous `prefix`, and nothing else

#### Scenario: Leader over the Lua prompt
- **WHEN** the floating style is in use, the Lua prompt is open and focused, and the user presses Ctrl+Space
- **THEN** the window list is open and focused

### Requirement: Desktop groups
The module `gband.desktop` SHALL define these groups as defaults, in the defaults layer the highlights capability defines, when it is first required: `DesktopButton` as `{ bold = true }`, `DesktopClose` as `{ bold = true }`, `DesktopMinimized` as `{ dim = true }` and `DesktopShortcut` as `{ link = "KeyListKey" }`. No bundled colorscheme or theme SHALL set `DesktopClose`. A change of a group's resolved style SHALL show in the next frame, on the title bar buttons and in an open desktop menu.

#### Scenario: Defaults without a colorscheme
- **WHEN** `user/colors/blank.lua` is empty and `user/init.lua` calls `gband.colorscheme("blank")` and `gband.keystyle.use("floating")`
- **THEN** `DesktopButton` resolves to `{ bold = true }`, `DesktopClose` to `{ bold = true }`, `DesktopMinimized` to `{ dim = true }` and `DesktopShortcut` to `{ bold = true }`
- **AND** `gband.hl.get("DesktopShortcut")` returns `{ link = "KeyListKey" }`

#### Scenario: Close button in every bundled theme
- **WHEN** the floating style is in use and each name that `gband.settings.themes()` returns is made the active colorscheme in turn
- **THEN** `DesktopClose` resolves to `{ bold = true }` each time

#### Scenario: Restyle the close button
- **WHEN** the floating style is in use and a binding function calls `gband.hl.set("DesktopClose", { fg = 2 })`
- **THEN** the next frame draws each floating window's `[X]` with the foreground palette index 2

#### Scenario: Shortcuts follow the key list's style
- **WHEN** the floating style is in use, a binding function calls `gband.hl.set("KeyListKey", { fg = 4, bold = true })`, and the window list is open
- **THEN** `DesktopShortcut` resolves to `{ fg = 4, bold = true }`, and the cell of `s` on the row of `Settings`, which is not selected, is drawn bold with the foreground palette index 4
