## MODIFIED Requirements

### Requirement: Driving a case
The handle SHALL provide:

| function | effect |
|---|---|
| `g.keys(keys)` | writes each key of `keys`, key names as the configuration capability defines separated by spaces, to the client's terminal as an xterm terminal sends it; after a key whose bytes end in ESC, it waits until the client has read that key, as the test-channel capability's "Input read" defines, before writing the next |
| `g.type(text)` | writes `text` to the client's terminal as typed characters |
| `g.paste(text)` | writes `text` as a bracketed paste when the client enabled bracketed paste, and as typed characters otherwise |
| `g.run(line)` | types `line` followed by Enter |
| `g.mouse(kind, button, col, row, mods)` | writes a mouse report to the client's terminal as an xterm with SGR mouse encoding sends it: `kind` is `"press"`, `"release"`, `"drag"`, `"move"` or `"scroll"`, `button` is `"left"`, `"middle"` or `"right"`, or for `"scroll"` `"up"`, `"down"`, `"left"` or `"right"`, and nil for `"move"`, `col` and `row` are the terminal cell counted from 0, and `mods` is an optional string of modifiers as a key name writes them, such as `"ctrl+shift"` |
| `g.resize(size)` | resizes the client's terminal to `"<cols>x<rows>"` |
| `g.write(path, contents)` | writes a file relative to the case's configuration directory |
| `g.reload()` | reloads the client's and the server's configuration as the test-channel capability defines, and returns the error message of a failed load, or nil |
| `g.settle()` | waits as the test-channel capability defines |
| `g.wait(predicate, opts)` | calls `predicate` with the screen until it returns a value other than `nil` and `false`, and returns that value |
| `g.wait_text(text, opts)` | waits until the screen's text holds `text` |
| `g.client(chunk, ...)`, `g.server(chunk, ...)` | evaluate `chunk` with the arguments as the test-channel capability defines, and return its results |
| `g.set_time(time)` | changes the frozen instant as the test-channel capability defines |

`g.wait` and `g.wait_text` SHALL give up after `opts.timeout` seconds, 5 unless given, and fail the case with an error that holds the latest screenshot. An invalid argument SHALL be an error at the line of the call.

#### Scenario: Open a window by key
- **WHEN** a case with the default configuration calls `g.keys("ctrl+space enter")` and then `g.settle()`
- **THEN** `g.client("return #gband.layout().bands[1].columns")` returns 2

#### Scenario: Wait for program output
- **WHEN** a case calls `g.run("echo hi")` and then `g.wait_text("hi")`
- **THEN** the wait returns once the window shows `hi`

#### Scenario: Reload after editing a plugin
- **WHEN** a case writes a plugin file through `g.write` that changes the text its bar shows and calls `g.reload()`
- **THEN** `g.reload()` returns nil and a later `g.settle()` shows the new text

#### Scenario: Wait gives up
- **WHEN** a case calls `g.wait_text("never", { timeout = 1 })`
- **THEN** the case fails after about one second
- **AND** the failure holds the screenshot taken when it gave up

#### Scenario: Click a window
- **WHEN** a case with the default configuration has two windows open with the second focused, `col` is a terminal column inside the first window, and the case calls `g.mouse("press", "left", col, 2)`, `g.mouse("release", "left", col, 2)` and `g.settle()`
- **THEN** `g.client("return gband.view().window")` returns the first window's number

#### Scenario: Invalid mouse kind
- **WHEN** a case calls `g.mouse("hover", "left", 0, 0)`
- **THEN** the call raises an error at its line naming `hover`

#### Scenario: Escape then another key
- **WHEN** a case calls `g.keys("escape up")`
- **THEN** the client reads Escape and then Up, not Alt+Up
