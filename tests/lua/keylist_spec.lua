local t = require("gband.test")

local MINIMAL = [[
  gband.opt.statusline_position = "off"
  gband.plugin("gband.keylist")
  gband.keymap.set("prefix", "h", gband.action.focus_column_left, { desc = "focus the column to the left" })
  gband.keymap.set("prefix", "l", gband.action.focus_column_right, { desc = "focus the column to the right" })
  gband.keymap.set("prefix", "enter", gband.action.open_window, { desc = "open a window running the user's shell" })
  gband.keymap.set("prefix", "q", gband.action.close_window, { desc = "close the window" })
  gband.keymap.set("prefix", "u", gband.action.focus_band_down, { desc = "view the band below" })
  gband.keymap.set("prefix", "x", function() ran = true end, { desc = "say hi" })
  gband.keymap.set("prefix", "?", gband.action["keylist.open"])
]]

local function view(g)
  return g.client("local v = gband.view() return { window = v.window, plugin_window = v.plugin_window }")
end

local function lists(g)
  return g.client("return gband.win.list()")
end

local function info(g, win)
  return g.client("return gband.win.info(...)", win)
end

local function open(g)
  g.keys("ctrl+space ?")
  g.settle()
  local win = view(g).plugin_window
  t.ok(win, "the key list is focused")
  return win
end

local function line_of(g, win, key)
  return g.client([[
    for index, entry in ipairs(gband.keymap.list("prefix")) do
      if entry.key == ... then
        return index
      end
    end
  ]], key)
end

local function move_to(g, win, key)
  local line = line_of(g, win, key)
  g.client("gband.win.set_cursor(...)", win, line)
  g.settle()
end

local function row_of(g, text)
  local screen = g.screen()
  for row = 0, screen.rows - 1 do
    if screen.row(row):find(text, 1, true) then
      return row, screen.row(row)
    end
  end
end

local function cell_at(g, row, text, word)
  local pos = text:find(word, 1, true)
  return g.screen().cell(row, utf8.len(text:sub(1, pos - 1)))
end

local function prompts(g, count)
  g.wait(function(screen)
    local _, found = screen.text():gsub("│%$", "")
    return found == count
  end)
end

t.case("action registered", function(g)
  g.start({ config = MINIMAL })
  t.eq(g.client([[
    for _, action in ipairs(gband.action.list()) do
      if action.name == "keylist.open" then
        return action.desc
      end
    end
  ]]), "list the keys")
end)

t.case("unknown option", function(g)
  g.start({ config = [[gband.plugin("gband.keylist", { table = "root" })]] })
  g.settle()
  local logged = table.concat(g.log("client"), "\n")
  t.match(logged, "keylist")
  t.match(logged, "`table`")
end)

t.case("default list", function(g)
  g.start({})
  prompts(g, 1)
  local win = open(g)
  t.ok(row_of(g, "┌prefix keys"), "the title is drawn")
  local first, text = row_of(g, "focus the column to the left")
  t.match(text, "│h%s+focus the column to the left")
  t.eq(first, row_of(g, "┌prefix keys") + 1)
  t.eq(info(g, win).cursor, 1)
  t.ok(cell_at(g, first, text, "focus").inverse, "the cursor line is drawn")
  t.ok(row_of(g, "q        close the window"))
  g.expect_screenshot("first page")
  g.keys("end")
  g.settle()
  t.ok(row_of(g, "C-space  send the prefix key to the focused window"))
  local widest = g.client([[
    local key_form = require("gband.keyform")
    local keys, descs = 0, 0
    for _, entry in ipairs(gband.keymap.list("prefix")) do
      local key = entry.key == "prefix" and key_form(gband.opt.prefix) or key_form(entry.key)
      keys = math.max(keys, gband.ui.width(key))
      descs = math.max(descs, gband.ui.width(entry.desc))
    end
    return keys + 2 + descs
  ]])
  local shown = info(g, win)
  t.eq(shown.cols, widest)
  t.ok(shown.width <= 80)
  t.eq(shown.height, 15)
  g.expect_screenshot("last page")
end)

t.case("function binding is muted", function(g)
  g.start({ config = MINIMAL })
  open(g)
  local x, x_text = row_of(g, "say hi")
  local own, own_text = row_of(g, "list the keys")
  local h, h_text = row_of(g, "focus the column to the left")
  t.eq(cell_at(g, x, x_text, "say hi").fg, "#9aa5ce")
  t.eq(cell_at(g, own, own_text, "list the keys").fg, "#9aa5ce")
  t.eq(cell_at(g, h, h_text, "focus the column").fg, nil)
  t.eq(cell_at(g, x, x_text, "x").fg, "#7aa2f7")
end)

t.case("muted without a status line", function(g)
  g.start({ config = MINIMAL .. [[
    gband.hl.set("StatusLineMuted", nil)
    gband.hl.set("StatusLineAccent", nil)
  ]] })
  open(g)
  local x, x_text = row_of(g, "say hi")
  local h, h_text = row_of(g, "close the window")
  t.ok(cell_at(g, x, x_text, "say hi").dim, "the muted line is dim")
  t.ok(not cell_at(g, h, h_text, "close").dim, "the runnable line is not dim")
  t.ok(cell_at(g, h, h_text, "q ").bold, "the key is bold")
end)

t.case("moving through the list", function(g)
  g.start({ config = MINIMAL })
  local win = open(g)
  g.keys("j down k")
  g.settle()
  t.eq(info(g, win).cursor, 2)
end)

t.case("run a focus action", function(g)
  g.start({ config = MINIMAL })
  prompts(g, 1)
  g.keys("ctrl+space enter")
  prompts(g, 2)
  local second = view(g).window
  local win = open(g)
  g.keys("enter")
  g.settle()
  local now = view(g)
  t.ok(now.window ~= second, "the first column is focused")
  t.eq(now.plugin_window, win)
  g.keys("j")
  g.settle()
  t.eq(info(g, win).cursor, 2)
end)

t.case("run a resize from the list", function(g)
  g.start({})
  prompts(g, 1)
  g.keys("ctrl+space enter")
  prompts(g, 2)
  g.keys("ctrl+space [")
  g.settle()
  g.keys("ctrl+space k")
  g.settle()
  local function top()
    return g.client("return gband.layout().bands[1].columns[1].windows[1]")
  end
  local before = top()
  t.eq(view(g).window, before.id)
  local win = open(g)
  move_to(g, win, "+")
  g.keys("enter")
  g.wait(function()
    local after = top()
    return after.weight ~= before.weight or after.rows ~= before.rows
  end)
  t.eq(view(g).plugin_window, win)
end)

t.case("run close from the list", function(g)
  g.start({ config = MINIMAL })
  prompts(g, 1)
  local win = open(g)
  move_to(g, win, "q")
  g.keys("enter")
  g.settle()
  t.eq(lists(g), {})
  t.eq(#g.client("return gband.layout().bands[1].columns"), 1)
end)

t.case("run an action that opens a floating plugin window", function(g)
  g.start({ config = MINIMAL .. [[
    gband.action.register("popup", function()
      gband.win.open({ title = "popup", lines = { "hello" } })
    end, { desc = "open a popup" })
    gband.keymap.set("prefix", "e", gband.action.popup)
  ]] })
  local win = open(g)
  move_to(g, win, "e")
  g.keys("enter")
  g.settle()
  local popup = view(g).plugin_window
  t.ok(popup ~= win, "the new floating plugin window has focus")
  t.eq(lists(g), { win, popup })
  g.keys("q")
  g.settle()
  t.eq(lists(g), { win })
end)

t.case("function binding does nothing", function(g)
  g.start({ config = MINIMAL })
  local win = open(g)
  move_to(g, win, "x")
  g.keys("enter")
  g.settle()
  t.eq(g.client("return ran"), nil)
  t.eq(lists(g), { win })
  move_to(g, win, "?")
  g.keys("enter")
  g.settle()
  t.eq(lists(g), { win })
end)

t.case("q closes the list", function(g)
  g.start({ config = MINIMAL })
  prompts(g, 1)
  open(g)
  g.keys("q")
  g.settle()
  t.eq(lists(g), {})
  t.ok(not g.screen().text():find("│q", 1, true), "the shell received nothing")
end)

t.case("escape closes the list", function(g)
  g.start({ config = MINIMAL })
  open(g)
  g.keys("escape")
  g.wait(function(screen)
    return not screen.text():find("prefix keys", 1, true)
  end)
  t.eq(lists(g), {})
end)

t.case("prefix q on the empty band", function(g)
  g.start({ config = MINIMAL })
  g.keys("ctrl+space u")
  g.settle()
  t.eq(view(g).window, nil)
  open(g)
  g.keys("ctrl+space q")
  g.settle()
  t.eq(lists(g), {})
end)

t.case("open again while open", function(g)
  g.start({ config = MINIMAL })
  prompts(g, 1)
  g.keys("ctrl+space enter")
  prompts(g, 2)
  g.keys("ctrl+space h")
  g.settle()
  local win = open(g)
  g.keys("ctrl+space l")
  g.settle()
  t.eq(view(g).plugin_window, nil)
  t.eq(open(g), win)
  t.eq(lists(g), { win })
  g.keys("q")
  g.settle()
  t.eq(lists(g), {})
end)
