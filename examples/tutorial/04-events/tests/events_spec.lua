local t = require("gband.test")
local chapter = require("chapter")

local function count(g, text)
  local found = 0
  for _, line in ipairs(g.log("client")) do
    if line:find(text, 1, true) then
      found = found + 1
    end
  end
  return found
end

local function start(g)
  chapter.start(g, { size = "40x8", env = { SHELL = "/bin/cat" } })
end

t.case("a mark raises marks.changed", function(g)
  start(g)
  g.keys("ctrl+space m")
  g.settle()
  t.eq(count(g, "marks changed\t1\ta"), 1)
  g.keys("m")
  g.settle()
  t.eq(count(g, "marks changed\t1\tnil"), 1)
end)

t.case("closing a marked window forgets its mark", function(g)
  start(g)
  g.keys("alt+n")
  g.settle()
  g.keys("ctrl+space m q")
  g.settle()
  t.eq(count(g, "marks changed\t2\tnil"), 1)
  g.keys("m")
  g.settle()
  t.eq(count(g, "marks changed\t1\ta"), 1)
end)

t.case("clearing the group stops the handlers", function(g)
  start(g)
  g.client([[gband.augroup("marks")]])
  g.keys("ctrl+space m")
  g.settle()
  t.eq(g.notifications(), { { body = "marked window 1 as a" } })
  t.eq(count(g, "marks changed"), 0)
end)
