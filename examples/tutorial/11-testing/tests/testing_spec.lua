local t = require("gband.test")
local chapter = require("chapter")

local function start(g)
  chapter.start(g, {
    size = "40x8",
    env = { SHELL = "/bin/cat" },
    files = { "user/colors/dusk.lua" },
    plugins = { "../plugins/marks" },
  })
end

t.case("the configuration sets the plugin up", function(g)
  start(g)
  t.eq(g.client("return #gband.errors()"), 0)
  t.eq(g.client("return gband.plugins()"), { { name = "marks", version = "0.2.0", client = ">= 0.2", failed = false } })
  g.keys("alt+n")
  g.settle()
  g.keys("ctrl+space m h m escape")
  g.settle()
  g.client([[
    gband.rpc("marks.list", nil, function(ok, result)
      listed = result
    end)
  ]])
  g.settle()
  t.eq(g.client("return listed"), { a = 2, b = 1 })
  g.keys("ctrl+space ' a")
  g.settle()
  t.eq(g.client("return gband.view().window"), 2)
  t.match(g.screen().row(0), "^Ia")
  g.expect_screenshot("two windows")
end)
