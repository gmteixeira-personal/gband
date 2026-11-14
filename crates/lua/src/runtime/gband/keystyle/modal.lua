gband.plugin("gband.keylist")
gband.plugin("gband.prompt")

local set = gband.keymap.set
local action = gband.action

gband.keymap.mode("prefix", { label = "navigation" })
require("gband.navigation").bind("prefix")

set("root", "mod+leftmouse", action.drag_window, { desc = "move the window with the mouse" })
set("root", "mod+rightmouse", action.drag_resize_window, { desc = "resize the window with the mouse" })
set("root", "mod+middlemouse", action.drag_band, { desc = "slide the band or switch bands with the mouse" })
set("root", "mod+wheeldown", action.focus_band_down, { desc = "view the band below" })
set("root", "mod+wheelup", action.focus_band_up, { desc = "view the band above" })
