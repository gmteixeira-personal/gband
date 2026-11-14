## MODIFIED Requirements

### Requirement: Key list window
Opening the key list SHALL open a floating plugin window, as the plugin-windows capability defines, that belongs to the plugin `keylist` and takes focus. It SHALL also make `root` the active table, as `gband.keymap.enter("root")` does, so the keys that follow reach the floating plugin window rather than a mode. The listed table SHALL be the table that `gband.keymap.current_table()` returns just before the key list makes `root` active, or `prefix` when that table is `root`. The floating plugin window SHALL have a border, the title `gband.keymap.label(t)` followed by ` keys`, where `t` is the listed table, and its cursor line on, starting on the first line, and SHALL be centered in the ribbon area. Every other requirement of this capability that names the `prefix` table SHALL mean the listed table of the open key list.

The floating plugin window SHALL hold one line for each binding that `gband.keymap.list(t)` returns when it opens, in that order. A line SHALL show the binding's key in the form "Key form" defines, so the binding `prefix` shows as the prefix key, such as `C-space`. The key SHALL be in the group `KeyListKey` and padded with spaces to two cells more than the widest key of the list. The key SHALL be followed by the binding's description. A binding with no description SHALL show its action's description, as `gband.action.list()` gives it, or the action's name when that is empty too, and a function binding with no description SHALL show `function`. The description SHALL be in the group `PluginWindow` when Enter can run the binding, as "Running any binding" defines, and in `KeyListMuted` otherwise.

The floating plugin window's width SHALL be its longest line plus 2 for the border, and at most the ribbon area's width. Its height SHALL be its line count plus 2, at least 3, and at most 15 and the ribbon area's height.

The plugin SHALL define the groups `KeyListKey`, with the style `{ bold = true }`, and `KeyListMuted`, with the style `{ dim = true }`, as defaults.

#### Scenario: Default list
- **WHEN** the default configuration is in use and the user presses Ctrl+Space then `?`
- **THEN** a focused floating plugin window titled `navigation keys` shows `h` and `focus the column to the left` on its first line, with the cursor line there
- **AND** a later line shows `q` and `close the window`
- **AND** the line for the prefix key shows `C-space`, and no description is cut on a terminal of 80 columns
- **AND** `root` is the active table

#### Scenario: Title of a prefix table that is not a mode
- **WHEN** `user/init.lua` binds `prefix ?` to `gband.action["keylist.open"]`, sets up `gband.keylist`, declares no mode, and the user presses Ctrl+Space then `?`
- **THEN** the floating plugin window's title is `prefix keys`

#### Scenario: Listed table from direct navigation mode
- **WHEN** the direct key style is saved, no `user/init.lua` exists, and the user presses Ctrl+Space, Space, then `?`
- **THEN** a focused floating plugin window titled `navigation keys` shows the bindings of `gband.keymap.list("navigation")`, with lines for Escape and Enter and no line for Space
- **AND** `root` is the active table

#### Scenario: Listed table after the direct prefix key
- **WHEN** the direct key style is saved, no `user/init.lua` exists, and the user presses Ctrl+Space then `?`
- **THEN** the floating plugin window's title is `prefix keys` and it holds a line for Space with the description `navigation mode`

#### Scenario: Function binding can run
- **WHEN** `user/init.lua` sets up `gband.keylist`, binds `prefix ?` to `gband.action["keylist.open"]` and `prefix x` to a function with the description `say hi`, and the user presses Ctrl+Space then `?`
- **THEN** the line for `x` shows `say hi` in `PluginWindow`, and the line for `?` shows `list the keys` in `KeyListMuted`

#### Scenario: Only the key list's own line is muted
- **WHEN** the default configuration is in use and the key list opens
- **THEN** the line for `?` is the only line whose description is in `KeyListMuted`

#### Scenario: Muted without colorscheme settings
- **WHEN** `user/init.lua` sets up `gband.keylist`, nothing sets `KeyListMuted` or `KeyListKey`, and the key list opens
- **THEN** the muted lines are drawn dim and the runnable lines are not

#### Scenario: Moving through the list
- **WHEN** the key list is open on its first line and the user presses Down, Down, then Up
- **THEN** the cursor line is on the second line

