local t = require("gband.test")

local function labels(screen)
  local found = {}
  for label in screen.row(1):gmatch("W(%d)") do
    found[#found + 1] = label
  end
  return table.concat(found, ",")
end

local function shows(g, expected)
  g.settle()
  t.eq(labels(g.screen()), expected)
end

local function three_windows(g, config)
  g.start({ config = config })
  for index = 1, 3 do
    if index > 1 then
      g.keys("ctrl+space enter")
      g.settle()
    end
    g.run("clear; printf W" .. index)
    g.wait_text("W" .. index)
  end
end

t.case("focus right goes round the strip without end", function(g)
  three_windows(g)
  shows(g, "3,1")
  for _, expected in ipairs({ "3,1", "1,2", "2,3", "3,1", "1,2", "2,3", "3,1" }) do
    g.keys("ctrl+space l")
    shows(g, expected)
  end
end)

t.case("focus left goes round the strip without end", function(g)
  three_windows(g)
  g.keys("ctrl+space l")
  shows(g, "3,1")
  for _, expected in ipairs({ "3,1", "2,3", "1,2", "3,1", "2,3", "1,2", "3,1" }) do
    g.keys("ctrl+space h")
    shows(g, expected)
  end
end)

t.case("looping off stops focus at the last column", function(g)
  three_windows(g, [[
    gband.set { loop_bands = false }
    gband.keymap.set("prefix", "enter", gband.action.open_window)
    gband.keymap.set("prefix", "l", gband.action.focus_column_right)
  ]])
  shows(g, "2,3")
  g.keys("ctrl+space l")
  shows(g, "2,3")
end)
