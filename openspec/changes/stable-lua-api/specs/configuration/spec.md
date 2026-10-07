## MODIFIED Requirements

### Requirement: Configuration directory
The configuration directory SHALL be `gband` under `$XDG_CONFIG_HOME` when that variable holds an absolute path, otherwise `.config/gband` under the user's home directory. It SHALL hold a `defaults` directory and a `user` directory. The default client configuration, the key style presets of the key-style capability, the default server configuration, and the bundled modules and colorschemes SHALL be built into the executable. Each process SHALL prepare the directory when it starts, before it loads the configuration:

- It SHALL create the configuration directory, `defaults`, `defaults/keystyle`, `defaults/lua/gband`, `defaults/colors` and `user`, and any missing parents, when they do not exist.
- It SHALL write the default client configuration to `defaults/init.lua`, the modal preset to `defaults/keystyle/modal.lua`, the direct preset to `defaults/keystyle/direct.lua`, the default server configuration to `defaults/server.lua`, and the bundled modules and colorschemes where the lua-api capability's "Bundled sources on disk" places them, each when that file does not exist or its content differs from the built-in one, replacing the file in one step so that no process reads it half written.
- It SHALL leave the content of `user` unchanged.

The files under `defaults` SHALL be copies for the user to read. Loading SHALL use the built-in text, never these files.

A failure to prepare the directory SHALL be recorded in the process's log, and SHALL NOT stop the process: it loads the configuration as "Configuration file" and the server-runtime capability define.

#### Scenario: First run
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `/tmp/cfg/gband` does not exist, and a client attaches
- **THEN** `/tmp/cfg/gband/defaults/init.lua` holds the default client configuration
- **AND** `/tmp/cfg/gband/defaults/keystyle/modal.lua` and `/tmp/cfg/gband/defaults/keystyle/direct.lua` hold the two presets
- **AND** `/tmp/cfg/gband/defaults/server.lua` holds the default server configuration
- **AND** `/tmp/cfg/gband/defaults/lua/gband/keylist.lua` and `/tmp/cfg/gband/defaults/colors/gruvbox.lua` hold the bundled key list and `gruvbox` theme
- **AND** `/tmp/cfg/gband/user` exists and is empty
- **AND** no error is reported

#### Scenario: Missing user directory
- **WHEN** `gband/defaults/init.lua` exists, `gband/user` does not, and a client attaches
- **THEN** `gband/user` exists and is empty

#### Scenario: Edited defaults are restored
- **WHEN** the user changes `gband/defaults/init.lua`, `gband/defaults/keystyle/modal.lua`, `gband/defaults/server.lua` and `gband/defaults/lua/gband/sidebar.lua`, and then a client starts a server
- **THEN** the four files hold the built-in text again
- **AND** the user's changes had no effect on any binding, option or bar

#### Scenario: User files are kept
- **WHEN** `gband/user/init.lua`, `gband/user/server.lua` and `gband/user/notes.txt` exist and a client attaches
- **THEN** all three files keep their content

#### Scenario: Read-only configuration directory
- **WHEN** the configuration directory cannot be written, `user/init.lua` does not exist, and a client attaches
- **THEN** the client log records the failure
- **AND** the client attaches with the bindings of the default configuration
