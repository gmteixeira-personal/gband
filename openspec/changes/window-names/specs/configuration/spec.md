## MODIFIED Requirements

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

Reading `gband.opt.<name>` SHALL return the option's current value: a key name for `prefix`, a number for a width or a step, a list of numbers in ascending order for `width_presets`, a string for `center_focused_column` and `notify_style`, a list of side names in the held order for `tile_border_sides` and `floating_border_sides`, the name or a new list of the eight strings for `tile_border_chars` and `floating_border_chars`, a boolean for `loop_bands` and `window_titles`, and the value as set for a declared option. Assigning `gband.opt.<name>` SHALL set the option. A value that the option's type rejects SHALL be reported as a configuration error at the line of the assignment, SHALL NOT fail the load, and SHALL set the option to its declared default. An assignment to a name no option declares SHALL be held until an option of that name is declared later in the same load, and then validated; one still undeclared when loading finishes SHALL be reported as a configuration error at the line of the assignment, and SHALL NOT fail the load. The error for a built-in option of the other side SHALL name the side that owns it.

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

#### Scenario: Window titles off
- **WHEN** `user/init.lua` calls `gband.set({ window_titles = false })` and then reads `gband.opt.window_titles`
- **THEN** it reads `false`

#### Scenario: Window titles of the wrong type
- **WHEN** line 2 of `user/init.lua` calls `gband.set({ window_titles = "no" })`
- **THEN** a configuration error at `user/init.lua` line 2 names `window_titles`
