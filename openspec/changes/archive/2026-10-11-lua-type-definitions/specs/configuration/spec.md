## MODIFIED Requirements

### Requirement: Configuration directory
The configuration directory SHALL be `gband` under `$XDG_CONFIG_HOME` when that variable holds an absolute path, otherwise `.config/gband` under the user's home directory. It SHALL hold a `defaults` directory and a `user` directory. The default client configuration, the key style presets of the key-style capability, the default server configuration, the bundled modules and colorschemes, and the type definition file of the lua-api capability's "Type definitions" SHALL be built into the executable. Each process SHALL prepare the directory when it starts, before it loads the configuration:

- It SHALL create the configuration directory, `defaults`, `defaults/keystyle`, `defaults/lua/gband`, `defaults/colors`, `defaults/meta` and `user`, and any missing parents, when they do not exist.
- It SHALL write the default client configuration to `defaults/init.lua`, the modal preset to `defaults/keystyle/modal.lua`, the direct preset to `defaults/keystyle/direct.lua`, the floating preset to `defaults/keystyle/floating.lua`, the default server configuration to `defaults/server.lua`, the type definition file to `defaults/meta/gband.lua`, and the bundled modules and colorschemes where the lua-api capability's "Bundled sources on disk" places them, each when that file does not exist or its content differs from the built-in one, replacing the file in one step so that no process reads it half written.
- It SHALL write the built-in editor settings to `.luarc.json` at the top of the configuration directory when no file of that name exists there, creating the file in one step so that no process reads it half written. It SHALL NOT change a `.luarc.json` that exists, whatever its content. The built-in editor settings SHALL be a JSON object that sets lua-language-server's runtime version to Lua 5.4.
- It SHALL leave the content of `user` unchanged.

The files under `defaults` SHALL be copies for the user to read. Loading SHALL use the built-in text, never these files.

A failure to prepare the directory SHALL be recorded in the process's log, and SHALL NOT stop the process: it loads the configuration as "Configuration file" and the server-runtime capability define.

#### Scenario: First run
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `/tmp/cfg/gband` does not exist, and a client attaches
- **THEN** `/tmp/cfg/gband/defaults/init.lua` holds the default client configuration
- **AND** `/tmp/cfg/gband/defaults/keystyle/modal.lua`, `/tmp/cfg/gband/defaults/keystyle/direct.lua` and `/tmp/cfg/gband/defaults/keystyle/floating.lua` hold the three presets, and `defaults/keystyle` holds no other file
- **AND** `/tmp/cfg/gband/defaults/server.lua` holds the default server configuration
- **AND** `/tmp/cfg/gband/defaults/lua/gband/keylist.lua`, `/tmp/cfg/gband/defaults/lua/gband/desktop.lua` and `/tmp/cfg/gband/defaults/colors/gruvbox.lua` hold the bundled key list, the bundled desktop plugin and the `gruvbox` theme
- **AND** `/tmp/cfg/gband/defaults/meta/gband.lua` holds the type definition file, and `defaults/meta` holds no other file
- **AND** `/tmp/cfg/gband/.luarc.json` holds the built-in editor settings
- **AND** `/tmp/cfg/gband/user` exists and is empty
- **AND** no error is reported

#### Scenario: Missing user directory
- **WHEN** `gband/defaults/init.lua` exists, `gband/user` does not, and a client attaches
- **THEN** `gband/user` exists and is empty

#### Scenario: Edited defaults are restored
- **WHEN** the user changes `gband/defaults/init.lua`, `gband/defaults/keystyle/modal.lua`, `gband/defaults/server.lua`, `gband/defaults/lua/gband/sidebar.lua` and `gband/defaults/meta/gband.lua`, and then a client starts a server
- **THEN** the five files hold the built-in text again
- **AND** the user's changes had no effect on any binding, option or bar

#### Scenario: User files are kept
- **WHEN** `gband/user/init.lua`, `gband/user/server.lua` and `gband/user/notes.txt` exist and a client attaches
- **THEN** all three files keep their content

#### Scenario: Existing editor settings are kept
- **WHEN** `gband/.luarc.json` holds `{"diagnostics.globals": ["vim"]}` and a client starts a server
- **THEN** `gband/.luarc.json` still holds `{"diagnostics.globals": ["vim"]}`

#### Scenario: Removed editor settings are written again
- **WHEN** the user deletes `gband/.luarc.json` and a client attaches
- **THEN** `gband/.luarc.json` holds the built-in editor settings

#### Scenario: Read-only configuration directory
- **WHEN** the configuration directory cannot be written, `user/init.lua` does not exist, and a client attaches
- **THEN** the client log records the failure
- **AND** the client attaches with the bindings of the default configuration
