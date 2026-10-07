local t = require("gband.test")

local CONFIG = [[
gband.keystyle.use()
gband.core.provide("decorations", function(info)
  if info.floating then
    return { "[_]", "[□]", "[X]" }
  end
end)
]]

local BUTTONS = "[_][□][X]"

local function column_of(line, text)
  local at = line:find(text, 1, true)
  return at and utf8.len(line:sub(1, at - 1))
end

local function floated(g, opts)
  opts = opts or {}
  g.start({ window_titles = true, config = opts.config or CONFIG, size = opts.size })
  g.wait_text("$")
  g.client("gband.action.open_window()")
  g.wait(function()
    return g.client("return #gband.layout().bands[1].columns") == 2
  end)
  local second = g.client("return gband.layout().bands[1].columns[2].windows[1].id")
  g.wait(function()
    return g.client("return gband.view().window") == second
  end)
  g.client("gband.action.toggle_window_floating({ window = ..., floating = true })", second)
  g.wait(function()
    return #g.client("return gband.layout().bands[1].floating") == 1
  end)
  g.wait_text("╭sh #2")
  g.settle()
end

local function title_bar(screen)
  for row = 0, screen.rows - 1 do
    local line = screen.row(row)
    local col = column_of(line, BUTTONS)
    if col then
      return row, col, line
    end
  end
end

t.case("a floating window's title bar beside a tile", function(g)
  floated(g)
  local screen = g.screen()
  local row, col, line = title_bar(screen)
  t.ok(row, "a row shows the buttons")
  t.ok(line:find("[X]─╮", 1, true), line)
  t.ok(column_of(line, "╭sh") < col, line)
  t.ok(not screen.row(0):find("[", 1, true), "the tile shows no span")
  g.expect_screenshot("title_bar")
end)

t.case("the spans take the focused and the unfocused border style", function(g)
  floated(g)
  local function styles()
    local screen = g.screen()
    local row, col = title_bar(screen)
    local span, border = screen.cell(row, col + 1), screen.cell(row, col + 9)
    return { span.fg, span.bold, span.dim }, { border.fg, border.bold, border.dim }
  end
  local focused, focused_border = styles()
  t.eq(focused, focused_border)
  g.client("gband.action.switch_focus_floating_tiled()")
  g.settle()
  local unfocused, unfocused_border = styles()
  t.eq(unfocused, unfocused_border)
  t.ok(focused[2] ~= unfocused[2], "the focused spans differ")
end)

t.case("a narrow floating window draws no span", function(g)
  floated(g, { size = "24x10" })
  local text = g.screen().text()
  t.ok(not text:find("[X]", 1, true), text)
  t.ok(text:find("╭sh", 1, true), text)
  g.expect_screenshot("narrow")
end)

t.case("a failing function is reported once", function(g)
  g.start({
    config = [[
      gband.keystyle.use()
      gband.core.provide("decorations", function() error("boom") end)
    ]],
  })
  g.wait_text("$")
  g.keys("ctrl+space n")
  g.settle()
  g.keys("ctrl+space h")
  g.settle()
  local errors = g.client("return gband.errors()")
  t.eq(#errors, 1)
  t.match(errors[1], "^decorations: .*boom")
end)

t.case("the default configuration draws no decorations", function(g)
  g.start({ window_titles = true })
  g.wait_text("$")
  g.keys("ctrl+space n")
  g.wait_text("sh #2")
  g.settle()
  local row = g.screen().row(0)
  t.ok(not row:find("[", 1, true), row)
end)
