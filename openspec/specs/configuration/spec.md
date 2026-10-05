# configuration Specification

## Purpose
Defines gband's Lua configuration: where the configuration file lives, how the client and the server evaluate it on top of the built-in defaults, the `gband` API it uses to set options and bind keys, how errors are reported, and how a changed file is reloaded.

## Requirements

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

### Requirement: Configuration file
The configuration file SHALL be `user/init.lua` in the configuration directory. The client and the server SHALL each load the configuration when they start, after preparing the configuration directory. Loading SHALL use one new Lua state, starting from the declared defaults of the options and no key bindings, and SHALL evaluate the init file and then the plugin files the plugins capability sources from the runtimepath. The init file SHALL be the configuration file when it exists, and the default configuration otherwise. Nothing else SHALL be evaluated before the init file: the configuration file replaces the default configuration rather than adding to it. A missing configuration file SHALL NOT be an error. Each process SHALL apply only the parts of the configuration it owns: the server the column width options, and the client the prefix, the key tables, the camera policy and the callbacks.

#### Scenario: No configuration file
- **WHEN** no `user/init.lua` exists, no plugin directory exists, and a client attaches
- **THEN** no error is reported
- **AND** the key bindings, options and behaviour are those of the default configuration

#### Scenario: XDG_CONFIG_HOME is honoured
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `/tmp/cfg/gband/user/init.lua` sets `width_presets` to `{ 1/4, 1/2 }` and binds `prefix r` to `gband.action.cycle_column_width`, and a client starts a server and cycles a column of width 1/2
- **THEN** the column's width is 1/4

#### Scenario: User file replaces the defaults
- **WHEN** `user/init.lua` holds only `gband.bind("alt+h", gband.action.focus_column_left)` and a client attaches
- **THEN** Alt+H focuses the column to the left
- **AND** Ctrl+Space then `q` reaches the focused pane as `\x00` and then `q`

#### Scenario: Plugin files see the init file's options
- **WHEN** `user/init.lua` sets `gband.opt.prefix` to `"ctrl+b"` and `user/plugin/check.lua` reads `gband.opt.prefix`
- **THEN** `check.lua` reads `"ctrl+b"`

#### Scenario: Plugin sets a width option
- **WHEN** a plugin directory's `plugin/widths.lua` sets `gband.opt.default_column_width` to `1/3`, and a client starts a server and opens a pane
- **THEN** the new pane's column has width 1/3

### Requirement: Defaults use the public API
The default configuration SHALL use only the `gband` API that the configuration file can use. It SHALL make every key binding with `gband.keymap.set`, giving each the description of the action it binds, as the actions capability lists them. Evaluated alone, it SHALL produce the declared defaults of the options and the default key bindings of the client-attach capability. The default key bindings SHALL exist only in the default configuration.

#### Scenario: Defaults reproduce the built-in behaviour
- **WHEN** the default configuration is evaluated alone
- **THEN** the options equal the defaults in the "Options" table
- **AND** the bindings equal the client-attach capability's default table, entry for entry

#### Scenario: Every default binding is described
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** every entry's `desc` is the description `gband.action.list()` gives its action

#### Scenario: Copied defaults load unchanged
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua`
- **THEN** loading succeeds
- **AND** the options and bindings equal those of the default configuration

### Requirement: Options
Every option SHALL have a name, a type, a declared default and a description. The built-in options SHALL be:

| option | value | default |
|---|---|---|
| `prefix` | one key, as "Key names" defines | `"ctrl+space"` |
| `default_column_width` | a number greater than 0 and at most 10000 | `1/2` |
| `width_presets` | a list of one or more numbers, each greater than 0 and at most 10000 | `{ 1/3, 1/2, 2/3 }` |
| `center_focused_column` | `"never"`, `"always"` or `"on-overflow"` | `"never"` |

A width SHALL be read as the fraction closest to the number whose denominator is at most 100, in lowest terms, so `1/3` reads as 1/3. The width presets SHALL be held in ascending order with duplicates removed.

`gband.set` SHALL take one table of options. Each call SHALL change only the options the table names, and a later call SHALL override an earlier one. An unknown option name, a value of the wrong type, a value out of range or an unknown camera policy given to `gband.set` SHALL be a configuration error naming the option.

Reading `gband.opt.<name>` SHALL return the option's current value: a key name for `prefix`, a number for a width, a list of numbers in ascending order for `width_presets`, a string for `center_focused_column`, and the value as set for a declared option. Assigning `gband.opt.<name>` SHALL set the option. A value that the option's type rejects SHALL be reported as a configuration error at the line of the assignment, SHALL NOT fail the load, and SHALL set the option to its declared default. An assignment to a name no option declares SHALL be held until an option of that name is declared later in the same load, and then validated; one still undeclared when loading finishes SHALL be reported as a configuration error at the line of the assignment, and SHALL NOT fail the load.

`gband.opt.declare(name, spec)` SHALL declare an option under its full name, as the plugins capability defines, and return that full name. `spec.type` SHALL be `"boolean"`, `"integer"`, `"number"` or `"string"`. `spec.values`, optional, SHALL list the only values allowed. `spec.default` SHALL be a valid value, and `spec.desc` an optional description. Declaring a name already declared, an unknown type, or an invalid default SHALL be an error at the line of the call. `gband.opt.list()` SHALL return one table per option, built-in and declared, in ascending byte order of names, each holding `name`, `type`, `default`, `value` and `desc`.

Options SHALL be set and declared only while the configuration loads. Setting or declaring an option later SHALL be an error. Each process SHALL use the options as they stand when loading finishes, whichever of `gband.set` and `gband.opt` set them.

#### Scenario: Default prefix
- **WHEN** `user/init.lua` binds `prefix q` to `gband.action.close_pane` and does not set `prefix`, and two panes are open
- **THEN** Ctrl+Space then `q` closes the focused pane
- **AND** Ctrl+A reaches the focused pane as `\x01`

#### Scenario: Partial update
- **WHEN** `init.lua` calls `gband.set { default_column_width = 1/3 }`
- **THEN** the default column width is 1/3
- **AND** the width presets are 1/3, 1/2 and 2/3

#### Scenario: Later call wins
- **WHEN** `init.lua` calls `gband.set { default_column_width = 1/3 }`, then `gband.set { default_column_width = 2/3 }`
- **THEN** the default column width is 2/3

#### Scenario: Presets are sorted
- **WHEN** `init.lua` sets `width_presets` to `{ 2/3, 1/4, 2/3 }`
- **THEN** the width presets are 1/4 and 2/3

#### Scenario: Decimal width
- **WHEN** `init.lua` sets `default_column_width` to `0.35`
- **THEN** the default column width is 7/20

#### Scenario: Unknown option
- **WHEN** line 3 of `init.lua` is `gband.set { colum_width = 1/2 }`
- **THEN** loading fails with an error at `init.lua` line 3 naming `colum_width`

#### Scenario: Wrong type
- **WHEN** line 2 of `init.lua` is `gband.set { width_presets = "1/2" }`
- **THEN** loading fails with an error at `init.lua` line 2 naming `width_presets`

#### Scenario: Unknown camera policy
- **WHEN** `init.lua` sets `center_focused_column` to `"sometimes"`
- **THEN** loading fails with an error naming `center_focused_column`

#### Scenario: Option read and set through gband.opt
- **WHEN** `init.lua` sets `gband.opt.default_column_width = 1/3` and then reads `gband.opt.width_presets`
- **THEN** the default column width is 1/3
- **AND** the value read is `{ 1/3, 1/2, 2/3 }`

#### Scenario: Invalid value falls back to the default
- **WHEN** line 4 of `init.lua` sets `gband.opt.default_column_width = 1/3` and line 5 sets `gband.opt.default_column_width = "wide"`
- **THEN** loading succeeds with the default column width 1/2
- **AND** an error at `init.lua` line 5 naming `default_column_width` is reported

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

### Requirement: Key names
A key name SHALL be a key optionally preceded by modifiers, joined by `+`. The modifiers SHALL be `ctrl`, `alt` and `shift`, in any order. The key SHALL be one character, or one of `enter`, `tab`, `backtab`, `backspace`, `escape`, `esc`, `space`, `up`, `down`, `left`, `right`, `home`, `end`, `insert`, `delete`, `pageup`, `pagedown` and `f1` to `f12`. Modifier and key names longer than one character SHALL be read without regard to case, and a one-character key SHALL be read as written, so `D` and `d` differ. A `+` that ends the name SHALL be the key, so `alt++` is Alt with `+`. `shift` with a lowercase letter SHALL name the uppercase letter. `shift` with any other character SHALL be an error, because the shifted character is written instead. Any other name SHALL be an error.

#### Scenario: Modifier and character
- **WHEN** a binding names `alt+h`
- **THEN** it names `h` with Alt

#### Scenario: Shift with a letter
- **WHEN** a binding names `shift+d`
- **THEN** it names the same key as `D`

#### Scenario: Plus as the key
- **WHEN** a binding names `alt++`
- **THEN** it names `+` with Alt

#### Scenario: Named key in any case
- **WHEN** a binding names `Ctrl+PageUp`
- **THEN** it names Page Up with Ctrl

#### Scenario: Unknown key
- **WHEN** line 5 of `init.lua` binds `alt+hyper`
- **THEN** loading fails with an error at `init.lua` line 5 naming `alt+hyper`

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
- **THEN** Ctrl+Space then `q` detaches, and no binding closes the pane

#### Scenario: Unbind a default
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua` that then calls `gband.unbind("prefix q")`
- **THEN** Ctrl+Space then `q` discards both keys

#### Scenario: Prefix changed after binding
- **WHEN** `user/init.lua` binds `prefix h` to `gband.action.focus_column_left` and later sets `prefix` to `"ctrl+b"`
- **THEN** Ctrl+B then `h` focuses the column to the left
- **AND** Ctrl+Space reaches the focused pane as `\x00`

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

### Requirement: Action values
`gband.action` SHALL hold one value for every built-in action, under the Lua name the actions capability gives it, and one for every registered action, under its full name. Calling an action value inside a callback SHALL dispatch that action, with the same effect as pressing a key bound to it.

`gband.action.register(name, fn, opts)` SHALL register an action under its full name, as the plugins capability defines, with the optional string `opts.desc` as its description, and SHALL return its action value. Dispatching a registered action SHALL run `fn` with no arguments, at once, as a callback that belongs to the action's plugin. A name that is not a non-empty string, an `fn` that is not a function, a full name already registered or held by a built-in action, and the names `register` and `list` SHALL be an error at the line of the call. Actions SHALL be registered only while the configuration loads; registering later SHALL be an error.

`gband.action.list()` SHALL return one table per action, built-in and registered, in ascending byte order of names, each holding `name` and `desc`.

#### Scenario: Action called from a function
- **WHEN** `init.lua` binds `alt+w` to a function that calls `gband.action.focus_column_right()` twice, and the first of three columns is focused
- **THEN** pressing Alt+W focuses the third column

#### Scenario: Registered action bound to a key
- **WHEN** `user/init.lua` registers `twice` with a function that calls `gband.action.focus_column_right()` twice, binds `alt+t` to `gband.action.twice`, and the first of three columns is focused
- **THEN** pressing Alt+T focuses the third column

#### Scenario: Registered action called from another callback
- **WHEN** a binding function calls `gband.action["hello.greet"]()`
- **THEN** the function `hello` registered as `greet` runs before the binding function continues

#### Scenario: Built-in name taken
- **WHEN** line 3 of `user/init.lua` calls `gband.action.register("detach", fn)`
- **THEN** loading fails with an error at `user/init.lua` line 3 naming `detach`

### Requirement: Spawn a program
`gband.spawn` SHALL take a table whose optional `cmd` field is a string or a list of strings. Called inside a binding function, it SHALL dispatch open pane, as the actions capability resolves it, naming the program to run: a string as a command line the user's shell runs, a list as the program and its arguments, and no `cmd` as the user's shell. A `cmd` of any other type, an empty list, or a field other than `cmd` SHALL be an error.

#### Scenario: Spawn a command line
- **WHEN** `init.lua` binds `alt+n` to `function() gband.spawn({ cmd = "fish" }) end` and the user presses Alt+N
- **THEN** a new pane opens right of the focused pane's column, running `fish`, and is focused

#### Scenario: Spawn an argument list
- **WHEN** a binding function calls `gband.spawn({ cmd = { "htop", "-d", "10" } })`
- **THEN** a new pane runs `htop` with the arguments `-d` and `10`

### Requirement: Binding functions
A callback SHALL be a binding function, the function of a registered action, the function of a command, or an event handler. A callback SHALL run in the client, never in the server. A binding function SHALL run with no arguments when its keys are pressed. Calling an action value, `gband.spawn` or `gband.keymap.enter` outside a callback SHALL be a configuration error. Wherever this capability allows a call inside a binding function, the call SHALL be allowed inside any callback. An error raised while a callback runs SHALL be reported as "Configuration errors" defines, and the actions the callback dispatched before the error SHALL stand.

#### Scenario: Action during evaluation
- **WHEN** line 7 of `init.lua` calls `gband.action.close_pane()` at the top level
- **THEN** loading fails with an error at `init.lua` line 7

#### Scenario: Error in a binding function
- **WHEN** a binding function calls `gband.action.focus_column_left()` and then raises an error on line 9 of `init.lua`
- **THEN** the column to the left is focused
- **AND** the client shows an error at `init.lua` line 9

#### Scenario: Spawn from an event handler
- **WHEN** a `User` handler calls `gband.spawn({ cmd = "fish" })` and a binding function emits that event
- **THEN** a new pane running `fish` opens

### Requirement: Configuration errors
A configuration error SHALL be reported as the file's path, a colon, the line, a colon and a message. The line SHALL be the line of a syntax error, the line where a runtime error was raised, or the line of the call or assignment that received the invalid value. A plugin error SHALL be reported the same way, preceded by the plugin's name, a colon and a space. Each process SHALL record every configuration error and plugin error in its log. The client SHALL also show the latest of them on its terminal's bottom row, over the ribbon and cut to the terminal's width, until the configuration next loads with neither. The banner SHALL NOT change the screen area the client reports.

#### Scenario: Syntax error
- **WHEN** `XDG_CONFIG_HOME` is unset, the home directory is `/home/u`, line 12 of `user/init.lua` holds a syntax error, and a client attaches
- **THEN** the client's bottom row shows the error, beginning `/home/u/.config/gband/user/init.lua:12:`
- **AND** the client log and the server log record it

#### Scenario: Plugin error on the banner
- **WHEN** `XDG_DATA_HOME` is `/tmp/data` and line 3 of `/tmp/data/gband/plugins/hello/plugin/hello.lua` raises `boom`, and a client attaches
- **THEN** the client's bottom row shows `hello: /tmp/data/gband/plugins/hello/plugin/hello.lua:3: boom`

#### Scenario: Banner cleared by a good load
- **WHEN** the client shows a configuration error and the user fixes `user/init.lua`
- **THEN** the banner disappears once the configuration reloads

### Requirement: Last good configuration
Loading SHALL apply all of a configuration or none of it. An error raised by the init file, including one a `gband` call raises while it runs, SHALL fail the load. A plugin error and an invalid value set through `gband.opt` SHALL NOT fail the load. When loading fails, the process SHALL keep the configuration it last loaded, or the default configuration with the plugin files of the runtimepath when it has loaded none.

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
Each process SHALL watch the `user` directory and every directory beneath it. Within one second of a file whose name ends in `.lua` being created, written, replaced or removed there, the process SHALL load the configuration again in a new Lua state and, when loading succeeds, use the new configuration from then on. Changes to `defaults/init.lua` and to the plugins directory SHALL NOT cause a reload. Options the server owns SHALL apply to session actions received after the reload, and no column's width SHALL change because of a reload. A key sequence in progress SHALL end without effect when the client reloads.

#### Scenario: New binding without restart
- **WHEN** a client is attached and the user adds `gband.bind("alt+l", gband.action.focus_column_right)` to `user/init.lua`
- **THEN** within a second Alt+L focuses the column to the right

#### Scenario: Editor replaces the file
- **WHEN** an editor saves `user/init.lua` by writing a new file and renaming it over the old one
- **THEN** the processes load the new content

#### Scenario: User plugin file edited
- **WHEN** a client is attached and the user saves `user/plugin/keys.lua` binding `alt+k`
- **THEN** within a second Alt+K runs the new binding

#### Scenario: New default width
- **WHEN** a session holds a column of width 1/2 and the user sets `default_column_width` to `1/3` in `user/init.lua`
- **THEN** that column keeps width 1/2
- **AND** a pane opened after the reload has a column of width 1/3

#### Scenario: File removed
- **WHEN** `user/init.lua` bound `alt+h` and the user deletes it
- **THEN** the default configuration applies and Alt+H reaches the focused pane

#### Scenario: Defaults file edited while running
- **WHEN** a client is attached and the user saves a change to `defaults/init.lua`
- **THEN** the bindings stay as they were
