# 11 · Testing

The last chapter writes tests for the `marks` plugin.
`gband test` starts a real gband in a terminal of its own, drives it with keys, and reads back the screen and both sides' Lua, so a test checks what you would check by hand, every time.

The finished files are in [examples/tutorial/11-testing/](../../examples/tutorial/11-testing/).

## Where tests live

A plugin keeps its tests in `tests/`, beside its manifest.
Every file whose name ends in `_spec.lua` is a test file:

```text
plugins/marks/
├── plugin.lua
├── client.lua
├── server.lua
├── lua/marks/init.lua
└── tests/
    ├── marks_spec.lua
    └── screenshots/marks_spec/
```

Run `gband test` in the plugin's directory.
gband sees `plugin.lua` there and installs the plugin in every case, and nothing you have configured or running is touched.

## A test file

A test file requires the test module, `gband.test`, and registers cases with `t.case`.
Each case gets a handle, `g`, and starts gband with `g.start`:

```lua
local t = require("gband.test")

local CONFIG = [[
gband.plugin("gband.sidebar")
gband.plugin("marks")
gband.keymap.set("prefix", "m", gband.action["marks.mark"])
gband.keymap.set("prefix", "M", gband.action["marks.list"])
gband.keymap.set("prefix", "n", gband.action.open_window)
gband.keymap.set("prefix", "h", gband.action.focus_column_left)
]]

local function start(g)
  g.start({ size = "40x8", config = CONFIG, env = { SHELL = "/bin/cat" } })
end
```

`config` is the case's `user/init.lua`.
It replaces the default configuration, so it sets up only what the tests need: the sidebar, the plugin and a few keys, without a key style, so that each key after the prefix acts once.
A small terminal keeps screenshots short, and `/bin/cat` as the shell gives windows that print no prompt, so nothing on screen depends on timing.
[The case](../testing.md#the-case) lists every field of `g.start`.

## Driving and reading

`g.keys` presses keys, named as bindings name them.
`g.settle()` waits until the client and the server have finished reacting, and the screen shows the result.
`g.client(chunk)` and `g.server(chunk)` run Lua in each process and return its results, so a test can check the window state the server holds:

```lua
t.case("marks the focused window", function(g)
  start(g)
  g.keys("ctrl+space m")
  g.settle()
  t.eq(g.server([=[return gband.window_state("default", 1)["marks.letter"]]=]), "a")
  t.match(g.screen().row(0), "^Ia")
  g.expect_screenshot("marked")
end)
```

`t.eq`, `t.ok` and `t.match` are the assertions; a failing one names the expected and the actual value.
`g.screen()` returns the terminal's text and the style of each cell.
[Driving](../testing.md#driving) and [Observing](../testing.md#observing) list the rest of the handle, such as `g.notifications()` and `g.clipboard()`.

A chunk can act too: setting window state in the server checks that the client draws a mark it did not make:

```lua
t.case("shows a mark the server sets", function(g)
  start(g)
  g.server([=[gband.window_state("default", 1)["marks.letter"] = "b"]=])
  g.settle()
  t.match(g.screen().row(0), "^Ib")
end)
```

Use `g.settle()` for gband's own work, and `g.wait_text` for what programs in windows print, which happens in their own time.
Never sleep. [Settle and wait](../testing.md#settle-and-wait) explains the difference.

## Screenshots

`g.expect_screenshot(name)` compares the whole screen, text and styles, with a reference file under `tests/screenshots/`.
The first run has no reference, so it fails and shows the screenshot.
Run `gband test --update` to write the references, read them, and commit them:

```sh
cd plugins/marks
gband test --update
git diff --stat
```

From then on, a change that draws differently fails with a diff of the rows that changed.
The list's reference, from the case `lists the marks in letter order`:

```screen plugins/marks/tests/screenshots/marks_spec/lists-the-marks-in-letter-order--two-marks.txt
Ib╭─────────────────╮╭─────────────────╮
  │                 ││                 │
1 │       ┌marks───────────────┐       │
2 │       │a window 2 in band 1│       │
  │       │b window 1 in band 1│       │
  │       └────────────────────┘       │
  │                 ││                 │
  ╰─────────────────╯╰─────────────────╯
```

The time is frozen at noon on 1 January 2025 in every case, so a clock in a bar draws the same on every run.
[Screenshots](../testing.md#screenshots) describes the file format and how references are named.

## Testing your configuration

The tests of each chapter of this tutorial run your configuration rather than the plugin's.
Their `g.start` reads `user/init.lua` from the chapter's directory, with a small helper module in `tests/`:

```lua
function M.read(path)
  local file = io.open(path)
  if file == nil then
    return nil
  end
  local text = file:read("a")
  file:close()
  return text
end
```

Test files can read files, and `gband test` runs in the chapter's directory, so `user/init.lua` resolves there.
`plugins = { "../plugins/marks" }`, relative to the test file, installs the chapter's plugin in each case.
Copy any chapter's directory, replace its `user/` with yours, and `gband test` tells you whether your configuration still does what the chapter says.

## Run them all

```sh
cd examples/tutorial/11-testing
gband test
cd plugins/marks
gband test
```

[Testing a plugin](../testing.md) is the full reference.
That is the whole tutorial: from an empty file to a plugin with a client half, a server half and tests.
Back to [the index](README.md).
