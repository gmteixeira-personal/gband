local t = require("gband.test")

local SIZE = "60x6"
local LAST = 5

local function bar(g, row)
  local line = g.screen().row(row)
  local cut = utf8.offset(line, 21)
  if cut then
    line = line:sub(1, cut - 1)
  end
  return (line:gsub("%s+$", ""))
end

local function prompts(g, count)
  g.wait(function(screen)
    local _, found = screen.text():gsub("│%$", "")
    return found == count
  end)
end

local function setup(source)
  return 'gband.plugin("gband.statusline")\n' .. (source or "")
end

t.case("default status line", function(g)
  g.start({ size = SIZE })
  t.eq(bar(g, 0), "band 1")
  t.eq(bar(g, 1), "C-space navigation")
  t.eq(bar(g, LAST), "1/1")
  prompts(g, 1)
  g.expect_screenshot()
end)

t.case("band", function(g)
  g.start({ size = SIZE, config = setup([[gband.plugin("gband.statusline.band")]]) })
  t.eq(bar(g, 0), "band 1")
  t.eq(g.screen().cell(0, 0).fg, "#c0caf5")
  prompts(g, 1)
  g.expect_screenshot()
end)

t.case("mode", function(g)
  g.start({
    size = SIZE,
    config = setup([[
      gband.plugin("gband.statusline.band")
      gband.plugin("gband.statusline.mode")
      gband.keymap.set("prefix", "enter", gband.action.open_window)
    ]]),
  })
  t.eq(bar(g, 0), "band 1")
  t.eq(bar(g, 1), "")
  g.keys("ctrl+space")
  g.settle()
  t.eq(bar(g, 1), "prefix")
  prompts(g, 1)
  g.expect_screenshot("prefix")
  g.keys("escape")
  g.settle()
  t.eq(bar(g, 1), "")
end)

t.case("hints", function(g)
  g.start({
    size = SIZE,
    config = setup([[
      gband.keymap.set("prefix", "enter", gband.action.open_window)
      gband.keymap.set("prefix", "x", gband.action.close_window)
      gband.keymap.set("prefix", "d", gband.action.detach)
      gband.plugin("gband.statusline.hints", { labels = { close_window = "kill" } })
    ]]),
  })
  t.eq(bar(g, 0), "C-space prefix")
  g.keys("ctrl+space")
  g.settle()
  t.eq(bar(g, 0), "enter new  x kill")
  t.eq(bar(g, 1), "d detach")
  prompts(g, 1)
  g.expect_screenshot("prefix")
end)

t.case("position", function(g)
  g.start({
    size = SIZE,
    config = setup([[
      gband.keymap.set("prefix", "enter", gband.action.open_window)
      gband.keymap.set("prefix", "h", gband.action.focus_column_left)
      gband.plugin("gband.statusline.position")
    ]]),
  })
  t.eq(bar(g, LAST), "1/1")
  g.keys("ctrl+space enter")
  g.settle()
  t.eq(bar(g, LAST), "2/2")
  g.keys("ctrl+space h")
  g.settle()
  t.eq(bar(g, LAST), "1/2")
  prompts(g, 2)
  g.expect_screenshot("first of two")
end)

t.case("clock", function(g)
  g.start({ size = SIZE, config = setup([[gband.plugin("gband.statusline.clock")]]) })
  t.eq(bar(g, LAST), "12:00")
  prompts(g, 1)
  g.expect_screenshot()
  g.set_time("2025-01-01 12:05:00")
  g.wait_text("12:05", { timeout = 3 })
end)

t.case("clock format", function(g)
  g.start({
    size = SIZE,
    time = "2025-06-30 23:59:58",
    config = setup([[gband.plugin("gband.statusline.clock", { format = "%Y-%m-%d %H:%M:%S" })]]),
  })
  t.eq(bar(g, LAST), "2025-06-30 23:59:58")
end)

t.case("right side", function(g)
  g.start({
    size = SIZE,
    config = [[
      gband.plugin("gband.statusline", { side = "right" })
      gband.plugin("gband.statusline.band")
    ]],
  })
  t.match(g.screen().row(0), "band 1 *$")
  t.match(g.screen().row(0), "^┌")
end)
