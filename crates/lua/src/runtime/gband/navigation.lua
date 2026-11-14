local core = gband.core

local function interactive()
  gband.keymap.enter("root")
end

local function bind(table)
  if not core.loading() then
    error("gband.navigation.bind can only be called while the configuration loads", 2)
  end
  if type(table) ~= "string" or table == "" then
    error("gband.navigation.bind expects a key table name as a non-empty string", 2)
  end
  if table == "root" then
    error("gband.navigation.bind cannot bind navigation mode in the root table", 2)
  end

  local function set(key, target, opts)
    gband.keymap.set(table, key, target, opts)
  end
  local action = gband.action
  local interactive_on_new = gband.settings.interactive_on_new() == true

  set("h", action.focus_column_left, { desc = "focus the column to the left" })
  set("l", action.focus_column_right, { desc = "focus the column to the right" })
  set("j", action.focus_window_down, { desc = "focus the window below" })
  set("k", action.focus_window_up, { desc = "focus the window above" })
  set("u", action.focus_band_down, { desc = "view the band below" })
  set("i", action.focus_band_up, { desc = "view the band above" })
  require("gband.jump").setup(table)
  set("c", action.center_column, { desc = "center the focused column" })
  set("n", function()
    action.open_window()
    if interactive_on_new then
      interactive()
    end
  end, { desc = "open a window" })
  set("q", action.close_window, { desc = "close the window" })
  set("[", action.consume_or_expel_left, { desc = "consume or expel the window to the left" })
  set("]", action.consume_or_expel_right, { desc = "consume or expel the window to the right" })
  set("r", action.cycle_column_width, { desc = "cycle the width of the window's column" })
  set("f", action.toggle_full_width, { desc = "toggle full width of the window's column" })
  set("w", action.toggle_column_tabbed, { desc = "toggle tabbed display of the window's column" })
  set("-", action.shrink_column_width, { desc = "shrink the width of the window's column" })
  set("=", action.grow_column_width, { desc = "grow the width of the window's column" })
  set("_", action.shrink_window_height, { desc = "shrink the height of the window" })
  set("+", action.grow_window_height, { desc = "grow the height of the window" })
  set("R", action.reset_window_height, { desc = "reset the height of the window" })
  set("v", action.toggle_window_floating, { desc = "float or tile the window" })
  set("V", action.switch_focus_floating_tiled, { desc = "switch focus between floating and tiled windows" })
  set("ctrl+h", action.move_column_left, { desc = "move the column or floating window to the left" })
  set("ctrl+l", action.move_column_right, { desc = "move the column or floating window to the right" })
  set("ctrl+j", action.move_window_down, { desc = "move the window down" })
  set("ctrl+k", action.move_window_up, { desc = "move the window up" })
  set("ctrl+left", action.move_column_left, { desc = "move the column or floating window to the left" })
  set("ctrl+right", action.move_column_right, { desc = "move the column or floating window to the right" })
  set("ctrl+down", action.move_window_down, { desc = "move the window down" })
  set("ctrl+up", action.move_window_up, { desc = "move the window up" })
  set("?", action["keylist.open"], { desc = "list the keys" })
  set(":", action["prompt.open"], { desc = "run Lua" })
  set("N", action["prompt.rename"], { desc = "rename the window" })
  set("s", gband.settings.open, { desc = "settings" })
  set("!", action.reload, { desc = "reload the configuration" })
  set("m", function()
    action.toggle_multi()
    interactive()
  end, { desc = "multi mode" })
  set("e", function()
    local open = action["errors.open"]
    if open then
      open()
    end
  end, { desc = "list the errors" })
  set("E", function()
    local clear = action["errors.clear"]
    if clear then
      clear()
    end
  end, { desc = "clear the errors" })
  set("p", action.paste_image, { desc = "paste an image from the clipboard" })
  set("D", action.detach, { desc = "detach" })
  set("escape", interactive, { desc = "interactive mode" })
  set("enter", interactive, { desc = "interactive mode" })
  set("left", action.focus_column_left, { desc = "focus the column to the left" })
  set("right", action.focus_column_right, { desc = "focus the column to the right" })
  set("down", action.focus_window_down, { desc = "focus the window below" })
  set("up", action.focus_window_up, { desc = "focus the window above" })
  set("prefix", function()
    action.send_prefix()
    interactive()
  end, { desc = "send the prefix key" })
  set("leftmouse", action.drag_window, { desc = "move the window with the mouse" })
  set("rightmouse", action.drag_resize_window, { desc = "resize the window with the mouse" })
  set("middlemouse", action.drag_band, { desc = "slide the band or switch bands with the mouse" })
  set("mod+leftmouse", action.drag_window, { desc = "move the window with the mouse" })
  set("mod+rightmouse", action.drag_resize_window, { desc = "resize the window with the mouse" })
  set("mod+middlemouse", action.drag_band, { desc = "slide the band or switch bands with the mouse" })
  set("mod+wheeldown", action.focus_band_down, { desc = "view the band below" })
  set("mod+wheelup", action.focus_band_up, { desc = "view the band above" })
end

gband.navigation = {
  bind = bind,
}
return gband.navigation
