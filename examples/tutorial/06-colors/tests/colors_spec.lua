local t = require("gband.test")
local chapter = require("chapter")

local function start(g, opts)
  opts.size = "40x8"
  opts.env = { SHELL = "/bin/cat" }
  opts.files = { "user/colors/dusk.lua" }
  chapter.start(g, opts)
end

t.case("Mark defaults to bold yellow", function(g)
  start(g, {})
  t.eq(g.client([[return gband.hl.get("Mark")]]), { fg = "yellow", bold = true })
  t.eq(g.client([[return gband.hl.get("Mark", { resolve = true })]]), { fg = "yellow", bold = true })
  t.eq(g.client([[return gband.colorscheme()]]), "terminal")
end)

t.case("dusk sets Mark and the palette", function(g)
  start(g, { theme = "dusk", window_titles = true })
  t.eq(g.client([[return gband.colorscheme()]]), "dusk")
  t.eq(g.client([[return gband.hl.get("Mark", { resolve = true })]]), { fg = "#f6c177", bold = true })
  t.eq(g.client([[return gband.palette.get().bg]]), "#232136")
  g.keys("ctrl+space m escape")
  g.settle()
  t.match(g.screen().row(0), "mark a")
  g.expect_screenshot()
end)

t.case("the prompt switches the colorscheme", function(g)
  start(g, {})
  local themes = g.client([[return gband.settings.themes()]])
  t.eq(themes[#themes], "dusk")
  g.keys("ctrl+space :")
  g.run([[gband.colorscheme("dusk")]])
  g.settle()
  t.eq(g.client([[return gband.colorscheme()]]), "dusk")
  g.keys("ctrl+space :")
  g.run([[print(gband.palette.get().bg)]])
  g.settle()
  local found = false
  for _, line in ipairs(g.log("client")) do
    found = found or line:find("#232136", 1, true) ~= nil
  end
  t.ok(found, "the client log holds the background")
end)
