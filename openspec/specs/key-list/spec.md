# key-list Specification

## Purpose
Defines the key list: a bundled client plugin that shows the `prefix` table's bindings in a floating plugin window and runs the one the user chooses.

## Requirements

### Requirement: Key list plugin
gband SHALL bundle the client plugin module `gband.keylist`, whose plugin name is `keylist`. Its `setup` SHALL take no options, and an options table holding any field SHALL make it raise an error naming the field. `setup` SHALL register the action `keylist.open` with the description `list the keys`. Dispatching `keylist.open` SHALL open the key list, as "Key list floating plugin window" defines. Dispatching it while the key list is already open SHALL focus that key list, as `gband.win.focus` does, and SHALL open no second one. It SHALL then make `root` the active table, as opening does.

#### Scenario: Action registered
- **WHEN** a configuration calls `gband.plugin("gband.keylist")` and reads `gband.action.list()`
- **THEN** it holds an entry named `keylist.open` with the description `list the keys`

#### Scenario: Unknown option
- **WHEN** a configuration calls `gband.plugin("gband.keylist", { table = "root" })`
- **THEN** the plugin `keylist` reports an error naming `table`

#### Scenario: Open again while open
- **WHEN** the default configuration is in use, the key list is open, the user presses Ctrl+Space then `l`, which focuses another column and leaves the key list unfocused, and then presses `?` while navigation mode is still active
- **THEN** exactly one key list is drawn and it has focus
- **AND** `root` is the active table, so a further `q` closes it

### Requirement: Closing the key list
Escape SHALL close the key list while it is focused, as the plugin-windows capability defines for a floating plugin window with no `keys` entry for it. When the `prefix` table binds `q`, `q` SHALL run that line, as "Running a line by its key" defines, so the default binding to close window closes the key list. When the `prefix` table does not bind `q`, `q` SHALL close the key list, as the plugin-windows capability defines for a floating plugin window with no `keys` entry for it. Ctrl+Space then `q` SHALL close it too while it is focused, as close window closes the focused floating plugin window, including on a band with no window.

#### Scenario: q closes the list
- **WHEN** the default configuration is in use, the key list is open, and the user presses `q`
- **THEN** the key list closes, every window stays open, and the focused window receives nothing

#### Scenario: q with no q binding
- **WHEN** `user/init.lua` sets up `gband.keylist`, binds `prefix ?` to `gband.action["keylist.open"]` and binds no `prefix q`, the key list is open, and the user presses `q`
- **THEN** the key list closes

#### Scenario: Escape closes the list
- **WHEN** the key list is open and the user presses Escape
- **THEN** the key list closes

#### Scenario: Prefix q on the empty band
- **WHEN** the viewed band holds no window, the key list is open, and the user presses Ctrl+Space then `q`
- **THEN** the key list closes and nothing is sent to the server

### Requirement: Key list floating plugin window
Opening the key list SHALL open a floating plugin window, as the plugin-windows capability defines, that belongs to the plugin `keylist` and takes focus. It SHALL also make `root` the active table, as `gband.keymap.enter("root")` does, so the keys that follow reach the floating plugin window rather than a mode. The floating plugin window SHALL have a border, the title `gband.keymap.label("prefix")` followed by ` keys`, and its cursor line on, starting on the first line, and SHALL be centered in the ribbon area.

The floating plugin window SHALL hold one line for each binding that `gband.keymap.list("prefix")` returns when it opens, in that order. A line SHALL show the binding's key in the form the key-hints capability's "Key form" defines, so the binding `prefix` shows as the prefix key, such as `C-space`. The key SHALL be in the group `KeyListKey` and padded with spaces to two cells more than the widest key of the list. The key SHALL be followed by the binding's description. A binding with no description SHALL show its action's description, as `gband.action.list()` gives it, or the action's name when that is empty too, and a function binding with no description SHALL show `function`. The description SHALL be in the group `PluginWindow` when Enter can run the binding, as "Running any binding" defines, and in `KeyListMuted` otherwise.

The floating plugin window's width SHALL be its longest line plus 2 for the border, and at most the ribbon area's width. Its height SHALL be its line count plus 2, at least 3, and at most 15 and the ribbon area's height.

The plugin SHALL define the groups `KeyListKey`, linked to `StatusLineAccent`, and `KeyListMuted`, linked to `StatusLineMuted`, as defaults. It SHALL also give `StatusLineAccent` and `StatusLineMuted` the defaults the status-line capability gives them, as defaults, so the two links resolve when no status line module has loaded.

#### Scenario: Default list
- **WHEN** the default configuration is in use and the user presses Ctrl+Space then `?`
- **THEN** a focused floating plugin window titled `navigation keys` shows `h` and `focus the column to the left` on its first line, with the cursor line there
- **AND** a later line shows `q` and `close the window`
- **AND** the line for the prefix key shows `C-space`, and no description is cut on a terminal of 80 columns
- **AND** `root` is the active table

#### Scenario: Title of a prefix table that is not a mode
- **WHEN** `user/init.lua` binds `prefix ?` to `gband.action["keylist.open"]`, sets up `gband.keylist`, declares no mode, and the user presses Ctrl+Space then `?`
- **THEN** the floating plugin window's title is `prefix keys`

#### Scenario: Function binding can run
- **WHEN** `user/init.lua` sets up `gband.keylist`, binds `prefix ?` to `gband.action["keylist.open"]` and `prefix x` to a function with the description `say hi`, and the user presses Ctrl+Space then `?`
- **THEN** the line for `x` shows `say hi` in `PluginWindow`, and the line for `?` shows `list the keys` in `KeyListMuted`

#### Scenario: Only the key list's own line is muted
- **WHEN** the default configuration is in use and the key list opens
- **THEN** the line for `?` is the only line whose description is in `KeyListMuted`

#### Scenario: Muted without a status line
- **WHEN** `user/init.lua` sets up `gband.keylist` and no status line module, and the key list opens
- **THEN** the muted lines are drawn dim and the runnable lines are not

#### Scenario: Moving through the list
- **WHEN** the key list is open on its first line and the user presses Down, Down, then Up
- **THEN** the cursor line is on the second line

### Requirement: Running any binding
Enter SHALL run the binding on the cursor line, unless it binds `keylist.open`, as `gband.keymap.run("prefix", key)` runs it with the line's key. A binding to an action SHALL dispatch that action with no target, as calling its value in `gband.action` does. A binding to a function SHALL run the function. An action resolved against the view SHALL act on the focused window behind the key list, as its description says, except close window, which closes the key list, as the actions capability defines. Nothing SHALL reach the focused window. Enter on the binding of `keylist.open` SHALL do nothing.

The key list SHALL stay open. As the plugin-windows capability defines for a floating plugin window's own key, it SHALL stay focused through any change of the focused window or the viewed band that the binding causes. A floating plugin window that the binding opens with focus SHALL take focus, as the plugin-windows capability defines, and the key list SHALL stay open behind it.

#### Scenario: Run a focus action
- **WHEN** two columns are open with the second focused, the key list is open, and the user presses Enter on the line for `h`
- **THEN** the first column is focused
- **AND** the key list is still drawn and focused, so a further Down moves its cursor line

#### Scenario: Run a resize from the list
- **WHEN** a column holds two windows with automatic heights, the top one focused, the key list is open, and the user presses Enter on the line for `+`
- **THEN** the top window grows and the key list stays open and focused

#### Scenario: Run close from the list
- **WHEN** the key list is open and the user moves to the line for `q` and presses Enter
- **THEN** the key list closes and every window stays open

#### Scenario: Run an action that opens a floating plugin window
- **WHEN** a plugin registers an action that opens a focused floating plugin window, `prefix e` binds it, the key list is open, and the user presses Enter on the line for `e`
- **THEN** the new floating plugin window has focus, the key list is still drawn, and `q` closes the new floating plugin window

#### Scenario: Run a function binding
- **WHEN** the default configuration is in use, one window is open, the key list is open, and the user moves to the line for `n` and presses Enter
- **THEN** a second window opens
- **AND** the key list stays open and focused, and the focused window receives nothing

#### Scenario: The key list's own line does nothing
- **WHEN** the cursor line is on the binding of `keylist.open` and the user presses Enter
- **THEN** nothing is dispatched and the key list stays open

### Requirement: Running a line by its key
While the key list is focused, a key that matches the key of one of its lines, as a key matches a binding, SHALL move the cursor line to that line and then run that line exactly as Enter on it does, as "Running any binding" defines. The key list SHALL stay open and focused on the same terms, a binding that closes the focused plugin window SHALL close the key list, and the line of `keylist.open` SHALL only move the cursor line.

The key list's own keys SHALL be Up, Down, PageUp, PageDown, Home and End, which move the cursor line as the plugin-windows capability defines, Enter, which runs the cursor line, and Escape, which closes the key list. An own key SHALL keep its meaning when a line shows it, so such a line runs only through Enter. The line of the prefix key SHALL also run only through Enter, because the prefix key enters the `prefix` table before the key list receives it.

Every other key a line shows SHALL run that line, even when the plugin-windows capability gives the key a default. So `j` and `k` run their lines when the `prefix` table binds them, and the key list SHALL offer no other key in their place for moving the cursor line. A key that no line shows SHALL take its plugin-windows default, when it has one.

#### Scenario: A line's key runs it
- **WHEN** the default configuration is in use, two columns are open with the first focused, the key list is open on its first line, and the user presses `l`
- **THEN** the second column is focused and the cursor line is on the line for `l`
- **AND** the key list is still drawn and focused

#### Scenario: j runs its binding
- **WHEN** the default configuration is in use, a column holds two windows with the top one focused, the key list is open, and the user presses `j`
- **THEN** the bottom window is focused, the cursor line is on the line for `j`, and the key list stays open and focused

#### Scenario: A function binding by its key
- **WHEN** the default configuration is in use, one window is open, the key list is open on its first line, and the user presses `n`
- **THEN** a second window opens and the cursor line is on the line for `n`
- **AND** the key list stays open and focused, and the focused window receives nothing

#### Scenario: The key list's own line by its key
- **WHEN** the default configuration is in use, the key list is open on its first line, and the user presses `?`
- **THEN** the cursor line is on the line for `?`, nothing is dispatched, and exactly one key list is drawn

#### Scenario: Arrows move the cursor line
- **WHEN** the default configuration is in use, two windows are open, the key list is open on its first line, and the user presses Down, Down, then Up
- **THEN** the cursor line is on the second line and the focused window has not changed

#### Scenario: A line under an own key runs only through Enter
- **WHEN** `user/init.lua` sets up `gband.keylist`, binds `prefix ?` to `gband.action["keylist.open"]` and `prefix home` to `gband.action.focus_column_left`, two columns are open with the second focused, the key list is open on its last line, and the user presses Home
- **THEN** the cursor line is on the first line and the second column is still focused
- **AND** moving the cursor line to the line for `home` and pressing Enter focuses the first column

#### Scenario: An unbound j moves the cursor line
- **WHEN** `user/init.lua` sets up `gband.keylist`, binds `prefix ?` to `gband.action["keylist.open"]`, `prefix h` and `prefix l`, binds no `prefix j`, the key list is open on its first line, and the user presses `j`
- **THEN** the cursor line is on the second line

### Requirement: Mouse bindings in the key list
The key list SHALL hold no line for a binding whose key is a mouse name, as the configuration capability defines mouse names. The order of the other lines SHALL be unchanged.

#### Scenario: Default key list
- **WHEN** the default configuration is in use and the user opens the key list
- **THEN** no line shows `leftmouse`, `rightmouse` or `middlemouse`
