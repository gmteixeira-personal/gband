## MODIFIED Requirements

### Requirement: Server Lua state
The server process SHALL hold one Lua state of its own, which serves every session the process hosts. It SHALL load it when it starts, after preparing the configuration directory and before it creates its first session. `gband.side` SHALL be the string `"server"` and `gband.api_version` the integer `2` before the server configuration file runs. The server SHALL NOT evaluate `user/init.lua`, the default client configuration, or any plugin's `client.lua`. No Lua state SHALL be shared with a client, and no Lua value SHALL cross between them except as the plugin-bridge capability defines.

#### Scenario: Side in the server
- **WHEN** `user/server.lua` stores `gband.side` and `gband.api_version` in a window's state
- **THEN** a client reads `"server"` and `2` from that state

#### Scenario: Client file not evaluated
- **WHEN** `user/init.lua` raises an error on its first line and a client starts a server
- **THEN** the server log records no error from `user/init.lua`
