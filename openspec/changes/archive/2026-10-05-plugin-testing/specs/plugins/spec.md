## MODIFIED Requirements

### Requirement: Side and API version
`gband.side` SHALL be the string `"client"` in a client, `"server"` in the server, and `"test"` in a test file that `gband test` runs, as the plugin-testing capability defines. `gband.api_version` SHALL be the integer `1` on every side. Both SHALL be set before the init file or the test file runs.

#### Scenario: Side and version
- **WHEN** `user/init.lua` reads `gband.side` and `gband.api_version`
- **THEN** they are `"client"` and `1`

#### Scenario: Server side
- **WHEN** `user/server.lua` reads `gband.side`
- **THEN** it is `"server"`

#### Scenario: Test side
- **WHEN** a test file read by `gband test -` prints `gband.side`
- **THEN** standard output holds `test`

### Requirement: Side guard
Each process's `gband` table SHALL hold only the API of its side. Reading a field of `gband`, or of `gband.action`, that only another side provides SHALL raise an error naming the field and the side that provides it, at the line of the read. The fields only the client provides SHALL be `bind`, `unbind`, `spawn`, `keymap`, `ui`, `hl`, `colorscheme`, `rpc`, `notify`, `bell`, `clipboard`, `open`, and the view and client actions in `gband.action`. The fields only the server provides SHALL be `sessions` and `session`. Every other field the plugins, configuration, lua-events and lua-commands capabilities define SHALL exist on the client and the server, with each side's own behaviour where the server-runtime capability defines one. The test side's `gband` SHALL hold only `side` and `api_version`; reading any field that the client or the server provides there SHALL raise an error naming the field and the sides that provide it.

#### Scenario: Client API in the server
- **WHEN** line 3 of a plugin's `server.lua` reads `gband.keymap`
- **THEN** a plugin error at that line names `gband.keymap` and the client

#### Scenario: Server API in the client
- **WHEN** line 2 of `user/init.lua` calls `gband.sessions()`
- **THEN** loading fails with an error at that line naming `gband.sessions` and the server

#### Scenario: View action in the server
- **WHEN** a server handler calls `gband.action.focus_column_left()`
- **THEN** a plugin error names `focus_column_left` and the client

#### Scenario: Client API in a test file
- **WHEN** line 4 of a test file reads `gband.opt`
- **THEN** the file fails with an error at line 4 naming `gband.opt`, the client and the server

### Requirement: Print goes to the log
In a client and in the server, `print` SHALL write its arguments, converted as Lua's `tostring` converts them and separated by tabs, to the process's log at the info level, naming the plugin the calling code belongs to when it belongs to one. It SHALL NOT write to the terminal. In the test side, `print` SHALL write the same line, followed by a newline, to the standard output of `gband test`.

#### Scenario: Print from a plugin
- **WHEN** the plugin `hello` calls `print("ready", 3)` while the client runs
- **THEN** the client log records `ready	3` naming `hello`
- **AND** the client's screen is unchanged

#### Scenario: Print in a test
- **WHEN** a case calls `print("step", 2)`
- **THEN** the standard output of `gband test` holds the line `step	2`
