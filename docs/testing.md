# Testing a gband plugin

`gband test` runs Lua test files against real gband clients and servers.
Each case starts `gband attach` in a terminal of its own, in a directory tree of its own, drives it with keys, and reads back what the terminal shows.
A case can also reach into the client's and the server's Lua, wait until both have finished reacting, and freeze the time they read.
Nothing you have configured or running is touched.

This guide assumes you know [the plugin guide](plugins.md).

## Layout

A plugin keeps its tests in `tests/`, beside its manifest:

```
pane/
├── plugin.lua
├── lua/pane/init.lua
└── tests/
    ├── pane_spec.lua
    ├── helpers.lua
    └── screenshots/
        └── pane_spec/
            └── shows-the-focused-pane--two-panes.txt
```

Every file whose name ends in `_spec.lua` under `tests/`, at any depth, is a test file.
Other files, such as `helpers.lua`, are modules the test files can `require`.
`screenshots/` holds the committed screenshot references, as "Screenshots" describes.

## Running tests

| command | runs |
|---|---|
| `gband test` | every `_spec.lua` file under `tests/` of the working directory, in byte order of their paths |
| `gband test FILE...` | the named files, in the order given |
| `gband test -` | one test file read from standard input, named `stdin` |

| option | effect |
|---|---|
| `--update` | write every screenshot reference the run compares, and remove its `.new` file |
| `--show` | print every screenshot the run takes |
| `--filter PATTERN` | run only the cases whose name holds `PATTERN` as plain text |
| `--plugin PATH` | install the plugin at `PATH` in every case; repeatable |

When the working directory holds `plugin.lua`, it is the plugin under test, and every case installs it.
So `gband test` in a plugin's directory tests that plugin.

The report goes to standard output: one line per case, `PASS` or `FAIL`, the file, the case and how long it took, and a last line counting the cases passed and failed.
A failed case is followed by its error with the file and line, the diff or the screenshot when a screenshot did not match, and the lines its client and server logs hold from `print` and from plugin and configuration errors.

The exit status is 0 when every case passed, 1 when a case or a file failed, and 2 for an invalid invocation: a file that does not exist, no test file found, `-` given with a file, two plugins with the same directory name, or `-s`, `-S` or `-p`, which do not apply to `test`.
An invalid invocation prints one line on standard error and runs nothing.

`gband test` needs no terminal, writes no log of its own, and never connects to, starts or stops a server you run.
It works from inside one of your own gband windows too.

## Test files

A test file runs in a Lua state of its own, the test side.
There `gband.side` is `"test"`, `gband.api_version` is `1`, and every other field of `gband` is an error naming the side that has it: test code drives gband from outside, so it reaches the client and the server only through the case handle.
`print` writes its line to standard output.
`require` also finds modules in the test file's own directory.

```lua
local t = require("gband.test")

t.case("the segment shows the focused pane", function(g)
  g.start({ config = [[gband.plugin("pane")]] })
  t.match(g.screen().row(23), "pane 1")
end)
```

`t.case(name, fn)` or `t.case(name, opts, fn)` registers a case.
The name is a non-empty string, unique in its file.
After the file's top level returns, its cases run one at a time in the order registered, each calling `fn` with a new handle.
A case passes when `fn` returns, and fails when it raises an error or runs longer than its time limit: `opts.timeout` seconds, 30 by default.
A failing case does not stop the next one.
When the file's top level raises an error or runs longer than 30 seconds, the file fails and none of its cases run.

### Assertions

| function | fails when |
|---|---|
| `t.eq(actual, expected, message)` | the values differ; tables are compared key by key, at any depth |
| `t.ok(value, message)` | `value` is nil or false |
| `t.match(text, pattern, message)` | `text` is not a string, or the Lua pattern does not match it |

A failing assertion raises an error at the line of its call, naming the expected and the actual value, and `message` when given:

```
tests/pane_spec.lua:9: expected 4, got 3
```

## The case

`g.start(opts)` starts the case's gband.
Call it once, before any other function of the handle.

| field | meaning | default |
|---|---|---|
| `size` | the terminal size, `"<cols>x<rows>"` | `"80x24"` |
| `config` | the contents of `user/init.lua` | none: the default configuration |
| `server_config` | the contents of `user/server.lua` | none: the default server configuration |
| `files` | a table from paths relative to the configuration directory to contents, such as `{ ["user/lua/extra.lua"] = "..." }` | none |
| `plugins` | a list of further plugin directories for this case, relative to the test file's directory | none |
| `env` | environment variables to set, or to remove with `false` | none |
| `time` | the frozen instant: Unix seconds, `"YYYY-MM-DD HH:MM:SS"` in UTC, or `false` for the real time | `"2025-01-01 12:00:00"` |

A `config` replaces the default configuration, as `user/init.lua` does, so give the status line segments and bindings the case needs.

Each case gets a new directory tree under the system's temporary directory, holding its own configuration, data, state and runtime directories and a working directory.
The plugin under test, every `--plugin` and every `plugins` entry are linked into its plugins directory under their directory names.
Its gband runs with:

- `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME` and `XDG_RUNTIME_DIR` inside the tree
- `SHELL=/bin/sh`, `PS1='$ '`, `INPUTRC=/dev/null`, `TERM=xterm-256color`, `COLORTERM=truecolor`, `TZ=UTC` and `GBAND_ANIMATIONS=off`
- a directory of the tree first in `PATH`, holding `xdg-open` and `open`, which record their argument for `g.opened()` and open nothing
- `GBAND_TEST_SOCKET`, as "The test channel" describes, and no `GBAND`, `GBAND_SESSION`, `GBAND_PANE` or `GBAND_LOG`
- then the changes `env` asks for

`g.start` runs `gband attach` from the same executable as `gband test`, so the client, the server and the runner are one build.
It returns once the client has drawn its first frame with the configuration loaded.
When the case ends, passed or failed, the runner stops the client, the server and every process they started, and removes the tree.

### Driving

| function | effect |
|---|---|
| `g.keys(keys)` | presses keys, named as in key bindings and separated by spaces, such as `"ctrl+space enter"`; each is written as xterm sends it |
| `g.type(text)` | types `text` |
| `g.paste(text)` | pastes `text`, bracketed when the client enabled bracketed paste |
| `g.run(line)` | types `line` and Enter |
| `g.resize(size)` | resizes the terminal to `"<cols>x<rows>"`, and returns once the client has taken the size |
| `g.write(path, contents)` | writes a file relative to the configuration directory |
| `g.reload()` | reloads the client's and the server's configuration and plugins; returns the error message of a failed load, or nil |
| `g.settle()` | waits until the client and the server have finished reacting, as "Settle and wait" describes |
| `g.wait(predicate, opts)` | calls `predicate(screen)` until it returns something other than nil and false, and returns that |
| `g.wait_text(text, opts)` | waits until the screen's text holds `text` |
| `g.client(chunk, ...)` | runs `chunk` in the client's Lua, with the arguments as `...`, and returns its results |
| `g.server(chunk, ...)` | the same in the server's Lua |
| `g.set_time(time)` | moves the frozen instant, as `time` of `g.start` |

`g.wait` and `g.wait_text` give up after `opts.timeout` seconds, 5 by default, and fail the case with the screenshot taken when they gave up.
An invalid argument to any of them is an error at the line of the call.

### Running Lua in the client and the server

`g.client` and `g.server` send a chunk of Lua source to the process, which runs it as a callback that belongs to no plugin, between its other work.
The chunk can do anything a callback of that side can: read `gband.layout()`, dispatch actions, set window state, emit events.
Actions it dispatches take effect after it returns, so follow it with `g.settle()` before looking at the result.
Arguments and results are plain data, as [the plugin guide](plugins.md#plain-data) defines.

```lua
local side, count = g.client("return gband.side, select('#', ...)", 1, 2)
g.server([[gband.pane_state("default", 1).agent = "waiting"]])
g.settle()
t.eq(g.client("return gband.pane_state(1).agent"), "waiting")
```

A chunk that fails to compile, raises an error, hits the instruction limit or returns something that is not plain data fails the call with the reason, at the line of the call.
The process keeps running, and the error is not shown or logged as a plugin error.

### Observing

`g.screen()` returns a new table describing the terminal as the runner reads it:

| field | value |
|---|---|
| `cols`, `rows` | the terminal size |
| `cursor` | `{ row, col, visible }` |
| `row(n)` | row `n` as text, without trailing spaces |
| `text()` | every row, joined with newlines |
| `cell(row, col)` | `{ char, fg, bg, bold, dim, italic, underline, inverse }` |

Rows and columns count from 0 at the top left.
A colour is a string `"#rrggbb"`, an integer from 0 to 255 for a palette colour, or nil for the terminal's default.
Call `row`, `text` and `cell` with a dot: `g.screen().cell(23, 0).fg`.

The handle also returns what the client wrote to its terminal since `g.start`, as new lists:

| function | each entry |
|---|---|
| `g.notifications()` | `{ title, body }` for a desktop notification; OSC 9 carries no title |
| `g.clipboard()` | the text of a clipboard write |
| `g.opened()` | the argument of each run of `xdg-open` or `open` |
| `g.bells()` | the number of bells, not a list |
| `g.log(side)` | the lines of the `"client"` or `"server"` log so far, including those of `print` |

## Screenshots

`g.screenshot(opts)` returns the screen as text:

```
size 60x4 cursor 1:3 shown
0|┌────────────────────────────┐┌────────────────────────────┐
1|│$                           ││$                           │
2|└────────────────────────────┘└────────────────────────────┘
3|band 1                                                pane 1
--
0:0-29 bold
0:30-59 dim
1:0 bold
1:29 bold
1:30 dim
1:59 dim
2:0-29 bold
2:30-59 dim
3:0-53 fg=#c0caf5 bg=#24283b
3:54-59 fg=#7aa2f7 bg=#24283b bold
```

- The header gives the size, the cursor's row and column, and `shown` or `hidden`.
- One line per row follows, its number right-aligned, `|`, and its text without trailing spaces. A wide character appears once.
- After `--`, one line per run of cells on a row that share a style other than the default: `row:first-last`, or `row:col` for one cell, then `fg=` and `bg=` for colours that are not the default, as `#rrggbb` or the palette index, then `bold`, `dim`, `italic`, `underline` and `inverse` as they apply, in that order.

`opts.styles = false` leaves out `--` and the style lines, for cases about text alone.

`g.expect_screenshot(name, opts)` takes a screenshot and compares it with its reference, `screenshots/<file>/<case>.txt` in the test file's directory.
`<file>` is the test file's name without `.lua`.
`<case>` is the case's name, followed by `--` and `name` when given, lowercased, with every run of other characters than ASCII letters and digits turned into one `-`.
So the case `Shows the focused pane` of `tests/pane_spec.lua` calling `g.expect_screenshot("two panes")` compares with `tests/screenshots/pane_spec/shows-the-focused-pane--two-panes.txt`.
Two comparisons with one reference in a run are an error, so give each screenshot of a case its own name.

- A matching reference passes.
- A missing reference fails the case and the report shows the screenshot.
- A reference that differs fails the case and the report shows a diff of the rows that changed, `-` for the reference and `+` for the screen.
- Either way the runner writes the screenshot beside the reference, with `.new` added, for you to inspect.
- `--update` writes every compared screenshot as its reference and removes the `.new` files. Review the change with `git diff` and commit the references.

In a test file read from standard input, `g.expect_screenshot` prints the screenshot and compares and writes nothing, so a quick experiment leaves no files behind.

## Frozen time

Each case freezes the time the client's and the server's Lua read at `2025-01-01 12:00:00` UTC, so a clock segment draws `12:00` on every run.
`os.time()` without a table returns the instant, and `os.date(format)` without a time formats it; calls with an explicit time behave as Lua defines.
The instant does not advance by itself, and timers still fire on the real clock, so a component's `redraw_interval` keeps working.
`TZ=UTC` makes `os.date` the same on every machine.

```lua
g.start({ config = [[gband.plugin("gband.statusline.clock")]] })
t.match(g.screen().row(23), "12:00$")
g.set_time("2025-01-01 12:05:00")
g.wait_text("12:05")
```

Give `time = false` to `g.start` for the real time.

## Settle and wait

`g.settle()` returns once gband has finished reacting to everything the case did so far:

1. the client has read and handled every key the case wrote,
2. the server has handled every message the client sent, applied the actions, run its Lua handlers and sent the changes,
3. the client has handled every message the server sent, run its handlers, finished any animation and drawn the result,
4. the runner has read the frame,

and again for the effects of those effects, until a round changes nothing, up to ten rounds.
Handlers that keep triggering each other make it fail after ten rounds.
A settle that does not finish in 10 seconds fails naming the step that did not.
`g.start` returns after a settle.

Settle does not wait for programs in windows: a shell may print its prompt or a command its output at any time.
Wait for those with `g.wait_text` or `g.wait`:

```lua
g.run("echo hi")
g.wait_text("hi")
```

Use settle for gband's own work, such as keys, bindings, handlers, the status line, and wait for what programs in windows print.
Never sleep.

A few things to know when comparing screenshots with window contents:

- A window's program starts before the client attaches, at the size of an 80 by 24 screen, and takes its tile's size shortly after. Output it prints before then wraps at the old width. Use the default `80x24` terminal when a screenshot holds the output of the first window's program, or wait for the program to see its final size.
- Wait for the prompt before a screenshot that shows a shell, since the shell prints it in its own time.
- A program that runs `$SHELL` gets `/bin/sh`, whose prompt differs between systems once `PS1` is lost; `env = { SHELL = "/bin/cat" }` gives windows without a prompt.

## The test channel

The runner reaches the client and the server through a Unix socket in the case's tree.
When `GBAND_TEST_SOCKET` names a socket, `gband attach` and `gband server` connect to it before loading their configuration, check that the process listening runs as the same user, and say which side they are.
When the connection fails, the process prints one line naming the path and the reason, logs it, and exits with status 1.
When the variable is unset or empty, gband behaves as it always does.
The server removes the variable from every window's environment, so a gband started in a window of the case does not join the channel.
When the channel closes, the client detaches and the server stops as on SIGTERM, so a killed runner leaves nothing running.

The channel carries chunks only between the runner and the process its own environment named.
No Lua crosses the connection between a client and a server, and a server never forwards a chunk to a client.

## A worked example

The [pane sample](../examples/plugins/pane) shows the focused window in the status line.
The first case of its `tests/pane_spec.lua` opens a second window, moves back to the first, and keeps the screen as a reference, the screenshot shown under "Screenshots":

```lua
local t = require("gband.test")

local CONFIG = [[
  gband.keymap.set("prefix", "enter", gband.action.open_pane)
  gband.keymap.set("prefix", "h", gband.action.focus_column_left)
  gband.plugin("gband.statusline.band")
  gband.plugin("pane")
]]

local function prompts(g, count)
  g.wait(function(screen)
    local _, found = screen.text():gsub("│%$", "")
    return found == count
  end)
end

t.case("shows the focused pane", function(g)
  g.start({ size = "60x4", config = CONFIG })
  t.match(g.screen().row(3), "pane 1$")
  g.keys("ctrl+space enter")
  g.settle()
  t.match(g.screen().row(3), "pane 2$")
  g.keys("ctrl+space h")
  g.settle()
  t.match(g.screen().row(3), "pane 1$")
  prompts(g, 2)
  g.expect_screenshot("two panes")
end)
```

`g.settle()` is enough for the status line, which gband draws, and `prompts` waits for the shells, which print in their own time.

Start with a chunk on standard input and look at what gband draws:

```sh
cd examples/plugins/pane
gband test - --show <<'EOF'
local t = require("gband.test")
t.case("look", function(g)
  g.start({ size = "60x4", config = [[gband.plugin("gband.statusline.band") gband.plugin("pane")]] })
  g.expect_screenshot()
end)
EOF
```

Then move the case into `tests/pane_spec.lua`, run `gband test --update` to write its reference, check the reference with `git diff`, and commit it.
From then on `gband test` fails with a diff whenever the segment draws differently.
