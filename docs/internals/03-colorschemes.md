# 03 · Colorschemes

[01](01-themes.md) read a theme and [02](02-highlights.md) the modules it calls.
This chapter reads the two modules in between: `gband.colorscheme`, which finds and loads a colorscheme, in `defaults/lua/gband/colorscheme.lua`, and `gband.theme`, which turns a palette and five interface colors into highlight groups, in `defaults/lua/gband/theme.lua`.

## Finding a colorscheme

```lua defaults/lua/gband/colorscheme.lua
local core = gband.core
local hl = require("gband.hl")
```

`require("gband.hl")` returns the exports of [02](02-highlights.md), not `gband.hl`.
The prelude has already required `gband.hl`, so this `require` returns the same table and runs nothing again.

```lua defaults/lua/gband/colorscheme.lua
local active = nil
```

`active` is the name of the loaded colorscheme, which `gband.colorscheme()` returns.

```lua defaults/lua/gband/colorscheme.lua
local function find(name)
  for _, entry in ipairs(gband.runtimepath or {}) do
    local path = entry .. "/colors/" .. name .. ".lua"
    local file = io.open(path, "r")
    if file then
      file:close()
      return function()
        return core.load(path)()
      end
    end
  end
  return core.bundled(name)
end
```

`find` returns a function that runs the colorscheme, or nil.
It looks for `colors/<name>.lua` in each runtimepath entry, so `user/colors/gruvbox.lua` replaces the bundled `gruvbox`, and a plugin can ship themes in its own `colors/`.
It only checks that the file opens; `gband.core.load` compiles it later, inside the function, so a syntax error becomes a failure of the load below rather than of `find`.
When no entry holds the file, `gband.core.bundled` returns the bundled colorscheme of that name as a function, or nil.

## Loading one

```lua defaults/lua/gband/colorscheme.lua
local function load(name, level)
  local label = "colors/" .. tostring(name)
  if type(name) ~= "string" or not name:match("^[A-Za-z0-9][A-Za-z0-9_-]*$") then
    core.report(label, "invalid colorscheme name `" .. tostring(name) .. "`", level)
    return false
  end
  local loader = find(name)
  if not loader then
    core.report(label, "no colorscheme `" .. name .. "` is on the runtimepath or bundled with gband", level)
    return false
  end
```

A colorscheme's failures are reported, not raised.
`gband.core.report` puts the message in the error list under the label `colors/<name>`, at the line `level` frames up the stack.
The name must look like a file name, so a name such as `../init` can never reach another file.

```lua defaults/lua/gband/colorscheme.lua
  local saved = hl.snapshot()
  local palette = core.palette.get()
  local quiet = hl.quiet
  hl.clear()
  core.palette.set({})
  hl.quiet = true
  local ok = core.call(nil, label, loader)
  hl.quiet = quiet
```

Loading a colorscheme replaces the previous one entirely.
Before the new one runs, every explicit highlight style is cleared and the palette is handed back to the terminal, so a theme that sets fewer groups than the last one does not inherit the rest.
`hl.quiet` stops every `gband.hl.set` in the theme from emitting its own `HighlightChanged`; a single `ColorschemeChanged` follows instead.
The previous value of `quiet` is restored rather than set to `false`, in case a caller had set it.

`gband.core.call` runs the colorscheme as code of no plugin, with an instruction limit of its own, so a theme that loops forever cannot hang the client.
It returns `false` when the colorscheme raised an error, after reporting it with the label.

```lua defaults/lua/gband/colorscheme.lua
  if not ok then
    hl.restore(saved)
    core.palette.set(palette)
    return false
  end
```

A failed colorscheme leaves no trace: the snapshot of explicit styles and the saved palette go back, and the previous theme stays active.

```lua defaults/lua/gband/colorscheme.lua
  local previous = active
  active = name
  if not core.loading() then
    core.emit("ColorschemeChanged", { name = name, previous = previous })
  end
  return true
end
```

Like `HighlightChanged` in [02](02-highlights.md), `ColorschemeChanged` is not emitted while the configuration loads.

## The public function

```lua defaults/lua/gband/colorscheme.lua
function gband.colorscheme(name)
  if name == nil then
    return active
  end
  local loaded = load(name, 3)
  return loaded
end
```

`gband.colorscheme()` with no argument returns the active name.
With a name it loads that colorscheme and returns whether it loaded.
It passes level 3 to `load`, which hands it to `gband.core.report`: level 1 is `load`, level 2 is `gband.colorscheme` and level 3 is the line that called it, so an error names `user/init.lua:7:` rather than a line of this file.

## The start theme

```lua defaults/lua/gband/colorscheme.lua
local function start()
  local saved = gband.settings.theme()
  if saved and load(saved, nil) then
    return
  end
  load("default", nil)
end
```

`start` is what the prelude's last line runs.
It reads the saved theme from `user/theme.lua` through `gband.settings.theme()`, which [07](07-settings.md) reads.
`gband.settings` is installed after this module, but `start` runs only once the prelude has required every module, so it is there by then.
A saved theme that no longer loads, because you deleted `user/colors/dusk.lua`, falls back to `default`, and its failure stays in the error list.
Level nil tells `gband.core.report` to name no line, since no line of configuration asked for the theme.

```lua defaults/lua/gband/colorscheme.lua
return { start = start }
```

The module's export is `start`, used only by the prelude.

## `gband.theme`

```lua defaults/lua/gband/theme.lua
local theme = {}
```

```lua defaults/lua/gband/theme.lua
function theme.apply(spec)
  local palette, ui = spec.palette, spec.ui
  local fg, bg = palette.fg, palette.bg
  gband.palette.set(palette)
  gband.hl.set("Bar", { bg = ui.surface })
  gband.hl.set("SidebarMode", { fg = ui.accent, bold = true })
  gband.hl.set("SidebarBand", { fg = ui.muted })
  gband.hl.set("SidebarBandActive", { fg = fg, bold = true })
  gband.hl.set("SidebarError", { fg = ui.error, bold = true })
  gband.hl.set("KeyListKey", { fg = ui.accent, bold = true })
  gband.hl.set("KeyListMuted", { fg = ui.muted })
  gband.hl.set("PluginWindow", { fg = fg, bg = ui.surface })
  gband.hl.set("PluginWindowBorder", { fg = ui.muted, bg = ui.surface })
  gband.hl.set("PluginWindowTitle", { fg = ui.accent, bg = ui.surface, bold = true })
  gband.hl.set("PluginWindowCursorLine", { fg = fg, bg = ui.selection, bold = true })
  gband.hl.set("PromptCursor", { fg = bg, bg = ui.accent })
  gband.hl.set("SettingsLabel", { fg = ui.muted })
  gband.hl.set("WindowBorder", { fg = ui.muted })
  gband.hl.set("WindowBorderFocused", { fg = ui.accent, bold = true })
  gband.hl.set("ErrorBanner", { fg = bg, bg = ui.error, bold = true })
end
```

`gband.theme` is not an API module: the prelude does not require it, and it sets no field of `gband`.
It is an ordinary module that colorschemes `require`, as `gruvbox` does.

`apply` sets the palette, then gives every group the interface draws with an explicit style from the five interface colors.
Keeping this table here, rather than in each theme, means a group added to gband is styled by every theme at once, and a theme of your own is only a list of colors, as [the scripting tutorial](../tutorial/06-colors.md) shows.
`PromptCursor` and `ErrorBanner` use the palette's `bg` as their foreground, so their text reads as a cut-out of the background.

```lua defaults/lua/gband/theme.lua
return theme
```
