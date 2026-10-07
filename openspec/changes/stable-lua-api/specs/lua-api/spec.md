## Purpose

Defines the boundary between gband's executable and Lua. The executable provides a small, stable, documented set of primitives, and gband's own Lua builds the rest of the API on them. No Lua file uses anything private, so users can read, copy and replace any bundled part of gband. A rule keeps the whole documented API stable between releases.

## ADDED Requirements

### Requirement: One public API
A Lua chunk that gband runs reaches gband only through documented API. This holds for a configuration file, a plugin file, a module, a colorscheme, a test file and a bundled file alike. The documented API SHALL be:
- the `gband` global of the chunk's process, `gband.core` included;
- the modules that `require` finds, as the plugins capability defines, and their documented exports;
- in a test file, the module `gband.test`.

gband SHALL pass no chunk an argument that a module found on the runtimepath would not receive. It SHALL set no global and no `package.loaded` entry that the documentation does not describe. No function the executable provides SHALL be reachable from Lua except through a documented path.

#### Scenario: Bundled chunk receives nothing private
- **WHEN** gband loads the bundled module `gband.sidebar`
- **THEN** the chunk's `...` holds only what `require` passes to a module found on the runtimepath: the module name and its path

#### Scenario: Every primitive is public
- **WHEN** `user/init.lua` reads `gband.core.owner`, `gband.core.provide` and `gband.core.present_window`
- **THEN** each is a function

### Requirement: Core primitives
The client SHALL provide the table `gband.core`, holding the primitives below. Each primitive SHALL check its arguments and raise an error at the line of the call when one has the wrong type or value. "Callback" means code that runs where an action can be dispatched, as the configuration capability defines.

| primitive | effect |
|---|---|
| `owner()` | returns the name of the plugin whose code is running, or nil for code of no plugin |
| `loading()` | returns `true` while the configuration loads |
| `dispatching()` | returns `true` while a callback runs |
| `failed(plugin)` | returns `true` when the plugin named `plugin` is marked failed, as the plugins capability defines, and `false` for nil |
| `call(owner, label, fn, ...)` | runs `fn` with the arguments as code of the plugin `owner`, or of no plugin when `owner` is nil, with its own instruction budget. It returns `true` and `fn`'s values, or reports the failure as the plugins capability defines, under `label` when given, and returns `false` and the message |
| `report(label, message, level)` | reports an error with `message`, preceded by `label` and a colon when `label` is given, located at the file and line of the stack level `level` when given |
| `warn(text)` | records `text` in the client log as a warning |
| `load(path)` | compiles the file at `path` as Lua source text and returns the chunk, or raises the compile error |
| `bundled(name)` | returns the bundled colorscheme `name` as a chunk, or nil |
| `colorschemes()` | returns a new list of the names of every `colors/<name>.lua` on the runtimepath, sorted by byte value |
| `bundled_themes` | the list of the bundled themes' names, in the colorschemes capability's order |
| `events` | the list of the client's built-in event names |
| `emit(name, payload)` | runs the handlers of the built-in event `name` with `payload`, as the lua-events capability defines, then every `after_event` function |
| `after_event(fn)` | adds `fn`, which runs with the event's name after each built-in event's handlers |
| `on_state(fn)` | adds `fn`, which runs when the active key table, the viewed band's position, the number of bands or the error list changes |
| `state()` | returns a new table: `table`, the active key table; `band`, with the viewed band's `number`, its 1-based `index` and the band `count`; `window`, the focused window or nil; `width` and `height`, the terminal's size; `error`, the latest error or nil; and `ribbon`, with the ribbon area's `cols` and `rows` |
| `timer(ms, fn)` | runs `fn` as a callback every `ms` milliseconds, at least 1, until cancelled, and returns its id |
| `cancel(id)` | cancels the timer `id` |
| `palette.set(spec)`, `palette.get()` | set and read the terminal palette, as the colorschemes capability defines |
| `next_window()` | returns a plugin window number never returned before in this client |
| `present_window(id, frame)`, `forget_window(id)` | set, or remove, what the client draws for the plugin window `id` |
| `request(entry)` | asks the server to open or close a tiled plugin window's drawn window |
| `focus_window(window)` | focuses the window `window` as `gband.window.focus` does, without checking that the client's layout holds it yet |
| `parse_key(name)` | returns the canonical form of the key name `name`, or nil when it is not a key name |
| `border(spec)` | returns a border spec in its canonical form, or nil and a message |
| `width(value)` | returns a column width as a numerator and a denominator, or nil and a message |
| `open_target(band, after)` | returns `true` and the band and window a new tile goes to, or `false` and a message |
| `removed(name)` | returns the message that names the replacement of a removed API name, or nil |
| `place_bars(slots, cols)` | places bars against `cols` columns, as the bars capability defines, and returns each slot's columns, or `false` when not shown, then the ribbon area's columns |
| `present_bars(list)` | sets what the client draws for its bars |
| `error_marker(shown)` | records whether a bar draws the error marker, so the client draws no error banner while it does |
| `reopen_settings(line)` | remembers the settings window line to reopen after the next load succeeds, or forgets it for nil |
| `provide(kind, implementation)` | registers an implementation, as "Providers" defines |

The frame, entry, slot and bar tables SHALL have the fields the API documentation gives. The documentation SHALL describe every field, every return value and every error of each primitive. A primitive SHALL do exactly what gband did through the private table that this change removes.

#### Scenario: Owner inside a plugin
- **WHEN** the plugin `hello`'s `setup` calls `gband.core.owner()`
- **THEN** it returns `hello`

#### Scenario: Isolated call
- **WHEN** a binding function calls `gband.core.call(nil, nil, function() while true do end end)` and then focuses the column to the left
- **THEN** the call returns `false` and a message naming the instruction limit, and the column to the left is focused

#### Scenario: Wrong argument
- **WHEN** line 4 of `user/init.lua` calls `gband.core.timer("soon", print)`
- **THEN** loading fails with an error at `user/init.lua` line 4

#### Scenario: Timer
- **WHEN** a binding function starts `gband.core.timer(100, fn)` and cancels it after `fn` ran twice
- **THEN** `fn` runs no more

#### Scenario: Focus before the layout holds the window
- **WHEN** the client reports that a tiled plugin window's drawn window is window 5, the layout the client last gave Lua does not hold window 5, and a binding function calls `gband.win.focus` on the plugin window
- **THEN** the client focuses window 5 and reports no error

#### Scenario: Several state functions
- **WHEN** two plugins each call `gband.core.on_state` and the viewed band changes
- **THEN** both functions run

### Requirement: Providers
`gband.core.provide(kind, implementation)` SHALL register the implementation the client calls for `kind`. A later call for the same kind SHALL replace the earlier implementation. A reload SHALL clear every registration before the new configuration loads. The kinds and the functions the client calls on them are:

| kind | implementation | the client calls |
|---|---|---|
| `"windows"` | a table | `key(id, name, text)`, `mouse(id, event)`, `set_box(id, col, row, width, height)`, `raise(id)`, `unfocus()`, `paste(id, text)`, `close_focused()`, `release()`, `opened(id, window)`, `window_resized(id, cols, rows)`, `window_closed(id)`, `ribbon_resized()`, `focused()`, `plugin_window_of(window)` and `flush()`, when the plugin window input, layout or drawing events they name happen |
| `"bars"` | a table | `flush()`, before the client draws |
| `"settings"` | a table | `open(line)`, after a load that followed `reopen_settings(line)` succeeds |
| `"styles"` | a function | the function, before the client draws. It returns `border`, `border_focused` and `banner`: the drawn styles of window borders, focused borders and the error banner |

The API documentation SHALL give each function's arguments, return value and the moment the client calls it. A kind with no registration SHALL make the client do what it does without that feature: draw no plugin windows, no bars, no reopened settings window, or default styles. An unknown kind, or an implementation of the wrong type, SHALL raise an error at the line of the call.

#### Scenario: Bundled plugin windows
- **WHEN** the default configuration is in use
- **THEN** the windows provider is the bundled `gband.win` module's, and `gband.win.open({ lines = { "hi" } })` draws a plugin window

#### Scenario: Replaced bar implementation
- **WHEN** `user/lua/gband/bar.lua` is a copy of the bundled module whose `flush` also records each call, and the default configuration is in use
- **THEN** the sidebar is drawn as with the bundled module, and the record grows with each frame

#### Scenario: Unknown kind
- **WHEN** line 2 of `user/init.lua` calls `gband.core.provide("menus", {})`
- **THEN** loading fails with an error at `user/init.lua` line 2

### Requirement: Lua prelude
Before the init file runs, the client SHALL require the module `gband.prelude` and run nothing else of its own Lua. The bundled prelude SHALL require `gband.hl`, `gband.palette`, `gband.colorscheme`, `gband.bar`, `gband.win`, `gband.settings` and `gband.keystyle`, in that order. Each of these modules installs its part of the `gband` API and registers its providers. The prelude SHALL then load the start theme through the `start` export of `gband.colorscheme`. Because `require` searches the runtimepath first, a module of the same name in a runtimepath entry SHALL replace the bundled prelude or any of these modules. An error raised while the prelude loads SHALL fail the load, as an error in the init file does.

#### Scenario: Defaults through the prelude
- **WHEN** no runtimepath entry holds `lua/gband/prelude.lua` and a client attaches
- **THEN** `gband.win`, `gband.bar`, `gband.hl`, `gband.colorscheme`, `gband.palette`, `gband.settings` and `gband.keystyle` are installed before `user/init.lua` runs

#### Scenario: API module replaced
- **WHEN** `user/lua/gband/win.lua` is a copy of the bundled module whose `open` also calls `gband.notify("opened")`, and a binding function calls `gband.win.open({ lines = { "hi" } })`
- **THEN** the plugin window opens and the client notifies `opened`

#### Scenario: Broken prelude override
- **WHEN** the client runs with a good configuration and the user saves a `user/lua/gband/prelude.lua` that raises `boom`
- **THEN** the reload fails, the error names `user/lua/gband/prelude.lua`, and the previous configuration stays in use

### Requirement: Documented API
Every documented part of the API SHALL be described in gband's API documentation. That covers, with only the API installed and before any configuration file runs:
- every field of the client's and the server's `gband` table, `gband.core` included;
- every field of each table reachable from those through named fields;
- every export of the API modules the prelude requires;
- every built-in event of each side;
- every field of `gband.core` and of `require("gband.test")` on the test side.

The documentation is `docs/plugins.md` for the client and the server, built-in actions included, and `docs/testing.md` for the test side. It SHALL give each function's arguments, return values and errors, each event's payload, and each value's meaning. A field that the documentation does not describe SHALL NOT exist.

#### Scenario: Every client field documented
- **WHEN** a client has installed its API and run no configuration file, and a test walks its `gband` table, every table reachable from it through named fields, and the export tables of the prelude's modules, collecting each path such as `gband.core.present_window`
- **THEN** `docs/plugins.md` names every collected path

#### Scenario: Every server field documented
- **WHEN** a test walks the server's `gband` table the same way
- **THEN** `docs/plugins.md` names every collected path

#### Scenario: Every test field documented
- **WHEN** a test walks the test side's `gband.core` and the table `require("gband.test")` returns
- **THEN** `docs/testing.md` names every collected field

### Requirement: Bundled Lua consumes the API
Every Lua file bundled with gband SHALL use only the documented API, and none SHALL receive anything private. This covers:
- the default client and server configurations;
- the prelude and the API modules;
- the key style presets;
- the bundled plugins `gband.sidebar`, `gband.keylist`, `gband.errors` and `gband.prompt`;
- the modules `gband.theme`, `gband.theme.catppuccin` and `gband.keyform`;
- every bundled colorscheme;
- on the test side, `gband.test`.

A copy of any of these files, placed where the runtimepath finds it first, SHALL behave as the bundled file does: a module at `lua/<path>.lua` of a runtimepath entry, and a colorscheme at `colors/<name>.lua`. The one difference is the settings capability's theme list, which names every colorscheme on the runtimepath, so a copy of the alias `catppuccin` is listed where the bundled alias is not.

#### Scenario: Copied sidebar
- **WHEN** `user/lua/gband/sidebar.lua` holds the bundled sidebar's text, the default configuration is in use, and an error is reported
- **THEN** the sidebar shows the mode letter, the band labels and `!`, exactly as the bundled sidebar does
- **AND** no banner is drawn

#### Scenario: Copied prompt
- **WHEN** `user/lua/gband/prompt.lua` holds the bundled prompt's text and the user runs `error("boom")` from the Lua prompt
- **THEN** the client reports the error `prompt:1: boom`

#### Scenario: Copied theme
- **WHEN** `user/colors/nord.lua` holds the bundled `nord` theme's text and `user/init.lua` calls `gband.colorscheme("nord")`
- **THEN** every group and the palette resolve as they do with the bundled `nord`

#### Scenario: Every bundled file copied
- **WHEN** gband's Lua spec suite runs with every bundled module copied into `user/lua/` and every bundled colorscheme other than the alias `catppuccin` into `user/colors/` of each case's configuration directory
- **THEN** every case passes, with the same screens as without the copies

### Requirement: Bundled sources on disk
Each process SHALL write the text of every bundled module to `defaults/lua/gband/` of the configuration directory, at the module's path, except the key style presets: a module `gband.a.b` goes to `defaults/lua/gband/a/b.lua`. This includes the prelude and the API modules. It SHALL write every bundled colorscheme to `defaults/colors/<name>.lua`. It SHALL write them as the configuration capability's "Configuration directory" writes the files under `defaults`. They SHALL be copies for the user to read: loading SHALL use the built-in text, never these files. `defaults` SHALL NOT be a runtimepath entry.

#### Scenario: Sources written
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `/tmp/cfg/gband` does not exist, and a client attaches
- **THEN** `/tmp/cfg/gband/defaults/lua/gband/sidebar.lua`, `/tmp/cfg/gband/defaults/lua/gband/win.lua` and `/tmp/cfg/gband/defaults/lua/gband/prelude.lua` hold the bundled text of those modules
- **AND** `/tmp/cfg/gband/defaults/colors/nord.lua` holds the bundled `nord` theme's text

#### Scenario: Copies have no effect
- **WHEN** the user edits `defaults/lua/gband/sidebar.lua` and reloads
- **THEN** the sidebar behaves as the built-in text defines

#### Scenario: Copy to override
- **WHEN** the user copies `defaults/lua/gband/sidebar.lua` to `user/lua/gband/sidebar.lua` and changes its mode letter for `root` from `I` to `i`
- **THEN** after the reload the sidebar's first row shows `i` in interactive mode

### Requirement: API version rule
`gband.api_version` SHALL identify the documented API, whether the executable or a bundled Lua module provides a part of it. A build SHALL raise `gband.api_version` by one when it removes a documented function, table, field, primitive, provider function, module export, event or payload field, or changes the documented arguments, return values, errors or effect of one. A build that only adds to the documented API, or changes only what the documentation does not describe, SHALL keep it. The API documentation SHALL state this rule, and SHALL list what changed for each value of `gband.api_version` above 1.

#### Scenario: Version after this change
- **WHEN** `user/init.lua` reads `gband.api_version`
- **THEN** it reads `1`

#### Scenario: Rule documented
- **WHEN** a reader opens the "API and stability" section of `docs/plugins.md`
- **THEN** it states when `gband.api_version` changes and lists no change for version 1
