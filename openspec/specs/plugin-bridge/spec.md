# plugin-bridge Specification

## Purpose

Defines what passes between the server's Lua and each client's Lua: the plain-data contract, server events and shared window state as the client sees them, commands a client calls in the server, and plugin requirements the server advertises, with no code ever crossing the connection.

## Requirements

### Requirement: Plain data
A plain data value SHALL be nil, a boolean, an integer, a float, a string, or a table whose keys are strings or integers and whose values are plain data other than nil. A table SHALL NOT hold itself, directly or through other tables, and SHALL be nested at most 32 tables deep. Functions, userdata, threads, metatables and light userdata SHALL NOT be plain data; a table's metatable SHALL be ignored, and its raw contents used. A string SHALL be passed as its bytes, unchanged. A table SHALL keep every key and value, so the receiving side reads a table equal to the one sent, except that integer keys and string keys holding the same digits stay distinct. One value SHALL encode to at most 1 MiB.

Every API that takes plain data SHALL check it before sending anything. A value that is not plain data SHALL be an error at the line of the call, naming the path to the offending value, such as `data.window.fn`. Every plain data value a side receives SHALL be a new table or value, unconnected to any other.

#### Scenario: Nested data round trip
- **WHEN** a server handler emits `{ title = "a", n = 2.5, tags = { "x", "y" }, ok = true }`
- **THEN** the client handler's `data` equals that table, key for key

#### Scenario: Function inside a table
- **WHEN** line 7 of a plugin's `server.lua` emits `{ cb = { run = function() end } }`
- **THEN** a plugin error at that line names `data.cb.run`
- **AND** no client receives the event

#### Scenario: Cycle
- **WHEN** a server handler emits a table `t` with `t.self = t`
- **THEN** the call raises an error naming `data.self`

#### Scenario: Binary string
- **WHEN** a server handler emits the string `"\0\255"`
- **THEN** the client receives the two bytes 0 and 255

### Requirement: Server events in the client
The client SHALL deliver each event the server sends it as the built-in event `ServerEvent`, with the payload `{ name, data, queued, time }`: the name and data given to `gband.emit` in the server, `queued` `true` when the event waited in the server's queue and `false` otherwise, and `time`, the emission time in milliseconds since the Unix epoch. A `ServerEvent` handler registered with `pattern` SHALL run only for events of that name. Events SHALL be delivered in the order the client receives them.

#### Scenario: Notify on a waiting agent
- **WHEN** a client's `client.lua` calls `gband.on("ServerEvent", fn, { pattern = "agent.waiting" })` and the server emits `agent.waiting` with `{ title = "build" }`
- **THEN** `fn` runs once with `data.title` `"build"` and `queued` `false`

#### Scenario: Pattern filters
- **WHEN** the server emits `agent.done` and the only `ServerEvent` handler has the pattern `agent.waiting`
- **THEN** no handler runs

### Requirement: Window state in the client
After the layout and the snapshots of an attach, and before any queued event, the server SHALL send the client the state of every window of its session that has a non-empty state. It SHALL then send every change to the state of a window of the client's session, in the order made. `gband.window_state(window)` in the client SHALL return a new table holding a copy of that window's latest state as the client received it, an empty table for a window of the layout with no state, and nil for a window not in the client's layout. Changing the returned table SHALL change nothing else.

Each change received after the initial states SHALL emit `WindowStateChanged` with `window`, `key`, `value`, the new value or nil when removed, and `previous`, the value before. The initial states SHALL emit no event. A window's state SHALL be dropped from the client when the window leaves its layout.

#### Scenario: State at attach
- **WHEN** window 2's state holds `agent` `"waiting"` and a client attaches
- **THEN** `gband.window_state(2).agent` reads `"waiting"` in the client's `Attached` handler
- **AND** no `WindowStateChanged` runs

#### Scenario: Change event
- **WHEN** a client is attached and a server handler sets window 2's `agent` from `"waiting"` to nil
- **THEN** `WindowStateChanged` runs with `window` 2, `key` `"agent"`, `value` nil and `previous` `"waiting"`

#### Scenario: Read-only copy
- **WHEN** client code sets `gband.window_state(2).agent = "x"`
- **THEN** a later `gband.window_state(2).agent` still reads the server's value

### Requirement: Command calls
`gband.rpc(name, args, callback)` in the client SHALL ask the server to run the server command whose full name is `name` with `args`, plain data that SHALL be a table or nil. `callback`, optional, SHALL be a function. `gband.rpc` SHALL return at once, before the command runs. The server SHALL run the commands each client calls in the order it calls them, with the calling client in the context, as the server-runtime capability defines. When the command returns, the client SHALL call `callback` with `true` and the result. When the name is unknown, the command is disabled, or it raises an error or returns a result that is not plain data, the client SHALL call `callback` with `false` and a message naming the reason. The callback SHALL run as a callback that belongs to the plugin that called `gband.rpc`. A call whose connection closes before its answer SHALL never call `callback`. An invalid argument SHALL be an error at the line of the call. `gband.rpc` SHALL be callable only in a callback.

#### Scenario: Jump to the next waiting agent
- **WHEN** a client binding calls `gband.rpc("agents.next_waiting", {}, cb)`, and the server command calls `ctx.focus(4)` and returns `4`
- **THEN** the client focuses window 4
- **AND** `cb` runs with `true` and `4`

#### Scenario: Unknown command
- **WHEN** a client calls `gband.rpc("absent", nil, cb)`
- **THEN** `cb` runs with `false` and a message naming `absent`

#### Scenario: Failing command
- **WHEN** a server command raises `boom`
- **THEN** the calling client's callback runs with `false` and a message containing `boom`
- **AND** the server log records the error

### Requirement: Plugin requirements
A plugin manifest MAY hold `client`, a requirement on the version of the plugin of the same name in the client. A requirement SHALL be one or more comparisons separated by commas, each one of `>=`, `>`, `<=`, `<` or `=` followed by a version. A version SHALL be one to three non-negative integers separated by full stops, with missing parts read as 0, and SHALL be compared part by part.

After the layout and the snapshots of an attach, the server SHALL send the client the name and requirement of every plugin whose manifest it loaded with a `client` field and whose `server.lua` did not fail. The client SHALL compare each with the manifests it loaded. For each required plugin it did not load, and each whose version does not meet the requirement, the client SHALL report a configuration error naming the plugin, the requirement and the version it holds, if any. The session SHALL work as it does without the check.

#### Scenario: Missing client side
- **WHEN** the server loaded `agent-status` whose manifest holds `client = ">= 0.1"`, and the client has no plugin `agent-status`
- **THEN** the client reports an error naming `agent-status` and `>= 0.1`

#### Scenario: Old client side
- **WHEN** the requirement is `">= 0.2, < 1"` and the client loaded `agent-status` version `0.1.4`
- **THEN** the client reports an error naming `agent-status`, the requirement and `0.1.4`

#### Scenario: Met
- **WHEN** the requirement is `">= 0.1"` and the client loaded version `0.1`
- **THEN** no error is reported

### Requirement: No code crosses the connection
No message between client and server SHALL carry Lua code, a function or bytecode. A client SHALL evaluate only Lua from its own machine: the files the plugins capability defines, and the chunks a test runner sends through a test channel that the client's own environment named, as the test-channel capability defines. A client SHALL treat every value it receives from a server as data. Attaching to a server SHALL NOT cause a client to load, evaluate or call anything the server sent. A server SHALL NOT forward a test channel chunk to a client.

#### Scenario: Code as data
- **WHEN** a server emits the string `os.exit(1)` and a client handler stores it
- **THEN** the client keeps running and holds the string unchanged

#### Scenario: Chunk from a local runner
- **WHEN** a runner starts a client with `GBAND_TEST_SOCKET` naming its socket and sends it `return gband.side`
- **THEN** the client evaluates the chunk and answers `"client"`

#### Scenario: Server sends no chunk
- **WHEN** a runner holds a test channel to the server only, and a client attaches without `GBAND_TEST_SOCKET`
- **THEN** the client evaluates no chunk the runner sends the server
