## ADDED Requirements

### Requirement: Configuration directory in Lua
`gband.config_dir` SHALL hold the path of the configuration directory, as "Configuration directory" defines it, as a string, on the client and on the server. It SHALL be nil when the process has no configuration directory, because neither `XDG_CONFIG_HOME` nor the home directory gives one, and when the default configuration is evaluated alone. Assigning `gband.config_dir` SHALL NOT change the directory the process loads its configuration from or watches.

#### Scenario: Path from XDG_CONFIG_HOME
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg` and `user/init.lua` reads `gband.config_dir`
- **THEN** it reads `/tmp/cfg/gband`

#### Scenario: Path from the home directory
- **WHEN** `XDG_CONFIG_HOME` is unset, the home directory is `/home/u`, and `user/server.lua` reads `gband.config_dir`
- **THEN** it reads `/home/u/.config/gband`

#### Scenario: Defaults evaluated alone
- **WHEN** the default configuration is evaluated alone and a callback reads `gband.config_dir`
- **THEN** it reads nil

## MODIFIED Requirements

### Requirement: Configuration directory
The configuration directory SHALL be `gband` under `$XDG_CONFIG_HOME` when that variable holds an absolute path, otherwise `.config/gband` under the user's home directory. It SHALL hold a `defaults` directory and a `user` directory. The default client configuration, the key style presets of the key-style capability and the default server configuration SHALL be built into the executable. Each process SHALL prepare the directory when it starts, before it loads the configuration:

- It SHALL create the configuration directory, `defaults`, `defaults/keystyle` and `user`, and any missing parents, when they do not exist.
- It SHALL write the default client configuration to `defaults/init.lua`, the modal preset to `defaults/keystyle/modal.lua`, the direct preset to `defaults/keystyle/direct.lua`, and the default server configuration to `defaults/server.lua`, each when that file does not exist or its content differs from the built-in one, replacing the file in one step so that no process reads it half written.
- It SHALL leave the content of `user` unchanged.

The files under `defaults` SHALL be copies for the user to read. Loading SHALL use the built-in text, never these files.

A failure to prepare the directory SHALL be recorded in the process's log, and SHALL NOT stop the process: it loads the configuration as "Configuration file" and the server-runtime capability define.

#### Scenario: First run
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `/tmp/cfg/gband` does not exist, and a client attaches
- **THEN** `/tmp/cfg/gband/defaults/init.lua` holds the default client configuration
- **AND** `/tmp/cfg/gband/defaults/keystyle/modal.lua` and `/tmp/cfg/gband/defaults/keystyle/direct.lua` hold the two presets
- **AND** `/tmp/cfg/gband/defaults/server.lua` holds the default server configuration
- **AND** `/tmp/cfg/gband/user` exists and is empty
- **AND** no error is reported

#### Scenario: Missing user directory
- **WHEN** `gband/defaults/init.lua` exists, `gband/user` does not, and a client attaches
- **THEN** `gband/user` exists and is empty

#### Scenario: Edited defaults are restored
- **WHEN** the user changes `gband/defaults/init.lua`, `gband/defaults/keystyle/modal.lua` and `gband/defaults/server.lua`, and then a client starts a server
- **THEN** the three files hold the built-in text again
- **AND** the user's changes had no effect on any binding or option

#### Scenario: User files are kept
- **WHEN** `gband/user/init.lua`, `gband/user/server.lua` and `gband/user/notes.txt` exist and a client attaches
- **THEN** all three files keep their content

#### Scenario: Read-only configuration directory
- **WHEN** the configuration directory cannot be written, `user/init.lua` does not exist, and a client attaches
- **THEN** the client log records the failure
- **AND** the client attaches with the bindings of the default configuration

### Requirement: Defaults use the public API
The default configuration SHALL use only the `gband` API that the configuration file can use. It SHALL set the options to their declared defaults. It SHALL make its key bindings only by calling `gband.keystyle.use()` with no argument, as the key-style capability defines, so the saved key style, or the modal key style when none is saved, makes them. It SHALL make no other binding and declare no mode. It SHALL set up the key list plugin `gband.keylist` and the Lua prompt plugin `gband.prompt` only through that call. It SHALL then set up, with `gband.plugin` and no options, the bundled error list plugin `gband.errors`, then the bundled status line plugin `gband.statusline`, then the bundled segment plugins `gband.statusline.band`, `gband.statusline.mode`, `gband.statusline.hints` and `gband.statusline.position`, in that order, and register the offer of the key-style capability's "Offer on the first start". Evaluated alone, with no configuration directory, it SHALL produce the declared defaults of the options and the default key bindings of the client-attach capability for the modal key style.

The default key bindings and the declaration of the `navigation` mode SHALL exist only in the key style presets. The setup of the error list, status line and segment plugins and the offer of the chooser SHALL exist only in the default configuration.

#### Scenario: Defaults reproduce the built-in behaviour
- **WHEN** the default configuration is evaluated alone
- **THEN** the options equal the defaults in the "Options" table
- **AND** the bindings equal the client-attach capability's default table for the modal key style, entry for entry
- **AND** `gband.ui.statusline.list()` names exactly the components `band`, `hints`, `mode` and `position`
- **AND** `gband.bar.list()` holds exactly one bar, `statusline`, on the side `left`, 20 columns wide on an 80×24 terminal

#### Scenario: Every default binding is described
- **WHEN** the default configuration is evaluated with either key style saved and `gband.keymap.list("prefix")` is read
- **THEN** every entry with an action has the description `gband.action.list()` gives its action
- **AND** every entry without an action has the description the client-attach capability's default table gives its key

#### Scenario: Navigation mode declared by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.label("prefix")` is read
- **THEN** it returns `navigation`

#### Scenario: Direct style from the saved choice
- **WHEN** `user/keystyle.lua` holds `return "direct"`, no `user/init.lua` exists, and a client attaches
- **THEN** `gband.keymap.label("prefix")` returns `prefix`
- **AND** the bindings equal the client-attach capability's bindings for the direct key style, entry for entry

#### Scenario: Key list set up by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** the entry for `?` has the action `keylist.open`
- **AND** it comes after the entry for `R` and before the entry for `D`

#### Scenario: Prompt set up by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** the entry for `:` has the action `prompt.open` and the description `run Lua`
- **AND** it comes right after the entry for `?` and before the entry for `D`
- **AND** `gband.keymap.list("root")` holds no entry

#### Scenario: No binding in the defaults file
- **WHEN** `defaults/init.lua` is read
- **THEN** it calls `gband.keystyle.use()` once
- **AND** it calls none of `gband.keymap.set`, `gband.bind` and `gband.keymap.mode`

#### Scenario: Copied defaults load unchanged
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua`
- **THEN** loading succeeds
- **AND** the options, bindings and modes equal those of the default configuration with the same saved key style
