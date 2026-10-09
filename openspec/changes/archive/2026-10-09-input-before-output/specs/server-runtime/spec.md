## MODIFIED Requirements

### Requirement: Server events
`gband.on`, `gband.augroup` and their options SHALL behave in the server as the lua-events capability defines, with these built-in events and no `User` event:

| event | payload | emitted when |
|---|---|---|
| `SessionCreated` | `session`: its name | a session is created |
| `SessionEnded` | `session` | a session is removed |
| `WindowOpened` | `session`, `window`, `band` | a window enters a session's layout |
| `WindowClosed` | `session`, `window`, `band`: the band it left | a window leaves a session's layout |
| `WindowExited` | `session`, `window`, `code`: the exit code, nil when a signal ended it, `signal`: the signal number, nil otherwise | a window's program exits |
| `WindowOutput` | `session`, `window`, `data`: a string holding the bytes read | the window's program writes output |
| `WindowInput` | `session`, `window`, `client`: the client's number | a key or paste from a client is written to the window |
| `ClientAttached` | `session`, `client` | a client attaches to a session |
| `ClientDetached` | `session`, `client` | a client detaches or disconnects |
| `ConfigReloaded` | empty | a reload succeeded, to the handlers of the new configuration |

A session's events SHALL reach the handlers in the order the session applied the changes. A key's or paste's `WindowInput` SHALL reach the handlers before any `WindowOutput` holding bytes the window's program wrote after reading that key or paste. The `data` of a window's `WindowOutput` events, joined in order, SHALL equal the bytes the program wrote, except bytes dropped as "Handlers never slow a session" defines. A chunk's boundaries are arbitrary. `WindowInput` SHALL NOT carry the input itself.

#### Scenario: Agent prompt detected
- **WHEN** a `WindowOutput` handler appends each `data` of window 1 to a buffer, and window 1 runs `printf 'Do you want\nto proceed?\n'`
- **THEN** the buffer contains `Do you want\r\nto proceed?`

#### Scenario: Exit then close
- **WHEN** a window's program exits with status 3
- **THEN** `WindowExited` runs with `code` 3, then `WindowClosed` runs naming the same window

#### Scenario: Input notice
- **WHEN** a client types `ls` into window 2
- **THEN** `WindowInput` runs naming window 2 and that client, and its payload holds no `ls`

#### Scenario: Input before the output it causes
- **WHEN** a `WindowInput` handler counts its calls for window 1, a `WindowOutput` handler records that count when its `data` first holds `input`, and a client pastes `printf 'in%s\n' put` into window 1 and presses Enter
- **THEN** the recorded count is 2

#### Scenario: User event refused
- **WHEN** line 1 of `user/server.lua` calls `gband.on("User", fn)`
- **THEN** loading fails with an error at `user/server.lua` line 1 naming `User`
