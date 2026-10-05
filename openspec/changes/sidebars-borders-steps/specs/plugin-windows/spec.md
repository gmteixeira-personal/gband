## MODIFIED Requirements

### Requirement: Open a window
`gband.win.open(opts)` SHALL open a window and return its number. The number is a positive integer, unique within the client process and never reused. The window SHALL belong to the plugin whose code opened it. `opts` SHALL be a table with these optional fields:

| field | kinds | value | default |
|---|---|---|---|
| `kind` | both | `"float"` or `"pane"` | `"float"` |
| `lines` | both | the window's lines, as "Window contents" defines | no lines |
| `focus` | both | boolean | `true` |
| `cursorline` | both | boolean | `false` |
| `keys` | both | a table from key names to functions | empty |
| `on_close` | both | function | none |
| `on_resize` | both | function | none |
| `row`, `col` | float | an integer of at least 0, or `"center"` | `"center"` |
| `width`, `height` | float | an integer of at least 1 | half the ribbon area's width or height, rounded down, and at least 1 |
| `border` | float | a boolean, or a border table as the borders capability defines it | `true` |
| `title` | float | string | none |
| `band`, `after` | pane | band and pane numbers, as the `open_pane` target takes them | with neither given, the viewed band and the focused pane, as open pane resolves against the view |
| `column_width` | pane | a width, as `gband.pane.set_width` takes it | the `default_column_width` option |

A field for the other kind SHALL be an error. A key name in `keys` SHALL be valid as the configuration capability defines key names.

#### Scenario: Numbers are not reused
- **WHEN** a binding function opens a float, closes it, and opens another
- **THEN** the second window's number differs from the first's

#### Scenario: Field of the other kind
- **WHEN** a binding function calls `gband.win.open({ kind = "pane", row = 2 })`
- **THEN** the call raises an error naming `row`

#### Scenario: Invalid key name
- **WHEN** a binding function opens a window with `keys = { ["ctrl+shift+1"] = fn }`
- **THEN** the call raises an error naming `ctrl+shift+1`

#### Scenario: Invalid border table
- **WHEN** a binding function calls `gband.win.open({ border = { sides = { "middle" } } })`
- **THEN** the call raises an error naming `middle`

### Requirement: Floats
A float SHALL be drawn only by the client that opened it, and SHALL NOT change the layout, the view, the camera or the shown panes. Its box SHALL be placed in the ribbon area each time it is drawn:
- The width SHALL be at most the ribbon area's width, and the height at most its height.
- A `"center"` row or column SHALL be half the room left beside the box, rounded down.
- A row or column given as a number SHALL be cut so that the box ends inside the ribbon area.

A float whose `border` is `true` or a border table has its border on. It SHALL draw a one-cell border around its content area, with the sides and characters the borders capability gives its `border`, in `WindowBorder`'s resolved style. The title SHALL be drawn whether or not the top side is drawn. It SHALL draw its title, with control characters removed, on the top border from the box's second column, cut to the box's width less 2, in `WindowTitle`'s resolved style applied over `WindowBorder`'s.

`gband.win.set_config(win, config)` SHALL change the float's `row`, `col`, `width`, `height`, `border` and `title` that `config` names, and keep the others. Calling it on a pane window SHALL be an error.

Floats SHALL be drawn after the tiles and before a configuration error banner. They SHALL be stacked in the order they were last opened or focused, with the latest on top. A float SHALL be drawn at rest during animations.

#### Scenario: Centered float
- **WHEN** the ribbon area is 80×23 and a binding function opens a float with width 40 and height 11
- **THEN** its box spans columns 20 to 59 and rows 6 to 16 of the ribbon area
- **AND** its content area is 38×9

#### Scenario: Float cut to the ribbon area
- **WHEN** the ribbon area is 80×23 and a float has `col = 70` and width 20
- **THEN** its box spans columns 60 to 79

#### Scenario: Resize a float
- **WHEN** a float's `on_resize` records its arguments and a binding function calls `gband.win.set_config(win, { width = 30, height = 12 })` on it, with border on
- **THEN** the float's box is 30×12 and `on_resize` runs with 28 and 10

#### Scenario: Other clients do not see it
- **WHEN** two clients view the same band and the first opens a float
- **THEN** the second client's screen is unchanged

#### Scenario: Title over an undrawn top side
- **WHEN** a binding function opens a float of width 20 with `title = "list"` and `border = { sides = { "left", "right" } }`
- **THEN** the box's top row is blank except `list` from its second column
