# test-channel Specification

## Purpose

Defines the local channel through which a test runner reaches into the client and the server it launched: evaluating Lua in their states, waiting until they have settled, reloading them and freezing the time their Lua reads, without changing the connection between client and server.

## Requirements

### Requirement: Test socket
When the environment variable `GBAND_TEST_SOCKET` is set and not empty, `gband attach` and `gband server` SHALL connect to the Unix socket at the path it holds before loading their configuration, and SHALL tell the runner whether they are the client or the server. A process SHALL accept the channel only when the process listening on the socket runs as the same user. When the connection fails or the listener runs as another user, the process SHALL print one line naming the path and the reason to standard error, record it in its log, and exit with status 1 without loading its configuration. When the variable is unset or empty, no process SHALL connect to a channel, and gband SHALL behave as it does without this capability. Other subcommands SHALL ignore the variable.

A server that `gband attach` starts inherits the client's environment, as the client-attach capability defines, so it SHALL connect to the same channel as its client.

#### Scenario: Both sides join
- **WHEN** a runner listens on a socket and starts `gband attach` with `GBAND_TEST_SOCKET` naming it, and no server is running
- **THEN** the runner receives one connection from the client and one from the server it started

#### Scenario: Missing socket
- **WHEN** `gband attach` runs with `GBAND_TEST_SOCKET=/nonexistent/sock`
- **THEN** standard error holds one line naming `/nonexistent/sock`
- **AND** the process exits with status 1

#### Scenario: Unset
- **WHEN** `GBAND_TEST_SOCKET` is empty and the user runs `gband attach`
- **THEN** the client attaches as it does without the variable

### Requirement: Channel lifetime
A process connected to a test channel SHALL end when the channel closes: the client SHALL leave as when the user detaches, and the server SHALL stop as it does on SIGTERM, as the session-server capability defines.

#### Scenario: Runner killed
- **WHEN** the runner holding a channel is killed while its client and server run
- **THEN** the client and the server exit, and the server's window programs receive SIGHUP

### Requirement: Evaluate a chunk
The runner SHALL be able to send a process a chunk, a string of Lua source, with a list of plain data arguments as the plugin-bridge capability defines. The process SHALL run the chunk in its current Lua state with the arguments as `...`, as a callback that belongs to no plugin, between its other work and never while another callback runs, in the order the runner sent the chunks. A chunk SHALL be able to do anything a callback of its side can, and the actions it dispatches SHALL take effect after it returns, as for any callback. The instruction limit SHALL apply to it as to a callback.

When the chunk returns, the process SHALL answer with its return values, each of which SHALL be plain data. When the chunk fails to compile, raises an error, hits the instruction limit, or returns a value that is not plain data, the process SHALL answer with a message naming the reason, and SHALL keep running. A chunk's error SHALL NOT be reported as a configuration or plugin error.

#### Scenario: Read client state
- **WHEN** the runner sends the client `return gband.side, select("#", ...)` with the arguments `1` and `2`
- **THEN** the answer is `"client"` and `2`

#### Scenario: Act in the server
- **WHEN** the runner sends the server a chunk that sets window 1's state key `agent` to `"waiting"` in the session `default`
- **THEN** the client's `gband.window_state(1).agent` reads `"waiting"` after a settle

#### Scenario: Error answered
- **WHEN** the runner sends the client `error("boom")`
- **THEN** the answer is a failure whose message holds `boom`
- **AND** the client shows no error and keeps running

#### Scenario: Function result refused
- **WHEN** the runner sends the client `return function() end`
- **THEN** the answer is a failure naming a value that is not plain data

### Requirement: Settle
The runner SHALL be able to ask the client and the server to settle. A settle SHALL complete once:

1. the client has read and handled all terminal input the runner wrote before asking,
2. the server has handled every message the client sent before that point, applied the actions they asked for, delivered the resulting events to its Lua handlers, and sent the resulting changes to the client,
3. the client has handled every message the server sent it before that point, delivered the resulting events to its Lua handlers, finished any running animation, and drawn a frame showing the result,
4. the runner's emulator has read every byte the client wrote to its terminal before that frame ended.

The client and the server SHALL repeat these steps for the effects of the effects until one round changes nothing, up to ten rounds. A settle SHALL NOT wait for output that programs in windows write in response. A settle that does not complete within 10 seconds SHALL fail with a message naming the step that did not finish.

#### Scenario: Key effect is drawn
- **WHEN** the runner writes the key that opens a window and then settles
- **THEN** the runner's screen shows two tiles as soon as the settle completes

#### Scenario: Server handler effect is drawn
- **WHEN** a server handler of `WindowOpened` sets a window state key, a client handler of `WindowStateChanged` writes the key's value into a bar with `gband.bar.set_lines`, and the runner opens a window and settles
- **THEN** the bar's new text is on the runner's screen as soon as the settle completes

### Requirement: Reload request
The runner SHALL be able to ask the client and the server to reload their configuration. Each SHALL load it again as the configuration capability's "Reload on change" defines, as if a file under `user/` had changed, and answer once the load has finished, with the error message when it failed. A reload request SHALL also re-read the plugins directory, so that changes to a plugin's files take effect.

#### Scenario: Plugin file changed
- **WHEN** a plugin's `client.lua` is rewritten to draw `new` instead of `old`, and the runner asks for a reload and settles
- **THEN** the client draws `new`

#### Scenario: Broken file
- **WHEN** `user/init.lua` raises `bad` on line 1 and the runner asks for a reload
- **THEN** the client's answer is a failure whose message holds `bad`
- **AND** the client keeps the configuration it last loaded

### Requirement: Frozen time
Before a process loads its configuration, the runner SHALL tell it either an instant, as Unix seconds, or that no instant is set. While an instant is set, in every Lua state the process creates, `os.time()` called without a table SHALL return the instant, and `os.date(format)` called without a time SHALL format the instant. Calls with an explicit time or table SHALL behave as Lua defines. The instant SHALL NOT advance by itself. The runner SHALL be able to set a new instant at any time, which SHALL take effect for every later call in that process. Frozen time SHALL NOT change when timers fire. Without an instant, `os.time` and `os.date` SHALL behave as Lua defines.

#### Scenario: Clock segment frozen
- **WHEN** the instant is `2025-01-01 12:00:00` UTC, `TZ` is `UTC` and the client's `user/init.lua` adds a bar whose line is `os.date("%H:%M")`
- **THEN** the bar draws `12:00` on every run

#### Scenario: Time moved
- **WHEN** the runner sets the instant to `2025-01-01 12:05:00` UTC and then a binding function sets the bar's line to `os.date("%H:%M")` with `gband.bar.set_lines`
- **THEN** the bar draws `12:05`

#### Scenario: Explicit time unchanged
- **WHEN** an instant is set and a chunk returns `os.date("%Y", 0)`
- **THEN** the answer is `"1970"`
