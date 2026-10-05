local t = require("gband.test")

local SIZE = "60x4"

local function status(g)
  return g.screen().row(3)
end

local function prompts(g, count)
  g.wait(function(screen)
    local _, found = screen.text():gsub("│%$", "")
    return found == count
  end)
end

t.case("default status line", function(g)
  g.start({ size = SIZE })
  t.eq(status(g), "band 1 │ C-space prefix                                  1/1")
  prompts(g, 1)
  g.expect_screenshot()
end)

t.case("band", function(g)
  g.start({ size = SIZE, config = [[gband.plugin("gband.statusline.band")]] })
  t.eq(status(g), "band 1")
  t.eq(g.screen().cell(3, 0).fg, "#c0caf5")
  prompts(g, 1)
  g.expect_screenshot()
end)

t.case("mode", function(g)
  g.start({
    size = SIZE,
    config = [[
      gband.plugin("gband.statusline.band")
      gband.plugin("gband.statusline.mode")
      gband.keymap.set("prefix", "enter", gband.action.open_pane)
    ]],
  })
  t.eq(status(g), "band 1")
  g.keys("ctrl+space")
  g.settle()
  t.eq(status(g), "band 1 │ prefix")
  prompts(g, 1)
  g.expect_screenshot("prefix")
  g.keys("escape")
  g.settle()
  t.eq(status(g), "band 1")
end)

t.case("hints", function(g)
  g.start({
    size = SIZE,
    config = [[
      gband.keymap.set("prefix", "enter", gband.action.open_pane)
      gband.keymap.set("prefix", "x", gband.action.close_pane)
      gband.keymap.set("prefix", "d", gband.action.detach)
      gband.plugin("gband.statusline.hints", { labels = { close_pane = "kill" } })
    ]],
  })
  t.eq(status(g), "C-space prefix")
  g.keys("ctrl+space")
  g.settle()
  t.eq(status(g), "enter new  x kill  d detach")
  prompts(g, 1)
  g.expect_screenshot("prefix")
end)

t.case("position", function(g)
  g.start({
    size = SIZE,
    config = [[
      gband.keymap.set("prefix", "enter", gband.action.open_pane)
      gband.keymap.set("prefix", "h", gband.action.focus_column_left)
      gband.plugin("gband.statusline.position")
    ]],
  })
  t.match(status(g), "1/1$")
  g.keys("ctrl+space enter")
  g.settle()
  t.match(status(g), "2/2$")
  g.keys("ctrl+space h")
  g.settle()
  t.match(status(g), "1/2$")
  prompts(g, 2)
  g.expect_screenshot("first of two")
end)

t.case("clock", function(g)
  g.start({ size = SIZE, config = [[gband.plugin("gband.statusline.clock")]] })
  t.match(status(g), "12:00$")
  prompts(g, 1)
  g.expect_screenshot()
  g.set_time("2025-01-01 12:05:00")
  g.wait_text("12:05", { timeout = 3 })
end)

t.case("clock format", function(g)
  g.start({
    size = SIZE,
    time = "2025-06-30 23:59:58",
    config = [[gband.plugin("gband.statusline.clock", { format = "%Y-%m-%d %H:%M:%S" })]],
  })
  t.match(status(g), "2025%-06%-30 23:59:58$")
end)
