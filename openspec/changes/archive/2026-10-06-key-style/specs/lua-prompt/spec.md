## MODIFIED Requirements

### Requirement: Default setup
Each key style preset of the key-style capability SHALL set up `gband.prompt` with `gband.plugin` and no options, after `gband.keylist` and before it makes its key bindings. It SHALL bind `prefix :` to `prompt.open`, as the client-attach capability's default table lists it for both key styles. The default configuration SHALL set up `gband.prompt` only through `gband.keystyle.use()`. Neither the presets nor the default configuration SHALL bind a key in `root`, so `:` typed in interactive mode SHALL reach the focused window.

#### Scenario: Colon in interactive mode
- **WHEN** the default configuration is in use, `root` is active, and the user types `echo a:b` and Enter at a shell prompt
- **THEN** the focused window prints `a:b` and no prompt opens

#### Scenario: Hint for the prompt
- **WHEN** the default configuration is in use with the modal key style saved, the client's terminal is 80×60, and the user presses Ctrl+Space
- **THEN** the hints segment shows the hint `: run Lua` right after the hint `? list the keys` and right before the hint `D detach`

#### Scenario: Prompt in the key list
- **WHEN** the default configuration is in use and the key list opens
- **THEN** a line shows `:` and `run Lua` in `PluginWindow`, between the lines for `?` and `D`

#### Scenario: Prompt with the direct key style
- **WHEN** the direct key style is saved, no `user/init.lua` exists, and the user presses Ctrl+Space then `:`
- **THEN** the Lua prompt opens and has focus, and `root` is active
