local t = require("gband.test")

local function config(opts)
  return string.format(
    [[
      gband.plugin("hello", %s)
      gband.keymap.set("prefix", "g", gband.action["hello.greet"])
      gband.keymap.set("prefix", "enter", gband.action.open_window)
    ]],
    opts or "nil"
  )
end

local function wait_logged(g, text)
  g.wait(function()
    for _, line in ipairs(g.log("client")) do
      if line:find(text, 1, true) then
        return true
      end
    end
  end)
end

t.case("greet opens a window that prints the greeting", function(g)
  g.start({
    size = "60x6",
    env = { SHELL = "/bin/cat" },
    config = config([[{ greeting = "hi there" }]]),
  })
  g.keys("ctrl+space g")
  g.settle()
  t.eq(g.client("return #gband.layout().bands[1].columns"), 2)
  g.wait_text("hi there")
  g.expect_screenshot()
end)

t.case("say logs the greeting", function(g)
  g.start({ config = config() })
  t.eq(g.client([[return gband.cmd.run("hello.say", { who = "tests" })]]), true)
  wait_logged(g, "hello, tests")
end)

t.case("focus changes are printed", function(g)
  g.start({ config = config() })
  g.keys("ctrl+space enter")
  g.settle()
  wait_logged(g, "focused window\t2")
end)

t.case("the greeting is an option", function(g)
  g.start({ config = config('{ greeting = "howdy" }') })
  t.eq(g.client([=[return gband.opt["hello.greeting"]]=]), "howdy")
end)
