# Scripting gband: a tutorial

This tutorial teaches gband's Lua API by building one feature from an empty `user/init.lua`.
The feature marks windows with a letter: you mark a window, jump back to it from anywhere, list the marks in a plugin window, and see the focused window's mark in a bar.
Along the way it becomes a plugin with a client half, a server half that keeps the marks for every attached client, and tests.

Each chapter teaches by example and links to the reference, [the plugin guide](../plugins.md) and [the testing guide](../testing.md), for the full story.
It assumes you have used gband and know a little Lua.

## Chapters

- [00 · Setup](00-setup.md): the configuration directory, an empty `user/init.lua`, reloading, the Lua prompt and the error list
- [01 · Keys](01-keys.md): key bindings, key tables, modes, key styles and mouse names
- [02 · Actions and commands](02-actions.md): actions, commands, action targets, starting programs, notifications and the clipboard
- [03 · Options](03-options.md): gband's options, an option of your own, and the settings window
- [04 · Events](04-events.md): handlers, groups and events of your own
- [05 · Layout](05-layout.md): reading the layout and the view, and acting on windows and bands
- [06 · Colors](06-colors.md): highlight groups, colorschemes, themes and the terminal palette
- [07 · Plugin windows](07-plugin-windows.md): a list of the marks
- [08 · Bars](08-bars.md): a bar beside the sidebar
- [09 · A plugin](09-plugin.md): the manifest, `setup`, options of a plugin, ownership and the side guard
- [10 · The server half](10-server.md): window state, server commands, server events and `gband.rpc`
- [11 · Testing](11-testing.md): the plugin's tests and screenshots

## The chapter directories

Every chapter has a directory under [examples/tutorial/](../../examples/tutorial/) holding the files you have written by the end of it, laid out as you install them:

| in the chapter directory | install at |
|---|---|
| `user/` | `~/.config/gband/user/` |
| `plugins/<name>/` | `~/.local/share/gband/plugins/<name>/` |

Start at any chapter by copying its directory's files into place.
Each directory also holds `tests/`, which start gband with the chapter's files and check what the chapter says they do.
To check your own copy, put your files in a copy of the directory and run:

```sh
cd examples/tutorial/05-layout
gband test
```

gband's own test suite runs every chapter's tests, and checks that every Lua block in a chapter is in the chapter's files, so the code here works with the gband it ships with.

## Next

[The internals tutorial](../internals/README.md) reads gband's own Lua, the API modules and the bundled plugins this tutorial uses, line by line.
