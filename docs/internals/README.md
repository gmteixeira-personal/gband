# gband's Lua, line by line

Much of gband is written in Lua: highlight groups, colorschemes, bars, plugin windows, the settings window, the key styles, the sidebar and the other bundled plugins.
All of it is built on the same public API your own configuration and plugins use, plus `gband.core`, a small set of primitives the executable provides.
This tutorial reads that Lua, every line of it, and explains why it is built the way it is.

[The scripting tutorial](../tutorial/README.md) teaches the API from the outside, by building a feature of your own; read it first.
This one is for when you want to know how a part of gband works, change it, or replace it.

## The files it reads

Each gband process writes a copy of its bundled Lua under `defaults/` of the configuration directory, `~/.config/gband/defaults/`, as it starts.
The tutorial follows those files as they are, in the order the prelude loads them.
It does not ask you to write replacements, except in the last chapter.

Code from them is shown in blocks whose first line names the file, such as `defaults/lua/gband/bar.lua`, a path relative to the configuration directory.
Open that file to see the lines around a quote.
Every line of every file is quoted exactly once, and gband's test suite checks each quote against the file, so what you read here is what your gband runs.
The colorschemes other than `gruvbox`, and the color tables of the catppuccin themes, hold only colors, and are not quoted.

## Chapters

- [00 · The boundary](00-boundary.md): what the executable provides, `gband.core`, the providers, the prelude, `require` and the copies under `defaults/`
- [01 · A theme](01-themes.md): the `gruvbox` colorscheme
- [02 · The palette and highlight groups](02-highlights.md): `gband.palette`, `gband.hl` and the styles provider
- [03 · Colorschemes](03-colorschemes.md): `gband.colorscheme` and `gband.theme`
- [04 · Bars](04-bars.md): `gband.bar` and the bars provider
- [05 · Plugin windows](05-plugin-windows.md): the state, drawing and API of `gband.win`
- [06 · The windows provider](06-window-provider.md): how `gband.win` receives keys, clicks and sizes
- [07 · Settings](07-settings.md): `gband.settings` and the settings provider
- [08 · Key styles](08-key-styles.md): `gband.keystyle`, the modal, direct and floating presets, and the bundled plugin `gband.desktop`
- [09 · The sidebar and the error list](09-sidebar-and-errors.md): the bundled plugins `gband.sidebar` and `gband.errors`
- [10 · The key list and the prompt](10-keylist-and-prompt.md): `gband.keylist`, `gband.prompt` and `gband.keyform`
- [11 · The default configurations](11-default-configs.md): `defaults/init.lua` and `defaults/server.lua`
- [12 · Your own prelude](12-own-prelude.md): replacing the prelude with your own

## Chapter directories

A chapter with code of your own has a directory under [examples/internals/](../../examples/internals/) holding its files, laid out as the [scripting tutorial's](../tutorial/README.md#the-chapter-directories) are, with tests that `gband test` runs.
Only the last chapter has one.
