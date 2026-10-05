local M = { name = "pane", api = 1 }

function M.setup(opts)
  gband.hl.default("PaneSegment", { link = "StatusLineAccent" })
  gband.ui.statusline.add({
    align = opts.align or "right",
    priority = opts.priority or 15,
    order = opts.order or 5,
    hl = "PaneSegment",
    redraw_on = { "FocusChanged" },
    render = function(ctx)
      if ctx.pane == nil then
        return nil
      end
      return "pane " .. ctx.pane
    end,
  })
end

return M
