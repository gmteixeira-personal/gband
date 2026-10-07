# 09 · A plugin

This chapter moves the marks out of `user/init.lua` into a plugin called `marks`, with options of its own, so that you can share it, test it, and keep your configuration short.

The finished files are in [examples/tutorial/09-plugin/](../../examples/tutorial/09-plugin/).

## Where plugins live

A plugin is a directory in the plugins directory, `~/.local/share/gband/plugins/`, or `$XDG_DATA_HOME/gband/plugins/` when that is set.
gband searches a list of directories for modules, colorschemes and plugins, `gband.runtimepath`.
It starts with your `user/` directory, then every directory under the plugins directory, in order of their names.
Run `print(gband.runtimepath[2])` at the prompt once the plugin is installed, and the log shows its directory.

The plugin's directory, `marks/`, holds:

| file | what it is |
|---|---|
| `plugin.lua` | the manifest |
| `client.lua` | code every client runs |
| `lua/marks/init.lua` | the module `require("marks")` finds, with `setup` |

Copy [examples/tutorial/09-plugin/plugins/marks/](../../examples/tutorial/09-plugin/plugins/marks/) there, or link it.
[Where gband looks](../plugins.md#where-gband-looks) lists every file a plugin can hold.

## The manifest

`plugin.lua` names the plugin and its version:

```lua
return {
  name = "marks",
  version = "0.1.0",
}
```

The name must equal the directory's.
`gband.plugins()` returns every plugin whose manifest loaded, each with `name`, `version` and `failed`.
[The manifest](../plugins.md#the-manifest) has its rules.

## Code every client runs

gband runs `client.lua` in each client after your `user/init.lua`, whether or not your configuration sets the plugin up.
It suits what needs no options, such as the default style of `Mark`, which a theme can then style even before the plugin is set up:

```lua
gband.hl.default("Mark", { fg = "yellow", bold = true })
```

## The module and `setup`

Everything else moves into the module, `lua/marks/init.lua`.
A plugin module is a table with a `setup` function, an optional `name`, and `api`, the version of gband's Lua API it was written for:

```lua
local M = { name = "marks", api = 1 }

local marks = {}
local letters
local list
local bar
```

`gband.api_version` is the version this gband provides.
When a plugin's `api` differs, gband logs a warning and sets the plugin up anyway, so you hear about a plugin that may need changes before it breaks.

The functions of chapters 01 to 08 move into the module unchanged, as locals above `setup`.
The registrations move into `setup`, which receives the options your configuration passes:

```lua
function M.setup(opts)
  assert(gband.side == "client", "set marks up in user/init.lua")
  if opts.bar ~= nil and opts.bar ~= false and opts.bar ~= "left" and opts.bar ~= "right" then
    error("`bar` is \"left\", \"right\" or false, found " .. tostring(opts.bar))
  end

  letters = gband.opt.declare("letters", {
    type = "string",
    default = "abc",
    desc = "the letters that marks take, in order",
  })
```

The one option, `bar`, says which side the bar goes on, or `false` for none:

```lua
  if opts.bar ~= false then
    bar = gband.bar.add({ side = opts.bar or "left", size = 1, order = 1 })
  end
```

`gband.opt.declare` returns the option's full name, which `toggle` now reads it by:

```lua
  for letter in gband.opt[letters]:gmatch(".") do
```

[Plugin modules and `gband.plugin`](../plugins.md#plugin-modules-and-gbandplugin) describes the module's shape.

## Names belong to the plugin

Code that `setup` runs belongs to the plugin, and so does every callback it registers.
Names it registers are prefixed with the plugin's name: the action `mark` becomes `marks.mark`, the option `letters` becomes `marks.letters`, and the bar's id defaults to `marks`.
So two plugins can each have an action called `list`.
Highlight groups, key tables and event names are shared by everyone, which is why the event is called `marks.changed` and the jump table `marks`.

An error in the plugin's code names the plugin, as `marks: .../lua/marks/init.lua:12: ...`, and marks it failed: its bindings, handlers and actions stop until the next load.
[Ownership and names](../plugins.md#ownership-and-names) and [Errors](../plugins.md#errors) have the rules.

## The two sides

The server runs Lua too, as chapter 10 shows, and each side's `gband` holds only that side's API.
`gband.side` is `"client"` in a client and `"server"` in the server.
Reading a field of the other side raises an error at once, naming the field:

```text
user/server.lua:1: `gband.keymap` is a client API; this is the server
```

A module on the runtimepath can be required on either side, so `setup` checks the side first with `gband.side`, and a `gband.plugin("marks")` in `user/server.lua` fails with a message that says where it belongs.
[The side guard](../plugins.md#the-side-guard) lists which fields belong to which side.

## Setting it up

`gband.plugin(name, opts)` requires the module and calls `setup(opts)`.
It returns `true` when `setup` returns, and `false` when the plugin failed, so the bindings can depend on it.
`user/init.lua` keeps its own bindings and sets the plugin up:

```lua
if gband.plugin("marks", { bar = "left" }) then
  gband.keymap.set("prefix", "m", gband.action["marks.mark"])
  gband.keymap.set("prefix", "y", gband.action["marks.copy"])
  gband.keymap.set("prefix", "X", gband.action["marks.close"])
  gband.keymap.set("prefix", "M", gband.action["marks.list"])
  gband.keymap.set("prefix", "'", gband.action["marks.jump"])
  gband.keymap.set("root", "ctrl+rightmouse", function(event)
    if event.window then
      gband.cmd.run("marks.mark", { window = event.window })
    end
  end, { desc = "mark the window under the pointer" })
end
```

Without the `if`, a plugin that failed would leave `gband.action["marks.mark"]` nil, and the binding would fail the whole file.
Choose the letters before the call, as chapter 03 did:

```lua
gband.opt["marks.letters"] = "xyz"
```

## The whole plugin

The complete files are in [examples/tutorial/09-plugin/](../../examples/tutorial/09-plugin/): `user/init.lua`, and `plugins/marks/` with [lua/marks/init.lua](../../examples/tutorial/09-plugin/plugins/marks/lua/marks/init.lua).

Next: [10 · The server half](10-server.md).
