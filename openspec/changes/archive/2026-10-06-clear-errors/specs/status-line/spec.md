## MODIFIED Requirements

### Requirement: Error item
While the status line is drawn and the configuration capability reports an error, including an error of the server's Lua, the status line SHALL show the error item: the word `error` in `StatusLineError`, as the first line of the top region, before every component. It SHALL have a priority above every component's, so it is dropped last, and it SHALL be cut like any line. The error item SHALL be shown until the configuration capability empties the error list, by a load with none of the client's own errors or by `gband.clear_errors()`. While no error is reported, the error item SHALL take no row. The error's text SHALL be read through the error list, as the configuration and error-list capabilities define.

#### Scenario: Plugin error at start
- **WHEN** line 3 of the plugin `hello`'s `client.lua` raises `boom`, and a client attaches
- **THEN** row 0 of the status line shows `error`, and the ribbon area is unchanged
- **AND** `gband.errors()` holds `hello: <path>/client.lua:3: boom`

#### Scenario: Server plugin error
- **WHEN** line 5 of the plugin `agent-status`'s `server.lua` raises `boom`, and a client attaches
- **THEN** row 0 of the status line shows `error`
- **AND** `gband.errors()` holds `server: agent-status: <path>/server.lua:5: boom`

#### Scenario: Cleared by a good load
- **WHEN** the error item shows an error and the user fixes `user/init.lua`
- **THEN** the error item disappears once the configuration reloads

#### Scenario: Cleared by hand
- **WHEN** the error item shows an error and a binding function calls `gband.clear_errors()`
- **THEN** the error item disappears once the binding function returns, and the components below it move up one row

#### Scenario: Disabled component stays disabled
- **WHEN** a component was disabled because its `render` raised an error, the error item shows it, and a binding function calls `gband.clear_errors()`
- **THEN** the error item disappears
- **AND** `gband.ui.statusline.list()` still gives the component `enabled` false, and it stays hidden until the configuration next loads
