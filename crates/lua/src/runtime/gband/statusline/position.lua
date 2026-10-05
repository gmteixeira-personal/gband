return {
  name = "position",
  api = 1,
  setup = function(opts)
    if type(opts) ~= "table" then
      error("the options of `position` must be a table", 2)
    end
    gband.ui.statusline.add({
      align = opts.align or "right",
      priority = opts.priority or 10,
      order = opts.order or 10,
      hl = opts.hl or "StatusLineMuted",
      redraw_on = { "FocusChanged", "BandChanged", "LayoutChanged" },
      render = function(ctx)
        if ctx.column == nil then
          return nil
        end
        return ctx.column.index .. "/" .. ctx.column.count
      end,
    })
  end,
}
