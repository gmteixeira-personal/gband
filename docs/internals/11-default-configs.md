# 11 · The default configurations

After the prelude, the client runs its init file: `user/init.lua` when it exists, and otherwise the default client configuration, `defaults/init.lua`.
The server does the same with `user/server.lua` and `defaults/server.lua`.
Both defaults are ordinary configuration, using the modules of the earlier chapters as your own `user/init.lua` would, and this chapter reads them in full.

## The client

```lua defaults/init.lua
gband.opt.prefix = "ctrl+space"
gband.opt.center_focused_column = "never"
gband.opt.loop_bands = true
gband.opt.window_titles = true
```

```lua defaults/init.lua
gband.opt.animations = true
gband.opt.animation_speed = 1
```

These two set the animation options to their declared defaults: animations on, at the speed where every motion settles within 400 ms.
`GBAND_ANIMATIONS=off` still turns them off for one client, whatever these lines say.

```lua defaults/init.lua
gband.opt.notify_style = "osc9"
gband.opt.tile_border_sides = { "top", "right", "bottom", "left" }
gband.opt.tile_border_chars = "rounded"
gband.opt.focused_tile_border_chars = "rounded"
gband.opt.floating_border_sides = { "top", "right", "bottom", "left" }
gband.opt.floating_border_chars = "rounded"
gband.opt.focused_floating_border_chars = "rounded"
gband.opt.width_step = 1/10
gband.opt.height_step = 1/10
gband.opt.mouse_mod = "alt"
```

Every option is set to the value it already has; the [README](../../README.md) lists each option and its default.
The lines are there for you: copy `defaults/init.lua` to `user/init.lua`, and every client option is in front of you to change.

```lua defaults/init.lua
gband.keystyle.use()
```

The key bindings come from the saved key style, or the modal one, through `gband.keystyle.use` of [08](08-key-styles.md).
Called with no argument, it follows the settings window's `keys` line.
A `user/init.lua` that binds its own keys after this line keeps them, since a later binding of the same key replaces the preset's.

```lua defaults/init.lua
gband.plugin("gband.errors")
if gband.settings.sidebar() ~= false then
  gband.plugin("gband.sidebar")
end
```

The error list is always set up, and the sidebar unless the settings window turned it off.
`gband.settings.sidebar()` returns nil when nothing is saved, which counts as on, so the comparison is with `false` rather than a plain test.
Turning the sidebar off in the settings window saves `user/sidebar.lua`, which reloads the configuration and runs this file again, now without the sidebar.

```lua defaults/init.lua
gband.on("Attached", function()
  local settings = gband.settings
  if gband.config_dir and settings.theme() == nil and settings.sidebar() == nil and gband.keystyle.saved() == nil then
    settings.open()
  end
end)
```

`Attached` runs once the client has the session's layout, before it handles a key.
On a first start, nothing is saved yet, and the settings window opens so you can pick a theme, the sidebar and the key style.
Once any of the three is saved, it no longer opens by itself.
Without a configuration directory there is nowhere to save, so it never opens.
The window opens in a handler and not at top level, since `gband.settings.open` refuses to run while the configuration loads, as [07](07-settings.md) shows.

## The server

```lua defaults/server.lua
gband.opt.default_column_width = 1/2
gband.opt.width_presets = { 1/3, 1/2, 2/3 }
```

The server's default configuration sets its two options to their defaults: the width of a new column, half the screen, and the widths that cycling a column steps through.
The server has no prelude and no `gband.core`; everything it offers is the Rust API.

## What the earlier chapters add up to

Following the client from start to the first key:

1. The executable installs `gband`, with `gband.core`, and requires `gband.prelude` ([00](00-boundary.md)).
2. The prelude installs `gband.hl`, `gband.palette`, `gband.colorscheme`, `gband.bar`, `gband.win`, `gband.settings` and `gband.keystyle`, and each registers its provider ([02](02-highlights.md) to [08](08-key-styles.md)).
3. The prelude loads the saved theme, which sets the palette and the highlight groups ([01](01-themes.md), [03](03-colorschemes.md)).
4. `defaults/init.lua` sets the options, requires the key style preset, which sets up the key list and the prompt ([08](08-key-styles.md), [10](10-keylist-and-prompt.md)), and sets up the error list and the sidebar ([09](09-sidebar-and-errors.md)).
5. The client flushes the providers: `gband.bar` places and presents the sidebar, which draws itself in `on_resize`.
6. On `Attached`, the settings window opens on a first start ([07](07-settings.md)).

Every step after the first is a Lua file under `defaults/` that you can read, copy and replace.
