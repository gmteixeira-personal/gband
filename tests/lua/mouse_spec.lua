local t = require("gband.test")

local NAVIGATION = [[
  gband.keymap.mode("prefix")
  gband.keymap.set("prefix", "n", function() gband.action.open_window() end)
  gband.keymap.set("prefix", "leftmouse", gband.action.drag_window)
  gband.keymap.set("prefix", "rightmouse", gband.action.drag_resize_window)
]]

local function row_of(g, pattern)
  local screen = g.screen()
  for row = 0, screen.rows - 1 do
    if screen.row(row):find(pattern) then
      return row
    end
  end
  error("no row matches " .. pattern)
end

local function drag(g, from_col, from_row, to_col, to_row)
  g.mouse("press", "left", from_col, from_row)
  g.mouse("drag", "left", to_col, to_row)
  g.mouse("release", "left", to_col, to_row)
  g.settle()
end

local function inverse_cells(g, row)
  local screen = g.screen()
  local found = {}
  for col = 0, screen.cols - 1 do
    if screen.cell(row, col).inverse then
      found[#found + 1] = col
    end
  end
  return found
end

local function printed(g, text)
  g.run("printf '" .. text .. "\\n'")
  g.wait(function(screen)
    return screen.text():find("\n│" .. text:gsub("\\n", ".*\n│"):gsub("%-", "%%-"))
  end)
  g.settle()
end

t.case("select across rows", function(g)
  g.start({ size = "40x8", config = "" })
  printed(g, "hello world\\nsecond line")
  local row = row_of(g, "^│hello world")
  drag(g, 1 + 6, row, 1 + 5, row + 1)
  t.eq(g.clipboard(), { "world\nsecond" })
  t.eq(#inverse_cells(g, row), 12)
  t.eq(inverse_cells(g, row + 1), { 1, 2, 3, 4, 5, 6 })
  g.expect_screenshot("selection")
end)

t.case("drag back before the anchor", function(g)
  g.start({ size = "40x8", config = "" })
  printed(g, "abcdef")
  local row = row_of(g, "^│abcdef")
  drag(g, 1 + 4, row, 1 + 1, row)
  t.eq(g.clipboard(), { "bcde" })
end)

t.case("click copies nothing", function(g)
  g.start({ size = "40x8", config = "" })
  printed(g, "abcdef")
  local row = row_of(g, "^│abcdef")
  drag(g, 1, row, 1 + 1, row)
  t.eq(g.clipboard(), { "ab" })
  g.mouse("press", "left", 1 + 3, row)
  g.mouse("release", "left", 1 + 3, row)
  g.settle()
  t.eq(g.clipboard(), { "ab" })
  t.eq(inverse_cells(g, row), {})
end)

t.case("typing clears the selection", function(g)
  g.start({ size = "40x8", config = "" })
  printed(g, "abcdef")
  local row = row_of(g, "^│abcdef")
  drag(g, 1, row, 1 + 3, row)
  t.eq(#inverse_cells(g, row), 4)
  g.type("x")
  g.settle()
  t.eq(inverse_cells(g, row), {})
end)

t.case("lifted tile and its drop place", function(g)
  g.start({ size = "80x12", config = NAVIGATION })
  g.keys("ctrl+space n")
  g.settle()
  g.keys("ctrl+space")
  g.settle()
  local first = g.client("return gband.layout().bands[1].columns[1].windows[1].id")
  g.mouse("press", "left", 10, 5)
  g.mouse("drag", "left", 70, 6)
  g.settle()
  g.expect_screenshot("lifted")
  g.mouse("release", "left", 70, 6)
  g.settle()
  t.eq(g.client("return gband.layout().bands[1].columns[2].windows[1].id"), first)
  t.eq(g.client("return gband.view().window"), first)
end)

t.case("drag a floating plugin window", function(g)
  g.start({ size = "80x24", config = NAVIGATION })
  local win = g.client("return gband.win.open({ width = 20, height = 10, col = 5, row = 3 })")
  g.keys("ctrl+space")
  g.settle()
  drag(g, 7, 5, 11, 7)
  local info = g.client("return gband.win.info(...)", win)
  t.eq({ info.col, info.row }, { 9, 5 })
end)

t.case("resize a floating plugin window", function(g)
  g.start({ size = "80x24", config = NAVIGATION })
  local win = g.client([[
    sizes = {}
    return gband.win.open({ width = 30, height = 12, col = 10, row = 3,
      on_resize = function(_, cols, rows) sizes[#sizes + 1] = cols .. "x" .. rows end })
  ]])
  g.keys("ctrl+space")
  g.settle()
  g.mouse("press", "right", 38, 13)
  g.mouse("drag", "right", 42, 15)
  g.mouse("drag", "right", 44, 15)
  g.mouse("release", "right", 44, 15)
  g.settle()
  local info = g.client("return gband.win.info(...)", win)
  t.eq({ info.col, info.row, info.width, info.height }, { 10, 3, 36, 14 })
  t.eq(g.client("return sizes"), { "32x12", "34x12" })
end)

t.case("click a plugin window line", function(g)
  g.start({ size = "80x24", config = "" })
  local win = g.client([[
    local lines = {}
    for n = 1, 10 do lines[n] = "line " .. n end
    return gband.win.open({ height = 12, col = 5, row = 3, focus = false, cursorline = true, lines = lines })
  ]])
  g.mouse("press", "left", 8, 3 + 1 + 3)
  g.mouse("release", "left", 8, 3 + 1 + 3)
  g.settle()
  local info = g.client("return gband.win.info(...)", win)
  t.eq(info.cursor, 4)
  t.ok(info.focused, "the plugin window is focused")
end)

t.case("wheel scrolls a plugin window", function(g)
  g.start({ size = "80x24", config = "" })
  local win = g.client([[
    local lines = {}
    for n = 1, 25 do lines[n] = "line " .. n end
    return gband.win.open({ height = 12, col = 5, row = 3, focus = false, lines = lines })
  ]])
  g.mouse("scroll", "down", 8, 6)
  g.settle()
  local info = g.client("return gband.win.info(...)", win)
  t.eq(info.top, 2)
  t.ok(not info.focused, "focus does not change")
end)

t.case("mouse events reach handlers", function(g)
  g.start({
    size = "80x24",
    config = [[
      seen = {}
      gband.on("MousePressed", function(e)
        seen[#seen + 1] = e.button .. " " .. e.target .. " " .. tostring(e.content_col) .. "," .. tostring(e.content_row)
      end)
    ]],
  })
  g.mouse("press", "left", 5, 3)
  g.mouse("release", "left", 5, 3)
  g.settle()
  t.eq(g.client("return seen"), { "left window 4,2" })
end)
