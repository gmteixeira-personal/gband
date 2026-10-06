gband.opt.prefix = "ctrl+space"
gband.opt.center_focused_column = "never"
gband.opt.loop_bands = true
gband.opt.notify_style = "osc9"
gband.opt.tile_border_sides = { "top", "right", "bottom", "left" }
gband.opt.tile_border_chars = "plain"
gband.opt.floating_border_sides = { "top", "right", "bottom", "left" }
gband.opt.floating_border_chars = "plain"
gband.opt.width_step = 1/10
gband.opt.height_step = 1/10

gband.keystyle.use()

gband.plugin("gband.errors")
gband.plugin("gband.sidebar")

gband.on("Attached", function()
  if gband.config_dir and not gband.keystyle.saved() then
    gband.keystyle.choose()
  end
end)
