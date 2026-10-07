local t = require("gband.test")

local MINIMAL = [[
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

local function current_table(g)
  return g.client("return gband.keymap.current_table()")
end

local function open(g)
  if current_table(g) == "prefix" then
    g.keys("?")
  else
    g.keys("ctrl+space ?")
  end
  g.settle()
  local win = view(g).plugin_window
  t.ok(win, "the key list is focused")
  t.eq(current_table(g), "root")
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

local MUTED = 8

local function muted_lines(g, win)
  local box = info(g, win)
  local screen = g.screen()
  local left = box.col + screen.cols - g.client("return gband.view().cols")
  local muted = {}
  for row = box.row + 1, box.row + box.height - 2 do
    local chars = {}
    for col = left + 1, left + box.width - 2 do
      chars[#chars + 1] = screen.cell(row, col).char
    end
    local text = table.concat(chars)
    local start, desc = text:match("^%S+%s+()(.-)%s*$")
    if start and screen.cell(row, left + start).fg == MUTED then
      muted[#muted + 1] = desc
    end
  end
  return muted
end

local function prompts(g, count)
  g.wait(function(screen)
    local _, found = screen.text():gsub("│%$", "")
    return found == count
  end)
end

local function prompted_columns(g, count)
  g.wait(function(screen)
    local windows = g.client("return #gband.layout().bands[1].columns")
    local _, found = screen.text():gsub("│%$", "")
    return windows == count and found == count
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
  t.ok(row_of(g, "┌navigation keys"), "the title is drawn")
  local first, text = row_of(g, "focus the column to the left")
  t.match(text, "│h%s+focus the column to the left")
  t.eq(first, row_of(g, "┌navigation keys") + 1)
  t.eq(info(g, win).cursor, 1)
  t.eq(cell_at(g, first, text, "focus").bg, 4, "the cursor line is drawn")
  t.ok(row_of(g, "q        close the window"))
  g.expect_screenshot("first page")
  g.keys("end")
  g.settle()
  t.ok(row_of(g, "C-space  send the prefix key"))
  local widest = g.client([[
    local key_form = require("gband.keyform")
    local keys, descs = 0, 0
    for _, entry in ipairs(gband.keymap.list("prefix")) do
      if not key_form.is_mouse(entry.key) then
        local key = entry.key == "prefix" and key_form(gband.opt.prefix) or key_form(entry.key)
        keys = math.max(keys, gband.ui.width(key))
        descs = math.max(descs, gband.ui.width(entry.desc))
      end
    end
    return keys + 2 + descs
  ]])
  local shown = info(g, win)
  t.eq(shown.cols, widest)
  t.ok(shown.width <= 80)
  t.eq(shown.height, 15)
  t.eq(shown.line_count, g.client("return #gband.keymap.list('prefix')") - 8)
  t.ok(not g.screen().text():find("mouse"), "no mouse binding is listed")
  t.ok(not g.screen().text():find("wheel"), "no wheel binding is listed")
  g.expect_screenshot("last page")
end)

t.case("prefix key in the key list", function(g)
  g.start({ config = [[
    gband.opt.prefix = "ctrl+b"
    gband.keystyle.use()
  ]] })
  prompts(g, 1)
  g.keys("ctrl+b ? end")
  g.settle()
  local _, line = row_of(g, "send the prefix key")
  t.match(line, "│C%-b +send the prefix key")
end)

t.case("title of a prefix table that is not a mode", function(g)
  g.start({ config = MINIMAL })
  open(g)
  t.ok(row_of(g, "┌prefix keys"), "the title is drawn")
end)

t.case("function binding can run", function(g)
  g.start({ config = MINIMAL })
  open(g)
  local x, x_text = row_of(g, "say hi")
  local own, own_text = row_of(g, "list the keys")
  local h, h_text = row_of(g, "focus the column to the left")
  t.eq(cell_at(g, x, x_text, "say hi").fg, 7)
  t.eq(cell_at(g, own, own_text, "list the keys").fg, MUTED)
  t.ok(cell_at(g, h, h_text, "focus the column").fg ~= MUTED, "the runnable line is not muted")
  t.eq(cell_at(g, x, x_text, "x").fg, 4)
end)

t.case("only the key list's own line is muted", function(g)
  g.start({})
  local win = open(g)
  local muted = {}
  local count = #g.client("return gband.keymap.list('prefix')")
  for _, line in ipairs({ 1, 19, count }) do
    g.client("gband.win.set_cursor(...)", win, line)
    g.settle()
    for _, desc in ipairs(muted_lines(g, win)) do
      muted[desc] = true
    end
  end
  t.eq(muted, { ["list the keys"] = true })
end)

t.case("muted without colorscheme settings", function(g)
  g.start({ config = MINIMAL .. [[
    gband.hl.set("KeyListMuted", nil)
    gband.hl.set("KeyListKey", nil)
  ]] })
  open(g)
  local own, own_text = row_of(g, "list the keys")
  local x, x_text = row_of(g, "say hi")
  local h, h_text = row_of(g, "close the window")
  t.ok(cell_at(g, own, own_text, "list the keys").dim, "the muted line is dim")
  t.ok(not cell_at(g, x, x_text, "say hi").dim, "the function line is not dim")
  t.ok(not cell_at(g, h, h_text, "close").dim, "the runnable line is not dim")
  t.ok(cell_at(g, h, h_text, "q ").bold, "the key is bold")
end)

t.case("moving through the list", function(g)
  g.start({ config = MINIMAL })
  local win = open(g)
  g.keys("down down up")
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
  g.keys("down")
  g.settle()
  t.eq(info(g, win).cursor, 2)
end)

t.case("run a resize from the list", function(g)
  g.start({})
  prompts(g, 1)
  g.keys("ctrl+space n enter")
  prompted_columns(g, 2)
  g.keys("ctrl+space [")
  g.settle()
  g.keys("k")
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
  t.eq(lists(g), { popup })
  g.keys("q")
  g.settle()
  t.eq(lists(g), {})
end)

local function settings_focused(g)
  local focused = view(g).plugin_window
  t.ok(focused, "a plugin window has focus")
  t.eq(lists(g), { focused })
  t.eq(info(g, focused).line_count, 4)
end

t.case("open the settings from the list", function(g)
  g.start({})
  local win = open(g)
  move_to(g, win, "s")
  g.keys("enter")
  g.settle()
  settings_focused(g)
  t.ok(g.screen().text():find("settings", 1, true), "the settings window is drawn")
end)

t.case("open the settings by the line's key", function(g)
  g.start({})
  open(g)
  g.keys("s")
  g.settle()
  settings_focused(g)
end)

t.case("run a function binding", function(g)
  g.start({ config = MINIMAL })
  local win = open(g)
  move_to(g, win, "x")
  g.keys("enter")
  g.settle()
  t.eq(g.client("return ran"), true)
  t.eq(lists(g), { win })
  t.eq(view(g).plugin_window, win)
end)

t.case("run n from the list", function(g)
  g.start({})
  prompts(g, 1)
  local win = open(g)
  move_to(g, win, "n")
  g.keys("enter")
  prompted_columns(g, 2)
  g.settle()
  t.eq(#g.client("return gband.layout().bands[1].columns"), 2)
  t.eq(lists(g), { win })
  t.eq(view(g).plugin_window, win)
  t.eq(current_table(g), "root")
  t.ok(not g.screen().text():find("│%$ %S"), "no window received a key")
end)

t.case("the key list's own line does nothing", function(g)
  g.start({ config = MINIMAL })
  local win = open(g)
  move_to(g, win, "?")
  g.keys("enter")
  g.settle()
  t.eq(lists(g), { win })
  t.eq(view(g).plugin_window, win)
end)

t.case("a line's key runs it", function(g)
  g.start({})
  prompts(g, 1)
  g.keys("ctrl+space n enter")
  prompted_columns(g, 2)
  g.keys("ctrl+space h")
  g.settle()
  local first = view(g).window
  local win = open(g)
  t.eq(info(g, win).cursor, 1)
  g.keys("l")
  g.settle()
  local now = view(g)
  t.ok(now.window ~= first, "the second column is focused")
  t.eq(now.plugin_window, win)
  t.eq(info(g, win).cursor, line_of(g, win, "l"))
  t.eq(lists(g), { win })
end)

t.case("j runs its binding", function(g)
  g.start({})
  prompts(g, 1)
  g.keys("ctrl+space n enter")
  prompted_columns(g, 2)
  g.keys("ctrl+space [")
  g.settle()
  g.keys("k")
  g.settle()
  local column = g.client("return gband.layout().bands[1].columns[1].windows")
  t.eq(view(g).window, column[1].id)
  local win = open(g)
  g.keys("j")
  g.settle()
  local now = view(g)
  t.eq(now.window, column[2].id)
  t.eq(now.plugin_window, win)
  t.eq(info(g, win).cursor, line_of(g, win, "j"))
  t.eq(lists(g), { win })
end)

t.case("a function binding by its key", function(g)
  g.start({})
  prompts(g, 1)
  local win = open(g)
  t.eq(info(g, win).cursor, 1)
  g.keys("n")
  prompted_columns(g, 2)
  g.settle()
  t.eq(#g.client("return gband.layout().bands[1].columns"), 2)
  t.eq(info(g, win).cursor, line_of(g, win, "n"))
  t.eq(lists(g), { win })
  t.eq(view(g).plugin_window, win)
  t.ok(not g.screen().text():find("│%$ %S"), "no window received a key")
end)

t.case("the key list's own line by its key", function(g)
  g.start({})
  prompts(g, 1)
  local win = open(g)
  local before = view(g).window
  g.keys("?")
  g.settle()
  t.eq(info(g, win).cursor, line_of(g, win, "?"))
  t.eq(lists(g), { win })
  t.eq(view(g), { window = before, plugin_window = win })
  t.eq(#g.client("return gband.layout().bands[1].columns"), 1)
end)

t.case("arrows move the cursor line", function(g)
  g.start({})
  prompts(g, 1)
  g.keys("ctrl+space n")
  prompted_columns(g, 2)
  local before = view(g).window
  local win = open(g)
  g.keys("down down up")
  g.settle()
  t.eq(info(g, win).cursor, 2)
  t.eq(view(g), { window = before, plugin_window = win })
end)

t.case("a line under an own key runs only through enter", function(g)
  g.start({ config = [[
    gband.plugin("gband.keylist")
    gband.keymap.set("prefix", "n", gband.action.open_window)
    gband.keymap.set("prefix", "?", gband.action["keylist.open"])
    gband.keymap.set("prefix", "home", gband.action.focus_column_left)
  ]] })
  prompts(g, 1)
  local first = view(g).window
  g.keys("ctrl+space n")
  prompts(g, 2)
  local second = view(g).window
  t.ok(second ~= first, "the second column is focused")
  local win = open(g)
  local count = #g.client("return gband.keymap.list('prefix')")
  g.client("gband.win.set_cursor(...)", win, count)
  g.settle()
  g.keys("home")
  g.settle()
  t.eq(info(g, win).cursor, 1)
  t.eq(view(g), { window = second, plugin_window = win })
  move_to(g, win, "home")
  g.keys("enter")
  g.settle()
  t.eq(view(g), { window = first, plugin_window = win })
end)

t.case("an unbound j moves the cursor line", function(g)
  g.start({ config = [[
    gband.plugin("gband.keylist")
    gband.keymap.set("prefix", "?", gband.action["keylist.open"])
    gband.keymap.set("prefix", "h", gband.action.focus_column_left)
    gband.keymap.set("prefix", "l", gband.action.focus_column_right)
  ]] })
  local win = open(g)
  t.eq(info(g, win).cursor, 1)
  g.keys("j")
  g.settle()
  t.eq(info(g, win).cursor, 2)
end)

t.case("q closes the list", function(g)
  g.start({})
  prompts(g, 1)
  open(g)
  g.keys("q")
  g.settle()
  t.eq(lists(g), {})
  t.eq(#g.client("return gband.layout().bands[1].columns"), 1)
  t.ok(not g.screen().text():find("│q", 1, true), "the shell received nothing")
end)

t.case("q with no q binding", function(g)
  g.start({ config = [[
    gband.plugin("gband.keylist")
    gband.keymap.set("prefix", "?", gband.action["keylist.open"])
  ]] })
  prompts(g, 1)
  open(g)
  g.keys("q")
  g.settle()
  t.eq(lists(g), {})
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
  g.start({})
  prompts(g, 1)
  g.keys("ctrl+space n enter")
  prompted_columns(g, 2)
  g.keys("ctrl+space h")
  g.settle()
  local win = open(g)
  g.keys("ctrl+space l")
  g.settle()
  t.eq(view(g).plugin_window, nil)
  t.eq(current_table(g), "prefix")
  t.eq(open(g), win)
  t.eq(lists(g), { win })
  g.keys("q")
  g.settle()
  t.eq(lists(g), {})
end)
