# plugin-windows Specification

## Purpose

Defines the plugin windows that Lua code opens and writes in the client: floating plugin windows drawn by one client over its ribbon area, and drawn windows in the shared layout. It covers their contents, scrolling, cursor line, focus and keys, geometry, callbacks, closing and highlight groups.

## Requirements

### Requirement: Plugin window API
`gband.win` SHALL hold the functions `open`, `close`, `set_lines`, `scroll`, `set_cursor`, `focus`, `set_config`, `info` and `list`. `info` and `list` SHALL be callable from any code that runs after loading. The other functions SHALL be callable wherever an action value is, as the configuration capability defines. Calling any of them while the configuration loads SHALL be an error at the line of the call. Plugin windows SHALL exist only in the client.

A function that takes a plugin window SHALL take its number. A number that names no open plugin window SHALL be an error at the line of the call, except for `close`. An argument of the wrong type, an unknown option field, or a value out of range SHALL be an error at the line of the call.

#### Scenario: Open while loading
- **WHEN** line 5 of `user/init.lua` calls `gband.win.open({})` at the top level
- **THEN** loading fails with an error at `user/init.lua` line 5

#### Scenario: Unknown plugin window
- **WHEN** a binding function calls `gband.win.set_lines(42, {})` and no plugin window 42 is open
- **THEN** the call raises an error naming plugin window 42

### Requirement: Open a plugin window
`gband.win.open(opts)` SHALL open a plugin window and return its number. The number is a positive integer, unique within the client process and never reused. The plugin window SHALL belong to the plugin whose code opened it. `opts` SHALL be a table with these optional fields:

| field | kinds | value | default |
|---|---|---|---|
| `kind` | both | `"floating"` or `"tiled"` | `"floating"` |
| `lines` | both | the plugin window's lines, as "Plugin window contents" defines | no lines |
| `focus` | both | boolean | `true` |
| `cursorline` | both | boolean | `false` |
| `keys` | both | a table from key names to functions | empty |
| `on_close` | both | function | none |
| `on_resize` | both | function | none |
| `row`, `col` | floating | an integer of at least 0, or `"center"` | `"center"` |
| `width`, `height` | floating | an integer of at least 1 | half the ribbon area's width or height, rounded down, and at least 1 |
| `border` | floating | boolean | `true` |
| `title` | floating | string | none |
| `band`, `after` | tiled | band and window numbers, as the `open_window` target takes them | with neither given, the viewed band and the focused window, as open window resolves against the view |
| `column_width` | tiled | a width, as `gband.window.set_width` takes it | the `default_column_width` option |

A field for the other kind SHALL be an error. A key name in `keys` SHALL be valid as the configuration capability defines key names.

#### Scenario: Numbers are not reused
- **WHEN** a binding function opens a floating plugin window, closes it, and opens another
- **THEN** the second plugin window's number differs from the first's

#### Scenario: Field of the other kind
- **WHEN** a binding function calls `gband.win.open({ kind = "tiled", row = 2 })`
- **THEN** the call raises an error naming `row`

#### Scenario: Invalid key name
- **WHEN** a binding function opens a plugin window with `keys = { ["ctrl+shift+1"] = fn }`
- **THEN** the call raises an error naming `ctrl+shift+1`

### Requirement: Plugin window contents
A plugin window's lines SHALL be a list, in which each line is a string or a list of spans. A span SHALL be a string or a table `{ text = <string>, hl = <group name> }`. A string line SHALL be one span. A span without `hl`, or a string span, SHALL use the group `PluginWindow`. Control characters, as the status-line capability defines them, SHALL be removed from every span's text. `gband.win.set_lines(win, lines)` SHALL replace the plugin window's lines.

A plugin window's **content area** SHALL be the cells it shows its lines in: a floating plugin window's box inside its border, or the whole box when it has no border, or a drawn window's screen. Row `r` of the content area, counted from 0, SHALL show line `top + r`, where `top` is the plugin window's first shown line. The row SHALL show the spans of that line from left to right at their display widths, as the status-line capability measures them. The line SHALL be cut at the content area's right edge. A wide character that does not fit wholly SHALL leave its cells blank. Each cell a span covers SHALL have the span group's resolved style applied over `PluginWindow`'s resolved style: a field the span's style sets replaces that field, and the other fields of `PluginWindow` stay. Cells that no span covers SHALL be blank in `PluginWindow`'s resolved style.

#### Scenario: Styled line
- **WHEN** `PluginWindow` resolves to `{ fg = 7 }`, `Key` resolves to `{ fg = 3, bold = true }`, and a 20-column floating plugin window shows the line `{ { text = "C-h", hl = "Key" }, " left" }`
- **THEN** the content row's first three cells read `C-h` with foreground 3 and bold
- **AND** the next five read ` left` with foreground 7, and the rest are blank

#### Scenario: Long line is cut
- **WHEN** a floating plugin window's content area is 5 columns wide and shows the line `abcdefgh`
- **THEN** the row reads `abcde`

#### Scenario: Control characters removed
- **WHEN** a plugin window shows the line `"a\tb"`
- **THEN** the row reads `ab`

### Requirement: Scrolling and the cursor line
Every plugin window SHALL have a first shown line `top` and a cursor line `cursor`, both starting at 1. `top` SHALL stay between 1 and the larger of 1 and `n − h + 1`, where `n` is the number of lines and `h` the content area's height. `cursor` SHALL stay between 1 and the larger of 1 and `n`.

- `gband.win.scroll(win, count)` SHALL add the integer `count` to `top`, within its range.
- `gband.win.set_cursor(win, line)` SHALL set `cursor` to `line`, within its range.
- When the plugin window's lines or its content area's size change, `top` and `cursor` SHALL be brought within their ranges.

While `cursorline` is on, `top` SHALL change by the least amount that keeps the cursor line in the content area, after each of these changes. The row showing the cursor line SHALL then have `PluginWindowCursorLine`'s resolved style applied over the style of each of its cells, those no span covers included.

#### Scenario: Scroll a long list
- **WHEN** a floating plugin window's content area is 10 rows high, it holds 25 lines, and a binding function calls `gband.win.scroll(win, 30)`
- **THEN** the content area shows lines 16 to 25

#### Scenario: Cursor keeps itself shown
- **WHEN** a plugin window with `cursorline = true` and a 10-row content area holds 25 lines, shows lines 1 to 10, and a binding function calls `gband.win.set_cursor(win, 14)`
- **THEN** the content area shows lines 5 to 14, and its bottom row has the `PluginWindowCursorLine` style

#### Scenario: Fewer lines than rows
- **WHEN** a plugin window with a 10-row content area holds 3 lines and is scrolled by 5
- **THEN** it shows lines 1 to 3, and the other rows are blank

### Requirement: Focused plugin window
The client SHALL have at most one focused floating plugin window. The **focused plugin window** SHALL be the focused floating plugin window when there is one. Otherwise it SHALL be the plugin window whose drawn window is the focused window, when there is one, and otherwise no plugin window.

- Opening a floating plugin window with `focus` on SHALL make it the focused floating plugin window.
- `gband.win.focus(win)` on a floating plugin window SHALL make it the focused floating plugin window.
- `gband.win.focus(win)` on a tiled plugin window SHALL leave no focused floating plugin window, and dispatch focus of its drawn window as `gband.window.focus` does.
- A change of the focused window or of the viewed band SHALL leave no focused floating plugin window, except as the next rule allows.
- After the focused floating plugin window runs one of its `keys` functions, a change of the focused window or of the viewed band SHALL leave that floating plugin window focused until the user presses another key. This holds whether the change comes from the client at once or from the server later, as the focus of an opened window does.
- Closing the focused floating plugin window SHALL leave no focused floating plugin window.

#### Scenario: Floating plugin window takes focus
- **WHEN** window 1 is focused and a binding function opens a floating plugin window
- **THEN** the floating plugin window is the focused plugin window and `gband.view().window` is still 1

#### Scenario: Moving focus leaves the floating plugin window
- **WHEN** a floating plugin window is focused and the user presses Ctrl+Space then `l`, with two columns open and the first focused
- **THEN** the second column is focused, the floating plugin window is still drawn, and no floating plugin window is focused

#### Scenario: Action from the floating plugin window's own key
- **WHEN** a focused floating plugin window has `keys = { enter = function() gband.action.focus_column_right() end }`, two columns are open with the first focused, and the user presses Enter
- **THEN** the second column is focused and the floating plugin window is still the focused plugin window
- **AND** when the user then presses Ctrl+Space then `h`, the first column is focused and no floating plugin window is focused

### Requirement: Keys in a focused plugin window
While a plugin window is focused, every key that the key bindings would send to the focused window SHALL go to the plugin window instead, and SHALL NOT be sent to the server. Every paste SHALL be discarded. A key that matches an entry of the plugin window's `keys`, as a key matches a binding, SHALL run that entry's function with the plugin window's number, as a callback that belongs to the plugin window's plugin. A key that matches no entry SHALL take its default, when it has one:

| key | default |
|---|---|
| Up or `k`, Down or `j` | move the cursor line up or down one line with `cursorline` on, otherwise scroll by one line |
| PageUp, PageDown | scroll by the content area's height, and move the cursor line by as many lines with `cursorline` on |
| Home, End | show the first or last line, and make it the cursor line with `cursorline` on |
| Escape | close the plugin window when it is a floating plugin window |

Any other key SHALL be discarded.

#### Scenario: Key entry runs
- **WHEN** a focused floating plugin window has `keys = { enter = fn }` and `cursorline = true`, its cursor is on line 3, and the user presses Enter
- **THEN** `fn` runs with the plugin window's number, and `gband.win.info(win).cursor` is 3 inside it

#### Scenario: Default scroll
- **WHEN** a focused floating plugin window without `cursorline` shows lines 1 to 10 of 25 and the user presses Down
- **THEN** it shows lines 2 to 11

#### Scenario: j and k scroll like the arrow keys
- **WHEN** a focused floating plugin window without `cursorline` shows lines 1 to 10 of 25 and the user presses `j`, then `j`, then `k`
- **THEN** it shows lines 2 to 11, then lines 3 to 12, then lines 2 to 11

#### Scenario: Escape closes a floating plugin window
- **WHEN** a focused floating plugin window binds no `escape` and the user presses Escape
- **THEN** the floating plugin window closes and the focused window receives nothing

#### Scenario: Bindings still win
- **WHEN** a floating plugin window is focused and the user presses Ctrl+Space then `q`
- **THEN** the focused window closes and the floating plugin window receives no key

#### Scenario: Unmatched key
- **WHEN** a floating plugin window is focused and the user types `x`
- **THEN** nothing reaches the focused window

### Requirement: Floating plugin windows
A floating plugin window SHALL be drawn only by the client that opened it, and SHALL NOT change the layout, the view, the camera or the shown windows. Its box SHALL be placed in the ribbon area each time it is drawn:
- The width SHALL be at most the ribbon area's width, and the height at most its height.
- A `"center"` row or column SHALL be half the room left beside the box, rounded down.
- A row or column given as a number SHALL be cut so that the box ends inside the ribbon area.

A floating plugin window with `border` on SHALL draw a one-cell border around its content area in `PluginWindowBorder`'s resolved style. It SHALL draw its title, with control characters removed, on the top border from the box's second column, cut to the box's width less 2, in `PluginWindowTitle`'s resolved style applied over `PluginWindowBorder`'s.

`gband.win.set_config(win, config)` SHALL change the floating plugin window's `row`, `col`, `width`, `height`, `border` and `title` that `config` names, and keep the others. Calling it on a tiled plugin window SHALL be an error.

Floating plugin windows SHALL be drawn after the tiles and before a configuration error banner. They SHALL be stacked in the order they were last opened or focused, with the latest on top. A floating plugin window SHALL be drawn at rest during animations.

#### Scenario: Centered floating plugin window
- **WHEN** the ribbon area is 80×23 and a binding function opens a floating plugin window with width 40 and height 11
- **THEN** its box spans columns 20 to 59 and rows 6 to 16 of the ribbon area
- **AND** its content area is 38×9

#### Scenario: Floating plugin window cut to the ribbon area
- **WHEN** the ribbon area is 80×23 and a floating plugin window has `col = 70` and width 20
- **THEN** its box spans columns 60 to 79

#### Scenario: Resize a floating plugin window
- **WHEN** a floating plugin window's `on_resize` records its arguments and a binding function calls `gband.win.set_config(win, { width = 30, height = 12 })` on it, with border on
- **THEN** the floating plugin window's box is 30×12 and `on_resize` runs with 28 and 10

#### Scenario: Other clients do not see it
- **WHEN** two clients view the same band and the first opens a floating plugin window
- **THEN** the second client's screen is unchanged

### Requirement: Tiled plugin windows
Opening a plugin window of kind `"tiled"` SHALL dispatch open window naming plugin content, the band and the window to follow, the column width, and whether to focus the new window, as the session-server capability defines. The plugin window's `window` SHALL be nil until the server tells the client which window it opened. When the server reports that it opened none, the plugin window SHALL close.

The content area of a tiled plugin window SHALL be its drawn window's screen, at the size of the latest snapshot of that window. Whenever the plugin window's lines, `top`, `cursor`, the resolved styles of the groups it uses, or its screen's size change, the client SHALL send the server the contents of the whole content area, drawn as "Plugin window contents" defines, with the cursor hidden. When several such changes happen before the client sends, it MAY send only the latest contents.

A tiled plugin window SHALL be resized, moved and focused through its window, like any window. Its drawn window leaving the layout, for any reason, SHALL close the plugin window.

#### Scenario: Tiled plugin window on the strip
- **WHEN** the screen area is 80×24, window 1 is focused alone in its column, and a binding function calls `gband.win.open({ kind = "tiled", lines = { "hello" }, column_width = 1/4 })`
- **THEN** a column of width 1/4 holding a new window is inserted right of window 1's column, and the new window is focused
- **AND** the new window's tile shows `hello` on its first row, and its cursor is hidden

#### Scenario: Resize through the window
- **WHEN** a tiled plugin window's drawn window is window 3 and a binding function calls `gband.window.set_width(3, 1/2)`
- **THEN** window 3's column has width 1/2
- **AND** once the server resizes window 3, `on_resize` runs with its new size and the plugin window's lines are drawn again at that size

#### Scenario: Seen by another client
- **WHEN** two clients view the same band and the first opens a tiled plugin window with the line `hello`
- **THEN** the second client also shows a new tile reading `hello`

#### Scenario: Closed by the user
- **WHEN** a tiled plugin window is focused, its `on_close` records its argument, and the user presses Ctrl+Space then `q`
- **THEN** its drawn window leaves the layout and `on_close` runs with the plugin window's number

### Requirement: Plugin window callbacks
A plugin window's `keys` functions, `on_close` and `on_resize` SHALL run as callbacks that belong to the plugin window's plugin. `on_close` SHALL run once, with the plugin window's number, when the plugin window closes, except when it closes because of a reload or because the client is ending. `on_resize` SHALL run with the plugin window's number and its content area's columns and rows each time that size changes. For a tiled plugin window, that SHALL include the first time its size becomes known.

#### Scenario: Failing callback
- **WHEN** a plugin window's `keys` entry for `x` raises an error and the user presses `x` with the plugin window focused
- **THEN** the error is reported as a plugin error, and the plugin window stays open

### Requirement: Closing plugin windows
`gband.win.close(win)` SHALL close the plugin window. For a tiled plugin window, it SHALL also dispatch close window naming its drawn window, or close that window as soon as the server reports it when the window is not yet known. Closing a plugin window that is not open SHALL do nothing. A plugin window SHALL also close when the plugin it belongs to is marked failed. A reload SHALL close every plugin window and every drawn window this client opened, before the new configuration's `ConfigReloaded` handlers run.

#### Scenario: Close a floating plugin window
- **WHEN** a binding function closes the focused floating plugin window
- **THEN** the floating plugin window is no longer drawn and no floating plugin window is focused

#### Scenario: Close a tiled plugin window
- **WHEN** a binding function closes a tiled plugin window whose drawn window is window 3
- **THEN** window 3 leaves the layout

#### Scenario: Reload
- **WHEN** a floating plugin window and a tiled plugin window are open and the user saves `user/init.lua`
- **THEN** after the reload no plugin window is open, the drawn window has left the layout, and no `on_close` has run

### Requirement: Plugin window information
`gband.win.list()` SHALL return the numbers of the open plugin windows in ascending order. `gband.win.info(win)` SHALL return a new table holding:
- `id`, `kind` and `focused`.
- `window`: the drawn window's number, or nil.
- `top`, `cursor` and `line_count`.
- `cols` and `rows`: the content area's size, or nil while a tiled plugin window's size is unknown.
- For a floating plugin window, `row`, `col`, `width` and `height`: its box as last placed.

#### Scenario: Info of a floating plugin window
- **WHEN** the ribbon area is 80×23 and a binding function opens a floating plugin window with width 40, height 11 and three lines, then calls `gband.win.info` on it
- **THEN** the result holds `kind = "floating"`, `focused = true`, `top = 1`, `cursor = 1`, `line_count = 3`, `cols = 38`, `rows = 9`, `row = 6`, `col = 20`, `width = 40` and `height = 11`

### Requirement: Plugin window highlight groups
The plugin window API SHALL own these groups and their defaults, set in the defaults layer the highlights capability defines:

| group | default |
|---|---|
| `PluginWindow` | no fields |
| `PluginWindowBorder` | `{ fg = 8 }` |
| `PluginWindowTitle` | `{ bold = true }` |
| `PluginWindowCursorLine` | `{ reverse = true }` |

A change of any group's resolved style SHALL redraw every plugin window whose drawing uses it.

#### Scenario: Theme overrides the border
- **WHEN** `user/init.lua` calls `gband.hl.set("PluginWindowBorder", { fg = "#ff0000" })` and a floating plugin window with a border is drawn on a truecolor terminal
- **THEN** its border has foreground `#ff0000`
