# 08 · Key styles

gband's keys come in three styles.
In the modal style, the prefix key, Ctrl+Space, enters a navigation mode where each key acts until you leave it.
In the direct style, each key after the prefix acts once.
In the floating style, every window floats as on a desktop, and the prefix key opens a list of the windows.
This chapter reads the module that chooses between them, `defaults/lua/gband/keystyle.lua`, the three presets it chooses from, `defaults/keystyle/modal.lua`, `defaults/keystyle/direct.lua` and `defaults/keystyle/floating.lua`, and the bundled plugin the floating preset sets up, `defaults/lua/gband/desktop.lua`, in full.

## `gband.keystyle`

```lua defaults/lua/gband/keystyle.lua
local core = gband.core
```

```lua defaults/lua/gband/keystyle.lua
local FILE = "user/keystyle.lua"
```

```lua defaults/lua/gband/keystyle.lua
local used
```

`used` holds the style that one load picked, so that one load makes the bindings of one style, and so that `current` below can name it.

```lua defaults/lua/gband/keystyle.lua
local function is_style(value)
  return value == "modal" or value == "direct" or value == "floating"
end
```

```lua defaults/lua/gband/keystyle.lua
local function saved()
  return require("gband.settings").read(FILE, is_style)
end
```

`saved` reads the style that the settings window's `keys` line saved, with the `read` export of `gband.settings` from [07](07-settings.md).
It requires the module rather than using `gband.settings`, because `read` is an export, not part of the public table.
Anything but `"modal"`, `"direct"` or `"floating"` reads as nothing saved.

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
    error('gband.keystyle.use expects "modal", "direct" or "floating", got `' .. tostring(style) .. "`", 2)
  end
  used = style
  require("gband.keystyle." .. style)
  return style
end
```

`use` makes bindings, and bindings can only be made while the configuration loads, so it refuses to run at any other time, and more than once.
With no argument it takes the saved style, or `modal` when there is none.
The work is a single `require`: each preset is a module whose top-level code makes the bindings.
Because the presets are found by `require`, a `user/lua/gband/keystyle/direct.lua` replaces the direct style for everyone who calls `use`.

```lua defaults/lua/gband/keystyle.lua
local function current()
  return used
end
```

`current` names the style `use` picked in this load, or nil.
A configuration that requires a preset by its module name never calls `use`, so `current` stays nil there: the sidebar of [09](09-sidebar-and-errors.md) asks it whether to show the floating style's apps character, and such a configuration keeps the mode letter.

```lua defaults/lua/gband/keystyle.lua
gband.keystyle = {
  use = use,
  saved = saved,
  current = current,
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
local interactive_on_new = gband.settings.interactive_on_new() == true
```

```lua defaults/keystyle/modal.lua
set("prefix", "n", function()
  action.open_window()
  if interactive_on_new then
    interactive()
  end
end, { desc = "open a window" })
```

`n` opens a window and leaves navigation mode active, so a run of layout keys can follow it, unless the `I on new` line of the settings window is turned on; then it leaves the mode so you can type in the new window.
The choice is read once, when the preset loads, and only a saved `true` turns it on: nothing saved reads as off.
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

## The floating preset

The floating preset, `defaults/keystyle/floating.lua`, keeps the shape of the other two and binds far less: its windows are driven by the mouse and a list, not by layout keys.

```lua defaults/keystyle/floating.lua
gband.plugin("gband.keylist")
gband.plugin("gband.prompt")
gband.plugin("gband.desktop")
```

It sets up a third bundled plugin after the key list and the Lua prompt: `gband.desktop`, which the next section reads.
Everything with state, the windows that float, the buttons, the menus, lives there, so the preset stays a list of bindings.

```lua defaults/keystyle/floating.lua
local set = gband.keymap.set
local action = gband.action
```

```lua defaults/keystyle/floating.lua
set("prefix", "n", function()
  action.open_window({ floating = true })
end, { desc = "open a floating window" })
set("prefix", "?", action["keylist.open"], { desc = "list the keys" })
set("prefix", ":", action["prompt.open"], { desc = "run Lua" })
set("prefix", "N", action["prompt.rename"], { desc = "rename the window" })
set("prefix", "s", gband.settings.open, { desc = "settings" })
set("prefix", "D", action.detach, { desc = "detach" })
set("prefix", "prefix", action.send_prefix, { desc = "send the prefix key to the focused window" })
```

The preset declares no mode, as the direct one does.
Its `n` opens a floating window, through a function because the action needs a target, and the rest are the keys that make sense without a tiled layout: the key list, the Lua prompt, the rename prompt, the settings, detach, and the prefix key twice.

```lua defaults/keystyle/floating.lua
set("root", "mod+leftmouse", action.drag_window, { desc = "move the window with the mouse" })
set("root", "mod+rightmouse", action.drag_resize_window, { desc = "resize the window with the mouse" })
set("root", "mod+middlemouse", action.drag_band, { desc = "slide the band or switch bands with the mouse" })
set("root", "mod+wheeldown", action.focus_band_down, { desc = "view the band below" })
set("root", "mod+wheelup", action.focus_band_up, { desc = "view the band above" })
set("root", "leftmouse", action["desktop.press"], {
  desc = "move, resize or press a button of a floating window",
})
set("root", "rightmouse", action["desktop.menu"], { desc = "open the menu for the cell under the pointer" })
```

`root` keeps the five `mod+` bindings of the other presets, so Alt with a drag still moves and resizes any window.
Then plain `leftmouse` and `rightmouse` go to two actions of the desktop plugin.
Binding them in `root` would take every plain click away from the programs, so both actions decline every press they do not use by returning `false`, and that press takes its interactive defaults.
The preset binds no mouse name in `prefix`, because `prefix` is active for an instant only.

```lua defaults/keystyle/floating.lua
gband.on("KeyTableChanged", function(event)
  if event.table == "prefix" then
    gband.keymap.enter("root")
    action["desktop.leader"]()
  end
end)
```

This is the leader.
`prefix` holds bindings and is not a mode, so the prefix key makes it active, and the client emits `KeyTableChanged` before it reads the next key.
The handler returns to `root` at once and dispatches `desktop.leader`, which opens the window list, so the next key reaches the list instead of being looked up in `prefix`.
The list runs `prefix` bindings itself, as the key list of [10](10-keylist-and-prompt.md) does.

## The desktop plugin

`defaults/lua/gband/desktop.lua` is the largest bundled plugin, and it is built from the same public API as the others: layout reads, window functions, plugin windows, a provider and event handlers.

```lua defaults/lua/gband/desktop.lua
local hl = require("gband.hl")
local key_form = require("gband.keyform")
```

```lua defaults/lua/gband/desktop.lua
gband.hl.default("DesktopButton", { bold = true })
gband.hl.default("DesktopClose", { fg = 1, bold = true })
gband.hl.default("DesktopMinimized", { dim = true })
```

The groups are defined as defaults when the module is first required, as every bundled plugin does.

```lua defaults/lua/gband/desktop.lua
local MINIMIZE, MAXIMIZE, RESTORE, CLOSE = "[_]", "[□]", "[❐]", "[X]"
local LIST_TITLE = "windows"
local LIST_MIN_WIDTH = 24
local LIST_MAX_HEIGHT = 15
local MENU_MIN_WIDTH = 16
local MENU_HEIGHT = 7
local HINT = "? keys"
local LETTERS = "abcdefghijklmnopqrstuvwxyz"
local MENU_KEYS = {
  j = true, down = true, k = true, up = true, pagedown = true, pageup = true,
  home = true, ["end"] = true, enter = true, esc = true, q = true,
}
local CORNERS = {
  { "left", "top" },
  { "right", "top" },
  { "left", "bottom" },
  { "right", "bottom" },
}
```

The glyphs of the buttons, the sizes of the two menus, the keys the menus keep for themselves in key form, and the four corner zones in the order they are checked.

```lua defaults/lua/gband/desktop.lua
local remembered = {}
local pending = {}
local states = {}
local size = nil
local pressed = nil
local pressing = false
local menu = nil
```

The state of one client: the box to restore per window, the windows still to be placed by the cascade, the maximized or tiled state each floating window had in the last layout, the screen area's size of the last layout, a pressed title bar button, whether a mouse binding of this plugin has run for a press that its `MousePressed` handler has not seen yet, and the open menu.

```lua defaults/lua/gband/desktop.lua
local function find_box(layout, id)
  for _, band in ipairs(layout.bands) do
    for _, box in ipairs(band.floating) do
      if box.id == id then
        return box
      end
    end
  end
  return nil
end

local function box_width(box, cols)
  if box.full_width then
    return cols
  end
  return math.max(3, math.min(cols, math.floor(cols * box.width + 1e-9)))
end
```

`find_box` finds a window's table among the floating windows of every band.
`box_width` turns a box's width, a proportion of the screen area, into cells, as the floating-windows geometry does; the small epsilon keeps a proportion such as 1/3 from rounding down a cell short.

```lua defaults/lua/gband/desktop.lua
local function state_of(box, cols, rows)
  if box.row ~= 0 or box.rows < rows then
    return nil
  end
  if box.full_width or box.width == 1 then
    return box.col == 0 and "maximized" or nil
  end
  if box.width == 1 / 2 then
    if box.col == 0 then
      return "left"
    end
    if box.col == cols - cols // 2 then
      return "right"
    end
  end
  return nil
end

local function states_of(layout)
  local found = {}
  for _, band in ipairs(layout.bands) do
    for _, box in ipairs(band.floating) do
      found[box.id] = state_of(box, layout.cols, layout.rows)
    end
  end
  return found
end

local function remember_layout(layout)
  size = { cols = layout.cols, rows = layout.rows }
  states = states_of(layout)
end
```

A window's state is read from its box alone: at the top of the screen area and at least as tall, and either the whole width, or half of it on the left or on the right.
So the state survives a reload and a move from another client, and the plugin never has to remember it.
`states_of` takes the state of every floating window of one layout, and `remember_layout` keeps it with the size it was measured against.

```lua defaults/lua/gband/desktop.lua
local function set_box(id, width, rows, col, row)
  gband.window.set_width(id, width)
  gband.window.set_height(id, { rows = rows })
  gband.window.set_position(id, { col = col, row = row })
end

local function apply_state(id, state, cols, rows)
  if state == "maximized" then
    set_box(id, 1, rows, 0, 0)
  elseif state == "left" then
    set_box(id, 1 / 2, rows, 0, 0)
  else
    set_box(id, 1 / 2, rows, cols - cols // 2, 0)
  end
end
```

Every box change sets the width, then the height, then the position, so that the server never clamps the new position against the old size.

```lua defaults/lua/gband/desktop.lua
local function enter_state(id, state)
  local layout = gband.layout()
  local box = find_box(layout, id)
  if not box then
    return
  end
  if state_of(box, layout.cols, layout.rows) == nil then
    remembered[id] = {
      col = box.col,
      row = box.row,
      width = box.width,
      full_width = box.full_width,
      rows = box.rows,
    }
  end
  apply_state(id, state, layout.cols, layout.rows)
  gband.window.focus(id)
end

local function restore(id)
  local layout = gband.layout()
  local saved = remembered[id]
  remembered[id] = nil
  if saved then
    gband.window.set_width(id, saved.width)
    if saved.full_width then
      gband.action.toggle_full_width({ window = id })
    end
    gband.window.set_height(id, { rows = saved.rows })
    gband.window.set_position(id, { col = saved.col, row = saved.row })
  else
    local cols, rows = layout.cols, layout.rows
    local height = math.max(3, rows // 2)
    set_box(id, 1 / 2, height, (cols - cols // 2) // 2, (rows - height) // 2)
  end
  gband.window.focus(id)
end

local function is_maximized(id)
  local layout = gband.layout()
  local box = find_box(layout, id)
  return box ~= nil and state_of(box, layout.cols, layout.rows) == "maximized"
end

local function toggle_maximized(id)
  if is_maximized(id) then
    restore(id)
  else
    enter_state(id, "maximized")
  end
end
```

Maximizing or tiling a window that is in no state first remembers its box, so going from tiled left to maximized and back still restores the first box.
Restoring puts the remembered box back, full width included, or centres a box of half the screen area when nothing is remembered, such as after a reload.
Both focus the window afterwards.

```lua defaults/lua/gband/desktop.lua
local function window_buttons(box, cols, rows)
  local button = hl.drawn("DesktopButton")
  local maximized = state_of(box, cols, rows) == "maximized"
  return {
    { text = MINIMIZE, style = button },
    { text = maximized and RESTORE or MAXIMIZE, style = button },
    { text = CLOSE, style = hl.drawn("DesktopClose") },
  }
end

local function span_at(spans, width, col)
  local widths, total = {}, 0
  for index, span in ipairs(spans) do
    widths[index] = gband.ui.width(span.text)
    total = total + widths[index]
  end
  if total == 0 or total > width - 4 then
    return nil
  end
  local first = width - 2 - total
  for index, span_width in ipairs(widths) do
    if col >= first and col < first + span_width then
      return index
    end
    first = first + span_width
  end
  return nil
end

local function decorations(info)
  if not info.floating then
    return nil
  end
  local layout = gband.layout()
  local box = find_box(layout, info.window)
  if not (box and box.name) then
    return nil
  end
  return window_buttons(box, layout.cols, layout.rows)
end
```

`window_buttons` builds the three spans of a window's title bar, with the restore glyph while the window is maximized.
`span_at` maps a box column to the span under it with the placement rule of the decorations provider: spans end at the box's third-last column, and are not drawn at all when they do not fit.
`decorations` is the provider itself, registered in `setup`, and it gives spans only to floating windows that run a program, a window with a `name`.
The press handler below calls the same two functions, so what is drawn and what is hit cannot disagree.

```lua defaults/lua/gband/desktop.lua
local function is_open(win)
  for _, id in ipairs(gband.win.list()) do
    if id == win then
      return true
    end
  end
  return false
end

local function menu_open()
  return menu ~= nil and is_open(menu.win)
end

local function list_open()
  return menu_open() and menu.kind == "list"
end

local function close_menu()
  local current = menu
  menu = nil
  if current and is_open(current.win) then
    gband.win.close(current.win)
  end
end
```

The menus are floating plugin windows, so `is_open` asks `gband.win.list()` rather than trusting the plugin's own variable, and `close_menu` forgets the menu before it closes the window, so its `on_close` finds nothing to forget.

```lua defaults/lua/gband/desktop.lua
local function pad(text, width)
  return text .. string.rep(" ", width - gband.ui.width(text))
end

local function border_line(left, right, width, title, spans)
  local total = 0
  for _, span in ipairs(spans) do
    total = total + gband.ui.width(span.text)
  end
  if total > width - 4 then
    spans, total = {}, 0
  end
  local cut = gband.ui.truncate(title or "", math.max(0, total > 0 and width - 4 - total or width - 2))
  local start = total > 0 and width - 2 - total or width - 1
  local line = {
    { text = left, hl = "PluginWindowBorder" },
    { text = cut, hl = "PluginWindowTitle" },
    { text = string.rep("─", math.max(0, start - 1 - gband.ui.width(cut))), hl = "PluginWindowBorder" },
  }
  for _, span in ipairs(spans) do
    line[#line + 1] = span
  end
  if total > 0 then
    line[#line + 1] = { text = "─", hl = "PluginWindowBorder" }
  end
  line[#line + 1] = { text = right, hl = "PluginWindowBorder" }
  return line
end

local function row_line(row, width, selected)
  if row.separator then
    return { { text = "├" .. string.rep("─", math.max(0, width - 2)) .. "┤", hl = "PluginWindowBorder" } }
  end
  local inner = math.max(0, width - 4)
  return {
    { text = "│", hl = "PluginWindowBorder" },
    {
      text = " " .. pad(gband.ui.truncate(row.text, inner), inner) .. " ",
      hl = selected and "PluginWindowCursorLine" or row.hl or "PluginWindow",
    },
    { text = "│", hl = "PluginWindowBorder" },
  }
end
```

A menu has no border of its own: a plugin window's border can show a title, but nothing at its right end, where the buttons go.
So the menu draws its frame in its lines: a top or bottom border with an optional title and right-aligned spans, placed as decorations are, and a row that is either an entry between two side borders or a separator.

```lua defaults/lua/gband/desktop.lua
local function shown_rows()
  return math.max(1, gband.win.info(menu.win).height - 2)
end

local function draw()
  if not menu_open() then
    return
  end
  local info = gband.win.info(menu.win)
  local shown = math.max(0, info.height - 2)
  if menu.selected < menu.top then
    menu.top = menu.selected
  elseif menu.selected > menu.top + shown - 1 then
    menu.top = menu.selected - shown + 1
  end
  menu.top = math.max(1, math.min(menu.top, #menu.rows - shown + 1))
  local lines = { border_line("┌", "┐", info.width, menu.title, menu.buttons()) }
  for offset = 0, shown - 1 do
    local index = menu.top + offset
    lines[#lines + 1] = row_line(menu.rows[index] or { text = "" }, info.width, index == menu.selected)
  end
  lines[#lines + 1] = border_line("└", "┘", info.width, nil, menu.hint())
  gband.win.set_lines(menu.win, lines)
end
```

`draw` keeps the selected entry in view by moving the first shown row the least amount, then writes every line.
It reads the menu's size from `gband.win.info`, which gives the size after the ribbon cut the box, so a small terminal draws a smaller frame.

```lua defaults/lua/gband/desktop.lua
local function entry_from(index, step)
  while menu.rows[index] and menu.rows[index].separator do
    index = index + step
  end
  return menu.rows[index] and index or nil
end

local function select(index)
  if index then
    menu.selected = index
    draw()
  end
end

local function step(by)
  select(entry_from(menu.selected + by, by))
end

local function page(by)
  local target = menu.selected + by * shown_rows()
  if target > #menu.rows then
    select(entry_from(#menu.rows, -1))
  elseif target < 1 then
    select(entry_from(1, 1))
  else
    select(entry_from(target, by) or entry_from(target, -by))
  end
end
```

Moving the selection skips separators.
`page` moves by the rows shown, and lands on the first or last entry past either end.

```lua defaults/lua/gband/desktop.lua
local function pick(index)
  local row = menu.rows[index]
  if not row or row.separator then
    return
  end
  close_menu()
  row.act()
end

local function own(fn)
  return function(win)
    if menu and menu.win == win then
      fn()
    end
  end
end
```

Picking closes the menu before the entry acts, so an entry that opens another plugin window gives it focus.
`own` wraps a key function so it acts only on the menu that is open now.

```lua defaults/lua/gband/desktop.lua
local function target_at(event)
  if event.box_row == nil then
    return nil
  end
  if event.box_row == 0 then
    local button = span_at(menu.buttons(), event.box_width, event.box_col)
    return button and { button = button } or nil
  end
  if event.box_row >= event.box_height - 1 then
    return nil
  end
  local index = menu.top + event.box_row - 1
  local row = menu.rows[index]
  if row and not row.separator then
    return { entry = index }
  end
  return nil
end

local function on_mouse(win, event)
  if not (menu and menu.win == win) then
    return
  end
  if event.kind == "scroll" then
    if event.direction == "down" then
      step(1)
    elseif event.direction == "up" then
      step(-1)
    end
    return
  end
  if event.button ~= "left" then
    return
  end
  local target = target_at(event)
  if event.kind == "press" then
    menu.pressed = target
    if target and target.entry then
      select(target.entry)
    end
  elseif event.kind == "release" then
    local before = menu.pressed
    menu.pressed = nil
    if not (before and target) then
      return
    end
    if before.entry and before.entry == target.entry then
      pick(target.entry)
    elseif before.button and before.button == target.button then
      menu.press(target.button)
    end
  end
end
```

Without a border, a box cell and a content cell are the same, and the plugin finds what is under the pointer from `box_row`: the top border holds the buttons, the bottom border nothing, and the rows between hold entries.
A left press selects an entry or arms a button, and only a release on the same entry or button acts, as on a desktop.
The wheel moves the selection.

```lua defaults/lua/gband/desktop.lua
local function first_entry(rows)
  for index, row in ipairs(rows) do
    if not row.separator then
      return index
    end
  end
  return 1
end

local function open_menu(spec, at)
  close_menu()
  gband.keymap.enter("root")
  spec.selected = first_entry(spec.rows)
  spec.top = 1
  spec.skip = pressing
  local keys = {
    j = own(function() step(1) end),
    down = own(function() step(1) end),
    k = own(function() step(-1) end),
    up = own(function() step(-1) end),
    pagedown = own(function() page(1) end),
    pageup = own(function() page(-1) end),
    home = own(function() select(entry_from(1, 1)) end),
    ["end"] = own(function() select(entry_from(#menu.rows, -1)) end),
    enter = own(function() pick(menu.selected) end),
    escape = own(close_menu),
    q = own(close_menu),
  }
  for key, fn in pairs(spec.keys or {}) do
    keys[key] = own(fn)
  end
  menu = spec
  spec.win = gband.win.open({
    border = false,
    width = spec.width,
    height = spec.height,
    col = at and math.max(0, at.col) or "center",
    row = at and math.max(0, at.row) or "center",
    keys = keys,
    on_mouse = on_mouse,
    on_resize = function(win)
      if menu and menu.win == win then
        draw()
      end
    end,
    on_close = function(win)
      if menu and menu.win == win then
        menu = nil
      end
    end,
  })
  draw()
end
```

`open_menu` closes any other menu first, so at most one is open, and returns to `root`, so the keys that follow reach the menu.
It records whether a press of this plugin is still on its way to the `MousePressed` handler, the press that opened the menu, which that handler must not take as a press outside.
The menu's keys are the movement keys, Enter, Escape and `q`, plus any the menu adds.

```lua defaults/lua/gband/desktop.lua
local function ribbon()
  local view = gband.view()
  return view.cols, view.rows
end

local function widest(rows)
  local found = 0
  for _, row in ipairs(rows) do
    if row.text then
      found = math.max(found, gband.ui.width(row.text))
    end
  end
  return found
end

local function band_label(position)
  if position <= 9 then
    return tostring(position)
  end
  if position <= 9 + #LETTERS then
    return LETTERS:sub(position - 9, position - 9)
  end
  return tostring(position)
end
```

The band label repeats the sidebar's rule from [09](09-sidebar-and-errors.md): positions 1 to 9, then letters.

```lua defaults/lua/gband/desktop.lua
local function list_rows(minimizing)
  local layout = gband.layout()
  local found, bands = {}, 0
  for position, band in ipairs(layout.bands) do
    local before = #found
    for _, column in ipairs(band.columns) do
      for _, window in ipairs(column.windows) do
        if window.name then
          found[#found + 1] = { position = position, window = window }
        end
      end
    end
    for _, box in ipairs(band.floating) do
      if box.name then
        found[#found + 1] = { position = position, window = box, minimized = box.minimized or box.id == minimizing }
      end
    end
    if #found > before then
      bands = bands + 1
    end
  end
  local rows = {
    { key = "new", text = "New window", act = function() gband.action.open_window({ floating = true }) end },
    { key = "settings", text = "Settings", act = function() gband.settings.open() end },
  }
  if #found > 0 then
    rows[#rows + 1] = { separator = true }
  end
  for _, item in ipairs(found) do
    local id = item.window.id
    local text = item.window.name
    if bands >= 2 then
      text = band_label(item.position) .. " " .. text
    end
    if item.minimized then
      text = text .. " (minimized)"
    end
    rows[#rows + 1] = {
      key = id,
      text = text,
      hl = item.minimized and "DesktopMinimized" or nil,
      act = function() gband.window.focus(id) end,
    }
  end
  return rows
end
```

The window list's rows: `New window`, `Settings`, then every window that runs a program, in layout order across every band, tiled windows first and floating ones in their floating order.
The band label appears once windows sit in two or more bands, and a window this client minimized is marked and dimmed.
`minimizing` names a window whose minimize this callback just dispatched, which the layout does not show yet.

```lua defaults/lua/gband/desktop.lua
local function list_size(rows)
  local cols, height = ribbon()
  return math.min(cols, math.max(LIST_MIN_WIDTH, widest(rows) + 4)),
    math.min(height, LIST_MAX_HEIGHT, #rows + 2)
end

local function binds_hint_key()
  for _, binding in ipairs(gband.keymap.list("prefix")) do
    if binding.key == "?" then
      return true
    end
  end
  return false
end

local function list_shortcuts()
  local keys = {}
  for _, binding in ipairs(gband.keymap.list("prefix")) do
    local key = binding.key
    if key ~= "prefix" and not key_form.is_mouse(key) and not MENU_KEYS[key_form(key)] then
      keys[key] = function()
        close_menu()
        gband.keymap.run("prefix", key)
      end
    end
  end
  return keys
end
```

The list is at least 24 cells wide and at most 15 rows high.
Its hint shows only while `prefix` binds `?`.
Every other `prefix` binding becomes a key of the list that closes it and runs the binding, except the prefix key itself, mouse names and the menu's own keys.

```lua defaults/lua/gband/desktop.lua
local function toggle_list_height()
  local info = gband.win.info(menu.win)
  if menu.restore then
    gband.win.set_config(menu.win, { row = menu.restore.row, height = menu.restore.height })
    menu.restore = nil
  else
    local _, rows = ribbon()
    menu.restore = { row = info.row, height = info.height }
    gband.win.set_config(menu.win, { row = 0, height = rows })
  end
  draw()
end

local function show_list(at)
  if list_open() and not at then
    gband.keymap.enter("root")
    gband.win.focus(menu.win)
    return
  end
  local rows = list_rows()
  local width, height = list_size(rows)
  local spec = { kind = "list", title = LIST_TITLE, rows = rows, width = width, height = height }
  function spec.buttons()
    return {
      { text = spec.restore and RESTORE or MAXIMIZE, hl = "DesktopButton" },
      { text = CLOSE, hl = "DesktopClose" },
    }
  end
  function spec.hint()
    return binds_hint_key() and { { text = HINT, hl = "PluginWindowTitle" } } or {}
  end
  function spec.press(button)
    if button == 1 then
      toggle_list_height()
    else
      close_menu()
    end
  end
  spec.keys = list_shortcuts()
  open_menu(spec, at)
end
```

The list's `[□]` gives it the ribbon's height from row 0 and remembers the row and height to give back.
`show_list` focuses an open list instead of opening a second one, unless a right press asks for it at the pointer, in which case it opens anew there.

```lua defaults/lua/gband/desktop.lua
local function rebuild(minimizing)
  if not list_open() then
    return
  end
  local chosen = menu.rows[menu.selected]
  local key = chosen and chosen.key
  menu.rows = list_rows(minimizing)
  menu.selected = first_entry(menu.rows)
  for index, row in ipairs(menu.rows) do
    if key ~= nil and row.key == key then
      menu.selected = index
    end
  end
  if not menu.restore then
    local width, height = list_size(menu.rows)
    local info = gband.win.info(menu.win)
    if width ~= info.width or height ~= info.height then
      gband.win.set_config(menu.win, { width = width, height = height })
    end
  end
  draw()
end

local function minimize(id)
  gband.window.minimize(id)
  rebuild(id)
end
```

While the list is open, layout and focus events rebuild its rows, keeping the selected entry when it is still there, and resizing the list unless its height is maximized.

```lua defaults/lua/gband/desktop.lua
local function window_menu(id, at)
  local layout = gband.layout()
  local box = find_box(layout, id)
  local maximized = state_of(box, layout.cols, layout.rows) == "maximized"
  local rows = {
    { text = "Close", act = function() gband.action.close_window({ window = id }) end },
    {
      text = maximized and "Restore" or "Maximize",
      act = function()
        if maximized then
          restore(id)
        else
          enter_state(id, "maximized")
        end
      end,
    },
    { text = "Minimize", act = function() minimize(id) end },
    { text = "Tile left", act = function() enter_state(id, "left") end },
    { text = "Tile right", act = function() enter_state(id, "right") end },
  }
  local cols, height = ribbon()
  local spec = {
    kind = "window",
    window = id,
    title = box.name,
    rows = rows,
    width = math.min(cols, math.max(MENU_MIN_WIDTH, widest(rows) + 4, gband.ui.width(box.name) + 2)),
    height = math.min(MENU_HEIGHT, height),
  }
  function spec.buttons()
    return {}
  end
  function spec.hint()
    return {}
  end
  function spec.press() end
  open_menu(spec, at)
end
```

The window menu holds five entries for one window, the second chosen by the window's state when it opens.

```lua defaults/lua/gband/desktop.lua
local function pointer(event)
  local col = event.col
  for _, bar in ipairs(gband.bar.list()) do
    if bar.shown and bar.side == "left" then
      col = col - bar.width
    end
  end
  return { col = col, row = event.row }
end

local function program_box(event)
  if event.target ~= "window" or event.content_col ~= nil then
    return nil
  end
  local box = find_box(gband.layout(), event.window)
  if box and box.name then
    return box
  end
  return nil
end
```

The pointer's cell in the ribbon is the press's terminal column less the shown bars on the left.
`program_box` finds the floating window that runs a program whose border is under a press, or nothing.

```lua defaults/lua/gband/desktop.lua
local function corner(x, y, w, h)
  for _, edges in ipairs(CORNERS) do
    local cx = edges[1] == "left" and 0 or w - 1
    local cy = edges[2] == "top" and 0 or h - 1
    local dx = edges[1] == "left" and 1 or -1
    local dy = edges[2] == "top" and 1 or -1
    if (x == cx or x == cx + dx) and y == cy or x == cx and y == cy + dy then
      return { edges[1], edges[2] }
    end
  end
  return nil
end

local function press(event)
  if type(event) ~= "table" then
    return
  end
  pressing = true
  local box = program_box(event)
  if not box then
    return false
  end
  local x, y, w, h = event.box_col, event.box_row, event.box_width, event.box_height
  if y == 0 then
    local layout = gband.layout()
    local button = span_at(window_buttons(box, layout.cols, layout.rows), w, x)
    if button then
      pressed = { window = event.window, button = button }
      return
    end
  end
  local edges = corner(x, y, w, h)
  if edges then
    gband.action.drag_resize_window({ edges = edges })
  elseif y == 0 then
    gband.action.drag_window()
  elseif x == 0 then
    gband.action.drag_resize_window({ edges = { "left" } })
  elseif x == w - 1 then
    gband.action.drag_resize_window({ edges = { "right" } })
  elseif y == h - 1 then
    gband.action.drag_resize_window({ edges = { "bottom" } })
  else
    return false
  end
end
```

`desktop.press`, bound to `leftmouse`.
A registered action bound to a mouse name receives the press, and returning `false` declines it, so every press but one on such a border keeps its interactive default: focus, forward to the program, select.
On a border it checks a button first, then the corner zones, then the top border, which moves the window, then the sides, each with `drag_resize_window` naming exactly the edges to move.
A button press only remembers the window and the button: it starts no gesture and changes no focus.

```lua defaults/lua/gband/desktop.lua
local function released(event)
  local before = pressed
  if event.button ~= "left" or not before then
    return
  end
  pressed = nil
  if event.target ~= "window" or event.window ~= before.window or event.box_row ~= 0 then
    return
  end
  local layout = gband.layout()
  local box = find_box(layout, before.window)
  if not box or span_at(window_buttons(box, layout.cols, layout.rows), event.box_width, event.box_col) ~= before.button then
    return
  end
  if before.button == 1 then
    minimize(before.window)
  elseif before.button == 2 then
    toggle_maximized(before.window)
  else
    gband.action.close_window({ window = before.window })
  end
end
```

The `MouseReleased` handler acts on the remembered button only when the release lies on the same button of the same window, so a press on `[X]` can still be cancelled by moving away.

```lua defaults/lua/gband/desktop.lua
local function open_menu_at(event)
  if type(event) ~= "table" then
    return
  end
  pressing = true
  local box = program_box(event)
  if box then
    window_menu(box.id, pointer(event))
  elseif event.target == "ribbon" then
    show_list(pointer(event))
  else
    return false
  end
end

local function dismiss(event)
  pressing = false
  if not menu_open() then
    return
  end
  if menu.skip then
    menu.skip = false
    return
  end
  if event.target == "plugin_window" and event.plugin_window == menu.win then
    return
  end
  close_menu()
end
```

`desktop.menu`, bound to `rightmouse`, opens the window menu on a border or the list on empty ribbon, and declines the rest, so a right click in content still pastes.
`dismiss` is the `MousePressed` handler: it runs for every press in every table, after the press is handled, and closes the menu unless the press is on the menu or is the one that opened it.
The press itself still takes its effect, so a click on a window both closes the list and focuses the window.

```lua defaults/lua/gband/desktop.lua
local function leader()
  if list_open() and gband.view().plugin_window == menu.win then
    close_menu()
    gband.keymap.run("prefix", "prefix")
    return
  end
  show_list()
end
```

The leader opens the list, or, pressed while the list has focus, closes it and runs the prefix key's own binding, `send_prefix` with the floating preset, so a program can still receive Ctrl+Space.

```lua defaults/lua/gband/desktop.lua
local function float_tiled(layout)
  for _, band in ipairs(layout.bands) do
    for _, column in ipairs(band.columns) do
      for _, window in ipairs(column.windows) do
        if window.name then
          gband.action.toggle_window_floating({ window = window.id, floating = true })
          pending[window.id] = true
        end
      end
    end
  end
end

local function sweep()
  remembered = {}
  local layout = gband.layout()
  float_tiled(layout)
  remember_layout(layout)
end
```

The sweep floats every tiled window that runs a program, at attach and after each reload, and names them for the cascade.
A reload also forgets the boxes to restore.

```lua defaults/lua/gband/desktop.lua
local function tiled_unowned(layout, id)
  for _, band in ipairs(layout.bands) do
    for _, column in ipairs(band.columns) do
      for _, window in ipairs(column.windows) do
        if window.id == id then
          return window.plugin_window == nil
        end
      end
    end
  end
  return false
end

local function tile_awaited()
  for _, id in ipairs(gband.win.list()) do
    local info = gband.win.info(id)
    if info.kind == "tiled" and info.window == nil then
      return true
    end
  end
  return false
end

local function opened(event)
  if tiled_unowned(gband.layout(), event.window) and not tile_awaited() then
    gband.action.toggle_window_floating({ window = event.window, floating = true })
  end
  pending[event.window] = true
  rebuild()
end
```

`WindowOpened` cannot use `name`: the server sends a window's name after the layout that adds the window, and the event runs while the client applies that layout.
So it floats any tiled window that is not this client's own tiled plugin window: one the layout marks with `plugin_window`, or any window while one of this client's tiled plugin windows has no window yet, since the client may learn that mapping after the layout too.
`toggle_window_floating` with `floating = true` floats once on the server, so every floating-style client can send it.

```lua defaults/lua/gband/desktop.lua
local function closed(event)
  remembered[event.window] = nil
  pending[event.window] = nil
  if menu_open() and menu.kind == "window" and menu.window == event.window then
    close_menu()
  end
  rebuild()
end
```

```lua defaults/lua/gband/desktop.lua
local function cascade(layout)
  for _, band in ipairs(layout.bands) do
    local taken = {}
    for _, box in ipairs(band.floating) do
      local cell = box.col .. ":" .. box.row
      taken[cell] = (taken[cell] or 0) + 1
    end
    for _, box in ipairs(band.floating) do
      if pending[box.id] then
        pending[box.id] = nil
        local from = box.col .. ":" .. box.row
        local width = box_width(box, layout.cols)
        local k = 1
        while taken[from] > 1 do
          local col, row = box.col + 2 * k, box.row + k
          if col + width > layout.cols or row + box.rows > layout.rows then
            break
          end
          local cell = col .. ":" .. row
          if (taken[cell] or 0) == 0 then
            gband.window.set_position(box.id, { col = col, row = row })
            taken[from] = taken[from] - 1
            taken[cell] = 1
            break
          end
          k = k + 1
        end
      end
    end
  end
end
```

The cascade places each named window once, in the first layout where it floats: when another floating window of its band has the same top-left cell, it moves two columns right and one row down, again until the cell is free, while the box fits in the screen area.
It counts the cells it has just dispatched, so the windows of one layout spread out.

```lua defaults/lua/gband/desktop.lua
local function refit(layout)
  local fresh = states_of(layout)
  if size and (size.cols ~= layout.cols or size.rows ~= layout.rows) then
    for _, band in ipairs(layout.bands) do
      for _, box in ipairs(band.floating) do
        local state = states[box.id]
        if state then
          apply_state(box.id, state, layout.cols, layout.rows)
          fresh[box.id] = state
        end
      end
    end
  end
  size = { cols = layout.cols, rows = layout.rows }
  states = fresh
end

local function layout_changed()
  local layout = gband.layout()
  refit(layout)
  cascade(layout)
  rebuild()
end
```

When a layout reports another screen area size, each window that was maximized or tiled in the last layout gets the box of that state for the new size, whichever client resized.
Every floating-style client computes the same boxes from the same layout, so their requests agree.

```lua defaults/lua/gband/desktop.lua
return {
  name = "desktop",
  api = 2,
  setup = function(opts)
    if type(opts) ~= "table" then
      error("the options of `desktop` must be a table", 2)
    end
    local field = next(opts)
    if field ~= nil then
      error("`desktop` takes no option `" .. tostring(field) .. "`", 2)
    end
    gband.action.register("desktop.list", function()
      show_list()
    end, { desc = "list the windows" })
    gband.action.register("desktop.leader", leader, {
      desc = "open the window list, or send the prefix key from it",
    })
    gband.action.register("desktop.press", press, {
      desc = "move, resize or press a button of a floating window",
    })
    gband.action.register("desktop.menu", open_menu_at, {
      desc = "open the menu for the cell under the pointer",
    })
    gband.core.provide("decorations", decorations)
    gband.on("Attached", sweep)
    gband.on("ConfigReloaded", sweep)
    gband.on("WindowOpened", opened)
    gband.on("WindowClosed", closed)
    gband.on("LayoutChanged", layout_changed)
    gband.on("FocusChanged", function()
      rebuild()
    end)
    gband.on("MousePressed", dismiss)
    gband.on("MouseReleased", released)
  end,
}
```

`setup` takes no options, registers the four actions, the decorations provider and the handlers.

## Why the presets are written to `defaults/keystyle/`

Every other bundled module is written to `defaults/lua/gband/`, at the path `require` would find it if `defaults/` were on the runtimepath.
The presets are modules too, `gband.keystyle.modal`, `gband.keystyle.direct` and `gband.keystyle.floating`, yet their copies are under `defaults/keystyle/`.

The presets were written there before the API modules were, so that you could read a preset beside `defaults/init.lua` and copy its bindings into your own `user/init.lua`: a preset is a configuration, not a library.
When the API modules gained copies under `defaults/lua/gband/`, the presets kept the place the README and the key style documentation already gave them.
To replace a preset as a module, copy it to `user/lua/gband/keystyle/`, as [00](00-boundary.md) shows.
