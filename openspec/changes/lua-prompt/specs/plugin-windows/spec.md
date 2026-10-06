## MODIFIED Requirements

### Requirement: Open a plugin window
`gband.win.open(opts)` SHALL open a plugin window and return its number. The number is a positive integer, unique within the client process and never reused. The plugin window SHALL belong to the plugin whose code opened it. `opts` SHALL be a table with these optional fields:

| field | kinds | value | default |
|---|---|---|---|
| `kind` | both | `"floating"` or `"tiled"` | `"floating"` |
| `lines` | both | the plugin window's lines, as "Plugin window contents" defines | no lines |
| `focus` | both | boolean | `true` |
| `cursorline` | both | boolean | `false` |
| `keys` | both | a table from key names to functions | empty |
| `on_input` | both | function, as "Keys in a focused plugin window" defines | none |
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

#### Scenario: Input handler of the wrong type
- **WHEN** a binding function calls `gband.win.open({ on_input = "text" })`
- **THEN** the call raises an error naming `on_input`

### Requirement: Keys in a focused plugin window
While a plugin window is focused, every key that the key bindings would send to the focused window SHALL go to the plugin window instead, and SHALL NOT be sent to the server. Every paste SHALL go to the plugin window too, and SHALL NOT be sent to the server.

A key **types a character** when it is a character key pressed without Ctrl and without Alt. Its text SHALL be that character, as the terminal reports it, so Shift+A gives `A` and the space bar gives a space.

A key SHALL be handled by the first of these that applies:
1. A key that matches an entry of the plugin window's `keys`, as a key matches a binding, SHALL run that entry's function with the plugin window's number, as a callback that belongs to the plugin window's plugin.
2. A key that types a character, when the plugin window has `on_input`, SHALL run `on_input` with the plugin window's number and the key's text, as a callback that belongs to the plugin window's plugin.
3. A key that has a default in this table SHALL take it:

| key | default |
|---|---|
| Up or `k`, Down or `j` | move the cursor line up or down one line with `cursorline` on, otherwise scroll by one line |
| PageUp, PageDown | scroll by the content area's height, and move the cursor line by as many lines with `cursorline` on |
| Home, End | show the first or last line, and make it the cursor line with `cursorline` on |
| `q`, Escape | close the plugin window when it is a floating plugin window |

Any other key SHALL be discarded.

A paste SHALL run `on_input` with the plugin window's number and the pasted text, unchanged, line breaks and other control characters included, as a callback that belongs to the plugin window's plugin. A paste into a plugin window without `on_input` SHALL be discarded.

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
- **WHEN** a floating plugin window with `keys = { h = fn }` is focused over the second of two columns and the user presses Ctrl+Space then `h`
- **THEN** the first column is focused and `fn` does not run

#### Scenario: q closes a floating plugin window
- **WHEN** a focused floating plugin window binds no `q`, has no `on_input`, and the user presses `q`
- **THEN** the floating plugin window closes, its `on_close` runs, and the focused window receives nothing

#### Scenario: A keys entry for q wins
- **WHEN** a focused floating plugin window has `keys = { q = fn }` and the user presses `q`
- **THEN** `fn` runs and the floating plugin window stays open

#### Scenario: Prefix q closes the focused floating plugin window
- **WHEN** a floating plugin window is focused over window 1 and the user presses Ctrl+Space then `q`
- **THEN** the floating plugin window closes, window 1 stays open and focused, and the floating plugin window's `keys` functions do not run

#### Scenario: q in a tiled plugin window
- **WHEN** a tiled plugin window is focused, binds no `q`, has no `on_input`, and the user presses `q`
- **THEN** the key is discarded and the tiled plugin window stays open

#### Scenario: Unmatched key
- **WHEN** a floating plugin window without `on_input` is focused and the user types `x`
- **THEN** nothing reaches the focused window

#### Scenario: Typed characters go to on_input
- **WHEN** a focused floating plugin window without `cursorline` shows lines 1 to 10 of 25, its `on_input` records its arguments, and the user types `j`, `q`, Shift+A and a space
- **THEN** `on_input` runs four times, with the plugin window's number and `j`, `q`, `A` and a space
- **AND** the floating plugin window stays open and still shows lines 1 to 10

#### Scenario: A keys entry wins over on_input
- **WHEN** a focused floating plugin window has `keys = { j = fn }` and `on_input`, and the user types `j`
- **THEN** `fn` runs and `on_input` does not

#### Scenario: Keys without text keep their defaults
- **WHEN** a focused floating plugin window without `cursorline` has `on_input`, shows lines 1 to 10 of 25, and the user presses Down, then Escape
- **THEN** it shows lines 2 to 11, then closes, and `on_input` never runs

#### Scenario: Ctrl and Alt keys are not text
- **WHEN** a focused floating plugin window has `on_input` and the user presses Ctrl+X, then Alt+X
- **THEN** `on_input` does not run and nothing reaches the focused window

#### Scenario: Paste goes to on_input
- **WHEN** a focused floating plugin window's `on_input` records its arguments and the user pastes `a\nb`
- **THEN** `on_input` runs once, with the plugin window's number and `a\nb`
- **AND** nothing is sent to the server

#### Scenario: Paste without on_input
- **WHEN** a focused floating plugin window has no `on_input` and the user pastes `hello`
- **THEN** the paste is discarded and nothing is sent to the server

### Requirement: Plugin window callbacks
A plugin window's `keys` functions, `on_input`, `on_close` and `on_resize` SHALL run as callbacks that belong to the plugin window's plugin. `on_input` SHALL run with the plugin window's number and a text, as "Keys in a focused plugin window" defines. `on_close` SHALL run once, with the plugin window's number, when the plugin window closes, except when it closes because of a reload or because the client is ending. `on_resize` SHALL run with the plugin window's number and its content area's columns and rows each time that size changes. For a tiled plugin window, that SHALL include the first time its size becomes known.

#### Scenario: Failing callback
- **WHEN** a plugin window's `keys` entry for `x` raises an error and the user presses `x` with the plugin window focused
- **THEN** the error is reported as a plugin error, and the plugin window stays open

#### Scenario: Failing input handler
- **WHEN** a plugin window's `on_input` raises an error and the user types `a` with the plugin window focused
- **THEN** the error is reported as a plugin error, and the plugin window stays open
- **AND** typing `b` runs `on_input` again
