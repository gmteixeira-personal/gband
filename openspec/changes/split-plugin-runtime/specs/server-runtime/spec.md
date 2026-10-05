## Purpose

Defines the server's own Lua runtime: what it loads, the events it reacts to, the shared state and events it publishes to clients, the commands clients can call in it, and how its errors are contained, so that plugin code that must outlive or span clients runs in the one process that does.

## ADDED Requirements

### Requirement: Server Lua state
The server process SHALL hold one Lua state of its own, which serves every session the process hosts. It SHALL load it when it starts, after preparing the configuration directory and before it creates its first session. `gband.side` SHALL be the string `"server"` and `gband.api_version` the integer `1` before the server configuration file runs. The server SHALL NOT evaluate `user/init.lua`, the default client configuration, or any plugin's `client.lua`. No Lua state SHALL be shared with a client, and no Lua value SHALL cross between them except as the plugin-bridge capability defines.

#### Scenario: Side in the server
- **WHEN** `user/server.lua` stores `gband.side` and `gband.api_version` in a pane's state
- **THEN** a client reads `"server"` and `1` from that state

#### Scenario: Client file not evaluated
- **WHEN** `user/init.lua` raises an error on its first line and a client starts a server
- **THEN** the server log records no error from `user/init.lua`

### Requirement: Server configuration file
The server configuration file SHALL be `user/server.lua` in the configuration directory of the machine the server runs on. Loading SHALL use one new Lua state, starting from the declared defaults of the server options, and SHALL evaluate the server init file and then the `server.lua` files the plugins capability sources from the runtimepath. The server init file SHALL be the server configuration file when it exists, and the default server configuration otherwise. The default server configuration SHALL be built into the executable, SHALL use only the server's `gband` API, and evaluated alone SHALL produce the declared defaults of the server options. A missing server configuration file SHALL NOT be an error. Loading SHALL apply all of the configuration or none of it as the configuration capability's "Last good configuration" defines, with the server init file in place of the init file.

#### Scenario: Width set for the server
- **WHEN** `user/server.lua` sets `gband.opt.default_column_width = 1/3`, and a client starts a server and opens a pane
- **THEN** the new pane's column has width 1/3

#### Scenario: No server configuration file
- **WHEN** no `user/server.lua` exists and a client starts a server
- **THEN** no error is reported and the server options have their declared defaults

#### Scenario: Broken server file at start
- **WHEN** `user/server.lua` sets `default_column_width` to `1/3` and then raises an error, and a client starts a server and opens a pane
- **THEN** the new pane's column has width 1/2
- **AND** the client shows the error prefixed with `server: `

### Requirement: Server events
`gband.on`, `gband.augroup` and their options SHALL behave in the server as the lua-events capability defines, with these built-in events and no `User` event:

| event | payload | emitted when |
|---|---|---|
| `SessionCreated` | `session`: its name | a session is created |
| `SessionEnded` | `session` | a session is removed |
| `PaneOpened` | `session`, `pane`, `band` | a pane enters a session's layout |
| `PaneClosed` | `session`, `pane`, `band`: the band it left | a pane leaves a session's layout |
| `PaneExited` | `session`, `pane`, `code`: the exit code, nil when a signal ended it, `signal`: the signal number, nil otherwise | a pane's program exits |
| `PaneOutput` | `session`, `pane`, `data`: a string holding the bytes read | the pane's program writes output |
| `PaneInput` | `session`, `pane`, `client`: the client's number | a key or paste from a client is written to the pane |
| `ClientAttached` | `session`, `client` | a client attaches to a session |
| `ClientDetached` | `session`, `client` | a client detaches or disconnects |
| `ConfigReloaded` | empty | a reload succeeded, to the handlers of the new configuration |

A session's events SHALL reach the handlers in the order the session applied the changes. The `data` of a pane's `PaneOutput` events, joined in order, SHALL equal the bytes the program wrote, except bytes dropped as "Handlers never slow a session" defines. A chunk's boundaries are arbitrary. `PaneInput` SHALL NOT carry the input itself.

#### Scenario: Agent prompt detected
- **WHEN** a `PaneOutput` handler appends each `data` of pane 1 to a buffer, and pane 1 runs `printf 'Do you want\nto proceed?\n'`
- **THEN** the buffer contains `Do you want\r\nto proceed?`

#### Scenario: Exit then close
- **WHEN** a pane's program exits with status 3
- **THEN** `PaneExited` runs with `code` 3, then `PaneClosed` runs naming the same pane

#### Scenario: Input notice
- **WHEN** a client types `ls` into pane 2
- **THEN** `PaneInput` runs naming pane 2 and that client, and its payload holds no `ls`

#### Scenario: User event refused
- **WHEN** line 1 of `user/server.lua` calls `gband.on("User", fn)`
- **THEN** loading fails with an error at `user/server.lua` line 1 naming `User`

### Requirement: Handlers never slow a session
Handlers SHALL run outside the sessions' own work. No pane's output, input, screen or layout SHALL wait for a handler. When `PaneOutput` data not yet delivered to handlers exceeds 4 MiB, the server SHALL drop further `PaneOutput` data until the handlers catch up, and SHALL record in its log how many bytes it dropped. When other events not yet delivered exceed 1024, the server SHALL drop the oldest and record how many it dropped. When no `PaneOutput` handler is registered, the server SHALL NOT hold pane output for the runtime.

#### Scenario: Slow handler
- **WHEN** a `PaneOutput` handler loops for one second on each call, and a pane writes 50 MiB
- **THEN** the pane's screen on every client follows the output as it would without the handler
- **AND** the server log records dropped `PaneOutput` bytes

### Requirement: Emit events to clients
`gband.emit(name, data, opts)` SHALL send an event named `name`, a non-empty string, with `data`, which SHALL be plain data as the plugin-bridge capability defines, to clients. `opts.session`, optional, SHALL name the only session whose clients receive it; without it every attached client of every session SHALL receive it. Each event SHALL carry the time it was emitted, in milliseconds since the Unix epoch. An invalid name, data, or `opts` field SHALL be an error at the line of the call. `gband.emit` SHALL be callable only in a callback; calling it while the configuration loads SHALL be an error.

When no client that the event targets is attached, the server SHALL queue the event. The queue SHALL hold at most 256 events, dropping the oldest and recording each drop in the log. When a client attaches, the server SHALL send it, after the layout and the snapshots and in the order emitted, every queued event that targets its session or every session, marked as queued, and remove them from the queue. Events SHALL be delivered to each attached client in the order emitted.

#### Scenario: Broadcast
- **WHEN** two clients are attached to `work` and `play`, and a handler calls `gband.emit("agent.waiting", { pane = 1 })`
- **THEN** both clients receive `agent.waiting` with `data.pane` 1

#### Scenario: One session
- **WHEN** clients are attached to `work` and `play` and a handler emits with `{ session = "work" }`
- **THEN** only the client of `work` receives it

#### Scenario: Queued while detached
- **WHEN** no client is attached, a handler emits `a` then `b`, and a client then attaches
- **THEN** the client receives `a` then `b`, each marked as queued, after the layout and snapshots
- **AND** a second client attaching later receives neither

#### Scenario: Queue limit
- **WHEN** no client is attached and handlers emit 300 events
- **THEN** the next client to attach receives the last 256

### Requirement: Shared pane state
Every pane SHALL have a state: a map from non-empty string keys to plain data values, empty when the pane opens. `gband.pane_state(session, pane)` SHALL return a table through which that state is read and written, or nil when the session holds no such pane. Reading a key SHALL return a copy of its value. Assigning a plain data value to a key SHALL set it, and assigning nil SHALL remove it. Iterating the table with `pairs` SHALL visit every key. A key that is not a non-empty string, a value that is not plain data, or an assignment that brings the state's encoded size over 64 KiB SHALL be an error at the line of the assignment, and SHALL leave the state unchanged. Assigning a value equal to the current one SHALL change nothing.

The server SHALL publish every change of a pane's state to the clients of its session, in the order the changes were made, as the plugin-bridge capability defines. A pane's state SHALL be removed when the pane leaves the layout. Pane states SHALL be kept across a reload of the server configuration. State keys SHALL NOT be namespaced, so that plugins can read each other's keys.

#### Scenario: Set and publish
- **WHEN** a `PaneOutput` handler runs `gband.pane_state(ev.session, ev.pane).agent = "waiting"`
- **THEN** every client of that session sees `agent` `"waiting"` in that pane's state

#### Scenario: Clear
- **WHEN** a `PaneInput` handler sets the pane's `agent` to nil
- **THEN** clients see no `agent` key in that pane's state

#### Scenario: Function refused
- **WHEN** line 5 of a plugin's `server.lua` handler assigns a function to a state key
- **THEN** a plugin error at that line is reported and the state is unchanged

#### Scenario: Kept across reload
- **WHEN** a pane's state holds `agent` `"waiting"` and the user saves `user/server.lua`
- **THEN** after the reload the pane's state still holds `agent` `"waiting"`

### Requirement: Server commands
`gband.cmd.register`, `gband.cmd.run` and `gband.cmd.list` SHALL behave in the server as the lua-commands capability defines. A server command's function SHALL be called with two arguments: the arguments table, and a context holding `session`, the calling client's session name or nil, `client`, the calling client's number or nil, and `focus`, a function. `ctx.focus(pane)` SHALL tell the calling client, and no other, to focus that pane of its session, and SHALL do nothing when the command has no calling client. The function's first return value SHALL be the command's result and SHALL be plain data. A result that is not plain data SHALL make the call fail as if the function raised an error.

#### Scenario: Command focuses for its caller
- **WHEN** a server command calls `ctx.focus(3)` for a call from the first of two clients attached to one session
- **THEN** the first client focuses pane 3 and the second client's focus is unchanged

#### Scenario: Local run
- **WHEN** a server handler calls `gband.cmd.run("agents.list")`
- **THEN** the command runs with `ctx.client` nil and `gband.cmd.run` returns `true`

### Requirement: Server session actions
The server's `gband.action` SHALL hold one function for each session action the actions capability names, under its Lua name. Each SHALL take one target table naming `session` and the pane the action acts on as `pane`; `open_pane` SHALL instead take `session`, `band`, an optional `after` pane and an optional `program`, a command line string or a list of argument strings. A missing session, a wrong type or an unknown field SHALL be an error at the line of the call. The actions SHALL be applied to their sessions after the callback returns, in the order called, as the session-server capability's "Session actions" defines, and SHALL tell no client to focus. An action naming a session, pane or band that no longer exists SHALL change nothing. `gband.action` SHALL be callable only in a callback.

#### Scenario: Close from a handler
- **WHEN** a `PaneOutput` handler calls `gband.action.close_pane({ session = ev.session, pane = ev.pane })` on seeing `bye`, and the pane prints `bye`
- **THEN** the pane's program receives SIGHUP and the pane leaves the layout

#### Scenario: Open without stealing focus
- **WHEN** a client focuses pane 1 and a server command opens a pane in band 1 after pane 1 running `htop`
- **THEN** every client receives a layout holding the new pane
- **AND** the client's focus stays on pane 1

### Requirement: Reading structure
`gband.sessions()` SHALL return the names of the sessions in ascending byte order. `gband.session(name)` SHALL return `{ name, bands, clients }` for the session `name`, or nil for an unknown session. `bands` SHALL list its bands from the top, each `{ band, columns }`, each column `{ width, full_width, panes }` from the left, and each pane `{ pane }` from the top. `clients` SHALL list the numbers of the clients attached to it in ascending order. Each call SHALL return new tables.

#### Scenario: Layout of a session
- **WHEN** the session `work` holds one band with two columns, holding panes 1 and 2, and one client is attached
- **THEN** `gband.session("work").bands[1].columns[2].panes[1].pane` is 2
- **AND** `gband.session("work").clients` holds one number

### Requirement: Server reload
The server SHALL watch the `user` directory and every directory beneath it, as the configuration capability's "Reload on change" defines, and load the server configuration again in a new Lua state on a change. When loading succeeds, the server SHALL stop running the old state's handlers and commands, use the new state from then on, and emit `ConfigReloaded` to the new handlers. Pane states, queued events and the sessions SHALL be unchanged by a reload.

#### Scenario: Handler replaced
- **WHEN** `user/server.lua` emits `one` on `PaneOpened`, and the user changes it to emit `two` and a pane opens after the reload
- **THEN** clients receive `two` and not `one`

### Requirement: Server errors reach clients
Configuration errors and plugin errors of the server's Lua SHALL be recorded in the server's log as the configuration capability defines. The server SHALL send each one to every client attached at the time, and SHALL send the latest one to each client that attaches, until the server configuration next loads with no error. A client SHALL report each as an error preceded by `server: `, as the configuration capability defines.

#### Scenario: Handler error shown
- **WHEN** a plugin's `server.lua` handler of `PaneOpened` raises `boom` on line 4 and a client opens a pane
- **THEN** the client shows `server: <plugin>: <path>/server.lua:4: boom`
