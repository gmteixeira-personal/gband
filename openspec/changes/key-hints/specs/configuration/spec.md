## MODIFIED Requirements

### Requirement: Defaults use the public API
The default configuration SHALL use only the `gband` API that the configuration file can use. It SHALL make every key binding with `gband.keymap.set`, giving each the description of the action it binds, as the actions capability lists them. Evaluated alone, it SHALL produce the declared defaults of the options and the default key bindings of the client-attach capability, and SHALL set up, with `gband.plugin` and no options, the bundled segment plugins `gband.statusline.band`, `gband.statusline.mode`, `gband.statusline.hints` and `gband.statusline.position`, in that order. The default key bindings and the setup of the segment plugins SHALL exist only in the default configuration.

#### Scenario: Defaults reproduce the built-in behaviour
- **WHEN** the default configuration is evaluated alone
- **THEN** the options equal the defaults in the "Options" table
- **AND** the bindings equal the client-attach capability's default table, entry for entry
- **AND** `gband.ui.statusline.list()` names exactly the components `band`, `hints`, `mode` and `position`

#### Scenario: Every default binding is described
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** every entry's `desc` is the description `gband.action.list()` gives its action

#### Scenario: Copied defaults load unchanged
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua`
- **THEN** loading succeeds
- **AND** the options and bindings equal those of the default configuration
