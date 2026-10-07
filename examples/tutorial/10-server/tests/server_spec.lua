local t = require("gband.test")
local chapter = require("chapter")

local function start(g, opts)
  opts = opts or {}
  opts.size = "40x8"
  opts.env = { SHELL = "/bin/cat" }
  opts.files = { "user/colors/dusk.lua" }
  opts.plugins = { "../plugins/marks" }
  chapter.start(g, opts)
end

local function mark(g)
  g.keys("ctrl+space m escape")
  g.settle()
end

t.case("a mark is window state in the server", function(g)
  start(g)
  t.eq(g.client("return #gband.errors()"), 0)
  mark(g)
  t.eq(g.server([[return gband.sessions()]]), { "default" })
  t.eq(g.server([=[return gband.window_state("default", 1)["marks.letter"]]=]), "a")
  t.eq(g.client([=[return gband.window_state(1)["marks.letter"]]=]), "a")
  t.eq(g.notifications(), { { body = "marked window 1 as a" } })
  t.match(g.screen().row(0), "^Ia╭")
  g.keys("ctrl+space m escape")
  g.settle()
  t.eq(g.server([=[return gband.window_state("default", 1)["marks.letter"]]=]), nil)
  t.match(g.screen().row(0), "^I ╭")
end)

t.case("the list command returns the session's marks", function(g)
  start(g)
  g.keys("alt+n")
  g.settle()
  mark(g)
  g.client([[
    gband.rpc("marks.list", nil, function(ok, result)
      listed = { ok = ok, result = result }
    end)
  ]])
  g.settle()
  t.eq(g.client("return listed"), { ok = true, result = { a = 2 } })
end)

t.case("a mark set in the server reaches the client", function(g)
  start(g)
  g.server([=[gband.window_state("default", 1)["marks.letter"] = "c"]=])
  g.settle()
  t.match(g.screen().row(0), "^Ic╭")
  g.keys("ctrl+space M")
  g.settle()
  t.match(g.screen().text(), "c window 1 in band 1")
end)

t.case("the exit of a marked window notifies", function(g)
  start(g)
  g.keys("alt+n")
  g.settle()
  mark(g)
  g.keys("ctrl+d")
  g.wait(function()
    return #g.notifications() == 2
  end)
  t.eq(g.notifications()[2], { body = "window 2, mark a, exited" })
end)

t.case("a reload keeps the marks", function(g)
  start(g)
  mark(g)
  t.eq(g.reload(), nil)
  g.settle()
  local found = false
  for _, line in ipairs(g.log("server")) do
    found = found or line:find("default keeps 1 marks", 1, true) ~= nil
  end
  t.ok(found, "the server log counts the marks")
  t.match(g.screen().row(0), "^Ia╭")
end)
