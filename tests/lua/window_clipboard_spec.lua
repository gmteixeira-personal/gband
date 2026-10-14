local t = require("gband.test")

local WRITE_HI = [[printf '\033]52;c;aGk=\007']]

t.case("a window program's write reaches the terminal", function(g)
  g.start()
  g.run(WRITE_HI)
  g.wait(function()
    return #g.clipboard() > 0
  end)
  g.settle()
  t.eq(g.clipboard(), { "hi" })
end)

t.case("the option off writes nothing and still runs handlers", function(g)
  g.start({
    config = [[
gband.set { window_clipboard = false }
gband.on("WindowClipboard", function(event)
  copied = event.window .. " " .. event.text
end)
]],
  })
  g.run(WRITE_HI)
  g.wait(function()
    return g.client("return copied")
  end)
  g.settle()
  t.eq(g.client("return copied"), "1 hi")
  t.eq(g.clipboard(), {})
end)
