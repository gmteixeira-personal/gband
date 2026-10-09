## MODIFIED Requirements

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

### Requirement: Window list
The window list SHALL be a desktop menu, as "Desktop menus" defines, titled `windows`. Its buttons SHALL be `[□]`, or `[❐]` while its height is maximized, then `[X]`, in the groups `DesktopButton` and `DesktopClose`. Its hint SHALL be `? keys`, shown only while the `prefix` table binds `?`.

Its rows SHALL be, in this order:

1. the entry `New window`, with the shortcut `n`;
2. the entry `Settings`, with the shortcut `s`;
3. a separator, when at least one window entry follows;
4. one window entry for each window of the layout that runs a program, as "Windows float" defines.

The windows that this client has focused SHALL come first among the window entries, from the highest `last_focus` down, as `gband.layout()` gives `last_focus`. The windows that this client has never focused SHALL follow, in layout order: the bands in order, and in each band its columns from left to right with their windows from top to bottom, then its floating windows in the order of its floating list. The first nine window entries SHALL have the shortcuts `1` to `9`, in order. The tenth SHALL have the shortcut `0`. A window entry after the tenth SHALL have no shortcut.

The label of `New window` and of `Settings` SHALL be its name. A window entry's label SHALL be the window's shown name, as `gband.layout()` gives `name`. When this client has minimized the window, ` (minimized)` SHALL follow the name in the label. While the windows that run a program are in two or more bands, each window entry SHALL have a band part: `band `, with its space, followed by the band label of the window's band. The band label SHALL be the band's position counted from 1, as the sidebar capability's "Band labels" defines.

An entry's `w − 4` cells, as "Desktop menus" defines them, SHALL hold, from left to right: its shortcut, or a space when it has none; a space; its label; spaces; and its band part when it has one, ending on the last of the `w − 4` cells. The label SHALL be cut, as `gband.ui.truncate` cuts, so that at least one space stays between it and the band part. Outside the selected entry, the shortcut SHALL be drawn in `DesktopShortcut`. The other cells of the entry of a window that this client has minimized SHALL be drawn in `DesktopMinimized`.

An entry's width SHALL be 2, plus the width of its label, plus 1 and the width of its band part when it has one. The window list SHALL be the same whatever opens it: `desktop.list`, `desktop.leader`, a right press on empty ribbon, or a press on the sidebar's apps character. Its width SHALL be the smallest of the ribbon area's width and the larger of 24 and the widest entry's width plus 4. Its height SHALL be the smallest of the ribbon area's height, 15, and its number of rows plus 2. Opened by a right press, it SHALL be placed as "Right press" defines. Otherwise it SHALL be centred in the ribbon area, each offset rounded down. Opening the list SHALL change neither the viewed band nor the focused window.

The band that the client views when the list opens SHALL be the list's band at opening. To return to the band at opening, the list SHALL make the client view that band, as `gband.band.view` does, also when the client views it already. When that band has left the layout, it SHALL view the band now at its position, or the last band when none is. Viewing a band ends a peek, as the layout-view capability's "Peek a window" defines. Focus then goes to the window that this client focused when the list opened, when that window is still in the layout, because previews record no focus.

Each time the selection lands on an entry, by a key, a press, a shortcut or the pointer, the list SHALL preview that entry:

- A window entry SHALL peek its window, as `gband.window.focus(window, { peek = true })` does, as the lua-control capability defines. The client then views the window's band, focuses the window and draws it above the other floating windows of its band, also when this client has minimized it.
- `New window` and `Settings` SHALL return to the band at opening.

A preview records nothing in this client's view: the focus order, the stacking order of every band, the window that each band remembers and the set of minimized windows stay as they were. The window list SHALL stay focused through its previews, as the plugin-windows capability's "Focused plugin window" defines for a floating plugin window's `keys` functions, and for the `on_mouse` of a floating plugin window opened with `hover = true`.

The window list SHALL close in one of two ways:

- To keep, it SHALL close and then, while the client peeks a window, as `gband.view()` gives `peek`, focus that window as `gband.window.focus` does. This records the focus, puts the window on top of the stacking order and restores it when this client has minimized it. It emits no `FocusChanged` and no `BandChanged`, as the layout-view capability's "Peek a window" defines for a focus of the peeked window.
- To cancel, it SHALL close and then return to the band at opening. The client then views the band, the focused window, the stacking order of every band and the minimized windows that it had when the list opened, except for the windows that have left the layout since.

Picking an entry SHALL first select it. Then it SHALL act as follows:

- A window entry SHALL keep. Its window is then focused, as `gband.window.focus` does: the client views its band, and restores the window when this client has minimized it.
- `New window` SHALL cancel, then open a window running the user's shell, floating, in the band that the client then views, as `gband.action.open_window({ floating = true, band = <that band> })` does.
- `Settings` SHALL cancel, then open the settings window, as `gband.settings.open()` does.

`[□]` SHALL give the list the ribbon area's height at row 0 and show `[❐]`. `[❐]` SHALL give it back the height and the row it had before. `[X]` SHALL cancel. Escape and `q` SHALL cancel. When code other than the desktop plugin closes the window list, as `gband.win.close` does, the list SHALL cancel. A reload that loads a new configuration SHALL close the list without keeping or cancelling, as the plugin-windows capability closes every plugin window. It ends a peek, as the layout-view capability's "Peek a window" defines: the client keeps the band it views and focuses the window it last focused there, and records nothing.

These keys SHALL also act while the window list is focused:

- `n`, `s` and the shortcut of each window entry SHALL act as a left press and release on that entry: they select it, then pick it.
- A digit from `0` to `9` that no entry has as its shortcut SHALL do nothing.
- A key that the `prefix` table binds SHALL keep, and then run that binding with `gband.keymap.run("prefix", key)`. This SHALL hold for every binding of `prefix` except the binding of the prefix key itself, the bindings of mouse names, the keys of the table in "Desktop menus", `n`, `s` and the digits `0` to `9`. Those keys SHALL keep their meaning in the list, whatever `prefix` binds. With the floating preset's bindings, `D` therefore detaches, `:` opens the Lua prompt, `?` opens the key list, and `N` renames the focused window, which is the window that the list previews.

When the pointer moves, with no button held, to a cell of an entry, as `on_mouse` reports with the kind `"move"`, that entry SHALL be selected. A move to a separator or to a cell of the frame SHALL change nothing. While the rows do not fit, a wheel step down over the list SHALL make the next row the first shown row, and a wheel step up the previous row, while `h − 2` rows stay shown. A wheel step SHALL NOT change the selected entry.

A right press that opens a desktop menu, as "Right press" defines, SHALL make the window list keep before the new menu opens. Every other press whose target is not the window list SHALL close it, as "Desktop menus" defines, once the press has taken its effect:

- When the press's payload names a `window`, as the lua-events capability defines it for `MousePressed`, the list SHALL focus that window as `gband.window.focus` does.
- Otherwise, when the press changed the viewed band or the focused window, the list SHALL leave the view as the press left it.
- Otherwise the list SHALL cancel.

While the window list is open, it SHALL build its rows again when `WindowOpened`, `WindowClosed`, `LayoutChanged` or `FocusChanged` runs, and when the desktop plugin minimizes a window. The order of the window entries SHALL be set when the list opens. An entry whose window has left the layout SHALL be removed. The windows that opened since the list opened SHALL follow the others, in layout order. The shortcuts, labels and band parts SHALL follow the new rows and the layout. The list SHALL keep the selected entry when that entry is still in the list. Otherwise it SHALL select the entry on the same row, or the last entry when no entry is on that row. Building the rows again SHALL preview nothing.

#### Scenario: List of two windows
- **WHEN** no `user/init.lua` exists, `user/keystyle.lua` holds `return "floating"`, the client's terminal is 80×24, the viewed band holds floating windows 1 and 2 named `notes` and `logs`, this client focused window 1 and then window 2, and a binding function dispatches `desktop.list`
- **THEN** a focused floating plugin window 24 columns wide and 7 rows high spans columns 27 to 50 and rows 8 to 14 of the 79-column ribbon area
- **AND** its rows read `┌windows────────[□][X]─┐`, `│ n New window         │`, `│ s Settings           │`, `├──────────────────────┤`, `│ 1 logs               │`, `│ 2 notes              │` and `└───────────────? keys─┘`
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
- **WHEN** the window list of "Most recent first" is open and the user presses `j` three times
- **THEN** the entry `2 logs` is selected, window 2 is focused and drawn over window 3, and the window list is still open and focused
- **AND** window 3's `last_focus` is still higher than window 2's

#### Scenario: Preview a minimized window
- **WHEN** the floating style is in use, the viewed band holds floating windows 1, 2 and 3 named `notes`, `logs` and `mail` with overlapping boxes, this client focused windows 1, 2 and 3 in that order and minimized window 1, and the user opens the window list and presses `j` four times
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
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")`, the client's terminal is 80×24, the window list of "Most recent first" is open at columns 28 to 51 and rows 8 to 15, and the user moves the pointer with no button held to column 35 and row 13
- **THEN** the entry `2 logs` is selected, window 2 is focused and drawn over window 3, and the window list is still focused

#### Scenario: Wheel scrolls the list
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")`, the client's terminal is 80×24, the viewed band holds 20 floating windows, and the user opens the window list and turns the wheel one step down over it
- **THEN** the first row between the list's borders is the entry `s Settings`, `New window` is still selected, and the focused window has not changed

#### Scenario: Escape returns to the state at opening
- **WHEN** the floating style is in use, band 1 holds floating windows 1, 2 and 3 named `notes`, `logs` and `mail` with overlapping boxes, band 2 holds floating windows 4 and 5 named `top` and `low` with overlapping boxes, this client focused windows 5, 4, 1, 2 and 3 in that order, minimized window 1 and views band 1, and the user opens the window list, presses `j` six times and then presses Escape
- **THEN** after the sixth `j` the client views band 2 with window 5 focused and drawn over window 4
- **AND** after Escape the window list is closed, the client views band 1 with window 3 focused, draws window 3 over window 2 and does not draw window 1, and `gband.layout()` marks window 1 `minimized`
- **AND** when the user then views band 2, the client focuses window 4 and draws it over window 5

#### Scenario: Press on empty ribbon cancels
- **WHEN** the window list of "Preview by keys" previews window 2 and the user presses the left button on a cell of empty ribbon
- **THEN** the window list is closed, and window 3 is focused and drawn over window 2

#### Scenario: Press on the previewed window keeps it
- **WHEN** the window list of "Preview a minimized window" previews window 1 and the user clicks the content of window 1
- **THEN** the window list is closed, window 1 is focused, no longer minimized, and drawn over windows 2 and 3

#### Scenario: Pick a window
- **WHEN** the window list of "List of two windows" is open and the user presses `j`, `j`, `j`, then Enter
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
- **AND** a click on `[❐]` gives it back rows 8 to 14

#### Scenario: Close by the button
- **WHEN** the window list of "List of two windows" is open, the user presses `j` three times, and then clicks its `[X]` at terminal column 48 and row 8
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
- **THEN** the window list's rows between its borders are `n New window`, `s Settings`, a separator, `1 logs` and `2 notes`

#### Scenario: Right press keeps the preview
- **WHEN** the window list of "Preview by keys" previews window 2 and the user presses the right button on empty ribbon
- **THEN** a new window list is open and focused at the pointer, and window 2 is focused with the highest `last_focus`

#### Scenario: Window closed under the list
- **WHEN** the window list of "Preview by keys" previews window 2 and the program of window 1 exits
- **THEN** the window list is still open and focused, its window entries read `1 mail` and `2 logs`, and `2 logs` is still selected

#### Scenario: Window focused at opening closes
- **WHEN** the window list of "Most recent first" is open, the user presses `j` four times, so that the list previews window 1, the program of window 3 exits, and the user presses Escape
- **THEN** the window list is closed, and window 2 is focused and drawn over window 1

#### Scenario: Reload from the key list
- **WHEN** the floating style is in use, the window list is open, the user presses `?`, then `!`
- **THEN** the configuration reloads as the configuration capability's "Forced reload" defines, the key list is closed, and `root` is active
