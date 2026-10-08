# 00 · Setup

This chapter writes a working `user/init.lua` from an empty file, and shows the three tools you will use in every later chapter: reloading on save, the Lua prompt, and the error list.

The finished files are in [examples/tutorial/00-setup/](../../examples/tutorial/00-setup/).

## The configuration directory

gband keeps your configuration in `~/.config/gband/`, or `$XDG_CONFIG_HOME/gband/` when that is set.
It creates two folders there when it starts:

- `defaults/` holds copies of everything gband ships: the default configuration, the key styles, the bundled modules and the themes. gband rewrites these files as it starts, so edits to them do nothing.
- `user/` holds your configuration. gband creates it empty.

Your Lua can read the directory's path from `gband.config_dir`.
[Where gband looks](../plugins.md#where-gband-looks) describes the directories in full.

## An empty `user/init.lua`

Create `~/.config/gband/user/init.lua` and leave it empty.
While gband runs, saving any `.lua` file under `user/` reloads the configuration, so the change takes effect at once.

A `user/init.lua` replaces the default configuration rather than adding to it.
With an empty file, gband binds no key at all: the prefix key, Ctrl+Space, does nothing, and the sidebar is gone.
That is the point: every line from here on is one you chose.
[Load order](../plugins.md#load-order) lists what runs before and after your file.

Start with the key bindings.
gband bundles three key styles, modal, direct and floating, and `gband.keystyle.use()` makes the bindings of the one saved in the settings window, or of the modal style when none is saved:

```lua
gband.keystyle.use()
```

Save, and Ctrl+Space works again: with the modal style it enters navigation mode, where `h`, `j`, `k` and `l` move focus and `n` opens a window.
Ctrl+Space then `?` lists every key.
[Key styles](../plugins.md#key-styles-gbandkeystyle) describes the three styles, and chapter 01 binds keys of your own.

Next, set up two plugins that gband bundles, the error list and the sidebar:

```lua
gband.plugin("gband.errors")
gband.plugin("gband.sidebar")
```

The sidebar is the column on the left.
Its first row shows the mode, `I` for interactive and `N` for navigation, or `⊞` with the floating style, and the rows below list the bands.
[The sidebar](../plugins.md#the-sidebar-gbandsidebar) describes every row.

The error list has no key yet.
Bind Ctrl+Space then `e` to it; chapter 01 explains `gband.keymap.set`:

```lua
gband.keymap.set("prefix", "e", gband.action["errors.open"], { desc = "list the errors" })
```

On a terminal 40 columns wide and 8 rows high, gband now looks like this:

```screen tests/screenshots/setup_spec/the-configuration-loads.txt
I╭─────────────────╮
 │                 │
1│                 │
2│                 │
 │                 │
 │                 │
 │                 │
 ╰─────────────────╯
```

## The Lua prompt

Ctrl+Space then `:` opens the Lua prompt on the bottom rows.
Type one line of Lua and press Enter to run it:

```lua
print(gband.config_dir)
```

`print` never writes to the terminal, where it would garble the screen.
It writes to the client's log, a file in `~/.local/state/gband/log/`, one per client.
Keep a `tail -f` of the newest one open in a window while you work through the tutorial.
[`print`](../plugins.md#print) and [The Lua prompt](../plugins.md#the-lua-prompt-gbandprompt) have the details.

## Errors

Make a mistake on purpose.
Add this line to the end of `user/init.lua` and save:

```lua
gband.plugn("gband.sidebar")
```

The load fails at that line, and gband keeps the configuration it last loaded, so every key still works.
The sidebar's last row shows a red `!`:

```screen tests/screenshots/setup_spec/a-broken-file-keeps-the-last-configuration--the-error-marker.txt
I╭─────────────────╮
 │                 │
1│                 │
2│                 │
 │                 │
 │                 │
 │                 │
!╰─────────────────╯
```

Ctrl+Space then `e` lists the errors with their files and line numbers: here `user/init.lua:7: attempt to call a nil value (field 'plugn')`.
`q` closes the list, and `c` in it clears the errors.

Your Lua can read the list too: `gband.errors()` returns the messages, oldest first, and `gband.clear_errors()` empties it.
Clear it from the prompt:

```lua
gband.clear_errors()
```

Clearing does not reload anything: fix the line and save to load the file again.
[Errors](../plugins.md#errors) describes what an error disables and for how long.

## The whole file

```lua
gband.keystyle.use()

gband.plugin("gband.errors")
gband.plugin("gband.sidebar")

gband.keymap.set("prefix", "e", gband.action["errors.open"], { desc = "list the errors" })
```

## Another way to start

You can also start from the full default configuration and edit it, as the README's [Configuration](../../README.md#configuration) section suggests:

```sh
cp ~/.config/gband/defaults/init.lua ~/.config/gband/user/init.lua
```

The copy sets every option to its default and reads the settings window's choices.
This tutorial starts from an empty file instead, so that each line is explained where it appears.

## Check your work

Each chapter's directory has tests that start gband with the chapter's files.
Copy your own files over the directory's `user/`, then run:

```sh
cd examples/tutorial/00-setup
gband test
```

Chapter 11 explains how these tests work. Next: [01 · Keys](01-keys.md).
