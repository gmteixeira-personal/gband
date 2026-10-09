## ADDED Requirements

### Requirement: Forced reload
`gband.action.reload` SHALL make the client load its configuration again in a new Lua state, as "Reload on change" defines for a changed file, and SHALL ask the server to load the server configuration again, as the server-runtime capability's "Server reload" defines for a change. The server SHALL answer the client once its load has ended, after it has sent any error of that load to the clients. Each load SHALL read the files of the plugins directory, and the plugins' files, as they are when it starts, so a change to a plugin's files takes effect.

A load that fails SHALL keep the configuration in use and SHALL be reported as a failed load after a file change is. After the client's load, whether it succeeded or failed, `root` SHALL be the active table. A forced reload SHALL load no other client's configuration.

#### Scenario: Plugin file changed
- **WHEN** a plugin's `client.lua` draws `old` in a bar, the user rewrites it to draw `new`, and then presses Ctrl+Space then `!`
- **THEN** the bar shows `new`

#### Scenario: Server plugin changed
- **WHEN** a plugin's `server.lua` emits `one` on `WindowOpened`, the user changes it to emit `two`, presses Ctrl+Space then `!`, and a window opens
- **THEN** the client receives `two` and not `one`

#### Scenario: Broken configuration
- **WHEN** `user/init.lua` binds `alt+l`, line 1 of a plugin's `client.lua` is changed to raise `bad`, and the user presses Ctrl+Space then `!`
- **THEN** the client shows an error holding `bad`
- **AND** Alt+L still runs its binding

#### Scenario: Back to typing
- **WHEN** the modal key style is in use and the user presses Ctrl+Space, `!`, then types `echo hi` and Enter
- **THEN** `hi` appears in the focused window

#### Scenario: Other clients keep theirs
- **WHEN** two clients are attached, a plugin's `client.lua` is rewritten from drawing `old` to drawing `new`, and the first client reloads with Ctrl+Space then `!`
- **THEN** the first client draws `new` and the second still draws `old`
