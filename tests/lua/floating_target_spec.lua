local t = require("gband.test")

local function shape(layout)
  local band = layout.bands[1]
  local columns = {}
  for _, column in ipairs(band.columns) do
    local windows = {}
    for _, window in ipairs(column.windows) do
      windows[#windows + 1] = window.id
    end
    columns[#columns + 1] = table.concat(windows, ",")
  end
  local floating = {}
  for _, window in ipairs(band.floating) do
    floating[#floating + 1] = window.id
  end
  return table.concat(columns, "|") .. " floating " .. table.concat(floating, ",")
end

t.case("float twice in one callback", function(g)
  g.start()
  local window = g.client("return gband.view().window")
  g.client([[
    local window = ...
    gband.action.toggle_window_floating({ window = window, floating = true })
    gband.action.toggle_window_floating({ window = window, floating = true })
  ]], window)
  g.settle()
  t.eq(shape(g.client("return gband.layout()")), " floating " .. window)
  g.wait_text("│$")
  g.expect_screenshot("floated")
end)

t.case("tile a tiled window", function(g)
  g.start()
  local before = shape(g.client("return gband.layout()"))
  g.client([[
    gband.action.toggle_window_floating({ window = gband.view().window, floating = false })
  ]])
  g.settle()
  t.eq(shape(g.client("return gband.layout()")), before)
end)

t.case("layout changed on resize", function(g)
  g.start({
    config = [[
      gband.on("LayoutChanged", function()
        local layout = gband.layout()
        record = layout.cols .. "x" .. layout.rows
      end)
    ]],
  })
  g.resize("100x30")
  g.settle()
  t.eq(g.client("return record"), "100x30")
end)

t.case("tiled plugin window beside the last tiled focus", function(g)
  g.start({
    config = [[
      gband.bind("f5", function() gband.action.open_window({ floating = true }) end)
      gband.bind("f6", function() gband.win.open({ kind = "tiled", lines = { "hi" } }) end)
    ]],
  })
  g.keys("f5")
  g.settle()
  t.ok(g.client("return gband.view().floating"))
  g.keys("f6")
  g.settle()
  t.eq(shape(g.client("return gband.layout()")), "1|3 floating 2")
  t.eq(#g.client("return gband.win.list()"), 1)
end)
