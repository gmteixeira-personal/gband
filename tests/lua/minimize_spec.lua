local t = require("gband.test")

local config = [[
  gband.bind("f5", function() gband.action.open_window({ floating = true }) end)
  gband.bind("f6", gband.action.minimize_window)
  gband.bind("f7", function()
    gband.window.focus(gband.layout().bands[1].floating[1].id)
  end)
]]

local function floating(g)
  return g.client([[
    local window = gband.layout().bands[1].floating[1]
    return { id = window.id, minimized = window.minimized == true }
  ]])
end

local function prompts(g, count)
  g.wait(function(screen)
    local _, found = screen.text():gsub("│%$", "")
    return found == count
  end)
end

local function open_floating(g)
  local tile = g.client("return gband.view().window")
  g.keys("f5")
  g.settle()
  prompts(g, 2)
  return tile
end

t.case("minimizing hides the floating window", function(g)
  g.start({ config = config })
  prompts(g, 1)
  local tile = open_floating(g)
  t.ok(g.client("return gband.view().floating"))
  g.keys("f6")
  g.settle()
  t.ok(floating(g).minimized)
  t.eq(g.client("return gband.view().window"), tile)
  t.ok(not g.client("return gband.view().floating"))
  t.eq(g.screen().row(2):find("╭"), nil)
  g.expect_screenshot("hidden")
end)

t.case("focus restores a minimized window", function(g)
  g.start({ config = config })
  prompts(g, 1)
  open_floating(g)
  local window = floating(g).id
  g.keys("f6")
  g.settle()
  g.keys("f7")
  g.settle()
  t.ok(not floating(g).minimized)
  t.eq(g.client("return gband.view().window"), window)
  t.ok(g.client("return gband.view().floating"))
  g.expect_screenshot("restored")
end)

t.case("a reload keeps the window minimized", function(g)
  g.start({ config = config })
  prompts(g, 1)
  open_floating(g)
  g.keys("f6")
  g.settle()
  local hidden = g.screen().text()
  t.eq(g.reload(), nil)
  g.settle()
  t.ok(floating(g).minimized)
  t.eq(g.screen().text(), hidden)
end)
