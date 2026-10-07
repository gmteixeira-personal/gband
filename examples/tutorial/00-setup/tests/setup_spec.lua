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

local function prompt(g, line)
  g.keys("ctrl+space :")
  g.run(line)
  g.settle()
end

t.case("the configuration loads", function(g)
  chapter.start(g, { size = "40x8", env = { SHELL = "/bin/cat" } })
  t.eq(g.client("return #gband.errors()"), 0)
  t.match(g.screen().row(0), "^I")
  t.match(g.screen().row(2), "^1")
  g.expect_screenshot()
end)

t.case("the prompt prints to the client log", function(g)
  chapter.start(g)
  prompt(g, [[print(gband.config_dir)]])
  local config_dir = g.client("return gband.config_dir")
  t.match(config_dir, "/gband$")
  t.ok(logged(g, config_dir), "the client log holds the configuration directory")
end)

t.case("a broken file keeps the last configuration", function(g)
  chapter.start(g, { size = "40x8", env = { SHELL = "/bin/cat" } })
  g.write("user/init.lua", chapter.read("user/init.lua") .. [[gband.plugn("gband.sidebar")]] .. "\n")
  t.match(g.reload(), "user/init.lua:7: attempt to call a nil value %(field 'plugn'%)")
  g.settle()
  t.eq(g.client("return #gband.errors()"), 1)
  t.match(g.screen().row(7), "^!")
  g.expect_screenshot("the error marker")

  g.keys("ctrl+space e")
  g.settle()
  t.match(g.screen().text(), "errors  c clear")
  g.keys("q")
  g.settle()

  prompt(g, [[gband.clear_errors()]])
  t.eq(g.client("return #gband.errors()"), 0)
  t.match(g.screen().row(7), "^ ")
end)
