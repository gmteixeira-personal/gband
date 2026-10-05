## MODIFIED Requirements

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
- **WHEN** a server handler of `WindowOpened` sets a window state key that a client status line segment draws, and the runner opens a window and settles
- **THEN** the segment's new text is on the runner's screen as soon as the settle completes
