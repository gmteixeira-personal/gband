# Writing a gband plugin

A gband plugin is a directory of Lua files.
gband finds it on the runtimepath, reads its manifest, runs the file of its side, and lets your configuration set it up.
Plugins can add actions, commands, options, key bindings, event handlers, side bars, plugin windows, highlight groups and colorschemes in the client, and event handlers, shared window state, events and commands in the server.

This guide is the reference.
To learn the API by building a plugin step by step, start with [the scripting tutorial](tutorial/README.md).

This guide uses the names that the README's [Vocabulary](../README.md#vocabulary) defines: a band is a circular row of columns, a column holds windows stacked vertically, and the ribbon is the part of the terminal that the bars leave, where the viewed band is drawn.

Much of gband is itself written with this API, to serve as examples.
The API modules `gband.hl`, `gband.palette`, `gband.colorscheme`, `gband.bar`, `gband.win`, `gband.settings` and `gband.keystyle`, the bundled plugins `gband.keylist`, `gband.errors`, `gband.prompt` and `gband.sidebar`, the key styles and the themes are Lua files in [crates/lua/src/runtime/gband/](../crates/lua/src/runtime/gband/).
Every one of them uses only the API this guide describes, and each process writes a copy of them under `defaults/lua/gband/` and `defaults/colors/` of the configuration directory.
Copy one into `user/lua/gband/` or `user/colors/` to change or replace it, as "The prelude and the API modules" describes.

The [sample plugin](../examples/plugins/hello) uses most of what the client API offers.
The [window sample](../examples/plugins/window) adds a side bar, a highlight group and a colorscheme.
The [agent status sample](../examples/plugins/agent-status) has both sides: its server half watches windows, and its client half notifies, counts and jumps.
[Testing a plugin](testing.md) describes `gband test`, which runs a plugin against a real client and server and checks what it draws.

## Two sides

The client and the server are separate processes.
The server may run on another machine than the client, or on the same one; a plugin behaves the same either way.
Each process runs its own Lua state, loaded from the files of the machine it runs on, and the two never share Lua values.

The rule for where code goes:

- What must keep working while no client is attached, or must be shared by every attached client, runs in the **server**: watching window output, tracking which window waits for input, counting, queueing a notification for later.
- What concerns one user's screen, keyboard or machine runs in the **client**: key bindings, side bars, plugin windows, colors, desktop notifications, the clipboard, opening a URL.

The sides talk through plain data only: the server emits events and publishes window state, and a client calls server commands and gets their results.
No code ever crosses the connection, as "Trust" below describes.

`gband.side` is `"client"` in a client and `"server"` in the server, and `"test"` in a test file that [`gband test`](testing.md) runs.
`gband.api_version` is `2` on both.

## Where gband looks

`gband.runtimepath` is a Lua list of directories.
Each process builds it from its own environment.
When the configuration starts to load, it holds:

1. the `user` directory of the configuration directory, `~/.config/gband/user/`
2. every directory directly under the plugins directory, in byte order of their names

`gband.config_dir` holds the configuration directory's path, such as `/home/u/.config/gband`, on the client and the server, and is nil when the process has none or when the default configuration is evaluated alone.
Assigning it changes nothing that gband loads or watches.

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
When no entry holds the module, it looks among the modules bundled with gband, such as `gband.sidebar`, and only then in Lua's own search path.
The first file found wins, so a module in `user/lua/` overrides a plugin's module of the same name, and `user/lua/gband/sidebar.lua` overrides the bundled one.
A bundled module's chunk receives what a module on the runtimepath receives, its name and its path, and `require` returns its value and that path, such as `gband/sidebar.lua`.
Errors in a bundled module name the same path, such as `gband/sidebar.lua:12:`.
The key style presets are bundled modules too, so `user/lua/gband/keystyle/direct.lua` replaces the direct key style wherever `gband.keystyle.use("direct")` runs.
So are the prelude and the API modules, which "The prelude and the API modules" describes.

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

1. gband sets `gband.side` to `"client"` and `gband.api_version` to `2`.
2. gband installs the client's `gband` API, then requires `gband.prelude`, which installs the API modules, their built-in highlight groups and their providers, and loads the start theme: the saved theme, or `default`, as "Themes" describes.
3. gband runs the init file: `user/init.lua` when it exists, the default client configuration otherwise.
4. gband runs every manifest, in runtimepath order.
5. For each plugin whose manifest is valid, in runtimepath order, gband runs its `client.lua`.

In the server:

1. gband sets `gband.side` to `"server"` and `gband.api_version` to `2`.
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
| `bind`, `unbind`, `spawn`, `keymap`, `keystyle`, `settings`, `ui`, `hl`, `colorscheme`, `palette`, `layout`, `view`, `window`, `band`, `win`, `bar`, `errors`, `clear_errors`, `rpc`, `notify`, `bell`, `clipboard`, `open`, `core` | `sessions`, `session` |

Every other field exists on both sides: `on`, `augroup`, `emit`, `cmd`, `opt`, `set`, `plugin`, `plugins`, `runtimepath`, `config_dir`, `side`, `api_version`, `window_state` and `action`.
`emit`, `window_state`, `action` and the events differ between the sides, as their sections describe.
A test file's `gband` holds only `side`, `api_version` and `core`, whose fields are the test primitives [Testing a plugin](testing.md#test-primitives-gbandcore) describes.
Reading any other field names the sides that provide it, such as ``tests/a_spec.lua:4: `gband.opt` is a client and server API; this is the test side``, and so does reading a client primitive such as `gband.core.owner` there.
In the server, `gband.action` holds the session actions only; reading a view or client action, such as `gband.action.focus_column_left`, is the same kind of error.

## Plugin modules and `gband.plugin`

A plugin that takes options exports a module with a `setup` function:

```lua
local M = { name = "hello", api = 2 }

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
An error there is a plugin error: gband records it in the process's log as `<plugin>: <file>:<line>: <message>`, and the client adds it to its error list.
While the sidebar is shown, its last row shows `!`, as "The sidebar" describes.
While it is not, the latest error shows as a banner on the bottom row of the ribbon.

`gband.errors()` returns a new list of the client's errors since the list was last emptied, oldest first.
A load with none of the client's own errors empties it, and so does `gband.clear_errors()`.

`gband.clear_errors()` is callable wherever an action value is, such as a binding function, a command or an event handler; calling it while the configuration loads is an error.
`gband.errors()` returns an empty list right after it, in the same callback, and once the callback returns the sidebar's `!` or the banner goes until the client reports another error.
An error raised in that callback, before or after the call, is listed after the clear.
Clearing changes nothing else:

- it does not reload the configuration;
- a callback that an error disabled stays disabled, and a failed plugin stays failed, until the next load;
- the logs keep every error;
- only this client's list is emptied: other clients keep theirs, and the server still sends its latest error to each client that attaches.

The bundled plugin `gband.errors`, plugin `errors`, registers the action and command `errors.open`, both described as `list the errors`.
They open the list in a plugin window, three quarters of the ribbon wide and half of it high, wrapping each error by display width and leaving an empty line between two, and enter `root` so the keys that follow reach it.
A floating list is titled `errors  c clear` while it holds errors, and `errors` once it shows `no errors`.
Its `setup` takes `kind`, `"floating"` by default or `"tiled"`, and the command takes the same `kind` in its arguments table: `gband.cmd.run("errors.open", { kind = "tiled" })`.
`q` closes the list, and so does Escape when it floats.

The plugin also registers the action and command `errors.clear`, described as `clear the errors`, and `c` in the list, floating or tiled, does the same.
Each calls `gband.clear_errors()` and, when the list is open, empties it so it shows `no errors`, keeping it open and focused.
A binding still wins over `c`, so Ctrl+Space then `c` centers the column and clears nothing.
A clear made by calling `gband.clear_errors()` directly leaves an open list as it was until it is opened again.

```lua
gband.keymap.set("prefix", "C", gband.action["errors.clear"], { desc = "clear the errors" })
```
The default configuration sets the plugin up and binds no key to it.

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
A run is the init file, one manifest, one side file, one `setup`, one call of a callback, or one colorscheme file.
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

The built-in actions, with the description `gband.action.list()` gives each:

| action | kind | description |
|---|---|---|
| `focus_column_left` | view | focus the column to the left |
| `focus_column_right` | view | focus the column to the right |
| `focus_window_down` | view | focus the window below |
| `focus_window_up` | view | focus the window above |
| `focus_band_down` | view | view the band below |
| `focus_band_up` | view | view the band above |
| `center_column` | view | center the focused column |
| `switch_focus_floating_tiled` | view | switch focus between floating and tiled windows |
| `minimize_window` | view | minimize the focused floating window |
| `open_window` | session | open a window running the user's shell |
| `close_window` | session | close the window |
| `consume_or_expel_left` | session | consume or expel the window to the left |
| `consume_or_expel_right` | session | consume or expel the window to the right |
| `move_column_left` | session | move the column or floating window to the left |
| `move_column_right` | session | move the column or floating window to the right |
| `move_window_down` | session | move the window down |
| `move_window_up` | session | move the window up |
| `toggle_window_floating` | session | float or tile the window |
| `cycle_column_width` | session | cycle the width of the window's column |
| `toggle_full_width` | session | toggle full width of the window's column |
| `grow_column_width` | session | grow the width of the window's column |
| `shrink_column_width` | session | shrink the width of the window's column |
| `grow_window_height` | session | grow the height of the window |
| `shrink_window_height` | session | shrink the height of the window |
| `reset_window_height` | session | reset the height of the window |
| `detach` | client | detach |
| `send_prefix` | client | send the prefix key to the focused window |
| `drag_window` | client | move the window with the mouse |
| `drag_resize_window` | client | resize the window with the mouse |
| `drag_band` | client | slide the band or switch bands with the mouse |

The view and client actions exist in the client only.
The session actions exist on both sides; in the server they take a target, as "Session actions in the server" describes.
The three `drag_` actions act only when a mouse press runs them, as "Mouse names" describes.

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
| client | `prefix`, `center_focused_column`, `loop_bands`, `window_titles`, `animations`, `animation_speed`, `notify_style`, `tile_border_sides`, `tile_border_chars`, `focused_tile_border_chars`, `floating_border_sides`, `floating_border_chars`, `focused_floating_border_chars`, `width_step`, `height_step`, `mouse_mod` |
| server | `default_column_width`, `width_presets` |

```lua
-- user/server.lua
gband.opt.default_column_width = 1/3
local presets = gband.opt.width_presets
```

`loop_bands`, `true` by default, makes bands circular: focus goes round from a band's last column to its first, and a long enough band is drawn as a loop; `false` stops focus at either end.

`window_titles`, `true` by default, draws each window's name on its top border; `false` draws none, while the names are still kept and `gband.layout()` still holds them.
The client reads `GBAND_WINDOW_TITLES` when it starts: `off` draws no names whatever `window_titles` holds, unset or `on` leaves the option in charge, and any other value is logged as a warning and read as unset.

`animations`, `true` by default, animates scrolling, band switches, moving windows and resizes; `false` draws every change at once.
`animation_speed`, `1` by default, scales how fast every animation runs: `2` is twice as fast and `0.5` half as fast, from `0.1` to `10`.
The client reads `GBAND_ANIMATIONS` when it starts: `off` turns animations off whatever `animations` holds, unset or `on` leaves the option in charge, and any other value is logged as a warning and read as unset.
A reload that turns `animations` off draws every moving part at its target on the next frame, and a reload that changes `animation_speed` carries every moving part on from where it is drawn, at the new speed.

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
| `LayoutChanged` | empty | the size of the screen area, the bands, columns, windows, column widths or floating boxes change |
| `TerminalResized` | `cols`, `rows` | the client's terminal changes size |
| `ConfigReloaded` | empty | a reload succeeded, to the new configuration's handlers |
| `KeyTableChanged` | `table`, `previous` | the active key table changes |
| `HighlightChanged` | `group` | a group's settings change after loading |
| `ColorschemeChanged` | `name`, `previous` | a colorscheme loads after loading |
| `ServerEvent` | `name`, `data`, `queued`, `time` | the server emits an event to this client |
| `WindowStateChanged` | `window`, `key`, `value`, `previous` | the server changes a window's state |
| `MousePressed` | `button`, and the mouse fields | a mouse button is pressed |
| `MouseReleased` | `button`, and the mouse fields | a mouse button is released |
| `MouseDragged` | `button`, and the mouse fields | the pointer moves to another cell with a button held |
| `MouseScrolled` | `direction`, and the mouse fields | the wheel turns one step |

`button` is `"left"`, `"middle"` or `"right"`, and `direction` is `"up"`, `"down"`, `"left"` or `"right"`.
The mouse fields are:

- `col`, `row`: the terminal cell under the pointer, from 0.
- `ctrl`, `alt`, `shift`: the modifiers held.
- `target`: `"window"`, `"plugin_window"`, `"ribbon"` or `"outside"`, what the cell shows.
- `window`: the window under the pointer, or the window of a tiled plugin window, else nil.
- `plugin_window`: the plugin window under the pointer, else nil.
- `content_col`, `content_row`: the cell inside the window's border, nil on the border.
- `box_col`, `box_row`: for the targets `"window"` and `"plugin_window"`, the cell inside the target's box, border included, from 0 at the box's top-left cell, else nil. A box the screen cuts counts from its real top-left cell.
- `box_width`, `box_height`: for those targets, the size of the box as drawn, border included, else nil.
- `table`: the key table active when the event arrived.

The mouse events only report: the click, drag or wheel step still reaches the window, plugin window or gesture that takes it.

`Attached` runs once the client has received the session's layout, its window states and the server's requirements, before the client handles its first key.
Attaching to a session emits no `WindowOpened`, `LayoutChanged`, `FocusChanged`, `BandChanged` or `WindowStateChanged` for what is already there.
The server's events are in "Server events".
A layout that opens or closes a window emits `LayoutChanged` after its `WindowOpened` and `WindowClosed` events.
A change of the screen area's size, the `cols` and `rows` of `gband.layout()`, emits `LayoutChanged` too, whichever client's resize, attach or bar change caused it.
This client's own resize runs `TerminalResized` at once, and `LayoutChanged` when the layout with the new screen area arrives.

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

### Mouse names

`leftmouse`, `middlemouse` and `rightmouse`, with `ctrl`, `alt` and `shift`, bind a press of a mouse button in a key table, as a key binds.
`wheelup`, `wheeldown`, `wheelleft` and `wheelright` bind one step of the wheel the same way.
They cannot be the `prefix` option or a key of a plugin window's `keys`.
A press matches the active table: in `root`, a binding that takes the press replaces the interactive defaults, which focus, forward to a program, select or paste.
In a mode or a key sequence it behaves as a key does.
A wheel step that no binding of the active table takes goes to the window under the pointer, as it always did, and leaves the active table as it was, so a stray wheel step after the prefix key does not end the sequence.
While a drag gesture runs, every wheel step goes to the window under the pointer.
After a wheel binding runs, further steps of the same name and modifiers are discarded until 150 ms have passed since its last run, so one flick of a free-spinning wheel or a touchpad runs it once.
The key list leaves mouse bindings out.

`mod` before a mouse name stands for the modifiers of the client option `mouse_mod`, `"alt"` by default, added to any other modifier the name gives.
It is read when the bindings are used, as `prefix` is, so setting the option before or after the binding gives the same result, and `gband.keymap.list` shows the name as written, such as `mod+leftmouse`.
`mouse_mod` takes `ctrl`, `alt` and `shift` joined by `+`, in any order and case, and reads back in lowercase as `ctrl`, `alt`, `shift`.
`mod` before a key that is not a mouse name is an error.
When two bindings of one table match the same mouse event, such as `mod+leftmouse` and `alt+leftmouse`, the one made last runs, so your own `alt+leftmouse` after `gband.keystyle.use()` takes the place of the preset's `mod+leftmouse`.

A function bound to a button runs with one argument, the `MousePressed` payload, and a function bound to a wheel name with the `MouseScrolled` payload.
The drag actions `drag_window`, `drag_resize_window` and `drag_band` start a gesture when a mouse press runs them, from its binding or a function it calls, and do nothing at any other time.
`drag_resize_window` moves the edges nearest the press, by thirds of the box, or the edges its target names, as [Action targets](#action-targets) describes.
The modal and direct key styles bind them to `mod+leftmouse`, `mod+rightmouse` and `mod+middlemouse` in `root` and in `prefix`, and to `leftmouse`, `rightmouse` and `middlemouse` in `prefix`, and bind `mod+wheeldown` and `mod+wheelup` to `focus_band_down` and `focus_band_up` in both tables.
The floating key style binds the same five `mod+` names in `root` only, then `leftmouse` to `desktop.press` and `rightmouse` to `desktop.menu`, and binds no mouse name in `prefix`, as [The desktop](#the-desktop-gbanddesktop) describes.
With no binding, Ctrl and Alt held together select text over a program that asked for the mouse, and Alt alone reaches the program.
`drag_band`, and `drag_window` pressed on empty ribbon, slide the band sideways or switch bands vertically, whichever axis the drag starts on, and never both in one drag.

```lua
gband.keymap.set("root", "ctrl+leftmouse", function(e)
  if e.target == "window" then
    gband.action.drag_window()
  end
end, { desc = "move the window under the pointer" })
```

A function bound to a mouse name, or a registered action bound to one, declines the press or wheel step when it returns `false`.
The event then takes what it would take if the table did not bind the name: the interactive defaults in `root`, a discard in a mode or a key sequence, and the window or plugin window under the pointer for a wheel step.
Any other return value, nil included, takes the event, and so does a built-in action, a function that raises an error and a function whose plugin has failed.
The actions the function dispatched run first and the fallback after them, so a table the function enters with `gband.keymap.enter` is active after the event.
A declined press starts no gesture, so a drag action the function dispatched does nothing, and its drags and release go where they would go with no binding.
A declined wheel step starts no cooldown and keeps a key sequence active.
An earlier binding of the same table that matches the event does not run in its place.
A key binding's return value is still ignored.

```lua
gband.keymap.set("root", "leftmouse", function(e)
  if e.content_col ~= nil then
    return false
  end
  gband.action.drag_window()
end, { desc = "move a window by its border" })
```

Bindings are made only while the configuration loads.

### Modes

`gband.keymap.mode(table, opts)`, while the configuration loads, makes `table` a mode.
`opts` is nil or a table whose `label` is nil or a non-empty string naming the mode for display.
Declaring a mode again replaces its label, and `gband.keymap.set` and `gband.keymap.del` on its table keep the declaration.
`root` cannot be a mode.

While a mode is active, a bound key runs its binding, any other key is discarded, and neither reaches a window or a plugin window.
The mode stays active after each key and emits no `KeyTableChanged`, until a binding enters another table.
The modal key style makes `prefix` a mode labelled `navigation`, and leaves it with Escape, Enter, `n`, `?`, `:` and the prefix key; the direct and floating key styles declare no mode.
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
- `windows`: one table per window, top to bottom, each with `id` and either `rows`, a fixed height, or `weight`, an automatic height's weight. A window that shows a tiled plugin window this client opened also has `plugin_window`. A window that runs a program also has `name`, the name its border shows, with ` #n` when another window of the band has the same name, and `manual_name`, the name set by `gband.window.rename`, when it has one. A window this client has focused has `last_focus`, as described below.
- `floating`: one table per floating window, in the band's floating order, each with `id`, `width` and `full_width` as a column has them, `rows`, the box's height, and `col` and `row`, the box's top-left cell as placed in the screen area. A window that shows a plugin window this client opened also has `plugin_window`, and one that runs a program has `name` and `manual_name` as a window table has them. A window this client has focused has `last_focus`. A window this client has minimized has `minimized`, true; other windows have no `minimized`.

Each client counts the focuses it records, from 0 when it attaches.
`last_focus` is the count at the window's last focus by this client, so a higher `last_focus` is a more recent focus.
A window this client never focused has no `last_focus`, a reload keeps every `last_focus`, and a peek, described under `gband.window.focus`, changes none.
This client draws the floating windows of a band that it has not minimized in this order, from bottom to top: those without `last_focus` in the order of `floating`, then the others by ascending `last_focus`.

`gband.view()` returns `band`, the viewed band, `window`, the focused window or nil, `floating`, true while the band's floating layer has focus, `plugin_window`, the focused plugin window or nil, `table`, the active key table, and `cols` and `rows`, the size of the ribbon.
While the focused window is one this client peeks, it also returns `peek`, true; otherwise it has no `peek`.

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

`grow_column_width`, `shrink_column_width`, `grow_window_height` and `shrink_window_height` also take `step`, the amount to grow or shrink by as a fraction of the screen: greater than 0 and at most 10000 for a width, and at most 1 for a height.
Without `step` they use the client's `width_step` or `height_step` option, 1/10 by default.
A target with `step` and no `window` acts on the focused window: `gband.action.grow_column_width({ step = 1/4 })`.
The server's session actions take `step` the same way, and use 1/10 without it.

`open_window` takes `band`, `after` or both: the new column follows `after`'s column, or is `band`'s first column when only `band` is given.
`after` must be a tiled window.
`floating = true` opens the window floating instead, in `band` or in the viewed band, centred at the default width and two height steps shorter than the screen; it cannot be given with `after`.

`toggle_window_floating` takes `window` and an optional `after`, a tiled window of the same band.
A floating window is tiled again as a new column after `after`, or, without it, after the tiled window this client focused last in that band, or as the band's first column.
`after` is ignored when `window` is tiled.
`floating = true` floats `window`, and leaves a window that already floats as it is.
`floating = false` tiles `window`, after `after` or by the rule above, and leaves a tiled window as it is.
Without `floating` the action toggles.
The server decides whether the window moves, so two clients that both float a window float it once.

```lua
gband.on("WindowOpened", function(event)
  gband.action.toggle_window_floating({ window = event.window, floating = true })
end)
```

`send_prefix` takes `window` and sends the prefix key to it.
`gband.spawn` takes `band` and `after` beside `cmd`.

`drag_resize_window` takes `edges`, a list of the edges the drag moves: `"left"`, `"right"`, `"top"` and `"bottom"`, in any order.
The list names one edge or two, and never both of a pair: not `left` with `right`, and not `top` with `bottom`.
The drag then moves exactly those edges, wherever in the window the press lies; without a target it picks the edges nearest the press.
It still acts only when a mouse press runs it, and does nothing from a key.

```lua
gband.keymap.set("prefix", "rightmouse", function()
  gband.action.drag_resize_window({ edges = { "bottom" } })
end, { desc = "resize the window from its bottom edge" })
```

A target on a view action or on `detach`, `drag_window` or `drag_band`, a field the action does not take, a `step` out of its range, or a window or band not in the layout is an error at the line of the call.
So are a `floating` that is not a boolean, `after` together with `floating = true`, and a `toggle_window_floating` target without `window`.
So is an `edges` that is missing, not a list of edge names, empty, or that names an edge twice or both edges of a pair.
A call that raises one of these errors dispatches nothing.

```lua
gband.keymap.set("prefix", "N", function()
  local view = gband.view()
  gband.action.open_window({ band = view.band })
end, { desc = "open a window as the band's first column" })
```

## Windows and bands: `gband.window`, `gband.band`

These dispatch like actions, in the order they are called together with the callback's actions, and are errors outside a callback.
A window or band number not in the layout is an error at the line of the call.

- `gband.window.focus(window, opts)` views the window's band and focuses the window. Focusing a window this client has minimized restores it, on top of the other floating windows. `opts` is optional: a table that holds at most `peek`, a boolean. With `{ peek = true }` it peeks the window instead, as described below.
- `gband.window.minimize(window)` hides a floating window from this client only, until it is focused again. Other clients still draw it, and its program keeps its size. A window that is not floating is an error; a window already minimized stays minimized.
- `gband.band.view(band)` views a band, focusing the window last focused there. Viewing the band already viewed ends a peek and otherwise changes nothing.
- `gband.window.set_width(window, width)` sets the width of the window's column, or of its box when it floats, a number greater than 0 and at most 10000, and turns full width off.
- `gband.window.set_height(window, { rows = n })` gives a fixed height; `gband.window.set_height(window, { weight = w })` gives an automatic height of weight `w`.
- `gband.window.set_position(window, { col = c, row = r })` places a floating window's box with its top-left cell at `c` and `r`, kept inside the screen area. A window that is not floating is an error.
- `gband.window.send_keys(window, keys)` sends a key name, or a list of them, as key presses.
- `gband.window.send_text(window, text)` sends each character as a key press, a line feed or carriage return as Enter and a tab as Tab. Other control characters are an error.
- `gband.window.paste(window, text)` sends `text` as a paste.
- `gband.window.rename(window, name)` sets the window's manual name, which its border and every client show instead of its title or command, with leading and trailing whitespace trimmed. A `name` that is nil or empty clears it. A drawn window of a plugin window, a `name` that is neither a string nor nil, and a `name` holding a control character are errors.

Keys sent this way run no binding and leave the active key table alone.

A peek shows a window without recording it.
It focuses the window as a focus does: it views the window's band, shows the window's layer as active, moves the camera to it and emits `FocusChanged`, and `BandChanged` when the band changes.
It records nothing: every `last_focus`, the stacking order, the window and the layer each band remembers and the set of minimized windows stay as they were, and a minimized window is not restored.
While the peek lasts, the window is the focused window, drawn above every other floating window of its band, also when this client has minimized it, and shown and a pointer target.
A peeked tiled window is drawn as the focused tile is, under the floating windows.

The peek ends with a new focus, recorded as usual, when any focus other than a peek happens, when another window is peeked, or when the viewed band changes.
A focus of the peeked window itself records it and restores and raises the window, and emits no `FocusChanged`, since the focused window stays the same.
The peek ends without a new focus when `gband.band.view` views the band already viewed, when this client minimizes the peeked window, when the peeked window leaves the layout, and when a reload succeeds, which closes every plugin window; a reload that fails keeps the peek.
Focus then returns to the window this client last focused in the viewed band, recording nothing, or, when that window is gone or minimized, goes where viewing the band sends it.
The window returns to its place in the stacking order, or is hidden again when it is minimized.

An `opts` that is neither nil nor a table, a field of `opts` other than `peek`, and a `peek` that is not a boolean are errors at the line of the call, and the call dispatches nothing.

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
| `on_input` | both | function of the number and a text | none |
| `on_close`, `on_resize` | both | function | none |
| `on_mouse` | both | function of the number and a mouse table | none |
| `hover` | both | boolean: `on_mouse` also runs as the pointer moves over the plugin window | `false` |
| `row`, `col` | floating | integer of at least 0, or `"center"` | `"center"` |
| `width`, `height` | floating | integer of at least 1 | half the ribbon |
| `border` | floating | a boolean, or a border table `{ sides = ..., chars = ... }` | `true` |
| `title` | floating | string | none |
| `band`, `after` | tiled | as the `open_window` target takes them | the viewed band and focused window |
| `column_width` | tiled | a width | the server's `default_column_width` |

A border table draws the sides it lists, all four without `sides`, with the characters `chars` names, `"plain"` without it; sides and characters take the same values as the client's border options.
A side left out still takes its cell, so the content area is the same whatever the table says, and the title is drawn on the top row even when the top side is not.
`true` is all four sides in `"plain"`.

A line is a string or a list of spans; a span is a string or `{ text = ..., hl = "Group" }`.
Spans without `hl` use `PluginWindow`, and a span's style is its group's resolved style over `PluginWindow`'s.
Control characters are removed, and a line longer than the plugin window is cut.

`gband.ui.width(text)` returns the cells `text` takes: two for a wide character, zero for a zero-width one, and one for any other.
`gband.ui.truncate(text, width)` returns `text` when it fits in `width` cells, and otherwise the longest prefix that fits in `width - 1` cells followed by `…`, or the empty string when `width` is 0.
Both remove control characters first, so they measure text as plugin windows and bars draw it.

- `gband.win.set_lines(win, lines)` replaces the lines.
- `gband.win.scroll(win, count)` moves the first shown line, and `gband.win.set_cursor(win, line)` the cursor line. With `cursorline`, the plugin window scrolls just enough to keep the cursor line shown and draws it in `PluginWindowCursorLine`.
- `gband.win.focus(win)` focuses a floating plugin window, or the window of a tiled plugin window.
- `gband.win.set_config(win, config)` changes a floating plugin window's `row`, `col`, `width`, `height`, `border` or `title`. Any other field, `hover` included, is an error.
- `gband.win.close(win)` closes a plugin window, and for a tiled one its window too. Closing a plugin window that is not open does nothing.
- `gband.win.info(win)` returns `id`, `kind`, `focused`, `window`, `top`, `cursor`, `line_count`, `cols`, `rows` and `hover`, and for a floating plugin window `row`, `col`, `width` and `height`.
- `gband.win.list()` returns the open plugin windows' numbers in ascending order.

The focused plugin window is the focused floating plugin window, otherwise the plugin window shown in the focused window.
A newly opened floating plugin window with `focus` takes focus; moving focus or viewing another band leaves no floating plugin window focused.
The exception is a focus change caused by the floating plugin window's own `keys` function, or, for one opened with `hover = true`, by its `on_mouse` for any kind of event: it stays focused until the next key press, whether the change happens at once or arrives later from the server, as an opened window's focus does.
A floating plugin window that such code opens with focus, or focuses with `gband.win.focus`, takes that hold over, also when the code closed the first one.
So a list can run actions on Enter and stay open for the next choice, and preview the window under the pointer with a peek.
While a plugin window is focused, every key the bindings leave unused goes to it and never to a program, and so does every paste.
A key goes to the first of these that takes it:

1. A key in `keys` runs its function with the plugin window's number.
2. A key that types a character, a character key pressed without Ctrl and without Alt, runs `on_input` with the number and that character, when the plugin window has `on_input`. Shift+A gives `A` and the space bar a space.
3. Up or `k` and Down or `j` move the cursor line or scroll, PageUp and PageDown scroll a page, Home and End show the first or last line, and `q` and Escape close a floating plugin window.

Any other key is discarded.
With `on_input` set, `j`, `k` and `q` are text and no longer scroll or close; Escape and the keys that type nothing keep their defaults.
A paste runs `on_input` with the number and the pasted text exactly as pasted, line breaks and other control characters included, and is discarded by a plugin window without `on_input`.
The `close_window` action, such as Ctrl+Space then `q`, closes the focused floating plugin window as `gband.win.close` does, running its `on_close` and sending nothing to the server, also on a band with no window; with no floating plugin window focused, it closes the focused window.
A `close_window` dispatched with a target closes the window it names and leaves the floating plugin window open.

The `keys` functions, `on_input`, `on_close` and `on_resize` run as callbacks of the plugin that opened the plugin window: an error in one is reported as that plugin's error, and the plugin window stays open.
`on_close` runs once with the plugin window's number when the plugin window closes, but not when a reload closes it.
`on_resize` runs with the number and the new columns and rows when the plugin window's content area changes size, including when a tiled plugin window's size first becomes known.
A plugin window closes when its plugin is marked failed.

Without `on_mouse`, a left click on a plugin window with `cursorline` moves the cursor line to the line under the pointer, and the wheel acts as Down and Up do, in any mode and whether or not the plugin window has focus.
A wheel step that a binding takes, such as Alt with the wheel in the default configuration, reaches neither these defaults nor `on_mouse`.
With `on_mouse`, those defaults are off.
`on_mouse` runs for a press on the plugin window that no `root` binding takes, a press its `root` binding declines included, for that press's drags and release, and for each wheel step over it in any mode that no binding takes, a declined one included, with the plugin window's number and a table:

- `kind`: `"press"`, `"release"`, `"drag"`, `"scroll"` or `"move"`.
- `button`: `"left"`, `"middle"` or `"right"`, except for `"scroll"` and `"move"`.
- `direction`: `"up"`, `"down"`, `"left"` or `"right"`, for `"scroll"`.
- `content_col`, `content_row`: the cell inside the border, from 0, nil on the border.
- `line`: the line shown at `content_row`, from 1, nil past the last line.
- `box_col`, `box_row`, `box_width`, `box_height`: the cell inside the plugin window's box, border included, from 0, and the box's size as drawn, all nil when the pointer is off the plugin window.
- `ctrl`, `alt`, `shift`: the modifiers held.

A plugin window opened with `hover = true` also gets `on_mouse` with the kind `"move"` each time the pointer moves, with no button held, to another cell over it, in any mode and whether or not it has focus, with the same fields as a press and no `button`.
A move focuses nothing and emits no built-in event.
A plugin window without `hover` gets no move, and one with `hover` but without `on_mouse` ignores moves.
`hover` is fixed when the plugin window opens.

A press focuses the plugin window first, and `on_mouse` runs as a callback of the plugin that opened it.
The drag actions move and resize floating plugin windows as they do floating windows, running `on_resize` for each new size.

```lua
gband.keymap.set("prefix", "K", function()
  local lines = {}
  for _, binding in ipairs(gband.keymap.list("prefix")) do
    lines[#lines + 1] = {
      { text = ("%-10s"):format(binding.key), hl = "KeyListKey" },
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

## Side bars: `gband.bar`

A bar reserves columns at the left or right of one client's terminal and shows lines of styled text there.
The client reports the columns its bars leave as its size, so column widths, full width included, are measured against that space.
Adding, resizing or removing a bar resizes the windows, as a terminal resize does, while moving a bar to the other side keeps their size.
Several bar changes from one callback or one load are reported as one resize.
Each client has its own bars, and the server sees only the size they leave.

`gband.bar.add(spec)` adds a bar and returns its full id:

| field | value | default |
|---|---|---|
| `id` | a non-empty string, namespaced like action names | the plugin's name |
| `side` | `"left"` or `"right"` | required |
| `size` | the width in columns, an integer of at least 1 | `1` |
| `order` | a number; bars on one side stack from the terminal's edge inward in ascending order | `0` |
| `lines` | the lines, as a plugin window takes them | none |
| `hl` | the group of spans without `hl` and of empty cells | `"Bar"` |
| `on_resize` | a function | none |

- `gband.bar.set_lines(id, lines)` replaces the lines; row `r` shows line `r + 1`, cut at the bar's edge, and a bar does not scroll.
- `gband.bar.set_config(id, config)` changes `side`, `size`, `order` and `hl`.
- `gband.bar.remove(id)` removes the bar and returns `true`, or returns `false` when no bar has that id.
- `gband.bar.info(id)` returns `id`, `side`, `size`, `order`, `hl`, `plugin`, `shown`, and `col`, `width` and `height` while it is shown.
- `gband.bar.list()` returns the same for every bar, in byte order of ids.

All of them can be called while the configuration loads and in any callback.
An id no bar has is an error, except for `remove`.

The client places its bars each time it draws, from the whole terminal, in ascending `order` and then in the order they were added: a left bar takes the first `size` columns left, a right bar the last.
A bar whose `size` is not less than the columns left is not shown, and the next bar is placed as if it were absent.
`on_resize(id, width, height)` runs each time the bar's shown size changes, first after loading, with 0 and 0 while it is not shown.
A reload removes every bar of the previous configuration, and a bar goes when its plugin is marked failed.
The group `Bar` has no fields by default.

```lua
gband.bar.add({
  id = "clock",
  side = "right",
  size = 7,
  lines = { os.date("%H:%M") },
})
```

A plugin module adds its bar in `setup` and draws into it from its handlers.
The [window sample](../examples/plugins/window) shows the focused window in a bar 12 columns wide on the right, whose id is the plugin's name, `window`:

```lua
local M = { name = "window", api = 2 }

function M.setup(opts)
  gband.hl.default("WindowSegment", { link = "SidebarMode" })

  local bar
  local function draw(window)
    if window == nil then
      gband.bar.set_lines(bar, {})
      return
    end
    gband.bar.set_lines(bar, { { { text = "window " .. window, hl = "WindowSegment" } } })
  end

  bar = gband.bar.add({
    side = "right",
    size = 12,
    order = opts.order or 0,
    on_resize = function()
      draw(gband.view().window)
    end,
  })
  gband.on("Attached", function()
    draw(gband.view().window)
  end)
  gband.on("FocusChanged", function(ev)
    draw(ev.window)
  end)
end

return M
```

## The sidebar: `gband.sidebar`

gband bundles the sidebar, module `gband.sidebar`, plugin `sidebar`.
Its `setup` adds one bar with `gband.bar.add`, 1 column wide, in the group `Bar`, so the bar's id is `sidebar`.
The default configuration sets up `gband.errors` and then `gband.sidebar`, unless `gband.settings.sidebar()` returns `false`, so on an 80×24 terminal the sidebar is column 0 and the ribbon spans columns 1 to 79.
A `user/init.lua` replaces the default configuration, so it gets the sidebar only by setting it up, here on the right:

```lua
gband.plugin("gband.sidebar", { side = "right" })
```

| option | value | default |
|---|---|---|
| `side` | `"left"` or `"right"` | `"left"` |
| `order` | a number, its order among the bars of its side | `0` |

Any other option, or a wrong type or value, makes `setup` raise an error naming the option, and adds no bar.
The width is not an option: a plugin that wants more columns adds a bar of its own.

On a bar of height `h`, counting rows from 0:

| row | shows | group |
|---|---|---|
| 0 | the mode letter, or `∷` with the floating key style | `SidebarMode` |
| 1 | nothing | `Bar` |
| 2 to `h - 2` | one band label each, in layout order | `SidebarBandActive` for the viewed band, `SidebarBand` for the others |
| `h - 1` | `!` while the client reports an error, otherwise nothing | `SidebarError` |

The mode letter is `I` while the active key table is `root`.
For any other table it is the first character of `gband.keymap.label(table)`, uppercased when it is an ASCII lowercase letter: navigation mode shows `N`, the direct key style's `prefix` shows `P`, and a mode labelled `resize` shows `R`.
While `gband.keystyle.current()` returns `floating`, row 0 shows the apps character `∷`, U+2237, instead, whatever table is active, so a mode of your own shows no letter there.
`∷` is East Asian ambiguous width, so a terminal that draws ambiguous characters two cells wide draws it over the ribbon; a copy of the sidebar can draw `⸬`, U+2E2C, a lookalike of neutral width that fewer fonts hold.
A configuration that requires `gband.keystyle.floating` by name, without `gband.keystyle.use`, keeps the mode letter.

A band's label is its position in the layout, counted from 1, not its number: `1` to `9`, then `a` to `z` for positions 10 to 35.
The empty last band has a label too, so a new session shows `1` and `2`.
A band past the 35th, or whose row would be the last row or below, is not drawn.
A bar of 3 rows or fewer shows the mode letter and the marker and no band, and a bar of 1 row shows the marker while an error is reported and the mode letter otherwise.
The viewed band differs from the others by its group alone, so a screenshot without styles reads the same whichever band is viewed.

The error marker shows no text; `gband.errors()` and the `errors.open` action give the errors' text.
The client draws no banner while the marker is drawn, and draws the banner on the ribbon's bottom row when an error is reported and no marker is drawn: without the sidebar, or while it is not shown.

The sidebar redraws in the frame that follows a change of the viewed band, the bands, the active key table or the error list, and when one of its groups changes.

A press of the left button on a band's label views that band, as `gband.band.view` does, whatever key table is active.
The sidebar does this from its own `MousePressed` handler, since a press on a bar cell reaches handlers with the target `outside`.
A press of the left button on `∷` dispatches `desktop.list`, which opens the window list, when `gband.action` holds it.
Any other press on the sidebar does nothing.

## The key list: `gband.keylist`

gband also bundles the key list, module `gband.keylist`, plugin `keylist`.
Its `setup` takes no options, and registers the action `keylist.open`, described as `list the keys`.
Every key style preset sets it up before its key bindings and binds Ctrl+Space then `?` to it:

```lua
gband.plugin("gband.keylist")
gband.keymap.set("prefix", "?", gband.action["keylist.open"], { desc = "list the keys" })
```

`keylist.open` opens a focused floating plugin window titled with `gband.keymap.label("prefix")` and ` keys`, `navigation keys` with the modal key style and `prefix keys` with the direct and floating ones, centred in the ribbon, with its cursor line on the first line.
It enters `root` as it opens, and as it focuses an open list, so the keys that follow reach the list and not navigation mode.
It holds one line per binding of the `prefix` table, in the order `gband.keymap.list("prefix")` gives them when it opens: the key in its short form, as "Key form" describes, such as `C-space` for the prefix key, padded to two cells more than the widest key, then the binding's description, its action's description when it has none, its action's name when that is empty too, or `function`.
The plugin window is as wide as its longest line plus its border and as high as its lines plus its border, at most 15 rows, and the ribbon caps both.
Dispatching `keylist.open` while the list is open focuses it and opens no second one.

Enter runs the binding on the cursor line with `gband.keymap.run("prefix", key)`: an action is dispatched with no target, so it acts on the focused window behind the list, and a function runs, as a key bound to it would.
The list stays open and focused, by the rule for a floating plugin window's own `keys`, so you can choose again.
When another floating plugin window has focus once the binding has run, such as the settings window or the Lua prompt it opened, the list closes itself.
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

The key list draws keys in `KeyListKey` and the description it cannot run in `KeyListMuted`; it defines them as defaults when its module is first required, `KeyListKey` as `{ bold = true }` and `KeyListMuted` as `{ dim = true }`.

### Key form

The key list shows keys short: `C-`, `A-` and `S-` for Ctrl, Alt and Shift, in that order, named keys in lowercase, `escape` as `esc`, a one-character key as written, and `shift` with a lowercase letter as the uppercase letter.
`ctrl+space` shows as `C-space`, `shift+d` as `D`, `Alt+PageUp` as `A-pageup` and `alt++` as `A-+`.
In the list, the key `prefix` shows as the key the `prefix` option names.

The bundled module `gband.keyform` gives the same form, so a plugin window can show keys as the key list does:

```lua
local keyform = require("gband.keyform")
keyform("ctrl+space") -- "C-space"
```

## The Lua prompt: `gband.prompt`

gband also bundles the Lua prompt, module `gband.prompt`, plugin `prompt`.
Its `setup` takes no options, and registers the actions `prompt.open`, described as `run Lua`, and `prompt.rename`, described as `rename the window`.
Every key style preset sets it up right after the key list, before its key bindings, and binds Ctrl+Space then `:` to `prompt.open`, right after `?`, and Ctrl+Space then `N` to `prompt.rename`, right after `:`:

```lua
gband.plugin("gband.prompt")
gband.keymap.set("prefix", ":", gband.action["prompt.open"], { desc = "run Lua" })
gband.keymap.set("prefix", "N", gband.action["prompt.rename"], { desc = "rename the window" })
```

`prompt.open` enters `root` and opens a focused floating plugin window with a border, titled `lua`, 3 rows high and as wide as the ribbon, on the ribbon's last three rows from its first column, sized from `gband.view()` as it opens.
Dispatching it while the prompt is open focuses that prompt, keeps its line and opens no second one.
The window is a plain user of `gband.win`: it takes typed characters and pastes through `on_input`, and Enter, Backspace and Ctrl+U through `keys`.

Its one line is `:`, the typed text, then one space in `PromptCursor` as the cursor.
When that is wider than the content area, the fewest leading characters are left out that let the rest and the cursor fit, so the end of the line stays shown; the line is fitted again when the content area's width changes.

| input | effect |
|---|---|
| a typed character | append it, `j`, `k`, `q` and space included |
| a paste | append it, with each `\r\n` and every other control character turned into one space |
| Backspace | remove the last character; on an empty line, close the prompt and run nothing |
| Ctrl+U | clear the line |
| Escape | close the prompt and run nothing |
| Enter | close the prompt, then run the line |

Enter closes the prompt before the line runs, so a line that fails leaves no prompt behind, and a floating plugin window the line opens takes focus.
An empty line runs nothing.
A line runs as Lua source text, never a binary chunk, compiled as chunk `@prompt`, inside the Enter callback, so its actions and other dispatches take effect when the callback returns, in order, as a binding function's do.
It belongs to no plugin, as code of `user/init.lua` does, although the plugin `prompt` runs it, and it has an instruction budget of its own: a line that exceeds it stops alone, and `prompt` is not marked failed.
A syntax error, a runtime error and a stop by the instruction limit are each reported as a configuration error read `prompt:1: <message>`, such as `prompt:1: instruction limit exceeded`; what the line dispatched before the error stands.
Return values are dropped.

`prompt.rename` does nothing unless `gband.view()` names a focused window that runs a program.
Then it enters `root` and opens the rename prompt for that window: a floating plugin window placed and sized as the Lua prompt, titled `rename`, whose line starts as the window's manual name and shows no `:`.
It edits its line as the Lua prompt does, but Enter closes it and calls `gband.window.rename` on the window it was opened for, unless that window has left the layout.
Dispatching `prompt.rename` while the rename prompt is open focuses it and keeps its line and its window.
At most one of the two prompts is open: opening either closes the other first, dropping its line.

The prompt gives `PromptCursor` the default `{ reverse = true }` when its module is first required.

## The desktop: `gband.desktop`

gband also bundles the desktop, module `gband.desktop`, plugin `desktop`, which turns gband into a desktop like MS Windows: every window floats, a window's border moves and resizes it, its title bar has buttons, and one list holds every window.
The floating key style sets it up after the key list and the Lua prompt.
Its `setup` takes no options, raises an error naming any field it is given, and registers these actions:

| action | description | effect |
|---|---|---|
| `desktop.list` | `list the windows` | opens the window list centred in the ribbon, or focuses it when it is open |
| `desktop.leader` | `open the window list, or send the prefix key from it` | while the window list is the focused plugin window, closes it by keeping what it shows and runs `gband.keymap.run("prefix", "prefix")`; otherwise acts as `desktop.list` |
| `desktop.press` | `move, resize or press a button of a floating window` | the left press on a border, below |
| `desktop.menu` | `open the menu for the cell under the pointer` | the right press, below |

`desktop.press` and `desktop.menu` act only when a mouse binding runs them, and do nothing from a key or `gband.keymap.run`.
Setting the plugin up under another key style works too, and floats windows there as well: a modal configuration can bind `prefix w` to `desktop.list`.

**Windows float.**
A window runs a program when its `gband.layout()` table holds `name`.
The plugin floats a window with `toggle_window_floating({ window = id, floating = true })`:

- on `WindowOpened`, when the layout holds the window tiled, does not mark it with `plugin_window`, and no tiled plugin window of this client still waits for its window. A window's name reaches the client only after its `WindowOpened`, so the drawn window of another client's tiled plugin window floats too, with no buttons;
- on `Attached` and on `ConfigReloaded`, every tiled window that runs a program, in every band, in layout order.

It floats nothing else, so a window that someone tiles afterwards stays tiled until the next attach or reload of a floating-style client.
The layout is shared: while a client with the floating style is attached, every client sees these windows float, whatever its key style, and so do the boxes that maximize, restore and tile set.
The server floats each window once, so several floating-style clients agree.

Every first float and every `open_window({ floating = true })` gives the same centred box, so the plugin cascades: the first time a window it saw open, or that its sweep named, floats, and another floating window of its band has the same top-left cell, it moves the window two columns right and one row down, again until the cell is free, as long as the box still fits in the screen area.
Otherwise the window keeps its box.
A window the user moves later is never placed again.

**Title bar buttons.**
A decorations provider gives every floating window that runs a program `[_][□][X]`: minimize, maximize and close.
While the window counts as maximized, `[❐]` replaces `[□]` and restores it.
They take the box's columns `w − 11` to `w − 3`; a box narrower than 13 cells shows none.
`□` is East Asian ambiguous width, as gband's border characters are; copy the plugin and use `▢` if your terminal draws it wide.

**Left press on a border.**
`desktop.press` takes a press only on the border of a floating window that runs a program, and declines every other press, which then takes the interactive defaults: focus, forward to the program, select.
For a press at `box_col` `x` and `box_row` `y` of a `w`×`h` box, the first rule that matches applies:

1. a button: nothing happens until the left button is released on the same button of the same window, which then acts; a release elsewhere does nothing;
2. a corner zone, the corner cell and the cell beside it along each side, checked top-left, top-right, bottom-left, bottom-right: `drag_resize_window` with that corner's two edges;
3. the rest of the top border: `drag_window`;
4. the rest of the left, right or bottom border: `drag_resize_window` with that one edge.

The top edge resizes only from a corner, since the top border is the title bar.
Alt with a drag still moves and resizes any window from its content, through the `mod+` bindings.

**Right press.**
`desktop.menu` opens the window menu on the border of a floating window that runs a program, and the window list on empty ribbon, at the pointer, without changing the focused window.
It declines every other press, so a right click in content still pastes, and a right press on a bar belongs to that bar.

**Maximize, restore and tile.**
Maximize gives a window the whole screen area, Tile left and Tile right its left or right half at full height, with `gband.window.set_width`, `set_height` and `set_position` in that order.
Whether a window is maximized or tiled is read from its box, so the `[❐]` glyph survives a reload and another client's moves.
The box to restore is kept per window in this client only; a reload forgets it, and Restore then centres a box of half the screen area's width and height.
When `LayoutChanged` reports another screen area size, from any client's resize or attach, each maximized or tiled window gets the box of its state for the new size.

**The menus.**
The window list and the window menu are borderless floating plugin windows that draw their own frame in `PluginWindowBorder`, `PluginWindowTitle`, `PluginWindow` and `PluginWindowCursorLine`.
At most one is open.

| key | effect |
|---|---|
| `j`, Down / `k`, Up | the next or previous entry |
| PageDown, PageUp | a page down or up |
| Home, End | the first or last entry |
| Enter | pick the entry |
| Escape, `q` | close the menu |

A left press selects an entry and the release on it picks it, and a press anywhere else closes the menu and still takes its own effect.
The wheel moves the window menu's selection, and scrolls the window list by one row without moving its selection.
A menu moves its shown rows only as far as the selection, a wheel step or a change of its rows or height needs.

The window list, titled `windows`, holds `New window`, which opens a floating window running your shell, `Settings`, then every window that runs a program, by its shown name.
The windows this client focused come first, the most recent at the top, as `last_focus` orders them, then the windows it never focused, in layout order; the order is taken when the list opens and stays while it is open, with windows that close dropped and windows that open added at the end.
Each entry starts with its shortcut, in `DesktopShortcut`: `n` for `New window`, `s` for `Settings`, `1` to `9` for the first nine windows and `0` for the tenth; later windows have none.
While windows sit in two or more bands, each window entry ends with `band ` and the band's sidebar label, right-aligned, and a window this client minimized has ` (minimized)` after its name and its entry in `DesktopMinimized`.
Its `[□]` gives it the ribbon's height and `[❐]` gives its height back, and its bottom border shows `? keys` while `prefix` binds `?`.

The window list previews the entry the selection lands on, by a key, a press, a shortcut or the pointer: a window entry is peeked with `gband.window.focus(id, { peek = true })`, so it shows from its band, on top and also when minimized, and `New window` and `Settings` show the band and focus the list opened on.
A preview records nothing, so the focus order, the stacking order and the minimized windows stay as they were.
The list is opened with `hover = true`, so moving the pointer onto an entry selects and previews it, and the list stays focused through its previews; the window menu is opened without `hover`.

The window list closes by keeping what it shows or by cancelling:

- Picking a window, by Enter, a click or its shortcut, keeps: it focuses the window as `gband.window.focus` does, which records it, raises it and restores it when minimized.
- Picking `New window` or `Settings` cancels, then opens the window in the band the list opened on, or the settings window.
- `[X]`, Escape, `q` and closing by other code with `gband.win.close` cancel: the list views the band it opened on with `gband.band.view`, which ends the peek, so the band, the focused window, the stacking order and the minimized windows are as they were.
- A press elsewhere focuses the window it names; otherwise it leaves the view as the press changed it, or cancels when the press changed nothing.
- A right press that opens a menu, and a `prefix` key, keep.

The keys `n`, `s` and `0` to `9` belong to the list, whatever `prefix` binds: each picks the entry with that shortcut, and a digit no entry has does nothing.
Any other key that `prefix` binds keeps and runs that binding, so with the floating preset `D` detaches, `:` opens the Lua prompt, `?` opens the key list and `N` renames the window the list shows; a `prefix` binding of a digit, `n` or `s` stays reachable from the key list.

The window menu, titled with the window's name, holds `Close`, `Maximize` or `Restore`, `Minimize`, `Tile left` and `Tile right`, closes when its window leaves the layout, and leaves the focused window as it was when it closes.

**The leader.**
The floating preset's `prefix` is not a mode, so the prefix key makes it active; its `KeyTableChanged` handler then returns to `root` and dispatches `desktop.leader`, so the prefix key opens the window list and the keys that follow reach it.
Pressing the prefix key again from the list keeps the window it shows and sends the prefix key to it, as the other styles' prefix key twice does.

The plugin defines `DesktopButton` as `{ bold = true }`, `DesktopClose` as `{ bold = true }`, `DesktopMinimized` as `{ dim = true }` and `DesktopShortcut` as `{ link = "KeyListKey" }` when its module is first required, so `[X]` takes the border's colour and the shortcuts take the key list's.
No bundled theme sets `DesktopClose`; `gband.hl.set("DesktopClose", { fg = 1, bold = true })` brings back a red close button.

## Key styles: `gband.keystyle`

gband bundles three key style presets, the modules `gband.keystyle.modal`, `gband.keystyle.direct` and `gband.keystyle.floating`.
Each is a file of plain top-level calls, as a `user/init.lua` is: it sets up `gband.keylist` and `gband.prompt`, whose actions it binds, then makes its bindings in `prefix`, and binds no key in `root`.
The modal preset declares `prefix` a mode labelled `navigation` and binds Escape and Enter to return to interactive mode.
Its `n` opens a window and returns to interactive mode only when `gband.settings.interactive_on_new()` returned `true` when the preset loaded, and otherwise leaves navigation mode active.
The direct preset declares no mode, so each key after the prefix key acts once; it binds `n` to `open_window` and the prefix key to `send_prefix` directly.
Both bind Ctrl+Space then `s` to `gband.settings.open`, described `settings`, right after `:`.
The floating preset also sets up `gband.desktop`, declares no mode, and binds only `n` (a floating window), `?`, `:`, `N`, `s`, `D` and the prefix key in `prefix`, and `leftmouse` and `rightmouse` in `root`, as "The desktop" describes.
gband writes copies of all three to `defaults/keystyle/` for you to read; loading never reads the copies.

`gband.keystyle` is a client API:

| function | effect |
|---|---|
| `gband.keystyle.use(style)` | makes the bindings of `"modal"`, `"direct"` or `"floating"` by requiring `gband.keystyle.<style>`, and returns the style's name; with no argument it uses the saved style, or `"modal"` when none is saved |
| `gband.keystyle.saved()` | the saved style, `"modal"`, `"direct"` or `"floating"`, or nil |
| `gband.keystyle.current()` | the style `use` picked in this load, or nil when it was not called; requiring a preset by name does not change it |

`use` runs only while the configuration loads, and once per load; any other style, a second call and a call after loading are errors at the line of the call.
Bindings made after it replace the preset's binding for the same key.
The default configuration calls `gband.keystyle.use()` with no argument, so the saved style applies where a configuration does the same: a `user/init.lua` that binds its own keys keeps them, and the settings window's `keys` line then only saves the style.

`saved` reads `user/keystyle.lua` as a text chunk with an empty environment, and returns its value when it is `"modal"`, `"direct"` or `"floating"`.
It returns nil when `gband.config_dir` is nil, and when the file is missing, does not compile, raises an error or returns anything else; it reports nothing.
The file is never run as configuration, a plugin file or a module.
The settings window's `keys` line saves it, as "Settings" describes.

## Settings: `gband.settings`

`gband.settings` is a client API for the settings window and the settings it saves:

| function | effect |
|---|---|
| `gband.settings.open()` | enters `root` and opens the settings window, or focuses it when it is open |
| `gband.settings.theme()` | the theme saved in `user/theme.lua`, or nil |
| `gband.settings.sidebar()` | the sidebar setting saved in `user/sidebar.lua`, `true` or `false`, or nil |
| `gband.settings.interactive_on_new()` | the `I on new` setting saved in `user/interactive_on_new.lua`, `true` or `false`, or nil |
| `gband.settings.themes()` | a new list of the names the theme list shows |

`theme`, `sidebar` and `interactive_on_new` read their file as `gband.keystyle.saved()` reads `user/keystyle.lua`, and return its value when it is a valid colorscheme name or a boolean.
They return nil when `gband.config_dir` is nil, and when the file is missing, does not compile, raises an error or returns anything else; they report nothing.
None of these files is ever run as configuration, a plugin file or a module.
`themes` lists the bundled themes first, in the order "Themes" gives, without the alias `catppuccin`, then every other name for which a runtimepath entry holds `colors/<name>.lua`, in byte order.
A name appears once, so a `user/colors/nord.lua` keeps `nord`'s place.
These four are callable while the configuration loads and in any callback.

`open` is callable wherever an action value is, and calling it while the configuration loads is an error.
Called while the theme list is open, it closes the list as Escape does and focuses the settings window.

The settings window is a floating plugin window with a border, titled `settings`, 31 columns wide and as high as its lines plus its border, at most the ribbon's size, centred in the ribbon, with its cursor line on its first line.
It holds these lines, each a label in `SettingsLabel` padded to 9 cells, then the value; the `I on new` line only while the `keys` line shows `modal`:

| line | value | Enter | `h`, Left, `l`, Right |
|---|---|---|---|
| `theme` | the name `gband.colorscheme()` returns | opens the theme list | `l` and Right load the theme after the active one in `themes()`, `h` and Left the one before, and save it |
| `sidebar` | `off` when `sidebar()` returns `false`, `on` otherwise | saves the other value | the same as Enter |
| `keys` | the style `gband.keystyle.saved()` returns, or `modal` | saves the next style of `modal`, `direct`, `floating`, wrapping | `l` and Right save the next style, as Enter does, `h` and Left the previous one |
| `I on new` | `on` when `interactive_on_new()` returns `true`, `off` otherwise | saves the other value | the same as Enter |

The `theme` line wraps at both ends of the list, and an active colorscheme missing from it steps to the first or the last theme.
A theme that fails to load is reported, as "Colorschemes" describes, and not saved.
Every other key is a floating plugin window default: `j`, `k`, Up and Down move the cursor line, and Escape and `q` close the window and save nothing.

The theme list is a floating plugin window with a border, titled `theme`, with one line per name of `themes()`, as wide as the longest name plus its border and as high as its lines plus its border, capped by the ribbon and centred in it.
Its cursor line starts on the active colorscheme.
Each move of the cursor line, by `j`, `k`, the arrow keys, PageUp, PageDown, Home, End, a click or the wheel, loads that line's theme with `gband.colorscheme`, so the ribbon previews it.
Enter saves the line's theme and closes the list; when that theme failed to load, it only closes the list.
Escape and `q` load again the colorscheme that was active when the list opened, save nothing and close the list.
Closing the list focuses the settings window.

Saving writes `return "<name>"`, `return true` or `return false`, or `return "<style>"`, and a newline, to a temporary file in `user/`, and renames it over `user/theme.lua`, `user/sidebar.lua`, `user/keystyle.lua` or `user/interactive_on_new.lua`.
That reloads the configuration, as any saved `.lua` file under `user/` does.
The client keeps the line the setting belongs to across that reload, and when the load succeeds it opens the settings window again with its cursor line there, whatever configuration file is in use.
When there is no configuration directory or the write fails, the key raises an error naming the file and the reason, the settings window stays open, and the setting in use stays.

The saved theme applies to every configuration, since it loads before the init file.
The sidebar, the key style and `I on new` apply where a configuration asks for them: the default configuration sets up `gband.sidebar` unless `sidebar()` returns `false`, and calls `gband.keystyle.use()` with no argument, whose modal preset reads `interactive_on_new()`.
A `user/init.lua` with its own `n` binding follows the setting by comparing `interactive_on_new()` with `true`, since nil means it was never saved and reads as off.
A `user/init.lua` that sets up the sidebar or binds its own keys keeps its choice, and the settings window then only saves the setting.

On `Attached`, the default configuration opens the settings window when `gband.config_dir` is set and `theme()`, `sidebar()` and `gband.keystyle.saved()` all return nil.
`Attached` fires once per client and never for a reload, so the window opens at most once per start, and every start offers it until one setting is saved.

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

After loading, each `gband.hl.set` or `gband.hl.default` that changes a group emits `HighlightChanged` with `group`.

The sidebar defines these groups, as defaults, when its module is first required:

| group | default | use |
|---|---|---|
| `SidebarMode` | `{ bold = true }` | the mode letter |
| `SidebarBand` | `{ dim = true }` | the labels of the bands not viewed |
| `SidebarBandActive` | `{ bold = true }` | the viewed band's label |
| `SidebarError` | `{ fg = 1, bold = true }` | the error marker |

The key list adds `KeyListKey`, `{ bold = true }`, and `KeyListMuted`, `{ dim = true }`, the same way.
The Lua prompt adds `PromptCursor`, `{ reverse = true }`, which draws its cursor cell.

The plugin window API defines these groups, as defaults:

| group | default | use |
|---|---|---|
| `PluginWindow` | `{}` | every plugin window cell |
| `PluginWindowBorder` | `{ fg = 8 }` | a floating plugin window's border |
| `PluginWindowTitle` | `{ bold = true }` | a floating plugin window's title, over `PluginWindowBorder` |
| `PluginWindowCursorLine` | `{ reverse = true }` | the cursor line |

A change of any group redraws every plugin window and every bar.

The client draws its own parts with these groups, which it defines as defaults before the init file runs:

| group | default | use |
|---|---|---|
| `WindowBorder` | `{ dim = true }` | the border of each tiled or floating window that is not focused |
| `WindowBorderFocused` | `{ fg = "#b1b9f9", bold = true }` | the focused tiled or floating window's border, and a lifted tile's drop outline |
| `ErrorBanner` | `{ fg = "red", reverse = true }` | the error banner on the ribbon's bottom row |
| `SettingsLabel` | `{ dim = true }` | the labels of the settings window |

A change of one of them redraws the client.
Floating plugin windows keep their own groups.

The client draws window borders with its own options, `tile_border_sides` and `tile_border_chars` for tiles and `floating_border_sides` and `floating_border_chars` for floating windows.
The focused tiled window takes the characters of `focused_tile_border_chars` and the focused floating window those of `focused_floating_border_chars`, each keeping its sides, and a lifted tile's drop outline is drawn with all four sides in `focused_tile_border_chars`.
All four character options default to `"rounded"`.
A list of sides holds `"top"`, `"right"`, `"bottom"` and `"left"`, and a character set is `"plain"`, `"rounded"`, `"double"`, `"thick"`, or eight one-cell strings in the order top-left, top, top-right, right, bottom-right, bottom, bottom-left, left.
A corner where only one of its sides is drawn takes that side's character, and the cells of a side not drawn stay blank, so borders never change a window's size.

In a bar, a span's style is its group's resolved style over the bar's `hl` group's, field by field.

### Colors in the terminal

The client draws 24-bit colors when its own `COLORTERM` is `truecolor` or `24bit`.
Otherwise it draws each `"#rrggbb"` color as the nearest of the palette indexes 16 to 255.
Index and named colors are drawn as their palette index either way.
Floating plugin windows follow these rules.
The terminal palette, as "The terminal palette" describes, then maps the default colors and the indexes 0 to 15 of every cell drawn.
A tiled plugin window's contents are sent to the server as 24-bit colors, and every client shows them as its terminal does, as it shows a program's colors.

## Colorschemes: `gband.colorscheme`

A colorscheme is a Lua file that sets groups with `gband.hl.set` and the terminal palette with `gband.palette.set`.
`gband.colorscheme(name)` runs `colors/<name>.lua` from the first runtimepath entry that holds one, or the colorscheme of that name bundled with gband.
Names start with an ASCII letter or digit and hold letters, digits, `_` and `-`.

`gband.colorscheme()` returns the active colorscheme's name.

Switching first removes every group's explicit setting, keeping the defaults, and empties the terminal palette, then runs the file.
Set your own groups after the `gband.colorscheme` call.
The code in a colorscheme file belongs to no plugin.

A switch is atomic.
It returns `true` and makes `name` active when the file runs to completion.
When the colorscheme is missing, its name is invalid, or its file raises an error or hits the instruction limit, gband restores every explicit setting and the terminal palette, keeps the active colorscheme, reports the error as `colors/<name>: ...` and returns `false`.
That error fails neither the load nor any plugin.

After loading, a successful switch emits `ColorschemeChanged` with `name` and `previous`, once, and no `HighlightChanged` for the groups the file sets.

```lua
-- colors/dusk.lua
gband.hl.set("Bar", { fg = "#e0def4", bg = "#232136" })
gband.hl.set("SidebarMode", { fg = "#c4a7e7", bold = true })
gband.hl.set("WindowSegment", { fg = "#f6c177", bold = true })
```

### Themes

gband bundles these colorschemes, called themes, in this order: `default`, `terminal`, `catppuccin-latte`, `catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`, `tokyo-night`, `dracula`, `nord`, `gruvbox`, `one-dark`, `solarized`, `kanagawa`, `rose-pine` and `vesper`.
It also bundles `catppuccin`, which sets exactly what `catppuccin-mocha` sets.
A `colors/<name>.lua` on the runtimepath shadows the bundled theme of that name.

When loading starts, before the init file, the client loads the start theme as `gband.colorscheme` does: the theme `gband.settings.theme()` returns, or `default` when none is saved or the saved one fails to load.
The failure is reported as any failed colorscheme is.
The init file may load another colorscheme, which replaces the start theme.

`default` sets no group and no palette.
Every group draws with its default, as "Highlight groups" lists them, and the terminal draws in its own palette.
It leaves `Bar` unset, so bars, the sidebar included, draw on the terminal's default background.

Every other theme sets each of these groups: `Bar`, `SidebarMode`, `SidebarBand`, `SidebarBandActive`, `SidebarError`, `KeyListKey`, `KeyListMuted`, `PluginWindow`, `PluginWindowBorder`, `PluginWindowTitle`, `PluginWindowCursorLine`, `PromptCursor`, `SettingsLabel`, `WindowBorder`, `WindowBorderFocused` and `ErrorBanner`.
`terminal` uses only the 16 named colors, the indexes 0 to 15 and attributes, and sets no palette, so the host terminal's palette decides every color it draws.
Each of the other themes also sets every field of the terminal palette, from the default colors and 16 ANSI colors its authors publish for terminals, and its groups use hex colors.

Each of them is built with the bundled module `gband.theme`, which a colorscheme of your own can use too, from a terminal palette and five interface colors:

```lua
-- colors/ember.lua
require("gband.theme").apply({
  palette = {
    fg = "#e6d5c3",
    bg = "#1c1714",
    red = "#d0574a",
    bright_red = "#e8796c",
  },
  ui = {
    surface = "#2a221d",
    selection = "#43362e",
    muted = "#8a7a6c",
    accent = "#e3a857",
    error = "#d0574a",
  },
})
```

`apply` calls `gband.palette.set(palette)`, so `palette` takes the fields of "The terminal palette"; give it `fg` and `bg`, which the groups use too.
It then sets every theme group from these colors: `surface` is the background of bars and plugin windows, `selection` the cursor line's, `muted` the unfocused borders, the labels of bands not viewed, the key list's muted line and the settings labels, `accent` the mode letter, keys, titles, the prompt cursor and the focused border, and `error` the error marker and the banner.
Set a group after the call to change it.

### The terminal palette: `gband.palette`

`gband.palette` is a client API for the terminal palette: the colors that the client's terminal draws its default colors and its 16 ANSI colors with.

| function | effect |
|---|---|
| `gband.palette.set(spec)` | replaces the whole palette with `spec`; `gband.palette.set({})` empties it |
| `gband.palette.get()` | a new table of the fields the palette sets, with hex colors in lowercase |

`spec` is a table of up to 18 fields: `fg` and `bg`, the terminal's default foreground and background, and `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white` and their `bright_` forms, the indexes 0 to 15.
Each value is a `"#rrggbb"` color, in either case; a name or an index is not accepted.
A `spec` that is not a table, a field not listed or a value that is not a hex color is an error at the line of the call naming the field, and leaves the palette unchanged.
Both functions are callable while the configuration loads and in any callback, and after loading a change of the palette redraws the client.

The palette is empty when loading starts, before the start theme loads.
A colorscheme switch empties it before the file runs and restores it when the switch fails, so set it after the `gband.colorscheme` call to keep it.

After every other step of drawing, the client maps each cell of its terminal:

- the default foreground becomes `fg`, and the default background `bg`;
- index `n`, from 0 to 15, becomes the field of the `n`-th color name, `black` being 0.

Each mapping applies only when the palette sets that field.
It covers every cell: program output in windows, borders, the empty ribbon, bars, plugin windows and the banner.
A mapped color is drawn as a hex color, so a client without 24-bit color draws the nearest of the indexes 16 to 255.
Indexes 16 to 255 and hex colors are drawn unchanged.
The palette changes nothing the server stores or sends, and nothing a program in a window reads.

## The prelude and the API modules

Part of the client's API is written in Lua, on top of the primitives "Core primitives" describes.
Before the init file runs, the client requires the bundled module `gband.prelude` and runs nothing else of its own Lua.
The bundled prelude requires the API modules in this order, because they call into each other as they load:

| module | installs |
|---|---|
| `gband.hl` | `gband.hl` and the built-in highlight groups, and provides the drawn styles |
| `gband.palette` | `gband.palette` |
| `gband.colorscheme` | `gband.colorscheme` |
| `gband.bar` | `gband.bar`, and provides the bars |
| `gband.win` | `gband.win`, and provides the plugin windows |
| `gband.settings` | `gband.settings`, and provides the settings window |
| `gband.keystyle` | `gband.keystyle` |

It then loads the start theme with `require("gband.colorscheme").start()`.
An error while the prelude loads fails the load, as an error in the init file does.

Each of these is an ordinary module, so `require` finds a file of the same name on the runtimepath first.
To change one, copy it from `defaults/lua/gband/` of the configuration directory to `user/lua/gband/` and edit the copy; it replaces the bundled module at the next reload.
A copy of `prelude.lua` decides which API modules load at all.
A replaced module owns what it installs: a broken `user/lua/gband/win.lua` fails the load, and gband keeps the last configuration that loaded.
The files under `defaults/` are copies for reading; editing them changes nothing, and each process writes them again as it starts.
[The internals tutorial](internals/README.md) reads the prelude, every API module and every bundled plugin line by line.

Three API modules return a table of exports that other modules use, described below.
A copy that replaces one of them must keep its exports, with the same meaning, because the other modules require them.

### Exports of `require("gband.hl")`

| export | value |
|---|---|
| `drawn(name)` | the resolved style of the group `name`, as `gband.hl.get(name, { resolve = true })` returns it, with a named color turned into its palette index; the style tables of frames and bars take it as it is |
| `snapshot()` | a new table mapping each group to its explicit style, the one `gband.hl.set` gave it |
| `restore(saved)` | gives every group the explicit style `saved` holds for it, and none when `saved` holds none |
| `clear()` | removes every group's explicit style, keeping its default |
| `quiet` | a boolean, `false` at first; while it is `true`, changes to groups emit no `HighlightChanged` |

### Exports of `require("gband.settings")`

| export | value |
|---|---|
| `read(file, valid)` | runs `file`, a path relative to the configuration directory such as `user/theme.lua`, as Lua in an empty environment, and returns its value when `valid(value)` returns true; otherwise, and when the file is missing, broken or there is no configuration directory, nil |
| `write(file, text)` | replaces `file`, relative to the configuration directory, with `text` in one step; raises an error naming the file when it cannot |

### Exports of `require("gband.colorscheme")`

| export | value |
|---|---|
| `start()` | loads the saved theme, or `default` when there is none or it fails to load, as "Themes" describes |

## Core primitives: `gband.core`

`gband.core` holds the client's primitives: the small set of functions the Lua API modules are built from.
Configuration rarely needs them; they are there so that any part of the Lua API can be replaced or extended.
`gband.core` exists in the client only.
Each primitive checks its arguments, and a wrong one is an error at the line of the call that names the primitive, such as ``user/init.lua:4: gband.core.timer: expected a period in milliseconds as a non-negative integer, found string``.
"Callback" means code that runs where an action can be dispatched, as "Register only at top level" describes.

### Running code

| primitive | effect |
|---|---|
| `owner()` | the name of the plugin whose code is running, or nil for code of no plugin |
| `loading()` | `true` while the configuration loads |
| `dispatching()` | `true` while a callback runs |
| `failed(plugin)` | `true` when the plugin named `plugin` is failed, as "Errors" describes; `false` for nil |
| `call(owner, label, fn, ...)` | runs `fn` with the arguments as code of the plugin `owner`, or of no plugin when `owner` is nil, with an instruction limit of its own. It returns `true` and `fn`'s values. When `fn` fails, the failure is reported as a plugin error, preceded by `label` and a colon when `label` is given, and `call` returns `false` and the message. A plugin that hits the instruction limit is marked failed |
| `report(label, message, level)` | adds an error to the error list and the log: `message`, preceded by `label` and a colon when `label` is given, at the file and line of the function `level` levels up the stack when `level` is given, `1` being the function that calls `report` |
| `warn(text)` | writes `text` to the client log as a warning |
| `load(path)` | compiles the file at `path` and returns it as a function without running it; raises the compile error. Errors in the file then name `path` |
| `timer(ms, fn)` | runs `fn` as a callback of no plugin every `ms` milliseconds, at least 1, until cancelled, and returns the timer's id. An error in `fn` is reported and the timer goes on. A period missed while the client was busy is skipped |
| `cancel(id)` | stops the timer `id`; an id that is not running is ignored |

### Events and state

| primitive | effect |
|---|---|
| `events` | a list of the client's built-in event names |
| `emit(name, payload)` | runs the handlers of the built-in event `name` with a copy of the table `payload`, as "Events" describes, then every `after_event` function; a name that is not a built-in event is an error |
| `after_event(fn)` | adds `fn`, which runs after the handlers of each built-in event with the event's name, and with nil when a load finishes or the plugins are refreshed |
| `on_state(fn)` | adds `fn`, which runs with no arguments when the active key table, the viewed band's position, the number of bands or the error list changes |
| `state()` | a new table: `table`, the active key table; `band`, with the viewed band's `number`, its 1-based `index` and the band `count`; `window`, the focused window or nil; `width` and `height`, the terminal's size; `error`, the latest error message or nil; and `ribbon`, with the ribbon area's `cols` and `rows` |
| `error_marker(shown)` | records whether a bar draws the error marker; while it is `true`, the client draws no error banner |

### Colors and themes

| primitive | effect |
|---|---|
| `gband.core.palette.set(spec)` | replaces the terminal palette with `spec`, whose fields `fg`, `bg` and the sixteen color names of "The terminal palette" hold `#rrggbb`; a missing field leaves that color to the terminal |
| `gband.core.palette.get()` | a new table of the palette's fields that are set |
| `bundled(name)` | the bundled colorscheme `name` as a function that applies it, or nil |
| `colorschemes()` | a new list of the names of every `colors/<name>.lua` on the runtimepath, in byte order |
| `bundled_themes` | the list of the bundled themes' names, in the order "Themes" lists them, without the alias `catppuccin` |

### Plugin windows

| primitive | effect |
|---|---|
| `next_window()` | a plugin window number that this client never returned before, also across reloads |
| `present_window(id, frame)` | sets what the client draws for the plugin window `id`, as the frame table below describes |
| `forget_window(id)` | stops drawing the plugin window `id` |
| `request(entry)` | asks the server to open or close the drawn window of a tiled plugin window, as the entry table below describes; only in a callback |
| `focus_window(window)` | focuses the window `window`, as `gband.window.focus` does, but without checking that the client's layout holds it yet, since the client can report a tiled plugin window's drawn window before its layout does; only in a callback |
| `parse_key(name)` | the canonical form of the key name `name`, such as `ctrl+a` or `shift+tab`, as the plugin windows' `keys` use it, or nil when `name` is not a key name |
| `border(spec)` | the border table `spec`, as "Plugin windows" describes it, in canonical form: `sides`, a list of side names, and `chars`, a character set name or a list of 8 strings; or nil and a message |
| `width(value)` | a column width, a number from 0 to 1, as a numerator and a denominator; or nil and a message |
| `open_target(band, after)` | checks a target for a new tile, the `band` and `after` of `gband.win.open`, against the layout, and returns `true` and the band and the window to open after, each possibly nil; or `false` and a message |
| `removed(name)` | the message naming the replacement of an API name that a release removed, such as `` `kind = "pane"` is now `kind = "tiled"` ``, or nil |

A frame is a table:

| field | value |
|---|---|
| `kind` | `"floating"` or `"tiled"` |
| `base` | the style of the frame's cells |
| `lines` | a list of lines, each a list of runs; a run is `{ text = "...", style = { ... } }`, and control characters in `text` are dropped |
| `row`, `col`, `width`, `height` | floating only: the box, borders included, in terminal cells from the ribbon area's top left |
| `border` | floating only: `true`, `false`, nil or a border table |
| `title` | floating only: the title on the top border, or nil |
| `border_style`, `title_style` | floating only: the styles of the border and the title |
| `z` | floating only: the stacking order; a higher `z` is drawn over a lower one |
| `focused` | floating only: whether the window has focus |
| `cols`, `rows` | tiled only: the size of the drawn window's content |
| `hover` | optional: `true` for the client to report moves over the plugin window to the windows provider's `mouse` |

A style is a table of `fg` and `bg`, each `#rrggbb` or a palette index from 0 to 255, and the booleans `bold`, `italic`, `underline`, `reverse` and `dim`; a missing field is the default.

A request entry is `{ id = ..., op = "open" | "close" }`.
An open request also takes `band` and `after`, as `open_target` returns them, `num` and `den`, the column width, and `focus`, whether the new tile takes focus.
The client answers it through the windows provider's `opened`.

### Bars

| primitive | effect |
|---|---|
| `place_bars(slots, cols)` | places a list of slots against `cols` terminal columns, as "Side bars" describes, and returns a list holding, for each slot, its `{ col, width }` or `false` when it is not shown, then the ribbon area's `{ col, width }` |
| `present_bars(list)` | sets what the client draws for its bars, a list of bar tables |

A slot is `{ side = "left" | "right", size = ..., order = ..., seq = ... }`: the side, the width in columns, the order number, and a sequence number that breaks ties between equal orders.
A bar table is a slot with `id`, the bar's id, `base`, its style, and `lines`, lines of runs as a frame holds them.

### Settings and providers

| primitive | effect |
|---|---|
| `reopen_settings(line)` | remembers the settings window line to reopen after the next load succeeds, or forgets it for nil |
| `provide(kind, implementation)` | registers the implementation the client calls for `kind`, replacing an earlier one; every load starts with none |

The kinds of provider:

| kind | implementation | the client calls |
|---|---|---|
| `"windows"` | a table | the functions below |
| `"bars"` | a table | `flush()`, before it draws |
| `"settings"` | a table | `open(line)`, after a load that followed `reopen_settings(line)` succeeds |
| `"styles"` | a function | the function, before it draws; it returns `border`, `border_focused` and `banner`, the styles of window borders, of the focused window's border and of the error banner |
| `"decorations"` | a function | the function, with an info table, when it draws a window's top border; it returns the spans drawn at the right end of that border, as [Window decorations](#window-decorations) describes |

The windows provider's functions, each called when the event it names happens:

| function | called |
|---|---|
| `key(id, name, text)` | a key reaches the focused plugin window `id`: `name` is its canonical name, and `text` the character it types, or nil |
| `mouse(id, event)` | a mouse event reaches the plugin window `id`: `event` holds `kind` (`"press"`, `"release"`, `"drag"` or `"scroll"`, or `"move"` for a motion with no button held over a plugin window whose latest frame sets `hover`), `button` or `direction`, `content_col` and `content_row` or nil on the border, `box_col`, `box_row`, `box_width` and `box_height` or nil off the plugin window, and `ctrl`, `alt` and `shift` |
| `paste(id, text)` | text is pasted into the focused plugin window `id` |
| `set_box(id, col, row, width, height)` | the mouse moved or resized the floating plugin window `id` |
| `raise(id)` | a press on the plugin window `id` focuses it |
| `unfocus()` | focus moves to a window, so no floating plugin window has it |
| `close_focused()` | the window close action runs; returns `true` when it closed the focused floating plugin window, and the action then closes no window |
| `release()` | the mouse button is released |
| `opened(id, window)` | the server opened the drawn window `window` for the tiled plugin window `id`, or could not, with nil |
| `window_resized(id, cols, rows)` | the drawn window of the plugin window `id` changed size |
| `window_closed(id)` | the drawn window of the plugin window `id` closed |
| `ribbon_resized()` | the ribbon area changed size |
| `focused()` | the client asks which plugin window has focus; returns its id or nil |
| `plugin_window_of(window)` | the client asks which plugin window draws in `window`; returns its id or nil |
| `flush()` | after each callback, before the client draws |

A kind with no provider leaves its feature out: no plugin windows, no bars, no reopened settings window, the default styles, or no decorations.
An unknown kind, or an implementation of the wrong type, is an error at the line of the call.

#### Window decorations

The decorations function supplies text, such as buttons, for the right end of a window's top border.
The client calls it for each tile, lifted tile and floating window whose border draws the top side and whose top row is inside the ribbon area, a tiled plugin window's drawn window included.
It is not called for floating plugin windows or for the drop outline of a lifted tile.

It receives one argument, a new table:

| field | value |
|---|---|
| `window` | the window's number |
| `floating` | `true` when the window floats, `false` when it is tiled |
| `focused` | `true` when the window's border is drawn in the focused style |
| `width` | the width of the tile or box in cells, border included, as drawn in that frame |

It returns nil, or a list of spans.
A span is a string, or a table `{ text = <string>, style = <style table> }` whose `style` is optional and takes the fields of a frame's style.
Get a highlight group's style with `require("gband.hl").drawn(name)`.
Control characters are removed from the text, and each span is as wide as `gband.ui.width` measures its text.
Nil and an empty list draw nothing.

The client calls the function at most once per window for each frame.
It keeps each window's result, and draws it again without calling while the info table holds the same values and no other Lua code of the client has run since: no callback, timer, event handler, state function, other provider or load.
So a still screen calls nothing, a key press calls it once per window, and an animation that changes a box's width calls it for that window at each frame.
Keep the function cheap, and return the same spans for the same state and info.

Each call runs outside a callback, as code of no plugin, with its own instruction limit.
`gband.core.dispatching()` is `false` in it, and an action, `gband.spawn` or `gband.keymap.enter` is an error.

For a box `w` cells wide and spans `d` cells wide in all, the spans are drawn one after another from column `w − 2 − d`, counted from 0 at the left border, so the last one ends at column `w − 3`.
The corner cell and the cell beside it stay border at both ends.
In a 40-column box, `[_]`, `[□]` and `[X]` take columns 29 to 31, 32 to 34 and 35 to 37, and columns 38 and 39 show `─` and `╮`.
When `d` is greater than `w − 4`, no span is drawn at all, so the function can return fewer spans for a narrow box when it wants some shown.
The spans follow the box as its border does, also during an animation, and the screen's edge cuts them as it cuts the border.

While spans are drawn, the window's name is cut to `w − 4 − d` cells, so at least one border cell separates the two; otherwise it keeps `w − 2` cells.
A span without `style` is drawn in the window's border style, `WindowBorder` or `WindowBorderFocused`, and a span's `style` replaces only the fields it sets.
Decorations are drawn whether window titles are on or off.

A call that raises an error, reaches the instruction limit or returns anything other than nil or a list of spans fails.
The client reports it once, preceded by `decorations: `, such as `decorations: span 2: expected a string or a table with a string text, found integer`, and marks no plugin failed.
It then draws no decorations and calls the function no more until `gband.core.provide("decorations", fn)` registers one again or a load succeeds.

The client does not hit-test the spans.
A mouse binding finds the span under a press from the placement rule and the mouse fields `box_col`, `box_row` and `box_width`, as [Events](#events-gbandon-gbandaugroup-gbandemit) describes them, by calling the same function with an info table built from the event:

```lua
local function buttons(info)
  if info.floating then
    return { "[_]", "[□]", "[X]" }
  end
end
gband.core.provide("decorations", buttons)

local function span_at(spans, box_width, box_col)
  local widths, total = {}, 0
  for index, span in ipairs(spans or {}) do
    widths[index] = gband.ui.width(type(span) == "string" and span or span.text)
    total = total + widths[index]
  end
  if total == 0 or total > box_width - 4 then
    return nil
  end
  local first = box_width - 2 - total
  for index, width in ipairs(widths) do
    if box_col >= first and box_col < first + width then
      return index
    end
    first = first + width
  end
end

local function floats(window)
  for _, band in ipairs(gband.layout().bands) do
    for _, box in ipairs(band.floating) do
      if box.id == window then
        return true
      end
    end
  end
  return false
end

gband.keymap.set("root", "leftmouse", function(e)
  if e.target ~= "window" or e.box_row ~= 0 then
    return false
  end
  local spans = buttons({
    window = e.window,
    floating = floats(e.window),
    focused = e.window == gband.view().window,
    width = e.box_width,
  })
  if span_at(spans, e.box_width, e.box_col) == 3 then
    gband.action.close_window({ window = e.window })
  else
    return false
  end
end, { desc = "close a floating window from its [X]" })
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
It also takes an optional `floating`: `true` floats the window and leaves a floating window as it is, `false` tiles it and leaves a tiled window as it is, and without it the action toggles; `floating = true` cannot be given with `after`.

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

A plugin that counts across windows keeps its own count from these events, and can show it in a bar of its own:

```lua
local waiting = {}
local bar = gband.bar.add({ id = "waiting", side = "right", size = 3 })

local function draw()
  local count = 0
  for _ in pairs(waiting) do
    count = count + 1
  end
  gband.bar.set_lines(bar, { count > 0 and tostring(count) or "" })
end

gband.on("WindowStateChanged", function(ev)
  if ev.key == "agent" then
    waiting[ev.window] = ev.value == "waiting" or nil
    draw()
  end
end)

gband.on("WindowClosed", function(ev)
  waiting[ev.window] = nil
  draw()
end)
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

## API and stability

`gband.api_version` identifies the documented API: everything this guide and [Testing a plugin](testing.md) describe, whether the executable or a bundled Lua module provides it.
That covers the `gband` tables of both sides, `gband.core`, the providers and their functions, the exports of the API modules, the built-in events and their payloads.

A release raises `gband.api_version` by one when it removes a documented function, table, field, primitive, provider function, module export, event or payload field, or changes the documented arguments, return values, errors or effect of one.
A release that only adds to the documented API, or changes only what this guide does not describe, keeps it.
A plugin's `api` field, in "Plugin modules and `gband.plugin`", names the version it was written for.

What changed in each version:

| version | changes |
|---|---|
| 1 | the first version; nothing changed |
| 2 | a function bound to a mouse name that returns `false` declines the press or wheel step, which then takes what it would take with no binding, where version 1 let it replace the default |

## Still to come

Later changes will add their own extension points, each with its own API:

- a command palette, which will read `gband.cmd.list()` and `gband.action.list()`
- an SSH transport, so a client can attach to a server on another machine; plugins written to this guide need no change for it
