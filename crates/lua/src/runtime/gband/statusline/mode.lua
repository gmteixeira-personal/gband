return {
  name = "mode",
  api = 1,
  setup = function(opts)
    if type(opts) ~= "table" then
      error("the options of `mode` must be a table", 2)
    end
    gband.ui.statusline.add({
      align = opts.align or "left",
      priority = opts.priority or 30,
      order = opts.order or 20,
      hl = opts.hl or "StatusLineAccent",
      redraw_on = { "KeyTableChanged" },
      render = function(ctx)
        if ctx.table == "root" then
          return nil
        end
        return gband.keymap.label(ctx.table)
      end,
    })
  end,
}
