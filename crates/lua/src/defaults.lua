gband.opt.prefix = "ctrl+space"
gband.opt.center_focused_column = "never"
gband.opt.loop_bands = true
gband.opt.window_titles = true
gband.opt.notify_style = "osc9"
gband.opt.tile_border_sides = { "top", "right", "bottom", "left" }
gband.opt.tile_border_chars = "rounded"
gband.opt.focused_tile_border_chars = "rounded"
gband.opt.floating_border_sides = { "top", "right", "bottom", "left" }
gband.opt.floating_border_chars = "rounded"
gband.opt.focused_floating_border_chars = "rounded"
gband.opt.width_step = 1/10
gband.opt.height_step = 1/10
gband.opt.mouse_mod = "alt"

gband.keystyle.use()

gband.plugin("gband.errors")
if gband.settings.sidebar() ~= false then
  gband.plugin("gband.sidebar")
end

gband.on("Attached", function()
  local settings = gband.settings
  if gband.config_dir and settings.theme() == nil and settings.sidebar() == nil and gband.keystyle.saved() == nil then
    settings.open()
  end
end)
