## MODIFIED Requirements

### Requirement: Options
Every option SHALL have a name, a type, a declared default and a description, and SHALL belong to one side. The built-in client options SHALL be:

| option | value | default |
|---|---|---|
| `prefix` | one key, as "Key names" defines | `"ctrl+space"` |
| `center_focused_column` | `"never"`, `"always"` or `"on-overflow"` | `"never"` |
| `notify_style` | `"osc9"`, `"osc777"`, `"bell"` or `"none"` | `"osc9"` |
| `tile_border_sides` | a list of side names, as the borders capability defines them | `{ "top", "right", "bottom", "left" }` |
| `tile_border_chars` | a character set, as the borders capability defines it | `"plain"` |
| `floating_border_sides` | a list of side names | `{ "top", "right", "bottom", "left" }` |
| `floating_border_chars` | a character set | `"plain"` |
| `width_step` | a number greater than 0 and at most 10000 | `1/10` |
| `height_step` | a number greater than 0 and at most 1 | `1/10` |

The built-in server options SHALL be:

| option | value | default |
|---|---|---|
| `default_column_width` | a number greater than 0 and at most 10000 | `1/2` |
| `width_presets` | a list of one or more numbers, each greater than 0 and at most 10000 | `{ 1/3, 1/2, 2/3 }` |

A process SHALL know only the options of its own side, and options declared in its own Lua state. A width and a step SHALL each be read as the fraction closest to the number whose denominator is at most 100, in lowest terms, so `1/3` reads as 1/3. The width presets SHALL be held in ascending order with duplicates removed. A list of side names SHALL hold only `top`, `right`, `bottom` and `left`, and SHALL be held in that order with duplicates removed; an empty list is valid.

`gband.set` SHALL take one table of options. Each call SHALL change only the options the table names, and a later call SHALL override an earlier one. An unknown option name, a value of the wrong type, a value out of range or an unknown camera policy given to `gband.set` SHALL be a configuration error naming the option.

Reading `gband.opt.<name>` SHALL return the option's current value: a key name for `prefix`, a number for a width or a step, a list of numbers in ascending order for `width_presets`, a string for `center_focused_column` and `notify_style`, a list of side names in the held order for `tile_border_sides` and `floating_border_sides`, the name or a new list of the eight strings for `tile_border_chars` and `floating_border_chars`, and the value as set for a declared option. Assigning `gband.opt.<name>` SHALL set the option. A value that the option's type rejects SHALL be reported as a configuration error at the line of the assignment, SHALL NOT fail the load, and SHALL set the option to its declared default. An assignment to a name no option declares SHALL be held until an option of that name is declared later in the same load, and then validated; one still undeclared when loading finishes SHALL be reported as a configuration error at the line of the assignment, and SHALL NOT fail the load. The error for a built-in option of the other side SHALL name the side that owns it.

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

#### Scenario: Invalid step
- **WHEN** line 2 of `user/init.lua` sets `gband.opt.width_step = 0`
- **THEN** loading succeeds with a width step of 1/10
- **AND** an error at `user/init.lua` line 2 naming `width_step` is reported

#### Scenario: Sides are held in order
- **WHEN** `user/init.lua` sets `tile_border_sides` to `{ "left", "top", "left" }` and then reads it
- **THEN** the value read is `{ "top", "left" }`

#### Scenario: Invalid status line height
- **WHEN** line 2 of `user/init.lua` sets `gband.opt.statusline_height = 2`, an option that no longer exists, and nothing declares it
- **THEN** loading succeeds
- **AND** an error at `user/init.lua` line 2 naming `statusline_height` is reported

### Requirement: Defaults use the public API
The default configuration SHALL use only the `gband` API that the configuration file can use. It SHALL declare `prefix` a mode with the label `navigation`, with `gband.keymap.mode`. It SHALL make every key binding with `gband.keymap.set`. A binding to an action SHALL take the description of the action it binds, as the actions capability lists them or as `gband.action.list()` gives it for a registered action. A binding to a Lua function SHALL take the description the client-attach capability's default table gives its key. Evaluated alone, it SHALL produce the declared defaults of the options and the default key bindings of the client-attach capability. It SHALL set up, with `gband.plugin` and no options, the bundled key list plugin `gband.keylist` and then the bundled error list plugin `gband.errors` before it makes its key bindings, then the bundled status line plugin `gband.statusline`, then the bundled segment plugins `gband.statusline.band`, `gband.statusline.mode`, `gband.statusline.hints` and `gband.statusline.position`, in that order. The default key bindings, the declaration of the `navigation` mode and the setup of the bundled plugins SHALL exist only in the default configuration.

#### Scenario: Defaults reproduce the built-in behaviour
- **WHEN** the default configuration is evaluated alone
- **THEN** the options equal the defaults in the "Options" table
- **AND** the bindings equal the client-attach capability's default table, entry for entry
- **AND** `gband.ui.statusline.list()` names exactly the components `band`, `hints`, `mode` and `position`
- **AND** `gband.bar.list()` holds exactly one bar, `statusline`, on the side `left`, 20 columns wide on an 80×24 terminal

#### Scenario: Every default binding is described
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** every entry with an action has the description `gband.action.list()` gives its action
- **AND** every entry without an action has the description the client-attach capability's default table gives its key

#### Scenario: Navigation mode declared by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.label("prefix")` is read
- **THEN** it returns `navigation`

#### Scenario: Key list set up by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** the entry for `?` has the action `keylist.open`
- **AND** it comes after the entry for `R` and before the entry for `D`

#### Scenario: Copied defaults load unchanged
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua`
- **THEN** loading succeeds
- **AND** the options, bindings and modes equal those of the default configuration
### Requirement: Configuration errors
A configuration error SHALL be reported as the file's path, a colon, the line, a colon and a message. The line SHALL be the line of a syntax error, the line where a runtime error was raised, or the line of the call or assignment that received the invalid value. A plugin error SHALL be reported the same way, preceded by the plugin's name, a colon and a space. Each process SHALL record every configuration error and plugin error of its own Lua in its log. The client SHALL also report the errors of the server's Lua that the server sends it, as the server-runtime capability defines, preceded by `server: `, and the plugin requirement errors of the plugin-bridge capability. The client SHALL keep its error list: every error it has reported since its configuration last loaded with none of its own, oldest first. A load with none of its own SHALL empty the list. `gband.errors()` SHALL return a new list of the error list's texts, oldest first. The client SHALL show the latest error until the list is emptied. While the status line is drawn, the client SHALL show the status line's error item, as the status-line capability defines, and the error's text only through the error list. While no status line is drawn, the client SHALL show it as a banner on the bottom row of the ribbon area, over the ribbon and cut to the ribbon area's width. Neither SHALL change the size the client reports.

#### Scenario: Syntax error
- **WHEN** `XDG_CONFIG_HOME` is unset, the home directory is `/home/u`, line 12 of `user/init.lua` holds a syntax error, and a client attaches
- **THEN** the default configuration's status line shows `error` on row 0
- **AND** `gband.errors()` holds the error, beginning `/home/u/.config/gband/user/init.lua:12:`
- **AND** the client log records it

#### Scenario: Plugin error in the status line
- **WHEN** `XDG_DATA_HOME` is `/tmp/data`, line 3 of `/tmp/data/gband/plugins/hello/client.lua` raises `boom`, and a client attaches
- **THEN** the status line's error item shows `error`
- **AND** `gband.errors()` holds `hello: /tmp/data/gband/plugins/hello/client.lua:3: boom`

#### Scenario: Server error in the client
- **WHEN** line 2 of `user/server.lua` holds a syntax error and a client starts a server
- **THEN** the client shows an error beginning `server: ` and naming `user/server.lua:2:`
- **AND** the server log records it

#### Scenario: Plugin error on the banner
- **WHEN** `XDG_DATA_HOME` is `/tmp/data`, `user/init.lua` does not set up `gband.statusline`, line 3 of `/tmp/data/gband/plugins/hello/client.lua` raises `boom`, and a client attaches
- **THEN** the client's bottom row shows `hello: /tmp/data/gband/plugins/hello/client.lua:3: boom`

#### Scenario: Banner without a status line
- **WHEN** `user/init.lua` does not set up `gband.statusline`, line 3 of a plugin's `client.lua` raises `boom`, and a client attaches
- **THEN** the client's bottom row shows the error over the ribbon

#### Scenario: Banner cleared by a good load
- **WHEN** the client shows a configuration error and the user fixes `user/init.lua`
- **THEN** the error disappears once the configuration reloads

#### Scenario: Errors kept in order
- **WHEN** two plugins raise errors while the configuration loads, `alpha` first and then `beta`
- **THEN** `gband.errors()` returns the error of `alpha` and then the error of `beta`
