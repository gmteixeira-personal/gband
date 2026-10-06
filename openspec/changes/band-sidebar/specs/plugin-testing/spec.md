## MODIFIED Requirements

### Requirement: Case environment
`g.start(opts)` SHALL start the case's gband. It SHALL be called once, before any other function of the handle; calling it again, or calling another function first, SHALL be an error. `opts` fields, all optional:

| field | meaning | default |
|---|---|---|
| `size` | the terminal size, `"<cols>x<rows>"` | `"80x24"` |
| `config` | the contents of `user/init.lua` | no file: the default configuration applies |
| `server_config` | the contents of `user/server.lua` | no file: the default server configuration applies |
| `keystyle` | the key style saved in `user/keystyle.lua`, as the key-style capability defines: `"modal"` or `"direct"`, or `false` to save none | `"modal"` |
| `files` | a table from paths relative to the configuration directory to file contents | none |
| `plugins` | a list of paths of further plugins for this case | none |
| `env` | a table of environment variables to set, or to remove with `false` | none |
| `time` | the instant the test-channel capability freezes, as Unix seconds or `"YYYY-MM-DD HH:MM:SS"` in UTC, or `false` for the real time | `"2025-01-01 12:00:00"` |

Each case SHALL run in a new directory tree of its own under the system's temporary directory, holding the configuration, data, state and runtime directories of the case and a working directory. The runner SHALL save the `keystyle` style in the case's configuration directory before it writes `files`, so a case with the default configuration starts with no key style chooser open, and a `files` entry for `user/keystyle.lua` replaces it. The plugin under test, every `--plugin` path and every path in `plugins` SHALL be linked into the case's plugins directory under its last path component. The directory first in `PATH` SHALL hold the programs `xdg-open` and `open`, which record their argument for `g.opened()` and open nothing. The case's gband SHALL run with an environment holding `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME` and `XDG_RUNTIME_DIR` set to the tree, `SHELL=/bin/sh`, `PS1` set to `$ `, `TERM=xterm-256color`, `COLORTERM=truecolor`, `TZ=UTC`, `GBAND_ANIMATIONS=off`, `INPUTRC=/dev/null`, `PATH` with a directory of the tree first, and `GBAND_TEST_SOCKET` set as the test-channel capability defines, and without `GBAND`, `GBAND_SESSION`, `GBAND_WINDOW` and `GBAND_LOG`, then changed by `env`. `g.start` SHALL run `gband attach` from the executable running the test, in the working directory of the tree, in a new PTY of the size given, and SHALL return once the client has drawn its first frame with the configuration loaded.

When the case ends, whether it passed or not, the runner SHALL stop the case's client and server and every process they started, and remove the tree.

#### Scenario: Configuration of the case
- **WHEN** a case starts with `config = [[gband.plugin("gband.sidebar", { side = "right" })]]`
- **THEN** the case's sidebar is drawn on the terminal's last column
- **AND** the user's own `~/.config/gband/user/init.lua` is not read

#### Scenario: Default configuration without the chooser
- **WHEN** a case starts with no `config` and no `keystyle`
- **THEN** no floating plugin window is open
- **AND** `g.client("return gband.keystyle.saved()")` returns `modal`

#### Scenario: First start in a case
- **WHEN** a case starts with `keystyle = false` and no `config`, and calls `g.settle()`
- **THEN** the key style chooser is open and focused

#### Scenario: Direct style in a case
- **WHEN** a case starts with `keystyle = "direct"` and no `config`
- **THEN** `g.client('return gband.keymap.label("prefix")')` returns `prefix`

#### Scenario: Plugin under test is installed
- **WHEN** `gband test` runs in the plugin directory `window` and a case starts with `config = [[gband.plugin("window")]]`
- **THEN** the plugin's bar is drawn, read from the working directory's files

#### Scenario: Nothing left behind
- **WHEN** a case opens three windows and then fails
- **THEN** after the run, none of the case's processes is running and its directory tree does not exist

#### Scenario: Run from inside a window
- **WHEN** the user runs `gband test` from a window of their own gband server
- **THEN** each case's client starts its own server in the case's tree instead of refusing to nest

### Requirement: Driving a case
The handle SHALL provide:

| function | effect |
|---|---|
| `g.keys(keys)` | writes each key of `keys`, key names as the configuration capability defines separated by spaces, to the client's terminal as an xterm terminal sends it |
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
- **WHEN** a case with the default configuration has two windows open with the second focused and calls `g.mouse("press", "left", 2, 2)`, `g.mouse("release", "left", 2, 2)` and `g.settle()`
- **THEN** `g.client("return gband.view().window")` returns the first window's number

#### Scenario: Invalid mouse kind
- **WHEN** a case calls `g.mouse("hover", "left", 0, 0)`
- **THEN** the call raises an error at its line naming `hover`

### Requirement: Observing a case
`g.screen()` SHALL return a new table describing the client's terminal as the runner's emulator holds it: `cols`, `rows`, `cursor` holding `row`, `col` and `visible`, `row(n)`, a function returning row `n` as text, `text()`, a function returning every row joined with newlines, and `cell(row, col)`, a function returning `{ char, fg, bg, bold, dim, italic, underline, inverse }` for one cell. Rows and columns SHALL be counted from 0, top left. Text SHALL omit trailing spaces. A colour SHALL be a string `#rrggbb` for a direct colour, an integer 0 to 255 for a palette colour, and nil for the default colour.

The handle SHALL also return, as new lists, everything the client wrote to its terminal since `g.start`:

| function | each entry |
|---|---|
| `g.notifications()` | `{ title, body }` for a desktop notification sent with OSC 9, which carries no title, or OSC 777 |
| `g.clipboard()` | the text of a clipboard write sent with OSC 52 |
| `g.opened()` | the argument of each run of the opener, as "Case environment" defines |

`g.bells()` SHALL return the number of bells the client wrote since `g.start`.

`g.log(side)` SHALL return the lines the case's `"client"` or `"server"` process has written to its log so far.

#### Scenario: Status line colour
- **WHEN** a case with the default configuration loads a colorscheme that gives `SidebarMode` the foreground `#7aa2f7`
- **THEN** `g.screen().cell(0, 0).char` is `I` and its `fg` is `"#7aa2f7"`

#### Scenario: Notification observed
- **WHEN** a case's configuration sets `notify_style` to `"osc777"` and a plugin's client code calls `gband.notify("build done", { title = "ci" })`
- **THEN** `g.notifications()` holds one entry with `title` `ci` and `body` `build done`

#### Scenario: Opener recorded
- **WHEN** a plugin's binding calls `gband.open("https://example.com")`
- **THEN** `g.opened()` holds `https://example.com`
- **AND** no browser starts

#### Scenario: Print observed
- **WHEN** a plugin's `client.lua` calls `print("ready")`
- **THEN** a line of `g.log("client")` holds `ready`
