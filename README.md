# gband

gband is a terminal multiplexer inspired by the [niri](https://github.com/YaLTeR/niri) window manager.
It arranges your windows in bands.
A band is a row of windows that scrolls sideways, and it is circular: past its last window, it goes on with its first.
gband keeps as many bands as you need, stacked vertically.

gband is animated, includes themes and is fully scriptable in Lua.
Much of gband itself is written in Lua, and every part of it starts with sensible defaults, so it works without any configuration.
gband is written in Rust.

## Features

- **Multiple bands.** Each band holds its own row of windows, and the view slides up and down between bands.
- **Circular bands.** Focus goes round from a band's last column to its first, and a long enough band is drawn as a loop.
- **Animated.** Scrolling, band switches, moving windows and resizes animate.
- **Fully scriptable.** The configuration, key bindings and plugins are Lua, in the client and in the server; see [Scripting](#scripting).
- **Written in Lua, as examples.** The key styles, the desktop, the key list, the sidebar, the settings window and the themes are Lua code, written that way so you can read how they use the API.
- **Themes.** gband bundles 15 themes, previews each one as you move through the theme list, and loads colorschemes of your own; see [Settings](#settings).
- **User friendly.** The settings window opens on the first start to pick a theme, the sidebar and a key style.
  Ctrl+Space then `?` lists every key and runs the one you choose.
  The mouse focuses, moves and resizes windows, and selects text.
  The floating key style makes gband a desktop like MS Windows: every window floats, with minimize, maximize and close buttons on its title bar, and one list holds every window.
  Saving the configuration reloads it at once, and an error names its file and line while the last working configuration stays loaded.
- **Sensible defaults.** gband needs no configuration file: the defaults bind the keys, set up the sidebar and apply the `default` theme.

## Status

gband is usable and stable, but in early development.
Configuration, the Lua API and the plugin API can still change between releases.

## How it works

Multiplexers such as tmux and Zellij split a fixed screen area into panes.
Each new pane shrinks the panes already on screen.

gband follows niri's scrollable tiling model instead, with bands in place of niri's workspaces:

- A band is a row of columns, and each column holds one or more windows stacked vertically.
- A new window adds a column to the band, and existing windows keep their size.
- The view scrolls sideways to follow focus, so the focused window is always on screen.
- Bands are circular.
  Focus goes round from a band's last column to its first, and from its first to its last.
  A band long enough to wrap around the screen is drawn as a loop, its first column following its last.
  `gband.opt.loop_bands = false` gives every band two ends instead.
- Bands stack vertically, and the view slides up and down between them.
- An empty band always waits below the last one.
  Opening a window in it adds a new empty band below, and a band other than the last is removed when its last window closes.

Scrolling, band switches, moving windows and resizes animate.
`gband.opt.animations = false` turns the animations off, and `gband.opt.animation_speed` makes them faster or slower; see [Options](docs/plugins.md#options-gbandopt).
Set `GBAND_ANIMATIONS=off` before you attach to turn the animations off, whatever the `animations` option holds.

Each window shows its name on its top border; see [Window names](#window-names).
Set `GBAND_WINDOW_TITLES=off` before you attach to draw no names, whatever the `window_titles` option holds.

## Vocabulary

gband keeps niri's layout model but calls niri's workspace a band.
The documentation uses these names:

| name | meaning |
|---|---|
| window | a terminal running a program; tmux and Zellij call it a pane |
| column | one or more windows stacked vertically, side by side with the other columns of its band |
| band | a circular row of columns that scrolls sideways; niri calls it a workspace |
| tiled window | a window in a column of its band |
| floating window | a window in a box over its band, which stays where it is while the band scrolls |
| plugin window | text that a plugin draws, either floating over the windows or tiled in a column like a window |
| bar | columns that Lua code reserves at the left or right edge of the terminal, such as the sidebar |
| ribbon | the part of the terminal that the bars leave, where the viewed band is drawn |

[docs/plugins.md](docs/plugins.md#plugin-windows-gbandwin) describes plugin windows, and [docs/plugins.md](docs/plugins.md#side-bars-gbandbar) describes bars.

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
The prefix key, Ctrl+Space, gives the keys below their gband meaning, in one of three key styles:

- **modal**: Ctrl+Space enters navigation mode, and the sidebar shows `N`.
  Each key below acts and navigation mode stays active, so `l` `l` `l` moves three columns and `=` `=` widens the column twice.
  A key with no binding does nothing.
  Escape or Enter returns to interactive mode, and so do `?`, `:`, `N`, `s` and Ctrl+Space once they have acted.
  `n` returns to interactive mode only when the settings window's `I on new` is `on`, which it is not until you save it on.
- **direct**: Ctrl+Space then one key acts once, and the keys that follow reach the window again, as in tmux.
  Ctrl+Space `l` `l` moves one column and types `l`.
  Escape, Enter and any other key with no binding after Ctrl+Space are discarded.
- **floating**: every window floats, as on a desktop like MS Windows, and Ctrl+Space opens the window list.
  Each floating window has `[_][□][X]` on its title bar to minimize, maximize and close it, and its border moves and resizes it with the left button; see [Mouse](#mouse).
  The window list holds `New window`, `Settings` and every window, the most recent first, each with a shortcut: `n`, `s`, then `1` to `9` and `0`.
  `j`, `k`, the arrows and the pointer move through it, and moving onto a window shows it, from another band or minimized, without changing anything yet.
  Enter, a click or a shortcut keeps the window shown, and Escape or `q` returns to how things were when the list opened.
  Its other keys from the table below, such as `D`, `:` and `?`, keep the window shown and run from the list, and Ctrl+Space again sends Ctrl+Space to it.

The key style is chosen in the settings window, which Ctrl+Space then `s` opens; see [Settings](#settings).

The modal and direct styles bind the same keys after Ctrl+Space; the floating style binds only `n`, which opens a floating window, `?`, `:`, `N`, `s`, `D` and Ctrl+Space:

| key | action |
|---|---|
| `h`, `l`, or the left or right arrow | focus the column to the left or right |
| `j`, `k`, or the down or up arrow | focus the window below or above |
| `u`, `i` | view the band below or above |
| `c` | center the focused column in the view, or the focused floating window on the screen |
| `n` | open a window running your shell; modal returns to interactive mode only when `I on new` is `on` |
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
| `:` | open the Lua prompt, which runs one line of Lua |
| `N` | rename the focused window; see [Window names](#window-names) |
| `s` | open the settings window |
| `D` | detach |
| Escape, Enter | modal only: return to interactive mode |
| Ctrl+Space | send Ctrl+Space to the window; modal returns to interactive mode |

Navigation mode changed two habits.
Enter no longer opens a window: `n` does.
After Ctrl+Space and a layout key, what you type no longer reaches the window until Escape or Enter returns to interactive mode.

### Settings

Ctrl+Space then `s` opens the settings window, a box titled `settings` over the windows, and returns to interactive mode so the box takes the keys that follow.
It has these lines:

| line | shows |
|---|---|
| `theme` | the active theme |
| `sidebar` | `on` or `off` |
| `keys` | the key style, `modal`, `direct` or `floating` |
| `I on new` | `on` or `off`: whether `n` in navigation mode returns to interactive mode; `off` until saved `on`; shown with the modal key style only |

`j`, `k` and the up and down arrows move between the lines.
On `theme`, `l` or Right loads the next theme and `h` or Left the one before, and Enter opens the theme list.
On `sidebar` and `I on new`, Enter, `h`, `l`, Left and Right switch to the other value.
On `keys`, Enter, `l` and Right pick the next style of `modal`, `direct` and `floating`, and `h` and Left the one before.
Escape or `q` closes the box and saves nothing.

Each change is saved at once.
The configuration then reloads, and the box opens again on the same line.

The theme list, titled `theme`, has one line per theme: the bundled themes, then every colorscheme of your own, such as `user/colors/dusk.lua`, or of a plugin.
Moving through it previews the theme under the cursor line.
Enter saves that theme, and Escape or `q` restores the theme that was active when the list opened.

gband bundles these themes:

| theme | look |
|---|---|
| `default` | sets no color: bars on your terminal's background, gband's own parts in their built-in styles, and programs in your terminal's colors |
| `terminal` | colors gband's parts with the 16 colors of your terminal's palette only, so your terminal decides every color |
| `catppuccin-latte`, `catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`, `tokyo-night`, `dracula`, `nord`, `gruvbox`, `one-dark`, `solarized`, `kanagawa`, `rose-pine`, `vesper` | colors gband's parts, and replaces the default colors and the 16 ANSI colors that programs in windows draw with, using the theme's published terminal colors |

`catppuccin` loads `catppuccin-mocha`.
A program's 256-color and 24-bit output keeps its own colors under every theme.

With the default configuration, gband opens the settings window when it starts while no theme, sidebar or key style is saved.
Escape closes it and saves nothing: the `default` theme, the sidebar and the modal key style apply, and the next start offers it again.

### Mouse

gband takes the mouse while it is attached.
In interactive mode:

- A click focuses the window under the pointer, and raises a floating window.
- When the window's program asked for the mouse, such as `htop` or `less --mouse`, the click, its drags and its release go to the program. Hold Ctrl and Alt to select text instead.
- Otherwise a left drag selects text, shown reversed. Releasing copies it to gband's copy buffer and to your terminal's clipboard with OSC 52.
- A right click pastes the copy buffer into the window under the pointer.
  With the floating key style, a right click on a floating window's border opens its window menu, `Close`, `Maximize`, `Minimize`, `Tile left` and `Tile right`, and one on empty ribbon opens the window list.
- In a plugin window, such as the key list, a click moves the cursor line.

With Alt held, the wheel views the band below or above, one band per flick.
Every other turn of the wheel reaches the window under the pointer, in every mode, and the program receives it when it asked for the mouse.
gband keeps no scrollback, so the wheel scrolls nothing of its own; over a plugin window it scrolls the lines.

Every key style drags windows the way niri does with its modifier held, with Alt held; the modal and direct styles also drag with no modifier in navigation mode or after Ctrl+Space.

| button | drag |
|---|---|
| left | move a floating window, or lift a tiled window and drop it beside or into another column; on empty ribbon, slide the band or switch bands as the middle button does |
| right | resize the window from the edges nearest the press |
| middle | slide the band sideways or switch bands vertically, whichever axis the drag starts on; a sideways release focuses the column at the middle of the view, and a vertical release views the band holding the middle row |

| mouse | modal | direct |
|---|---|---|
| Alt with a drag or the wheel | acts in interactive and navigation mode | acts at any time |
| Ctrl+Space then a drag | acts, and navigation mode stays active | acts once, and the keys that follow reach the window |
| Ctrl+Space then Alt with a drag or the wheel | acts, and navigation mode stays active | acts once |

The floating key style drags a floating window by its border with the left button and no modifier: the top border moves it, a side or the bottom resizes that edge, and a corner, with the cell beside it, resizes both of its edges.
A press on a title bar button acts when the button is released on it.
Every other plain click keeps its interactive-mode meaning.

Some Linux desktops, such as Xfce, take Alt with a drag to move their own windows, so the terminal never sees it.
Pick another modifier for every one of these bindings in `user/init.lua`:

```lua
gband.opt.mouse_mod = "ctrl+alt"
```

While gband has the mouse, a plain drag no longer selects text in your terminal.
Most terminals, such as kitty, Alacritty, WezTerm, foot and those built on VTE, still select natively with Shift held.
Copying to the clipboard needs OSC 52 allowed in the terminal: kitty's `clipboard_control` must include `write-clipboard`, and tmux needs `set-clipboard on`.
Without it, the copy buffer still pastes inside gband.

## Scripting

gband embeds a Lua runtime in the client and in the server.
Configuration and automation are Lua scripts, so key bindings, layout behavior and custom commands are code you can change.
Plugins add actions, commands, key bindings, bars, plugin windows and colorschemes; see [Plugins](#plugins).
The Lua prompt, Ctrl+Space then `:`, runs one line of Lua while gband runs.
[The scripting tutorial](docs/tutorial/README.md) builds a plugin step by step from an empty `user/init.lua`, with runnable files for every chapter.

Much of gband is itself written in Lua:

- the modal, direct and floating key styles, and the desktop the floating style sets up
- the key list, the Lua prompt and the rename box
- the sidebar and the error list
- the settings window and the theme list
- every bundled theme
- the APIs for bars, plugin windows, highlight groups and colorschemes

These parts are written in Lua on purpose, to serve as examples of the API.
Their source is in [crates/lua/src/runtime/gband/](crates/lua/src/runtime/gband/), and gband writes a copy of every file to `defaults/lua/gband/` and `defaults/colors/` in the configuration directory; see [Configuration](#configuration).
Every bundled file, the APIs included, uses only the documented API that your configuration and plugins have.
To change or replace one, copy it from `defaults/lua/gband/` to `user/lua/gband/`, or from `defaults/colors/` to `user/colors/`, and edit the copy; it takes the bundled file's place at the next reload.
[The internals tutorial](docs/internals/README.md) reads every line of these files and explains how they are built on `gband.core`.
The [plugin guide](docs/plugins.md) documents the whole API, including the primitives in `gband.core` that the bundled Lua is built on, and how `gband.api_version` changes between releases.

Every part comes with sensible defaults, so gband needs no configuration until you want to change something.

## Configuration

gband keeps its configuration in `$XDG_CONFIG_HOME/gband/`, or `~/.config/gband/` when `XDG_CONFIG_HOME` is unset.
gband creates the directory when it starts, with two folders in it:

- `defaults/init.lua` holds the full default client configuration, and `defaults/server.lua` the default server configuration.
  `defaults/keystyle/modal.lua`, `defaults/keystyle/direct.lua` and `defaults/keystyle/floating.lua` hold the bindings of the three key styles.
  `defaults/lua/gband/` holds every Lua module bundled with gband, such as `defaults/lua/gband/sidebar.lua`, and `defaults/colors/` every bundled theme, such as `defaults/colors/nord.lua`; copy one into `user/` to change it, as [Scripting](#scripting) describes.
  gband owns these files: it writes each when it is missing and overwrites it when its content differs from the defaults of the running build.
  Edits to them have no effect.
- `user/` holds your configuration.
  gband creates it empty.
  The settings window saves each setting in a file of its own: `user/theme.lua` holds `return "<theme>"`, `user/sidebar.lua` and `user/interactive_on_new.lua` hold `return true` or `return false`, and `user/keystyle.lua` holds `return "modal"`, `return "direct"` or `return "floating"`.
  gband only reads a value from these files and never runs them as configuration, and nothing writes `user/init.lua`.
  Delete `user/theme.lua`, `user/sidebar.lua` and `user/keystyle.lua` to be offered the settings window again.

The configuration has two files, one per process:

- `user/init.lua` configures each client: the prefix key, key bindings, the sidebar, colors and notifications.
- `user/server.lua` configures the server: the column widths, and the server halves of plugins.

The server reads its file from the machine it runs on, and each client reads its own.
Neither process runs the other's file.

When `user/init.lua` exists, gband runs it instead of the default client configuration, not on top of it.
Your file starts from the default options and no key bindings, so it must bind every key you want.
To start, copy `defaults/init.lua` to `user/init.lua` and edit the copy:

```sh
cp ~/.config/gband/defaults/init.lua ~/.config/gband/user/init.lua
```

The copy binds its keys with `gband.keystyle.use()`, which makes the bindings of the saved key style, or of modal when none is saved.
`gband.keystyle.use("direct")` picks a style whatever is saved, and a binding made after the call replaces the style's binding for that key.
To edit the bindings themselves, copy a style's bindings from `defaults/keystyle/` into `user/init.lua` in place of the `gband.keystyle.use()` call.

The saved theme loads before `user/init.lua` runs, so it applies to your file too, unless your file calls `gband.colorscheme` itself.
The settings window's `sidebar`, `keys` and `I on new` lines apply only where your file asks for them, as the copy does: it sets up the sidebar unless `gband.settings.sidebar()` returns `false`, and calls `gband.keystyle.use()` with no argument, whose modal `n` returns to interactive mode only when `gband.settings.interactive_on_new()` returns `true`.
A file that sets up the sidebar or binds its own keys keeps its choice, and the settings window then only saves the setting.
An `n` binding of your own can compare `gband.settings.interactive_on_new()` with `true` to follow the `I on new` line.

Without `user/init.lua`, the defaults apply.
Saving `user/init.lua`, or any other `.lua` file under `user/`, reloads the configuration while gband runs, and deleting `user/init.lua` returns to the defaults.
An error in the file shows a red `!` at the bottom of the sidebar, and gband keeps the last configuration that loaded.
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
| `window_titles` | `true` or `false`: whether windows show their names on their top borders | `true` |
| `animations` | `true` or `false`: whether the client animates changes of the layout and the view | `true` |
| `animation_speed` | how fast animations run, from `0.1` to `10`, `2` being twice as fast as `1` | `1` |
| `tile_border_sides` | the sides of a tiled window's border that are drawn: a list of `"top"`, `"right"`, `"bottom"` and `"left"` | `{ "top", "right", "bottom", "left" }` |
| `tile_border_chars` | the border's characters: `"plain"`, `"rounded"`, `"double"`, `"thick"`, or a list of eight one-cell strings | `"rounded"` |
| `focused_tile_border_chars` | the focused tiled window border's characters, with the same sides | `"rounded"` |
| `floating_border_sides` | the drawn sides of a floating window's border | `{ "top", "right", "bottom", "left" }` |
| `floating_border_chars` | a floating window border's characters | `"rounded"` |
| `focused_floating_border_chars` | the focused floating window border's characters, with the same sides | `"rounded"` |
| `width_step` | how much growing or shrinking changes a column's width, as a fraction of the screen | `1/10` |
| `height_step` | how much growing or shrinking changes a window's height, as a fraction of the screen, at most 1 | `1/10` |
| `mouse_mod` | the modifiers that `mod` stands for in a mouse name: `ctrl`, `alt` and `shift` joined by `+` | `"alt"` |
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
The modal key style makes `prefix` a mode labelled `navigation`.
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

The direct key style, `gband.keystyle.use("direct")`, is the one-key prefix bindings ready made.

A key is a key name with optional `ctrl`, `alt` and `shift` modifiers joined by `+`, such as `alt+h`, `ctrl+PageUp` or `alt++`.
A key name is one character, or `enter`, `tab`, `backtab`, `backspace`, `escape`, `space`, the arrow keys `up`, `down`, `left` and `right`, `home`, `end`, `insert`, `delete`, `pageup`, `pagedown`, or `f1` to `f12`.
`leftmouse`, `middlemouse` and `rightmouse` name a press of a mouse button, and `wheelup`, `wheeldown`, `wheelleft` and `wheelright` one step of the wheel, with the same modifiers, such as `alt+rightmouse`. They bind in key tables only, not as the prefix or in a plugin window's `keys`.
Before a mouse name, `mod` stands for the modifiers of the `mouse_mod` option, so `mod+leftmouse` is Alt with the left button until you change it.
A bound wheel step runs its binding at most once every 150 ms, and an unbound one still reaches the window under the pointer.
A function bound to a mouse name can return `false` to pass the press or wheel step on, as if the name were unbound.
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
| `minimize_window` | hide the focused floating window from this client until it is focused again |
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
| `drag_window` | when bound to a mouse button, move the window with the mouse |
| `drag_resize_window` | when bound to a mouse button, resize the window with the mouse; a target's `edges` names the edges it moves |
| `drag_band` | when bound to a mouse button, slide the band or switch bands with the mouse |

`gband.action.list()` lists every action with its description.
A target table names the window an action acts on, the target of `drag_resize_window` names in `edges` the edges a drag moves, and the target of `toggle_window_floating` can name in `floating` the layer to put the window in, floating or tiling it without toggling.
The four grow and shrink actions also take a `step`, so one binding can resize by another amount than the options give:

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
gband.opt.focused_tile_border_chars = "double"
gband.opt.floating_border_chars = "thick"
```

Every border is rounded by default.
The focused tiled window and the focused floating window take the characters of `focused_tile_border_chars` and `focused_floating_border_chars`, with the sides of the other windows, and a tile lifted with the mouse marks its drop place with the focused tile's characters.
Rounded has no heavy form: a `"thick"` or `"double"` focused border has square corners beside the rounded ones.
For the square borders gband drew before, set all four character options to `"plain"`:

```lua
gband.set {
  tile_border_chars = "plain",
  focused_tile_border_chars = "plain",
  floating_border_chars = "plain",
  focused_floating_border_chars = "plain",
}
```

The groups `WindowBorder` and `WindowBorderFocused` color the borders, and each theme other than `default` sets both; [docs/plugins.md](docs/plugins.md#highlight-groups-gbandhl) lists them.

Lua can draw decorations, such as buttons, at the right end of a window's top border, as [docs/plugins.md](docs/plugins.md#window-decorations) describes.

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

`minimize_window` hides the focused floating window, and `gband.window.minimize` hides a floating window by number.
A minimized window is hidden only on the client that minimized it: other clients still draw it, and its program keeps its size.
The modal and direct key styles bind no key to it, since they have no window list to bring it back; the floating key style minimizes from a window's `[_]` button and its window list brings it back.
`gband.window.focus` brings it back, on top of the other floating windows.

`gband.window.focus(window, { peek = true })` shows a window on top, minimized or not, without changing the stacking order or restoring it, until focus moves on.
`gband.layout()` gives each window `last_focus`, which rises each time this client focuses the window, so a script can list windows by recent use.

### Key list

Ctrl+Space then `?` opens a list of the navigation keys in a box titled `navigation keys`, or `prefix keys` with the direct and floating key styles, over the windows, each with its description, and returns to interactive mode so the list takes the keys that follow.
Pressing a line's key selects that line and runs its binding, an action or a function, on the window behind the list, which stays open for the next choice: `l` focuses the column to the right, and `j` and `k` focus the window below and above.
A binding that opens another box, such as `s` or `:`, closes the list, and that box takes the keys.
Up and Down move through the list, PageUp, PageDown, Home and End jump through it, and Enter runs the selected line.
Lines whose key is one of these, and the line of the prefix key, run only through Enter.
The list's own `?` line shows dimmed, and pressing `?` or Enter on it only selects it.
`q` runs its line, close the window, which closes the list; Escape, or Ctrl+Space then `q`, close it too.

The list is a plugin bundled with gband, set up by every key style.
A `user/init.lua` that calls no `gband.keystyle.use()` sets it up and binds it itself:

```lua
gband.plugin("gband.keylist")
gband.keymap.set("prefix", "?", gband.action["keylist.open"], { desc = "list the keys" })
```

### Lua prompt

Ctrl+Space then `:` opens a one-line box titled `lua` on the bottom rows of the ribbon, in the manner of Neovim's command line, and returns to interactive mode so the box takes what you type.
It shows `:`, the text typed so far and a reversed cell as the cursor; a line too long for the box shows its end.
Every character goes into the line, `j`, `k` and `q` included, and a paste does too, with each line break turned into a space.
Backspace deletes the last character, and on an empty line closes the prompt; Ctrl+U clears the line.
Enter closes the prompt and runs the line, and Escape closes it and runs nothing.

The line runs as a binding function of `user/init.lua` would, so it can call `gband.action.*`, `gband.win.*`, `gband.keymap.enter` and the rest of the API a binding function can, such as `gband.action.focus_column_left()`.
It runs with an instruction budget of its own: an endless loop stops only the line, and the prompt keeps working.
A syntax error, a runtime error or a stop by the instruction limit is reported like any configuration error, as `prompt:1: <message>`, so the sidebar shows `!`.
Return values are dropped; `gband.notify(tostring(value))` shows one.

The prompt is a plugin bundled with gband, set up by every key style.
A `user/init.lua` that calls no `gband.keystyle.use()` sets it up and binds it itself:

```lua
gband.plugin("gband.prompt")
gband.keymap.set("prefix", ":", gband.action["prompt.open"], { desc = "run Lua" })
gband.keymap.set("prefix", "N", gband.action["prompt.rename"], { desc = "rename the window" })
```

`gband.hl.set("PromptCursor", { ... })` styles the cursor, which is reversed by default.

### Window names

Every window that runs a program has a name, shown on its top border from the border's second column, in the border's colors.
A name too long for the border is cut to the border's width less 2 cells, and a window whose border draws no top side shows none.
On a border that shows decorations, the name is cut shorter, so that at least one border cell is left before them.
The server keeps the names, so every client attached to a session shows the same ones.

A window's name is, in this order:

1. Its manual name, set with Ctrl+Space then `N`.
2. Its title, as the program in it last set it with OSC 0 or OSC 2.
   A program that saves the title and restores it on exit, as vim does, gives the earlier title back.
3. The command in its foreground, such as `sleep` or `vim`, followed within a second.
   A login shell's leading `-` is dropped, so `-zsh` shows `zsh`.

When two or more windows of a band share a name, each shows ` #` and a number after it, counted in layout order: the columns left to right, each column's windows top to bottom, then the floating windows.
So two shells show `bash #1` and `bash #2`, and `vim` alone shows `vim`.
The numbers follow the layout as windows move.

Ctrl+Space then `N` opens a one-line box titled `rename` on the bottom rows, holding the focused window's manual name, and returns to interactive mode so the box takes what you type.
It edits as the Lua prompt does.
Enter saves the line as the window's manual name, and Enter on an empty line clears it, so the window takes its automatic name again.
Escape closes the box and changes nothing.
Opening the Lua prompt closes the rename box, and the other way round.

As in Zellij, a title wins over the command.
A shell that sets a title such as `user@host:~` at each prompt keeps showing it while a command runs, unless the shell also sets the title to the command.
fish does that by default.
zsh and bash need a hook that does it before each command:

```zsh
# ~/.zshrc
preexec() { printf '\e]2;%s\a' "${1%% *}" }
precmd() { printf '\e]2;%s\a' "${PWD/#$HOME/~}" }
```

```bash
# ~/.bashrc
trap 'printf "\e]2;%s\a" "${BASH_COMMAND%% *}"' DEBUG
PROMPT_COMMAND='printf "\e]2;%s\a" "${PWD/#$HOME/~}"'
```

A shell that sets no title at all shows its foreground command, such as `bash` at the prompt and `make` while it builds.
A manual name covers the rest.

`gband.opt.window_titles = false` draws no names, and `GBAND_WINDOW_TITLES=off` does the same whatever the option holds.
The names are still kept, and `gband.layout()` and the rename box still use them.
`gband.window.rename(window, name)` renames a window from Lua; [docs/plugins.md](docs/plugins.md) describes it.

### Sidebar

The sidebar is a bar one column wide at the left of the terminal, as tall as the terminal.
The windows keep the size they would have without it: they are drawn in the columns it leaves, and the view scrolls inside them, so a 1/2 column of an 80-column terminal is still 40 columns wide beside the bar.

| row | shows |
|---|---|
| first | the mode: `I` in interactive mode, `N` in navigation mode, or the first letter of another mode's label, uppercased |
| second | nothing |
| from the third | one label per band, from the top band down, the empty last band included: `1` to `9`, then `a` to `z` |
| last | a red `!` while a configuration or plugin error is reported |

A new session shows `I`, then `1` and `2`.
The viewed band's label is bold, and the others are dim, or in the theme's muted color.
A label is the band's position, so the labels stay `1`, `2`, `3` after a band is removed.
Bands that do not fit above the last row, and bands past the 35th, are not shown.
With the direct key style, the sidebar shows `P` after the prefix key until the key sequence ends.
With the floating key style, the first row shows `∷` instead of the mode, and clicking it opens the window list.
Clicking a band's label with the left button views that band, in any mode.
Turning the wheel over the sidebar views the band below or above the viewed band, one band per step, in any mode.

The sidebar is a plugin bundled with gband, `gband.sidebar`, which the default configuration sets up after `gband.errors` unless the settings window turned it off.
A `user/init.lua` replaces the defaults, so it sets it up itself with the same call; a copy of `defaults/init.lua` already holds it:

```lua
gband.plugin("gband.sidebar")
```

Leaving it out removes the sidebar, and the ribbon takes the whole terminal.
Its options place it:

```lua
gband.plugin("gband.sidebar", {
  side = "right",   -- "left" by default
  order = 0,        -- the order among bars on the same side
})
```

It is always one column wide.
`gband.colorscheme(name)` loads a colorscheme, and `gband.hl.set` styles the sidebar's groups, `SidebarMode`, `SidebarBand`, `SidebarBandActive` and `SidebarError`.
[docs/plugins.md](docs/plugins.md) describes the sidebar, highlight groups, colorschemes and adding bars of your own with `gband.bar`.

### Errors

The client keeps every configuration and plugin error since the last load without one, oldest first, and `gband.errors()` returns them.
`gband.clear_errors()`, called from a binding function or another callback, empties the list and dismisses the sidebar's `!` or the banner until the next error.
It does not reload the configuration, so a callback or a plugin that an error disabled stays disabled until the next load.
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
Without a sidebar, the latest error shows on the bottom row of the ribbon instead.

### Plugins

gband loads plugins from `$XDG_DATA_HOME/gband/plugins/`, or `~/.local/share/gband/plugins/`.
A plugin has a manifest, `plugin.lua`, and a `client.lua` that each client runs, a `server.lua` that the server runs, or both.
In the client, a plugin can add actions, commands, options, key bindings, event handlers, side bars, plugin windows, notifications and colorschemes.
In the server, it can watch window output and input, keep state per window, emit events to clients, queue them while no client is attached, and answer commands clients call.
[docs/plugins.md](docs/plugins.md) explains how to write one.
[examples/plugins/hello](examples/plugins/hello) is a sample to start from, [examples/plugins/window](examples/plugins/window) adds a side bar and a colorscheme, and [examples/plugins/agent-status](examples/plugins/agent-status) notifies you when a coding agent in a window waits for an answer.
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

## License

gband is licensed under the [GNU General Public License, version 3](LICENSE) or any later version.
A [plugin exception](LICENSE-EXCEPTION) lets plugins, init files and colorschemes carry any license, proprietary included, as long as they use gband only through its Lua API.
The documentation under `docs/`, the examples and the default configuration are licensed under the [MIT License](LICENSE-MIT), so you can copy them into your own configuration freely.
