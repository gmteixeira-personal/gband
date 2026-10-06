## Purpose

Defines gband's settings: the settings window that chooses the theme, the sidebar and the key style, the theme list that previews and saves a theme, the files that hold the saved theme and sidebar, and the offer of the settings window on the first start.

## ADDED Requirements

### Requirement: Saved theme
The saved theme SHALL live in the file `user/theme.lua` of the configuration directory. Saving a theme SHALL write the text `return "<name>"` followed by a newline to that file. It SHALL create the file or replace it in one step, so no process reads it half written.

`gband.settings.theme()` SHALL evaluate `user/theme.lua` as a Lua chunk in text mode with an empty environment. It SHALL return the chunk's value when that value is a string that is a valid colorscheme name, as the colorschemes capability defines. It SHALL return nil in these cases:
- `gband.config_dir` is nil;
- the file is missing or cannot be read;
- the file does not compile or raises an error;
- the file returns any other value.

It SHALL raise no error and report none. It SHALL be callable while the configuration loads and in any callback. Loading SHALL evaluate `user/theme.lua` only in this way, never as a configuration file, a plugin file or a module.

#### Scenario: Saved file
- **WHEN** the theme list saves `nord`
- **THEN** `user/theme.lua` holds `return "nord"` and a newline

#### Scenario: Written by hand
- **WHEN** `user/theme.lua` holds `return 'dracula'` and a callback calls `gband.settings.theme()`
- **THEN** the call returns `dracula`

#### Scenario: Invalid saved value
- **WHEN** `user/theme.lua` holds `return 42` and a callback calls `gband.settings.theme()`
- **THEN** the call returns nil and no error is reported

#### Scenario: Broken file is not a configuration error
- **WHEN** `user/theme.lua` holds `error("boom")` and a client attaches
- **THEN** loading succeeds with no error and `gband.colorscheme()` returns `gruvbox`

### Requirement: Saved sidebar
The saved sidebar setting SHALL live in the file `user/sidebar.lua` of the configuration directory. Saving it SHALL write `return true` or `return false`, followed by a newline, to that file, creating or replacing it in one step.

`gband.settings.sidebar()` SHALL evaluate `user/sidebar.lua` as "Saved theme" defines for `user/theme.lua`. It SHALL return the chunk's value when that value is a boolean, and nil in every case where `gband.settings.theme()` returns nil. It SHALL raise no error and report none. It SHALL be callable while the configuration loads and in any callback.

The default configuration SHALL set up the sidebar plugin when `gband.settings.sidebar()` does not return `false`, as the configuration capability defines. A configuration file that sets up the sidebar itself SHALL keep its own choice, whatever is saved.

#### Scenario: Sidebar turned off
- **WHEN** `user/sidebar.lua` holds `return false`, no `user/init.lua` exists, and a client attaches on an 80×24 terminal
- **THEN** `gband.bar.list()` holds no bar `sidebar` and the ribbon area spans columns 0 to 79

#### Scenario: Nothing saved
- **WHEN** no `user/sidebar.lua` exists and no `user/init.lua` exists
- **THEN** the sidebar is drawn on column 0

#### Scenario: Unknown saved value
- **WHEN** `user/sidebar.lua` holds `return "off"` and a callback calls `gband.settings.sidebar()`
- **THEN** the call returns nil and the default configuration draws the sidebar

### Requirement: Theme names
`gband.settings.themes()` SHALL return a new list of colorscheme names. The bundled themes SHALL come first, in the order the colorschemes capability's "Bundled themes" lists them, without the alias `catppuccin`. Every other colorscheme that `gband.colorscheme` can find SHALL follow, sorted by byte value: each valid colorscheme name `n` for which a runtimepath entry holds a file `colors/<n>.lua`. A name SHALL appear once, so a runtimepath file that shadows a bundled theme keeps the bundled theme's place. It SHALL be callable while the configuration loads and in any callback.

#### Scenario: Bundled themes only
- **WHEN** no runtimepath entry holds a `colors` directory and a callback calls `gband.settings.themes()`
- **THEN** it returns the 14 bundled theme names, starting with `terminal` and ending with `vesper`

#### Scenario: User colorschemes after the bundled themes
- **WHEN** `user/colors/zen.lua`, `user/colors/dusk.lua` and `user/colors/nord.lua` exist
- **THEN** `gband.settings.themes()` ends with `dusk` and then `zen`
- **AND** `nord` appears once, in the place of the bundled theme

### Requirement: Settings window
`gband.settings.open()` SHALL make `root` the active table, as `gband.keymap.enter("root")` does. It SHALL then open the settings window: a floating plugin window, as the plugin-windows capability defines, that takes focus. It SHALL be callable wherever an action value is, as the configuration capability defines. Calling it while the configuration loads SHALL be an error at the line of the call. Calling it while the settings window is open SHALL focus that window and open no second one. Calling it while the theme list is open SHALL close the theme list as Escape does, then focus the settings window.

The settings window SHALL have a border, the title `settings`, and its cursor line on. It SHALL hold these lines, in this order. Each label is in the group `SettingsLabel` and is padded with spaces to 9 cells, and each value follows it in the plugin window's own group:

| line | label | value |
|---|---|---|
| 1 | `theme` | the name `gband.colorscheme()` returns |
| 2 | `sidebar` | `off` when `gband.settings.sidebar()` returns `false`, `on` otherwise |
| 3 | `keys` | the style `gband.keystyle.saved()` returns, or `modal` when it returns nil |

The cursor line SHALL start on the first line, except as "Reopen after a save" defines. The window's width SHALL be the smaller of 31 and the ribbon area's width. Its height SHALL be the smaller of 5 and the ribbon area's height. It SHALL be centered in the ribbon area, rounding the left and top offsets down. A line wider than the content area SHALL be cut at the content area's edge, as the plugin-windows capability defines.

#### Scenario: Window opens
- **WHEN** no setting is saved, the active colorscheme is `gruvbox`, and a binding function calls `gband.settings.open()`
- **THEN** a focused floating plugin window titled `settings` shows `theme    gruvbox`, `sidebar  on` and `keys     modal`, with the cursor line on the first line

#### Scenario: Window beside the default sidebar
- **WHEN** the default configuration is in use on an 80×24 terminal and the settings window opens
- **THEN** the window is 31 columns wide and 5 rows high, and spans columns 24 to 54 and rows 9 to 13 of the 79-column ribbon area

#### Scenario: Saved values shown
- **WHEN** `user/sidebar.lua` holds `return false`, `user/keystyle.lua` holds `return "direct"`, and the settings window opens
- **THEN** its second line shows `sidebar  off` and its third line shows `keys     direct`

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

On the `keys` line, Enter, `h`, `l`, Left and Right SHALL save the other key style, `direct` when the line shows `modal` and `modal` when it shows `direct`, as the key-style capability's "Saved key style" defines.

When `gband.config_dir` is nil, or a file cannot be written, the key SHALL save nothing and the settings window SHALL stay open. The key SHALL raise an error naming the file, such as `user/sidebar.lua`, and the reason, which the client reports as the configuration capability defines for an error in a callback. The setting in use SHALL stay.

#### Scenario: Turn the sidebar off
- **WHEN** no `user/init.lua` exists, the settings window is open on its first line, and the user presses `j` then Enter
- **THEN** `user/sidebar.lua` holds `return false`
- **AND** within a second the configuration reloads and no sidebar is drawn

#### Scenario: Switch the key style
- **WHEN** no `user/init.lua` exists, the modal style is saved, the settings window is open on its third line, and the user presses `l`
- **THEN** `user/keystyle.lua` holds `return "direct"`
- **AND** within a second the configuration reloads with the direct style's bindings

#### Scenario: Next theme
- **WHEN** the active colorscheme is `gruvbox`, the settings window is open on its first line, and the user presses `l`
- **THEN** `gband.colorscheme()` returns `one-dark` and `user/theme.lua` holds `return "one-dark"`

#### Scenario: Theme wraps
- **WHEN** the active colorscheme is `terminal`, the settings window is open on its first line, and the user presses `h`
- **THEN** the last name `gband.settings.themes()` returns is the active colorscheme and is saved

#### Scenario: Dismiss
- **WHEN** the settings window is open and the user presses Escape
- **THEN** the window closes and no file in `user` changes

#### Scenario: Cannot save
- **WHEN** the `user` directory cannot be written and the user presses Enter on the settings window's second line
- **THEN** the client shows an error naming `user/sidebar.lua`
- **AND** the settings window stays open and the sidebar is still drawn

### Requirement: Theme list
Enter on the settings window's `theme` line SHALL open the theme list: a floating plugin window that takes focus, with a border, the title `theme`, and its cursor line on. It SHALL hold one line per name `gband.settings.themes()` returns, in that order. The cursor line SHALL start on the line of the active colorscheme, or on the first line when the active colorscheme is not in the list. Its width SHALL be the smaller of the longest name plus 2 and the ribbon area's width. Its height SHALL be the smaller of the number of names plus 2 and the ribbon area's height. It SHALL be centered in the ribbon area, rounding the offsets down, and SHALL keep the cursor line in view, as the plugin-windows capability defines.

The theme list SHALL take the default movement keys of the plugin-windows capability. Each move of its cursor line to another line SHALL load that line's theme with `gband.colorscheme`, so the ribbon shows it at once. A theme that fails to load SHALL be reported as the colorschemes capability defines, and the active colorscheme SHALL stay as it was.

Enter SHALL save the theme of the cursor line, as "Saved theme" defines, and close the theme list. When that theme is not active, because it failed to load, Enter SHALL save nothing and only close the list. Escape and `q` SHALL load again the colorscheme that was active when the list opened, save nothing, and close the list. Closing the theme list SHALL focus the settings window. The saved theme SHALL apply when the configuration next loads, as the colorschemes capability defines.

#### Scenario: List opens on the active theme
- **WHEN** the active colorscheme is `gruvbox` and the user presses Enter on the settings window's first line
- **THEN** a focused floating plugin window titled `theme` lists the bundled themes, with the cursor line on `gruvbox`

#### Scenario: List size beside the default sidebar
- **WHEN** the default configuration is in use on an 80×24 terminal, no user colorscheme exists, and the theme list opens
- **THEN** the theme list is 22 columns wide and 16 rows high, and spans columns 28 to 49 and rows 4 to 19 of the 79-column ribbon area

#### Scenario: Preview while moving
- **WHEN** the theme list is open on `gruvbox` and the user presses `j`
- **THEN** `gband.colorscheme()` returns `one-dark`
- **AND** `user/theme.lua` is unchanged

#### Scenario: Pick a theme
- **WHEN** the theme list is open on `gruvbox` and the user presses `k` then Enter
- **THEN** the theme list closes and `user/theme.lua` holds `return "nord"`
- **AND** after the reload, `gband.colorscheme()` returns `nord`

#### Scenario: Cancel the preview
- **WHEN** the active colorscheme is `gruvbox`, the theme list is open, and the user presses `j`, `j`, then Escape
- **THEN** the theme list closes, the settings window has focus, and `gband.colorscheme()` returns `gruvbox`
- **AND** `user/theme.lua` is unchanged

### Requirement: Reopen after a save
When a key of the settings window or the theme list saves a setting, the client SHALL remember the line of the settings window that the setting belongs to, until the next load ends. When that load succeeds, the client SHALL open the settings window and focus it, as `gband.settings.open()` does, with its cursor line on that line. This SHALL apply whatever configuration file is in use. A load that fails SHALL forget the line and open nothing, and so SHALL a load that no such save preceded.

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

### Requirement: Offer on the first start
The default configuration SHALL register a handler of `Attached`, as the lua-events capability defines. The handler SHALL call `gband.settings.open()` when `gband.config_dir` is not nil and `gband.settings.theme()`, `gband.settings.sidebar()` and `gband.keystyle.saved()` all return nil. A client emits `Attached` once, after it attaches, and never for a reload, so the offer SHALL open the settings window at most once per start of a client. Until one setting is saved, each start SHALL offer the settings window again.

#### Scenario: First start
- **WHEN** no `user/init.lua`, `user/theme.lua`, `user/sidebar.lua` or `user/keystyle.lua` exists and a client attaches
- **THEN** the settings window is open and focused, with the cursor line on its first line
- **AND** `gband.colorscheme()` returns `gruvbox` and the modal style is in use

#### Scenario: A setting already saved
- **WHEN** only `user/theme.lua` exists, holding `return "nord"`, no `user/init.lua` exists, and a client attaches
- **THEN** no floating plugin window is open

#### Scenario: Offered again on the next start
- **WHEN** the user dismisses the settings window with Escape, detaches, and attaches again
- **THEN** the settings window is open again

#### Scenario: No configuration directory
- **WHEN** the default configuration is evaluated with no configuration directory and `Attached` is emitted
- **THEN** no floating plugin window opens

#### Scenario: Own configuration
- **WHEN** `user/init.lua` binds only `alt+h`, no setting is saved, and a client attaches
- **THEN** no floating plugin window is open

### Requirement: Settings group
The client SHALL define `SettingsLabel` as `{ dim = true }` in the defaults layer the highlights capability defines, before the init file runs.

#### Scenario: Default label style
- **WHEN** `user/colors/blank.lua` is empty and `user/init.lua` calls only `gband.colorscheme("blank")`
- **THEN** `SettingsLabel` resolves to `{ dim = true }`

#### Scenario: Restyle the labels
- **WHEN** `user/init.lua` calls `gband.hl.set("SettingsLabel", { fg = 3 })` and the settings window opens
- **THEN** each label is drawn with foreground 3
