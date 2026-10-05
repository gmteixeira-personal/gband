## MODIFIED Requirements

### Requirement: Configuration file
The configuration file SHALL be `user/init.lua` in the configuration directory. The client and the server SHALL each load the configuration when they start, after preparing the configuration directory. Loading SHALL use one new Lua state, starting from the declared defaults of the options and no key bindings, and SHALL evaluate the init file and then the plugin files the plugins capability sources from the runtimepath. The init file SHALL be the configuration file when it exists, and the default configuration otherwise. Before the init file, loading SHALL install only the `gband` API, which includes the status line container and its built-in groups, as the status-line capability defines, and the bundled `default` colorscheme, as the colorschemes capability defines. Nothing of the default configuration SHALL be evaluated before the init file: the configuration file replaces the default configuration rather than adding to it. A missing configuration file SHALL NOT be an error. Each process SHALL apply only the parts of the configuration it owns: the server the column width options, and the client the prefix, the key tables, the camera policy, the status line options, the highlight groups, the status line components and the callbacks.

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
The default configuration SHALL use only the `gband` API that the configuration file can use. It SHALL make every key binding with `gband.keymap.set`, giving each the description of the action it binds, as the actions capability lists them. Evaluated alone, it SHALL produce the declared defaults of the options and the default key bindings of the client-attach capability, and SHALL set up, with `gband.plugin` and no options, the bundled segment plugins `gband.statusline.band`, `gband.statusline.mode` and `gband.statusline.position`, in that order. The default key bindings and the setup of the segment plugins SHALL exist only in the default configuration.

#### Scenario: Defaults reproduce the built-in behaviour
- **WHEN** the default configuration is evaluated alone
- **THEN** the options equal the defaults in the "Options" table
- **AND** the bindings equal the client-attach capability's default table, entry for entry
- **AND** `gband.ui.statusline.list()` names exactly the components `band`, `mode` and `position`

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
| `statusline_position` | `"bottom"`, `"top"` or `"off"` | `"bottom"` |
| `statusline_height` | an integer from 1 to 8 | `1` |
| `statusline_separator` | a string | `" │ "` |

A width SHALL be read as the fraction closest to the number whose denominator is at most 100, in lowest terms, so `1/3` reads as 1/3. The width presets SHALL be held in ascending order with duplicates removed.

`gband.set` SHALL take one table of options. Each call SHALL change only the options the table names, and a later call SHALL override an earlier one. An unknown option name, a value of the wrong type, a value out of range or an unknown camera policy given to `gband.set` SHALL be a configuration error naming the option.

Reading `gband.opt.<name>` SHALL return the option's current value: a key name for `prefix`, a number for a width, a list of numbers in ascending order for `width_presets`, a string for `center_focused_column`, `statusline_position` and `statusline_separator`, an integer for `statusline_height`, and the value as set for a declared option. Assigning `gband.opt.<name>` SHALL set the option. A value that the option's type rejects SHALL be reported as a configuration error at the line of the assignment, SHALL NOT fail the load, and SHALL set the option to its declared default. An assignment to a name no option declares SHALL be held until an option of that name is declared later in the same load, and then validated; one still undeclared when loading finishes SHALL be reported as a configuration error at the line of the assignment, and SHALL NOT fail the load.

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

#### Scenario: Invalid status line height
- **WHEN** line 2 of `user/init.lua` sets `gband.opt.statusline_height = 0`
- **THEN** loading succeeds with a status line height of 1
- **AND** an error at `user/init.lua` line 2 naming `statusline_height` is reported

### Requirement: Configuration errors
A configuration error SHALL be reported as the file's path, a colon, the line, a colon and a message. The line SHALL be the line of a syntax error, the line where a runtime error was raised, or the line of the call or assignment that received the invalid value. A plugin error SHALL be reported the same way, preceded by the plugin's name, a colon and a space. Each process SHALL record every configuration error and plugin error in its log. The client SHALL also show the latest of them until the configuration next loads with neither. While the status line is drawn, the client SHALL show it in the status line's error item, as the status-line capability defines. While no status line is drawn, the client SHALL show it as a banner on the bottom row of the ribbon area, over the ribbon and cut to the terminal's width. Neither SHALL change the size the client reports.

#### Scenario: Syntax error
- **WHEN** `XDG_CONFIG_HOME` is unset, the home directory is `/home/u`, line 12 of `user/init.lua` holds a syntax error, and a client attaches
- **THEN** the client's bottom row shows the error, beginning `/home/u/.config/gband/user/init.lua:12:`
- **AND** the client log and the server log record it

#### Scenario: Plugin error in the status line
- **WHEN** `XDG_DATA_HOME` is `/tmp/data`, line 3 of `/tmp/data/gband/plugins/hello/plugin/hello.lua` raises `boom`, and a client attaches
- **THEN** the status line's error item shows `hello: /tmp/data/gband/plugins/hello/plugin/hello.lua:3: boom`

#### Scenario: Banner without a status line
- **WHEN** `statusline_position` is `"off"`, line 3 of a plugin's `plugin/hello.lua` raises `boom`, and a client attaches
- **THEN** the client's bottom row shows the error over the ribbon

#### Scenario: Banner cleared by a good load
- **WHEN** the client shows a configuration error and the user fixes `user/init.lua`
- **THEN** the error disappears once the configuration reloads
