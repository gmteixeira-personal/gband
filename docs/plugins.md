# Writing a gband plugin

A gband plugin is a directory of Lua files.
gband finds it on the runtimepath, reads its manifest, runs the file of its side, and lets your configuration set it up.
Plugins can add actions, commands, options, key bindings, event handlers, status line components, plugin windows, highlight groups and colorschemes in the client, and event handlers, shared window state, events and commands in the server.

The [sample plugin](../examples/plugins/hello) uses most of what the client API offers.
The [status line sample](../examples/plugins/window) adds a component, a highlight group and a colorscheme.
The [agent status sample](../examples/plugins/agent-status) has both sides: its server half watches windows, and its client half notifies, counts and jumps.
[Testing a plugin](testing.md) describes `gband test`, which runs a plugin against a real client and server and checks what it draws.

## Two sides

The client and the server are separate processes.
The server may run on another machine than the client, or on the same one; a plugin behaves the same either way.
Each process runs its own Lua state, loaded from the files of the machine it runs on, and the two never share Lua values.

The rule for where code goes:

- What must keep working while no client is attached, or must be shared by every attached client, runs in the **server**: watching window output, tracking which window waits for input, counting, queueing a notification for later.
- What concerns one user's screen, keyboard or machine runs in the **client**: key bindings, the status line, plugin windows, colors, desktop notifications, the clipboard, opening a URL.

The sides talk through plain data only: the server emits events and publishes window state, and a client calls server commands and gets their results.
No code ever crosses the connection, as "Trust" below describes.

`gband.side` is `"client"` in a client and `"server"` in the server, and `"test"` in a test file that [`gband test`](testing.md) runs.
`gband.api_version` is `1` on both.

## Where gband looks

`gband.runtimepath` is a Lua list of directories.
Each process builds it from its own environment.
When the configuration starts to load, it holds:

1. the `user` directory of the configuration directory, `~/.config/gband/user/`
2. every directory directly under the plugins directory, in byte order of their names

The plugins directory is `$XDG_DATA_HOME/gband/plugins/`, or `~/.local/share/gband/plugins/` when `XDG_DATA_HOME` is unset or not absolute.
To install a plugin, copy or link its directory there, on the server's machine for its server half and on each client's machine for its client half.
A missing plugins directory, or a runtimepath entry that does not exist, is not an error.

A plugin directory holds:

| path | use |
|---|---|
| `plugin.lua` | the manifest; a directory without one is not a plugin |
| `server.lua` | the server half, run by the server only |
| `client.lua` | the client half, run by each client only |
| `lua/<name>.lua`, `lua/<name>/init.lua` | modules that `require("<name>")` finds, on either side |
| `colors/<name>.lua` | colorschemes that `gband.colorscheme("<name>")` finds, in the client |

A plugin may hold only one of `server.lua` and `client.lua`; the other side then runs nothing of it.
A directory without a manifest still contributes its modules and colorschemes, so a plugin set up with `gband.plugin` needs no manifest.
A directory that holds `server.lua` or `client.lua` but no `plugin.lua` is a plugin error, and neither file runs.
No file under a `plugin/` directory is ever run: plugins written for the earlier `plugin/*.lua` layout move that code to `client.lua`.

`require("a.b")` looks in each runtimepath entry in order, for `lua/a/b.lua` and then `lua/a/b/init.lua`.
In the client, when no entry holds the module, it looks among the modules bundled with gband, such as `gband.statusline.band`, and only then in Lua's own search path.
The first file found wins, so a module in `user/lua/` overrides a plugin's module of the same name, and `user/lua/gband/statusline/band.lua` overrides the bundled one.
Errors in a bundled module name its path under `gband/`, such as `gband/statusline/band.lua:12:`.

## The manifest

`plugin.lua` returns a table:

```lua
return {
  name = "agent-status",
  version = "0.1.0",
  client = ">= 0.1",
}
```

- `name`, required, must equal the directory's name.
- `version`, required, is one to three non-negative integers separated by full stops. Missing parts read as 0, so `0.1` equals `0.1.0`.
- `client`, optional, is a requirement on the version of the plugin of the same name in each client, as "Requirements" below describes. Only the server reads it.

gband runs the manifest as code of the plugin, with the instruction limit, in an environment holding no globals: it can compute, but it cannot reach `gband`, `require` or any other global.
A manifest that raises an error, returns anything else, or names another plugin is a plugin error that marks the plugin failed, and its side file does not run.

`gband.plugins()` returns one table per plugin whose manifest the load ran, in runtimepath order, each holding `name`, `version`, `client` and `failed`, the last `true` when the plugin is failed.
The tables are copies.

## Load order

Each load runs in a new Lua state.
In a client:

1. gband sets `gband.side` to `"client"` and `gband.api_version` to `1`.
2. gband installs the client's `gband` API, including the status line and its built-in highlight groups, and loads the `default` colorscheme.
3. gband runs the init file: `user/init.lua` when it exists, the default client configuration otherwise.
4. gband runs every manifest, in runtimepath order.
5. For each plugin whose manifest is valid, in runtimepath order, gband runs its `client.lua`.

In the server:

1. gband sets `gband.side` to `"server"` and `gband.api_version` to `1`.
2. gband installs the server's `gband` API.
3. gband runs the init file: `user/server.lua` when it exists, the default server configuration otherwise.
4. gband runs every manifest, in runtimepath order.
5. For each plugin whose manifest is valid, in runtimepath order, gband runs its `server.lua`.

The `user` directory is never a plugin: its init files are `user/init.lua` and `user/server.lua`.
The init file can change `gband.runtimepath`, and steps 4 and 5 use the list as it stands.
Each file runs once per load.

Saving any `.lua` file under `user/` reloads the configuration of each process that watches it: every client, and the server.
Changes in the plugins directory do not.
A server reload keeps every window state, every queued event and every session; the old state's handlers and commands stop, and the new configuration's `ConfigReloaded` handlers run.

## The side guard

Each side's `gband` table holds only that side's API.
Reading a field that only the other side provides raises an error naming the field and the side, at the line of the read, so a misplaced script fails as it loads:

```
user/server.lua:3: `gband.keymap` is a client API; this is the server
```

| only the client | only the server |
|---|---|
| `bind`, `unbind`, `spawn`, `keymap`, `ui`, `hl`, `colorscheme`, `layout`, `view`, `window`, `band`, `win`, `rpc`, `notify`, `bell`, `clipboard`, `open` | `sessions`, `session` |

Every other field exists on both sides: `on`, `augroup`, `emit`, `cmd`, `opt`, `set`, `plugin`, `plugins`, `runtimepath`, `side`, `api_version`, `window_state` and `action`.
`emit`, `window_state`, `action` and the events differ between the sides, as their sections describe.
A test file's `gband` holds only `side` and `api_version`, and reading any other field names the sides that provide it, such as ``tests/a_spec.lua:4: `gband.opt` is a client and server API; this is the test side``.
In the server, `gband.action` holds the session actions only; reading a view or client action, such as `gband.action.focus_column_left`, is the same kind of error.

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

A plugin without options can do its work in its `client.lua` or `server.lua` instead.

## Ownership and names

Code belongs to a plugin:

- a manifest and a side file belong to the plugin named by their directory
- code that `gband.plugin` runs belongs to the plugin it sets up
- a callback belongs to the plugin that registered it, and so does the code it runs

The init files, the default configurations and the files under `user/` belong to no plugin.

Names that a plugin registers for actions and commands, and declares for options, are namespaced as `<plugin>.<name>`.
The plugin `hello` registering `greet` creates `hello.greet`.
A name that already starts with `hello.` is kept.
A name with a `.` in it that does not start with `hello.` is an error.
Names registered by code that belongs to no plugin are used as given.

## Errors

gband runs every manifest, every side file, every `setup` and every callback protected.
An error there is a plugin error: gband records it in the process's log and shows the latest as `<plugin>: <file>:<line>: <message>`.
While the status line is drawn, the error shows as its first item.
While it is not, the error shows on the bottom row of the ribbon.

The server sends each error of its own Lua to every attached client, and the latest one to each client that attaches, until its configuration next loads without errors.
A client shows them preceded by `server: `, such as `server: agent-status: /srv/.../server.lua:5: boom`.

A plugin error while the configuration loads does not fail the load.
It marks the plugin failed for the rest of that load:

- its side file is not run, or stops where the error happened
- its callbacks are disabled, including those registered before the error: their keys do nothing, its handlers do not run, its commands return `false`, and its actions do nothing
- the options it declared keep their values

An error in a callback after loading is reported, and the callback stays enabled.
Actions it dispatched before the error still happen.

Errors in the init file's own code still fail the load, and the process keeps the configuration it last loaded, or, at start, its side's default configuration with the plugins' side files.

### The instruction limit

Each run of Lua code that gband starts stops with an error after 100,000,000 Lua VM instructions.
A run is the init file, one manifest, one side file, one `setup`, one call of a callback, one colorscheme file, or one call of a status line component's `render`.
A plugin stopped this way is marked failed, also when the run is a callback after loading.
A `pcall` inside the plugin does not catch the stop for long: once the limit is reached, every instruction raises it again until the run ends.

The limit counts Lua instructions only.
It cannot stop a blocking call into C, such as `os.execute("sleep 100")` or `io.read()`.
Keep such calls out of plugins.
In the server, a handler that runs long delays the next events and commands; it never slows a window, as "Server events" describes.

## `print`

`print` writes its arguments, converted with `tostring` and separated by tabs, to the process log at the info level, naming the plugin when the calling code belongs to one.
It never writes to the terminal, so it cannot corrupt the client's screen.
The logs are in `/gband/log/`, or `~/.local/state/gband/log/`, one file per client and one for the server.
In a test file, `print` writes its line to the standard output of `gband test` instead.

## Register only at top level

Keep top-level code and `setup` to declarations and registrations, and do the work in callbacks.
Registrations, options and bindings are made only while the configuration loads, and the actions, events and calls that change something are made only in callbacks.

## Actions: `gband.action`

`gband.action.<name>` holds every built-in action, and every registered action under its full name.
Calling an action value inside a callback dispatches it, with the same effect as pressing a key bound to it.

`gband.action.register(name, fn, { desc = "..." })` registers an action and returns its action value.
Dispatching it runs `fn` with no arguments, at once, before the caller continues.
Registering a name already taken, a built-in action's name, or the names `register` and `list` is an error.

`gband.action.list()` returns `{ name = ..., desc = ... }` for every built-in and registered action of the side, in byte order of names.
The server's built-in actions are the session actions, which take a target naming the session, as "Session actions in the server" describes.

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
A server command's function also receives a context, and a client calls it with `gband.rpc`, as "Server commands" describes.

## Options: `gband.opt`

`gband.opt.<name>` reads and sets an option.
Each built-in option belongs to one side, and each process knows only its own side's options:

| side | options |
|---|---|
| client | `prefix`, `center_focused_column`, `loop_bands`, `statusline_position`, `statusline_height`, `statusline_separator`, `notify_style` |
| server | `default_column_width`, `width_presets` |

```lua
-- user/server.lua
gband.opt.default_column_width = 1/3
local presets = gband.opt.width_presets
```

`loop_bands`, `true` by default, lets focus go round from a band's last column to its first; `false` stops focus at either end.

An invalid value is reported at the line of the assignment, does not fail the load, and resets the option to its default.
Setting an option of the other side is reported naming the side that owns it, and reading one returns nil.

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

`gband.opt.list()` returns `{ name, type, default, value, desc }` for every option of the side, built-in and declared, in byte order of names.

Options are set and declared only while the configuration loads.
`gband.set { ... }` still works, and raises an error on an invalid value.

## Events: `gband.on`, `gband.augroup`, `gband.emit`

`gband.on(event, fn, opts)` runs `fn` with a payload table each time `event` happens.
`event` is a built-in event of the process's side, or `User` in a client.

- `opts.group` puts the handler in a group, given by its id or its name.
- `opts.once = true` removes the handler before its first run.
- `opts.pattern`, for `User` and `ServerEvent` events only, runs the handler only for events of that name.

Handlers run in the order they were registered, and each gets its own copy of the payload.

`gband.augroup(name, opts)` returns the id of the group `name`, the same id each time within one load.
It first removes every handler in the group, unless `opts.clear` is `false`.
Defining a group at the top of a file keeps a reload from adding the same handlers twice.

In a client, `gband.emit(name, data)` emits a `User` event with the payload `{ name = name, data = data }`, and runs its handlers before it returns.
In the server, `gband.emit` sends an event to clients instead, as "Emitting events" describes.
It can be called only inside a callback.

The built-in events of the client:

| event | payload | emitted when |
|---|---|---|
| `Attached` | `session` | once, after the client attaches |
| `FocusChanged` | `window`, `previous`: window numbers, nil when no window | the focused window changes |
| `BandChanged` | `band`, `previous`: band numbers | the viewed band changes |
| `WindowOpened` | `window`, `band` | a window appears in the layout |
| `WindowClosed` | `window`, `band` | a window leaves the layout |
| `LayoutChanged` | empty | the bands, columns, windows, column widths or floating boxes change |
| `TerminalResized` | `cols`, `rows` | the client's terminal changes size |
| `ConfigReloaded` | empty | a reload succeeded, to the new configuration's handlers |
| `KeyTableChanged` | `table`, `previous` | the active key table changes |
| `HighlightChanged` | `group` | a group's settings change after loading |
| `ColorschemeChanged` | `name`, `previous` | a colorscheme loads after loading |
| `ServerEvent` | `name`, `data`, `queued`, `time` | the server emits an event to this client |
| `WindowStateChanged` | `window`, `key`, `value`, `previous` | the server changes a window's state |

`Attached` runs once the client has received the session's layout, its window states and the server's requirements, before the client handles its first key.
Attaching to a session emits no `WindowOpened`, `LayoutChanged`, `FocusChanged`, `BandChanged` or `WindowStateChanged` for what is already there.
The server's events are in "Server events".
A layout that opens or closes a window emits `LayoutChanged` after its `WindowOpened` and `WindowClosed` events.

A handler can dispatch actions, call `gband.spawn` and `gband.keymap.enter`.
The actions run after the handler returns, and the events they cause are emitted in turn.
After ten rounds of events caused by handlers, gband drops the next round and logs a warning.

```lua
local group = gband.augroup("follow")
gband.on("WindowOpened", function(event)
  print("opened window", event.window, "in band", event.band)
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
Entering a table with no binding is an error, except `root`: `gband.keymap.enter("root")` always works, and returns to interactive mode.
In a table that is not a mode, the next key ends the sequence: a bound key runs its binding, and any other key is discarded.

```lua
gband.keymap.set("move", "h", gband.action.focus_column_left, { desc = "left" })
gband.keymap.set("move", "l", gband.action.focus_column_right, { desc = "right" })
gband.keymap.set("prefix", "m", function() gband.keymap.enter("move") end, { desc = "move" })
```

`gband.bind` and `gband.unbind` still work: `gband.bind("alt+h", ...)` binds in `root`, and `gband.bind("prefix h", ...)` binds in `prefix`.

Bindings are made only while the configuration loads.

### Modes

`gband.keymap.mode(table, opts)`, while the configuration loads, makes `table` a mode.
`opts` is nil or a table whose `label` is nil or a non-empty string naming the mode for display.
Declaring a mode again replaces its label, and `gband.keymap.set` and `gband.keymap.del` on its table keep the declaration.
`root` cannot be a mode.

While a mode is active, a bound key runs its binding, any other key is discarded, and neither reaches a window or a plugin window.
The mode stays active after each key and emits no `KeyTableChanged`, until a binding enters another table.
The default configuration makes `prefix` a mode labelled `navigation`, and leaves it with Escape, Enter, `n`, `?` and the prefix key.
A mode of your own:

```lua
gband.keymap.mode("resize", { label = "RESIZE" })
gband.keymap.set("resize", "=", gband.action.grow_column_width)
gband.keymap.set("resize", "-", gband.action.shrink_column_width)
gband.keymap.set("resize", "escape", function() gband.keymap.enter("root") end, { desc = "done" })
gband.keymap.set("root", "alt+r", function() gband.keymap.enter("resize") end, { desc = "resize" })
```

`gband.keymap.label(table)` returns the label of a mode, or `table` itself for a mode without one and for any other table.
It works while loading and in any callback.

### Running a binding

`gband.keymap.run(table, key)`, inside a callback, runs the binding of `key` in `table` as pressing it in that table would, and returns `true`, or `false` when the key is unbound there.
`key` is written as `gband.keymap.set` takes it, `prefix` included.
An action binding is dispatched, and a function binding, or a registered action, runs at once under its own plugin, its dispatches joining the caller's.
It sends nothing to any window and changes the active table only through the binding's own `gband.keymap.enter`.

```lua
gband.keymap.set("root", "alt+n", function()
  gband.keymap.run("prefix", "n")
end, { desc = "the navigation n" })
```

## Layout and view: `gband.layout`, `gband.view`

`gband.layout()` returns a new table describing the layout the client holds:

- `cols` and `rows`: the size of the screen area.
- `bands`: one table per band, in order, each with `id`, `columns` and `floating`.
- `columns`: one table per column, left to right, each with `width` as a number, `full_width` and `windows`.
- `windows`: one table per window, top to bottom, each with `id` and either `rows`, a fixed height, or `weight`, an automatic height's weight. A window that shows a tiled plugin window this client opened also has `plugin_window`.
- `floating`: one table per floating window, in the band's floating order, each with `id`, `width` and `full_width` as a column has them, `rows`, the box's height, and `col` and `row`, the box's top-left cell as placed in the screen area. A window that shows a plugin window this client opened also has `plugin_window`.

`gband.view()` returns `band`, the viewed band, `window`, the focused window or nil, `floating`, true while the band's floating layer has focus, `plugin_window`, the focused plugin window or nil, `table`, the active key table, and `cols` and `rows`, the size of the ribbon.

Both can be called from any code that runs after loading.
They describe the state when they are called: an action the running callback dispatched takes effect only after it returns.

```lua
gband.keymap.set("prefix", "o", function()
  local view = gband.view()
  local count = 0
  for _, band in ipairs(gband.layout().bands) do
    for _, column in ipairs(band.columns) do
      count = count + #column.windows
    end
  end
  print(("band %d, window %s, %d windows"):format(view.band, tostring(view.window), count))
end, { desc = "describe the layout" })
```

## Action targets

A built-in action that acts on the focused window also takes a target, a table naming the window: `close_window`, `consume_or_expel_left`, `consume_or_expel_right`, `move_column_left`, `move_column_right`, `move_window_down`, `move_window_up`, `cycle_column_width`, `toggle_full_width`, `grow_column_width`, `shrink_column_width`, `grow_window_height`, `shrink_window_height` and `reset_window_height`.
`gband.action.close_window({ window = 3 })` closes window 3, whichever window is focused.

`open_window` takes `band`, `after` or both: the new column follows `after`'s column, or is `band`'s first column when only `band` is given.
`after` must be a tiled window.
`floating = true` opens the window floating instead, in `band` or in the viewed band, centred at the default width and two height steps shorter than the screen; it cannot be given with `after`.

`toggle_window_floating` takes `window` and an optional `after`, a tiled window of the same band.
A floating window is tiled again as a new column after `after`, or, without it, after the tiled window this client focused last in that band, or as the band's first column.
`after` is ignored when `window` is tiled.
`send_prefix` takes `window` and sends the prefix key to it.
`gband.spawn` takes `band` and `after` beside `cmd`.

A target on a view action or on `detach`, a field the action does not take, or a window or band not in the layout is an error at the line of the call.

```lua
gband.keymap.set("prefix", "N", function()
  local view = gband.view()
  gband.action.open_window({ band = view.band })
end, { desc = "open a window as the band's first column" })
```

## Windows and bands: `gband.window`, `gband.band`

These dispatch like actions, in the order they are called together with the callback's actions, and are errors outside a callback.
A window or band number not in the layout is an error at the line of the call.

- `gband.window.focus(window)` views the window's band and focuses the window.
- `gband.band.view(band)` views a band, focusing the window last focused there.
- `gband.window.set_width(window, width)` sets the width of the window's column, or of its box when it floats, a number greater than 0 and at most 10000, and turns full width off.
- `gband.window.set_height(window, { rows = n })` gives a fixed height; `gband.window.set_height(window, { weight = w })` gives an automatic height of weight `w`.
- `gband.window.set_position(window, { col = c, row = r })` places a floating window's box with its top-left cell at `c` and `r`, kept inside the screen area. A window that is not floating is an error.
- `gband.window.send_keys(window, keys)` sends a key name, or a list of them, as key presses.
- `gband.window.send_text(window, text)` sends each character as a key press, a line feed or carriage return as Enter and a tab as Tab. Other control characters are an error.
- `gband.window.paste(window, text)` sends `text` as a paste.

Keys sent this way run no binding and leave the active key table alone.

```lua
gband.keymap.set("prefix", "e", function()
  local view = gband.view()
  local first = gband.layout().bands[1].columns[1].windows[1].id
  if first ~= view.window then
    gband.window.send_text(first, "make\n")
  end
end, { desc = "run make in the first window" })
```

## Plugin windows: `gband.win`

A plugin window shows lines of styled text that Lua writes.
It is one of two kinds:

- a **floating** plugin window, of kind `"floating"`, drawn by this client only, over the ribbon, at a position and size in cells. Floating plugin windows change no layout, view or window, and other clients never see them.
- a **tiled** plugin window, of kind `"tiled"`, shown in a window of the shared layout that runs no program. It is placed, resized, moved, focused and closed like any window, and every client sees its contents. The server drops keys and pastes sent to it, and it closes when the client that opened it detaches, disconnects or reloads.

`gband.win.open(opts)` opens a plugin window and returns its number, which is never reused.
`info` and `list` can be called from any code that runs after loading; every other function only inside a callback.

| option | kinds | value | default |
|---|---|---|---|
| `kind` | both | `"floating"` or `"tiled"` | `"floating"` |
| `lines` | both | the lines | none |
| `focus` | both | boolean | `true` |
| `cursorline` | both | boolean | `false` |
| `keys` | both | key names to functions | none |
| `on_close`, `on_resize` | both | function | none |
| `row`, `col` | floating | integer of at least 0, or `"center"` | `"center"` |
| `width`, `height` | floating | integer of at least 1 | half the ribbon |
| `border` | floating | boolean | `true` |
| `title` | floating | string | none |
| `band`, `after` | tiled | as the `open_window` target takes them | the viewed band and focused window |
| `column_width` | tiled | a width | the server's `default_column_width` |

A line is a string or a list of spans; a span is a string or `{ text = ..., hl = "Group" }`.
Spans without `hl` use `PluginWindow`, and a span's style is its group's resolved style over `PluginWindow`'s.
Control characters are removed, and a line longer than the plugin window is cut.

- `gband.win.set_lines(win, lines)` replaces the lines.
- `gband.win.scroll(win, count)` moves the first shown line, and `gband.win.set_cursor(win, line)` the cursor line. With `cursorline`, the plugin window scrolls just enough to keep the cursor line shown and draws it in `PluginWindowCursorLine`.
- `gband.win.focus(win)` focuses a floating plugin window, or the window of a tiled plugin window.
- `gband.win.set_config(win, config)` changes a floating plugin window's `row`, `col`, `width`, `height`, `border` or `title`.
- `gband.win.close(win)` closes a plugin window, and for a tiled one its window too. Closing a plugin window that is not open does nothing.
- `gband.win.info(win)` returns `id`, `kind`, `focused`, `window`, `top`, `cursor`, `line_count`, `cols` and `rows`, and for a floating plugin window `row`, `col`, `width` and `height`.
- `gband.win.list()` returns the open plugin windows' numbers in ascending order.

The focused plugin window is the focused floating plugin window, otherwise the plugin window shown in the focused window.
A newly opened floating plugin window with `focus` takes focus; moving focus or viewing another band leaves no floating plugin window focused.
The exception is a focus change caused by the floating plugin window's own `keys` function: it stays focused until the next key press, whether the change happens at once or arrives later from the server, as an opened window's focus does.
So a list can run actions on Enter and stay open for the next choice.
While a plugin window is focused, every key the bindings leave unused goes to it and never to a program, and pastes are discarded.
A key in `keys` runs its function with the plugin window's number.
Without an entry, Up or `k` and Down or `j` move the cursor line or scroll, PageUp and PageDown scroll a page, Home and End show the first or last line, and `q` and Escape close a floating plugin window.
The `close_window` action, such as Ctrl+Space then `q`, closes the focused floating plugin window as `gband.win.close` does, running its `on_close` and sending nothing to the server, also on a band with no window; with no floating plugin window focused, it closes the focused window.
A `close_window` dispatched with a target closes the window it names and leaves the floating plugin window open.

`on_close` runs once with the plugin window's number when the plugin window closes, but not when a reload closes it.
`on_resize` runs with the number and the new columns and rows when the plugin window's content area changes size, including when a tiled plugin window's size first becomes known.
A plugin window closes when its plugin is marked failed.

```lua
gband.keymap.set("prefix", "K", function()
  local lines = {}
  for _, binding in ipairs(gband.keymap.list("prefix")) do
    lines[#lines + 1] = {
      { text = ("%-10s"):format(binding.key), hl = "KeyHintKey" },
      binding.desc or binding.action or "function",
    }
  end
  gband.keymap.enter("root")
  local title = gband.keymap.label("prefix") .. " keys"
  gband.win.open({ title = title, width = 50, height = 15, cursorline = true, lines = lines })
end, { desc = "list the prefix keys" })

gband.keymap.set("prefix", "P", function()
  gband.win.open({ kind = "tiled", column_width = 1/3, lines = { "notes", "", "- write the tests" } })
end, { desc = "open a notes window" })
```

## Status line: `gband.ui.statusline`

The status line takes `statusline_height` rows of the client's terminal, 1 by default.
`statusline_position` places it: `"bottom"`, the default, `"top"` or `"off"`.
The ribbon gets the rows the status line leaves, and gband reports that size to programs, so `tput lines` counts only the ribbon.
The status line is not drawn when the terminal has no more rows than `statusline_height`.
A reload that moves or removes the status line resizes the ribbon like a terminal resize.

The status line is only a container.
Everything on it is a component, and the segments gband bundles are plugins written with the same API as yours.

### Components

`gband.ui.statusline.add(spec)` adds a component and returns its full id:

| field | value | default |
|---|---|---|
| `id` | a non-empty string | the plugin's name |
| `render` | a function | required |
| `align` | `"left"`, `"center"` or `"right"` | `"left"` |
| `priority` | a number; the lowest is dropped first when the line is too wide | `0` |
| `order` | a number; components in a region are placed in ascending order | `0` |
| `hl` | a highlight group name | `"StatusLineSegment"` |
| `redraw_on` | a list of built-in event names and `"User"` | empty |
| `redraw_interval` | an integer number of milliseconds, at least 100 | none |
| `fill` | a boolean; a fill component renders again when the width left to it changes | `false` |

Component ids are namespaced like action names.
In the plugin `window`, `id = "count"` gives `window.count`, and a plugin adding one component can omit `id` to get `window`.
Code that belongs to no plugin must give `id`.
An invalid field, an unknown event name or an id already taken is an error at the line of the call.

`gband.ui.statusline.remove(id)` removes a component and returns `true`, or returns `false` when no component has that id.
`gband.ui.statusline.list()` returns `{ id, align, priority, order, hl, fill, plugin, enabled }` for each component, in byte order of ids.
The tables are copies.

All three can be called while the configuration loads and in any callback.

### Render output and context

`render(ctx)` returns:

- nil, an empty string, or only empty spans, to hide the component; it then takes no cell and no separator
- a string, drawn in the component's `hl` group
- a list of strings and spans `{ text = "...", hl = "Group" }`; a span without `hl` uses the component's group

gband removes control characters from the text, so a component cannot write escape sequences.

`ctx` is a new table for each call:

| field | value |
|---|---|
| `id` | the component's full id |
| `side` | `"client"` |
| `total_width` | the status line's width in cells |
| `width` | the cells left for this component, given the other components' latest output, separators and gaps; a hint, at least 0 |
| `table` | the active key table |
| `band` | `{ number, index, count }`: the viewed band's number, its position from the top counting from 1, and the number of bands |
| `column` | `{ index, count }`: the focused column's position counting from 1, and the band's column count; nil when the band is empty or a floating window is focused |
| `window` | the focused window's number, or nil |
| `windows` | every window of the layout, bands from the top, columns from the left and windows from the top, then within each band its floating windows, each `{ window, band, state }`, `state` a copy of its window state |

`gband.ui.width(text)` returns the cells `text` takes: two for a wide character, zero for a zero-width one.
`gband.ui.truncate(text, width)` returns `text` when it fits in `width` cells, and otherwise the longest prefix that fits in `width - 1` cells followed by `…`.
Both remove control characters first.

### When components render

gband calls an enabled component's `render` only while the status line is drawn, and only:

- once when the client attaches, and once after each successful reload
- once when the component is added after loading
- once each time an event its `redraw_on` names is emitted, after that event's handlers
- once each `redraw_interval` milliseconds
- once when the terminal's width changes, and once when the status line starts being drawn
- once on each `HighlightChanged` and `ColorschemeChanged`
- for a fill component, once more after any of these, when the width left to it differs from the `width` of its latest call

When one trigger renders several components, the others render first and the fill components after them, in descending `priority`, then in the order they were added.
gband then lays the line out, and renders once more each enabled fill component whose `width` has changed, in the same order.
Those renders trigger no further render.
A component that fits its output to `ctx.width`, like the hints segment, sets `fill = true` so that width is never stale.

Frames, animations included, draw the latest output and never call `render`.
Keep `render` cheap anyway: it runs in the client's event loop.

### Layout

Components sit in the left, center or right region, by `order` and then in the order they were added.
The value of the `statusline_separator` option, `" │ "` by default, goes between adjacent components of a region, and regions are at least one cell apart.
The left region starts at the first column, the right region ends at the last, and the center region is centered, moved as little as keeps it clear of the other two.

When the line is too wide, gband drops the component with the lowest `priority`, the one added last on a tie, until the line fits or one component is left.
That last one is cut to fit with `…`.
Dropping only hides a component for that layout; it stays enabled.

### Errors

Each `render` call runs protected, as a callback of the component's plugin, with its own instruction budget.
A loop in one component stops only that call: the other components and the code that triggered the render go on.
When `render` raises an error, returns something else than the forms above, or hits the instruction limit, gband reports a plugin error and disables the component until the next load.
Hitting the limit also marks the plugin failed, which hides all its components.

The status line's error item shows the latest configuration or plugin error, the server's included, first in the left region, in `StatusLineError`.
It is dropped last and cut like any last component.
It stays until the configuration next loads without errors.

### Bundled segments

gband bundles five segment plugins:

| module | plugin | shows | redraws on | align | priority | order | group |
|---|---|---|---|---|---|---|---|
| `gband.statusline.band` | `band` | `band ` and the viewed band's index | `BandChanged`, `LayoutChanged` | left | 20 | 10 | `StatusLineSegment` |
| `gband.statusline.mode` | `mode` | the active key table's label, as `gband.keymap.label` gives it; hidden in `root` | `KeyTableChanged` | left | 30 | 20 | `StatusLineAccent` |
| `gband.statusline.hints` | `hints` | the keys of the active key table and what each does, below | `KeyTableChanged`, and as a fill component | left | 0 | 30 | `KeyHintLabel` |
| `gband.statusline.position` | `position` | the focused column and the column count, such as `3/7`; hidden in an empty band and while a floating window is focused | `FocusChanged`, `BandChanged`, `LayoutChanged` | right | 10 | 10 | `StatusLineMuted` |
| `gband.statusline.clock` | `clock` | the local time, `os.date(opts.format)`, `"%H:%M"` by default | every `opts.interval` milliseconds, 1000 by default | right | 5 | 20 | `StatusLineMuted` |

Each takes the options `align`, `priority` and `order`, and each but `hints` takes `hl`.
An option of the wrong type makes `setup` fail.
The default configuration sets up `band`, `mode`, `hints` and `position`, in that order.
A `user/init.lua` replaces the default configuration, so it gets the segments only with the same calls:

```lua
gband.plugin("gband.statusline.band")
gband.plugin("gband.statusline.mode", { align = "right", order = 1 })
gband.plugin("gband.statusline.hints")
gband.plugin("gband.statusline.position")
gband.plugin("gband.statusline.clock", { format = "%H:%M:%S" })
```

Without them the status line is drawn empty.
Set `statusline_position = "off"` to give the ribbon the whole terminal.

### The hints segment

`hints` shows one hint per binding of the active key table, in the order `gband.keymap.list` gives them: the key in `KeyHintKey`, a space, and a label in `KeyHintLabel`, with two spaces between hints.
In `root` it starts with the prefix key labelled with `gband.keymap.label("prefix")`, when the `prefix` table holds a binding, so the default line shows `C-space navigation`, and navigation mode's hints after Ctrl+Space.
A `prefix` table that is not a mode shows `C-space prefix`.

Keys are shown short: `C-`, `A-` and `S-` for Ctrl, Alt and Shift, in that order, named keys in lowercase, `escape` as `esc`, and `shift` with a lowercase letter as the uppercase letter.
`ctrl+space` shows as `C-space`, `shift+d` as `D` and `Alt+PageUp` as `A-pageup`.
The key `prefix` in another table shows as the key the `prefix` option names.

A hint's label is the first of these that applies:

1. the `labels` option's string for the binding's action; an action that `labels` maps to `false` takes no hint
2. the binding's own `desc`, when it is not empty and differs from its action's description
3. a built-in action's short label, below
4. a registered action's description, when it is not empty
5. the action's full name, such as `hello.greet`

A binding to a function shows its `desc`, and takes no hint without one.

| action | short label | action | short label |
|---|---|---|---|
| `focus_column_left` | `left` | `move_window_down` | `move down` |
| `focus_column_right` | `right` | `move_window_up` | `move up` |
| `focus_window_down` | `down` | `toggle_window_floating` | `float` |
| `focus_window_up` | `up` | `switch_focus_floating_tiled` | `layer` |
| `focus_band_down` | `band down` | `cycle_column_width` | `width` |
| `focus_band_up` | `band up` | `toggle_full_width` | `full` |
| `center_column` | `center` | `grow_column_width` | `wider` |
| `open_window` | `new` | `shrink_column_width` | `narrower` |
| `close_window` | `close` | `grow_window_height` | `taller` |
| `consume_or_expel_left` | `stack left` | `shrink_window_height` | `shorter` |
| `consume_or_expel_right` | `stack right` | `reset_window_height` | `reset height` |
| `move_column_left` | `move left` | `detach` | `detach` |
| `move_column_right` | `move right` | `send_prefix` | `send prefix` |


The segment fits itself to its `ctx.width`: it shows the leading hints that fit, followed by ` …` when some are left out, and hides itself when not even the first fits.
It is a fill component, so that width follows the other segments of the same render.

Its options, besides `align`, `priority` and `order`:

| option | value | default |
|---|---|---|
| `labels` | a table from action names to a label string, or `false` to leave the action out | `{}` |
| `root` | `false` hides the segment while `root` is active | `true` |

```lua
gband.plugin("gband.statusline.hints", { labels = { close_window = "kill", detach = false } })
```

Its groups are `KeyHintKey`, linked to `StatusLineAccent` by default, and `KeyHintLabel`, linked to `StatusLineSegment`.
A colorscheme or `gband.hl.set` can style them like any group.

### A component

```lua
local M = { name = "window", api = 1 }

function M.setup()
  gband.hl.default("WindowSegment", { link = "StatusLineAccent" })
  gband.ui.statusline.add({
    align = "right",
    hl = "WindowSegment",
    redraw_on = { "FocusChanged" },
    render = function(ctx)
      return ctx.window and ("window " .. ctx.window)
    end,
  })
end

return M
```

## The key list: `gband.keylist`

gband bundles one more client plugin beside the status line segments: the key list, module `gband.keylist`, plugin `keylist`.
Its `setup` takes no options, and registers the action `keylist.open`, described as `list the keys`.
The default configuration sets it up before its key bindings and binds Ctrl+Space then `?` to it:

```lua
gband.plugin("gband.keylist")
gband.keymap.set("prefix", "?", gband.action["keylist.open"], { desc = "list the keys" })
```

`keylist.open` opens a focused floating plugin window titled with `gband.keymap.label("prefix")` and ` keys`, `navigation keys` with the defaults, centred in the ribbon, with its cursor line on the first line.
It enters `root` as it opens, and as it focuses an open list, so the keys that follow reach the list and not navigation mode.
It holds one line per binding of the `prefix` table, in the order `gband.keymap.list("prefix")` gives them when it opens: the key in the hints segment's short form, such as `C-space` for the prefix key, padded to two cells more than the widest key, then the binding's description, its action's description when it has none, its action's name when that is empty too, or `function`.
The plugin window is as wide as its longest line plus its border and as high as its lines plus its border, at most 15 rows, and the ribbon caps both.
Dispatching `keylist.open` while the list is open focuses it and opens no second one.

Enter runs the binding on the cursor line with `gband.keymap.run("prefix", key)`: an action is dispatched with no target, so it acts on the focused window behind the list, and a function runs, as a key bound to it would.
The list stays open and focused, by the rule for a floating plugin window's own `keys`, so you can choose again; a floating plugin window the binding opens takes focus above it.
`close_window` closes the list itself, as it closes any focused floating plugin window.
Enter does nothing on the line of `keylist.open` itself, the one line whose description is drawn in `KeyListMuted`.

Pressing a line's key, matched as a binding's key is, moves the cursor line to that line and runs it exactly as Enter on it does: the list stays open and focused on the same terms, `close_window` closes it, and the line of `keylist.open` only moves the cursor line.
The list keeps its own keys: Up, Down, PageUp, PageDown, Home and End move the cursor line, Enter runs it, and Escape closes the list.
An own key keeps that meaning when a line shows it, so such a line runs only through Enter.
The line of the prefix key also runs only through Enter, because the prefix key enters the `prefix` table before the list receives it.
Every other key a line shows runs that line, even where a floating plugin window gives the key a default: with the defaults `j` and `k` focus the window below and above, and only Up and Down move the cursor line.
A key no line shows keeps its floating plugin window default, so a `prefix` table without `j` leaves `j` moving the cursor line.
`q` runs its line when the `prefix` table binds it, so the default `close_window` closes the list; without a `q` binding, `q` closes the list as it closes any floating plugin window.
Escape and Ctrl+Space then `q` close the list too.

The key list draws keys in `KeyListKey` and the description it cannot run in `KeyListMuted`; it defines them as defaults when its module is first required, and gives `StatusLineAccent` and `StatusLineMuted` their usual defaults so the links resolve without a status line.

## Highlight groups: `gband.hl`

A highlight group is a named style.
Group names start with an ASCII letter and hold letters, digits, `_` and `.`.
They are not namespaced: every plugin, colorscheme and configuration file addresses a group by the same name, so a theme can style any plugin's groups.

A style spec is a table whose fields are all optional:

| field | value |
|---|---|
| `fg`, `bg` | a color |
| `bold`, `italic`, `underline`, `reverse`, `dim` | a boolean |
| `link` | another group's name |

A color is `"#rrggbb"`, a palette index from 0 to 255, or one of `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white` and their `bright_` forms, which are indexes 0 to 15.
An invalid name, field or color, or a link to the group itself, is an error at the line of the call.

Each group holds two settings:

- `gband.hl.set(name, spec)` replaces its explicit setting. `gband.hl.set(name, {})` sets an empty one, which hides the default; `gband.hl.set(name, nil)` removes it, so the default shows again.
- `gband.hl.default(name, spec)` replaces its default setting.

The explicit setting wins whenever it exists, whatever ran first.
A plugin should declare its groups with `gband.hl.default`, so that colorschemes and the user override them.

A group with `link` inherits the linked group's resolved style, and the fields it sets itself win.
Links are followed each time a style is resolved.
A call that would make a link cycle is an error naming the groups.
A cycle that appears later, when another setting changes, is cut where resolution meets a group twice, and gband logs a warning.

`gband.hl.get(name)` returns a copy of the group's definition, or nil.
`gband.hl.get(name, { resolve = true })` returns the resolved style, without `link`, or an empty table.

After loading, each `gband.hl.set` or `gband.hl.default` that changes a group emits `HighlightChanged` with `group`, and every status line component renders again.

The status line defines these groups, as defaults:

| group | default | use |
|---|---|---|
| `StatusLine` | `{ reverse = true }` | every status line cell |
| `StatusLineSegment` | `{}` | components' default group |
| `StatusLineSeparator` | `{ dim = true }` | separators |
| `StatusLineMuted` | `{ dim = true }` | secondary text |
| `StatusLineAccent` | `{ bold = true }` | emphasised text |
| `StatusLineError` | `{ fg = "red", bold = true }` | the error item |

The hints segment adds `KeyHintKey`, `{ link = "StatusLineAccent" }`, and `KeyHintLabel`, `{ link = "StatusLineSegment" }`, when its module is first required.
The key list adds `KeyListKey`, `{ link = "StatusLineAccent" }`, and `KeyListMuted`, `{ link = "StatusLineMuted" }`, the same way.

The plugin window API defines these groups, as defaults:

| group | default | use |
|---|---|---|
| `PluginWindow` | `{}` | every plugin window cell |
| `PluginWindowBorder` | `{ fg = 8 }` | a floating plugin window's border |
| `PluginWindowTitle` | `{ bold = true }` | a floating plugin window's title, over `PluginWindowBorder` |
| `PluginWindowCursorLine` | `{ reverse = true }` | the cursor line |

A change of any group redraws every plugin window.

A span's style is its group's resolved style over `StatusLine`'s, field by field.

### Colors in the terminal

The client draws 24-bit colors when its own `COLORTERM` is `truecolor` or `24bit`.
Otherwise it draws each `"#rrggbb"` color as the nearest of the palette indexes 16 to 255.
Index and named colors are drawn as their palette index either way.
Floating plugin windows follow these rules.
A tiled plugin window's contents are sent to the server as 24-bit colors, and every client shows them as its terminal does, as it shows a program's colors.

## Colorschemes: `gband.colorscheme`

A colorscheme is a Lua file that sets groups with `gband.hl.set`.
`gband.colorscheme(name)` runs `colors/<name>.lua` from the first runtimepath entry that holds one, or the colorscheme of that name bundled with gband.
Names start with an ASCII letter or digit and hold letters, digits, `_` and `-`.

gband bundles `default`, which sets every built-in status line group, and loads it before the init file.
`gband.colorscheme()` returns the active colorscheme's name.

Switching first removes every group's explicit setting, keeping the defaults, then runs the file.
Set your own groups after the `gband.colorscheme` call.
The code in a colorscheme file belongs to no plugin.

A switch is atomic.
It returns `true` and makes `name` active when the file runs to completion.
When the colorscheme is missing, its name is invalid, or its file raises an error or hits the instruction limit, gband restores every explicit setting, keeps the active colorscheme, reports the error as `colors/<name>: ...` and returns `false`.
That error fails neither the load nor any plugin.

After loading, a successful switch emits `ColorschemeChanged` with `name` and `previous`, once, and no `HighlightChanged` for the groups the file sets.

```lua
-- colors/dusk.lua
gband.hl.set("StatusLine", { fg = "#e0def4", bg = "#232136" })
gband.hl.set("StatusLineAccent", { fg = "#c4a7e7", bold = true })
gband.hl.set("WindowSegment", { fg = "#f6c177", bold = true })
```

## Plain data

Everything that crosses between the sides is plain data: event data, window state values, command arguments and command results.
A plain data value is nil, a boolean, an integer, a floating-point number, a string, or a table whose keys are strings or integers and whose values are plain data.

- A string passes as its bytes, unchanged, so `"\0\255"` arrives as those two bytes.
- Integer keys and string keys stay apart: `{ [1] = "a", ["1"] = "b" }` arrives with both keys.
- A table's metatable is ignored, and its raw contents are used.
- A table must not hold itself, directly or through other tables, and tables nest at most 32 deep.
- One value encodes to at most 1 MiB.
- Functions, userdata and threads are not plain data.

Every API that takes plain data checks it before sending anything.
A value that is not plain data is an error at the line of the call, naming the path to the offending value:

```
server.lua:7: `data.cb.run` is a function, which is not plain data
```

Every plain data value a side receives is a new table or value, unconnected to any other.

## The server API

The server's Lua runs on a thread of its own, fed by the sessions.
One Lua state serves every session the server hosts; events name their session.

### Server events

`gband.on`, `gband.augroup` and their options work in the server as they do in a client, with these events and no `User` event:

| event | payload | emitted when |
|---|---|---|
| `SessionCreated` | `session` | a session is created |
| `SessionEnded` | `session` | a session is removed |
| `WindowOpened` | `session`, `window`, `band` | a window enters a session's layout |
| `WindowClosed` | `session`, `window`, `band`: the band it left | a window leaves a session's layout |
| `WindowExited` | `session`, `window`, `code`: the exit code, nil when a signal ended it, `signal`: the signal number, nil otherwise | a window's program exits |
| `WindowOutput` | `session`, `window`, `data`: a string of the bytes read | the window's program writes output |
| `WindowInput` | `session`, `window`, `client`: the client's number | a key or paste from a client is written to the window |
| `ClientAttached` | `session`, `client` | a client attaches to a session |
| `ClientDetached` | `session`, `client` | a client detaches or disconnects |
| `ConfigReloaded` | empty | a reload succeeded, to the new configuration's handlers |

A session's events reach the handlers in the order the session applied the changes: a window's `WindowExited` comes before its `WindowClosed`, and its last `WindowOutput` before both.
`WindowInput` says that input happened, never what it was.

`WindowOutput` delivers the raw bytes in chunks whose boundaries are arbitrary, so a prompt can be split between two calls.
Keep a short tail per window and match against it, as the agent status sample does.
Joined in order, a window's `data` equals what its program wrote, except when the handlers fall behind:

- No handler ever slows a window, a screen or a session.
- When more than 4 MiB of output waits for the handlers, the server drops further output until they catch up, and logs how many bytes it dropped.
- When more than 1024 other events wait, the server drops the oldest and logs how many.
- Without a `WindowOutput` handler, the server copies no output for Lua at all.

Naming an event of the other side, such as `gband.on("WindowOutput", fn)` in a client, is an error naming the event and its side.

### Emitting events: `gband.emit`

In the server, `gband.emit(name, data, opts)` sends an event to clients instead of running `User` handlers.

- `name` is a non-empty string, and `data` is plain data.
- `opts.session`, optional, names the only session whose clients receive the event; without it every attached client of every session does.
- Each event carries the time it was emitted, in milliseconds since the Unix epoch.
- `gband.emit` can be called only in a callback.

When no client the event targets is attached, the server queues it, up to 256 events, dropping the oldest and logging each drop.
When a client attaches, it receives every queued event that targets its session or every session, in the order emitted and marked as queued, and those events leave the queue.
So an event emitted while you are detached reaches you once, when you next attach.

### Window state: `gband.window_state`

Every window has a state: a map from non-empty string keys to plain data values, empty when the window opens.

In the server, `gband.window_state(session, window)` returns a table through which that state is read and written, or nil when the session holds no such window:

```lua
local state = gband.window_state(ev.session, ev.window)
state.agent = "waiting"
print(state.agent)
for key, value in pairs(state) do print(key, value) end
state.agent = nil
```

- Reading a key returns a copy of its value.
- Assigning a plain data value sets the key, and assigning nil removes it.
- Assigning a value equal to the current one changes nothing and sends nothing.
- A key that is not a non-empty string, a value that is not plain data, or an assignment that would bring the state's encoded size over 64 KiB is an error at the line of the assignment, and leaves the state unchanged.

The server sends every change to the clients of the window's session, in the order made, and a full copy of the session's states to each client that attaches.
A window's state is removed when it leaves the layout.
States live in the server, outside the Lua state, so a reload keeps them.

Keys are not namespaced, so plugins can read each other's keys.
Prefix a key you do not mean to share with your plugin's name.

### Server commands

`gband.cmd.register`, `gband.cmd.run` and `gband.cmd.list` work in the server as in a client, but a server command's function is called with two arguments: the arguments table, and a context.

| field | value |
|---|---|
| `ctx.session` | the calling client's session name, or nil |
| `ctx.client` | the calling client's number, or nil |
| `ctx.focus(window)` | tells the calling client, and no other, to focus that window of its session; does nothing without a calling client |

The function's first return value is the command's result, sent back to the caller, and must be plain data.
A result that is not plain data fails the call as if the function raised an error.
`gband.cmd.run` in the server calls a command with no calling client.

```lua
gband.cmd.register("next_waiting", function(args, ctx)
  local window = find_waiting(ctx.session, args.after)
  if window then
    ctx.focus(window)
  end
  return window
end)
```

### Session actions in the server

The server's `gband.action` holds the session actions: `open_window`, `close_window`, `consume_or_expel_left`, `consume_or_expel_right`, `move_column_left`, `move_column_right`, `move_window_down`, `move_window_up`, `toggle_window_floating`, `cycle_column_width`, `toggle_full_width`, `grow_column_width`, `shrink_column_width`, `grow_window_height`, `shrink_window_height` and `reset_window_height`.
Each takes one target table naming `session` and, in `window`, the window it acts on:

```lua
gband.action.close_window({ session = ev.session, window = ev.window })
```

`open_window` takes `session`, `band`, an optional `after` window, an optional `program`, a command line string or a list of argument strings, and an optional `floating`.
`floating = true` opens the window floating in `band`, and cannot be given with `after`.
`toggle_window_floating` also takes an optional `after` window: a floating window is tiled after it, or as the band's first column without it, since the server knows no client's focus.

```lua
gband.action.open_window({ session = "work", band = 1, after = 2, program = { "htop", "-d", "10" } })
```

A missing session, a wrong type or an unknown field is an error at the line of the call.
The actions apply to their sessions after the callback returns, in the order called, and tell no client to focus anything.
An action naming a session, window or band that no longer exists changes nothing.
They can be called only in a callback.

### Reading structure: `gband.sessions`, `gband.session`

`gband.sessions()` returns the session names in ascending byte order.
`gband.session(name)` returns `{ name, bands, clients }`, or nil for an unknown session:

- `bands` lists the bands from the top, each `{ band, columns, floating }`
- each column is `{ width, full_width, windows }`, from the left
- each window is `{ window }`, from the top
- each floating window is `{ window, width, full_width, rows, col, row }`, in the band's floating order, with `col` and `row` the box's top-left cell as placed in the session's screen area
- `clients` lists the numbers of the attached clients, ascending

Each call returns new tables.
Focus and the view are client state: the server knows neither.

## The client side of the bridge

### Server events: `ServerEvent`

A client delivers each event the server sends as the built-in event `ServerEvent`, with the payload `{ name, data, queued, time }`: the name and data given to `gband.emit`, whether the event waited in the server's queue, and its emission time in milliseconds since the Unix epoch.
`opts.pattern` runs a handler only for events of that name:

```lua
gband.on("ServerEvent", function(ev)
  gband.notify("Agent waiting: " .. ev.data.title)
end, { pattern = "agent.waiting" })
```

### Window state in the client

`gband.window_state(window)` returns a new table holding a copy of the window's latest state, an empty table for a window of the layout without state, and nil for a window not in the layout.
Changing the returned table changes nothing else.

Each change after attaching emits `WindowStateChanged` with `window`, `key`, `value`, the new value or nil when removed, and `previous`.
The states a client receives as it attaches emit nothing: they are already there in its `Attached` handlers.

The status line's render context lists every window with its state in `ctx.windows`, so a component can count across windows:

```lua
gband.ui.statusline.add({
  id = "waiting",
  redraw_on = { "WindowStateChanged" },
  render = function(ctx)
    local waiting = 0
    for _, entry in ipairs(ctx.windows) do
      if entry.state.agent == "waiting" then waiting = waiting + 1 end
    end
    return waiting > 0 and ("waiting " .. waiting) or nil
  end,
})
```

### Calling server commands: `gband.rpc`

`gband.rpc(name, args, callback)` asks the server to run the server command `name`, its full name, with `args`, a plain data table or nil.
It returns at once, before the command runs.
When the command returns, the client calls `callback(true, result)`.
When the name is unknown, the command is disabled, or it raises an error or returns something that is not plain data, the client calls `callback(false, message)`.
The callback is optional, runs as a callback of the plugin that called `gband.rpc`, and never runs when the connection closes first.
The server runs each client's commands in the order it calls them.
`gband.rpc` can be called only in a callback.

```lua
gband.keymap.set("prefix", "a", function()
  gband.rpc("agent-status.next_waiting", { after = gband.view().window }, function(ok, result)
    if not ok then print(result) end
  end)
end, { desc = "focus the next waiting agent" })
```

## Local actions

These act on the machine and terminal the client runs on, and can be called only in a callback.

- `gband.notify(text, opts)` asks the terminal for a desktop notification of `text`, with `opts.title`, optional. The client option `notify_style` chooses how: `"osc9"`, the default, writes `ESC ] 9 ; text BEL` and leaves the title out; `"osc777"` writes `ESC ] 777 ; notify ; title ; text BEL`, with the title `gband` when none is given; `"bell"` rings the bell; `"none"` does nothing. Control characters, and `;` in the title, are removed, and each is cut to 1024 bytes.
- `gband.bell()` rings the terminal's bell.
- `gband.clipboard(text)` sets the terminal's clipboard to `text`, at most 1 MiB, through `ESC ] 52`.
- `gband.open(target)` opens a URL or a path with the machine's opener, `xdg-open`, or `open` on macOS, with `target` as its only argument and without a shell. It returns `true` when the opener started, and `false` when it could not, logging the reason. It does not wait for the opener.

The client writes these sequences between frames, never inside one.
Whether a notification appears depends on the terminal: many show OSC 9 or OSC 777, and some need a setting for OSC 52.

## Requirements

A server plugin with a client half can say which client half it needs, in its manifest's `client` field.
A requirement is one or more comparisons separated by commas, each `>=`, `>`, `<=`, `<` or `=` followed by a version, such as `">= 0.2, < 1"`.

After a client attaches, the server sends it the name and requirement of each plugin whose manifest holds `client` and whose `server.lua` did not fail.
The client compares them with its own manifests, and reports a configuration error for each plugin it does not have, or whose version does not meet the requirement, naming the plugin, the requirement and the version it holds.
The session works as it does without the check.

## Trust

The server never sends code.
A client runs only the Lua files of its own machine, and treats every value from a server as data: an event whose data is the string `os.exit(1)` is just that string.
Attaching to a server, including a remote one, cannot run code in the client.
The one other Lua a client runs is a chunk from a test runner that its own environment named with `GBAND_TEST_SOCKET`, as [the testing guide](testing.md#the-test-channel) describes; a server never forwards such a chunk.

## Still to come

Later changes will add their own extension points, each with its own API:

- a command palette, which will read `gband.cmd.list()` and `gband.action.list()`
- an SSH transport, so a client can attach to a server on another machine; plugins written to this guide need no change for it
