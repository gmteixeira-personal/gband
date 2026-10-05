return {
  name = "band",
  api = 1,
  setup = function(opts)
    if type(opts) ~= "table" then
      error("the options of `band` must be a table", 2)
    end
    gband.ui.statusline.add({
      align = opts.align or "left",
      priority = opts.priority or 20,
      order = opts.order or 10,
      hl = opts.hl or "StatusLineSegment",
      redraw_on = { "BandChanged", "LayoutChanged" },
      render = function(ctx)
        return "band " .. ctx.band.index
      end,
    })
  end,
}
