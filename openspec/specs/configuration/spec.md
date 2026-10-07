# configuration Specification

## Purpose
Defines gband's Lua configuration: where the configuration file lives, how the client and the server evaluate it on top of the built-in defaults, the `gband` API it uses to set options and bind keys, how errors are reported, and how a changed file is reloaded.

## Requirements

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

### Requirement: Configuration file
The client configuration file SHALL be `user/init.lua` in the configuration directory. Each client SHALL load its configuration when it starts, after preparing the configuration directory. Loading SHALL use one new Lua state, starting from the declared defaults of the client options and no key bindings, and SHALL evaluate the init file and then the `client.lua` files the plugins capability sources from the runtimepath. The init file SHALL be the configuration file when it exists, and the default client configuration otherwise. Before the init file, loading SHALL install only the client's `gband` API and load the start theme: the saved theme, or the bundled `default` theme, as the colorschemes capability defines. Nothing of the default configuration SHALL be evaluated before the init file: the configuration file replaces the default configuration rather than adding to it. A missing configuration file SHALL NOT be an error. The client SHALL use the prefix, the key tables, the camera policy, the notification style, the highlight groups, the bars and the callbacks. The server SHALL NOT evaluate the client configuration; it loads the server configuration as the server-runtime capability defines.

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

### Requirement: Defaults use the public API
The default configuration SHALL use only the `gband` API that the configuration file can use. It SHALL set the options to their declared defaults. It SHALL make its key bindings only by calling `gband.keystyle.use()` with no argument, as the key-style capability defines, so the saved key style, or the modal key style when none is saved, makes them. It SHALL make no other binding and declare no mode. It SHALL set up the key list plugin `gband.keylist` and the Lua prompt plugin `gband.prompt` only through that call. It SHALL then set up, with `gband.plugin` and no options, the bundled error list plugin `gband.errors`, then the bundled sidebar plugin `gband.sidebar` unless `gband.settings.sidebar()` returns `false`, in that order, and register the offer of the settings capability's "Offer on the first start". Evaluated alone, with no configuration directory, it SHALL produce the declared defaults of the options and the default key bindings of the client-attach capability for the modal key style.

The default key bindings and the declaration of the `navigation` mode SHALL exist only in the key style presets. The setup of the error list and sidebar plugins and the offer of the settings window SHALL exist only in the default configuration.

#### Scenario: Defaults reproduce the built-in behaviour
- **WHEN** the default configuration is evaluated alone
- **THEN** the options equal the defaults in the "Options and their values" table
- **AND** the bindings equal the client-attach capability's default table for the modal key style, entry for entry
- **AND** `gband.bar.list()` holds exactly one bar, `sidebar`, on the side `left`, 1 column wide on an 80×24 terminal

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

#### Scenario: Rename bound by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** the entry for `N` has the action `prompt.rename` and the description `rename the window`
- **AND** it comes right after the entry for `:` and before the entry for `D`

#### Scenario: Settings bound by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** the entry for `s` has the description `settings`
- **AND** it comes right after the entry for `N` and before the entry for `D`

#### Scenario: Sidebar left out by the saved setting
- **WHEN** `user/sidebar.lua` holds `return false`, no `user/init.lua` exists, and a client attaches
- **THEN** `gband.bar.list()` holds no bar
- **AND** an error is shown as the banner on the ribbon area's bottom row

#### Scenario: Saved theme with a user file
- **WHEN** `user/theme.lua` holds `return "nord"` and `user/init.lua` binds only `alt+h`
- **THEN** `gband.colorscheme()` returns `nord` once the file has loaded

#### Scenario: No binding in the defaults file
- **WHEN** `defaults/init.lua` is read
- **THEN** it calls `gband.keystyle.use()` once
- **AND** it calls none of `gband.keymap.set`, `gband.bind` and `gband.keymap.mode`

#### Scenario: Copied defaults load unchanged
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua`
- **THEN** loading succeeds
- **AND** the options, bindings and modes equal those of the default configuration with the same saved key style

### Requirement: Key names
A key name SHALL be a key optionally preceded by modifiers, joined by `+`. The modifiers SHALL be `ctrl`, `alt` and `shift`, in any order, and, before a mouse name only, `mod`, as "Mouse modifier" defines. The key SHALL be one character, or one of `enter`, `tab`, `backtab`, `backspace`, `escape`, `esc`, `space`, `up`, `down`, `left`, `right`, `home`, `end`, `insert`, `delete`, `pageup`, `pagedown` and `f1` to `f12`, or a mouse name. The mouse names SHALL be `leftmouse`, `middlemouse` and `rightmouse`, which name a press of that mouse button, and `wheelup`, `wheeldown`, `wheelleft` and `wheelright`, which name one wheel step in that direction, as the mouse capability defines. Modifier and key names longer than one character SHALL be read without regard to case, and a one-character key SHALL be read as written, so `D` and `d` differ. A `+` that ends the name SHALL be the key, so `alt++` is Alt with `+`. `shift` with a lowercase letter SHALL name the uppercase letter. `shift` with any other character SHALL be an error, because the shifted character is written instead. `shift` with a mouse name SHALL name that mouse name with Shift. `mod` with a key that is not a mouse name SHALL be an error. A mouse name SHALL be valid only as the key of a binding in a key table, and SHALL be an error as the `prefix` option or as a key of a plugin window's `keys`. Any other name SHALL be an error.

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

#### Scenario: Mouse name with modifiers
- **WHEN** a binding names `Shift+RightMouse`
- **THEN** it names a press of the right button with Shift

#### Scenario: Wheel name
- **WHEN** a binding names `Alt+WheelDown`
- **THEN** it names one wheel step down with Alt

#### Scenario: Unknown wheel name
- **WHEN** line 4 of `init.lua` binds `prefix scrollup`
- **THEN** loading fails with an error at `init.lua` line 4 naming `scrollup`

#### Scenario: Mod with a key
- **WHEN** line 6 of `init.lua` binds `mod+h`
- **THEN** loading fails with an error at `init.lua` line 6 naming `mod+h`

#### Scenario: Mouse name as the prefix
- **WHEN** line 3 of `init.lua` sets `gband.opt.prefix = "leftmouse"`
- **THEN** loading reports a configuration error at `init.lua` line 3 naming `leftmouse`

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
`gband.spawn` SHALL take a table whose optional `cmd` field is a string or a list of strings, and whose optional `band` and `after` fields name a band and a window as the `open_window` target does in the lua-control capability. Called inside a binding function, it SHALL dispatch open window naming the program to run: a string as a command line the user's shell runs, a list as the program and its arguments, and no `cmd` as the user's shell. With neither `band` nor `after`, open window SHALL be resolved as the actions capability defines. With either, it SHALL name the band and the window to open after as that target does. A `cmd` of any other type, an empty list, a `band` or `after` that the target would refuse, or any other field SHALL be an error.

#### Scenario: Spawn a command line
- **WHEN** `init.lua` binds `alt+n` to `function() gband.spawn({ cmd = "fish" }) end` and the user presses Alt+N
- **THEN** a new window opens right of the focused window's column, running `fish`, and is focused

#### Scenario: Spawn an argument list
- **WHEN** a binding function calls `gband.spawn({ cmd = { "htop", "-d", "10" } })`
- **THEN** a new window runs `htop` with the arguments `-d` and `10`

#### Scenario: Spawn after a named window
- **WHEN** band 1 holds columns with windows 1 and 2, window 2 is focused, and a binding function calls `gband.spawn({ cmd = "fish", after = 1 })`
- **THEN** band 1 holds window 1's column, a new column running `fish`, and window 2's column, in that order

#### Scenario: Unknown field
- **WHEN** a binding function calls `gband.spawn({ cmd = "fish", width = 1/2 })`
- **THEN** the call raises an error naming `width`

### Requirement: Binding functions
A callback SHALL be a binding function, the function of a registered action, the function of a command, or an event handler. A callback SHALL run in the client, never in the server. A binding function SHALL run with no arguments when its keys are pressed, except that a binding function bound to a mouse name SHALL run with one argument, the mouse event's payload, as the mouse capability defines. Calling an action value, `gband.spawn` or `gband.keymap.enter` outside a callback SHALL be a configuration error. Wherever this capability allows a call inside a binding function, the call SHALL be allowed inside any callback. An error raised while a callback runs SHALL be reported as "Reporting configuration errors" defines, and the actions the callback dispatched before the error SHALL stand.

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

#### Scenario: Mouse binding function
- **WHEN** navigation mode binds `leftmouse` to a function that records its argument, and the user clicks terminal column 12 and row 3
- **THEN** the function runs with a table whose `button` is `"left"`, `col` is 12 and `row` is 3

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

### Requirement: Modes
`gband.keymap.mode(table, opts)` SHALL declare the key table `table` a mode. `opts` SHALL be nil or a table, and `opts.label` SHALL be nil or a non-empty string naming the mode for display. Declaring a table a mode again SHALL replace its label. A table SHALL NOT need a binding to be declared a mode. Declaring `root` a mode, a table name that is not a non-empty string, or an `opts` or `label` of the wrong type SHALL be a configuration error at the line of the call. Modes SHALL be declared only while the configuration loads; calling `gband.keymap.mode` later SHALL be an error.

A mode SHALL keep its declaration across `gband.keymap.set` and `gband.keymap.del` calls on its table. How the client handles keys while a mode is active is defined by the client-attach capability.

`gband.keymap.label(table)` SHALL return the label of `table` when it is a mode with a label, and `table` itself otherwise. `gband.keymap.label` SHALL be callable while the configuration loads and in any callback.

`gband.keymap.enter("root")` SHALL be allowed in any callback, whether or not `root` holds a binding, and SHALL make `root` the active table when the callback's dispatches run.

#### Scenario: Declare a mode with a label
- **WHEN** `user/init.lua` calls `gband.keymap.mode("prefix", { label = "navigation" })` and then `gband.keymap.label("prefix")`
- **THEN** the call returns `navigation`

#### Scenario: Label of a table that is not a mode
- **WHEN** `user/init.lua` binds `h` in the table `move` and calls `gband.keymap.label("move")`
- **THEN** the call returns `move`

#### Scenario: Mode without a label
- **WHEN** `user/init.lua` calls `gband.keymap.mode("resize")` and then `gband.keymap.label("resize")`
- **THEN** the call returns `resize`

#### Scenario: Root cannot be a mode
- **WHEN** line 3 of `user/init.lua` calls `gband.keymap.mode("root")`
- **THEN** loading fails with an error at `user/init.lua` line 3

#### Scenario: Invalid label
- **WHEN** line 2 of `user/init.lua` calls `gband.keymap.mode("prefix", { label = 3 })`
- **THEN** loading fails with an error at `user/init.lua` line 2

#### Scenario: Mode declared after the load
- **WHEN** a binding function calls `gband.keymap.mode("move")` on line 7 of `user/init.lua`
- **THEN** the client shows an error at `user/init.lua` line 7

#### Scenario: Enter root with no root binding
- **WHEN** `user/init.lua` binds nothing in `root`, binds `prefix escape` to a function that calls `gband.keymap.enter("root")`, and the function runs
- **THEN** no error is raised and `root` becomes the active table

### Requirement: Run a binding
`gband.keymap.run(table, key)` SHALL run the binding of `key` in `table`, as pressing that key while `table` is active runs it: a binding to an action SHALL dispatch it, and a binding to a function SHALL run the function. `key` SHALL be written as `gband.keymap.set` takes it, `prefix` included. It SHALL return `true` when a binding ran and `false` when `key` is unbound in `table`. It SHALL send no key to any window or plugin window, and SHALL change the active table only through the binding's own `gband.keymap.enter`. It SHALL be callable only in a callback; calling it elsewhere, a table name that is not a non-empty string, or an invalid key name SHALL be an error.

#### Scenario: Run an action binding
- **WHEN** the default configuration is in use, two columns are open with the second focused, `root` is active, and a binding function calls `gband.keymap.run("prefix", "h")`
- **THEN** the call returns `true` and the first column is focused
- **AND** `root` is still the active table

#### Scenario: Run a function binding
- **WHEN** `user/init.lua` binds `prefix x` to a function that records a call, and a callback calls `gband.keymap.run("prefix", "x")`
- **THEN** the function runs once and the call returns `true`

#### Scenario: Unbound key
- **WHEN** a callback calls `gband.keymap.run("prefix", "z")` and `prefix` does not bind `z`
- **THEN** the call returns `false` and nothing is dispatched

#### Scenario: Run outside a callback
- **WHEN** line 5 of `user/init.lua` calls `gband.keymap.run("prefix", "h")` while the configuration loads
- **THEN** the client shows an error at `user/init.lua` line 5

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

### Requirement: Options and their values
Every option SHALL have a name, a type, a declared default and a description, and SHALL belong to one side. The built-in client options SHALL be:

| option | value | default |
|---|---|---|
| `prefix` | one key, as "Key names" defines | `"ctrl+space"` |
| `center_focused_column` | `"never"`, `"always"` or `"on-overflow"` | `"never"` |
| `loop_bands` | a boolean | `true` |
| `window_titles` | a boolean: whether windows show their names on their top borders, as the window-names capability defines | `true` |
| `notify_style` | `"osc9"`, `"osc777"`, `"bell"` or `"none"` | `"osc9"` |
| `tile_border_sides` | a list of side names, as the borders capability defines them | `{ "top", "right", "bottom", "left" }` |
| `tile_border_chars` | a character set, as the borders capability defines it | `"rounded"` |
| `focused_tile_border_chars` | a character set, for the focused tiled window's border | `"rounded"` |
| `floating_border_sides` | a list of side names | `{ "top", "right", "bottom", "left" }` |
| `floating_border_chars` | a character set | `"rounded"` |
| `focused_floating_border_chars` | a character set, for the focused floating window's border | `"rounded"` |
| `width_step` | a number greater than 0 and at most 10000 | `1/10` |
| `height_step` | a number greater than 0 and at most 1 | `1/10` |

The built-in server options SHALL be:

| option | value | default |
|---|---|---|
| `default_column_width` | a number greater than 0 and at most 10000 | `1/2` |
| `width_presets` | a list of one or more numbers, each greater than 0 and at most 10000 | `{ 1/3, 1/2, 2/3 }` |

A process SHALL know only the options of its own side, and options declared in its own Lua state. A width and a step SHALL each be read as the fraction closest to the number whose denominator is at most 100, in lowest terms, so `1/3` reads as 1/3. The width presets SHALL be held in ascending order with duplicates removed. A list of side names SHALL hold only `top`, `right`, `bottom` and `left`, and SHALL be held in that order with duplicates removed; an empty list is valid.

`gband.set` SHALL take one table of options. Each call SHALL change only the options the table names, and a later call SHALL override an earlier one. An unknown option name, a value of the wrong type, a value out of range or an unknown camera policy given to `gband.set` SHALL be a configuration error naming the option.

Reading `gband.opt.<name>` SHALL return the option's current value: a key name for `prefix`, a number for a width or a step, a list of numbers in ascending order for `width_presets`, a string for `center_focused_column` and `notify_style`, a list of side names in the held order for `tile_border_sides` and `floating_border_sides`, the name or a new list of the eight strings for `tile_border_chars`, `focused_tile_border_chars`, `floating_border_chars` and `focused_floating_border_chars`, a boolean for `loop_bands` and `window_titles`, and the value as set for a declared option. Assigning `gband.opt.<name>` SHALL set the option. A value that the option's type rejects SHALL be reported as a configuration error at the line of the assignment, SHALL NOT fail the load, and SHALL set the option to its declared default. An assignment to a name no option declares SHALL be held until an option of that name is declared later in the same load, and then validated; one still undeclared when loading finishes SHALL be reported as a configuration error at the line of the assignment, and SHALL NOT fail the load. The error for a built-in option of the other side SHALL name the side that owns it.

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

#### Scenario: Invalid step
- **WHEN** line 2 of `user/init.lua` sets `gband.opt.width_step = 0`
- **THEN** loading succeeds with a width step of 1/10
- **AND** an error at `user/init.lua` line 2 naming `width_step` is reported

#### Scenario: Sides are held in order
- **WHEN** `user/init.lua` sets `tile_border_sides` to `{ "left", "top", "left" }` and then reads it
- **THEN** the value read is `{ "top", "left" }`

#### Scenario: Looping bands by default
- **WHEN** `user/init.lua` does not set `loop_bands` and reads `gband.opt.loop_bands`
- **THEN** the value read is `true`

#### Scenario: Looping bands turned off
- **WHEN** line 3 of `user/init.lua` is `gband.set { loop_bands = false }`
- **THEN** loading succeeds and `gband.opt.loop_bands` reads `false`

#### Scenario: Looping bands of the wrong type
- **WHEN** line 3 of `user/init.lua` sets `gband.opt.loop_bands = "yes"`
- **THEN** loading succeeds with `loop_bands` set to `true`
- **AND** an error at `user/init.lua` line 3 naming `loop_bands` is reported

#### Scenario: Rounded borders by default
- **WHEN** `user/init.lua` sets no border option and reads `gband.opt.tile_border_chars`, `gband.opt.focused_tile_border_chars`, `gband.opt.floating_border_chars` and `gband.opt.focused_floating_border_chars`
- **THEN** each value read is `"rounded"`

#### Scenario: Focused border characters of the wrong type
- **WHEN** line 2 of `user/init.lua` sets `gband.opt.focused_tile_border_chars = "heavy"`
- **THEN** loading succeeds with `focused_tile_border_chars` set to `"rounded"`
- **AND** an error at `user/init.lua` line 2 naming `focused_tile_border_chars` is reported

#### Scenario: Window titles off
- **WHEN** `user/init.lua` calls `gband.set({ window_titles = false })` and then reads `gband.opt.window_titles`
- **THEN** it reads `false`

#### Scenario: Window titles of the wrong type
- **WHEN** line 2 of `user/init.lua` calls `gband.set({ window_titles = "no" })`
- **THEN** a configuration error at `user/init.lua` line 2 names `window_titles`

### Requirement: Reporting configuration errors
A configuration error SHALL be reported as the file's path, a colon, the line, a colon and a message. The line SHALL be the line of a syntax error, the line where a runtime error was raised, or the line of the call or assignment that received the invalid value. A plugin error SHALL be reported the same way, preceded by the plugin's name, a colon and a space. Each process SHALL record every configuration error and plugin error of its own Lua in its log. The client SHALL also report the errors of the server's Lua that the server sends it, as the server-runtime capability defines, preceded by `server: `, and the plugin requirement errors of the plugin-bridge capability. The client SHALL keep its error list: every error it has reported since the list was last emptied, oldest first. A load with none of its own SHALL empty the list, and so SHALL clearing the errors. `gband.errors()` SHALL return a new list of the error list's texts, oldest first. The client SHALL show the latest error until the list is emptied. While the sidebar's error marker is drawn, as the sidebar capability defines, the client SHALL show the error's text only through the error list. While no error marker is drawn, the client SHALL show it as a banner on the bottom row of the ribbon area, over the ribbon and cut to the ribbon area's width. Neither SHALL change the size the client reports.

`gband.clear_errors()` SHALL clear the client's errors. It SHALL be callable wherever an action value is, and calling it while the configuration loads SHALL be an error at the line of the call. It SHALL empty the error list at once, so `gband.errors()` returns an empty list in the same callback. Once that callback returns, the client SHALL show no error until it reports another. An error raised while that callback runs, before or after the call, SHALL be reported after the clear.

Clearing SHALL change nothing else:
- It SHALL NOT reload the configuration.
- It SHALL NOT enable a callback that an error disabled, and SHALL NOT clear a plugin's failed mark. These SHALL wait for the next load, as before.
- It SHALL NOT remove an error from any log.
- It SHALL clear only the calling client's errors. An error of the server's Lua SHALL leave only this client's error list. Other clients SHALL keep it, and the server SHALL still send its latest error to each client that attaches, as the server-runtime capability defines.

#### Scenario: Syntax error
- **WHEN** `XDG_CONFIG_HOME` is unset, the home directory is `/home/u`, line 12 of `user/init.lua` holds a syntax error, and a client attaches
- **THEN** the default configuration's sidebar shows `!` on its bottom row
- **AND** `gband.errors()` holds the error, beginning `/home/u/.config/gband/user/init.lua:12:`
- **AND** the client log records it

#### Scenario: Plugin error in the sidebar
- **WHEN** `XDG_DATA_HOME` is `/tmp/data`, line 3 of `/tmp/data/gband/plugins/hello/client.lua` raises `boom`, and a client attaches
- **THEN** the sidebar shows `!` on its bottom row
- **AND** `gband.errors()` holds `hello: /tmp/data/gband/plugins/hello/client.lua:3: boom`

#### Scenario: Server error in the client
- **WHEN** line 2 of `user/server.lua` holds a syntax error and a client starts a server
- **THEN** the client shows an error beginning `server: ` and naming `user/server.lua:2:`
- **AND** the server log records it

#### Scenario: Plugin error on the banner
- **WHEN** `XDG_DATA_HOME` is `/tmp/data`, `user/init.lua` does not set up `gband.sidebar`, line 3 of `/tmp/data/gband/plugins/hello/client.lua` raises `boom`, and a client attaches
- **THEN** the client's bottom row shows `hello: /tmp/data/gband/plugins/hello/client.lua:3: boom`

#### Scenario: Banner without a sidebar
- **WHEN** `user/init.lua` does not set up `gband.sidebar`, line 3 of a plugin's `client.lua` raises `boom`, and a client attaches
- **THEN** the client's bottom row shows the error over the ribbon

#### Scenario: Banner cleared by a good load
- **WHEN** the client shows a configuration error and the user fixes `user/init.lua`
- **THEN** the error disappears once the configuration reloads

#### Scenario: Errors kept in order
- **WHEN** two plugins raise errors while the configuration loads, `alpha` first and then `beta`
- **THEN** `gband.errors()` returns the error of `alpha` and then the error of `beta`

#### Scenario: Cleared by hand
- **WHEN** `gband.errors()` holds two errors and a binding function calls `gband.clear_errors()` and then reads `gband.errors()`
- **THEN** the binding function reads an empty list
- **AND** once it returns, the sidebar shows no error marker
- **AND** the client log still records both errors

#### Scenario: Banner cleared by hand
- **WHEN** `user/init.lua` does not set up `gband.sidebar`, the client's bottom row shows a plugin error over a tile, and a binding function calls `gband.clear_errors()`
- **THEN** the bottom row shows the tile again

#### Scenario: Error after a clear
- **WHEN** a binding function has called `gband.clear_errors()`, and another binding function then raises `boom`
- **THEN** `gband.errors()` holds only that error
- **AND** the sidebar shows `!` on its bottom row

#### Scenario: Error in the clearing callback
- **WHEN** a binding function calls `gband.clear_errors()` and then raises an error on line 9 of `user/init.lua`
- **THEN** `gband.errors()` holds only the error at `user/init.lua` line 9

#### Scenario: Clear while loading
- **WHEN** line 4 of `user/init.lua` calls `gband.clear_errors()` at the top level
- **THEN** loading fails with an error at `user/init.lua` line 4

#### Scenario: Clearing keeps a failed plugin failed
- **WHEN** the plugin `broken` registers the action `broken.go`, binds `alt+b` to it, then raises an error in `setup`, and a binding function calls `gband.clear_errors()`
- **THEN** `gband.errors()` returns an empty list and no `ConfigReloaded` event is emitted
- **AND** Alt+B still does nothing

#### Scenario: Server error cleared in one client
- **WHEN** line 2 of `user/server.lua` holds a syntax error, two clients are attached and both show an error beginning `server: `, and a binding function in the first calls `gband.clear_errors()`
- **THEN** the first client shows no error
- **AND** the second client still shows the error
- **AND** a third client that attaches then shows the error

### Requirement: Mouse modifier
The client SHALL have the built-in option `mouse_mod`, with the description `modifiers that mod stands for in a mouse name`. Its value SHALL be one or more of the modifiers `ctrl`, `alt` and `shift`, joined by `+`, in any order and case, and its default SHALL be `"alt"`. Any other value SHALL be rejected as "Options and their values" defines for a value of the wrong type. Reading `gband.opt.mouse_mod` SHALL return the modifiers as set, in lowercase, in the order `ctrl`, `alt`, `shift`, joined by `+`.

`mod` in the key name of a binding SHALL stand for the modifiers that `mouse_mod` names when the bindings are used, added to any other modifier the name gives, as `prefix` stands for the key the `prefix` option names. Setting `mouse_mod` before or after the binding is made, in the same load, SHALL give the same binding. `gband.keymap.list` SHALL give the key of such a binding as written, such as `mod+leftmouse`.

When more than one binding of one table matches the same mouse event, the binding made last SHALL run, so a binding made after `gband.keystyle.use()` takes the place of a preset's binding for the same event. Binding a key name again SHALL make that binding the last made.

#### Scenario: Default mouse modifier
- **WHEN** `user/init.lua` binds `mod+leftmouse` in `root` to `gband.action.drag_window` and does not set `mouse_mod`, and the user drags a floating window 5 cells right with the left button and Alt held
- **THEN** the floating window's box moves 5 cells right

#### Scenario: Changed mouse modifier
- **WHEN** `user/init.lua` calls `gband.keystyle.use()` and then sets `gband.opt.mouse_mod = "ctrl+alt"`, and the user drags a floating window with the left button and Ctrl and Alt held
- **THEN** the floating window's box moves with the pointer
- **AND** a left drag with Alt alone over a program that reports the mouse reaches the program

#### Scenario: Read back
- **WHEN** `user/init.lua` sets `gband.opt.mouse_mod = "Alt+Ctrl"` and reads `gband.opt.mouse_mod`
- **THEN** it reads `ctrl+alt`

#### Scenario: Invalid mouse modifier
- **WHEN** line 2 of `user/init.lua` sets `gband.opt.mouse_mod = "super"`
- **THEN** a configuration error at `user/init.lua` line 2 naming `mouse_mod` is reported
- **AND** `gband.opt.mouse_mod` reads `alt`

#### Scenario: Own binding after the preset
- **WHEN** `user/init.lua` calls `gband.keystyle.use()` and then binds `alt+leftmouse` in `root` to a function, and the user drags a floating window with the left button and Alt held
- **THEN** the function runs and the floating window's box does not move

#### Scenario: Listed as written
- **WHEN** `user/init.lua` binds `mod+rightmouse` in `root` and reads `gband.keymap.list("root")`
- **THEN** the entry's `key` is `mod+rightmouse`

### Requirement: Animation options
The client SHALL have two more built-in options, alongside those "Options and their values" lists:

| option | value | default | description |
|---|---|---|---|
| `animations` | a boolean | `true` | `whether the client animates changes of the layout and the view` |
| `animation_speed` | a number at least 0.1 and at most 10 | `1` | `how fast animations run, 2 being twice as fast as 1` |

Their effect SHALL be as the animations capability defines. A value of the wrong type or out of range SHALL be rejected as "Options and their values" defines. Reading `gband.opt.animations` SHALL return a boolean, and reading `gband.opt.animation_speed` SHALL return a number equal to the value set. `gband.opt.list()` SHALL hold an entry for each, with the type `"boolean"` or `"number"`, and the default and description above. The default configuration SHALL set both options to their defaults, with the other client options.

#### Scenario: Defaults
- **WHEN** the default configuration is evaluated alone and reads `gband.opt.animations` and `gband.opt.animation_speed`
- **THEN** they read `true` and `1`

#### Scenario: Set in the defaults file
- **WHEN** `defaults/init.lua` is read
- **THEN** it holds `gband.opt.animations = true` and `gband.opt.animation_speed = 1`

#### Scenario: Read back
- **WHEN** `user/init.lua` sets `gband.opt.animation_speed = 1.5` and `gband.set({ animations = false })`, then reads both options
- **THEN** `gband.opt.animation_speed` reads `1.5`
- **AND** `gband.opt.animations` reads `false`

#### Scenario: Speed out of range
- **WHEN** line 2 of `user/init.lua` sets `gband.opt.animation_speed = 0`
- **THEN** a configuration error at `user/init.lua` line 2 naming `animation_speed` is reported
- **AND** `gband.opt.animation_speed` reads `1`

#### Scenario: Wrong type
- **WHEN** line 3 of `user/init.lua` sets `gband.opt.animations = "no"`
- **THEN** a configuration error at `user/init.lua` line 3 naming `animations` is reported
- **AND** `gband.opt.animations` reads `true`

#### Scenario: Client side only
- **WHEN** line 1 of `user/server.lua` sets `gband.opt.animations = false`
- **THEN** the error reported at `user/server.lua` line 1 names the client side
