## MODIFIED Requirements

### Requirement: Settings window keys
j, k, the arrow keys Up and Down, and the other default movement keys of the plugin-windows capability SHALL move the cursor line. Escape and `q` SHALL close the settings window and save nothing, as the plugin-windows capability defines for a floating plugin window with no `keys` entry for them.

On the `theme` line:
- Enter SHALL open the theme list, as "Theme list" defines.
- `l` and Right SHALL load the theme that follows the active colorscheme in the list `gband.settings.themes()` returns, and `h` and Left the theme before it. The list wraps at both ends. When the active colorscheme is not in the list, `l` and Right SHALL load the first theme and `h` and Left the last. A theme that loads SHALL be saved, as "Saved theme" defines. A theme that fails to load SHALL be reported as the colorschemes capability defines and SHALL NOT be saved.

On the `sidebar` line, Enter, `h`, `l`, Left and Right SHALL save the other value: `false` when the line shows `on`, and `true` when it shows `off`.

On the `keys` line, Enter, `l` and Right SHALL save the next key style in the order `modal`, `direct`, `floating`, and `modal` when the line shows `floating`. `h` and Left SHALL save the previous key style in that order, and `floating` when the line shows `modal`. Each SHALL save as the key-style capability's "Saved key style" defines.

On the `I on new` line, Enter, `h`, `l`, Left and Right SHALL save the other value, as "Saved I on new" defines: `false` when the line shows `on`, and `true` when it shows `off`.

On the `reload` line, Enter SHALL dispatch `gband.action.reload`, as the configuration capability's "Forced reload" defines, and save nothing. `h`, `l`, Left and Right SHALL do nothing there.

When `gband.config_dir` is nil, or a file cannot be written, the key SHALL save nothing and the settings window SHALL stay open. The key SHALL raise an error naming the file, such as `user/sidebar.lua`, and the reason, which the client reports as the configuration capability defines for an error in a callback. The setting in use SHALL stay.

#### Scenario: Turn the sidebar off
- **WHEN** no `user/init.lua` exists, the settings window is open on its first line, and the user presses `j` then Enter
- **THEN** `user/sidebar.lua` holds `return false`
- **AND** within two seconds the configuration reloads and no sidebar is drawn

#### Scenario: Switch the key style
- **WHEN** no `user/init.lua` exists, the modal style is saved, the settings window is open on its third line, and the user presses `l`
- **THEN** `user/keystyle.lua` holds `return "direct"`
- **AND** within two seconds the configuration reloads with the direct style's bindings, and the reopened settings window holds four lines

#### Scenario: Switch to the floating style
- **WHEN** no `user/init.lua` exists, the direct style is saved, the settings window is open on its third line, and the user presses Enter
- **THEN** `user/keystyle.lua` holds `return "floating"`
- **AND** within two seconds the configuration reloads with the floating style's bindings, and the reopened settings window holds four lines

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
