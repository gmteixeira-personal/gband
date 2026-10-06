## MODIFIED Requirements

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
