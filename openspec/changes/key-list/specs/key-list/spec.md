## Purpose

Defines the key list: a bundled client plugin that shows the `prefix` table's bindings in a float and runs the one the user chooses.

## ADDED Requirements

### Requirement: Key list plugin
gband SHALL bundle the client plugin module `gband.keylist`, whose plugin name is `keylist`. Its `setup` SHALL take no options, and an options table holding any field SHALL make it raise an error naming the field. `setup` SHALL register the action `keylist.open` with the description `list the keys`. Dispatching `keylist.open` SHALL open the key list, as "Key list window" defines.

#### Scenario: Action registered
- **WHEN** a configuration calls `gband.plugin("gband.keylist")` and reads `gband.action.list()`
- **THEN** it holds an entry named `keylist.open` with the description `list the keys`

#### Scenario: Unknown option
- **WHEN** a configuration calls `gband.plugin("gband.keylist", { table = "root" })`
- **THEN** the plugin `keylist` reports an error naming `table`

### Requirement: Key list window
Opening the key list SHALL open a float, as the plugin-windows capability defines, that belongs to the plugin `keylist` and takes focus. The float SHALL have a border, the title `prefix keys`, a width of 50 and a height of 15, centered in the ribbon area, and its cursor line on, starting on the first line.

The float SHALL hold one line for each binding that `gband.keymap.list("prefix")` returns when it opens, in that order. A line SHALL show the binding's key, as `gband.keymap.list` writes it, padded with spaces to 10 cells, in the group `KeyListKey`, followed by the binding's description, or the name of its action when it has no description, or `function` for a function binding with no description. The description SHALL be in the group `Window` when Enter can run the binding, as "Running a binding" defines, and in `KeyListMuted` otherwise.

The plugin SHALL define the groups `KeyListKey`, linked to `StatusLineAccent`, and `KeyListMuted`, linked to `StatusLineMuted`, as defaults.

#### Scenario: Default list
- **WHEN** the default configuration is in use and the user presses Ctrl+Space then `?`
- **THEN** a focused float titled `prefix keys` shows `h` and `focus the column to the left` on its first line, with the cursor line there
- **AND** a later line shows `q` and `close the window`

#### Scenario: Function binding is muted
- **WHEN** `user/init.lua` binds `prefix x` to a function with the description `say hi`, and the key list opens
- **THEN** the line for `x` shows `say hi` in `KeyListMuted`

#### Scenario: Moving through the list
- **WHEN** the key list is open on its first line and the user presses `j`, Down, then `k`
- **THEN** the cursor line is on the second line

### Requirement: Running a binding
Enter SHALL run the binding on the cursor line when it binds an action other than `keylist.open`: it SHALL dispatch that action, as calling its value in `gband.action` does, with no target. The key list SHALL stay open, and, as the plugin-windows capability defines for a float's own key, stay focused through any change of focus that action causes. Enter on a function binding or on the binding of `keylist.open` SHALL do nothing. A dispatched action that closes the focused float, such as `close_pane` resolved against the view, SHALL close the key list.

#### Scenario: Run a focus action
- **WHEN** two columns are open with the second focused, the key list is open, and the user presses Enter on the line for `h`
- **THEN** the first column is focused
- **AND** the key list is still drawn and focused, so a further `j` moves its cursor line

#### Scenario: Run close from the list
- **WHEN** the key list is open and the user moves to the line for `q` and presses Enter
- **THEN** the key list closes and every pane stays open

#### Scenario: Function binding does nothing
- **WHEN** the cursor line is on a function binding and the user presses Enter
- **THEN** nothing is dispatched and the key list stays open

### Requirement: Closing the key list
`q` and Escape SHALL close the key list, as the plugin-windows capability defines for a float with no `keys` entry for them. Ctrl+Space then `q` SHALL close it too, as close pane closes the focused float.

#### Scenario: q closes the list
- **WHEN** the key list is open and the user presses `q`
- **THEN** the key list closes and the focused pane receives nothing

#### Scenario: Escape closes the list
- **WHEN** the key list is open and the user presses Escape
- **THEN** the key list closes
