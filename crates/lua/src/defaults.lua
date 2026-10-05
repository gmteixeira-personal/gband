gband.opt.prefix = "ctrl+space"
gband.opt.center_focused_column = "never"
gband.opt.statusline_position = "bottom"
gband.opt.statusline_height = 1
gband.opt.statusline_separator = " │ "
gband.opt.notify_style = "osc9"

local set = gband.keymap.set
local action = gband.action

set("prefix", "h", action.focus_column_left, { desc = "focus the column to the left" })
set("prefix", "l", action.focus_column_right, { desc = "focus the column to the right" })
set("prefix", "j", action.focus_pane_down, { desc = "focus the pane below" })
set("prefix", "k", action.focus_pane_up, { desc = "focus the pane above" })
set("prefix", "u", action.focus_band_down, { desc = "view the band below" })
set("prefix", "i", action.focus_band_up, { desc = "view the band above" })
set("prefix", "enter", action.open_pane, { desc = "open a pane running the user's shell" })
set("prefix", "q", action.close_pane, { desc = "close the pane" })
set("prefix", "[", action.consume_or_expel_left, { desc = "consume or expel the pane to the left" })
set("prefix", "]", action.consume_or_expel_right, { desc = "consume or expel the pane to the right" })
set("prefix", "r", action.cycle_column_width, { desc = "cycle the width of the pane's column" })
set("prefix", "f", action.toggle_full_width, { desc = "toggle full width of the pane's column" })
set("prefix", "-", action.shrink_column_width, { desc = "shrink the width of the pane's column" })
set("prefix", "=", action.grow_column_width, { desc = "grow the width of the pane's column" })
set("prefix", "_", action.shrink_pane_height, { desc = "shrink the height of the pane" })
set("prefix", "+", action.grow_pane_height, { desc = "grow the height of the pane" })
set("prefix", "R", action.reset_pane_height, { desc = "reset the height of the pane" })
set("prefix", "D", action.detach, { desc = "detach" })
set("prefix", "prefix", action.send_prefix, { desc = "send the prefix key to the focused pane" })

gband.plugin("gband.statusline.band")
gband.plugin("gband.statusline.mode")
gband.plugin("gband.statusline.hints")
gband.plugin("gband.statusline.position")
