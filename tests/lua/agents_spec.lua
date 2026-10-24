local t = require("gband.test")

local FIXTURES = "crates/server/tests/fixtures/claude/"

local function read(name)
  local file = assert(io.open(FIXTURES .. name))
  local text = file:read("a")
  file:close()
  return text
end

local FILES = {}
for _, name in ipairs({ "stand-in.sh", "start.txt", "working.txt", "permission.txt", "finished.txt" }) do
  FILES["stand-in/" .. name] = read(name)
end

local function column(screen)
  local cells = {}
  for row = 0, screen.rows - 1 do
    cells[#cells + 1] = screen.cell(row, 0).char
  end
  local trimmed = table.concat(cells):gsub("%s+$", "")
  return (trimmed:gsub(" +", " "))
end

local function start(g, size)
  g.start({ size = size or "80x24", files = FILES })
  g.wait_text("$")
end

local function run_agent(g, window)
  local dir = g.client("return gband.config_dir")
  g.run(("clear; bash -c 'exec -a claude bash %s/stand-in/stand-in.sh'"):format(dir))
  g.wait_text("agent " .. window)
end

local function open_window(g)
  local before = #g.client("return gband.layout().bands[1].columns")
  g.keys("ctrl+space n escape")
  g.settle()
  g.wait(function(screen)
    return #g.client("return gband.layout().bands[1].columns") > before and screen.text():find("│%$")
  end)
end

local function wait_column(g, expected)
  return g.wait(function(screen)
    return column(screen) == expected
  end)
end

local function wait_fg(g, row, fg, underline)
  g.wait(function(screen)
    local cell = screen.cell(row, 0)
    return cell.fg == fg and cell.underline == underline and not cell.bold
  end)
end

local function focused(g)
  return g.client("return gband.view().window")
end

local function two_agents_beside_a_shell(g)
  start(g, "120x24")
  run_agent(g, 1)
  open_window(g)
  run_agent(g, 2)
  open_window(g)
  wait_column(g, "I 12 ─12")
  t.eq(focused(g), 3)
end

t.case("agent starts", function(g)
  start(g)
  run_agent(g, 1)
  wait_column(g, "I 12 ─1")
  t.eq(g.screen().cell(12, 0).char, "─")
  t.eq(g.screen().cell(13, 0).char, "1")
  t.eq(g.client("return gband.window_state(1).agent_state"), { agent = "claude", state = "idle" })
  g.expect_screenshot("idle agent")
end)

t.case("agent exits", function(g)
  start(g)
  run_agent(g, 1)
  wait_column(g, "I 12 ─1")
  g.keys("q")
  wait_column(g, "I 12 ─")
  t.eq(g.client("return gband.window_state(1).agent_state"), nil)
end)

t.case("rows with agents", function(g)
  start(g)
  run_agent(g, 1)
  open_window(g)
  run_agent(g, 2)
  wait_column(g, "I 12 ─12")
  g.expect_screenshot("two agents")
end)

t.case("agent colors", function(g)
  start(g)
  run_agent(g, 1)
  open_window(g)
  run_agent(g, 2)
  wait_column(g, "I 12 ─12")
  g.keys("w")
  wait_fg(g, 14, 3, true)
  g.client([[gband.window.send_keys(1, "b")]])
  wait_fg(g, 13, 1, false)
  wait_fg(g, 2, 1, true)
  g.expect_screenshot("blocked and working")
  g.client([[gband.window.send_keys(1, "w")]])
  wait_fg(g, 13, 3, false)
  g.client([[gband.window.send_keys(1, "i")]])
  wait_fg(g, 13, 2, false)
  t.eq(g.client("return gband.window_state(1).agent_state.state"), "done")
  g.expect_screenshot("done and working")
  g.keys("ctrl+space h escape")
  wait_fg(g, 13, 4, true)
  t.eq(g.client("return gband.window_state(1).agent_state.state"), "idle")
end)

t.case("click an agent in another band", function(g)
  start(g)
  run_agent(g, 1)
  g.keys("ctrl+space u n escape")
  g.settle()
  g.wait_text("│$")
  run_agent(g, 2)
  wait_column(g, "I 123 ─12")
  g.keys("ctrl+space i escape")
  g.settle()
  t.eq(focused(g), 1)
  g.mouse("press", "left", 0, 14)
  g.mouse("release", "left", 0, 14)
  g.settle()
  t.eq(focused(g), 2)
  t.eq(g.client("return gband.view().band"), g.client("return gband.layout().bands[2].id"))
  wait_fg(g, 14, 4, true)
end)

t.case("prefix a then a position focuses that agent", function(g)
  two_agents_beside_a_shell(g)
  g.keys("ctrl+space a 2")
  g.settle()
  t.eq(focused(g), 2)
  t.eq(g.client("return gband.keymap.current_table()"), "root")
  t.eq(column(g.screen()):sub(1, 1), "I")
end)

t.case("up, down, j and k in agents mode", function(g)
  two_agents_beside_a_shell(g)
  g.keys("ctrl+space a")
  g.settle()
  t.eq(column(g.screen()):sub(1, 1), "A")
  g.keys("down")
  g.settle()
  t.eq(focused(g), 1)
  g.keys("down")
  g.settle()
  t.eq(focused(g), 2)
  g.keys("down")
  g.settle()
  t.eq(focused(g), 1)
  g.keys("up")
  g.settle()
  t.eq(focused(g), 2)
  g.keys("k")
  g.settle()
  t.eq(focused(g), 1)
  g.keys("j")
  g.settle()
  t.eq(focused(g), 2)
  t.eq(g.client("return gband.keymap.current_table()"), "agents")
  g.expect_screenshot("agents mode")
  g.keys("enter")
  g.settle()
  t.eq(g.client("return gband.keymap.current_table()"), "root")
  t.eq(focused(g), 2)
end)

t.case("prefix b then a position views that band", function(g)
  start(g)
  g.keys("ctrl+space u n escape")
  g.settle()
  g.wait_text("│$")
  g.keys("ctrl+space i escape")
  g.settle()
  g.keys("ctrl+space b 2")
  g.settle()
  t.eq(g.client("return gband.view().band"), g.client("return gband.layout().bands[2].id"))
  t.eq(g.client("return gband.keymap.current_table()"), "bands")
end)

t.case("up, down, j and k keep bands mode active", function(g)
  start(g)
  g.keys("ctrl+space u n escape")
  g.settle()
  g.wait_text("│$")
  g.keys("ctrl+space i escape")
  g.settle()
  local function viewed()
    return g.client([[
      local view = gband.view().band
      for position, band in ipairs(gband.layout().bands) do
        if band.id == view then
          return position
        end
      end
    ]])
  end
  local function current()
    return g.client("return gband.keymap.current_table()")
  end
  g.keys("ctrl+space b")
  g.settle()
  t.eq(column(g.screen()):sub(1, 1), "B")
  g.keys("down")
  g.settle()
  t.eq(viewed(), 2)
  g.keys("j")
  g.settle()
  t.eq(viewed(), 3)
  g.keys("k")
  g.settle()
  t.eq(viewed(), 2)
  t.eq(current(), "bands")
  g.expect_screenshot("bands mode")
  g.keys("up")
  g.settle()
  t.eq(viewed(), 1)
  t.eq(current(), "bands")
  g.keys("escape")
  g.settle()
  t.eq(current(), "root")
  t.eq(viewed(), 1)
end)
