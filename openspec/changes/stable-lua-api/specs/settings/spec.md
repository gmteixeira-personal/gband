## ADDED Requirements

### Requirement: Save a setting
`gband.settings.save(name, value, opts)` SHALL save one setting. `name` and `value` SHALL be one of these:

| `name` | `value` | file |
|---|---|---|
| `"theme"` | a valid colorscheme name, as the colorschemes capability defines | `user/theme.lua`, as "Saved theme" defines |
| `"sidebar"` | a boolean | `user/sidebar.lua`, as "Saved sidebar" defines |
| `"keystyle"` | `"modal"` or `"direct"` | `user/keystyle.lua`, as the key-style capability's "Saved key style" defines |
| `"interactive_on_new"` | a boolean | `user/interactive_on_new.lua`, as "Saved I on new" defines |

It SHALL write the file's text as that requirement defines, creating the file or replacing it in one step, so no process reads it half written, and SHALL change no other file. The write reloads the configuration, as the configuration capability's "Reload on change" defines. `opts` SHALL be nil or a table. `opts.reopen`, when given, SHALL be a positive integer: the settings window line that "Reopen after a save" opens once that reload succeeds.

An unknown `name`, a `value` invalid for it, or an `opts` or `opts.reopen` of the wrong type SHALL raise an error at the line of the call and write nothing. When `gband.config_dir` is nil or the file cannot be written, the call SHALL raise an error naming the file, such as `user/sidebar.lua`, and the reason, and SHALL remember no line. The call SHALL be allowed wherever an action value is, and calling it while the configuration loads SHALL be an error at the line of the call.

#### Scenario: Save a theme
- **WHEN** a binding function calls `gband.settings.save("theme", "nord")`
- **THEN** `user/theme.lua` holds `return "nord"` and a newline, and the configuration reloads
- **AND** no settings window opens after the reload

#### Scenario: Save and reopen
- **WHEN** a binding function calls `gband.settings.save("sidebar", false, { reopen = 2 })`
- **THEN** after the reload the settings window is open and focused with its cursor line on its second line, which shows `sidebar  off`

#### Scenario: Invalid value
- **WHEN** a binding function calls `gband.settings.save("keystyle", "vim")`
- **THEN** the client reports an error at the line of the call, and `user/keystyle.lua` is unchanged

#### Scenario: Unknown setting
- **WHEN** a binding function calls `gband.settings.save("font", "mono")`
- **THEN** the client reports an error at the line of the call, and no file is written

#### Scenario: No configuration directory
- **WHEN** `gband.config_dir` is nil and a binding function calls `gband.settings.save("sidebar", true)`
- **THEN** the client reports an error naming `user/sidebar.lua`

#### Scenario: Save while loading
- **WHEN** line 2 of `user/init.lua` calls `gband.settings.save("sidebar", true)` at the top level
- **THEN** loading fails with an error at `user/init.lua` line 2

## MODIFIED Requirements

### Requirement: Settings window
`gband.settings.open(opts)` SHALL make `root` the active table, as `gband.keymap.enter("root")` does. It SHALL then open the settings window by requiring the module `gband.settings.window`, as the plugins capability's "Module lookup" finds it, and calling the module's `open` with a table whose `line` is `opts.line`, or nil when `opts` or `opts.line` is nil. So a module `gband.settings.window` on the runtimepath replaces the bundled one. `opts` SHALL be nil or a table, and `opts.line` nil or a positive integer, or the call SHALL raise an error at its line. `gband.settings.open` SHALL be callable wherever an action value is, as the configuration capability defines. Calling it while the configuration loads SHALL be an error at the line of the call. An error raised by requiring the module or by its `open` SHALL be reported as an error in the calling callback.

The bundled module `gband.settings.window` SHALL open the settings window: a floating plugin window, as the plugin-windows capability defines, that takes focus. It SHALL use only the documented API, as the lua-api capability defines, and SHALL save each setting with `gband.settings.save`, passing the line the setting belongs to as `opts.reopen`. Its `open` called while the settings window is open SHALL focus that window and open no second one. Its `open` called while the theme list is open SHALL close the theme list as Escape does, then focus the settings window.

The settings window SHALL have a border, the title `settings`, and its cursor line on. It SHALL hold these lines, in this order. Each label is in the group `SettingsLabel` and is padded with spaces to 9 cells, and each value follows it in the plugin window's own group:

| line | label | value |
|---|---|---|
| 1 | `theme` | the name `gband.colorscheme()` returns |
| 2 | `sidebar` | `off` when `gband.settings.sidebar()` returns `false`, `on` otherwise |
| 3 | `keys` | the style `gband.keystyle.saved()` returns, or `modal` when it returns nil |
| 4 | `I on new` | `off` when `gband.settings.interactive_on_new()` returns `false`, `on` otherwise |

The fourth line SHALL be present only when the third line shows `modal`. When the third line shows `direct`, the window SHALL hold the first three lines only.

The cursor line SHALL start on the line `open` receives as `line`, or on the first line when it receives none or a line the window does not hold. The window's width SHALL be the smaller of 31 and the ribbon area's width. Its height SHALL be the smaller of the number of its lines plus 2 and the ribbon area's height. It SHALL be centered in the ribbon area, rounding the left and top offsets down. A line wider than the content area SHALL be cut at the content area's edge, as the plugin-windows capability defines.

#### Scenario: Window opens
- **WHEN** no setting is saved, the active colorscheme is `default`, and a binding function calls `gband.settings.open()`
- **THEN** a focused floating plugin window titled `settings` shows `theme    default`, `sidebar  on`, `keys     modal` and `I on new on`, with the cursor line on the first line

#### Scenario: Window beside the default sidebar
- **WHEN** the default configuration is in use with the modal style on an 80×24 terminal and the settings window opens
- **THEN** the window is 31 columns wide and 6 rows high, and spans columns 24 to 54 and rows 9 to 14 of the 79-column ribbon area

#### Scenario: Window with the direct style
- **WHEN** `user/keystyle.lua` holds `return "direct"`, the default configuration is in use on an 80×24 terminal, and the settings window opens
- **THEN** the window holds three lines, the last showing `keys     direct`
- **AND** it is 31 columns wide and 5 rows high, and spans columns 24 to 54 and rows 9 to 13 of the 79-column ribbon area

#### Scenario: Saved values shown
- **WHEN** `user/sidebar.lua` holds `return false`, `user/interactive_on_new.lua` holds `return false`, and the settings window opens
- **THEN** its second line shows `sidebar  off` and its fourth line shows `I on new off`

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

#### Scenario: Open on a line
- **WHEN** the modal style is in use and a binding function calls `gband.settings.open({ line = 3 })`
- **THEN** the settings window is open with its cursor line on its third line

#### Scenario: Replaced window module
- **WHEN** `user/lua/gband/settings/window.lua` returns a table whose `open` calls `gband.notify("mine")`, and a binding function calls `gband.settings.open()`
- **THEN** the client notifies `mine` and no bundled settings window opens

### Requirement: Reopen after a save
When a call of `gband.settings.save` with `opts.reopen` writes its file, the client SHALL remember that line until the next load ends. The settings window's keys and the theme list pass the line of the settings window that the setting belongs to, as "Settings window" defines. When that load succeeds, the client SHALL call `gband.settings.open({ line = <line> })`, so the settings window opens and takes focus with its cursor line on that line. This SHALL apply whatever configuration file is in use. A load that fails SHALL forget the line and open nothing, and so SHALL a load that no such save preceded.

#### Scenario: Back on the same line
- **WHEN** the settings window is open on its third line and the user presses Enter
- **THEN** after the reload the settings window is open and focused, with the cursor line on its third line
- **AND** its third line shows the new key style

#### Scenario: Back after picking a theme
- **WHEN** the user picks `nord` in the theme list
- **THEN** after the reload the settings window is open on its first line and shows `theme    nord`

#### Scenario: No reopen for an unrelated reload
- **WHEN** the user closes the settings window with Escape and then saves `user/lua/extra.lua`
- **THEN** the configuration reloads and no settings window opens

#### Scenario: No reopen without the option
- **WHEN** a binding function calls `gband.settings.save("sidebar", false)` and the reload succeeds
- **THEN** no settings window opens
