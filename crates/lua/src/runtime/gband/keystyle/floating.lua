gband.plugin("gband.keylist")
gband.plugin("gband.prompt")
gband.plugin("gband.desktop")

local set = gband.keymap.set
local action = gband.action

set("prefix", "n", function()
  action.open_window({ floating = true })
end, { desc = "open a floating window" })
set("prefix", "?", action["keylist.open"], { desc = "list the keys" })
set("prefix", ":", action["prompt.open"], { desc = "run Lua" })
set("prefix", "N", action["prompt.rename"], { desc = "rename the window" })
set("prefix", "s", gband.settings.open, { desc = "settings" })
set("prefix", "!", action.reload, { desc = "reload the configuration" })
set("prefix", "D", action.detach, { desc = "detach" })
set("prefix", "prefix", action.send_prefix, { desc = "send the prefix key to the focused window" })

set("root", "mod+leftmouse", action.drag_window, { desc = "move the window with the mouse" })
set("root", "mod+rightmouse", action.drag_resize_window, { desc = "resize the window with the mouse" })
set("root", "mod+middlemouse", action.drag_band, { desc = "slide the band or switch bands with the mouse" })
set("root", "mod+wheeldown", action.focus_band_down, { desc = "view the band below" })
set("root", "mod+wheelup", action.focus_band_up, { desc = "view the band above" })
set("root", "leftmouse", action["desktop.press"], {
  desc = "move, resize or press a button of a floating window",
})
set("root", "rightmouse", action["desktop.menu"], { desc = "open the menu for the cell under the pointer" })

gband.on("KeyTableChanged", function(event)
  if event.table == "prefix" then
    gband.keymap.enter("root")
    action["desktop.leader"]()
  end
end)
