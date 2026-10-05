# gband

gband is a terminal multiplexer inspired by the [niri](https://github.com/YaLTeR/niri) window manager.
It places panes on an infinitely side-scrolling strip and groups strips into workspaces.
It is written in Rust and scriptable with Lua.

## Status

gband is in early development and is not usable yet.

## How it works

Multiplexers such as tmux and Zellij split a fixed screen area.
Each new pane shrinks the panes already on screen.

gband follows niri's scrollable tiling model instead:

- Panes sit in columns on a strip that extends without limit to the left and right.
- A new pane adds a column to the strip, and existing panes keep their size.
- The view scrolls sideways to follow focus, so the focused pane is always on screen.
- A column can hold several panes stacked vertically.
- Each workspace has its own strip.
  Workspaces stack vertically, and you move between them up and down.

## Scripting

gband embeds a Lua runtime.
Configuration and automation are Lua scripts, so key bindings, layout behavior and custom commands are code you can change.

## Configuration

gband keeps its configuration in `$XDG_CONFIG_HOME/gband/`, or `~/.config/gband/` when `XDG_CONFIG_HOME` is unset.
gband creates the directory when it starts, with two folders in it:

- `defaults/init.lua` holds the full default configuration.
  gband owns this file: it writes the file when it is missing and overwrites it when its content differs from the defaults of the running build.
  Edits to it have no effect.
- `user/` holds your configuration.
  gband creates it empty.

When `user/init.lua` exists, gband runs it instead of the defaults, not on top of them.
Your file starts from the default options and no key bindings, so it must bind every key you want.
To start, copy `defaults/init.lua` to `user/init.lua` and edit the copy:

```sh
cp ~/.config/gband/defaults/init.lua ~/.config/gband/user/init.lua
```

Without `user/init.lua`, the defaults apply.
Saving `user/init.lua` reloads it while gband runs, and deleting it returns to the defaults.
An error in the file shows on the bottom row with its line number, and gband keeps the last configuration that loaded.

```lua
-- Change options. Each call changes only the options it names.
gband.set {
  prefix = "ctrl+b",
  default_column_width = 1/3,
  width_presets = { 1/3, 1/2, 2/3, 1 },
  center_focused_column = "on-overflow",
}

-- A direct binding acts without the prefix key.
gband.bind("alt+h", gband.action.focus_column_left)
gband.bind("alt+l", gband.action.focus_column_right)

-- A prefix binding acts on the key pressed after the prefix key.
gband.bind("prefix x", gband.action.close_pane)

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

The options and their defaults:

| option | value | default |
|---|---|---|
| `prefix` | one key | `"ctrl+a"` |
| `default_column_width` | the width of a new column, as a fraction of the screen | `1/2` |
| `width_presets` | the widths that cycling the column width steps through | `{ 1/3, 1/2, 2/3 }` |
| `center_focused_column` | `"never"`, `"always"` or `"on-overflow"` | `"never"` |

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
| `focus_workspace_down`, `focus_workspace_up` | view the workspace below or above |
| `open_pane` | open a pane running your shell right of the focused column |
| `close_pane` | close the focused pane |
| `consume_or_expel_left`, `consume_or_expel_right` | move the focused pane into or out of the neighbouring column |
| `cycle_column_width` | step the column's width through the presets |
| `toggle_full_width` | toggle full width of the column |
| `grow_column_width`, `shrink_column_width` | widen or narrow the column by a tenth of the screen |
| `grow_pane_height`, `shrink_pane_height`, `reset_pane_height` | change or reset the height of the focused pane |
| `detach` | detach the client |
| `send_prefix` | send the prefix key to the focused pane |

## Building

Install a stable Rust toolchain, version 1.89 or newer, and a C compiler such as `gcc` or `clang`.
The C compiler builds the Lua runtime that gband bundles, so no system Lua is needed.
Then run:

```sh
git clone git@github.com:gmteixeira-personal/gband.git
cd gband
cargo build --release
```

The binary is `target/release/gband`.

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
