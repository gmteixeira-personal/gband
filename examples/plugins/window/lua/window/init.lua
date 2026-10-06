local M = { name = "window", api = 1 }

function M.setup(opts)
  gband.hl.default("WindowSegment", { link = "StatusLineAccent" })
  gband.ui.statusline.add({
    align = opts.align or "bottom",
    priority = opts.priority or 15,
    order = opts.order or 5,
    hl = "WindowSegment",
    redraw_on = { "FocusChanged" },
    render = function(ctx)
      if ctx.window == nil then
        return nil
      end
      return "window " .. ctx.window
    end,
  })
end

return M
