## MODIFIED Requirements

### Requirement: Key list plugin window
Opening the key list SHALL open a floating plugin window, as the plugin-windows capability defines, that belongs to the plugin `keylist` and takes focus. It SHALL also make `root` the active table, as `gband.keymap.enter("root")` does, so the keys that follow reach the floating plugin window rather than a mode. The floating plugin window SHALL have a border, the title `gband.keymap.label("prefix")` followed by ` keys`, a width of 50 and a height of 15, centered in the ribbon area, and its cursor line on, starting on the first line.

The floating plugin window SHALL hold one line for each binding that `gband.keymap.list("prefix")` returns when it opens, in that order. A line SHALL show the binding's key, as `gband.keymap.list` writes it, padded with spaces to 10 cells, in the group `KeyListKey`, followed by the binding's description, or the name of its action when it has no description, or `function` for a function binding with no description. The description SHALL be in the group `PluginWindow` when Enter can run the binding, as "Running a binding" defines, and in `KeyListMuted` otherwise.

The plugin SHALL define the groups `KeyListKey`, linked to `StatusLineAccent`, and `KeyListMuted`, linked to `StatusLineMuted`, as defaults.

#### Scenario: Default list
- **WHEN** the default configuration is in use and the user presses Ctrl+Space then `?`
- **THEN** a focused floating plugin window titled `navigation keys` shows `h` and `focus the column to the left` on its first line, with the cursor line there
- **AND** a later line shows `q` and `close the window`
- **AND** `root` is the active table

#### Scenario: Title of a prefix table that is not a mode
- **WHEN** `user/init.lua` binds `prefix ?` to `gband.action["keylist.open"]`, sets up `gband.keylist`, declares no mode, and the user presses Ctrl+Space then `?`
- **THEN** the floating plugin window's title is `prefix keys`

#### Scenario: Function binding can run
- **WHEN** `user/init.lua` binds `prefix x` to a function with the description `say hi`, and the key list opens
- **THEN** the line for `x` shows `say hi` in `PluginWindow`

#### Scenario: Only the key list's own line is muted
- **WHEN** the default configuration is in use and the key list opens
- **THEN** the line for `?` is the only line whose description is in `KeyListMuted`

#### Scenario: Moving through the list
- **WHEN** the key list is open on its first line and the user presses `j`, Down, then `k`
- **THEN** the cursor line is on the second line

### Requirement: Running a binding
Enter SHALL run the binding on the cursor line, unless it binds `keylist.open`, as `gband.keymap.run("prefix", key)` runs it with the line's key: a binding to an action SHALL dispatch that action with no target, and a binding to a function SHALL run the function. Nothing SHALL reach the focused window. The key list SHALL stay open, and, as the plugin-windows capability defines for a floating plugin window's own key, stay focused through any change of focus that the binding causes. Enter on the binding of `keylist.open` SHALL do nothing. A binding that closes the focused plugin window, such as one dispatching `close_window`, SHALL close the key list.

#### Scenario: Run a focus action
- **WHEN** two columns are open with the second focused, the key list is open, and the user presses Enter on the line for `h`
- **THEN** the first column is focused
- **AND** the key list is still drawn and focused, so a further `j` moves its cursor line

#### Scenario: Run close from the list
- **WHEN** the key list is open and the user moves to the line for `q` and presses Enter
- **THEN** the key list closes and every window stays open

#### Scenario: Run a function binding
- **WHEN** the default configuration is in use, one window is open, the key list is open, and the user moves to the line for `n` and presses Enter
- **THEN** a second window opens
- **AND** the key list stays open and focused, and the focused window receives nothing

#### Scenario: The key list's own line does nothing
- **WHEN** the cursor line is on the binding of `keylist.open` and the user presses Enter
- **THEN** nothing is dispatched and the key list stays open
