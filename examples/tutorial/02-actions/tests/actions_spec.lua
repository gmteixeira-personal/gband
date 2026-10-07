local t = require("gband.test")
local chapter = require("chapter")

local function start(g)
  chapter.start(g, { size = "40x8", env = { SHELL = "/bin/cat" } })
end

t.case("marking notifies", function(g)
  start(g)
  g.keys("ctrl+space m")
  g.settle()
  t.eq(g.notifications(), { { body = "marked window 1 as a" } })
  t.eq(g.client([[return gband.cmd.run("mark", { window = 1 })]]), true)
  g.settle()
  t.eq(g.notifications()[2], { body = "unmarked window 1" })
end)

t.case("a fourth mark rings the bell", function(g)
  start(g)
  g.client([[
    for window = 1, 4 do
      gband.cmd.run("mark", { window = window })
    end
  ]])
  g.settle()
  t.eq(#g.notifications(), 3)
  t.eq(g.bells(), 1)
end)

t.case("copy_marks writes the clipboard", function(g)
  start(g)
  g.keys("alt+n")
  g.settle()
  g.keys("ctrl+space m h m y")
  g.settle()
  t.eq(g.clipboard(), { "a 2\nb 1" })
end)

t.case("close_marked closes the marked windows", function(g)
  start(g)
  g.keys("alt+n")
  g.settle()
  g.keys("ctrl+space m X")
  g.settle()
  t.eq(g.client("return #gband.layout().bands[1].columns"), 1)
  t.eq(g.client("return gband.view().window"), 1)
end)

t.case("prefix H opens the tutorial", function(g)
  start(g)
  g.keys("ctrl+space H")
  g.wait(function()
    return #g.opened() > 0
  end)
  t.eq(g.opened(), { "https://github.com/gmteixeira-personal/gband/blob/main/docs/tutorial/README.md" })
end)

t.case("prefix t opens a window beside the focused one", function(g)
  chapter.start(g, { size = "40x8" })
  g.keys("ctrl+space t")
  g.settle()
  t.eq(g.client("return #gband.layout().bands[1].columns"), 2)
end)
