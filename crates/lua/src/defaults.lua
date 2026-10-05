gband.set {
  prefix = "ctrl+space",
  default_column_width = 1/2,
  width_presets = { 1/3, 1/2, 2/3 },
  center_focused_column = "never",
}

gband.bind("prefix h", gband.action.focus_column_left)
gband.bind("prefix l", gband.action.focus_column_right)
gband.bind("prefix j", gband.action.focus_pane_down)
gband.bind("prefix k", gband.action.focus_pane_up)
gband.bind("prefix u", gband.action.focus_workspace_down)
gband.bind("prefix i", gband.action.focus_workspace_up)
gband.bind("prefix enter", gband.action.open_pane)
gband.bind("prefix q", gband.action.close_pane)
gband.bind("prefix [", gband.action.consume_or_expel_left)
gband.bind("prefix ]", gband.action.consume_or_expel_right)
gband.bind("prefix r", gband.action.cycle_column_width)
gband.bind("prefix f", gband.action.toggle_full_width)
gband.bind("prefix -", gband.action.shrink_column_width)
gband.bind("prefix =", gband.action.grow_column_width)
gband.bind("prefix _", gband.action.shrink_pane_height)
gband.bind("prefix +", gband.action.grow_pane_height)
gband.bind("prefix R", gband.action.reset_pane_height)
gband.bind("prefix D", gband.action.detach)
gband.bind("prefix prefix", gband.action.send_prefix)
