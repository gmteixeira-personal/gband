## MODIFIED Requirements

### Requirement: Settings window
`gband.settings.open()` SHALL make `root` the active table, as `gband.keymap.enter("root")` does. It SHALL then open the settings window: a floating plugin window, as the plugin-windows capability defines, that takes focus. It SHALL be callable wherever an action value is, as the configuration capability defines. Calling it while the configuration loads SHALL be an error at the line of the call. Calling it while the settings window is open SHALL focus that window and open no second one. Calling it while the theme list is open SHALL close the theme list as Escape does, then focus the settings window.

The settings window SHALL have a border, the title `settings`, and its cursor line on. It SHALL hold these lines, in this order. Each label is in the group `SettingsLabel` and is padded with spaces to 9 cells, and each value follows it in the plugin window's own group:

| line | label | value |
|---|---|---|
| 1 | `theme` | the name `gband.colorscheme()` returns |
| 2 | `sidebar` | `off` when `gband.settings.sidebar()` returns `false`, `on` otherwise |
| 3 | `keys` | the style `gband.keystyle.saved()` returns, or `modal` when it returns nil |
| 4 | `I on new` | `on` when `gband.settings.interactive_on_new()` returns `true`, `off` otherwise |

The fourth line SHALL be present only when the third line shows `modal`. When the third line shows `direct` or `floating`, the window SHALL hold the first three lines only.

The cursor line SHALL start on the first line, except as "Reopen after a save" defines. The window's width SHALL be the smaller of 31 and the ribbon area's width. Its height SHALL be the smaller of the number of its lines plus 2 and the ribbon area's height. It SHALL be centered in the ribbon area, rounding the left and top offsets down. A line wider than the content area SHALL be cut at the content area's edge, as the plugin-windows capability defines.

#### Scenario: Window opens
- **WHEN** no setting is saved, the active colorscheme is `default`, and a binding function calls `gband.settings.open()`
- **THEN** a focused floating plugin window titled `settings` shows `theme    default`, `sidebar  on`, `keys     modal` and `I on new off`, with the cursor line on the first line

#### Scenario: Window beside the default sidebar
- **WHEN** the default configuration is in use with the modal style on an 80×24 terminal and the settings window opens
- **THEN** the window is 31 columns wide and 6 rows high, and spans columns 24 to 54 and rows 9 to 14 of the 79-column ribbon area

#### Scenario: Window with the direct style
- **WHEN** `user/keystyle.lua` holds `return "direct"`, the default configuration is in use on an 80×24 terminal, and the settings window opens
- **THEN** the window holds three lines, the last showing `keys     direct`
- **AND** it is 31 columns wide and 5 rows high, and spans columns 24 to 54 and rows 9 to 13 of the 79-column ribbon area

#### Scenario: Window with the floating style
- **WHEN** `user/keystyle.lua` holds `return "floating"`, the default configuration is in use on an 80×24 terminal, and the settings window opens
- **THEN** the window holds three lines, the last showing `keys     floating`
- **AND** it is 31 columns wide and 5 rows high, and spans columns 24 to 54 and rows 9 to 13 of the 79-column ribbon area

#### Scenario: Saved values shown
- **WHEN** `user/sidebar.lua` holds `return false`, `user/interactive_on_new.lua` holds `return true`, and the settings window opens
- **THEN** its second line shows `sidebar  off` and its fourth line shows `I on new on`

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

On the `keys` line, Enter, `l` and Right SHALL save the next key style in the order `modal`, `direct`, `floating`, and `modal` when the line shows `floating`. `h` and Left SHALL save the previous key style in that order, and `floating` when the line shows `modal`. Each SHALL save as the key-style capability's "Saved key style" defines.

On the `I on new` line, Enter, `h`, `l`, Left and Right SHALL save the other value, as "Saved I on new" defines: `false` when the line shows `on`, and `true` when it shows `off`.

When `gband.config_dir` is nil, or a file cannot be written, the key SHALL save nothing and the settings window SHALL stay open. The key SHALL raise an error naming the file, such as `user/sidebar.lua`, and the reason, which the client reports as the configuration capability defines for an error in a callback. The setting in use SHALL stay.

#### Scenario: Turn the sidebar off
- **WHEN** no `user/init.lua` exists, the settings window is open on its first line, and the user presses `j` then Enter
- **THEN** `user/sidebar.lua` holds `return false`
- **AND** within a second the configuration reloads and no sidebar is drawn

#### Scenario: Switch the key style
- **WHEN** no `user/init.lua` exists, the modal style is saved, the settings window is open on its third line, and the user presses `l`
- **THEN** `user/keystyle.lua` holds `return "direct"`
- **AND** within a second the configuration reloads with the direct style's bindings, and the reopened settings window holds three lines

#### Scenario: Switch to the floating style
- **WHEN** no `user/init.lua` exists, the direct style is saved, the settings window is open on its third line, and the user presses Enter
- **THEN** `user/keystyle.lua` holds `return "floating"`
- **AND** within a second the configuration reloads with the floating style's bindings, and the reopened settings window holds three lines

#### Scenario: Back from the modal style
- **WHEN** no `user/init.lua` exists, the modal style is saved, the settings window is open on its third line, and the user presses `h`
- **THEN** `user/keystyle.lua` holds `return "floating"`

#### Scenario: Forward from the floating style
- **WHEN** no `user/init.lua` exists, the floating style is saved, the settings window is open on its third line, and the user presses Right
- **THEN** `user/keystyle.lua` holds `return "modal"`

#### Scenario: Turn I on new on
- **WHEN** no `user/init.lua` and no `user/interactive_on_new.lua` exist, the modal style is in use, the settings window is open on its fourth line, and the user presses Enter
- **THEN** `user/interactive_on_new.lua` holds `return true`
- **AND** after the reload the settings window is open on its fourth line showing `I on new on`, and Ctrl+Space then `n` returns to interactive mode

#### Scenario: Turn I on new off
- **WHEN** no `user/init.lua` exists, `user/interactive_on_new.lua` holds `return true`, the modal style is in use, the settings window is open on its fourth line, and the user presses Enter
- **THEN** `user/interactive_on_new.lua` holds `return false`
- **AND** after the reload the settings window is open on its fourth line showing `I on new off`, and Ctrl+Space then `n` leaves navigation mode active

#### Scenario: Turn I on new back on
- **WHEN** `user/interactive_on_new.lua` holds `return false`, the settings window is open on its fourth line, and the user presses `h`
- **THEN** `user/interactive_on_new.lua` holds `return true`

#### Scenario: Next theme
- **WHEN** the active colorscheme is `gruvbox`, the settings window is open on its first line, and the user presses `l`
- **THEN** `gband.colorscheme()` returns `one-dark` and `user/theme.lua` holds `return "one-dark"`

#### Scenario: Theme wraps
- **WHEN** the active colorscheme is `default`, the settings window is open on its first line, and the user presses `h`
- **THEN** the last name `gband.settings.themes()` returns is the active colorscheme and is saved

#### Scenario: Dismiss
- **WHEN** the settings window is open and the user presses Escape
- **THEN** the window closes and no file in `user` changes

#### Scenario: Cannot save
- **WHEN** the `user` directory cannot be written and the user presses Enter on the settings window's second line
- **THEN** the client shows an error naming `user/sidebar.lua`
- **AND** the settings window stays open and the sidebar is still drawn

