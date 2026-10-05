# gband

gband is a terminal multiplexer inspired by the [niri](https://github.com/YaLTeR/niri) window manager.
It places panes (niri's windows) on an infinitely side-scrolling strip.
Each strip belongs to a band (niri's workspace), and bands stack vertically without limit.
It is written in Rust and scriptable with Lua.

## Status

gband is in early development and is not usable yet.

## How it works

Multiplexers such as tmux and Zellij split a fixed screen area.
Each new pane shrinks the panes already on screen.

gband follows niri's scrollable tiling model instead:

- Panes sit in columns on a strip that scrolls sideways without limit.
- A new pane adds a column to the strip, and existing panes keep their size.
- The view scrolls sideways to follow focus, so the focused pane is always on screen.
- A column can hold several panes stacked vertically.
- Each band has its own strip.
  Bands stack vertically, and the view slides up and down between them.
- An empty band always waits below the last one.
  Opening a pane in it adds a new empty band below, and a band other than the last is removed when its last pane closes.

Scrolling, band switches and resizes animate.
Set `GBAND_ANIMATIONS=off` before you attach to turn the animations off.

## Names

gband keeps niri's layout model but renames two of its parts:

| niri | gband |
|---|---|
| window | pane |
| workspace | band |

Columns and the strip keep their niri names.

In gband, a window is something else: text that a plugin draws, either as a float over the panes or as a pane of its own.
[docs/plugins.md](docs/plugins.md#windows-gbandwin) describes windows.

## Usage

`gband` attaches to the session named `default`, and starts a server first when none runs.
`gband -s work` attaches to the session named `work` instead, and creates it when it does not exist.
`-S NAME` addresses a separate server named `NAME`.

The prefix key followed by `D` detaches, and the session keeps running on the server.
Run `gband` again to attach to it.

- `gband list-sessions` lists the sessions of the running server.
- `gband kill-session -s work` ends the session `work` and its panes.
- `gband kill-server` stops the server and every session.

The default key bindings all follow the prefix key, Ctrl+Space:

| key | action |
|---|---|
| `h`, `l` | focus the column to the left or right |
| `j`, `k` | focus the pane below or above |
| `u`, `i` | view the band below or above |
| `enter` | open a pane running your shell |
| `q` | close the pane |
| `[`, `]` | move the pane into or out of the column to the left or right |
| `r` | cycle the column's width through the presets |
| `f` | toggle full width of the column |
| `-`, `=` | narrow or widen the column |
| `_`, `+` | shorten or heighten the pane |
| `R` | reset the pane's height |
| `D` | detach |
| Ctrl+Space | send Ctrl+Space to the pane |

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
An error in the file shows in the status line with its line number, and gband keeps the last configuration that loaded.

```lua
-- Set options one at a time through gband.opt.
gband.opt.prefix = "ctrl+b"

-- Or change several at once. Each call changes only the options it names.
gband.set {
  center_focused_column = "on-overflow",
  statusline_position = "top",
}

-- A root binding acts without the prefix key.
gband.keymap.set("root", "alt+h", gband.action.focus_column_left, { desc = "focus left" })
gband.bind("alt+l", gband.action.focus_column_right)

-- A prefix binding acts on the key pressed after the prefix key.
gband.keymap.set("prefix", "x", gband.action.close_pane, { desc = "close the pane" })
gband.bind("prefix c", gband.action.cycle_column_width)

-- Remove a binding made earlier, such as one copied from the defaults.
gband.unbind("prefix q")

-- A function binding can call actions and open panes running a command.
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
| `statusline_position` | `"bottom"`, `"top"` or `"off"` | `"bottom"` |
| `statusline_height` | the rows the status line takes, from 1 to 8 | `1` |
| `statusline_separator` | the text between two status line segments | `" │ "` |
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

A key is a key name with optional `ctrl`, `alt` and `shift` modifiers joined by `+`, such as `alt+h`, `ctrl+PageUp` or `alt++`.
A key name is one character, or `enter`, `tab`, `backtab`, `backspace`, `escape`, `space`, the arrow keys `up`, `down`, `left` and `right`, `home`, `end`, `insert`, `delete`, `pageup`, `pagedown`, or `f1` to `f12`.
`prefix prefix` binds the prefix key pressed twice.

`gband.spawn { cmd = ... }` takes a command line as a string, which your shell runs, or a list of a program and its arguments.
Without `cmd`, it opens your shell.

The actions in `gband.action`:

| action | effect |
|---|---|
| `focus_column_left`, `focus_column_right` | focus the column to the left or right |
| `focus_pane_down`, `focus_pane_up` | focus the pane below or above |
| `focus_band_down`, `focus_band_up` | view the band below or above |
| `open_pane` | open a pane running your shell right of the focused column |
| `close_pane` | close the focused pane |
| `consume_or_expel_left`, `consume_or_expel_right` | move the focused pane into or out of the neighbouring column |
| `cycle_column_width` | step the column's width through the presets |
| `toggle_full_width` | toggle full width of the column |
| `grow_column_width`, `shrink_column_width` | widen or narrow the column by a tenth of the screen |
| `grow_pane_height`, `shrink_pane_height`, `reset_pane_height` | change or reset the height of the focused pane |
| `detach` | detach the client |
| `send_prefix` | send the prefix key to the focused pane |

`gband.action.list()` lists every action with its description.

### Status line

The status line takes the bottom row of the terminal, and the panes get the rows above it.
By default it shows the viewed band on the left, such as `band 1`, the active key table after the prefix key, such as `prefix`, hints for the keys of that table, and the focused column on the right, such as `2/3`.
The hints show `C-space prefix` until the prefix key is pressed, then each key of the prefix table with a short label, such as `h left  l right`, cut with `…` when the line is full.
The latest configuration or plugin error shows first, in red.

`statusline_position = "top"` moves it to the top row, and `"off"` removes it, so the panes get the whole terminal.
`statusline_height` gives it more rows.
Both apply on the next reload, and the panes are resized to match.

Each segment is a plugin bundled with gband, set up by the default configuration.
A `user/init.lua` replaces the defaults, so it sets the segments up itself with the same calls; a copy of `defaults/init.lua` already holds them:

```lua
gband.plugin("gband.statusline.band")
gband.plugin("gband.statusline.mode")
gband.plugin("gband.statusline.hints")
gband.plugin("gband.statusline.position")
```

Without these calls the status line is drawn empty.
Each takes the options `align`, `priority` and `order`, all but `hints` also take `hl`, and `gband.plugin("gband.statusline.clock")` adds a clock.
`hints` also takes `labels`, which renames or hides an action's hint, and `root = false`, which hides it until the prefix key.
A `user/init.lua` written before the hints segment existed adds the `gband.plugin("gband.statusline.hints")` line to get it.
`gband.colorscheme(name)` loads a colorscheme, and `gband.hl.set` styles any part of the line.
[docs/plugins.md](docs/plugins.md) describes the segments, highlight groups, colorschemes and writing your own segment.

### Plugins

gband loads plugins from `$XDG_DATA_HOME/gband/plugins/`, or `~/.local/share/gband/plugins/`.
A plugin has a manifest, `plugin.lua`, and a `client.lua` that each client runs, a `server.lua` that the server runs, or both.
In the client, a plugin can add actions, commands, options, key bindings, event handlers, status line segments, windows, notifications and colorschemes.
In the server, it can watch pane output and input, keep state per pane, emit events to clients, queue them while no client is attached, and answer commands clients call.
[docs/plugins.md](docs/plugins.md) explains how to write one.
[examples/plugins/hello](examples/plugins/hello) is a sample to start from, [examples/plugins/pane](examples/plugins/pane) adds a status line segment and a colorscheme, and [examples/plugins/agent-status](examples/plugins/agent-status) notifies you when a coding agent in a pane waits for an answer.
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
