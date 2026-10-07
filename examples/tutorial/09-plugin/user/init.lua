gband.keystyle.use()

gband.plugin("gband.errors")
if gband.settings.sidebar() ~= false then
  gband.plugin("gband.sidebar")
end

gband.keymap.set("prefix", "e", gband.action["errors.open"], { desc = "list the errors" })

gband.keymap.set("root", "alt+h", gband.action.focus_column_left, { desc = "focus the column to the left" })
gband.keymap.set("root", "alt+l", gband.action.focus_column_right, { desc = "focus the column to the right" })
gband.bind("alt+n", gband.action.open_window)
gband.unbind("prefix D")

gband.set {
  width_step = 1/20,
  height_step = 1/20,
}

gband.keymap.mode("resize", { label = "resize" })
gband.keymap.set("resize", "h", gband.action.shrink_column_width, { desc = "narrower" })
gband.keymap.set("resize", "l", gband.action.grow_column_width, { desc = "wider" })
gband.keymap.set("resize", "escape", function()
  gband.keymap.enter("root")
end, { desc = "done" })
gband.keymap.set("root", "alt+r", function()
  gband.keymap.enter("resize")
end, { desc = "resize" })

gband.keymap.set("prefix", "t", function()
  gband.spawn({ cmd = "top", after = gband.view().window })
end, { desc = "open top beside the window" })
gband.keymap.set("prefix", "H", function()
  gband.open("https://github.com/gmteixeira-personal/gband/blob/main/docs/tutorial/README.md")
end, { desc = "open the tutorial" })
gband.keymap.set("prefix", "g", function()
  gband.band.view(gband.layout().bands[1].id)
end, { desc = "view the first band" })

if gband.plugin("marks", { bar = "left" }) then
  gband.keymap.set("prefix", "m", gband.action["marks.mark"])
  gband.keymap.set("prefix", "y", gband.action["marks.copy"])
  gband.keymap.set("prefix", "X", gband.action["marks.close"])
  gband.keymap.set("prefix", "M", gband.action["marks.list"])
  gband.keymap.set("prefix", "'", gband.action["marks.jump"])
  gband.keymap.set("root", "ctrl+rightmouse", function(event)
    if event.window then
      gband.cmd.run("marks.mark", { window = event.window })
    end
  end, { desc = "mark the window under the pointer" })
end
