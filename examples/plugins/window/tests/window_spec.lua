local t = require("gband.test")

local CONFIG = [[
  gband.keymap.set("prefix", "enter", gband.action.open_window)
  gband.keymap.set("prefix", "h", gband.action.focus_column_left)
  gband.plugin("gband.statusline")
  gband.plugin("gband.statusline.band")
  gband.plugin("window")
]]

local function prompts(g, count)
  g.wait(function(screen)
    local _, found = screen.text():gsub("│%$", "")
    return found == count
  end)
end

t.case("shows the focused window", function(g)
  g.start({ size = "60x4", config = CONFIG })
  t.match(g.screen().row(3), "^window 1 ")
  g.keys("ctrl+space enter")
  g.settle()
  t.match(g.screen().row(3), "^window 2 ")
  g.keys("ctrl+space h")
  g.settle()
  t.match(g.screen().row(3), "^window 1 ")
  prompts(g, 2)
  g.expect_screenshot("two windows")
end)

t.case("links its group to the accent", function(g)
  g.start({ size = "60x4", config = CONFIG })
  local cell = g.screen().cell(3, 7)
  t.eq(cell.char, "1")
  t.eq(cell.fg, "#7aa2f7")
  t.eq(cell.bold, true)
end)

t.case("dusk colors the segment", function(g)
  g.start({ size = "60x4", config = 'gband.colorscheme("dusk")\n' .. CONFIG })
  local cell = g.screen().cell(3, 7)
  t.eq(cell.fg, "#f6c177")
  t.eq(cell.bg, "#232136")
  t.eq(cell.bold, true)
  prompts(g, 1)
  g.expect_screenshot("dusk")
end)
