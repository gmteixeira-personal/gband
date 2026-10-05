# pane

A sample gband status line plugin. It shows the parts of the status line API that most segments use:

- a component that shows the focused pane, such as `pane 3`, and redraws when the focus changes
- its own highlight group, `PaneSegment`, whose default links to `StatusLineAccent`
- the colorscheme `dusk`, which sets the built-in status line groups and overrides `PaneSegment`

Read [the plugin guide](../../../docs/plugins.md) for the API it uses.

## Install

gband loads plugins from `$XDG_DATA_HOME/gband/plugins/`, or `~/.local/share/gband/plugins/` when `XDG_DATA_HOME` is unset.
Link this directory there:

```sh
mkdir -p ~/.local/share/gband/plugins
ln -s "$PWD/examples/plugins/pane" ~/.local/share/gband/plugins/pane
```

Then load the colorscheme and set the plugin up in `~/.config/gband/user/init.lua`:

```lua
gband.colorscheme("dusk")
gband.plugin("gband.statusline.band")
gband.plugin("gband.statusline.mode")
gband.plugin("gband.statusline.position")
gband.plugin("pane")
```

A `user/init.lua` replaces the default configuration, so start from a copy of `defaults/init.lua` to keep the default bindings and segments.
The right of the status line then shows `pane 1`, in the colors `dusk` gives `PaneSegment`.

`dusk` uses 24-bit colors.
gband draws them as such when the client's `COLORTERM` is `truecolor` or `24bit`, and as the nearest of the 256 palette colors otherwise.
