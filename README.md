# gband

gband is a terminal multiplexer inspired by the [niri](https://github.com/YaLTeR/niri) window manager.
Like niri, it places windows on an infinitely side-scrolling strip.
Each strip belongs to a band (niri's workspace), and bands stack vertically without limit.
It is written in Rust and scriptable with Lua.

## Status

gband is usable and stable, but in early development.
Configuration, the Lua API and the plugin API can still change between releases.

## How it works

Multiplexers such as tmux and Zellij split a fixed screen area into panes.
Each new pane shrinks the panes already on screen.

gband follows niri's scrollable tiling model instead:

- Windows sit in columns on a strip that scrolls sideways without limit.
- A new window adds a column to the strip, and existing windows keep their size.
- The view scrolls sideways to follow focus, so the focused window is always on screen.
- A column can hold several windows stacked vertically.
- Each band has its own strip.
  Bands stack vertically, and the view slides up and down between them.
- An empty band always waits below the last one.
  Opening a window in it adds a new empty band below, and a band other than the last is removed when its last window closes.

Scrolling, band switches and resizes animate.
Set `GBAND_ANIMATIONS=off` before you attach to turn the animations off.

## Names

gband keeps niri's layout model but renames one of its parts:

| niri | gband |
|---|---|
| workspace | band |

Windows, columns and the strip keep their niri names.
What gband calls a window, tmux and Zellij call a pane.

A plugin window is something else: text that a plugin draws, either floating over the windows or tiled in a column like a window.
[docs/plugins.md](docs/plugins.md#plugin-windows-gbandwin) describes plugin windows.

## Usage

`gband` attaches to the session named `default`, and starts a server first when none runs.
`gband -s work` attaches to the session named `work` instead, and creates it when it does not exist.
`-S NAME` addresses a separate server named `NAME`.

The prefix key followed by `D` detaches, and the session keeps running on the server.
Run `gband` again to attach to it.

- `gband list-sessions` lists the sessions of the running server.
- `gband kill-session -s work` ends the session `work` and its windows.
- `gband kill-server` stops the server and every session.

Keys typed in gband go to the focused window: this is interactive mode.
The prefix key, Ctrl+Space, enters navigation mode, and the status line shows `navigation`.
In navigation mode each key below acts and navigation mode stays active, so `l` `l` `l` moves three columns and `=` `=` widens the column twice.
A key with no binding does nothing.
Escape or Enter returns to interactive mode, and so do `n`, `?` and Ctrl+Space once they have acted:

| key | action |
|---|---|
| `h`, `l`, or the left or right arrow | focus the column to the left or right |
| `j`, `k`, or the down or up arrow | focus the window below or above |
| `u`, `i` | view the band below or above |
| `c` | center the focused column in the view, or the focused floating window on the screen |
| `n` | open a window running your shell, and return to interactive mode |
| `q` | close the window, or the key list or another floating plugin window when it has focus |
| `[`, `]` | move the window into or out of the column to the left or right |
| `r` | cycle the column's width through the presets |
| `f` | toggle full width of the column |
| `-`, `=` | narrow or widen the column |
| `_`, `+` | shorten or heighten the window |
| `R` | reset the window's height |
| `v` | float the window, or tile it again |
| `V` | move focus between the floating and the tiled windows |
| Ctrl+`h`, Ctrl+`l`, or Ctrl with the left or right arrow | move the column, or the floating window, to the left or right |
| Ctrl+`j`, Ctrl+`k`, or Ctrl with the down or up arrow | move the window, or the floating window, down or up |
| `?` | list these keys, and run the one you choose; the list takes the keys that follow |
| `D` | detach |
| Escape, Enter | return to interactive mode |
| Ctrl+Space | send Ctrl+Space to the window, and return to interactive mode |

Navigation mode changed two habits.
Enter no longer opens a window: `n` does.
After Ctrl+Space and a layout key, what you type no longer reaches the window until Escape or Enter returns to interactive mode.

## Scripting

gband embeds a Lua runtime.
Configuration and automation are Lua scripts, so key bindings, layout behavior and custom commands are code you can change.

## Configuration

gband keeps its configuration in `$XDG_CONFIG_HOME/gband/`, or `~/.config/gband/` when `XDG_CONFIG_HOME` is unset.
gband creates the directory when it starts, with two folders in it:

- `defaults/init.lua` holds the full default client configuration, and `defaults/server.lua` the default server configuration.
  gband owns these files: it writes each when it is missing and overwrites it when its content differs from the defaults of the running build.
  Edits to them have no effect.
- `user/` holds your configuration.
  gband creates it empty.

The configuration has two files, one per process:

- `user/init.lua` configures each client: the prefix key, key bindings, the status line, colors and notifications.
- `user/server.lua` configures the server: the column widths, and the server halves of plugins.

The server reads its file from the machine it runs on, and each client reads its own.
Neither process runs the other's file.

When `user/init.lua` exists, gband runs it instead of the default client configuration, not on top of it.
Your file starts from the default options and no key bindings, so it must bind every key you want.
To start, copy `defaults/init.lua` to `user/init.lua` and edit the copy:

```sh
cp ~/.config/gband/defaults/init.lua ~/.config/gband/user/init.lua
```

Without `user/init.lua`, the defaults apply.
Saving `user/init.lua`, or any other `.lua` file under `user/`, reloads the configuration while gband runs, and deleting `user/init.lua` returns to the defaults.
An error in the file shows `error` at the top of the status line, and gband keeps the last configuration that loaded.
Ctrl+Space then the key you bind to `errors.open` lists the errors with their files and line numbers; see [Errors](#errors).

```lua
-- Set options one at a time through gband.opt.
gband.opt.prefix = "ctrl+b"

-- Or change several at once. Each call changes only the options it names.
gband.set {
  center_focused_column = "on-overflow",
  width_step = 1/20,
}

-- A root binding acts without the prefix key.
gband.keymap.set("root", "alt+h", gband.action.focus_column_left, { desc = "focus left" })
gband.bind("alt+l", gband.action.focus_column_right)

-- A prefix binding acts on the key pressed after the prefix key.
gband.keymap.set("prefix", "x", gband.action.close_window, { desc = "close the window" })
gband.bind("prefix c", gband.action.cycle_column_width)

-- Remove a binding made earlier, such as one copied from the defaults.
gband.unbind("prefix q")

-- A function binding can call actions and open windows running a command.
gband.bind("alt+n", function()
  gband.spawn { cmd = "htop" }
end)
gband.bind("alt+w", function()
  gband.action.focus_column_right()
  gband.action.focus_column_right()
end)
```

The client options, set in `user/init.lua`, and their defaults:

| option | value | default |
|---|---|---|
| `prefix` | one key | `"ctrl+space"` |
| `center_focused_column` | `"never"`, `"always"` or `"on-overflow"` | `"never"` |
| `loop_bands` | `true` or `false`: whether focus goes round from a band's last column to its first, drawing a long enough band as a loop | `true` |
| `tile_border_sides` | the sides of a tiled window's border that are drawn: a list of `"top"`, `"right"`, `"bottom"` and `"left"` | `{ "top", "right", "bottom", "left" }` |
| `tile_border_chars` | the border's characters: `"plain"`, `"rounded"`, `"double"`, `"thick"`, or a list of eight one-cell strings | `"plain"` |
| `floating_border_sides` | the drawn sides of a floating window's border | `{ "top", "right", "bottom", "left" }` |
| `floating_border_chars` | a floating window border's characters | `"plain"` |
| `width_step` | how much growing or shrinking changes a column's width, as a fraction of the screen | `1/10` |
| `height_step` | how much growing or shrinking changes a window's height, as a fraction of the screen, at most 1 | `1/10` |
| `notify_style` | how a plugin's desktop notification reaches your terminal: `"osc9"`, `"osc777"`, `"bell"` or `"none"` | `"osc9"` |

The server options, set in `user/server.lua`, and their defaults:

| option | value | default |
|---|---|---|
| `default_column_width` | the width of a new column, as a fraction of the screen | `1/2` |
| `width_presets` | the widths that cycling the column width steps through | `{ 1/3, 1/2, 2/3 }` |

```lua
-- user/server.lua
gband.opt.default_column_width = 1/3
gband.set { width_presets = { 1/3, 1/2, 2/3, 1 } }
```

Saving `user/server.lua` reloads the server's configuration; new widths apply to the columns opened after it, and existing columns keep theirs.
Setting an option in the other side's file is an error naming the side that owns it, so a width set in `user/init.lua` reports that it belongs in `user/server.lua`.
An error in `user/server.lua` shows in each client prefixed with `server: `.

To keep the old Ctrl+A prefix, or if your desktop takes Ctrl+Space for itself, put `gband.opt.prefix = "ctrl+a"` in `user/init.lua`.

Reading `gband.opt.<name>` returns an option's current value.
A value set through `gband.opt` that the option rejects is reported, does not stop the rest of the file, and resets the option to its default.
`gband.set` instead stops the file at the invalid value.

`gband.keymap.set(table, key, action, { desc = ... })` binds a key in a key table: `root` for keys pressed on their own, `prefix` for keys pressed after the prefix key, or a table of your own that a function enters with `gband.keymap.enter`.
`gband.bind` is the short form: `gband.bind("alt+h", ...)` binds in `root`, and `gband.bind("prefix h", ...)` binds in `prefix`.
`gband.keymap.del` and `gband.unbind` remove bindings, and `gband.keymap.list(table)` lists them with their descriptions.

`gband.keymap.mode(table, { label = ... })` makes a key table a mode: its keys act as often as you press them, and it stays active until one of its bindings enters another table.
`gband.keymap.enter("root")` returns to interactive mode.
The default configuration makes `prefix` a mode labelled `navigation`.
A `user/init.lua` that declares no mode keeps one-key prefix bindings: the key after the prefix acts once, and the next key goes to the window.
To get navigation mode in such a file, declare it and bind the keys that leave it:

```lua
gband.keymap.mode("prefix", { label = "navigation" })

local function interactive()
  gband.keymap.enter("root")
end

gband.keymap.set("prefix", "escape", interactive, { desc = "interactive mode" })
gband.keymap.set("prefix", "enter", interactive, { desc = "interactive mode" })
gband.keymap.set("prefix", "n", function()
  gband.action.open_window()
  interactive()
end, { desc = "open a window" })
```

A copy of `defaults/init.lua` keeps one-key prefix bindings when you delete its `gband.keymap.mode` line.

A key is a key name with optional `ctrl`, `alt` and `shift` modifiers joined by `+`, such as `alt+h`, `ctrl+PageUp` or `alt++`.
A key name is one character, or `enter`, `tab`, `backtab`, `backspace`, `escape`, `space`, the arrow keys `up`, `down`, `left` and `right`, `home`, `end`, `insert`, `delete`, `pageup`, `pagedown`, or `f1` to `f12`.
`prefix prefix` binds the prefix key pressed twice.

`gband.spawn { cmd = ... }` takes a command line as a string, which your shell runs, or a list of a program and its arguments.
Without `cmd`, it opens your shell.

The actions in `gband.action`:

| action | effect |
|---|---|
| `focus_column_left`, `focus_column_right` | focus the column to the left or right |
| `focus_window_down`, `focus_window_up` | focus the window below or above |
| `focus_band_down`, `focus_band_up` | view the band below or above |
| `center_column` | scroll the view so the focused column sits in its middle, or move the focused floating window to the middle of the screen |
| `switch_focus_floating_tiled` | move focus between the band's floating windows and its tiled windows |
| `open_window` | open a window running your shell right of the focused column |
| `close_window` | close the focused floating plugin window, such as the key list, when one has focus, otherwise the focused window |
| `consume_or_expel_left`, `consume_or_expel_right` | move the focused window into or out of the neighbouring column |
| `move_column_left`, `move_column_right` | swap the column with its neighbour, or move a floating window left or right |
| `move_window_down`, `move_window_up` | swap the window with its neighbour in the column, or move a floating window down or up |
| `toggle_window_floating` | float the focused window over the band, or tile it again |
| `cycle_column_width` | step the column's width through the presets |
| `toggle_full_width` | toggle full width of the column |
| `grow_column_width`, `shrink_column_width` | widen or narrow the column by `width_step`, a tenth of the screen by default |
| `grow_window_height`, `shrink_window_height`, `reset_window_height` | change or reset the height of the focused window |
| `detach` | detach the client |
| `send_prefix` | send the prefix key to the focused window |

`gband.action.list()` lists every action with its description.
A target table names the window an action acts on, and the four grow and shrink actions also take a `step`, so one binding can resize by another amount than the options give:

```lua
gband.bind("prefix W", function()
  gband.action.grow_column_width({ step = 1/4 })
end)
```

### Borders

Each client draws window borders with its own options, so two clients attached to the same session can draw them differently.
A side that is not drawn still takes its cell, which stays blank, so the border options never change a window's size:

```lua
gband.opt.tile_border_sides = { "top", "bottom" }
gband.opt.tile_border_chars = "rounded"
gband.opt.floating_border_chars = "double"
```

### Floating windows

Each band has a floating layer drawn over its columns, as niri's floating windows are.
A floating window is a full window with its own program, placed in a box that stays where it is while the band scrolls.
Ctrl+Space then `v` floats the focused window in a box centred on the screen, as wide as `default_column_width` and two height steps shorter than the screen, and the same keys tile it again as a new column right of the tiled window you focused last.
A window floated again returns to the box it had.
Ctrl+Space then `V` moves focus between the floating windows and the tiled windows of the band.

The other actions work on a floating window too.
The focus keys move between floating windows while the floating layer has focus.
The width keys change the box's width by the same steps as a column's, and the height keys change its height.
Ctrl+Space then Ctrl with `h`, `l`, `j` or `k`, or with an arrow key, moves the box a tenth of the screen; on a tiled window the same keys swap its column with the next column, or the window with the next window in its column.

Each client stacks floating windows in its own order, with the one it focused last on top, and draws its floating plugin windows above them.

### Key list

Ctrl+Space then `?` opens a list of the navigation keys in a box titled `navigation keys` over the windows, each with its description, and returns to interactive mode so the list takes the keys that follow.
Pressing a line's key selects that line and runs its binding, an action or a function, on the window behind the list, which stays open for the next choice: `l` focuses the column to the right, and `j` and `k` focus the window below and above.
Up and Down move through the list, PageUp, PageDown, Home and End jump through it, and Enter runs the selected line.
Lines whose key is one of these, and the line of the prefix key, run only through Enter.
The list's own `?` line shows dimmed, and pressing `?` or Enter on it only selects it.
`q` runs its line, close the window, which closes the list; Escape, or Ctrl+Space then `q`, close it too.

The list is a plugin bundled with gband, set up by the default configuration.
A `user/init.lua` that replaces the defaults sets it up and binds it itself:

```lua
gband.plugin("gband.keylist")
gband.keymap.set("prefix", "?", gband.action["keylist.open"], { desc = "list the keys" })
```

### Status line

The status line is a side bar at the left of the terminal, 20 columns wide by default and as tall as the terminal.
The windows keep the size they would have without it: they are drawn in the columns it leaves, and the view scrolls inside them, so a 1/2 column of an 80-column terminal is still 40 columns wide beside the bar.
By default it shows the viewed band at the top, such as `band 1`, the label of the active mode below it after the prefix key, such as `navigation`, hints for the keys of that mode, and the focused column at the bottom, such as `2/3`.
The hints show `C-space navigation` until the prefix key is pressed, then each key of navigation mode with a short label, such as `h left  l right`, wrapped onto as many rows as fit and ended with `…` when some are left out.
While a configuration or plugin error is reported, the first row shows `error` in red.

The status line is a plugin bundled with gband, `gband.statusline`, and each segment is another.
The default configuration sets them up.
A `user/init.lua` replaces the defaults, so it sets them up itself with the same calls; a copy of `defaults/init.lua` already holds them:

```lua
gband.plugin("gband.statusline")
gband.plugin("gband.statusline.band")
gband.plugin("gband.statusline.mode")
gband.plugin("gband.statusline.hints")
gband.plugin("gband.statusline.position")
```

Leaving out `gband.plugin("gband.statusline")` removes the status line, and the windows take the whole terminal.
Its options place and size it:

```lua
gband.plugin("gband.statusline", {
  side = "right",   -- "left" by default
  min_width = 16,   -- 20 by default
  max_width = 30,   -- 40 by default
  order = 0,        -- the order among bars on the same side
})
```

The status line widens with its widest segment, between `min_width` and `max_width`, and is not drawn when the terminal is too narrow for it.
The `statusline_*` options of earlier versions are gone: a file that still sets one gets an error naming the option and loads anyway.
Turning the status line off becomes leaving the plugin out, and placing it on a row becomes choosing a `side`.

Each segment takes the options `align`, `"top"`, `"center"` or `"bottom"`, `priority` and `order`, all but `hints` also take `hl`, and `gband.plugin("gband.statusline.clock")` adds a clock.
`hints` also takes `labels`, which renames or hides an action's hint, and `root = false`, which hides it until the prefix key.
`gband.colorscheme(name)` loads a colorscheme, and `gband.hl.set` styles any part of the line.
[docs/plugins.md](docs/plugins.md) describes the segments, highlight groups, colorschemes, writing your own segment and adding bars of your own with `gband.bar`.

### Errors

The client keeps every configuration and plugin error since the last load without one, oldest first, and `gband.errors()` returns them.
`gband.clear_errors()`, called from a binding function or another callback, empties the list and dismisses the error item or the banner until the next error.
It does not reload the configuration, so a status line component or a plugin that an error disabled stays disabled until the next load.
The logs keep every error, and other clients keep theirs: a server error still shows in each client that attaches.

The bundled `gband.errors` plugin, set up by the default configuration, registers the action and command `errors.open`, which list them in a floating window; `q` or Escape closes it.
It also registers the action and command `errors.clear`, which clear the errors.
In the error list, floating or tiled, `c` clears them and the list shows `no errors`; a floating list holding errors is titled `errors  c clear`.
No key is bound to either action by default:

```lua
gband.keymap.set("prefix", "e", gband.action["errors.open"], { desc = "list the errors" })
gband.keymap.set("prefix", "C", gband.action["errors.clear"], { desc = "clear the errors" })
-- or in a tiled window
gband.bind("prefix E", function() gband.cmd.run("errors.open", { kind = "tiled" }) end)
```

`gband.plugin("gband.errors", { kind = "tiled" })` makes `errors.open` open a tiled window.
Without a status line, the latest error shows on the bottom row of the windows instead.

### Plugins

gband loads plugins from `$XDG_DATA_HOME/gband/plugins/`, or `~/.local/share/gband/plugins/`.
A plugin has a manifest, `plugin.lua`, and a `client.lua` that each client runs, a `server.lua` that the server runs, or both.
In the client, a plugin can add actions, commands, options, key bindings, event handlers, status line segments, plugin windows, notifications and colorschemes.
In the server, it can watch window output and input, keep state per window, emit events to clients, queue them while no client is attached, and answer commands clients call.
[docs/plugins.md](docs/plugins.md) explains how to write one.
[examples/plugins/hello](examples/plugins/hello) is a sample to start from, [examples/plugins/window](examples/plugins/window) adds a status line segment and a colorscheme, and [examples/plugins/agent-status](examples/plugins/agent-status) notifies you when a coding agent in a window waits for an answer.
`gband test` runs a plugin's Lua tests against a real client and server in a terminal of their own, and compares what they draw with committed screenshots; [docs/testing.md](docs/testing.md) explains how to write them.

## Installing

Install a stable Rust toolchain, version 1.89 or newer, and a C compiler such as `gcc` or `clang`.
The C compiler builds the Lua runtime that gband bundles, so no system Lua is needed.

Install release 0.1.0 with Cargo:

```sh
cargo install --locked --git https://github.com/gmteixeira-personal/gband --tag v0.1.0
```

Cargo puts the `gband` binary in `~/.cargo/bin/`, which must be on your `PATH`.
To install the development version instead, replace `--tag v0.1.0` with `--branch dev`.

## Building

To build from a clone, run:

```sh
git clone https://github.com/gmteixeira-personal/gband.git
cd gband
cargo build --release
```

The binary is `target/release/gband`.
`cargo install --locked --path .` installs it into `~/.cargo/bin/` instead.

## Shell completions

gband completes its subcommands and options in fish, bash and zsh.
Install the script where your shell loads it:

```sh
gband install-completions fish   # or bash, zsh
```

The command prints the path it wrote.
fish and bash load the script in new shells.
zsh searches no per-user directory by default, so add the printed directory to `fpath` in `~/.zshrc` before `compinit` runs:

```zsh
fpath=(~/.local/share/zsh/site-functions $fpath)
autoload -Uz compinit && compinit
```

To put the script somewhere else, for example where a zsh framework expects it, print it instead:

```sh
gband completions zsh > ~/.oh-my-zsh/completions/_gband
```

## Branches

- `dev` is the default branch.
  All development happens there.
- Every other working branch forks from `dev` and merges back into it.
- `main` holds releases only.
  It moves forward only when a release is made from `dev`.

Branch from `dev` and open pull requests against `dev`.
Never commit to `main` directly or branch from it.

## Acknowledgements

The layout model comes from [niri](https://github.com/YaLTeR/niri), a scrollable-tiling Wayland compositor.
