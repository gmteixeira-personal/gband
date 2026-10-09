local t = require("gband.test")

local function row_of(g, text)
  local screen = g.screen()
  for row = 0, screen.rows - 1 do
    local line = screen.row(row)
    local at = line:find(text, 1, true)
    if at then
      return row, utf8.len(line:sub(1, at - 1)), line
    end
  end
end

local function open(g)
  g.keys("ctrl+space :")
  g.settle()
  t.eq(g.client("return gband.keymap.current_table()"), "root")
end

t.case("the prompt holds the typed text and a reversed cursor cell", function(g)
  g.start({})
  g.wait_text("$")
  open(g)
  g.type("abc")
  g.settle()
  local row, col = row_of(g, ":abc")
  t.eq(row, 22)
  t.eq(col, 2)
  t.ok(g.screen().cell(row, col + 4).inverse, "the cursor cell is reversed")
  t.ok(not g.screen().cell(row, col + 3).inverse, "the typed text is not reversed")
  t.match(g.screen().row(21), "^.-lua")
  g.expect_screenshot("typed")
end)

t.case("a long line shows its end", function(g)
  g.start({ size = "21x8" })
  g.wait_text("$")
  open(g)
  g.type("abcdefghijklmnopqrstuvwxyz")
  g.settle()
  local row, col, line = row_of(g, "uvwxyz")
  t.ok(row, "the end of the line is shown")
  t.ok(not line:find(":", 1, true), "the colon is left out")
  t.ok(g.screen().cell(row, col + 6).inverse, "the cursor follows the last character")
  g.expect_screenshot("long")
end)

t.case("the key list holds the prompt between the key list and detach", function(g)
  g.start({})
  g.wait_text("$")
  g.keys("ctrl+space ? end")
  g.settle()
  local own = g.client([[
    for index, entry in ipairs(gband.keymap.list("prefix")) do
      if entry.key == "?" then
        return index
      end
    end
  ]])
  g.client("gband.win.set_cursor(gband.view().plugin_window, ...)", own)
  g.settle()
  local list = row_of(g, "list the keys")
  t.ok(list, "the key list's own line is shown")
  t.match(g.screen().row(list + 1), "│:%s+run Lua")
  t.match(g.screen().row(list + 2), "│N%s+rename the window")
  t.match(g.screen().row(list + 3), "│s%s+settings")
  t.match(g.screen().row(list + 4), "│!%s+reload the configuration")
  t.match(g.screen().row(list + 5), "│m%s+multi mode")
  t.match(g.screen().row(list + 6), "│D%s+detach")
  local order = g.client([[
    local keys = {}
    for _, entry in ipairs(gband.keymap.list("prefix")) do
      keys[#keys + 1] = entry.key
    end
    return table.concat(keys, " ")
  ]])
  t.match(order, "%? : N s ! m D ")
  g.expect_screenshot("hint")
end)
