# 08 · Key styles

gband's keys come in two styles.
In the modal style, the prefix key, Ctrl+Space, enters a navigation mode where each key acts until you leave it.
In the direct style, each key after the prefix acts once.
This chapter reads the module that chooses between them, `defaults/lua/gband/keystyle.lua`, and the two presets it chooses from, `defaults/keystyle/modal.lua` and `defaults/keystyle/direct.lua`, in full.

## `gband.keystyle`

```lua defaults/lua/gband/keystyle.lua
local core = gband.core
```

```lua defaults/lua/gband/keystyle.lua
local FILE = "user/keystyle.lua"
```

```lua defaults/lua/gband/keystyle.lua
local used = false
```

`used` makes sure that one load makes the bindings of one style.

```lua defaults/lua/gband/keystyle.lua
local function is_style(value)
  return value == "modal" or value == "direct"
end
```

```lua defaults/lua/gband/keystyle.lua
local function saved()
  return require("gband.settings").read(FILE, is_style)
end
```

`saved` reads the style that the settings window's `keys` line saved, with the `read` export of `gband.settings` from [07](07-settings.md).
It requires the module rather than using `gband.settings`, because `read` is an export, not part of the public table.
Anything but `"modal"` or `"direct"` reads as nothing saved.

```lua defaults/lua/gband/keystyle.lua
local function use(style)
  if not core.loading() then
    error("gband.keystyle.use can only be called while the configuration loads", 2)
  end
  if used then
    error("gband.keystyle.use can only be called once", 2)
  end
  if style == nil then
    style = saved() or "modal"
  end
  if not is_style(style) then
    error('gband.keystyle.use expects "modal" or "direct", got `' .. tostring(style) .. "`", 2)
  end
  used = true
  require("gband.keystyle." .. style)
  return style
end
```

`use` makes bindings, and bindings can only be made while the configuration loads, so it refuses to run at any other time, and more than once.
With no argument it takes the saved style, or `modal` when there is none.
The work is a single `require`: each preset is a module whose top-level code makes the bindings.
Because the presets are found by `require`, a `user/lua/gband/keystyle/direct.lua` replaces the direct style for everyone who calls `use`.

```lua defaults/lua/gband/keystyle.lua
gband.keystyle = {
  use = use,
  saved = saved,
}
```

Like `gband.palette`, `gband.bar` and `gband.win`, the module returns nothing: everything it offers is in `gband.keystyle`.

## The modal preset

A preset is a configuration file in all but name: plain top-level calls, as `user/init.lua` holds.

```lua defaults/keystyle/modal.lua
gband.plugin("gband.keylist")
gband.plugin("gband.prompt")
```

Both presets bind actions of the key list and the Lua prompt, which are bundled plugins, so they set them up first; [10](10-keylist-and-prompt.md) reads them.
`gband.plugin` requires each plugin's module and calls its `setup`, which registers the plugin's actions, so they exist before the bindings below name them.

```lua defaults/keystyle/modal.lua
local set = gband.keymap.set
local action = gband.action
```

```lua defaults/keystyle/modal.lua
gband.keymap.mode("prefix", { label = "navigation" })
```

`gband.keymap.mode` makes the `prefix` table a mode.
While a mode is active, a key it does not bind is discarded instead of reaching the window, and the mode stays active after each key.
The sidebar shows the label's first letter, `N`, while the mode is active, as [09](09-sidebar-and-errors.md) shows.

```lua defaults/keystyle/modal.lua
local function interactive()
  gband.keymap.enter("root")
end
```

Leaving the mode is entering `root` again, which the style calls interactive mode.

```lua defaults/keystyle/modal.lua
set("prefix", "h", action.focus_column_left, { desc = "focus the column to the left" })
set("prefix", "l", action.focus_column_right, { desc = "focus the column to the right" })
set("prefix", "j", action.focus_window_down, { desc = "focus the window below" })
set("prefix", "k", action.focus_window_up, { desc = "focus the window above" })
set("prefix", "u", action.focus_band_down, { desc = "view the band below" })
set("prefix", "i", action.focus_band_up, { desc = "view the band above" })
set("prefix", "c", action.center_column, { desc = "center the focused column" })
```

The navigation keys, `h`, `j`, `k` and `l` for columns and windows, and `u` and `i` for bands.

```lua defaults/keystyle/modal.lua
local interactive_on_new = gband.settings.interactive_on_new() ~= false
```

```lua defaults/keystyle/modal.lua
set("prefix", "n", function()
  action.open_window()
  if interactive_on_new then
    interactive()
  end
end, { desc = "open a window" })
```

`n` opens a window and, by default, leaves the mode so you can type in it.
The choice is read once, when the preset loads, from the `I on new` line of the settings window.
Changing that line saves the file, which reloads the configuration and loads the preset again.

```lua defaults/keystyle/modal.lua
set("prefix", "q", action.close_window, { desc = "close the window" })
set("prefix", "[", action.consume_or_expel_left, { desc = "consume or expel the window to the left" })
set("prefix", "]", action.consume_or_expel_right, { desc = "consume or expel the window to the right" })
set("prefix", "r", action.cycle_column_width, { desc = "cycle the width of the window's column" })
set("prefix", "f", action.toggle_full_width, { desc = "toggle full width of the window's column" })
set("prefix", "-", action.shrink_column_width, { desc = "shrink the width of the window's column" })
set("prefix", "=", action.grow_column_width, { desc = "grow the width of the window's column" })
set("prefix", "_", action.shrink_window_height, { desc = "shrink the height of the window" })
set("prefix", "+", action.grow_window_height, { desc = "grow the height of the window" })
set("prefix", "R", action.reset_window_height, { desc = "reset the height of the window" })
set("prefix", "v", action.toggle_window_floating, { desc = "float or tile the window" })
set("prefix", "V", action.switch_focus_floating_tiled, { desc = "switch focus between floating and tiled windows" })
```

```lua defaults/keystyle/modal.lua
set("prefix", "ctrl+h", action.move_column_left, { desc = "move the column or floating window to the left" })
set("prefix", "ctrl+l", action.move_column_right, { desc = "move the column or floating window to the right" })
set("prefix", "ctrl+j", action.move_window_down, { desc = "move the window down" })
set("prefix", "ctrl+k", action.move_window_up, { desc = "move the window up" })
set("prefix", "ctrl+left", action.move_column_left, { desc = "move the column or floating window to the left" })
set("prefix", "ctrl+right", action.move_column_right, { desc = "move the column or floating window to the right" })
set("prefix", "ctrl+down", action.move_window_down, { desc = "move the window down" })
set("prefix", "ctrl+up", action.move_window_up, { desc = "move the window up" })
```

The actions that change the layout.
Each of the `move_*` actions is bound twice, to Ctrl with `hjkl` and to Ctrl with an arrow.

```lua defaults/keystyle/modal.lua
set("prefix", "?", action["keylist.open"], { desc = "list the keys" })
set("prefix", ":", action["prompt.open"], { desc = "run Lua" })
set("prefix", "N", action["prompt.rename"], { desc = "rename the window" })
set("prefix", "s", gband.settings.open, { desc = "settings" })
set("prefix", "D", action.detach, { desc = "detach" })
set("prefix", "escape", interactive, { desc = "interactive mode" })
set("prefix", "enter", interactive, { desc = "interactive mode" })
```

`?`, `:` and `N` open plugin windows, which take the keys from there on, and `s` opens the settings window.
Escape and Enter return to interactive mode.

```lua defaults/keystyle/modal.lua
set("prefix", "left", action.focus_column_left, { desc = "focus the column to the left" })
set("prefix", "right", action.focus_column_right, { desc = "focus the column to the right" })
set("prefix", "down", action.focus_window_down, { desc = "focus the window below" })
set("prefix", "up", action.focus_window_up, { desc = "focus the window above" })
```

The arrows move focus as `hjkl` do.

```lua defaults/keystyle/modal.lua
set("prefix", "prefix", function()
  action.send_prefix()
  interactive()
end, { desc = "send the prefix key" })
```

Pressing the prefix key in the mode sends it to the window, so a program in the window can still receive Ctrl+Space, and leaves the mode.
The key name `prefix` stands for whatever `gband.opt.prefix` holds.

```lua defaults/keystyle/modal.lua
set("prefix", "leftmouse", action.drag_window, { desc = "move the window with the mouse" })
set("prefix", "rightmouse", action.drag_resize_window, { desc = "resize the window with the mouse" })
set("prefix", "middlemouse", action.drag_band, { desc = "slide the band or switch bands with the mouse" })
set("prefix", "mod+leftmouse", action.drag_window, { desc = "move the window with the mouse" })
set("prefix", "mod+rightmouse", action.drag_resize_window, { desc = "resize the window with the mouse" })
set("prefix", "mod+middlemouse", action.drag_band, { desc = "slide the band or switch bands with the mouse" })
set("prefix", "mod+wheeldown", action.focus_band_down, { desc = "view the band below" })
set("prefix", "mod+wheelup", action.focus_band_up, { desc = "view the band above" })
```

In the mode, the mouse buttons drag without a modifier: the left button moves a window, the right one resizes it, and the middle one slides the band.
`mod` is the modifier `gband.opt.mouse_mod` names, Alt by default.

```lua defaults/keystyle/modal.lua
set("root", "mod+leftmouse", action.drag_window, { desc = "move the window with the mouse" })
set("root", "mod+rightmouse", action.drag_resize_window, { desc = "resize the window with the mouse" })
set("root", "mod+middlemouse", action.drag_band, { desc = "slide the band or switch bands with the mouse" })
set("root", "mod+wheeldown", action.focus_band_down, { desc = "view the band below" })
set("root", "mod+wheelup", action.focus_band_up, { desc = "view the band above" })
```

Outside the mode, the same drags need the modifier, so that a plain click still reaches the program in the window.

## The direct preset

```lua defaults/keystyle/direct.lua
gband.plugin("gband.keylist")
gband.plugin("gband.prompt")
```

```lua defaults/keystyle/direct.lua
local set = gband.keymap.set
local action = gband.action
```

The direct preset declares no mode.
After the prefix key, gband looks the next key up in `prefix`, runs its binding and goes back to `root`.

```lua defaults/keystyle/direct.lua
set("prefix", "h", action.focus_column_left, { desc = "focus the column to the left" })
set("prefix", "l", action.focus_column_right, { desc = "focus the column to the right" })
set("prefix", "j", action.focus_window_down, { desc = "focus the window below" })
set("prefix", "k", action.focus_window_up, { desc = "focus the window above" })
set("prefix", "u", action.focus_band_down, { desc = "view the band below" })
set("prefix", "i", action.focus_band_up, { desc = "view the band above" })
set("prefix", "c", action.center_column, { desc = "center the focused column" })
```

```lua defaults/keystyle/direct.lua
set("prefix", "n", action.open_window, { desc = "open a window running the user's shell" })
```

`n` opens a window and nothing more: there is no mode to leave.

```lua defaults/keystyle/direct.lua
set("prefix", "q", action.close_window, { desc = "close the window" })
set("prefix", "[", action.consume_or_expel_left, { desc = "consume or expel the window to the left" })
set("prefix", "]", action.consume_or_expel_right, { desc = "consume or expel the window to the right" })
set("prefix", "r", action.cycle_column_width, { desc = "cycle the width of the window's column" })
set("prefix", "f", action.toggle_full_width, { desc = "toggle full width of the window's column" })
set("prefix", "-", action.shrink_column_width, { desc = "shrink the width of the window's column" })
set("prefix", "=", action.grow_column_width, { desc = "grow the width of the window's column" })
set("prefix", "_", action.shrink_window_height, { desc = "shrink the height of the window" })
set("prefix", "+", action.grow_window_height, { desc = "grow the height of the window" })
set("prefix", "R", action.reset_window_height, { desc = "reset the height of the window" })
set("prefix", "v", action.toggle_window_floating, { desc = "float or tile the window" })
set("prefix", "V", action.switch_focus_floating_tiled, { desc = "switch focus between floating and tiled windows" })
set("prefix", "ctrl+h", action.move_column_left, { desc = "move the column or floating window to the left" })
set("prefix", "ctrl+l", action.move_column_right, { desc = "move the column or floating window to the right" })
set("prefix", "ctrl+j", action.move_window_down, { desc = "move the window down" })
set("prefix", "ctrl+k", action.move_window_up, { desc = "move the window up" })
set("prefix", "ctrl+left", action.move_column_left, { desc = "move the column or floating window to the left" })
set("prefix", "ctrl+right", action.move_column_right, { desc = "move the column or floating window to the right" })
set("prefix", "ctrl+down", action.move_window_down, { desc = "move the window down" })
set("prefix", "ctrl+up", action.move_window_up, { desc = "move the window up" })
```

```lua defaults/keystyle/direct.lua
set("prefix", "?", action["keylist.open"], { desc = "list the keys" })
set("prefix", ":", action["prompt.open"], { desc = "run Lua" })
set("prefix", "N", action["prompt.rename"], { desc = "rename the window" })
set("prefix", "s", gband.settings.open, { desc = "settings" })
set("prefix", "D", action.detach, { desc = "detach" })
```

```lua defaults/keystyle/direct.lua
set("prefix", "left", action.focus_column_left, { desc = "focus the column to the left" })
set("prefix", "right", action.focus_column_right, { desc = "focus the column to the right" })
set("prefix", "down", action.focus_window_down, { desc = "focus the window below" })
set("prefix", "up", action.focus_window_up, { desc = "focus the window above" })
```

```lua defaults/keystyle/direct.lua
set("prefix", "prefix", action.send_prefix, { desc = "send the prefix key to the focused window" })
```

The prefix key twice sends one prefix key to the window.
There is no Escape or Enter binding, since there is no mode to leave.

```lua defaults/keystyle/direct.lua
set("prefix", "leftmouse", action.drag_window, { desc = "move the window with the mouse" })
set("prefix", "rightmouse", action.drag_resize_window, { desc = "resize the window with the mouse" })
set("prefix", "middlemouse", action.drag_band, { desc = "slide the band or switch bands with the mouse" })
set("prefix", "mod+leftmouse", action.drag_window, { desc = "move the window with the mouse" })
set("prefix", "mod+rightmouse", action.drag_resize_window, { desc = "resize the window with the mouse" })
set("prefix", "mod+middlemouse", action.drag_band, { desc = "slide the band or switch bands with the mouse" })
set("prefix", "mod+wheeldown", action.focus_band_down, { desc = "view the band below" })
set("prefix", "mod+wheelup", action.focus_band_up, { desc = "view the band above" })
```

```lua defaults/keystyle/direct.lua
set("root", "mod+leftmouse", action.drag_window, { desc = "move the window with the mouse" })
set("root", "mod+rightmouse", action.drag_resize_window, { desc = "resize the window with the mouse" })
set("root", "mod+middlemouse", action.drag_band, { desc = "slide the band or switch bands with the mouse" })
set("root", "mod+wheeldown", action.focus_band_down, { desc = "view the band below" })
set("root", "mod+wheelup", action.focus_band_up, { desc = "view the band above" })
```

The rest is the modal preset's.
The two files keep the same keys for the same actions, so switching styles changes only how long the prefix lasts.

## Why the presets are written to `defaults/keystyle/`

Every other bundled module is written to `defaults/lua/gband/`, at the path `require` would find it if `defaults/` were on the runtimepath.
The presets are modules too, `gband.keystyle.modal` and `gband.keystyle.direct`, yet their copies are `defaults/keystyle/modal.lua` and `defaults/keystyle/direct.lua`.

The presets were written there before the API modules were, so that you could read a preset beside `defaults/init.lua` and copy its bindings into your own `user/init.lua`: a preset is a configuration, not a library.
When the API modules gained copies under `defaults/lua/gband/`, the presets kept the place the README and the key style documentation already gave them.
To replace a preset as a module, copy it to `user/lua/gband/keystyle/`, as [00](00-boundary.md) shows.
