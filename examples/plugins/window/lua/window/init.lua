local M = { name = "window", api = 1 }

function M.setup(opts)
  gband.hl.default("WindowSegment", { link = "SidebarMode" })

  local bar
  local function draw(window)
    if window == nil then
      gband.bar.set_lines(bar, {})
      return
    end
    gband.bar.set_lines(bar, { { { text = "window " .. window, hl = "WindowSegment" } } })
  end

  bar = gband.bar.add({
    side = "right",
    size = 12,
    order = opts.order or 0,
    on_resize = function()
      draw(gband.view().window)
    end,
  })
  gband.on("Attached", function()
    draw(gband.view().window)
  end)
  gband.on("FocusChanged", function(ev)
    draw(ev.window)
  end)
end

return M
