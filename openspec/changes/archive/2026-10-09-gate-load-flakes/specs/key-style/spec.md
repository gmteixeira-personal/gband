## MODIFIED Requirements

### Requirement: Saved key style
The saved key style SHALL live in the file `user/keystyle.lua` of the configuration directory. Saving a style SHALL write the text `return "<style>"` followed by a newline to that file. It SHALL create the file or replace it in one step, so no process reads it half written. Saving SHALL change no other file in `user`, and SHALL never write `user/init.lua`. The settings window's `keys` line saves it, as the settings capability defines.

`gband.keystyle.saved()` SHALL evaluate `user/keystyle.lua` as a Lua chunk in text mode with an empty environment, and SHALL return the chunk's value when it is `"modal"`, `"direct"` or `"floating"`. It SHALL return nil when `gband.config_dir` is nil, and when the file is missing, cannot be read, does not compile, raises an error or returns any other value. It SHALL raise no error and report none. It SHALL be callable while the configuration loads and in any callback.

Loading SHALL evaluate `user/keystyle.lua` only through `gband.keystyle.saved()`, never as a configuration file, a plugin file or a module. Saving it SHALL reload the configuration, as the configuration capability's "Reload on change" defines for a file whose name ends in `.lua`. The reloaded configuration SHALL apply the saved style where it calls `gband.keystyle.use()` with no argument.

#### Scenario: Saved file
- **WHEN** the settings window saves the direct style
- **THEN** `user/keystyle.lua` holds `return "direct"` and a newline

#### Scenario: Written by hand
- **WHEN** `user/keystyle.lua` holds `return 'modal'` and a callback calls `gband.keystyle.saved()`
- **THEN** the call returns `modal`

#### Scenario: Floating saved
- **WHEN** `user/keystyle.lua` holds `return "floating"` and a callback calls `gband.keystyle.saved()`
- **THEN** the call returns `floating`

#### Scenario: Unknown saved value
- **WHEN** `user/keystyle.lua` holds `return "vi"`, no `user/init.lua` exists, and a client attaches
- **THEN** no error is reported and the modal style is in use

#### Scenario: Broken file is not a configuration error
- **WHEN** `user/keystyle.lua` holds `error("boom")` and `user/init.lua` calls `gband.keystyle.use()`
- **THEN** loading succeeds with no error and the modal style is in use

#### Scenario: Saving reloads the configuration
- **WHEN** a client is attached with no `user/init.lua`, the modal style is in use, and the settings window saves the direct style
- **THEN** within two seconds Ctrl+Space then `h` focuses the column to the left and the keys that follow reach the focused window

#### Scenario: User file untouched
- **WHEN** `user/init.lua` calls `gband.keystyle.use()` and binds `alt+h`, and the user picks the direct style in the settings window
- **THEN** `user/init.lua` keeps its content
- **AND** after the reload Alt+H still focuses the column to the left and the direct style's bindings apply
