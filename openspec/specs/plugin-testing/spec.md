# plugin-testing Specification

## Purpose

Defines `gband test`, which runs Lua test files against real gband clients and servers in isolated environments, so that plugin authors and agents can drive a plugin, see what it draws and keep the result as a regression test.

## Requirements

### Requirement: Test subcommand
`gband test` SHALL run Lua test files and report their results. Its arguments SHALL be:

| argument | effect |
|---|---|
| none | run every file whose name ends in `_spec.lua` under the `tests` directory of the working directory, at any depth, in ascending byte order of their paths |
| `FILE...` | run the named files, in the order given |
| `-` | read standard input to its end and run it as one test file named `stdin` |
| `--update` | write every screenshot reference the run compares, as "Screenshot references" defines |
| `--show` | print every screenshot the run takes, as "Report" defines |
| `--filter PATTERN` | run only the cases whose name holds `PATTERN` as plain text |
| `--plugin PATH`, repeatable | install the plugin at `PATH` in every case, as "Case environment" defines |

When the working directory holds a file `plugin.lua`, it SHALL be the plugin under test. `-` SHALL NOT be combined with a `FILE`. A file that does not exist, no test file found, or two plugins with the same last path component SHALL be reported in one line on standard error and end the run with exit status 2 before any case runs. `gband test` SHALL NOT need a terminal, SHALL NOT write a log file of its own, and SHALL NOT connect to, start or stop any server the user runs.

The exit status SHALL be 0 when every case that ran passed, 1 when a case failed or a test file failed outside its cases, and 2 for an invalid invocation.

#### Scenario: Plugin tests found
- **WHEN** a plugin directory holds `plugin.lua`, `tests/a_spec.lua`, `tests/ui/b_spec.lua` and `tests/helpers.lua`, and the user runs `gband test` in it
- **THEN** `tests/a_spec.lua` runs, then `tests/ui/b_spec.lua`
- **AND** `tests/helpers.lua` does not run as a test file

#### Scenario: Chunk from standard input
- **WHEN** the user runs `gband test -` with one case on standard input that passes
- **THEN** the report names the file `stdin` and the case, and the process exits with status 0

#### Scenario: No tests
- **WHEN** the user runs `gband test` in a directory with no `tests` directory
- **THEN** standard error holds one line saying that no test file was found
- **AND** the process exits with status 2

#### Scenario: User's server untouched
- **WHEN** the user's default server runs one session and the user runs `gband test` on a plugin whose cases open panes
- **THEN** the user's server still holds one session with its panes unchanged

### Requirement: Test files and cases
Each test file SHALL run in a new Lua state of the test side, as the plugins capability defines. `require("gband.test")` SHALL return the test module, and `require` SHALL also find modules in the test file's own directory. `t.case(name, fn)` or `t.case(name, opts, fn)` SHALL register a case: `name`, a non-empty string unique in its file, and `fn`, a function. A duplicate or invalid case SHALL be an error at the line of the call.

After the file's top-level code returns, its cases SHALL run one at a time in the order registered. Each case SHALL get a case environment of its own and a handle, and `fn` SHALL be called with the handle. A case SHALL pass when `fn` returns, and SHALL fail when `fn` raises an error or the case runs longer than its time limit, 30 seconds unless `opts.timeout` gives another number of seconds. When the file's top-level code raises an error or runs longer than 30 seconds, the file SHALL fail and none of its cases SHALL run. When `--filter` is given, a case whose name does not hold the pattern SHALL not run and SHALL not be reported.

#### Scenario: Cases run in order
- **WHEN** a test file registers the cases `one` and `two`, each printing its name
- **THEN** standard output shows `one` before `two`

#### Scenario: Failing case does not stop the file
- **WHEN** the first of two cases raises an error and the second passes
- **THEN** the report shows the first as failed and the second as passed
- **AND** the process exits with status 1

#### Scenario: Endless case
- **WHEN** a case with `{ timeout = 2 }` runs `while true do end`
- **THEN** the case fails after about 2 seconds with a message naming the time limit
- **AND** the next case runs

#### Scenario: Filter
- **WHEN** the user runs `gband test --filter colour` on a file with the cases `colour`, `colour two` and `width`
- **THEN** only `colour` and `colour two` run and appear in the report

### Requirement: Assertions
The test module SHALL provide:

| function | fails when |
|---|---|
| `t.eq(actual, expected, message)` | `actual` and `expected` differ |
| `t.ok(value, message)` | `value` is `nil` or `false` |
| `t.match(text, pattern, message)` | `text` is not a string or the Lua pattern `pattern` does not match it |

Two values SHALL be equal when they are equal as Lua values, or when both are tables with the same keys and equal values at every key, compared again this way. A failing assertion SHALL raise an error at the line of its call, naming the expected and the actual value, and `message` when given.

#### Scenario: Deep equality
- **WHEN** a case calls `t.eq({ a = { 1, 2 } }, { a = { 1, 2 } })`
- **THEN** the assertion passes

#### Scenario: Failure names both values
- **WHEN** line 9 of a test file calls `t.eq(3, 4)`
- **THEN** the case fails with an error at line 9 naming `3` and `4`

### Requirement: Case environment
`g.start(opts)` SHALL start the case's gband. It SHALL be called once, before any other function of the handle; calling it again, or calling another function first, SHALL be an error. `opts` fields, all optional:

| field | meaning | default |
|---|---|---|
| `size` | the terminal size, `"<cols>x<rows>"` | `"80x24"` |
| `config` | the contents of `user/init.lua` | no file: the default configuration applies |
| `server_config` | the contents of `user/server.lua` | no file: the default server configuration applies |
| `files` | a table from paths relative to the configuration directory to file contents | none |
| `plugins` | a list of paths of further plugins for this case | none |
| `env` | a table of environment variables to set, or to remove with `false` | none |
| `time` | the instant the test-channel capability freezes, as Unix seconds or `"YYYY-MM-DD HH:MM:SS"` in UTC, or `false` for the real time | `"2025-01-01 12:00:00"` |

Each case SHALL run in a new directory tree of its own under the system's temporary directory, holding the configuration, data, state and runtime directories of the case and a working directory. The plugin under test, every `--plugin` path and every path in `plugins` SHALL be linked into the case's plugins directory under its last path component. The directory first in `PATH` SHALL hold the programs `xdg-open` and `open`, which record their argument for `g.opened()` and open nothing. The case's gband SHALL run with an environment holding `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME` and `XDG_RUNTIME_DIR` set to the tree, `SHELL=/bin/sh`, `PS1` set to `$ `, `TERM=xterm-256color`, `COLORTERM=truecolor`, `TZ=UTC`, `GBAND_ANIMATIONS=off`, `INPUTRC=/dev/null`, `PATH` with a directory of the tree first, and `GBAND_TEST_SOCKET` set as the test-channel capability defines, and without `GBAND`, `GBAND_SESSION`, `GBAND_PANE` and `GBAND_LOG`, then changed by `env`. `g.start` SHALL run `gband attach` from the executable running the test, in the working directory of the tree, in a new PTY of the size given, and SHALL return once the client has drawn its first frame with the configuration loaded.

When the case ends, whether it passed or not, the runner SHALL stop the case's client and server and every process they started, and remove the tree.

#### Scenario: Configuration of the case
- **WHEN** a case starts with `config = [[gband.opt.statusline_position = "top"]]`
- **THEN** the case's status line is drawn on the top row
- **AND** the user's own `~/.config/gband/user/init.lua` is not read

#### Scenario: Plugin under test is installed
- **WHEN** `gband test` runs in the plugin directory `pane` and a case starts with `config = [[gband.plugin("pane")]]`
- **THEN** the plugin's segment is drawn, read from the working directory's files

#### Scenario: Nothing left behind
- **WHEN** a case opens three panes and then fails
- **THEN** after the run, none of the case's processes is running and its directory tree does not exist

#### Scenario: Run from inside a pane
- **WHEN** the user runs `gband test` from a pane of their own gband server
- **THEN** each case's client starts its own server in the case's tree instead of refusing to nest

### Requirement: Driving a case
The handle SHALL provide:

| function | effect |
|---|---|
| `g.keys(keys)` | writes each key of `keys`, key names as the configuration capability defines separated by spaces, to the client's terminal as an xterm terminal sends it |
| `g.type(text)` | writes `text` to the client's terminal as typed characters |
| `g.paste(text)` | writes `text` as a bracketed paste when the client enabled bracketed paste, and as typed characters otherwise |
| `g.run(line)` | types `line` followed by Enter |
| `g.resize(size)` | resizes the client's terminal to `"<cols>x<rows>"` |
| `g.write(path, contents)` | writes a file relative to the case's configuration directory |
| `g.reload()` | reloads the client's and the server's configuration as the test-channel capability defines, and returns the error message of a failed load, or nil |
| `g.settle()` | waits as the test-channel capability defines |
| `g.wait(predicate, opts)` | calls `predicate` with the screen until it returns a value other than `nil` and `false`, and returns that value |
| `g.wait_text(text, opts)` | waits until the screen's text holds `text` |
| `g.client(chunk, ...)`, `g.server(chunk, ...)` | evaluate `chunk` with the arguments as the test-channel capability defines, and return its results |
| `g.set_time(time)` | changes the frozen instant as the test-channel capability defines |

`g.wait` and `g.wait_text` SHALL give up after `opts.timeout` seconds, 5 unless given, and fail the case with an error that holds the latest screenshot. An invalid argument SHALL be an error at the line of the call.

#### Scenario: Open a pane by key
- **WHEN** a case with the default configuration calls `g.keys("ctrl+space enter")` and then `g.settle()`
- **THEN** `g.client("return #gband.layout().bands[1].columns")` returns 2

#### Scenario: Wait for program output
- **WHEN** a case calls `g.run("echo hi")` and then `g.wait_text("hi")`
- **THEN** the wait returns once the pane shows `hi`

#### Scenario: Reload after editing a plugin
- **WHEN** a case writes a plugin file through `g.write` that changes its segment's text and calls `g.reload()`
- **THEN** `g.reload()` returns nil and a later `g.settle()` shows the new text

#### Scenario: Wait gives up
- **WHEN** a case calls `g.wait_text("never", { timeout = 1 })`
- **THEN** the case fails after about one second
- **AND** the failure holds the screenshot taken when it gave up

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
- **WHEN** a case loads a colorscheme that gives `StatusLine` the foreground `#c0caf5`
- **THEN** `g.screen().cell(23, 0).fg` is `"#c0caf5"`

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

### Requirement: Screenshots
`g.screenshot(opts)` SHALL return the screen as text, in this form:

```
size 40x6 cursor 1:2 shown
 0|┌──────────────────┐
 1|│$                 │
 5|band 1 │ prefix
--
 0:0-19 bold
 5:0-5 fg=#c0caf5 bg=#1a1b26 bold
```

The first line SHALL be `size <cols>x<rows> cursor <row>:<col>` followed by `shown` or `hidden`. One line SHALL follow for every row, from the top: the row number right-aligned to the width of the largest row number, `|`, and the row's text without trailing spaces. A wide character SHALL appear once. Unless `opts.styles` is `false`, a line `--` SHALL follow, then one line for every run of cells on one row that share a style other than the default style, in the order of rows and then columns. A run SHALL be as long as possible. Its line SHALL be `<row>:<first>-<last>`, or `<row>:<col>` for a single cell, then `fg=` and `bg=` for each colour that is not the default, written as `#rrggbb` or the palette index, then each of `bold`, `dim`, `italic`, `underline` and `inverse` that applies, in that order, separated by single spaces. The default style SHALL have default colours and no attribute.

#### Scenario: Plain screen
- **WHEN** a client draws only unstyled text
- **THEN** the screenshot holds no line after `--`

#### Scenario: Text only
- **WHEN** a case calls `g.screenshot({ styles = false })`
- **THEN** the screenshot holds the header and the rows, and no `--` line

#### Scenario: Run of one colour
- **WHEN** row 5 shows `pane 1` in columns 25 to 30 with the foreground `#7aa2f7` and the background `#1a1b26`, between cells of another style
- **THEN** the screenshot holds the line `5:25-30 fg=#7aa2f7 bg=#1a1b26`

### Requirement: Screenshot references
`g.expect_screenshot(name, opts)` SHALL take a screenshot with `opts` and compare it with its reference file. `name` is optional. The reference SHALL be `screenshots/<file>/<case>.txt` in the test file's directory, where `<file>` is the test file's name without `.lua`, and `<case>` is the case's name, followed by `--` and `name` when given, lowercased, with each run of characters other than ASCII letters and digits replaced by one `-` and leading and trailing `-` removed. Two screenshots of one run with the same reference SHALL be an error.

Without `--update`, a reference that matches SHALL pass. A missing reference, or one that differs, SHALL fail the case, and the runner SHALL write the screenshot beside the reference with the suffix `.new` added. With `--update`, the runner SHALL write the screenshot as the reference, and remove any `.new` file beside it. A case that passes SHALL leave no `.new` file for its references. In a test file read from standard input, `g.expect_screenshot` SHALL print the screenshot as `--show` does, and SHALL compare and write nothing.

#### Scenario: First run
- **WHEN** a case calls `g.expect_screenshot()` and no reference exists, without `--update`
- **THEN** the case fails, naming the missing reference
- **AND** `tests/screenshots/<file>/<case>.txt.new` holds the screenshot

#### Scenario: Accept with update
- **WHEN** the same run is repeated with `--update`
- **THEN** the case passes, the reference holds the screenshot, and no `.new` file remains

#### Scenario: Named screenshot
- **WHEN** the case `Shows the focused pane` in `tests/pane_spec.lua` calls `g.expect_screenshot("two panes")`
- **THEN** its reference is `tests/screenshots/pane_spec/shows-the-focused-pane--two-panes.txt`

### Requirement: Report
`gband test` SHALL write its report to standard output, and SHALL write nothing to standard error except the one line of an invalid invocation. `print` in a test file SHALL write to standard output when called. The report SHALL hold one line per case that ran, naming the file, the case, whether it passed and how long it took, and a last line counting the cases passed and failed. For each failed case and each failed file, it SHALL hold the error message with its file and line, a diff from the reference to the screenshot when a screenshot differed, the screenshot when the reference was missing, and the lines the case's client and server logs hold from `print` and from plugin and configuration errors. With `--show`, it SHALL hold every screenshot the run took, under the name of its case and its own name.

#### Scenario: Failure shows the diff and the logs
- **WHEN** a case's screenshot differs from its reference in one row, and the plugin printed `drawn` in the client
- **THEN** the report holds a diff naming the changed row
- **AND** the report holds the client log line with `drawn`

#### Scenario: Show
- **WHEN** the user runs `gband test --show` and a case passes after two screenshots
- **THEN** standard output holds both screenshots under the case's name
