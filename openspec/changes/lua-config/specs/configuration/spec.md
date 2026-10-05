## Purpose

Defines gband's Lua configuration: where the configuration file lives, how the client and the server evaluate it on top of the built-in defaults, the `gband` API it uses to set options and bind keys, how errors are reported, and how a changed file is reloaded.

## ADDED Requirements

### Requirement: Configuration file
The configuration file SHALL be `gband/init.lua` under `$XDG_CONFIG_HOME` when that variable holds an absolute path, otherwise `.config/gband/init.lua` under the user's home directory. The client and the server SHALL each load the configuration when they start. Loading SHALL evaluate the built-in `defaults.lua`, then the configuration file when it exists, in one new Lua state. A missing configuration file SHALL NOT be an error. Each process SHALL apply only the parts of the configuration it owns: the server the column width options, and the client the prefix, the key bindings and the camera policy.

#### Scenario: No configuration file
- **WHEN** no `init.lua` exists and a client attaches
- **THEN** no error is reported
- **AND** the key bindings, options and behaviour are those of `defaults.lua`

#### Scenario: XDG_CONFIG_HOME is honoured
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `/tmp/cfg/gband/init.lua` sets `width_presets` to `{ 1/4, 1/2 }`, and a client starts a server and cycles a column of width 1/2
- **THEN** the column's width is 1/4

### Requirement: Defaults use the public API
`defaults.lua` SHALL be built into the executable and SHALL use only the `gband` API that the configuration file can use. Evaluated alone, it SHALL produce the default options of "Options" and the default key bindings of the client-attach capability, and nothing in gband SHALL hold a second copy of those defaults.

#### Scenario: Defaults reproduce the built-in behaviour
- **WHEN** `defaults.lua` is evaluated alone
- **THEN** the options equal the defaults in the "Options" table
- **AND** the bindings equal the client-attach capability's default table, entry for entry

### Requirement: Options
`gband.set` SHALL take one table of options. Each call SHALL change only the options the table names, and a later call SHALL override an earlier one. The options SHALL be:

| option | value | default |
|---|---|---|
| `prefix` | one key, as "Key names" defines | `"ctrl+a"` |
| `default_column_width` | a number greater than 0 and at most 10000 | `1/2` |
| `width_presets` | a list of one or more numbers, each greater than 0 and at most 10000 | `{ 1/3, 1/2, 2/3 }` |
| `center_focused_column` | `"never"`, `"always"` or `"on-overflow"` | `"never"` |

A width SHALL be read as the fraction closest to the number whose denominator is at most 100, in lowest terms, so `1/3` reads as 1/3. The width presets SHALL be held in ascending order with duplicates removed. An unknown option name, a value of the wrong type, a value out of range or an unknown camera policy SHALL be a configuration error naming the option.

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
`gband.bind(keys, action)` SHALL bind `keys` to `action`. `keys` SHALL be one key name, a direct binding, or `prefix` followed by a space and one key name, a prefix binding, where `prefix` stands for the key the `prefix` option names when the bindings are used. `prefix prefix` SHALL bind the prefix key pressed twice. `action` SHALL be a value from `gband.action` or a Lua function. Binding keys that are already bound SHALL replace the earlier binding, including one made by `defaults.lua`. `gband.unbind(keys)` SHALL remove the binding of `keys`, and SHALL do nothing when `keys` is unbound.

A key list of more than two keys, two keys whose first is not `prefix`, an invalid key name, or an action that is neither a `gband.action` value nor a function SHALL be a configuration error. Once the configuration is evaluated, a direct binding of the key the `prefix` option names SHALL be a configuration error at the line of that binding.

#### Scenario: Direct binding
- **WHEN** `init.lua` calls `gband.bind("alt+h", gband.action.focus_column_left)`
- **THEN** Alt+H focuses the column to the left without the prefix

#### Scenario: Override a default
- **WHEN** `init.lua` calls `gband.bind("prefix q", gband.action.detach)`
- **THEN** Ctrl+A then `q` detaches, and no other binding closes the pane

#### Scenario: Unbind a default
- **WHEN** `init.lua` calls `gband.unbind("prefix q")`
- **THEN** Ctrl+A then `q` discards both keys

#### Scenario: Prefix changed after binding
- **WHEN** `init.lua` sets `prefix` to `"ctrl+b"` after `defaults.lua` bound `prefix h`
- **THEN** Ctrl+B then `h` focuses the column to the left
- **AND** Ctrl+A reaches the focused pane as `\x01`

#### Scenario: Direct binding of the prefix key
- **WHEN** line 4 of `init.lua` binds `ctrl+a` while the prefix is `ctrl+a`
- **THEN** loading fails with an error at `init.lua` line 4

#### Scenario: Key chain too long
- **WHEN** `init.lua` binds `prefix w q`
- **THEN** loading fails with an error naming `prefix w q`

#### Scenario: Not an action
- **WHEN** `init.lua` calls `gband.bind("alt+h", "focus_column_left")`
- **THEN** loading fails with an error naming the binding of `alt+h`

### Requirement: Action values
`gband.action` SHALL hold one value for every action, under the Lua name the actions capability gives it. Calling an action value inside a binding function SHALL dispatch that action, with the same effect as pressing a key bound to it.

#### Scenario: Action called from a function
- **WHEN** `init.lua` binds `alt+w` to a function that calls `gband.action.focus_column_right()` twice, and the first of three columns is focused
- **THEN** pressing Alt+W focuses the third column

### Requirement: Spawn a program
`gband.spawn` SHALL take a table whose optional `cmd` field is a string or a list of strings. Called inside a binding function, it SHALL dispatch open pane, as the actions capability resolves it, naming the program to run: a string as a command line the user's shell runs, a list as the program and its arguments, and no `cmd` as the user's shell. A `cmd` of any other type, an empty list, or a field other than `cmd` SHALL be an error.

#### Scenario: Spawn a command line
- **WHEN** `init.lua` binds `alt+n` to `function() gband.spawn({ cmd = "fish" }) end` and the user presses Alt+N
- **THEN** a new pane opens right of the focused pane's column, running `fish`, and is focused

#### Scenario: Spawn an argument list
- **WHEN** a binding function calls `gband.spawn({ cmd = { "htop", "-d", "10" } })`
- **THEN** a new pane runs `htop` with the arguments `-d` and `10`

### Requirement: Binding functions
A binding function SHALL run in the client, with no arguments, when its keys are pressed. Calling an action value or `gband.spawn` outside a binding function SHALL be a configuration error. An error raised while a binding function runs SHALL be reported as "Configuration errors" defines, and the actions the function dispatched before the error SHALL stand. The server SHALL never run a binding function.

#### Scenario: Action during evaluation
- **WHEN** line 7 of `init.lua` calls `gband.action.close_pane()` at the top level
- **THEN** loading fails with an error at `init.lua` line 7

#### Scenario: Error in a binding function
- **WHEN** a binding function calls `gband.action.focus_column_left()` and then raises an error on line 9 of `init.lua`
- **THEN** the column to the left is focused
- **AND** the client shows an error at `init.lua` line 9

### Requirement: Configuration errors
A configuration error SHALL be reported as the configuration file's path, a colon, the line, a colon and a message. The line SHALL be the line of a syntax error, the line where a runtime error was raised, or the line of the `gband.set` or `gband.bind` call that received the invalid value. Each process SHALL record every configuration error in its log. The client SHALL also show the latest configuration error on its terminal's bottom row, over the ribbon and cut to the terminal's width, until the configuration next loads without error. The banner SHALL NOT change the screen area the client reports.

#### Scenario: Syntax error
- **WHEN** `XDG_CONFIG_HOME` is unset, the home directory is `/home/u`, line 12 of `init.lua` holds a syntax error, and a client attaches
- **THEN** the client's bottom row shows the error, beginning `/home/u/.config/gband/init.lua:12:`
- **AND** the client log and the server log record it

#### Scenario: Banner cleared by a good load
- **WHEN** the client shows a configuration error and the user fixes `init.lua`
- **THEN** the banner disappears once the configuration reloads

### Requirement: Last good configuration
Loading SHALL apply all of a configuration or none of it. When loading fails, the process SHALL keep the configuration it last loaded without error, or the configuration of `defaults.lua` alone when it has loaded none.

#### Scenario: Broken file at start
- **WHEN** `init.lua` sets `prefix` to `"ctrl+b"` and then raises an error, and a client attaches
- **THEN** the prefix is Ctrl+A and the default bindings apply

#### Scenario: Broken edit keeps the running configuration
- **WHEN** `init.lua` binds `alt+h` and has loaded, and the user saves a version that binds `alt+j` and has a syntax error
- **THEN** Alt+H still focuses the column to the left
- **AND** Alt+J reaches the focused pane

### Requirement: Reload on change
Each process SHALL watch the configuration file. Within one second of the file being created, written, replaced or removed, the process SHALL load the configuration again in a new Lua state and, when loading succeeds, use the new configuration from then on. Options the server owns SHALL apply to session actions received after the reload, and no column's width SHALL change because of a reload. A prefix sequence in progress SHALL end without effect when the client reloads.

#### Scenario: New binding without restart
- **WHEN** a client is attached and the user adds `gband.bind("alt+l", gband.action.focus_column_right)` to `init.lua`
- **THEN** within a second Alt+L focuses the column to the right

#### Scenario: Editor replaces the file
- **WHEN** an editor saves `init.lua` by writing a new file and renaming it over the old one
- **THEN** the processes load the new content

#### Scenario: New default width
- **WHEN** a session holds a column of width 1/2 and the user sets `default_column_width` to `1/3`
- **THEN** that column keeps width 1/2
- **AND** a pane opened after the reload has a column of width 1/3

#### Scenario: File removed
- **WHEN** `init.lua` bound `alt+h` and the user deletes it
- **THEN** the configuration of `defaults.lua` alone applies and Alt+H reaches the focused pane
