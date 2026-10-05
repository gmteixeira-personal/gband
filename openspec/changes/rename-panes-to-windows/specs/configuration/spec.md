## MODIFIED Requirements

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
- **AND** Ctrl+Space then `q` reaches the focused window as `\x00` and then `q`

#### Scenario: Plugin files see the init file's options
- **WHEN** `user/init.lua` sets `gband.opt.prefix` to `"ctrl+b"` and a plugin's `client.lua` reads `gband.opt.prefix`
- **THEN** `client.lua` reads `"ctrl+b"`

#### Scenario: Plugin sets a width option
- **WHEN** a plugin's `server.lua` sets `gband.opt.default_column_width` to `1/3`, and a client starts a server and opens a window
- **THEN** the new window's column has width 1/3

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
- **WHEN** `user/init.lua` binds `prefix q` to `gband.action.close_window` and does not set `prefix`, and two windows are open
- **THEN** Ctrl+Space then `q` closes the focused window
- **AND** Ctrl+A reaches the focused window as `\x01`

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

### Requirement: Bind and unbind keys
Key bindings SHALL live in key tables. The table `root` SHALL hold the bindings of keys pressed outside a sequence, `prefix` the bindings of the key pressed after the prefix key, and any other name a table a callback enters, as the client-attach capability defines.

`gband.keymap.set(table, key, action, opts)` SHALL bind `key` in `table` to `action`, with the optional string `opts.desc` as the binding's description. `key` SHALL be one key name, or, in a table other than `root`, `prefix`, which stands for the key the `prefix` option names when the bindings are used. `action` SHALL be a value from `gband.action` or a Lua function. Binding a key already bound in the table SHALL replace the earlier binding. `gband.keymap.del(table, key)` SHALL remove the binding of `key` from `table`, and SHALL do nothing when it is unbound. `gband.keymap.list(table)` SHALL return one table per binding of `table`, in the order the bindings were made, each holding `key` as written, `desc`, and `action`, the full name of the bound action or nil for a function. A table with no binding SHALL list as empty.

`gband.bind(keys, action)` SHALL bind as `gband.keymap.set` does, with no description: `keys` that are one key name bind in `root`, `prefix` followed by a space and one key name binds in `prefix`, and `prefix prefix` binds the prefix key in `prefix`. `gband.unbind(keys)` SHALL remove the binding those `keys` name, and SHALL do nothing when it is unbound.

A key list of more than two keys, two keys whose first is not `prefix`, an invalid key name, `prefix` as the key of a `root` binding, a table name that is not a non-empty string, or an action that is neither a `gband.action` value nor a function SHALL be a configuration error. Once the configuration is evaluated, a `root` binding of the key the `prefix` option names SHALL be a configuration error at the line of that binding. Bindings SHALL be made and removed only while the configuration loads; doing so later SHALL be an error.

#### Scenario: Direct binding
- **WHEN** `user/init.lua` calls `gband.bind("alt+h", gband.action.focus_column_left)`
- **THEN** Alt+H focuses the column to the left without the prefix

#### Scenario: Keymap binding with a description
- **WHEN** `user/init.lua` calls `gband.keymap.set("prefix", "g", gband.action.focus_column_left, { desc = "left" })`
- **THEN** Ctrl+Space then `g` focuses the column to the left
- **AND** `gband.keymap.list("prefix")` holds an entry with `key` `g`, `desc` `left` and `action` `focus_column_left`

#### Scenario: Old and new forms agree
- **WHEN** `user/init.lua` calls `gband.bind("prefix x", gband.action.detach)` and then `gband.keymap.del("prefix", "x")`
- **THEN** Ctrl+Space then `x` discards both keys

#### Scenario: Override a default
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua` that then binds `prefix q` to `gband.action.detach`
- **THEN** Ctrl+Space then `q` detaches, and no binding closes the window

#### Scenario: Unbind a default
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua` that then calls `gband.unbind("prefix q")`
- **THEN** Ctrl+Space then `q` discards both keys

#### Scenario: Prefix changed after binding
- **WHEN** `user/init.lua` binds `prefix h` to `gband.action.focus_column_left` and later sets `prefix` to `"ctrl+b"`
- **THEN** Ctrl+B then `h` focuses the column to the left
- **AND** Ctrl+Space reaches the focused window as `\x00`

#### Scenario: Direct binding of the prefix key
- **WHEN** line 4 of `user/init.lua` binds `ctrl+space` while the prefix is `ctrl+space`
- **THEN** loading fails with an error at `user/init.lua` line 4

#### Scenario: Prefix in the root table
- **WHEN** line 2 of `user/init.lua` calls `gband.keymap.set("root", "prefix", gband.action.detach)`
- **THEN** loading fails with an error at `user/init.lua` line 2

#### Scenario: Key chain too long
- **WHEN** `user/init.lua` binds `prefix w q`
- **THEN** loading fails with an error naming `prefix w q`

#### Scenario: Not an action
- **WHEN** `user/init.lua` calls `gband.bind("alt+h", "focus_column_left")`
- **THEN** loading fails with an error naming the binding of `alt+h`

#### Scenario: Binding after the load
- **WHEN** a binding function calls `gband.keymap.set("root", "alt+x", fn)` on line 6 of `user/init.lua`
- **THEN** the client shows an error at `user/init.lua` line 6

### Requirement: Spawn a program
`gband.spawn` SHALL take a table whose optional `cmd` field is a string or a list of strings, and whose optional `band` and `after` fields name a band and a window as the `open_window` target does in the lua-control capability. Called inside a binding function, it SHALL dispatch open window naming the program to run: a string as a command line the user's shell runs, a list as the program and its arguments, and no `cmd` as the user's shell. With neither `band` nor `after`, open window SHALL be resolved as the actions capability defines. With either, it SHALL name the band and the window to open after as that target does. A `cmd` of any other type, an empty list, a `band` or `after` that the target would refuse, or any other field SHALL be an error.

#### Scenario: Spawn a command line
- **WHEN** `init.lua` binds `alt+n` to `function() gband.spawn({ cmd = "fish" }) end` and the user presses Alt+N
- **THEN** a new window opens right of the focused window's column, running `fish`, and is focused

#### Scenario: Spawn an argument list
- **WHEN** a binding function calls `gband.spawn({ cmd = { "htop", "-d", "10" } })`
- **THEN** a new window runs `htop` with the arguments `-d` and `10`

#### Scenario: Spawn after a named pane
- **WHEN** band 1 holds columns with windows 1 and 2, window 2 is focused, and a binding function calls `gband.spawn({ cmd = "fish", after = 1 })`
- **THEN** band 1 holds window 1's column, a new column running `fish`, and window 2's column, in that order

#### Scenario: Unknown field
- **WHEN** a binding function calls `gband.spawn({ cmd = "fish", width = 1/2 })`
- **THEN** the call raises an error naming `width`

### Requirement: Binding functions
A callback SHALL be a binding function, the function of a registered action, the function of a command, or an event handler. A callback SHALL run in the client, never in the server. A binding function SHALL run with no arguments when its keys are pressed. Calling an action value, `gband.spawn` or `gband.keymap.enter` outside a callback SHALL be a configuration error. Wherever this capability allows a call inside a binding function, the call SHALL be allowed inside any callback. An error raised while a callback runs SHALL be reported as "Configuration errors" defines, and the actions the callback dispatched before the error SHALL stand.

#### Scenario: Action during evaluation
- **WHEN** line 7 of `init.lua` calls `gband.action.close_window()` at the top level
- **THEN** loading fails with an error at `init.lua` line 7

#### Scenario: Error in a binding function
- **WHEN** a binding function calls `gband.action.focus_column_left()` and then raises an error on line 9 of `init.lua`
- **THEN** the column to the left is focused
- **AND** the client shows an error at `init.lua` line 9

#### Scenario: Spawn from an event handler
- **WHEN** a `User` handler calls `gband.spawn({ cmd = "fish" })` and a binding function emits that event
- **THEN** a new window running `fish` opens

### Requirement: Last good configuration
Loading SHALL apply all of a configuration or none of it. An error raised by the init file, including one a `gband` call raises while it runs, SHALL fail the load. A plugin error and an invalid value set through `gband.opt` SHALL NOT fail the load. When loading fails, the process SHALL keep the configuration it last loaded, or, when it has loaded none, its side's default configuration with the side files of the runtimepath's plugins.

#### Scenario: Broken file at start
- **WHEN** `user/init.lua` sets `prefix` to `"ctrl+b"` and then raises an error, and a client attaches
- **THEN** the prefix is Ctrl+Space and the default bindings apply

#### Scenario: Broken edit keeps the running configuration
- **WHEN** `user/init.lua` binds `alt+h` and has loaded, and the user saves a version that binds `alt+j` and has a syntax error
- **THEN** Alt+H still focuses the column to the left
- **AND** Alt+J reaches the focused window

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
- **AND** a window opened after the reload has a column of width 1/3

#### Scenario: File removed
- **WHEN** `user/init.lua` bound `alt+h` and the user deletes it
- **THEN** the default configuration applies and Alt+H reaches the focused window

#### Scenario: Defaults file edited while running
- **WHEN** a client is attached and the user saves a change to `defaults/init.lua`
- **THEN** the bindings stay as they were
