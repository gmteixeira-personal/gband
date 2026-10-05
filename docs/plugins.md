# Writing a gband plugin

A gband plugin is a directory of Lua files.
gband finds it on the runtimepath, runs its startup files, and lets your configuration set it up.
Plugins run in the client.
They can add actions, commands, options, key bindings and event handlers.

The [sample plugin](../examples/plugins/hello) uses most of what this guide describes.

## Where gband looks

`gband.runtimepath` is a Lua list of directories.
When the configuration starts to load, it holds:

1. the `user` directory of the configuration directory, `~/.config/gband/user/`
2. every directory directly under the plugins directory, in byte order of their names

The plugins directory is `$XDG_DATA_HOME/gband/plugins/`, or `~/.local/share/gband/plugins/` when `XDG_DATA_HOME` is unset or not absolute.
To install a plugin, copy or link its directory there.
A missing plugins directory, or a runtimepath entry that does not exist, is not an error.

Each runtimepath entry can hold:

| path | use |
|---|---|
| `lua/<name>.lua`, `lua/<name>/init.lua` | modules that `require("<name>")` finds |
| `plugin/*.lua` | files gband runs at startup |
| `plugin/client/*.lua` | files gband runs at startup, after `plugin/*.lua` |
| `plugin/server/` | reserved for a future server runtime; gband never runs these files |

`require("a.b")` looks in each runtimepath entry in order, for `lua/a/b.lua` and then `lua/a/b/init.lua`, before Lua's own search path.
The first file found wins, so a module in `user/lua/` overrides a plugin's module of the same name.

## Load order

Each load runs in a new Lua state:

1. gband sets `gband.side` to `"client"` and `gband.api_version` to `1`.
2. gband runs the init file: `user/init.lua` when it exists, the default configuration otherwise.
3. For each runtimepath entry in order, gband runs `plugin/*.lua` in byte order of the file names, then `plugin/client/*.lua` in the same order.

The init file can change `gband.runtimepath`, and step 3 uses the list as it stands.
Each file runs once per load.
Saving any `.lua` file under `user/` reloads the configuration.
Changes in the plugins directory do not.

## Plugin modules and `gband.plugin`

A plugin that takes options exports a module with a `setup` function:

```lua
local M = { name = "hello", api = 1 }

function M.setup(opts)
  gband.action.register("greet", function()
    gband.spawn({ cmd = "echo " .. (opts.greeting or "hello") })
  end)
end

return M
```

- `setup`, required, is called with the options table.
- `name`, optional, is the plugin's name. It defaults to the module name.
- `api`, optional, is the plugin API version the plugin was written for. When it differs from `gband.api_version`, gband logs a warning and still sets the plugin up.

`gband.plugin(name, opts)` requires the module `name`, checks its shape and calls `setup(opts)`, or `setup({})` when `opts` is nil.
It returns `true` when `setup` returns.
It returns `false`, and reports a plugin error, when the module is missing, has the wrong shape, or `setup` raises an error.
Setting a plugin up twice in one load is a plugin error, and `setup` does not run again.

```lua
gband.plugin("hello", { greeting = "hi" })
```

A plugin without options can do its work in a `plugin/*.lua` file instead.

## Ownership and names

Code belongs to a plugin:

- a file sourced from a plugin directory belongs to the plugin named by that directory
- code that `gband.plugin` runs belongs to the plugin it sets up
- a callback belongs to the plugin that registered it, and so does the code it runs

The init file, the default configuration and the files under `user/` belong to no plugin.

Names that a plugin registers for actions and commands, and declares for options, are namespaced as `<plugin>.<name>`.
The plugin `hello` registering `greet` creates `hello.greet`.
A name that already starts with `hello.` is kept.
A name with a `.` in it that does not start with `hello.` is an error.
Names registered by code that belongs to no plugin are used as given.

## Errors

gband runs every plugin file, every `setup` and every callback protected.
An error there is a plugin error: gband records it in the log and shows the latest on the bottom row, as `<plugin>: <file>:<line>: <message>`.

A plugin error while the configuration loads does not fail the load.
It marks the plugin failed for the rest of that load:

- its remaining plugin files are not sourced
- its callbacks are disabled, including those registered before the error: their keys do nothing, its handlers do not run, its commands return `false`, and its actions do nothing
- the options it declared keep their values

An error in a callback after loading is reported, and the callback stays enabled.
Actions it dispatched before the error still happen.

Errors in the init file's own code still fail the load, and gband keeps the configuration it last loaded.

### The instruction limit

Each run of Lua code that gband starts stops with an error after 100,000,000 Lua VM instructions.
A run is the init file, one plugin file, one `setup`, or one call of a callback.
A plugin stopped this way is marked failed, also when the run is a callback after loading.
A `pcall` inside the plugin does not catch the stop for long: once the limit is reached, every instruction raises it again until the run ends.

The limit counts Lua instructions only.
It cannot stop a blocking call into C, such as `os.execute("sleep 100")` or `io.read()`.
Keep such calls out of plugins.

## `print`

`print` writes its arguments, converted with `tostring` and separated by tabs, to the process log at the info level, naming the plugin when the calling code belongs to one.
It never writes to the terminal, so it cannot corrupt the client's screen.
The client log is in `$XDG_STATE_HOME/gband/log/`, or `~/.local/state/gband/log/`.

## Register only at top level

The server evaluates the same configuration to read the column width options, plugins included, so a plugin's top-level code and its `setup` run in the server too.
`gband.side` is `"client"` there as well.
No callback ever runs in the server: it never presses keys, emits events or runs commands.

Keep top-level code and `setup` to declarations and registrations.
Do the work in callbacks.

## Actions: `gband.action`

`gband.action.<name>` holds every built-in action, and every registered action under its full name.
Calling an action value inside a callback dispatches it, with the same effect as pressing a key bound to it.

`gband.action.register(name, fn, { desc = "..." })` registers an action and returns its action value.
Dispatching it runs `fn` with no arguments, at once, before the caller continues.
Registering a name already taken, a built-in action's name, or the names `register` and `list` is an error.

`gband.action.list()` returns `{ name = ..., desc = ... }` for every built-in and registered action, in byte order of names.

```lua
gband.action.register("twice", function()
  gband.action.focus_column_right()
  gband.action.focus_column_right()
end, { desc = "move two columns right" })
gband.keymap.set("root", "alt+t", gband.action.twice)
```

## Commands: `gband.cmd`

Commands are named functions that take a table of arguments.
No palette or command line runs them yet; `gband.cmd.list()` exists so one can.

- `gband.cmd.register(name, fn, { desc = "...", args = { "who" } })` registers a command and returns its full name. `args` names the arguments, for display.
- `gband.cmd.run(name, args)` calls the command's function with `args`, or `{}` when `args` is nil. It returns `true` when the function returns, and `false` when it raises an error or the command is disabled. An unknown name is an error.
- `gband.cmd.list()` returns `{ name = ..., desc = ..., args = { ... } }` for every command, in byte order of names. The tables are copies.

Called inside a callback, a command's function can dispatch actions.

## Options: `gband.opt`

`gband.opt.<name>` reads and sets an option.
The built-in options are `prefix`, `default_column_width`, `width_presets` and `center_focused_column`.

```lua
gband.opt.default_column_width = 1/3
local presets = gband.opt.width_presets
```

An invalid value is reported at the line of the assignment, does not fail the load, and resets the option to its default.

`gband.opt.declare(name, spec)` declares a plugin option and returns its full name:

```lua
gband.opt.declare("greeting", {
  type = "string",
  default = "hello",
  values = { "hello", "hi" },
  desc = "the text to print",
})
```

`type` is `"boolean"`, `"integer"`, `"number"` or `"string"`.
`values`, optional, lists the only values allowed.
`default` is required and must be valid.

A value assigned before its option is declared waits for the declaration.
The user can set `gband.opt["hello.greeting"] = "hi"` in `user/init.lua`, and the plugin declares `greeting` later in the load.
An assignment still undeclared when loading finishes is reported, and does not fail the load.

`gband.opt.list()` returns `{ name, type, default, value, desc }` for every option, built-in and declared, in byte order of names.

Options are set and declared only while the configuration loads.
`gband.set { ... }` still works, and raises an error on an invalid value.

## Events: `gband.on`, `gband.augroup`, `gband.emit`

`gband.on(event, fn, opts)` runs `fn` with a payload table each time `event` happens.

- `opts.group` puts the handler in a group, given by its id or its name.
- `opts.once = true` removes the handler before its first run.
- `opts.pattern`, for `User` events only, runs the handler only for `User` events of that name.

Handlers run in the order they were registered, and each gets its own copy of the payload.

`gband.augroup(name, opts)` returns the id of the group `name`, the same id each time within one load.
It first removes every handler in the group, unless `opts.clear` is `false`.
Defining a group at the top of a file keeps a reload from adding the same handlers twice.

`gband.emit(name, data)` emits a `User` event with the payload `{ name = name, data = data }`, and runs its handlers before it returns.
It can be called only inside a callback.

The built-in events, emitted by the client:

| event | payload | emitted when |
|---|---|---|
| `Attached` | `session` | once, after the client attaches |
| `FocusChanged` | `pane`, `previous`: pane numbers, nil when no pane | the focused pane changes |
| `BandChanged` | `band`, `previous`: band numbers | the viewed band changes |
| `PaneOpened` | `pane`, `band` | a pane appears in the layout |
| `PaneClosed` | `pane`, `band` | a pane leaves the layout |
| `TerminalResized` | `cols`, `rows` | the client's terminal changes size |
| `ConfigReloaded` | empty | a reload succeeded, to the new configuration's handlers |
| `KeyTableChanged` | `table`, `previous` | the active key table changes |

Attaching to a session emits no `PaneOpened`, `FocusChanged` or `BandChanged` for what is already there.

A handler can dispatch actions, call `gband.spawn` and `gband.keymap.enter`.
The actions run after the handler returns, and the events they cause are emitted in turn.
After ten rounds of events caused by handlers, gband drops the next round and logs a warning.

```lua
local group = gband.augroup("follow")
gband.on("PaneOpened", function(event)
  print("opened pane", event.pane, "in band", event.band)
end, { group = group })
```

## Key bindings: `gband.keymap`

Bindings live in key tables:

- `root` holds the keys pressed outside a key sequence
- `prefix` holds the keys pressed after the prefix key
- any other name is a table that a callback enters

`gband.keymap.set(table, key, action, { desc = "..." })` binds `key` in `table` to an action value or a function.
In a table other than `root`, the key `prefix` stands for the prefix key.
Binding a key again replaces the earlier binding.

`gband.keymap.del(table, key)` removes a binding.
`gband.keymap.list(table)` returns `{ key, desc, action }` for each binding, in the order they were made, where `action` is the full name of the bound action, or nil for a function.

`gband.keymap.current_table()` returns the active table.
`gband.keymap.enter(table)`, inside a callback, makes `table` active for the next key.
The next key ends the sequence: a bound key runs its binding, and any other key is discarded.

```lua
gband.keymap.set("move", "h", gband.action.focus_column_left, { desc = "left" })
gband.keymap.set("move", "l", gband.action.focus_column_right, { desc = "right" })
gband.keymap.set("prefix", "m", function() gband.keymap.enter("move") end, { desc = "move" })
```

`gband.bind` and `gband.unbind` still work: `gband.bind("alt+h", ...)` binds in `root`, and `gband.bind("prefix h", ...)` binds in `prefix`.

Bindings are made only while the configuration loads.

## Still to come

Later changes will add their own extension points, each with its own API:

- a server-side runtime, which will source `plugin/server/`
- a status line
- overlays and a command palette, which will read `gband.cmd.list()` and `gband.action.list()`
- notifications
- highlight groups
