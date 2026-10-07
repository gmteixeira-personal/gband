## MODIFIED Requirements

### Requirement: Side and API version
`gband.side` SHALL be the string `"client"` in a client, `"server"` in the server, and `"test"` in a test file that `gband test` runs, as the plugin-testing capability defines. `gband.api_version` SHALL be the integer `2` on every side. Both SHALL be set before the init file or the test file runs.

#### Scenario: Side and version
- **WHEN** `user/init.lua` reads `gband.side` and `gband.api_version`
- **THEN** they are `"client"` and `2`

#### Scenario: Server side
- **WHEN** `user/server.lua` reads `gband.side`
- **THEN** it is `"server"`

#### Scenario: Test side
- **WHEN** a test file read by `gband test -` prints `gband.side`
- **THEN** standard output holds `test`

### Requirement: Plugin modules
A plugin module SHALL be a table with a `setup` function, an optional string `name` and an optional integer `api`. `gband.plugin(name, opts)` SHALL require the module `name`, check its shape, and call its `setup` with `opts`, or with an empty table when `opts` is nil. The plugin's name SHALL be the module's `name` field when it has one, and `name` otherwise. When the module's `api` differs from `gband.api_version`, the process SHALL record a warning naming the plugin and both versions in its log, and SHALL still set the plugin up. `gband.plugin` SHALL return `true` when `setup` returns, and `false` when the module cannot be found, has the wrong shape, or its `setup` raises an error, each reported as a plugin error. Calling `gband.plugin` again for a plugin already set up in this load SHALL be a plugin error, and SHALL NOT call `setup` again.

#### Scenario: Setup receives the options
- **WHEN** `user/init.lua` calls `gband.plugin("hello", { greeting = "hi" })` and `hello`'s `setup` stores `opts.greeting`
- **THEN** the stored value is `"hi"`
- **AND** `gband.plugin` returns `true`

#### Scenario: API mismatch
- **WHEN** a plugin module declares `api = 1`
- **THEN** the log records a warning naming the plugin, version 1 and version 2
- **AND** its `setup` runs

#### Scenario: Missing module
- **WHEN** `user/init.lua` calls `gband.plugin("absent")` and no runtimepath entry holds it
- **THEN** `gband.plugin` returns `false` and a plugin error names `absent`
- **AND** the rest of `user/init.lua` runs

#### Scenario: Wrong shape
- **WHEN** the module `hello` returns a table without `setup`
- **THEN** `gband.plugin("hello")` returns `false` and a plugin error names `hello`
