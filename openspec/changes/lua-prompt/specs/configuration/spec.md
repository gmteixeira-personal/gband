## MODIFIED Requirements

### Requirement: Defaults use the public API
The default configuration SHALL use only the `gband` API that the configuration file can use. It SHALL declare `prefix` a mode with the label `navigation`, with `gband.keymap.mode`. It SHALL make every key binding with `gband.keymap.set`. A binding to an action SHALL take the description of the action it binds, as the actions capability lists them or as `gband.action.list()` gives it for a registered action. A binding to a Lua function SHALL take the description the client-attach capability's default table gives its key. Evaluated alone, it SHALL produce the declared defaults of the options and the default key bindings of the client-attach capability. It SHALL set up, with `gband.plugin` and no options, the bundled key list plugin `gband.keylist` and the bundled prompt plugin `gband.prompt`, in that order, before it makes its key bindings, and the bundled segment plugins `gband.statusline.band`, `gband.statusline.mode`, `gband.statusline.hints` and `gband.statusline.position`, in that order. The default key bindings, the declaration of the `navigation` mode and the setup of the bundled plugins SHALL exist only in the default configuration.

#### Scenario: Defaults reproduce the built-in behaviour
- **WHEN** the default configuration is evaluated alone
- **THEN** the options equal the defaults in the "Options" table
- **AND** the bindings equal the client-attach capability's default table, entry for entry
- **AND** `gband.ui.statusline.list()` names exactly the components `band`, `hints`, `mode` and `position`

#### Scenario: Every default binding is described
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** every entry with an action has the description `gband.action.list()` gives its action
- **AND** every entry without an action has the description the client-attach capability's default table gives its key

#### Scenario: Navigation mode declared by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.label("prefix")` is read
- **THEN** it returns `navigation`

#### Scenario: Key list set up by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** the entry for `?` has the action `keylist.open`
- **AND** it comes after the entry for `R` and before the entry for `D`

#### Scenario: Prompt set up by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** the entry for `:` has the action `prompt.open` and the description `run Lua`
- **AND** it comes right after the entry for `?` and before the entry for `D`
- **AND** `gband.keymap.list("root")` holds no entry

#### Scenario: Copied defaults load unchanged
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua`
- **THEN** loading succeeds
- **AND** the options, bindings and modes equal those of the default configuration
