## ADDED Requirements

### Requirement: Configuration directory
The configuration directory SHALL be `gband` under `$XDG_CONFIG_HOME` when that variable holds an absolute path, otherwise `.config/gband` under the user's home directory. It SHALL hold a `defaults` directory and a `user` directory. The default configuration SHALL be built into the executable. Each process SHALL prepare the directory when it starts, before it loads the configuration:

- It SHALL create the configuration directory, `defaults` and `user`, and any missing parents, when they do not exist.
- It SHALL write the default configuration to `defaults/init.lua` when that file does not exist or its content differs from the default configuration, replacing the file in one step so that no process reads it half written.
- It SHALL leave the content of `user` unchanged.

A failure to prepare the directory SHALL be recorded in the process's log, and SHALL NOT stop the process: it loads the configuration as "Configuration file" defines.

#### Scenario: First run
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `/tmp/cfg/gband` does not exist, and a client attaches
- **THEN** `/tmp/cfg/gband/defaults/init.lua` holds the default configuration
- **AND** `/tmp/cfg/gband/user` exists and is empty
- **AND** no error is reported

#### Scenario: Missing user directory
- **WHEN** `gband/defaults/init.lua` exists, `gband/user` does not, and a client attaches
- **THEN** `gband/user` exists and is empty

#### Scenario: Edited defaults are restored
- **WHEN** the user changes `gband/defaults/init.lua` and then a client starts a server
- **THEN** `gband/defaults/init.lua` holds the default configuration again
- **AND** the user's change had no effect on any binding or option

#### Scenario: User files are kept
- **WHEN** `gband/user/init.lua` and `gband/user/notes.txt` exist and a client attaches
- **THEN** both files keep their content

#### Scenario: Read-only configuration directory
- **WHEN** the configuration directory cannot be written, `user/init.lua` does not exist, and a client attaches
- **THEN** the client log records the failure
- **AND** the client attaches with the bindings of the default configuration

## MODIFIED Requirements

### Requirement: Configuration file
The configuration file SHALL be `user/init.lua` in the configuration directory. The client and the server SHALL each load the configuration when they start, after preparing the configuration directory. Loading SHALL evaluate the configuration file alone in one new Lua state, starting from the defaults of "Options" and no key bindings. When the configuration file does not exist, the process SHALL evaluate the default configuration alone in the same way. A missing configuration file SHALL NOT be an error. Each process SHALL apply only the parts of the configuration it owns: the server the column width options, and the client the prefix, the key bindings and the camera policy.

#### Scenario: No configuration file
- **WHEN** no `user/init.lua` exists and a client attaches
- **THEN** no error is reported
- **AND** the key bindings, options and behaviour are those of the default configuration

#### Scenario: XDG_CONFIG_HOME is honoured
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `/tmp/cfg/gband/user/init.lua` sets `width_presets` to `{ 1/4, 1/2 }` and binds `prefix r` to `gband.action.cycle_column_width`, and a client starts a server and cycles a column of width 1/2
- **THEN** the column's width is 1/4

#### Scenario: User file replaces the defaults
- **WHEN** `user/init.lua` holds only `gband.bind("alt+h", gband.action.focus_column_left)` and a client attaches
- **THEN** Alt+H focuses the column to the left
- **AND** Ctrl+A then `q` reaches the focused pane as `\x01` and then `q`

### Requirement: Defaults use the public API
The default configuration SHALL use only the `gband` API that the configuration file can use. Evaluated alone, it SHALL produce the default options of "Options" and the default key bindings of the client-attach capability. The default key bindings SHALL exist only in the default configuration.

#### Scenario: Defaults reproduce the built-in behaviour
- **WHEN** the default configuration is evaluated alone
- **THEN** the options equal the defaults in the "Options" table
- **AND** the bindings equal the client-attach capability's default table, entry for entry

#### Scenario: Copied defaults load unchanged
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua`
- **THEN** loading succeeds
- **AND** the options and bindings equal those of the default configuration

### Requirement: Bind and unbind keys
`gband.bind(keys, action)` SHALL bind `keys` to `action`. `keys` SHALL be one key name, a direct binding, or `prefix` followed by a space and one key name, a prefix binding, where `prefix` stands for the key the `prefix` option names when the bindings are used. `prefix prefix` SHALL bind the prefix key pressed twice. `action` SHALL be a value from `gband.action` or a Lua function. Binding keys that are already bound SHALL replace the earlier binding. `gband.unbind(keys)` SHALL remove the binding of `keys`, and SHALL do nothing when `keys` is unbound.

A key list of more than two keys, two keys whose first is not `prefix`, an invalid key name, or an action that is neither a `gband.action` value nor a function SHALL be a configuration error. Once the configuration is evaluated, a direct binding of the key the `prefix` option names SHALL be a configuration error at the line of that binding.

#### Scenario: Direct binding
- **WHEN** `user/init.lua` calls `gband.bind("alt+h", gband.action.focus_column_left)`
- **THEN** Alt+H focuses the column to the left without the prefix

#### Scenario: Rebind a key
- **WHEN** `user/init.lua` binds `prefix q` to `gband.action.close_pane` and later binds `prefix q` to `gband.action.detach`
- **THEN** Ctrl+A then `q` detaches, and no binding closes the pane

#### Scenario: Unbind a key
- **WHEN** `user/init.lua` binds `prefix q` to `gband.action.close_pane` and later calls `gband.unbind("prefix q")`
- **THEN** Ctrl+A then `q` discards both keys

#### Scenario: Prefix changed after binding
- **WHEN** `user/init.lua` binds `prefix h` to `gband.action.focus_column_left` and later sets `prefix` to `"ctrl+b"`
- **THEN** Ctrl+B then `h` focuses the column to the left
- **AND** Ctrl+A reaches the focused pane as `\x01`

#### Scenario: Direct binding of the prefix key
- **WHEN** line 4 of `user/init.lua` binds `ctrl+a` while the prefix is `ctrl+a`
- **THEN** loading fails with an error at `user/init.lua` line 4

#### Scenario: Key chain too long
- **WHEN** `user/init.lua` binds `prefix w q`
- **THEN** loading fails with an error naming `prefix w q`

#### Scenario: Not an action
- **WHEN** `user/init.lua` calls `gband.bind("alt+h", "focus_column_left")`
- **THEN** loading fails with an error naming the binding of `alt+h`

### Requirement: Configuration errors
A configuration error SHALL be reported as the configuration file's path, a colon, the line, a colon and a message. The line SHALL be the line of a syntax error, the line where a runtime error was raised, or the line of the `gband.set` or `gband.bind` call that received the invalid value. Each process SHALL record every configuration error in its log. The client SHALL also show the latest configuration error on its terminal's bottom row, over the ribbon and cut to the terminal's width, until the configuration next loads without error. The banner SHALL NOT change the screen area the client reports.

#### Scenario: Syntax error
- **WHEN** `XDG_CONFIG_HOME` is unset, the home directory is `/home/u`, line 12 of `user/init.lua` holds a syntax error, and a client attaches
- **THEN** the client's bottom row shows the error, beginning `/home/u/.config/gband/user/init.lua:12:`
- **AND** the client log and the server log record it

#### Scenario: Banner cleared by a good load
- **WHEN** the client shows a configuration error and the user fixes `user/init.lua`
- **THEN** the banner disappears once the configuration reloads

### Requirement: Last good configuration
Loading SHALL apply all of a configuration or none of it. When loading fails, the process SHALL keep the configuration it last loaded without error, or the default configuration alone when it has loaded none.

#### Scenario: Broken file at start
- **WHEN** `user/init.lua` sets `prefix` to `"ctrl+b"` and then raises an error, and a client attaches
- **THEN** the prefix is Ctrl+A and the default bindings apply

#### Scenario: Broken edit keeps the running configuration
- **WHEN** `user/init.lua` binds `alt+h` and has loaded, and the user saves a version that binds `alt+j` and has a syntax error
- **THEN** Alt+H still focuses the column to the left
- **AND** Alt+J reaches the focused pane

### Requirement: Reload on change
Each process SHALL watch the configuration file. Within one second of the file being created, written, replaced or removed, the process SHALL load the configuration again in a new Lua state and, when loading succeeds, use the new configuration from then on. Changes to `defaults/init.lua` SHALL NOT cause a reload. Options the server owns SHALL apply to session actions received after the reload, and no column's width SHALL change because of a reload. A prefix sequence in progress SHALL end without effect when the client reloads.

#### Scenario: New binding without restart
- **WHEN** a client is attached and the user adds `gband.bind("alt+l", gband.action.focus_column_right)` to `user/init.lua`
- **THEN** within a second Alt+L focuses the column to the right

#### Scenario: Editor replaces the file
- **WHEN** an editor saves `user/init.lua` by writing a new file and renaming it over the old one
- **THEN** the processes load the new content

#### Scenario: New default width
- **WHEN** a session holds a column of width 1/2 and the user sets `default_column_width` to `1/3` in `user/init.lua`
- **THEN** that column keeps width 1/2
- **AND** a pane opened after the reload has a column of width 1/3

#### Scenario: File removed
- **WHEN** `user/init.lua` bound `alt+h` and the user deletes it
- **THEN** the default configuration alone applies and Alt+H reaches the focused pane

#### Scenario: Defaults file edited while running
- **WHEN** a client is attached and the user saves a change to `defaults/init.lua`
- **THEN** the bindings stay as they were
