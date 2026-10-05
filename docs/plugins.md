# Writing a gband plugin

A gband plugin is a directory of Lua files.
gband finds it on the runtimepath, runs its startup files, and lets your configuration set it up.
Plugins run in the client.
They can add actions, commands, options, key bindings, event handlers, status line components, windows, highlight groups and colorschemes.
They can read the layout and act on any pane or band.

The [sample plugin](../examples/plugins/hello) uses most of what this guide describes.
The [status line sample](../examples/plugins/pane) adds a component, a highlight group and a colorscheme.

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
| `colors/<name>.lua` | colorschemes that `gband.colorscheme("<name>")` finds |

`require("a.b")` looks in each runtimepath entry in order, for `lua/a/b.lua` and then `lua/a/b/init.lua`.
When no entry holds the module, it looks among the modules bundled with gband, such as `gband.statusline.band`, and only then in Lua's own search path.
The first file found wins, so a module in `user/lua/` overrides a plugin's module of the same name, and `user/lua/gband/statusline/band.lua` overrides the bundled one.
Errors in a bundled module name its path under `gband/`, such as `gband/statusline/band.lua:12:`.

## Load order

Each load runs in a new Lua state:

1. gband sets `gband.side` to `"client"` and `gband.api_version` to `1`.
2. gband installs the rest of the `gband` API, including the status line and its built-in highlight groups, and loads the `default` colorscheme.
3. gband runs the init file: `user/init.lua` when it exists, the default configuration otherwise.
4. For each runtimepath entry in order, gband runs `plugin/*.lua` in byte order of the file names, then `plugin/client/*.lua` in the same order.

The init file can change `gband.runtimepath`, and step 4 uses the list as it stands.
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
An error there is a plugin error: gband records it in the log and shows the latest as `<plugin>: <file>:<line>: <message>`.
While the status line is drawn, the error shows as its first item.
While it is not, the error shows on the bottom row of the ribbon.

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
A run is the init file, one plugin file, one `setup`, one call of a callback, one colorscheme file, or one call of a status line component's `render`.
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
The built-in options are `prefix`, `default_column_width`, `width_presets`, `center_focused_column`, `statusline_position`, `statusline_height` and `statusline_separator`.

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
| `LayoutChanged` | empty | the bands, columns, panes or column widths change |
| `TerminalResized` | `cols`, `rows` | the client's terminal changes size |
| `ConfigReloaded` | empty | a reload succeeded, to the new configuration's handlers |
| `KeyTableChanged` | `table`, `previous` | the active key table changes |
| `HighlightChanged` | `group` | a group's settings change after loading |
| `ColorschemeChanged` | `name`, `previous` | a colorscheme loads after loading |

Attaching to a session emits no `PaneOpened`, `LayoutChanged`, `FocusChanged` or `BandChanged` for what is already there.
A layout that opens or closes a pane emits `LayoutChanged` after its `PaneOpened` and `PaneClosed` events.

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

## Layout and view: `gband.layout`, `gband.view`

`gband.layout()` returns a new table describing the layout the client holds:

- `cols` and `rows`: the size of the screen area.
- `bands`: one table per band, in order, each with `id` and `columns`.
- `columns`: one table per column, left to right, each with `width` as a number, `full_width` and `panes`.
- `panes`: one table per pane, top to bottom, each with `id` and either `rows`, a fixed height, or `weight`, an automatic height's weight. A plugin pane of a window this client opened also has `window`.

`gband.view()` returns `band`, the viewed band, `pane`, the focused pane or nil, `window`, the focused window or nil, `table`, the active key table, and `cols` and `rows`, the size of the ribbon.

Both can be called from any code that runs after loading.
They describe the state when they are called: an action the running callback dispatched takes effect only after it returns.

```lua
gband.keymap.set("prefix", "o", function()
  local view = gband.view()
  local count = 0
  for _, band in ipairs(gband.layout().bands) do
    for _, column in ipairs(band.columns) do
      count = count + #column.panes
    end
  end
  print(("band %d, pane %s, %d panes"):format(view.band, tostring(view.pane), count))
end, { desc = "describe the layout" })
```

## Action targets

A built-in action that acts on the focused pane also takes a target, a table naming the pane: `close_pane`, `consume_or_expel_left`, `consume_or_expel_right`, `cycle_column_width`, `toggle_full_width`, `grow_column_width`, `shrink_column_width`, `grow_pane_height`, `shrink_pane_height` and `reset_pane_height`.
`gband.action.close_pane({ pane = 3 })` closes pane 3, whichever pane is focused.

`open_pane` takes `band`, `after` or both: the new column follows `after`'s column, or is `band`'s first column when only `band` is given.
`send_prefix` takes `pane` and sends the prefix key to it.
`gband.spawn` takes `band` and `after` beside `cmd`.

A target on a view action or on `detach`, a field the action does not take, or a pane or band not in the layout is an error at the line of the call.

```lua
gband.keymap.set("prefix", "N", function()
  local view = gband.view()
  gband.action.open_pane({ band = view.band })
end, { desc = "open a pane as the band's first column" })
```

## Panes and bands: `gband.pane`, `gband.band`

These dispatch like actions, in the order they are called together with the callback's actions, and are errors outside a callback.
A pane or band number not in the layout is an error at the line of the call.

- `gband.pane.focus(pane)` views the pane's band and focuses it.
- `gband.band.view(band)` views a band, focusing the pane last focused there.
- `gband.pane.set_width(pane, width)` sets the column's width, a number greater than 0 and at most 10000, and turns full width off.
- `gband.pane.set_height(pane, { rows = n })` gives a fixed height; `gband.pane.set_height(pane, { weight = w })` gives an automatic height of weight `w`.
- `gband.pane.send_keys(pane, keys)` sends a key name, or a list of them, as key presses.
- `gband.pane.send_text(pane, text)` sends each character as a key press, a line feed or carriage return as Enter and a tab as Tab. Other control characters are an error.
- `gband.pane.paste(pane, text)` sends `text` as a paste.

Keys sent this way run no binding and leave the active key table alone.

```lua
gband.keymap.set("prefix", "e", function()
  local view = gband.view()
  local first = gband.layout().bands[1].columns[1].panes[1].id
  if first ~= view.pane then
    gband.pane.send_text(first, "make\n")
  end
end, { desc = "run make in the first pane" })
```

## Windows: `gband.win`

A window shows lines of styled text that Lua writes.
It is one of two kinds:

- a **float**, drawn by this client only, over the ribbon, at a position and size in cells. Floats change no layout, view or pane, and other clients never see them.
- a **pane**, a plugin pane in the shared layout. It has no program. It is placed, resized, moved, focused and closed like any pane, and every client sees its contents. The server drops keys and pastes sent to it, and it closes when the client that opened it detaches, disconnects or reloads.

`gband.win.open(opts)` opens a window and returns its number, which is never reused.
`info` and `list` can be called from any code that runs after loading; every other function only inside a callback.

| option | kinds | value | default |
|---|---|---|---|
| `kind` | both | `"float"` or `"pane"` | `"float"` |
| `lines` | both | the lines | none |
| `focus` | both | boolean | `true` |
| `cursorline` | both | boolean | `false` |
| `keys` | both | key names to functions | none |
| `on_close`, `on_resize` | both | function | none |
| `row`, `col` | float | integer of at least 0, or `"center"` | `"center"` |
| `width`, `height` | float | integer of at least 1 | half the ribbon |
| `border` | float | boolean | `true` |
| `title` | float | string | none |
| `band`, `after` | pane | as the `open_pane` target takes them | the viewed band and focused pane |
| `column_width` | pane | a width | `default_column_width` |

A line is a string or a list of spans; a span is a string or `{ text = ..., hl = "Group" }`.
Spans without `hl` use `Window`, and a span's style is its group's resolved style over `Window`'s.
Control characters are removed, and a line longer than the window is cut.

- `gband.win.set_lines(win, lines)` replaces the lines.
- `gband.win.scroll(win, count)` moves the first shown line, and `gband.win.set_cursor(win, line)` the cursor line. With `cursorline`, the window scrolls just enough to keep the cursor line shown and draws it in `WindowCursorLine`.
- `gband.win.focus(win)` focuses a float, or focuses a pane window's pane.
- `gband.win.set_config(win, config)` changes a float's `row`, `col`, `width`, `height`, `border` or `title`.
- `gband.win.close(win)` closes a window, and its pane for a pane window. Closing a window that is not open does nothing.
- `gband.win.info(win)` returns `id`, `kind`, `focused`, `pane`, `top`, `cursor`, `line_count`, `cols` and `rows`, and for a float `row`, `col`, `width` and `height`.
- `gband.win.list()` returns the open windows' numbers in ascending order.

The focused window is the focused float, otherwise the window of the focused pane.
A newly opened float with `focus` takes focus; moving focus or viewing another band leaves no float focused.
The exception is a focus change caused by the float's own `keys` function: the float stays focused until the next key press, whether the change happens at once or arrives later from the server, as an opened pane's focus does.
So a list can run actions on Enter and stay open for the next choice.
While a window is focused, every key the bindings leave unused goes to it and never to a pane, and pastes are discarded.
A key in `keys` runs its function with the window's number.
Without an entry, Up or `k` and Down or `j` move the cursor line or scroll, PageUp and PageDown scroll a page, Home and End show the first or last line, and Escape closes a float.

`on_close` runs once with the window's number when the window closes, but not when a reload closes it.
`on_resize` runs with the number and the new columns and rows when the window's content area changes size, including when a pane window's size first becomes known.
A window closes when its plugin is marked failed.

```lua
gband.keymap.set("prefix", "?", function()
  local lines = {}
  for _, binding in ipairs(gband.keymap.list("prefix")) do
    lines[#lines + 1] = {
      { text = ("%-10s"):format(binding.key), hl = "KeyHintKey" },
      binding.desc or binding.action or "function",
    }
  end
  local win
  win = gband.win.open({
    title = "prefix keys",
    width = 50,
    height = 15,
    cursorline = true,
    lines = lines,
    keys = { q = function() gband.win.close(win) end },
  })
end, { desc = "list the prefix keys" })

gband.keymap.set("prefix", "P", function()
  gband.win.open({ kind = "pane", column_width = 1/3, lines = { "notes", "", "- write the tests" } })
end, { desc = "open a notes pane" })
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
In the plugin `pane`, `id = "count"` gives `pane.count`, and a plugin adding one component can omit `id` to get `pane`.
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
| `column` | `{ index, count }`: the focused column's position counting from 1, and the band's column count; nil when the band is empty |
| `pane` | the focused pane's number, or nil |

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

The status line's error item shows the latest configuration or plugin error, first in the left region, in `StatusLineError`.
It is dropped last and cut like any last component.
It stays until the configuration next loads without errors.

### Bundled segments

gband bundles five segment plugins:

| module | plugin | shows | redraws on | align | priority | order | group |
|---|---|---|---|---|---|---|---|
| `gband.statusline.band` | `band` | `band ` and the viewed band's index | `BandChanged`, `LayoutChanged` | left | 20 | 10 | `StatusLineSegment` |
| `gband.statusline.mode` | `mode` | the active key table; hidden in `root` | `KeyTableChanged` | left | 30 | 20 | `StatusLineAccent` |
| `gband.statusline.hints` | `hints` | the keys of the active key table and what each does, below | `KeyTableChanged`, and as a fill component | left | 0 | 30 | `KeyHintLabel` |
| `gband.statusline.position` | `position` | the focused column and the column count, such as `3/7`; hidden in an empty band | `FocusChanged`, `BandChanged`, `LayoutChanged` | right | 10 | 10 | `StatusLineMuted` |
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
In `root` it starts with the prefix key labelled `prefix`, when the `prefix` table holds a binding, so the default line shows `C-space prefix`, and the prefix table's hints after Ctrl+Space.

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
| `focus_column_left` | `left` | `consume_or_expel_right` | `stack right` |
| `focus_column_right` | `right` | `cycle_column_width` | `width` |
| `focus_pane_down` | `down` | `toggle_full_width` | `full` |
| `focus_pane_up` | `up` | `grow_column_width` | `wider` |
| `focus_band_down` | `band down` | `shrink_column_width` | `narrower` |
| `focus_band_up` | `band up` | `grow_pane_height` | `taller` |
| `open_pane` | `new` | `shrink_pane_height` | `shorter` |
| `close_pane` | `close` | `reset_pane_height` | `reset height` |
| `consume_or_expel_left` | `stack left` | `detach` | `detach` |
| | | `send_prefix` | `send prefix` |

The segment fits itself to its `ctx.width`: it shows the leading hints that fit, followed by ` …` when some are left out, and hides itself when not even the first fits.
It is a fill component, so that width follows the other segments of the same render.

Its options, besides `align`, `priority` and `order`:

| option | value | default |
|---|---|---|
| `labels` | a table from action names to a label string, or `false` to leave the action out | `{}` |
| `root` | `false` hides the segment while `root` is active | `true` |

```lua
gband.plugin("gband.statusline.hints", { labels = { close_pane = "kill", send_prefix = false } })
```

Its groups are `KeyHintKey`, linked to `StatusLineAccent` by default, and `KeyHintLabel`, linked to `StatusLineSegment`.
A colorscheme or `gband.hl.set` can style them like any group.

### A component

```lua
local M = { name = "pane", api = 1 }

function M.setup()
  gband.hl.default("PaneSegment", { link = "StatusLineAccent" })
  gband.ui.statusline.add({
    align = "right",
    hl = "PaneSegment",
    redraw_on = { "FocusChanged" },
    render = function(ctx)
      return ctx.pane and ("pane " .. ctx.pane)
    end,
  })
end

return M
```

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

The window API defines these groups, as defaults:

| group | default | use |
|---|---|---|
| `Window` | `{}` | every window cell |
| `WindowBorder` | `{ fg = 8 }` | a float's border |
| `WindowTitle` | `{ bold = true }` | a float's title, over `WindowBorder` |
| `WindowCursorLine` | `{ reverse = true }` | the cursor line |

A change of any group redraws every window.

A span's style is its group's resolved style over `StatusLine`'s, field by field.

### Colors in the terminal

The client draws 24-bit colors when its own `COLORTERM` is `truecolor` or `24bit`.
Otherwise it draws each `"#rrggbb"` color as the nearest of the palette indexes 16 to 255.
Index and named colors are drawn as their palette index either way.
Floats follow these rules.
A plugin pane's contents are sent to the server as 24-bit colors, and every client shows them as its terminal does, as it shows a program's colors.

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
gband.hl.set("PaneSegment", { fg = "#f6c177", bold = true })
```

## Still to come

Later changes will add their own extension points, each with its own API:

- a server-side runtime, which will source `plugin/server/`
- a command palette, which will read `gband.cmd.list()` and `gband.action.list()`
- notifications
