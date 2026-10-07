# 12 · Your own prelude

The prelude decides which of gband's Lua runs before your configuration.
[00](00-boundary.md) showed that `require` looks on the runtimepath before it looks among the bundled modules, and the prelude is a module like any other.
This chapter replaces it with a copy that adds a module of your own: a clock in a bar at the right edge.

The finished files are in [examples/internals/12-own-prelude/](../../examples/internals/12-own-prelude/).

## Copy the prelude

Copy `defaults/lua/gband/prelude.lua` to `user/lua/gband/prelude.lua`, and add one line at the end:

```lua
require("gband.hl")
require("gband.palette")
require("gband.colorscheme")
require("gband.bar")
require("gband.win")
require("gband.settings")
require("gband.keystyle")
require("gband.colorscheme").start()
require("mine.clock")
```

The client requires `gband.prelude` before it runs `user/init.lua`.
`user/` is the first runtimepath entry, so `require` finds `user/lua/gband/prelude.lua` and never reaches the bundled one.
From now on your copy decides what loads.
A later gband that adds a module to its own prelude does not add it to yours, so compare your copy with `defaults/lua/gband/prelude.lua` when you update.

Keep the bundled lines in their order: each module uses those above it as it loads, as [00](00-boundary.md) explains.
Leaving one out leaves its feature out, and whatever needs it fails.
Without `gband.settings`, for example, `start` on the last bundled line cannot read the saved theme, and the key style presets cannot bind the settings window.

## The clock

Write `user/lua/mine/clock.lua`:

```lua
local core = gband.core

local function now()
  return { os.date("%H:%M") }
end

gband.bar.add({ id = "clock", side = "right", size = 5, lines = now() })

core.timer(1000, function()
  gband.bar.set_lines("clock", now())
end)
```

The module is code of no plugin, like `user/init.lua`, so its bar must have an id, as [04](04-bars.md) showed.
It is added while the prelude loads; the bars provider presents it at the first flush after the load, with the others.

The time is read with `os.date`, so a bar of minutes needs redrawing.
None of gband's bundled Lua needs a timer, but `gband.core.timer` is there for code like this: it calls the function every 1000 milliseconds, as code of no plugin, until it is cancelled with `gband.core.cancel`.
`gband.bar.set_lines` only marks the bar dirty, so the bar is presented again at the next flush.

## The init file

Because the prelude ran first, `user/init.lua` already has the clock, and sets up the rest as the default configuration does:

```lua
gband.keystyle.use()
gband.plugin("gband.sidebar")
```

With one window open, the clock sits at the right edge and the sidebar at the left:

```screen tests/screenshots/own_prelude_spec/the-own-prelude-adds-the-clock--the-clock.txt
I╭───────────────╮                 12:00
 │               │
1│               │
2│               │
 │               │
 ╰───────────────╯
```

## The tests

`tests/own_prelude_spec.lua` starts gband with the chapter's files, read with `io.open`, and checks that the clock is there, that the client loaded `mine.clock`, which only your prelude requires, and that the clock follows the time.
`gband test` freezes the time at `2025-01-01 12:00:00`, so the clock always shows `12:00`, and `g.set_time` moves it; the timer then redraws the clock within a second.
Run them from the chapter's directory:

```sh
cd examples/internals/12-own-prelude
gband test
```

## Where to go from here

The same way, any file under `defaults/lua/gband/` can be copied to `user/lua/gband/` and changed, and the chapters before this one tell you what each line of it does.
Keep the exports listed in [the plugin guide](../plugins.md#the-prelude-and-the-api-modules), which other modules use, and expect to compare your copy with the bundled file when you update gband.
