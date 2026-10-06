return {
  name = "clock",
  api = 1,
  setup = function(opts)
    if type(opts) ~= "table" then
      error("the options of `clock` must be a table", 2)
    end
    local format = opts.format or "%H:%M"
    if type(format) ~= "string" then
      error("`format` must be a string", 2)
    end
    gband.ui.statusline.add({
      align = opts.align or "bottom",
      priority = opts.priority or 5,
      order = opts.order or 20,
      hl = opts.hl or "StatusLineMuted",
      redraw_interval = opts.interval or 1000,
      render = function()
        return os.date(format)
      end,
    })
  end,
}
