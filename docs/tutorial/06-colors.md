# 06 · Colors

This chapter gives marks a highlight group, writes a theme of your own that styles it, and reads the terminal palette.

The finished files are in [examples/tutorial/06-colors/](../../examples/tutorial/06-colors/).

## Highlight groups

A highlight group is a named style, such as `SidebarMode` for the sidebar's mode letter.
The plugin window and the bar of the next two chapters draw marks in a group called `Mark`.
Declare it with a default style:

```lua
gband.hl.default("Mark", { fg = "yellow", bold = true })
```

A group holds two settings, a default and an explicit one, and the explicit one wins whenever it exists.
`gband.hl.default` sets the default, so a theme or a later `gband.hl.set` can still change the group without caring what ran first.
Code that offers a group to others, as a plugin does, should always use `gband.hl.default`.

`gband.hl.get("Mark")` returns the group's definition, and `gband.hl.get("Mark", { resolve = true })` the style it draws with, after following links.
[Highlight groups](../plugins.md#highlight-groups-gbandhl) lists the fields of a style and every group gband draws with.

## A theme of your own

A colorscheme is a Lua file in `colors/` that sets groups.
gband bundles several, called themes, and the settings window lists them; a file in `user/colors/` joins the list.
The bundled module `gband.theme` builds a complete theme from a terminal palette and five interface colors.
Write `user/colors/dusk.lua`:

```lua
require("gband.theme").apply({
  palette = {
    fg = "#e0def4",
    bg = "#232136",
    black = "#393552",
    red = "#eb6f92",
    green = "#3e8fb0",
    yellow = "#f6c177",
    blue = "#9ccfd8",
    magenta = "#c4a7e7",
    cyan = "#ea9a97",
    white = "#e0def4",
  },
  ui = {
    surface = "#2a273f",
    selection = "#44415a",
    muted = "#6e6a86",
    accent = "#c4a7e7",
    error = "#eb6f92",
  },
})

gband.hl.set("Mark", { fg = "#f6c177", bold = true })
```

The last line styles `Mark` explicitly, so it wins over the default from `user/init.lua` while this theme is active.
Switching to another colorscheme removes every explicit setting, and `Mark` falls back to its default.

Pick `dusk` in the settings window's theme list, Ctrl+Space then `s`, to make it the theme gband starts with.
To try it once without saving it, run this at the prompt:

```lua
gband.colorscheme("dusk")
```

`gband.colorscheme()` with no argument returns the active colorscheme's name.
A colorscheme that fails to load is reported and changes nothing, so a typo in `dusk.lua` never leaves you with half a theme.
[Colorschemes](../plugins.md#colorschemes-gbandcolorscheme) and [Themes](../plugins.md#themes) have the details.

## The terminal palette

`apply` also calls `gband.palette.set`, which tells gband which colors to draw your terminal's default colors and its 16 ANSI colors with, in every cell: programs, borders, bars and plugin windows.
`gband.palette.get()` returns the palette as it stands:

```lua
print(gband.palette.get().bg)
```

With `dusk` active, the client log shows `#232136`.
[The terminal palette](../plugins.md#the-terminal-palette-gbandpalette) describes how the mapping works.

With `dusk`, a marked window looks the same in text, since a code block cannot show colors:

```screen tests/screenshots/colors_spec/dusk-sets-mark-and-the-palette.txt
I╭mark a───────────╮
 │                 │
1│                 │
2│                 │
 │                 │
 │                 │
 │                 │
 ╰─────────────────╯
```

Its reference file, under the chapter's `tests/screenshots/`, lists the colors of every cell.

## The whole file

The complete files are [examples/tutorial/06-colors/user/init.lua](../../examples/tutorial/06-colors/user/init.lua) and [examples/tutorial/06-colors/user/colors/dusk.lua](../../examples/tutorial/06-colors/user/colors/dusk.lua).

Next: [07 · Plugin windows](07-plugin-windows.md).
