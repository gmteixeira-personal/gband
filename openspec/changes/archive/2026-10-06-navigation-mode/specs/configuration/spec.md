## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Defaults use the public API
The default configuration SHALL use only the `gband` API that the configuration file can use. It SHALL declare `prefix` a mode with the label `navigation`, with `gband.keymap.mode`. It SHALL make every key binding with `gband.keymap.set`. A binding to an action SHALL take the description of the action it binds, as the actions capability lists them or as `gband.action.list()` gives it for a registered action. A binding to a Lua function SHALL take the description the client-attach capability's default table gives its key. Evaluated alone, it SHALL produce the declared defaults of the options and the default key bindings of the client-attach capability. It SHALL set up, with `gband.plugin` and no options, the bundled key list plugin `gband.keylist` before it makes its key bindings, and the bundled segment plugins `gband.statusline.band`, `gband.statusline.mode`, `gband.statusline.hints` and `gband.statusline.position`, in that order. The default key bindings, the declaration of the `navigation` mode and the setup of the bundled plugins SHALL exist only in the default configuration.

#### Scenario: Defaults reproduce the built-in behaviour
- **WHEN** the default configuration is evaluated alone
- **THEN** the options equal the defaults in the "Options" table
- **AND** the bindings equal the client-attach capability's default table, entry for entry
- **AND** `gband.ui.statusline.list()` names exactly the components `band`, `hints`, `mode` and `position`

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
