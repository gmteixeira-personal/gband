local t = require("gband.test")

local function agent(g, window)
  return g.client("local state = gband.window_state(...) return state and state.agent", window)
end

local function bar_text(g)
  local screen = g.screen()
  local chars = {}
  for col = screen.cols - 3, screen.cols - 1 do
    chars[#chars + 1] = screen.cell(0, col).char
  end
  return table.concat(chars)
end

local function waits(g, window)
  g.wait(function()
    return agent(g, window) == "waiting"
  end)
end

t.case("a prompt marks the window waiting", function(g)
  g.start()
  g.wait_text("$")
  g.run("printf 'Do you want to %s?\\n' proceed")
  waits(g, 1)
  g.settle()
  t.match(bar_text(g), "^1%s*$")
  t.eq(g.notifications(), { { body = "Agent waiting in window 1" } })
  g.wait(function(screen)
    local _, prompts = screen.text():gsub("│%$", "")
    return prompts == 2
  end)
  g.expect_screenshot({ styles = false })
end)

t.case("typing clears the waiting state", function(g)
  g.start({ size = "80x6" })
  g.wait_text("$")
  g.run("printf 'Do you want to %s?\\n' proceed")
  waits(g, 1)
  g.type("x")
  g.wait(function()
    return agent(g, 1) == nil
  end)
  g.settle()
  t.ok(not bar_text(g):find("%d"), "the bar is empty")
end)

t.case("closing a waiting window drops it", function(g)
  g.start({ size = "80x6" })
  g.keys("ctrl+space n enter")
  g.settle()
  g.server([[
    gband.window_state("default", 1).agent = "waiting"
    gband.window_state("default", 2).agent = "waiting"
  ]])
  g.settle()
  t.match(bar_text(g), "^2%s*$")
  g.run("exit")
  g.wait(function()
    return g.client("return #gband.layout().bands[1].columns") == 1
  end)
  g.settle()
  t.match(bar_text(g), "^1%s*$")
end)

t.case("a reload counts the windows already waiting", function(g)
  g.start({ size = "80x6" })
  g.server([[gband.window_state("default", 1).agent = "waiting"]])
  g.settle()
  t.eq(g.reload(), nil)
  g.settle()
  t.match(bar_text(g), "^1%s*$")
end)

t.case("prefix a focuses the next waiting window", function(g)
  g.start({ size = "80x6" })
  g.keys("ctrl+space n enter")
  g.settle()
  t.eq(g.client("return gband.view().window"), 2)
  g.server([[gband.window_state("default", 1).agent = "waiting"]])
  g.settle()
  g.keys("ctrl+space a")
  g.settle()
  t.eq(g.client("return gband.view().window"), 1)
end)
