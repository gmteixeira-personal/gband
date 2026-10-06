local t = require("gband.test")

local function column(screen)
  local cells = {}
  for row = 0, screen.rows - 1 do
    cells[#cells + 1] = screen.cell(row, 0).char
  end
  return table.concat(cells):gsub("%s+$", "")
end

local function prompts(g, count)
  g.wait(function(screen)
    local _, found = screen.text():gsub("│%$", "")
    return found == count
  end)
end

local function second_band(g)
  g.keys("ctrl+space u n escape")
  g.settle()
  prompts(g, 1)
end

t.case("rows of a new session", function(g)
  g.start({ size = "40x8" })
  prompts(g, 1)
  t.eq(column(g.screen()), "I 12")
  t.ok(g.screen().cell(2, 0).bold)
  t.ok(not g.screen().cell(3, 0).bold)
  g.expect_screenshot("new session")
end)

t.case("terminal background", function(g)
  g.start({ size = "40x8" })
  prompts(g, 1)
  local screen = g.screen()
  for row = 0, screen.rows - 1 do
    local cell = screen.cell(row, 0)
    t.eq(cell.bg, nil)
    if cell.char == " " or cell.char == "" then
      t.eq(cell.fg, nil)
    end
  end
end)

t.case("navigation mode shows its letter", function(g)
  g.start({ size = "40x8" })
  prompts(g, 1)
  g.keys("ctrl+space")
  g.settle()
  t.eq(g.screen().row(0):sub(1, 1), "N")
  g.expect_screenshot("navigation")
  g.keys("escape")
  g.settle()
  t.eq(g.screen().row(0):sub(1, 1), "I")
end)

t.case("a window opened in the empty band adds a band", function(g)
  g.start({ size = "40x8" })
  prompts(g, 1)
  second_band(g)
  t.eq(column(g.screen()), "I 123")
  g.expect_screenshot("three bands")
end)

t.case("the viewed band moves down", function(g)
  g.start({ size = "40x8" })
  prompts(g, 1)
  second_band(g)
  g.keys("ctrl+space i escape")
  g.settle()
  t.ok(g.screen().cell(2, 0).bold)
  g.keys("ctrl+space u escape")
  g.settle()
  local screen = g.screen()
  t.ok(screen.cell(3, 0).bold)
  t.ok(not screen.cell(2, 0).bold)
  t.ok(not screen.cell(4, 0).bold)
  g.expect_screenshot("second band viewed")
end)

t.case("click a band label", function(g)
  g.start({ size = "40x8" })
  prompts(g, 1)
  second_band(g)
  t.ok(g.screen().cell(3, 0).bold)
  g.mouse("press", "left", 0, 2)
  g.mouse("release", "left", 0, 2)
  g.settle()
  local screen = g.screen()
  t.ok(screen.cell(2, 0).bold)
  t.ok(not screen.cell(3, 0).bold)
  t.eq(g.client("return gband.view().band"), g.client("return gband.layout().bands[1].id"))
end)
