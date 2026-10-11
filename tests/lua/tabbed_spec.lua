local t = require("gband.test")

local NO_BAR = [[gband.keystyle.use()]]

local function act(g, chunk)
  g.client(chunk)
  g.settle()
end

local function tabbed(g, count)
  g.wait_text("$")
  for _ = 2, count do
    act(g, "gband.action.open_window()")
    act(g, "gband.action.consume_or_expel_left()")
  end
  act(g, "gband.action.toggle_column_tabbed()")
  t.eq(g.client("return gband.layout().bands[1].columns[1].tabbed"), true)
end

local function left_edge(g, first, last)
  local screen = g.screen()
  local cells = {}
  for row = first, last do
    cells[#cells + 1] = screen.cell(row, 0).char
  end
  return table.concat(cells)
end

t.case("three tabs", function(g)
  g.start({ config = NO_BAR })
  tabbed(g, 3)
  act(g, "gband.action.focus_window_up()")
  t.eq(g.client("return gband.view().window"), 2)
  g.wait_text("$")
  t.eq(left_edge(g, 0, 23), "╭" .. ("░"):rep(7) .. " " .. ("█"):rep(7) .. " " .. ("░"):rep(6) .. "╰")
  g.expect_screenshot("focused middle tab")
end)

t.case("one tab", function(g)
  g.start({ config = NO_BAR })
  tabbed(g, 1)
  t.eq(left_edge(g, 1, 22), ("█"):rep(22))
  g.expect_screenshot("lone window")
end)

t.case("unfocused tabbed column", function(g)
  g.start({ config = NO_BAR })
  tabbed(g, 2)
  act(g, "gband.action.open_window()")
  g.wait(function(screen)
    local _, prompts = screen.text():gsub("%$", "")
    return prompts == 2
  end)
  t.eq(g.client("return gband.view().window"), 3)
  t.eq(left_edge(g, 0, 23), "╭" .. ("░"):rep(11) .. " " .. ("█"):rep(10) .. "╰")
  t.eq(g.screen().cell(20, 0).fg, g.screen().cell(20, 39).fg)
  g.expect_screenshot("shown tab of another column")
end)

t.case("left side not drawn", function(g)
  g.start({ config = NO_BAR .. "\ngband.opt.tile_border_sides = { 'top', 'bottom' }" })
  tabbed(g, 1)
  g.run("tput cols")
  g.wait_text("38")
  t.eq(left_edge(g, 1, 22), ("█"):rep(22))
  g.expect_screenshot("indicator without the left side")
end)
