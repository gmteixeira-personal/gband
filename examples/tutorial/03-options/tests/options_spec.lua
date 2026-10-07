local t = require("gband.test")
local chapter = require("chapter")

local OVERRIDE = [[
gband.opt.mark_letters = "xyz"
]]

t.case("the mark letters default to abc", function(g)
  chapter.start(g, { size = "40x8", env = { SHELL = "/bin/cat" } })
  t.eq(g.client("return gband.opt.mark_letters"), "abc")
  t.eq(g.client([[
    for _, option in ipairs(gband.opt.list()) do
      if option.name == "mark_letters" then
        return option.default, option.type
      end
    end
  ]]), "abc")
  g.keys("ctrl+space m")
  g.settle()
  t.eq(g.notifications(), { { body = "marked window 1 as a" } })
end)

t.case("a value set before the declaration wins", function(g)
  g.start({
    size = "40x8",
    env = { SHELL = "/bin/cat" },
    config = OVERRIDE .. chapter.read("user/init.lua"),
  })
  t.eq(g.client("return gband.opt.mark_letters"), "xyz")
  g.keys("ctrl+space m")
  g.settle()
  t.eq(g.notifications(), { { body = "marked window 1 as x" } })
end)

t.case("the resize steps are a twentieth", function(g)
  chapter.start(g, { size = "40x8", env = { SHELL = "/bin/cat" } })
  t.eq(g.client("return gband.opt.width_step, gband.opt.height_step"), 0.05)
  g.keys("alt+r l escape")
  g.settle()
  t.eq(g.client("return gband.layout().bands[1].columns[1].width"), 0.55)
end)

t.case("animations run twice as fast", function(g)
  chapter.start(g, { size = "40x8", env = { SHELL = "/bin/cat" } })
  t.eq(g.client("return gband.opt.animation_speed"), 2)
  t.eq(g.client("return gband.opt.animations"), true)
end)

t.case("the sidebar follows the settings window", function(g)
  chapter.start(g, { size = "40x8", env = { SHELL = "/bin/cat" } })
  t.match(g.screen().row(0), "^I")
  g.keys("ctrl+space s")
  g.settle()
  t.match(g.screen().text(), "settings")
  g.keys("j enter")
  g.settle()
  t.eq(g.client("return gband.settings.sidebar()"), false)
  g.wait(function(screen)
    return screen.row(0):match("^╭")
  end)
end)
