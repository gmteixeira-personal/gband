## MODIFIED Requirements

### Requirement: Options
`gband.set` SHALL take one table of options. Each call SHALL change only the options the table names, and a later call SHALL override an earlier one. The options SHALL be:

| option | value | default |
|---|---|---|
| `prefix` | one key, as "Key names" defines | `"ctrl+space"` |
| `default_column_width` | a number greater than 0 and at most 10000 | `1/2` |
| `width_presets` | a list of one or more numbers, each greater than 0 and at most 10000 | `{ 1/3, 1/2, 2/3 }` |
| `center_focused_column` | `"never"`, `"always"` or `"on-overflow"` | `"never"` |

A width SHALL be read as the fraction closest to the number whose denominator is at most 100, in lowest terms, so `1/3` reads as 1/3. The width presets SHALL be held in ascending order with duplicates removed. An unknown option name, a value of the wrong type, a value out of range or an unknown camera policy SHALL be a configuration error naming the option.

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
- **AND** Ctrl+Space then `q` reaches the focused pane as `\x00` and then `q`

### Requirement: Bind and unbind keys
`gband.bind(keys, action)` SHALL bind `keys` to `action`. `keys` SHALL be one key name, a direct binding, or `prefix` followed by a space and one key name, a prefix binding, where `prefix` stands for the key the `prefix` option names when the bindings are used. `prefix prefix` SHALL bind the prefix key pressed twice. `action` SHALL be a value from `gband.action` or a Lua function. Binding keys that are already bound SHALL replace the earlier binding. `gband.unbind(keys)` SHALL remove the binding of `keys`, and SHALL do nothing when `keys` is unbound.

A key list of more than two keys, two keys whose first is not `prefix`, an invalid key name, or an action that is neither a `gband.action` value nor a function SHALL be a configuration error. Once the configuration is evaluated, a direct binding of the key the `prefix` option names SHALL be a configuration error at the line of that binding.

#### Scenario: Direct binding
- **WHEN** `user/init.lua` calls `gband.bind("alt+h", gband.action.focus_column_left)`
- **THEN** Alt+H focuses the column to the left without the prefix

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

#### Scenario: Key chain too long
- **WHEN** `user/init.lua` binds `prefix w q`
- **THEN** loading fails with an error naming `prefix w q`

#### Scenario: Not an action
- **WHEN** `user/init.lua` calls `gband.bind("alt+h", "focus_column_left")`
- **THEN** loading fails with an error naming the binding of `alt+h`

### Requirement: Last good configuration
Loading SHALL apply all of a configuration or none of it. When loading fails, the process SHALL keep the configuration it last loaded without error, or the default configuration alone when it has loaded none.

#### Scenario: Broken file at start
- **WHEN** `user/init.lua` sets `prefix` to `"ctrl+b"` and then raises an error, and a client attaches
- **THEN** the prefix is Ctrl+Space and the default bindings apply

#### Scenario: Broken edit keeps the running configuration
- **WHEN** `user/init.lua` binds `alt+h` and has loaded, and the user saves a version that binds `alt+j` and has a syntax error
- **THEN** Alt+H still focuses the column to the left
- **AND** Alt+J reaches the focused pane

