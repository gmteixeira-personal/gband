## MODIFIED Requirements

### Requirement: Configuration directory
The configuration directory SHALL be `gband` under `$XDG_CONFIG_HOME` when that variable holds an absolute path, otherwise `.config/gband` under the user's home directory. It SHALL hold a `defaults` directory and a `user` directory. The default client configuration and the default server configuration SHALL be built into the executable. Each process SHALL prepare the directory when it starts, before it loads the configuration:

- It SHALL create the configuration directory, `defaults` and `user`, and any missing parents, when they do not exist.
- It SHALL write the default client configuration to `defaults/init.lua`, and the default server configuration to `defaults/server.lua`, when that file does not exist or its content differs from the built-in one, replacing the file in one step so that no process reads it half written.
- It SHALL leave the content of `user` unchanged.

A failure to prepare the directory SHALL be recorded in the process's log, and SHALL NOT stop the process: it loads the configuration as "Configuration file" and the server-runtime capability define.

#### Scenario: First run
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `/tmp/cfg/gband` does not exist, and a client attaches
- **THEN** `/tmp/cfg/gband/defaults/init.lua` holds the default client configuration
- **AND** `/tmp/cfg/gband/defaults/server.lua` holds the default server configuration
- **AND** `/tmp/cfg/gband/user` exists and is empty
- **AND** no error is reported

#### Scenario: Missing user directory
- **WHEN** `gband/defaults/init.lua` exists, `gband/user` does not, and a client attaches
- **THEN** `gband/user` exists and is empty

#### Scenario: Edited defaults are restored
- **WHEN** the user changes `gband/defaults/init.lua` and `gband/defaults/server.lua`, and then a client starts a server
- **THEN** both files hold the built-in configurations again
- **AND** the user's changes had no effect on any binding or option

#### Scenario: User files are kept
- **WHEN** `gband/user/init.lua`, `gband/user/server.lua` and `gband/user/notes.txt` exist and a client attaches
- **THEN** all three files keep their content

#### Scenario: Read-only configuration directory
- **WHEN** the configuration directory cannot be written, `user/init.lua` does not exist, and a client attaches
- **THEN** the client log records the failure
- **AND** the client attaches with the bindings of the default configuration

### Requirement: Configuration file
The client configuration file SHALL be `user/init.lua` in the configuration directory. Each client SHALL load its configuration when it starts, after preparing the configuration directory. Loading SHALL use one new Lua state, starting from the declared defaults of the client options and no key bindings, and SHALL evaluate the init file and then the `client.lua` files the plugins capability sources from the runtimepath. The init file SHALL be the configuration file when it exists, and the default client configuration otherwise. Before the init file, loading SHALL install only the client's `gband` API, which includes the status line container and its built-in groups, as the status-line capability defines, and the bundled `default` colorscheme, as the colorschemes capability defines. Nothing of the default configuration SHALL be evaluated before the init file: the configuration file replaces the default configuration rather than adding to it. A missing configuration file SHALL NOT be an error. The client SHALL use the prefix, the key tables, the camera policy, the status line options, the notification style, the highlight groups, the status line components and the callbacks. The server SHALL NOT evaluate the client configuration; it loads the server configuration as the server-runtime capability defines.

#### Scenario: No configuration file
- **WHEN** no `user/init.lua` exists, no plugin directory exists, and a client attaches
- **THEN** no error is reported
- **AND** the key bindings, options and behaviour are those of the default configuration

#### Scenario: XDG_CONFIG_HOME is honoured
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `/tmp/cfg/gband/user/server.lua` sets `width_presets` to `{ 1/4, 1/2 }`, `/tmp/cfg/gband/user/init.lua` binds `prefix r` to `gband.action.cycle_column_width`, and a client starts a server and cycles a column of width 1/2
- **THEN** the column's width is 1/4

#### Scenario: User file replaces the defaults
- **WHEN** `user/init.lua` holds only `gband.bind("alt+h", gband.action.focus_column_left)` and a client attaches
- **THEN** Alt+H focuses the column to the left
- **AND** Ctrl+Space then `q` reaches the focused pane as `\x00` and then `q`

#### Scenario: Plugin files see the init file's options
- **WHEN** `user/init.lua` sets `gband.opt.prefix` to `"ctrl+b"` and a plugin's `client.lua` reads `gband.opt.prefix`
- **THEN** `client.lua` reads `"ctrl+b"`

#### Scenario: Plugin sets a width option
- **WHEN** a plugin's `server.lua` sets `gband.opt.default_column_width` to `1/3`, and a client starts a server and opens a pane
- **THEN** the new pane's column has width 1/3

### Requirement: Options
Every option SHALL have a name, a type, a declared default and a description, and SHALL belong to one side. The built-in client options SHALL be:

| option | value | default |
|---|---|---|
| `prefix` | one key, as "Key names" defines | `"ctrl+space"` |
| `center_focused_column` | `"never"`, `"always"` or `"on-overflow"` | `"never"` |
| `statusline_position` | `"bottom"`, `"top"` or `"off"` | `"bottom"` |
| `statusline_height` | an integer from 1 to 8 | `1` |
| `statusline_separator` | a string | `" │ "` |
| `notify_style` | `"osc9"`, `"osc777"`, `"bell"` or `"none"` | `"osc9"` |

The built-in server options SHALL be:

| option | value | default |
|---|---|---|
| `default_column_width` | a number greater than 0 and at most 10000 | `1/2` |
| `width_presets` | a list of one or more numbers, each greater than 0 and at most 10000 | `{ 1/3, 1/2, 2/3 }` |

A process SHALL know only the options of its own side, and options declared in its own Lua state. A width SHALL be read as the fraction closest to the number whose denominator is at most 100, in lowest terms, so `1/3` reads as 1/3. The width presets SHALL be held in ascending order with duplicates removed.

`gband.set` SHALL take one table of options. Each call SHALL change only the options the table names, and a later call SHALL override an earlier one. An unknown option name, a value of the wrong type, a value out of range or an unknown camera policy given to `gband.set` SHALL be a configuration error naming the option.

Reading `gband.opt.<name>` SHALL return the option's current value: a key name for `prefix`, a number for a width, a list of numbers in ascending order for `width_presets`, a string for `center_focused_column`, `statusline_position`, `statusline_separator` and `notify_style`, an integer for `statusline_height`, and the value as set for a declared option. Assigning `gband.opt.<name>` SHALL set the option. A value that the option's type rejects SHALL be reported as a configuration error at the line of the assignment, SHALL NOT fail the load, and SHALL set the option to its declared default. An assignment to a name no option declares SHALL be held until an option of that name is declared later in the same load, and then validated; one still undeclared when loading finishes SHALL be reported as a configuration error at the line of the assignment, and SHALL NOT fail the load. The error for a built-in option of the other side SHALL name the side that owns it.

`gband.opt.declare(name, spec)` SHALL declare an option under its full name, as the plugins capability defines, and return that full name. `spec.type` SHALL be `"boolean"`, `"integer"`, `"number"` or `"string"`. `spec.values`, optional, SHALL list the only values allowed. `spec.default` SHALL be a valid value, and `spec.desc` an optional description. Declaring a name already declared, an unknown type, or an invalid default SHALL be an error at the line of the call. `gband.opt.list()` SHALL return one table per option of its side, built-in and declared, in ascending byte order of names, each holding `name`, `type`, `default`, `value` and `desc`.

Options SHALL be set and declared only while the configuration loads. Setting or declaring an option later SHALL be an error. Each process SHALL use the options as they stand when loading finishes, whichever of `gband.set` and `gband.opt` set them.

#### Scenario: Default prefix
- **WHEN** `user/init.lua` binds `prefix q` to `gband.action.close_pane` and does not set `prefix`, and two panes are open
- **THEN** Ctrl+Space then `q` closes the focused pane
- **AND** Ctrl+A reaches the focused pane as `\x01`

#### Scenario: Partial update
- **WHEN** `user/server.lua` calls `gband.set { default_column_width = 1/3 }`
- **THEN** the default column width is 1/3
- **AND** the width presets are 1/3, 1/2 and 2/3

#### Scenario: Later call wins
- **WHEN** `user/server.lua` calls `gband.set { default_column_width = 1/3 }`, then `gband.set { default_column_width = 2/3 }`
- **THEN** the default column width is 2/3

#### Scenario: Presets are sorted
- **WHEN** `user/server.lua` sets `width_presets` to `{ 2/3, 1/4, 2/3 }`
- **THEN** the width presets are 1/4 and 2/3

#### Scenario: Decimal width
- **WHEN** `user/server.lua` sets `default_column_width` to `0.35`
- **THEN** the default column width is 7/20

#### Scenario: Unknown option
- **WHEN** line 3 of `init.lua` is `gband.set { colum_width = 1/2 }`
- **THEN** loading fails with an error at `init.lua` line 3 naming `colum_width`

#### Scenario: Server option in the client file
- **WHEN** line 2 of `user/init.lua` is `gband.set { default_column_width = 1/3 }`
- **THEN** loading fails with an error at `user/init.lua` line 2 naming `default_column_width` and the server

#### Scenario: Wrong type
- **WHEN** line 2 of `user/server.lua` is `gband.set { width_presets = "1/2" }`
- **THEN** loading fails with an error at `user/server.lua` line 2 naming `width_presets`

#### Scenario: Unknown camera policy
- **WHEN** `init.lua` sets `center_focused_column` to `"sometimes"`
- **THEN** loading fails with an error naming `center_focused_column`

#### Scenario: Option read and set through gband.opt
- **WHEN** `user/server.lua` sets `gband.opt.default_column_width = 1/3` and then reads `gband.opt.width_presets`
- **THEN** the default column width is 1/3
- **AND** the value read is `{ 1/3, 1/2, 2/3 }`

#### Scenario: Invalid value falls back to the default
- **WHEN** line 4 of `user/server.lua` sets `gband.opt.default_column_width = 1/3` and line 5 sets `gband.opt.default_column_width = "wide"`
- **THEN** loading succeeds with the default column width 1/2
- **AND** an error at `user/server.lua` line 5 naming `default_column_width` is reported

#### Scenario: Declared plugin option
- **WHEN** the plugin `hello` calls `gband.opt.declare("greeting", { type = "string", default = "hello" })`
- **THEN** `gband.opt["hello.greeting"]` reads `"hello"`

#### Scenario: Set before declared
- **WHEN** `user/init.lua` sets `gband.opt["hello.greeting"] = "hi"` and the plugin `hello` declares `greeting` later in the load
- **THEN** `gband.opt["hello.greeting"]` reads `"hi"`

#### Scenario: Never declared
- **WHEN** line 9 of `user/init.lua` sets `gband.opt["absent.flag"] = true` and nothing declares it
- **THEN** loading succeeds
- **AND** an error at `user/init.lua` line 9 naming `absent.flag` is reported

#### Scenario: Value outside the allowed list
- **WHEN** an option is declared with `values = { "a", "b" }` and default `"a"`, and `init.lua` sets it to `"c"`
- **THEN** the option reads `"a"` and an error naming the option is reported

#### Scenario: Invalid status line height
- **WHEN** line 2 of `user/init.lua` sets `gband.opt.statusline_height = 0`
- **THEN** loading succeeds with a status line height of 1
- **AND** an error at `user/init.lua` line 2 naming `statusline_height` is reported

### Requirement: Configuration errors
A configuration error SHALL be reported as the file's path, a colon, the line, a colon and a message. The line SHALL be the line of a syntax error, the line where a runtime error was raised, or the line of the call or assignment that received the invalid value. A plugin error SHALL be reported the same way, preceded by the plugin's name, a colon and a space. Each process SHALL record every configuration error and plugin error of its own Lua in its log. The client SHALL also report the errors of the server's Lua that the server sends it, as the server-runtime capability defines, preceded by `server: `, and the plugin requirement errors of the plugin-bridge capability. The client SHALL show the latest of all these until its configuration next loads with none of its own, or a newer one replaces it. While the status line is drawn, the client SHALL show it in the status line's error item, as the status-line capability defines. While no status line is drawn, the client SHALL show it as a banner on the bottom row of the ribbon area, over the ribbon and cut to the terminal's width. Neither SHALL change the size the client reports.

#### Scenario: Syntax error
- **WHEN** `XDG_CONFIG_HOME` is unset, the home directory is `/home/u`, line 12 of `user/init.lua` holds a syntax error, and a client attaches
- **THEN** the client's bottom row shows the error, beginning `/home/u/.config/gband/user/init.lua:12:`
- **AND** the client log records it

#### Scenario: Plugin error in the status line
- **WHEN** `XDG_DATA_HOME` is `/tmp/data`, line 3 of `/tmp/data/gband/plugins/hello/client.lua` raises `boom`, and a client attaches
- **THEN** the status line's error item shows `hello: /tmp/data/gband/plugins/hello/client.lua:3: boom`

#### Scenario: Server error in the client
- **WHEN** line 2 of `user/server.lua` holds a syntax error and a client starts a server
- **THEN** the client shows an error beginning `server: ` and naming `user/server.lua:2:`
- **AND** the server log records it

#### Scenario: Plugin error on the banner
- **WHEN** `XDG_DATA_HOME` is `/tmp/data`, `statusline_position` is `"off"`, line 3 of `/tmp/data/gband/plugins/hello/client.lua` raises `boom`, and a client attaches
- **THEN** the client's bottom row shows `hello: /tmp/data/gband/plugins/hello/client.lua:3: boom`

#### Scenario: Banner without a status line
- **WHEN** `statusline_position` is `"off"`, line 3 of a plugin's `client.lua` raises `boom`, and a client attaches
- **THEN** the client's bottom row shows the error over the ribbon

#### Scenario: Banner cleared by a good load
- **WHEN** the client shows a configuration error and the user fixes `user/init.lua`
- **THEN** the error disappears once the configuration reloads

### Requirement: Last good configuration
Loading SHALL apply all of a configuration or none of it. An error raised by the init file, including one a `gband` call raises while it runs, SHALL fail the load. A plugin error and an invalid value set through `gband.opt` SHALL NOT fail the load. When loading fails, the process SHALL keep the configuration it last loaded, or, when it has loaded none, its side's default configuration with the side files of the runtimepath's plugins.

#### Scenario: Broken file at start
- **WHEN** `user/init.lua` sets `prefix` to `"ctrl+b"` and then raises an error, and a client attaches
- **THEN** the prefix is Ctrl+Space and the default bindings apply

#### Scenario: Broken edit keeps the running configuration
- **WHEN** `user/init.lua` binds `alt+h` and has loaded, and the user saves a version that binds `alt+j` and has a syntax error
- **THEN** Alt+H still focuses the column to the left
- **AND** Alt+J reaches the focused pane

#### Scenario: Plugin error keeps the rest
- **WHEN** `user/init.lua` binds `alt+h` and calls `gband.plugin("broken")`, whose `setup` raises an error
- **THEN** loading succeeds and Alt+H focuses the column to the left

### Requirement: Reload on change
Each process SHALL watch the `user` directory and every directory beneath it. Within one second of a file whose name ends in `.lua` being created, written, replaced or removed there, the process SHALL load its side's configuration again in a new Lua state and, when loading succeeds, use the new configuration from then on. Changes to the `defaults` directory and to the plugins directory SHALL NOT cause a reload. Server options SHALL apply to session actions received after the server reloads, and no column's width SHALL change because of a reload. A key sequence in progress SHALL end without effect when the client reloads.

#### Scenario: New binding without restart
- **WHEN** a client is attached and the user adds `gband.bind("alt+l", gband.action.focus_column_right)` to `user/init.lua`
- **THEN** within a second Alt+L focuses the column to the right

#### Scenario: Editor replaces the file
- **WHEN** an editor saves `user/init.lua` by writing a new file and renaming it over the old one
- **THEN** the client loads the new content

#### Scenario: User plugin file edited
- **WHEN** a client is attached, `user/init.lua` calls `require("keys")`, and the user saves `user/lua/keys.lua` binding `alt+k`
- **THEN** within a second Alt+K runs the new binding

#### Scenario: New default width
- **WHEN** a session holds a column of width 1/2 and the user sets `default_column_width` to `1/3` in `user/server.lua`
- **THEN** that column keeps width 1/2
- **AND** a pane opened after the reload has a column of width 1/3

#### Scenario: File removed
- **WHEN** `user/init.lua` bound `alt+h` and the user deletes it
- **THEN** the default configuration applies and Alt+H reaches the focused pane

#### Scenario: Defaults file edited while running
- **WHEN** a client is attached and the user saves a change to `defaults/init.lua`
- **THEN** the bindings stay as they were
