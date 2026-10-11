## MODIFIED Requirements

### Requirement: Shared window state
Every window SHALL have a state: a map from non-empty string keys to plain data values, empty when the window opens. `gband.window_state(session, window)` SHALL return a table through which that state is read and written, or nil when the session holds no such window. Reading a key SHALL return a copy of its value. Assigning a plain data value to a key SHALL set it, and assigning nil SHALL remove it. Iterating the table with `pairs` SHALL visit every key. A key that is not a non-empty string, a value that is not plain data, or an assignment that brings the state's encoded size over 64 KiB SHALL be an error at the line of the assignment, and SHALL leave the state unchanged. Assigning a value equal to the current one SHALL change nothing.

The server SHALL publish every change of a window's state to the clients of its session, in the order the changes were made, as the plugin-bridge capability defines. A window's state SHALL be removed when the window leaves the layout. Window states SHALL be kept across a reload of the server configuration. State keys SHALL NOT be namespaced, so that plugins can read each other's keys. The key `agent_state` SHALL be set only by the server's agent detection, as the agent-state capability defines. Reading it SHALL work as for any key, and assigning it from Lua SHALL be an error at the line of the assignment naming `agent_state`, and SHALL leave the state unchanged.

#### Scenario: Set and publish
- **WHEN** a `WindowOutput` handler runs `gband.window_state(ev.session, ev.window).agent = "waiting"`
- **THEN** every client of that session sees `agent` `"waiting"` in that window's state

#### Scenario: Clear
- **WHEN** a `WindowInput` handler sets the window's `agent` to nil
- **THEN** clients see no `agent` key in that window's state

#### Scenario: Function refused
- **WHEN** line 5 of a plugin's `server.lua` handler assigns a function to a state key
- **THEN** a plugin error at that line is reported and the state is unchanged

#### Scenario: Kept across reload
- **WHEN** a window's state holds `agent` `"waiting"` and the user saves `user/server.lua`
- **THEN** after the reload the window's state still holds `agent` `"waiting"`

#### Scenario: Agent state is read-only
- **WHEN** line 7 of a plugin's `server.lua` handler assigns `"x"` to `gband.window_state(ev.session, ev.window).agent_state`
- **THEN** a plugin error at that line names `agent_state`, and the window's state is unchanged

#### Scenario: Agent state is readable
- **WHEN** window 2 runs Claude Code and its state is `idle`, and a server handler reads `gband.window_state(session, 2).agent_state.state`
- **THEN** it reads `idle`
