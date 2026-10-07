# 00 · The boundary

gband is a Rust executable with a Lua runtime inside it.
This chapter draws the line between the two: what the executable gives Lua, what gband's own Lua builds on top of it, and how the files that hold that Lua reach you.

## What the executable provides

Each load starts a new Lua state.
Before any Lua file runs, the executable installs the `gband` table.
Most of it is the API that [the plugin guide](../plugins.md) describes and that configuration calls: `gband.bind`, `gband.keymap`, `gband.opt`, `gband.on`, `gband.action`, `gband.layout`, `gband.window` and the rest.
Those fields are written in Rust and are the same in every load.

The client's `gband` also holds `gband.core`, a table of primitives.
A primitive is a small function that reaches into the client: it reads the client's state, registers something the client calls later, or hands the client something to draw.
None of them is a complete feature.
gband's highlight groups, colorschemes, bars, plugin windows, settings window and key styles are written in Lua on top of them, and that Lua is what this tutorial reads.
`gband.core` exists in the client only; the server's API is all Rust, and [the default server configuration](11-default-configs.md) is its only bundled Lua.

[Core primitives](../plugins.md#core-primitives-gbandcore) is the reference for every primitive.
Here they are by group, in the order the reference gives them, with the chapter where gband's own Lua uses each one.

### Running code

- `gband.core.owner` names the plugin whose code is running. `gband.bar` and `gband.win` record it as the owner of each bar and window a plugin makes, in [04](04-bars.md) and [05](05-plugin-windows.md).
- `gband.core.loading` is true while the configuration loads. Several modules stay quiet then: `gband.hl` emits no `HighlightChanged` in [02](02-highlights.md), and `gband.keystyle.use` refuses to run at any other time in [08](08-key-styles.md).
- `gband.core.dispatching` is true while a callback runs. `gband.win` uses it to tell a call from top-level code from a call from a binding.
- `gband.core.failed` tells whether a plugin is failed, so that `gband.win` and `gband.bar` stop drawing for it.
- `gband.core.call` runs a function as code of a plugin, with its own instruction limit, and reports its error instead of raising it. Every callback that gband's Lua runs on behalf of a plugin, such as a bar's `render`, goes through it.
- `gband.core.report` adds an error to the error list, and `gband.core.warn` writes a warning to the log.
- `gband.core.load` compiles a file without running it. `gband.colorscheme` loads `colors/<name>.lua` with it in [03](03-colorschemes.md).
- `gband.core.timer` and `gband.core.cancel` run a function every so many milliseconds until cancelled. No bundled file needs one; they are there for your own Lua.

### Events and state

- `gband.core.events` lists the client's built-in event names, and `gband.core.emit` runs the handlers of one of them. `gband.hl` emits `HighlightChanged` and `gband.colorscheme` emits `ColorschemeChanged` this way.
- `gband.core.after_event` adds a function that runs after the handlers of every built-in event. `gband.bar` and `gband.win` redraw from one, in [04](04-bars.md) and [06](06-window-provider.md).
- `gband.core.on_state` adds a function that runs when the key table, the viewed band or the error list changes, and `gband.core.state` returns that state. Bars and plugin windows read the state to size themselves, and the sidebar redraws on every change, in [09](09-sidebar-and-errors.md).
- `gband.core.error_marker` tells the client that a bar draws the error marker, so the client leaves out its own error banner. The sidebar calls it in [09](09-sidebar-and-errors.md).

### Colors and themes

- `gband.core.palette` holds `gband.core.palette.set` and `gband.core.palette.get`, which replace and read the terminal palette. `gband.palette` checks its argument and calls them in [02](02-highlights.md).
- `gband.core.bundled` returns a bundled colorscheme as a function, `gband.core.colorschemes` lists the colorschemes on the runtimepath, and `gband.core.bundled_themes` lists the bundled themes. [03](03-colorschemes.md) and [07](07-settings.md) use them.

### Plugin windows

- `gband.core.next_window` returns a new plugin window number.
- `gband.core.present_window` hands the client a frame to draw for a plugin window, and `gband.core.forget_window` stops drawing it.
- `gband.core.request` asks the server to open or close the drawn window of a tiled plugin window, and `gband.core.focus_window` focuses it.
- `gband.core.parse_key`, `gband.core.border`, `gband.core.width` and `gband.core.open_target` check a plugin's arguments the way the Rust API checks its own.
- `gband.core.removed` names the replacement of an API name that a release removed, so `gband.win` can say what to write instead.

[05](05-plugin-windows.md) and [06](06-window-provider.md) use all of them.

### Bars

- `gband.core.place_bars` places the bars against the terminal's width, and `gband.core.present_bars` hands the client what to draw. Both are in [04](04-bars.md).

### Settings and providers

- `gband.core.reopen_settings` asks the client to reopen the settings window after the next load. [07](07-settings.md) explains why.
- `gband.core.provide` registers a provider, which the next section describes.

## Providers

A primitive is a call from Lua into the client.
A provider goes the other way: an implementation, written in Lua, that the client calls.
`gband.core.provide(kind, implementation)` registers one.
Every load starts with none, so a feature whose provider nobody registers is simply missing.

| kind | registered by | the client calls it |
|---|---|---|
| `styles` | `gband.hl`, in [02](02-highlights.md) | before it draws, for the styles of window borders and the error banner |
| `bars` | `gband.bar`, in [04](04-bars.md) | before it draws, to place and draw the bars |
| `windows` | `gband.win`, in [06](06-window-provider.md) | when a key, a click, a paste or a resize reaches a plugin window, and before it draws |
| `settings` | `gband.settings`, in [07](07-settings.md) | after a load that asked to reopen the settings window |
| `decorations` | no bundled file | when it draws a window's top border, for the spans at its right end |

## The prelude

Before the init file runs, the client requires one module, `gband.prelude`, and runs nothing else of its own Lua.
Here is the whole of it, as gband writes it to `defaults/lua/gband/prelude.lua`:

```lua defaults/lua/gband/prelude.lua
require("gband.hl")
require("gband.palette")
require("gband.colorscheme")
require("gband.bar")
require("gband.win")
require("gband.settings")
require("gband.keystyle")
require("gband.colorscheme").start()
```

Each `require` installs one API module.
The order matters, because each module calls into those before it as it loads:

- `gband.hl` comes first, since every later module declares highlight groups.
- `gband.palette` and `gband.colorscheme` come next. A colorscheme sets groups and the palette.
- `gband.bar` and `gband.win` draw with highlight groups.
- `gband.settings` draws its window with `gband.win`.
- `gband.keystyle` reads the saved key style through `gband.settings`.

The last line loads the theme gband starts with, which `gband.settings` has saved.
An error anywhere in the prelude fails the load, exactly as an error in `user/init.lua` does.
The chapters follow the prelude's order from here on.

## `require` and the runtimepath

`require("gband.bar")` looks in each directory of `gband.runtimepath` in order, for `lua/gband/bar.lua` and then `lua/gband/bar/init.lua`.
The runtimepath starts with `user/` of the configuration directory, then every plugin directory.
Only when no directory holds the module does `require` fall back to the copy bundled inside the executable.
A bundled module's errors name the same path a file on the runtimepath would, such as `gband/bar.lua:12:`.

Colorschemes work the same way: `gband.colorscheme("gruvbox")` looks for `colors/gruvbox.lua` on the runtimepath before it asks for the bundled one, as [03](03-colorschemes.md) shows.

## The copies under `defaults/`

The bundled files live inside the executable, but each process writes a copy of them under `defaults/` of the configuration directory as it starts:

| path under `~/.config/gband/` | what it is |
|---|---|
| `defaults/init.lua`, `defaults/server.lua` | the default client and server configurations |
| `defaults/lua/gband/<name>.lua` | the prelude, the API modules and the bundled plugins |
| `defaults/lua/gband/theme/catppuccin.lua` | the color tables of the catppuccin themes |
| `defaults/keystyle/modal.lua`, `defaults/keystyle/direct.lua` | the two key style presets |
| `defaults/colors/<name>.lua` | the bundled colorschemes |

The copies are for reading.
`defaults/` is not on the runtimepath, nothing loads them, and each process writes them again when it starts, so editing them changes nothing.
They are byte for byte what the executable runs, and every quote in this tutorial is checked against them.
A quote names the copy it comes from, such as `defaults/lua/gband/bar.lua`, and the line numbers you see in an error, such as `gband/bar.lua:12:`, are line numbers of that copy.

## Replacing a bundled file

Because `require` searches the runtimepath first, a file of the same name under `user/` replaces a bundled one:

- Copy `defaults/lua/gband/sidebar.lua` to `user/lua/gband/sidebar.lua` and edit it. The next reload runs your copy instead of the bundled sidebar.
- Copy `defaults/keystyle/direct.lua` to `user/lua/gband/keystyle/direct.lua` to change the direct key style. The presets are the modules `gband.keystyle.modal` and `gband.keystyle.direct`, so the copy goes under `lua/gband/`.
- Copy `defaults/colors/gruvbox.lua` to `user/colors/gruvbox.lua` to change that theme.

A replaced module owns everything it installs.
A broken `user/lua/gband/win.lua` fails the load, and gband keeps the configuration that last loaded.
Three API modules return tables that the others use, listed in [the plugin guide](../plugins.md#the-prelude-and-the-api-modules); a replacement must keep them.
A copy of `prelude.lua` decides which API modules load at all, and [12](12-own-prelude.md) writes one.

## Where the files come from

The copies are written from these files of gband's repository, for contributors who want to change them:

| under `defaults/` | in the repository |
|---|---|
| `init.lua` | `crates/lua/src/defaults.lua` |
| `server.lua` | `crates/lua/src/defaults_server.lua` |
| `keystyle/modal.lua`, `keystyle/direct.lua` | `crates/lua/src/runtime/gband/keystyle/` |
| `lua/gband/<file>` | `crates/lua/src/runtime/gband/<file>` |
| `colors/<name>.lua` | `crates/lua/src/runtime/gband/colors/<name>.lua` |

gband's test suite checks that every line of the covered files is quoted in exactly one place in this tutorial.
A change to one of those files therefore changes the chapter that quotes it in the same commit.
