local t = require("gband.test")

local function started(g, size)
  g.start({ window_titles = true, size = size })
  g.wait_text("$")
  g.wait_text("╭sh")
  g.settle()
end

local function column_of(line, text)
  local at = line:find(text, 1, true)
  return at and utf8.len(line:sub(1, at - 1))
end

t.case("no window titles by default", function(g)
  g.start({})
  g.wait_text("$")
  g.settle()
  local row = g.screen().row(0)
  t.eq(row, "I╭" .. string.rep("─", 37) .. "╮")
end)

t.case("window titles asked for", function(g)
  started(g)
  local row = g.screen().row(0)
  t.eq(row, "I╭sh" .. string.rep("─", 35) .. "╮")
  g.expect_screenshot("tile")
end)

t.case("two shells are numbered in layout order", function(g)
  started(g)
  g.keys("ctrl+space n enter")
  g.wait_text("sh #2")
  g.settle()
  local row = g.screen().row(0)
  t.ok(column_of(row, "sh #1") < column_of(row, "sh #2"), row)
  g.expect_screenshot("numbered")
end)

t.case("a long title is cut to the border", function(g)
  started(g, "30x8")
  g.run("printf '\\033]2;cargo test --workspace\\007'")
  g.wait_text("╭cargo")
  g.settle()
  local row = g.screen().row(0)
  t.ok(row:find("╭cargo test -╮", 1, true), row)
  g.expect_screenshot("cut")
end)

t.case("the unfocused title takes the unfocused border's style", function(g)
  started(g)
  g.keys("ctrl+space n")
  g.wait_text("sh #2")
  g.settle()
  local screen = g.screen()
  local row = screen.row(0)
  local first, second = column_of(row, "sh #1"), column_of(row, "sh #2")
  for _, col in ipairs({ first, second }) do
    local title, border = screen.cell(0, col), screen.cell(0, col - 1)
    t.eq(
      { title.fg, title.bold, title.dim },
      { border.fg, border.bold, border.dim },
      "title at column " .. col
    )
  end
  t.ok(screen.cell(0, second).bold ~= screen.cell(0, first).bold, "the focused title differs")
end)

t.case("the rename prompt names the window", function(g)
  started(g)
  g.keys("ctrl+space N")
  g.settle()
  t.eq(g.client("return gband.keymap.current_table()"), "root")
  g.type("logs")
  g.settle()
  t.ok(g.screen().text():find("rename", 1, true), "the prompt is titled rename")
  g.expect_screenshot("prompt")
  g.keys("enter")
  g.wait_text("╭logs")
  g.settle()
  t.eq(
    g.client("local w = gband.layout().bands[1].columns[1].windows[1] return w.name, w.manual_name"),
    "logs"
  )
  g.expect_screenshot("renamed")
end)
