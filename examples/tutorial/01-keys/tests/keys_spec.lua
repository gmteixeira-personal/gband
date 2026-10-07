local t = require("gband.test")
local chapter = require("chapter")

local function logged(g, text)
  for _, line in ipairs(g.log("client")) do
    if line:find(text, 1, true) then
      return true
    end
  end
  return false
end

local function start(g)
  chapter.start(g, { size = "40x8", env = { SHELL = "/bin/cat" } })
end

t.case("root bindings open and focus windows", function(g)
  start(g)
  g.keys("alt+n")
  g.settle()
  t.eq(g.client("return #gband.layout().bands[1].columns"), 2)
  t.eq(g.client("return gband.view().window"), 2)
  g.keys("alt+h")
  g.settle()
  t.eq(g.client("return gband.view().window"), 1)
  g.keys("alt+l")
  g.settle()
  t.eq(g.client("return gband.view().window"), 2)
end)

t.case("unbind removes a preset binding", function(g)
  start(g)
  local keys = g.client([[
    local keys = {}
    for _, binding in ipairs(gband.keymap.list("prefix")) do
      keys[binding.key] = true
    end
    return keys
  ]])
  t.ok(keys.e, "prefix e is bound")
  t.ok(keys.n, "the key style binds prefix n")
  t.eq(keys.D, nil)
end)

t.case("the resize mode grows the column", function(g)
  start(g)
  g.keys("alt+r")
  g.settle()
  t.eq(g.client("return gband.keymap.current_table()"), "resize")
  t.eq(g.client("return gband.keymap.label('resize')"), "resize")
  t.match(g.screen().row(0), "^R")
  g.keys("l l")
  g.settle()
  t.eq(g.client("return gband.layout().bands[1].columns[1].width"), 0.7)
  g.keys("escape")
  g.settle()
  t.eq(g.client("return gband.keymap.current_table()"), "root")
end)

t.case("prefix m marks the focused window", function(g)
  start(g)
  g.keys("ctrl+space m")
  g.settle()
  t.ok(logged(g, "marked window 1 as a"), "the client log holds the mark")
  g.keys("m")
  g.settle()
  t.ok(logged(g, "unmarked window 1"), "the client log holds the unmark")
end)

t.case("ctrl and the right button mark the window under the pointer", function(g)
  start(g)
  g.keys("alt+n")
  g.settle()
  g.mouse("press", "right", 30, 3, "ctrl")
  g.mouse("release", "right", 30, 3, "ctrl")
  g.settle()
  t.ok(logged(g, "marked window 2 as a"), "the client log holds the mark")
end)

t.case("ctrl and the left button mark a window by its top border", function(g)
  start(g)
  g.keys("alt+n")
  g.settle()
  g.mouse("press", "left", 30, 0, "ctrl")
  g.mouse("release", "left", 30, 0, "ctrl")
  g.settle()
  t.ok(logged(g, "marked window 2"), "the client log holds the mark")
  g.mouse("press", "left", 8, 3, "ctrl")
  g.mouse("release", "left", 8, 3, "ctrl")
  g.settle()
  t.eq(g.client("return gband.view().window"), 1)
  t.ok(not logged(g, "marked window 1"), "the content click marks nothing")
end)
