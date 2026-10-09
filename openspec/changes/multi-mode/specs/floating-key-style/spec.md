## MODIFIED Requirements

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

