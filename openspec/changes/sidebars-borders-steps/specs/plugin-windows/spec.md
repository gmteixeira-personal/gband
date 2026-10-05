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
| `on_close` | both | function | none |
| `on_resize` | both | function | none |
| `row`, `col` | floating | an integer of at least 0, or `"center"` | `"center"` |
| `width`, `height` | floating | an integer of at least 1 | half the ribbon area's width or height, rounded down, and at least 1 |
| `border` | floating | a boolean, or a border table as the borders capability defines it | `true` |
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

#### Scenario: Invalid border table
- **WHEN** a binding function calls `gband.win.open({ border = { sides = { "middle" } } })`
- **THEN** the call raises an error naming `middle`

### Requirement: Floating plugin windows
A floating plugin window SHALL be drawn only by the client that opened it, and SHALL NOT change the layout, the view, the camera or the shown windows. Its box SHALL be placed in the ribbon area each time it is drawn:
- The width SHALL be at most the ribbon area's width, and the height at most its height.
- A `"center"` row or column SHALL be half the room left beside the box, rounded down.
- A row or column given as a number SHALL be cut so that the box ends inside the ribbon area.

A floating plugin window whose `border` is `true` or a border table has its border on. It SHALL draw a one-cell border around its content area, with the sides and characters the borders capability gives its `border`, in `PluginWindowBorder`'s resolved style. The title SHALL be drawn whether or not the top side is drawn. It SHALL draw its title, with control characters removed, on the top border from the box's second column, cut to the box's width less 2, in `PluginWindowTitle`'s resolved style applied over `PluginWindowBorder`'s.

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

#### Scenario: Title over an undrawn top side
- **WHEN** a binding function opens a floating plugin window of width 20 with `title = "list"` and `border = { sides = { "left", "right" } }`
- **THEN** the box's top row is blank except `list` from its second column
