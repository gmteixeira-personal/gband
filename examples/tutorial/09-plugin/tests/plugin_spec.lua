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

t.case("the plugin marks, lists and shows marks", function(g)
  start(g)
  t.eq(g.client("return #gband.errors()"), 0)
  g.keys("alt+n")
  g.settle()
  g.keys("ctrl+space m h m M")
  g.settle()
  t.match(g.screen().text(), "a window 2 in band 1")
  t.match(g.screen().text(), "b window 1 in band 1")
  g.keys("enter")
  g.settle()
  t.eq(g.client("return gband.view().window"), 2)
  t.match(g.screen().row(0), "^Ia╭")
  g.keys("ctrl+space ' b")
  g.settle()
  t.eq(g.client("return gband.view().window"), 1)
  t.match(g.screen().row(0), "^Ib╭")
end)

t.case("the plugin's names start with marks", function(g)
  start(g)
  local names = g.client([[
    local names = {}
    for _, action in ipairs(gband.action.list()) do
      names[action.name] = true
    end
    for _, command in ipairs(gband.cmd.list()) do
      names["cmd " .. command.name] = true
    end
    for _, option in ipairs(gband.opt.list()) do
      names["opt " .. option.name] = true
    end
    return names
  ]])
  for _, name in ipairs({ "marks.mark", "marks.copy", "marks.close", "marks.list", "marks.jump", "cmd marks.mark", "opt marks.letters" }) do
    t.ok(names[name], name)
  end
  t.eq(g.client([[return gband.bar.info("marks").plugin]]), "marks")
end)

t.case("the plugin is on the runtimepath", function(g)
  start(g)
  t.eq(g.client("return gband.plugins()"), { { name = "marks", version = "0.1.0", failed = false } })
  t.match(g.client("return gband.runtimepath[1]"), "/user$")
  t.match(g.client("return gband.runtimepath[2]"), "/plugins/marks$")
  t.eq(g.client("return gband.side"), "client")
  t.eq(g.client("return require('marks').api"), g.client("return gband.api_version"))
end)

t.case("the letters are an option of the plugin", function(g)
  start(g, { config = [[gband.opt["marks.letters"] = "xyz"]] .. "\n" .. chapter.read("user/init.lua") })
  g.keys("ctrl+space m escape")
  g.settle()
  t.eq(g.notifications(), { { body = "marked window 1 as x" } })
end)

t.case("a wrong option fails the plugin", function(g)
  local config = chapter.read("user/init.lua"):gsub('bar = "left"', 'bar = "top"')
  start(g, { config = config })
  local errors = g.client("return gband.errors()")
  t.eq(#errors, 1)
  t.match(errors[1], '^marks: .*lua/marks/init.lua:%d+: `bar` is "left", "right" or false, found top$')
  t.eq(g.client("return gband.plugins()[1].failed"), true)
  t.eq(g.client([=[return gband.action["marks.mark"]]=]), nil)
  t.eq(g.client([[return #gband.keymap.list("prefix") > 0]]), true)
end)

t.case("the plugin refuses the server", function(g)
  start(g, { server_config = [[gband.plugin("marks")]] })
  g.wait(function()
    return #g.client("return gband.errors()") > 0
  end)
  t.match(g.client("return gband.errors()[1]"), "^server: marks: .*set marks up in user/init.lua$")
end)
