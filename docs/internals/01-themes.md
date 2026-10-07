# 01 · A theme

The prelude's first modules exist to style what gband draws.
Before reading them, this chapter reads what they serve: a colorscheme file, the bundled `gruvbox` theme, as gband writes it to `defaults/colors/gruvbox.lua`.

## The whole file

```lua defaults/colors/gruvbox.lua
require("gband.theme").apply({
  palette = {
    fg = "#ebdbb2",
    bg = "#282828",
    black = "#282828",
    red = "#cc241d",
    green = "#98971a",
    yellow = "#d79921",
    blue = "#458588",
    magenta = "#b16286",
    cyan = "#689d6a",
    white = "#a89984",
    bright_black = "#928374",
    bright_red = "#fb4934",
    bright_green = "#b8bb26",
    bright_yellow = "#fabd2f",
    bright_blue = "#83a598",
    bright_magenta = "#d3869b",
    bright_cyan = "#8ec07c",
    bright_white = "#ebdbb2",
  },
  ui = {
    surface = "#3c3836",
    selection = "#665c54",
    muted = "#928374",
    accent = "#fabd2f",
    error = "#cc241d",
  },
})
```

A colorscheme is a Lua chunk with no return value.
`gband.colorscheme("gruvbox")` runs it, and whatever the chunk sets while it runs is the theme.
This one makes a single call.

## The palette

`palette` holds the terminal's own colors: the default foreground and background, `fg` and `bg`, and the sixteen ANSI colors from `black` to `bright_white`.
These are the colors the programs in your windows draw with when they ask for "red" or "bright blue", so a theme that sets them recolors your shell and your editor along with gband.
`gband.theme.apply` hands the table to `gband.palette.set`, which [02](02-highlights.md) reads.

## The interface colors

`ui` holds five colors for gband's own interface:

| field | used for |
|---|---|
| `surface` | the background of bars and plugin windows |
| `selection` | the line under the cursor in a plugin window |
| `muted` | secondary text, borders and the unfocused bands of the sidebar |
| `accent` | what should stand out: the mode letter, keys in the key list, titles and the focused border |
| `error` | the error marker and the error banner |

A theme author picks five colors, and `gband.theme` decides which highlight group gets which.
[03](03-colorschemes.md) shows that mapping, and why it lives in one module rather than in every theme.

## The other bundled themes

The tutorial does not walk through the other bundled themes, because they hold nothing `gruvbox` does not show:

- `dracula`, `kanagawa`, `nord`, `one-dark`, `rose-pine`, `solarized`, `tokyo-night` and `vesper` are the same single call with other colors.
- The four `catppuccin-*` flavors, and `catppuccin`, an alias of the mocha flavor, are one line each. They take their palette and interface colors from tables in the module `gband.theme.catppuccin`, so the four flavors share one file.
- `terminal` sets the same groups to named ANSI colors, such as `blue`, and sets no palette, so it follows the terminal's own colors.
- `default` is an empty file. Loading any colorscheme first clears every explicit style and the palette, so after this one every group draws with the default its module declares, and the terminal keeps its own colors.

Their copies are under `defaults/colors/` and `defaults/lua/gband/theme/` for you to read.
