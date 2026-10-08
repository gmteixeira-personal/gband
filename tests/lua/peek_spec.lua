local t = require("gband.test")

local config = [[
  gband.bind("f5", function() gband.action.open_window({ floating = true }) end)
  gband.bind("f6", gband.action.minimize_window)
  gband.bind("f7", function()
    gband.window.focus(gband.layout().bands[1].floating[1].id, { peek = true })
  end)
  gband.bind("f8", function()
    local ids, lines = {}, {}
    for _, band in ipairs(gband.layout().bands) do
      for _, column in ipairs(band.columns) do
        for _, window in ipairs(column.windows) do
          ids[#ids + 1] = window.id
        end
      end
      for _, window in ipairs(band.floating) do
        ids[#ids + 1] = window.id
      end
    end
    for index, id in ipairs(ids) do
      lines[index] = "window " .. id
    end
    list = gband.win.open({
      lines = lines,
      width = 20,
      height = 6,
      hover = true,
      on_mouse = function(_, event)
        if event.kind == "move" and event.line and ids[event.line] then
          gband.window.focus(ids[event.line], { peek = true })
        end
      end,
    })
  end)
]]

local function prompts(g, count)
  g.wait(function(screen)
    local _, found = screen.text():gsub("│%$", "")
    return found == count
  end)
end

local function floating(g, index)
  return g.client(
    [[
    local window = gband.layout().bands[1].floating[...]
    return { id = window.id, minimized = window.minimized == true, last_focus = window.last_focus }
  ]],
    index
  )
end

local function minimized_window(g)
  g.start({ config = config })
  prompts(g, 1)
  local tile = g.client("return gband.view().window")
  g.keys("f5")
  g.settle()
  prompts(g, 2)
  g.keys("f6")
  g.settle()
  return tile
end

t.case("a peek shows a minimized window on top", function(g)
  minimized_window(g)
  g.keys("f5")
  g.settle()
  prompts(g, 2)
  local other = floating(g, 2).id
  g.client("gband.window.set_position(..., { col = 30, row = 6 })", other)
  g.settle()
  local before = g.screen().text()
  local first = floating(g, 1)
  t.ok(first.minimized)
  g.keys("f7")
  g.settle()
  local view = g.client("return gband.view()")
  t.eq(view.window, first.id)
  t.ok(view.peek)
  local peeked = floating(g, 1)
  t.ok(peeked.minimized)
  t.eq(peeked.last_focus, first.last_focus)
  g.expect_screenshot("peeked")
  g.client("gband.window.focus(...)", other)
  g.settle()
  t.eq(g.screen().text(), before)
end)

t.case("hovering a line peeks its window", function(g)
  minimized_window(g)
  g.keys("f8")
  g.settle()
  local list = g.client("return list")
  local box = g.client("return gband.win.info(list)")
  local hidden = floating(g, 1).id
  g.mouse("move", nil, box.col + 2, box.row + 2)
  g.settle()
  t.eq(g.client("return gband.view().window"), hidden)
  t.ok(g.client("return gband.win.info(list).focused"))
  t.ok(g.client("return gband.view().peek"))
  g.expect_screenshot("hovered")
end)

t.case("a reload ends the peek", function(g)
  local tile = minimized_window(g)
  local hidden = g.screen().text()
  g.keys("f7")
  g.settle()
  t.ok(g.client("return gband.view().peek"))
  t.eq(g.reload(), nil)
  g.settle()
  t.eq(g.client("return gband.view().peek"), nil)
  t.eq(g.client("return gband.view().window"), tile)
  t.eq(g.screen().text(), hidden)
end)
