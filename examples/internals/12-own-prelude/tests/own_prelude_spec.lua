local t = require("gband.test")

local function read(path)
  local file = assert(io.open(path), path)
  local text = file:read("a")
  file:close()
  return text
end

local function start(g)
  g.start({
    size = "40x6",
    config = read("user/init.lua"),
    files = {
      ["user/lua/gband/prelude.lua"] = read("user/lua/gband/prelude.lua"),
      ["user/lua/mine/clock.lua"] = read("user/lua/mine/clock.lua"),
    },
    env = { SHELL = "/bin/cat" },
  })
end

t.case("the own prelude adds the clock", function(g)
  start(g)
  t.eq(g.client([[return gband.bar.info("clock").side]]), "right")
  t.match(g.screen().row(0), "12:00$")
  g.expect_screenshot("the clock")
end)

t.case("the client loads the own prelude in place of the bundled one", function(g)
  start(g)
  t.ok(g.client([[return package.loaded["mine.clock"] ~= nil]]), "mine.clock is loaded")
end)

t.case("the clock follows the time", function(g)
  start(g)
  g.set_time("2025-01-01 12:05:00")
  g.wait_text("12:05")
end)
