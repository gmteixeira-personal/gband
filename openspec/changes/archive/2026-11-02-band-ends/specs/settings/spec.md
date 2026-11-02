## ADDED Requirements

### Requirement: Saved band ends
The saved band ends setting SHALL live in the file `user/band_ends.lua` of the configuration directory. Saving it SHALL write `return "loop"`, `return "bounce"` or `return "stop"`, followed by a newline, to that file, creating or replacing it in one step, so no process reads it half written.

`gband.settings.band_ends()` SHALL evaluate `user/band_ends.lua` as "Saved theme" defines for `user/theme.lua`. It SHALL return the chunk's value when that value is the string `loop`, `bounce` or `stop`, and nil in every case where `gband.settings.theme()` returns nil. It SHALL raise no error and report none. It SHALL be callable while the configuration loads and in any callback. Loading SHALL evaluate `user/band_ends.lua` only in this way, never as a configuration file, a plugin file or a module.

The default configuration SHALL set the `band_ends` option to the value `gband.settings.band_ends()` returns, and to `"loop"` when it returns nil, as the configuration capability defines. A configuration file that sets `band_ends` itself SHALL keep its own value, whatever is saved.

#### Scenario: Saved file
- **WHEN** the settings window saves `bounce` on its `bands` line
- **THEN** `user/band_ends.lua` holds `return "bounce"` and a newline

#### Scenario: Saved value in use
- **WHEN** `user/band_ends.lua` holds `return "stop"`, no `user/init.lua` exists, and a client attaches
- **THEN** `gband.opt.band_ends` reads `"stop"`

#### Scenario: Nothing saved
- **WHEN** no `user/band_ends.lua` exists and no `user/init.lua` exists
- **THEN** `gband.opt.band_ends` reads `"loop"`

#### Scenario: Unknown saved value
- **WHEN** `user/band_ends.lua` holds `return "wrap"` and a callback calls `gband.settings.band_ends()`
- **THEN** the call returns nil, no error is reported, and the default configuration sets `band_ends` to `"loop"`

#### Scenario: Broken file is not a configuration error
- **WHEN** `user/band_ends.lua` holds `error("boom")` and a client attaches
- **THEN** loading succeeds with no error and `gband.opt.band_ends` reads `"loop"`

#### Scenario: Own configuration
- **WHEN** `user/band_ends.lua` holds `return "bounce"` and `user/init.lua` sets only `gband.opt.band_ends = "stop"`
- **THEN** `gband.opt.band_ends` reads `"stop"`

## MODIFIED Requirements

### Requirement: Settings window
`gband.settings.open()` SHALL make `root` the active table, as `gband.keymap.enter("root")` does. It SHALL then open the settings window: a floating plugin window, as the plugin-windows capability defines, that takes focus. It SHALL be callable wherever an action value is, as the configuration capability defines. Calling it while the configuration loads SHALL be an error at the line of the call. Calling it while the settings window is open SHALL focus that window and open no second one. Calling it while the theme list is open SHALL close the theme list as Escape does, then focus the settings window.

The settings window SHALL have a border, the title `settings`, and its cursor line on. It SHALL hold these lines, in this order. Each label is in the group `SettingsLabel` and is padded with spaces to 9 cells, and each value follows it in the plugin window's own group:

| line | label | value |
|---|---|---|
| 1 | `theme` | the name `gband.colorscheme()` returns |
| 2 | `sidebar` | `off` when `gband.settings.sidebar()` returns `false`, `on` otherwise |
| 3 | `bands` | the value `gband.settings.band_ends()` returns, or `loop` when it returns nil |
| 4 | `keys` | the style `gband.keystyle.saved()` returns, or `modal` when it returns nil |
| 5 | `I on new` | `on` when `gband.settings.interactive_on_new()` returns `true`, `off` otherwise |
| last | `reload` | `ok` when `gband.errors()` returns an empty list, `1 error` when it holds one error, and the number of errors followed by ` errors` otherwise |

The `I on new` line SHALL be present only when the fourth line shows `modal`, and the `reload` line SHALL then be the sixth. When the fourth line shows `direct` or `floating`, the window SHALL hold the first four lines and the `reload` line as its fifth.

The cursor line SHALL start on the first line, except as "Reopen after a save" defines. The window's width SHALL be the smaller of 31 and the ribbon area's width. Its height SHALL be the smaller of the number of its lines plus 2 and the ribbon area's height. It SHALL be centered in the ribbon area, rounding the left and top offsets down. A line wider than the content area SHALL be cut at the content area's edge, as the plugin-windows capability defines.

#### Scenario: Window opens
- **WHEN** no setting is saved, the active colorscheme is `default`, and a binding function calls `gband.settings.open()`
- **THEN** a focused floating plugin window titled `settings` shows `theme    default`, `sidebar  on`, `bands    loop`, `keys     modal`, `I on new off` and `reload   ok`, with the cursor line on the first line

#### Scenario: Window beside the default sidebar
- **WHEN** the default configuration is in use with the modal style on an 80×24 terminal and the settings window opens
- **THEN** the window is 31 columns wide and 8 rows high, and spans columns 24 to 54 and rows 8 to 15 of the 79-column ribbon area

#### Scenario: Window with the direct style
- **WHEN** `user/keystyle.lua` holds `return "direct"`, the default configuration is in use on an 80×24 terminal, and the settings window opens
- **THEN** the window holds five lines, the fourth showing `keys     direct` and the fifth `reload   ok`
- **AND** it is 31 columns wide and 7 rows high, and spans columns 24 to 54 and rows 8 to 14 of the 79-column ribbon area

#### Scenario: Window with the floating style
- **WHEN** `user/keystyle.lua` holds `return "floating"`, the default configuration is in use on an 80×24 terminal, and the settings window opens
- **THEN** the window holds five lines, the fourth showing `keys     floating` and the fifth `reload   ok`
- **AND** it is 31 columns wide and 7 rows high, and spans columns 24 to 54 and rows 8 to 14 of the 79-column ribbon area

#### Scenario: Saved values shown
- **WHEN** `user/sidebar.lua` holds `return false`, `user/band_ends.lua` holds `return "bounce"`, `user/interactive_on_new.lua` holds `return true`, and the settings window opens
- **THEN** its second line shows `sidebar  off`, its third line shows `bands    bounce`, and its fifth line shows `I on new on`

#### Scenario: Errors counted
- **WHEN** `gband.errors()` returns two errors and the settings window opens
- **THEN** its last line shows `reload   2 errors`

#### Scenario: One error counted
- **WHEN** `gband.errors()` returns one error and the settings window opens
- **THEN** its last line shows `reload   1 error`

#### Scenario: Opened from navigation mode
- **WHEN** the modal style is in use and the user presses Ctrl+Space then `s`
- **THEN** the settings window is open and focused
- **AND** `root` is active, so a following `j` moves the cursor line to the second line

#### Scenario: Open again while open
- **WHEN** the settings window is open, a binding moves focus to another column, and a binding function calls `gband.settings.open()` again
- **THEN** exactly one settings window is drawn and it has focus

#### Scenario: Open while loading
- **WHEN** line 5 of `user/init.lua` calls `gband.settings.open()` at the top level
- **THEN** loading fails with an error at `user/init.lua` line 5

#### Scenario: Opened from the Lua prompt
- **WHEN** the user opens the Lua prompt, types `gband.settings.open()` and presses Enter
- **THEN** the settings window is open and has focus

### Requirement: Settings window keys
j, k, the arrow keys Up and Down, and the other default movement keys of the plugin-windows capability SHALL move the cursor line. Escape and `q` SHALL close the settings window and save nothing, as the plugin-windows capability defines for a floating plugin window with no `keys` entry for them.

On the `theme` line:
- Enter SHALL open the theme list, as "Theme list" defines.
- `l` and Right SHALL load the theme that follows the active colorscheme in the list `gband.settings.themes()` returns, and `h` and Left the theme before it. The list wraps at both ends. When the active colorscheme is not in the list, `l` and Right SHALL load the first theme and `h` and Left the last. A theme that loads SHALL be saved, as "Saved theme" defines. A theme that fails to load SHALL be reported as the colorschemes capability defines and SHALL NOT be saved.

On the `sidebar` line, Enter, `h`, `l`, Left and Right SHALL save the other value: `false` when the line shows `on`, and `true` when it shows `off`.

On the `bands` line, Enter, `l` and Right SHALL save the next value in the order `loop`, `bounce`, `stop`, and `loop` when the line shows `stop`. `h` and Left SHALL save the previous value in that order, and `stop` when the line shows `loop`. Each SHALL save as "Saved band ends" defines.

On the `keys` line, Enter, `l` and Right SHALL save the next key style in the order `modal`, `direct`, `floating`, and `modal` when the line shows `floating`. `h` and Left SHALL save the previous key style in that order, and `floating` when the line shows `modal`. Each SHALL save as the key-style capability's "Saved key style" defines.

On the `I on new` line, Enter, `h`, `l`, Left and Right SHALL save the other value, as "Saved I on new" defines: `false` when the line shows `on`, and `true` when it shows `off`.

On the `reload` line, Enter SHALL dispatch `gband.action.reload`, as the configuration capability's "Forced reload" defines, and save nothing. `h`, `l`, Left and Right SHALL do nothing there.

When `gband.config_dir` is nil, or a file cannot be written, the key SHALL save nothing and the settings window SHALL stay open. The key SHALL raise an error naming the file, such as `user/sidebar.lua`, and the reason, which the client reports as the configuration capability defines for an error in a callback. The setting in use SHALL stay.

#### Scenario: Turn the sidebar off
- **WHEN** no `user/init.lua` exists, the settings window is open on its first line, and the user presses `j` then Enter
- **THEN** `user/sidebar.lua` holds `return false`
- **AND** within two seconds the configuration reloads and no sidebar is drawn

#### Scenario: Switch the key style
- **WHEN** no `user/init.lua` exists, the modal style is saved, the settings window is open on its fourth line, and the user presses `l`
- **THEN** `user/keystyle.lua` holds `return "direct"`
- **AND** within two seconds the configuration reloads with the direct style's bindings, and the reopened settings window holds five lines

#### Scenario: Switch to the floating style
- **WHEN** no `user/init.lua` exists, the direct style is saved, the settings window is open on its fourth line, and the user presses Enter
- **THEN** `user/keystyle.lua` holds `return "floating"`
- **AND** within two seconds the configuration reloads with the floating style's bindings, and the reopened settings window holds five lines

#### Scenario: Back from the modal style
- **WHEN** no `user/init.lua` exists, the modal style is saved, the settings window is open on its fourth line, and the user presses `h`
- **THEN** `user/keystyle.lua` holds `return "floating"`

#### Scenario: Forward from the floating style
- **WHEN** no `user/init.lua` exists, the floating style is saved, the settings window is open on its fourth line, and the user presses Right
- **THEN** `user/keystyle.lua` holds `return "modal"`

#### Scenario: Turn I on new on
- **WHEN** no `user/init.lua` and no `user/interactive_on_new.lua` exist, the modal style is in use, the settings window is open on its fifth line, and the user presses Enter
- **THEN** `user/interactive_on_new.lua` holds `return true`
- **AND** after the reload the settings window is open on its fifth line showing `I on new on`, and Ctrl+Space then `n` returns to interactive mode

#### Scenario: Turn I on new off
- **WHEN** no `user/init.lua` exists, `user/interactive_on_new.lua` holds `return true`, the modal style is in use, the settings window is open on its fifth line, and the user presses Enter
- **THEN** `user/interactive_on_new.lua` holds `return false`
- **AND** after the reload the settings window is open on its fifth line showing `I on new off`, and Ctrl+Space then `n` leaves navigation mode active

#### Scenario: Turn I on new back on
- **WHEN** `user/interactive_on_new.lua` holds `return false`, the settings window is open on its fifth line, and the user presses `h`
- **THEN** `user/interactive_on_new.lua` holds `return true`

#### Scenario: Bounce the band ends
- **WHEN** no `user/init.lua` and no `user/band_ends.lua` exist, the settings window is open on its third line, and the user presses `l`
- **THEN** `user/band_ends.lua` holds `return "bounce"`
- **AND** within two seconds the configuration reloads, `gband.opt.band_ends` reads `"bounce"`, and the reopened settings window is open on its third line showing `bands    bounce`

#### Scenario: Band ends step forward from bounce
- **WHEN** `user/band_ends.lua` holds `return "bounce"`, the settings window is open on its third line, and the user presses Enter
- **THEN** `user/band_ends.lua` holds `return "stop"`

#### Scenario: Band ends wrap back from loop
- **WHEN** no `user/band_ends.lua` exists, the settings window is open on its third line, and the user presses `h`
- **THEN** `user/band_ends.lua` holds `return "stop"`

#### Scenario: Band ends wrap forward from stop
- **WHEN** `user/band_ends.lua` holds `return "stop"`, the settings window is open on its third line, and the user presses Right
- **THEN** `user/band_ends.lua` holds `return "loop"`

#### Scenario: Next theme
- **WHEN** the active colorscheme is `gruvbox`, the settings window is open on its first line, and the user presses `l`
- **THEN** `gband.colorscheme()` returns `one-dark` and `user/theme.lua` holds `return "one-dark"`

#### Scenario: Theme wraps
- **WHEN** the active colorscheme is `default`, the settings window is open on its first line, and the user presses `h`
- **THEN** the last name `gband.settings.themes()` returns is the active colorscheme and is saved

#### Scenario: Dismiss
- **WHEN** the settings window is open and the user presses Escape
- **THEN** the window closes and no file in `user` changes

#### Scenario: Reload from the settings window
- **WHEN** a plugin's `client.lua` is rewritten from drawing `old` to drawing `new`, the settings window is open on its `reload` line, and the user presses Enter
- **THEN** the client draws `new` and no file in `user` changes

#### Scenario: Arrows on the reload line
- **WHEN** the settings window is open on its `reload` line and the user presses `l`
- **THEN** no configuration loads and no file in `user` changes

#### Scenario: Cannot save
- **WHEN** the `user` directory cannot be written and the user presses Enter on the settings window's second line
- **THEN** the client shows an error naming `user/sidebar.lua`
- **AND** the settings window stays open and the sidebar is still drawn

### Requirement: Record the layout entering the floating style

When the settings window's `keys` line switches a style other than `floating` to `floating`, the settings SHALL record the client's layout, as `gband.layout()` gives it, before the reload the switch causes floats the windows. The record SHALL hold only windows that run a program, as the floating-key-style capability's "Windows float" defines. It SHALL hold, for each band, its tiled columns left to right with each column's `width`, `full_width` and `tabbed` state and each column's windows top to bottom with each window's height, and its floating windows with each window's box, `col`, `row`, `width`, `full_width` and `rows`. A column that holds no window that runs a program SHALL NOT be recorded.

The record SHALL persist across the reload the switch causes and across the settings window closing and reopening. It SHALL be replaced when the `keys` line switches a non-floating style to `floating` again, and discarded when the `keys` line switches `floating` to another style, as "Restore the layout leaving the floating style" defines. The `keys` line SHALL still save the key style as "Settings window keys" defines.

#### Scenario: A record is kept entering floating
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds columns with windows 1 and 2, the settings window is open on its fourth line, and the user presses `h`
- **THEN** `user/keystyle.lua` holds `return "floating"`
- **AND** after the reload both windows float

### Requirement: Restore the layout leaving the floating style

When the settings window's `keys` line switches `floating` to `modal` or `direct` and a record exists, as "Record the layout entering the floating style" defines, the settings SHALL restore that layout, then discard the record. Each recorded window still in the client's layout SHALL return to the band, and tiled or floating state, the record holds for it:

- A window the record holds tiled SHALL tile in its recorded band, in a column at the recorded position from the left, at the recorded position from the top within that column, so a column's windows keep their order and stacking. Its column SHALL take the recorded `width`, `full_width` and `tabbed` state, and the window SHALL take its recorded height.
- A window the record holds floating SHALL float with its recorded box, `col`, `row`, `width`, `full_width` and `rows`.
- A floating window that runs a program and is not in the record, such as one opened while `floating` was in use, SHALL tile after the restored tiled windows of its band.

A recorded window no longer floating in the client's layout SHALL be skipped. When the `keys` line switches `floating` to another style and no record exists, every floating window that runs a program SHALL tile. A floating window that runs no program, such as the settings window, SHALL stay floating. The `keys` line SHALL still save the key style as "Settings window keys" defines.

#### Scenario: Columns restored across a round trip
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds columns with windows 1 and 2, the settings window is open on its fourth line, the user presses `h`, and after the reload presses `l`
- **THEN** `user/keystyle.lua` holds `return "modal"`
- **AND** after the reload the band holds columns with windows 1 and 2, in that order, and no window floats

#### Scenario: A stacked column restored
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds one column with windows 1 and 2 stacked, the settings window is open on its fourth line, the user presses `h`, and after the reload presses `l`
- **THEN** after the reload the band holds one column with windows 1 and 2 stacked, in that order

#### Scenario: A column width restored
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds a column of width 1/3 with window 1 and a column with window 2, the settings window is open on its fourth line, the user presses `h`, and after the reload presses `l`
- **THEN** after the reload window 1's column has width 1/3

#### Scenario: A window opened while floating tiles after the restored windows
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds one column with window 1, the settings window is open on its fourth line, the user presses `h`, after the reload a floating window 2 opens, and the user presses `l`
- **THEN** after the reload the band holds columns with windows 1 and 2, window 1 before window 2, and no window floats

#### Scenario: A previously floating window keeps its box
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds a column with window 1 and a floating window 3 with `col` 5, `row` 3, width 1/3 and `rows` 10, the settings window is open on its fourth line, the user presses `h`, and after the reload presses `l`
- **THEN** after the reload window 1 tiles and window 3 floats with `col` 5, `row` 3, width 1/3 and `rows` 10

#### Scenario: No record tiles every floating window
- **WHEN** `user/keystyle.lua` holds `return "floating"`, no `user/init.lua` exists, the viewed band holds floating windows 1 and 2, the settings window is open on its fourth line, and the user presses `l`
- **THEN** `user/keystyle.lua` holds `return "modal"`
- **AND** after the reload windows 1 and 2 tile and no window floats

### Requirement: Reopen after a save
When a key of the settings window or the theme list saves a setting, the client SHALL remember the line of the settings window that the setting belongs to, until the next load ends. When that load succeeds, the client SHALL open the settings window and focus it, as `gband.settings.open()` does, with its cursor line on that line. This SHALL apply whatever configuration file is in use. A load that fails SHALL forget the line and open nothing, and so SHALL a load that no such save preceded.

When Enter on the `reload` line dispatches the reload, the client SHALL remember that line until the forced reload has ended: its own load and the server's answer, as the configuration capability's "Forced reload" defines. The client SHALL then open the settings window and focus it with its cursor line on the `reload` line, whether the loads succeeded or failed. The window SHALL show the values as of that moment, and when it is still open, it SHALL show its lines again with those values.

#### Scenario: Back on the same line
- **WHEN** the settings window is open on its fourth line and the user presses Enter
- **THEN** after the reload the settings window is open and focused, with the cursor line on its fourth line
- **AND** its fourth line shows the new key style

#### Scenario: Back after picking a theme
- **WHEN** the user picks `nord` in the theme list
- **THEN** after the reload the settings window is open on its first line and shows `theme    nord`

#### Scenario: No reopen for an unrelated reload
- **WHEN** the user closes the settings window with Escape and then saves `user/lua/extra.lua`
- **THEN** the configuration reloads and no settings window opens

#### Scenario: Back on the reload line
- **WHEN** the modal style is in use, the settings window is open on its sixth line, `reload   ok`, and the user presses Enter
- **THEN** after the reload the settings window is open and focused, with the cursor line on its sixth line

#### Scenario: Failed reload counted
- **WHEN** the settings window shows `reload   ok`, a plugin's `client.lua` is changed to raise `bad` on line 1, and the user presses Enter on the `reload` line
- **THEN** after the reload the settings window is open and focused on its `reload` line, which shows `reload   1 error`

#### Scenario: Server error counted
- **WHEN** the settings window shows `reload   ok`, a plugin's `server.lua` is changed to raise `bad` on line 1, and the user presses Enter on the `reload` line
- **THEN** after the reload the settings window's `reload` line shows `reload   1 error`
