## MODIFIED Requirements

### Requirement: Plugin files
After the init file returns, loading SHALL source, for each plugin in runtimepath order whose manifest is valid, its side file once: `client.lua` in a client, and `server.lua` in the server. A plugin MAY hold only one side file, and the other side SHALL then source nothing of it without an error. The `user` directory SHALL have no side file: its init file is `user/init.lua` in a client and `user/server.lua` in the server. A runtimepath entry without a manifest SHALL contribute its modules and colorschemes only; when it holds `client.lua` or `server.lua`, the process SHALL report a plugin error naming the missing manifest and source neither. No file under a `plugin` directory, in any entry, SHALL be sourced.

#### Scenario: Load order
- **WHEN** the runtimepath holds `user`, then the plugins `alpha` and `beta`, each with a valid manifest and a `client.lua`, and each file appends its name to a shared list
- **THEN** the client's list after loading is `init`, `alpha`, `beta`, where `init` is appended by `user/init.lua`

#### Scenario: One file per side
- **WHEN** a plugin holds a valid manifest, a `server.lua` that appends `s` to a window's state key `log`, and a `client.lua` that raises an error
- **THEN** the server sources only `server.lua` and reports no error
- **AND** the client sources only `client.lua` and reports its error

#### Scenario: Client-only plugin
- **WHEN** a plugin holds a valid manifest and only `client.lua`
- **THEN** the server loads it with no error and sources nothing of it

#### Scenario: Legacy plugin files are ignored
- **WHEN** a plugin directory holds `plugin/keys.lua` and `plugin/client/k.lua`, each raising an error
- **THEN** neither process sources them and neither reports an error

#### Scenario: Server plugin files are ignored
- **WHEN** a plugin directory holds `plugin/server/s.lua`, which raises an error
- **THEN** neither process reports an error and `s.lua` is never run

#### Scenario: Default configuration with a plugin
- **WHEN** no `user/init.lua` exists and a plugin's `client.lua` binds `alt+g` with `gband.keymap.set`
- **THEN** the default bindings apply and Alt+G runs the plugin's binding

### Requirement: Plugin errors
Evaluating a plugin's manifest, sourcing a plugin's side file, setting a plugin up and running a callback that belongs to a plugin SHALL each run protected, so that an error raised there SHALL NOT escape to the code around it. Such an error is a plugin error. It SHALL be recorded in the process's log with the plugin's name, the file and the line, and reported as the configuration capability defines.

A plugin error while loading SHALL NOT fail the load. It SHALL mark the plugin failed for the rest of that load: its side file SHALL NOT be sourced when it is not yet, and every callback that belongs to it SHALL be disabled, whether registered before or after the error. A key bound to a disabled callback SHALL do nothing when pressed, a disabled event handler SHALL NOT run, running a disabled command SHALL return `false`, and calling a disabled registered action SHALL do nothing. Options the plugin declared SHALL keep their values.

A plugin error while a callback runs after loading SHALL be reported, and SHALL leave the callback enabled. The actions it dispatched before the error SHALL stand, and other callbacks SHALL be unaffected.

#### Scenario: Setup error
- **WHEN** the plugin `broken` registers the action `broken.go`, binds `alt+b` to it, then raises an error on line 6 of its file in `setup`
- **THEN** the client starts with the configuration's other bindings and options
- **AND** the client shows a plugin error naming `broken`, its file and line 6
- **AND** Alt+B does nothing

#### Scenario: Error in a plugin file
- **WHEN** a plugin's `client.lua` binds `alt+b` on line 1 and raises an error on line 2
- **THEN** the load succeeds, the plugin is failed, and Alt+B does nothing

#### Scenario: Error in a callback
- **WHEN** a plugin's handler for `FocusChanged` raises an error, and a second handler for `FocusChanged` belongs to another plugin
- **THEN** the second handler runs, the error is reported, and the first handler runs again on the next focus change

#### Scenario: Error in a server handler
- **WHEN** a plugin's `server.lua` handler of `WindowOpened` raises an error, and another plugin's handler of `WindowOpened` sets a window state key
- **THEN** the key is set, and the server keeps running its windows

### Requirement: Side guard
Each process's `gband` table SHALL hold only the API of its side. Reading a field of `gband`, or of `gband.action`, that only another side provides SHALL raise an error naming the field and the side that provides it, at the line of the read. The fields only the client provides SHALL be `bind`, `unbind`, `spawn`, `keymap`, `ui`, `hl`, `colorscheme`, `layout`, `view`, `window`, `band`, `win`, `rpc`, `notify`, `bell`, `clipboard`, `open`, and the view and client actions in `gband.action`. The fields only the server provides SHALL be `sessions` and `session`. Every other field the plugins, configuration, lua-events and lua-commands capabilities define SHALL exist on the client and the server, with each side's own behaviour where the server-runtime capability defines one. The test side's `gband` SHALL hold only `side` and `api_version`; reading any field that the client or the server provides there SHALL raise an error naming the field and the sides that provide it.

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
