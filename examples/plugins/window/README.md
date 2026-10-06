# window

A sample gband bar plugin. It shows the parts of the bar API that most bars use:

- a right bar 12 columns wide that shows the focused window, such as `window 3`, and redraws with `gband.bar.set_lines` when the focus changes
- its own highlight group, `WindowSegment`, whose default links to `SidebarMode`
- the colorscheme `dusk`, which sets `Bar` and the sidebar groups and overrides `WindowSegment`

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
gband.plugin("gband.errors")
gband.plugin("gband.sidebar")
gband.plugin("window")
```

A `user/init.lua` replaces the default configuration, so start from a copy of `defaults/init.lua` to keep the default bindings and the sidebar.
The top row of the terminal's last 12 columns then shows `window 1`, in the colors `dusk` gives `WindowSegment`.
`gband.plugin("window", { order = 1 })` moves the bar inward past right bars of a lower `order`.

`dusk` uses 24-bit colors.
gband draws them as such when the client's `COLORTERM` is `truecolor` or `24bit`, and as the nearest of the 256 palette colors otherwise.

## Tests

`tests/window_spec.lua` opens and focuses windows and compares the screen, with the sidebar on the left and the bar on the right, with the references in `tests/screenshots/`, with the default colors and with `dusk`.
Run it with `gband test` in this directory; [the testing guide](../../../docs/testing.md) walks through it.
