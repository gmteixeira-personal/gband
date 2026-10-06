## MODIFIED Requirements

### Requirement: Side guard
Each process's `gband` table SHALL hold only the API of its side. Reading a field of `gband`, or of `gband.action`, that only another side provides SHALL raise an error naming the field and the side that provides it, at the line of the read. The fields only the client provides SHALL be `bind`, `unbind`, `spawn`, `keymap`, `ui`, `hl`, `colorscheme`, `layout`, `view`, `window`, `band`, `win`, `bar`, `errors`, `clear_errors`, `rpc`, `notify`, `bell`, `clipboard`, `open`, and the view and client actions in `gband.action`. The fields only the server provides SHALL be `sessions` and `session`. Every other field the plugins, configuration, lua-events and lua-commands capabilities define SHALL exist on the client and the server, with each side's own behaviour where the server-runtime capability defines one. The test side's `gband` SHALL hold only `side` and `api_version`; reading any field that the client or the server provides there SHALL raise an error naming the field and the sides that provide it.

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

#### Scenario: Bars in the server
- **WHEN** line 2 of a plugin's `server.lua` reads `gband.bar`
- **THEN** a plugin error at that line names `gband.bar` and the client

#### Scenario: Clearing errors in the server
- **WHEN** line 4 of a plugin's `server.lua` calls `gband.clear_errors()`
- **THEN** a plugin error at that line names `gband.clear_errors` and the client
