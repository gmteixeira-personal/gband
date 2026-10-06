# window

A sample gband status line plugin. It shows the parts of the status line API that most segments use:

- a component that shows the focused window, such as `window 3`, and redraws when the focus changes
- its own highlight group, `WindowSegment`, whose default links to `StatusLineAccent`
- the colorscheme `dusk`, which sets the built-in status line groups and overrides `WindowSegment`

Read [the plugin guide](../../../docs/plugins.md) for the API it uses.

## Install

gband loads plugins from `$XDG_DATA_HOME/gband/plugins/`, or `~/.local/share/gband/plugins/` when `XDG_DATA_HOME` is unset.
Link this directory there:

```sh
mkdir -p ~/.local/share/gband/plugins
ln -s "$PWD/examples/plugins/window" ~/.local/share/gband/plugins/window
```

Then load the colorscheme and set the plugin up in `~/.config/gband/user/init.lua`:

```lua
gband.colorscheme("dusk")
gband.plugin("gband.statusline")
gband.plugin("gband.statusline.band")
gband.plugin("gband.statusline.mode")
gband.plugin("gband.statusline.position")
gband.plugin("window")
```

A `user/init.lua` replaces the default configuration, so start from a copy of `defaults/init.lua` to keep the default bindings and segments.
The bottom of the status line then shows `window 1`, in the colors `dusk` gives `WindowSegment`.

`dusk` uses 24-bit colors.
gband draws them as such when the client's `COLORTERM` is `truecolor` or `24bit`, and as the nearest of the 256 palette colors otherwise.

## Tests

`tests/window_spec.lua` opens and focuses windows and compares the status line with the references in `tests/screenshots/`, with the default colors and with `dusk`.
Run it with `gband test` in this directory; [the testing guide](../../../docs/testing.md) walks through it.
