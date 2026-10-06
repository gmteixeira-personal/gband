local t = require("gband.test")

local function agent(g, window)
  return g.client("local state = gband.window_state(...) return state and state.agent", window)
end

local function bar_rows(g)
  local screen = g.screen()
  local found = {}
  for row = 0, screen.rows - 1 do
    local line = screen.row(row)
    local cut = utf8.offset(line, 21)
    found[#found + 1] = cut and line:sub(1, cut - 1) or line
  end
  return table.concat(found, "\n")
end

local function waits(g, window)
  g.wait(function()
    return agent(g, window) == "waiting"
  end)
end

t.case("a prompt marks the window waiting", function(g)
  g.start()
  g.run("printf 'Do you want to %s?\\n' proceed")
  waits(g, 1)
  g.settle()
  t.match(bar_rows(g), "agents waiting: 1")
  t.eq(g.notifications(), { { body = "Agent waiting in window 1" } })
  g.wait(function(screen)
    local _, prompts = screen.text():gsub("│%$", "")
    return prompts == 2
  end)
  g.expect_screenshot({ styles = false })
end)

t.case("typing clears the waiting state", function(g)
  g.start({ size = "80x6" })
  g.run("printf 'Do you want to %s?\\n' proceed")
  waits(g, 1)
  g.type("x")
  g.wait(function()
    return agent(g, 1) == nil
  end)
  g.settle()
  t.ok(not bar_rows(g):find("agents waiting"), "the segment is hidden")
end)

t.case("prefix a focuses the next waiting window", function(g)
  g.start({ size = "80x6" })
  g.keys("ctrl+space n")
  g.settle()
  t.eq(g.client("return gband.view().window"), 2)
  g.server([[gband.window_state("default", 1).agent = "waiting"]])
  g.settle()
  g.keys("ctrl+space a")
  g.settle()
  t.eq(g.client("return gband.view().window"), 1)
end)
