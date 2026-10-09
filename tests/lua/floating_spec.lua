local t = require("gband.test")

local CONFIG = 'gband.keystyle.use("floating")'
local TILED = CONFIG .. [[

  gband.bind("f5", function() gband.win.open({ kind = "tiled", lines = { "hi" } }) end)
]]
local SAVED = { ["user/keystyle.lua"] = 'return "floating"\n' }

local function prompts(g, count)
  g.wait(function(screen)
    local _, found = screen.text():gsub("│%$", "")
    return found >= count
  end)
end

local function cells(text, first, count)
  local out = {}
  local index = 0
  for _, code in utf8.codes(text) do
    if index >= first and index < first + count then
      out[#out + 1] = utf8.char(code)
    end
    index = index + 1
  end
  local found = table.concat(out)
  return found .. string.rep(" ", count - utf8.len(found))
end

local function box(g, id)
  return g.client([[
    local layout = gband.layout()
    for _, band in ipairs(layout.bands) do
      for _, box in ipairs(band.floating) do
        if box.id == ... then
          local width = box.full_width and layout.cols or math.floor(layout.cols * box.width + 1e-9)
          return { width, math.min(box.rows, layout.rows), box.col, box.row }
        end
      end
    end
  ]], id)
end

local function floating_ids(g)
  return g.client([[
    local ids = {}
    for _, band in ipairs(gband.layout().bands) do
      for _, box in ipairs(band.floating) do
        ids[#ids + 1] = box.id
      end
    end
    return ids
  ]])
end

local function tiled_ids(g)
  return g.client([[
    local ids = {}
    for _, band in ipairs(gband.layout().bands) do
      for _, column in ipairs(band.columns) do
        for _, window in ipairs(column.windows) do
          ids[#ids + 1] = window.id
        end
      end
    end
    return ids
  ]])
end

local function in_layout(g, id)
  for _, found in ipairs(floating_ids(g)) do
    if found == id then
      return true
    end
  end
  for _, found in ipairs(tiled_ids(g)) do
    if found == id then
      return true
    end
  end
  return false
end

local function minimized(g, id)
  return g.client([[
    for _, band in ipairs(gband.layout().bands) do
      for _, box in ipairs(band.floating) do
        if box.id == ... then
          return box.minimized == true
        end
      end
    end
    return false
  ]], id)
end

local function place(g, id, width, rows, col, row)
  g.client([[
    local id, width, rows, col, row = ...
    gband.window.set_width(id, width)
    gband.window.set_height(id, { rows = rows })
    gband.window.set_position(id, { col = col, row = row })
  ]], id, width, rows, col, row)
  g.settle()
end

local function focused(g)
  return g.client("return gband.view().window")
end

local function click(g, col, row, button, mods)
  g.mouse("press", button or "left", col, row, mods)
  g.mouse("release", button or "left", col, row, mods)
  g.settle()
end

local function drag(g, from_col, from_row, to_col, to_row, mods)
  g.mouse("press", "left", from_col, from_row, mods)
  g.mouse("drag", "left", to_col, to_row, mods)
  g.mouse("release", "left", to_col, to_row, mods)
  g.settle()
end

local function open_window(g)
  g.client("gband.action.open_window({ floating = true })")
  g.settle()
  local ids = floating_ids(g)
  return ids[#ids]
end

local function scene(g, opts)
  opts = opts or {}
  g.start({ size = opts.size, config = CONFIG, theme = opts.theme })
  prompts(g, 1)
  g.client("gband.window.rename(1, 'notes')")
  g.settle()
  if opts.parked ~= false then
    local parked = open_window(g)
    g.client("gband.window.rename(..., 'parked')", parked)
    place(g, parked, 0.15, 3, 0, 21)
  end
  place(g, 1, 0.5, 12, 20, 4)
  g.client("gband.window.focus(1)")
  g.settle()
end

local function plugin_window(g)
  local info = g.client([[
    local id = gband.view().plugin_window
    if not id then
      return nil
    end
    local info = gband.win.info(id)
    info.left = 0
    for _, bar in ipairs(gband.bar.list()) do
      if bar.shown and bar.side == "left" then
        info.left = info.left + bar.width
      end
    end
    info.cursor_style = require("gband.hl").drawn("PluginWindowCursorLine")
    return info
  ]])
  if not info then
    return nil
  end
  local screen = g.screen()
  info.lines = {}
  for row = info.row, info.row + info.height - 1 do
    info.lines[#info.lines + 1] = cells(screen.row(row), info.left + info.col, info.width)
    local cell = screen.cell(row, info.left + info.col + 1)
    if cell.bg ~= nil and cell.bg == info.cursor_style.bg and cell.fg == info.cursor_style.fg then
      info.selected = info.lines[#info.lines]:sub(utf8.offset(info.lines[#info.lines], 3)):match("^(.-)%s*│$")
    end
  end
  return info
end

local function entries(info)
  local found = {}
  for index = 2, #info.lines - 1 do
    local line = info.lines[index]
    if line:find("^├") then
      found[#found + 1] = "-"
    else
      found[#found + 1] = line:match("^│ (.-)%s*│$")
    end
  end
  return found
end

local function open_list(g)
  g.keys("ctrl+space")
  g.settle()
  return plugin_window(g)
end

local function last_focus(g, id)
  return g.client([[
    for _, band in ipairs(gband.layout().bands) do
      for _, box in ipairs(band.floating) do
        if box.id == ... then
          return box.last_focus
        end
      end
    end
  ]], id)
end

local function most_recent(g)
  return g.client([[
    local found, best = nil, -1
    for _, band in ipairs(gband.layout().bands) do
      for _, box in ipairs(band.floating) do
        if (box.last_focus or -1) > best then
          found, best = box.id, box.last_focus
        end
      end
    end
    return found
  ]])
end

local function over(g, upper, lower)
  local a, b = box(g, upper), box(g, lower)
  local left = g.client([[
    local left = 0
    for _, bar in ipairs(gband.bar.list()) do
      if bar.shown and bar.side == "left" then
        left = left + bar.width
      end
    end
    return left
  ]])
  local function inside(p, q)
    return p[3] > q[3] and p[3] < q[3] + q[1] - 1 and p[4] > q[4] and p[4] < q[4] + q[2] - 1
  end
  if inside(a, b) then
    return g.screen().cell(a[4], left + a[3]).char == "╭"
  end
  if inside(b, a) then
    return g.screen().cell(b[4], left + b[3]).char ~= "╭"
  end
  error("the boxes of windows " .. upper .. " and " .. lower .. " do not overlap by a corner")
end

local function list_closed(g)
  return g.client([[
    for _, id in ipairs(gband.win.list()) do
      if gband.win.info(id).hover then
        return false
      end
    end
    return true
  ]])
end

local function viewed_band(g)
  return g.client([[
    for position, band in ipairs(gband.layout().bands) do
      if band.id == gband.view().band then
        return position
      end
    end
  ]])
end

local function focus_in_order(g, ids)
  for _, id in ipairs(ids) do
    g.client("gband.window.focus(...)", id)
    g.settle()
  end
end

local function named(g, names)
  for id, name in pairs(names) do
    g.client("local id, name = ... gband.window.rename(id, name)", id, name)
  end
  g.settle()
end

local function recent(g, opts)
  opts = opts or {}
  g.start({ config = opts.config or CONFIG })
  prompts(g, 1)
  open_window(g)
  open_window(g)
  named(g, { "notes", "logs", "mail" })
  prompts(g, 3)
  focus_in_order(g, { 1, 2, 3 })
end

local function two_bands(g)
  g.start({ config = CONFIG })
  prompts(g, 1)
  open_window(g)
  g.client("gband.action.focus_band_down()")
  g.settle()
  open_window(g)
  named(g, { "notes", "logs", "mail" })
  focus_in_order(g, { 3, 1, 2 })
end

t.case("first window of a new session", function(g)
  g.start({ files = SAVED })
  local record = g.client("return gband.layout().bands[1].floating[1]")
  t.eq({ record.width, record.full_width, record.rows, record.col, record.row }, { 0.5, false, 20, 20, 2 })
  t.eq(tiled_ids(g), {})
end)

t.case("spawned window floats", function(g)
  g.start({ config = CONFIG })
  g.client('gband.spawn({ cmd = "sh" })')
  g.settle()
  t.eq(floating_ids(g), { 1, 2 })
  t.eq(focused(g), 2)
  t.eq(tiled_ids(g), {})
end)

t.case("drawn window stays tiled", function(g)
  g.start({ config = TILED })
  g.keys("f5")
  g.settle()
  t.eq(#tiled_ids(g), 1)
  t.eq(floating_ids(g), { 1 })
end)

t.case("a tiled window stays tiled", function(g)
  g.start({ config = CONFIG })
  g.client("gband.action.toggle_window_floating({ window = 1 })")
  g.settle()
  t.eq(tiled_ids(g), { 1 })
  g.client("gband.window.rename(1, 'notes')")
  g.settle()
  g.client("gband.action.grow_column_width({ window = 1 })")
  g.settle()
  t.eq(tiled_ids(g), { 1 })
end)

t.case("new windows cascade", function(g)
  g.start({ config = CONFIG })
  prompts(g, 1)
  t.eq(box(g, 1), { 40, 20, 20, 2 })
  g.keys("ctrl+space")
  g.settle()
  g.keys("enter")
  g.settle()
  g.keys("ctrl+space")
  g.settle()
  g.keys("enter")
  g.settle()
  t.eq(box(g, 2), { 40, 20, 22, 3 })
  t.eq(box(g, 3), { 40, 20, 24, 4 })
  prompts(g, 3)
  g.expect_screenshot("three windows")
end)

t.case("cascade stops at the edge", function(g)
  g.start({ config = CONFIG })
  open_window(g)
  open_window(g)
  t.eq(box(g, 3), { 40, 20, 24, 4 })
  g.keys("ctrl+space")
  g.settle()
  g.keys("enter")
  g.settle()
  t.eq(box(g, 4), { 40, 20, 20, 2 })
end)

t.case("buttons on a floating window, restyled, minimized and closed", function(g)
  scene(g)
  t.eq(cells(g.screen().row(4), 49, 11), "[_][□][X]─╮")
  prompts(g, 2)
  g.expect_screenshot("buttons")

  g.client('gband.hl.set("DesktopClose", { fg = 2 })')
  g.settle()
  t.eq(g.screen().cell(4, 56).fg, 2)

  click(g, 50, 4)
  t.ok(minimized(g, 1))
  t.eq(box(g, 1), { 40, 12, 20, 4 })
  t.eq(g.screen().row(4), "")

  g.client("gband.window.focus(1)")
  g.settle()
  click(g, 56, 4)
  t.ok(not in_layout(g, 1))
end)

t.case("no buttons on a tiled window", function(g)
  g.start({ config = TILED })
  g.keys("f5")
  g.settle()
  local row = g.screen().row(0)
  t.eq(row:find("[", 1, true), nil)
  t.ok(row:find("╭", 1, true))
  prompts(g, 1)
  g.settle()
  g.expect_screenshot("tile")
end)

t.case("narrow box", function(g)
  g.start({ config = CONFIG })
  place(g, 1, 0.15, 6, 10, 4)
  t.eq(box(g, 1), { 12, 6, 10, 4 })
  t.eq(cells(g.screen().row(4), 10, 12), "╭──────────╮")
  g.expect_screenshot("narrow", { styles = false })
end)

t.case("close button style", function(g)
  scene(g, { theme = "default" })
  local minimize = g.screen().cell(4, 49)
  t.eq(minimize.char, "[")
  t.ok(minimize.bold)
  for col = 55, 57 do
    local cell = g.screen().cell(4, col)
    t.eq({ cell.fg, cell.bg, cell.bold }, { minimize.fg, minimize.bg, minimize.bold })
    t.ok(cell.fg ~= 1)
  end
end)

t.case("shortcuts follow the key list's style", function(g)
  g.start({ config = CONFIG })
  prompts(g, 1)
  g.client('gband.hl.set("KeyListKey", { fg = 4, bold = true })')
  local list = open_list(g)
  t.eq(g.client('return gband.hl.get("DesktopShortcut", { resolve = true })'), { fg = 4, bold = true })
  local cell = g.screen().cell(list.row + 2, list.left + list.col + 2)
  t.eq(cell.char, "s")
  t.eq(cell.fg, 4)
  t.ok(cell.bold)
end)

t.case("left press on a border", function(g)
  scene(g)
  g.mouse("press", "left", 56, 4)
  g.mouse("drag", "left", 40, 10)
  g.mouse("release", "left", 40, 10)
  g.settle()
  t.eq(box(g, 1), { 40, 12, 20, 4 })

  drag(g, 21, 4, 16, 2)
  t.eq(box(g, 1), { 45, 14, 15, 2 })

  place(g, 1, 0.5, 12, 20, 4)
  drag(g, 58, 15, 62, 18)
  t.eq(box(g, 1), { 44, 15, 20, 4 })

  place(g, 1, 0.5, 12, 20, 4)
  drag(g, 22, 15, 18, 18)
  t.eq(box(g, 1), { 40, 15, 20, 4 })

  place(g, 1, 0.5, 12, 20, 4)
  drag(g, 20, 10, 15, 10)
  t.eq(box(g, 1), { 45, 12, 15, 4 })

  place(g, 1, 0.5, 12, 20, 4)
  g.client("gband.window.focus(2)")
  g.settle()
  drag(g, 30, 4, 35, 6)
  t.eq(box(g, 1), { 40, 12, 25, 6 })
  t.eq(focused(g), 1)

  place(g, 1, 0.5, 12, 20, 4)
  drag(g, 30, 8, 36, 8, "alt")
  t.eq(box(g, 1), { 40, 12, 26, 4 })
end)

t.case("content press reaches the program", function(g)
  scene(g, { parked = false })
  g.client([[gband.window.send_text(1, "printf '\\033[?1000h\\033[?1006h'\n")]])
  g.settle()
  g.client([[gband.window.send_text(1, "cat -v\n")]])
  g.wait_text("$ cat -v")
  g.settle()
  click(g, 25, 7)
  g.wait_text("^[[<0;5;3M^[[<0;5;3m")
  t.eq(focused(g), 1)
  t.eq(box(g, 1), { 40, 12, 20, 4 })
end)

t.case("content presses select, close a menu and paste", function(g)
  scene(g)
  g.client("gband.window.focus(2)")
  g.settle()
  g.client([[gband.window.send_text(1, "printf 'hello world\\n'\n")]])
  g.wait(function(screen)
    return screen.row(6):find("│hello world")
  end)
  drag(g, 22, 6, 30, 6)
  t.eq(focused(g), 1)
  t.eq(g.clipboard(), { "ello worl" })
  t.eq(box(g, 1), { 40, 12, 20, 4 })

  g.client("gband.window.focus(2)")
  g.settle()
  open_list(g)
  t.ok(plugin_window(g))
  drag(g, 22, 6, 27, 6)
  t.eq(g.client("return #gband.win.list()"), 0)
  t.eq(focused(g), 1)
  t.eq(g.clipboard(), { "ello worl", "ello w" })

  g.client("gband.window.focus(2)")
  g.settle()
  click(g, 30, 10, "right")
  g.wait(function(screen)
    return screen.row(7):find("│%$ ello w")
  end)
  t.eq(focused(g), 1)
  t.eq(g.client("return #gband.win.list()"), 0)
end)

t.case("maximize, restore and tile", function(g)
  scene(g)
  g.client("gband.window.focus(2)")
  g.settle()
  click(g, 53, 4)
  t.eq(box(g, 1), { 80, 24, 0, 0 })
  t.eq(focused(g), 1)
  t.eq(cells(g.screen().row(0), 69, 9), "[_][❐][X]")
  prompts(g, 1)
  g.expect_screenshot("maximized")
  click(g, 73, 0)
  t.eq(box(g, 1), { 40, 12, 20, 4 })

  click(g, 30, 4, "right")
  g.keys("j j j enter")
  g.settle()
  t.eq(box(g, 1), { 40, 24, 0, 0 })
  t.eq(focused(g), 1)
  click(g, 33, 0)
  t.eq(box(g, 1), { 80, 24, 0, 0 })
  click(g, 73, 0)
  t.eq(box(g, 1), { 40, 12, 20, 4 })

  click(g, 53, 4)
  t.eq(g.reload(), nil)
  g.settle()
  t.eq(cells(g.screen().row(0), 69, 9), "[_][❐][X]")
  click(g, 73, 0)
  t.eq(box(g, 1), { 40, 12, 20, 6 })

  click(g, 53, 6)
  t.eq(box(g, 1), { 80, 24, 0, 0 })
  g.resize("100x30")
  g.settle()
  t.eq(box(g, 1), { 100, 30, 0, 0 })
  g.resize("80x24")
  g.settle()
  t.eq(box(g, 1), { 80, 24, 0, 0 })
  t.eq(cells(g.screen().row(0), 69, 9), "[_][❐][X]")
end)

t.case("tile right in an odd width", function(g)
  g.start({ files = SAVED })
  prompts(g, 1)
  local before = box(g, 1)
  click(g, 1 + before[3] + 5, before[4], "right")
  g.keys("j j j j enter")
  g.settle()
  t.eq(box(g, 1), { 39, 24, 40, 0 })
end)

t.case("desktop menu keys and wheel", function(g)
  g.start({ config = CONFIG })
  prompts(g, 1)
  g.client("gband.window.rename(1, 'notes')")
  g.settle()
  local list = open_list(g)
  t.eq(entries(list), { "n New window", "s Settings", "m Multi mode", "-", "1 notes" })
  t.eq(list.selected, "n New window")
  g.keys("j j j")
  g.settle()
  t.eq(plugin_window(g).selected, "1 notes")

  g.keys("escape")
  g.settle()
  t.eq(plugin_window(g), nil)
  g.run("echo hi")
  g.wait(function(screen)
    return screen.text():find("│hi")
  end)

end)

t.case("wheel moves the selection of the window menu", function(g)
  scene(g, { parked = false })
  click(g, 30, 4, "right")
  local menu = plugin_window(g)
  t.eq(menu.selected, "Close")
  g.mouse("scroll", "down", menu.col + 5, menu.row + 2)
  g.settle()
  t.eq(plugin_window(g).selected, "Maximize")
end)

t.case("hover only in the window list", function(g)
  scene(g, { parked = false })
  open_list(g)
  t.eq(g.client("return gband.win.info(gband.view().plugin_window).hover"), true)
  g.keys("escape")
  g.settle()
  click(g, 30, 4, "right")
  t.eq(plugin_window(g).lines[1]:match("^┌(%a+)"), "notes")
  t.eq(g.client("return gband.win.info(gband.view().plugin_window).hover"), false)
end)

t.case("most recent first", function(g)
  recent(g)
  t.eq(entries(open_list(g)), { "n New window", "s Settings", "m Multi mode", "-", "1 mail", "2 logs", "3 notes" })
end)

t.case("tenth window and after", function(g)
  g.start({ config = CONFIG })
  prompts(g, 1)
  for _ = 2, 11 do
    open_window(g)
  end
  local names = {}
  for id = 1, 11 do
    names[id] = "w" .. id
  end
  named(g, names)
  focus_in_order(g, { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 })
  open_list(g)
  g.keys("end")
  g.settle()
  local list = plugin_window(g)
  local found = entries(list)
  t.eq({ found[#found - 2], found[#found - 1], found[#found] }, { "9 w3", "0 w2", "  w1" })
  t.eq(list.lines[#list.lines - 1], "│   w1                 │")
end)

t.case("preview by keys, then back to the opening state", function(g)
  recent(g)
  open_list(g)
  g.keys("j j j j")
  g.settle()
  local list = plugin_window(g)
  t.eq(list.selected, "2 logs")
  t.eq(focused(g), 2)
  t.ok(over(g, 2, 3))
  t.ok(last_focus(g, 3) > last_focus(g, 2))
  g.expect_screenshot("logs")

  g.keys("home")
  g.settle()
  t.eq(plugin_window(g).selected, "n New window")
  t.eq(focused(g), 3)
  t.ok(over(g, 3, 2))
end)

t.case("preview and restore a minimized window", function(g)
  recent(g)
  g.client("gband.window.minimize(1)")
  g.settle()
  open_list(g)
  g.keys("j j j j j")
  g.settle()
  t.eq(plugin_window(g).selected, "3 notes (minimized)")
  t.eq(focused(g), 1)
  t.ok(over(g, 1, 2))
  t.ok(over(g, 1, 3))
  t.ok(minimized(g, 1))
  g.expect_screenshot("notes")

  g.keys("enter")
  g.settle()
  t.ok(list_closed(g))
  t.ok(not minimized(g, 1))
  t.eq(focused(g), 1)
  t.ok(over(g, 1, 2))
  t.ok(over(g, 1, 3))
end)

t.case("press on the previewed window keeps it", function(g)
  recent(g)
  g.client("gband.window.minimize(1)")
  g.settle()
  open_list(g)
  g.keys("j j j j j")
  g.settle()
  click(g, 22, 5)
  t.ok(list_closed(g))
  t.eq(focused(g), 1)
  t.ok(not minimized(g, 1))
  t.ok(over(g, 1, 2))
  t.ok(over(g, 1, 3))
end)

t.case("preview a window in another band, then a new window", function(g)
  two_bands(g)
  open_list(g)
  g.keys("end")
  g.settle()
  local list = plugin_window(g)
  t.eq(entries(list), {
    "n New window",
    "s Settings",
    "m Multi mode",
    "-",
    "1 logs        band 1",
    "2 notes       band 1",
    "3 mail        band 2",
  })
  t.eq(list.selected, "3 mail        band 2")
  t.eq(viewed_band(g), 2)
  t.eq(focused(g), 3)
  g.expect_screenshot("mail")

  g.keys("n")
  g.settle()
  t.ok(list_closed(g))
  t.eq(viewed_band(g), 1)
  local band = g.client("local floating = gband.layout().bands[1].floating return { #floating, floating[#floating].id }")
  t.eq(band[1], 3)
  t.eq(focused(g), band[2])
end)

t.case("preview by the pointer", function(g)
  recent(g)
  local list = open_list(g)
  t.eq({ list.left + list.col, list.row, list.width, list.height }, { 28, 7, 24, 9 })
  g.mouse("move", nil, 35, 13)
  g.settle()
  list = plugin_window(g)
  t.eq(list.selected, "2 logs")
  t.eq(focused(g), 2)
  t.ok(over(g, 2, 3))
  g.expect_screenshot("logs")

  g.mouse("move", nil, 35, 11)
  g.settle()
  t.eq(plugin_window(g).selected, "2 logs")
  g.mouse("move", nil, 35, 7)
  g.settle()
  t.eq(plugin_window(g).selected, "2 logs")
end)

t.case("wheel scrolls the list", function(g)
  g.start({ config = CONFIG })
  prompts(g, 1)
  for _ = 2, 20 do
    open_window(g)
  end
  local before = focused(g)
  local list = open_list(g)
  g.mouse("scroll", "down", list.left + list.col + 5, list.row + 3)
  g.settle()
  list = plugin_window(g)
  t.eq(entries(list)[1], "s Settings")
  t.eq(list.selected, nil)
  t.eq(focused(g), before)
  g.keys("enter")
  g.settle()
  t.eq(#floating_ids(g), 21)
end)

t.case("escape returns to the state at opening", function(g)
  g.start({ config = CONFIG })
  prompts(g, 1)
  open_window(g)
  open_window(g)
  g.client("gband.action.focus_band_down()")
  g.settle()
  open_window(g)
  open_window(g)
  named(g, { "notes", "logs", "mail", "top", "low" })
  focus_in_order(g, { 5, 4, 1, 2, 3 })
  g.client("gband.window.minimize(1)")
  g.settle()
  t.eq(viewed_band(g), 1)
  open_list(g)
  g.keys("j j j j j j j")
  g.settle()
  t.match(plugin_window(g).selected, "^5 low +band 2$")
  t.eq(viewed_band(g), 2)
  t.eq(focused(g), 5)
  t.ok(over(g, 5, 4))
  g.expect_screenshot("low")

  g.keys("escape")
  g.settle()
  t.ok(list_closed(g))
  t.eq(viewed_band(g), 1)
  t.eq(focused(g), 3)
  t.ok(over(g, 3, 2))
  t.ok(g.screen().cell(2, 20).char ~= "╭")
  t.ok(minimized(g, 1))
  g.expect_screenshot("back")

  g.client("gband.band.view(gband.layout().bands[2].id)")
  g.settle()
  t.eq(focused(g), 4)
  t.ok(over(g, 4, 5))
end)

t.case("press on empty ribbon cancels", function(g)
  recent(g)
  open_list(g)
  g.keys("j j j j")
  g.settle()
  click(g, 5, 10)
  t.ok(list_closed(g))
  t.eq(focused(g), 3)
  t.ok(over(g, 3, 2))
end)

t.case("press outside closes and takes its effect", function(g)
  recent(g)
  open_list(g)
  click(g, 21, 6)
  t.ok(list_closed(g))
  t.eq(focused(g), 1)
end)

t.case("right press keeps the preview", function(g)
  recent(g)
  open_list(g)
  g.keys("j j j j")
  g.settle()
  click(g, 5, 10, "right")
  local list = plugin_window(g)
  t.eq(list.lines[1]:match("^┌(%a+)"), "windows")
  t.eq({ list.left + list.col, list.row }, { 5, 10 })
  t.eq(focused(g), 2)
  t.eq(most_recent(g), 2)
end)

t.case("closed by other code", function(g)
  recent(g)
  open_list(g)
  g.keys("j j j j")
  g.settle()
  g.client("gband.win.close(gband.view().plugin_window)")
  g.settle()
  t.ok(list_closed(g))
  t.eq(focused(g), 3)
  t.ok(over(g, 3, 2))
end)

t.case("shortcut picks a window", function(g)
  recent(g)
  open_list(g)
  g.keys("2")
  g.settle()
  t.ok(list_closed(g))
  t.eq(focused(g), 2)
  t.ok(over(g, 2, 3))
  t.eq(most_recent(g), 2)
end)

t.case("digits before a prefix binding", function(g)
  recent(g, { config = CONFIG .. '\ngband.keymap.set("prefix", "1", gband.action.open_window)\n' })
  open_list(g)
  g.keys("1")
  g.settle()
  t.ok(list_closed(g))
  t.eq(focused(g), 3)
  t.eq(#floating_ids(g), 3)
end)

t.case("prefix key keeps the preview", function(g)
  recent(g)
  open_list(g)
  g.keys("j j j j")
  g.settle()
  g.keys(":")
  g.settle()
  t.ok(plugin_window(g).lines[1]:find("lua", 1, true))
  t.eq(focused(g), 2)
  t.eq(most_recent(g), 2)
end)

t.case("window closed under the list", function(g)
  recent(g)
  open_list(g)
  g.keys("j j j j")
  g.settle()
  g.client([[gband.window.send_text(1, "exit\n")]])
  g.wait(function()
    return not in_layout(g, 1)
  end)
  g.settle()
  local list = plugin_window(g)
  t.eq(entries(list), { "n New window", "s Settings", "m Multi mode", "-", "1 mail", "2 logs" })
  t.eq(list.selected, "2 logs")
end)

t.case("window focused at opening closes", function(g)
  recent(g)
  open_list(g)
  g.keys("j j j j j")
  g.settle()
  t.eq(focused(g), 1)
  g.client([[gband.window.send_text(3, "exit\n")]])
  g.wait(function()
    return not in_layout(g, 3)
  end)
  g.settle()
  g.keys("escape")
  g.settle()
  t.ok(list_closed(g))
  t.eq(focused(g), 2)
  t.ok(over(g, 2, 1))
end)

t.case("list of two windows, its buttons and a minimized entry", function(g)
  g.start({ files = SAVED })
  prompts(g, 1)
  open_window(g)
  g.client("gband.window.rename(1, 'notes') gband.window.rename(2, 'logs')")
  g.settle()
  g.client('gband.action["desktop.list"]()')
  g.settle()
  local list = plugin_window(g)
  t.eq({ list.width, list.height, list.col, list.row }, { 24, 8, 27, 8 })
  t.eq(list.lines, {
    "┌windows────────[□][X]─┐",
    "│ n New window         │",
    "│ s Settings           │",
    "│ m Multi mode         │",
    "├──────────────────────┤",
    "│ 1 logs               │",
    "│ 2 notes              │",
    "└───────────────? keys─┘",
  })
  t.eq(g.client("return gband.keymap.current_table()"), "root")
  prompts(g, 2)
  g.expect_screenshot("list")

  click(g, 45, 8)
  list = plugin_window(g)
  t.eq({ list.row, list.height }, { 0, 24 })
  t.eq(cells(list.lines[1], 16, 6), "[❐][X]")
  g.expect_screenshot("tall")
  click(g, 45, 0)
  list = plugin_window(g)
  t.eq({ list.row, list.height }, { 8, 8 })

  g.keys("5")
  g.settle()
  t.eq(plugin_window(g).selected, "n New window")
  t.eq(focused(g), 2)

  g.keys("j j j j")
  g.settle()
  t.eq(focused(g), 1)
  click(g, 48, 8)
  t.eq(g.client("return #gband.win.list()"), 0)
  t.eq(focused(g), 2)
  t.ok(over(g, 2, 1))

  g.client("gband.window.minimize(2)")
  g.settle()
  list = open_list(g)
  t.eq(entries(list)[6], "2 logs (minimized)")
  t.ok(g.screen().cell(list.row + 6, list.left + list.col + 4).dim)
  t.ok(not g.screen().cell(list.row + 6, list.left + list.col + 2).dim)
  g.keys("j j j j")
  g.settle()
  list = plugin_window(g)
  t.eq(list.selected, "2 logs (minimized)")
  g.expect_screenshot("minimized entry")
  g.keys("enter")
  g.settle()
  t.ok(not minimized(g, 2))
  t.eq(focused(g), 2)
end)

t.case("pick a window", function(g)
  g.start({ files = SAVED })
  open_window(g)
  t.eq(focused(g), 2)
  open_list(g)
  g.keys("j j j j enter")
  g.settle()
  t.eq(g.client("return #gband.win.list()"), 0)
  t.eq(focused(g), 1)
  t.eq(most_recent(g), 1)
end)

t.case("windows in two bands", function(g)
  g.start({ files = SAVED })
  prompts(g, 1)
  g.client("gband.window.rename(1, 'notes') gband.action.focus_band_down()")
  g.settle()
  g.client("gband.action.open_window({ floating = true })")
  g.settle()
  local band = g.client("return gband.layout().bands[2].id")
  local second = g.client("return gband.layout().bands[2].floating[1].id")
  g.client("gband.window.rename(..., 'logs') gband.band.view(gband.layout().bands[1].id)", second)
  g.settle()
  local list = open_list(g)
  t.eq(entries(list), { "n New window", "s Settings", "m Multi mode", "-", "1 notes       band 1", "2 logs        band 2" })
  g.expect_screenshot("two bands", { styles = false })
  g.keys("j j j j enter")
  g.settle()
  t.eq(g.client("return gband.view().band"), band)
  t.eq(focused(g), second)
end)

t.case("open a new window", function(g)
  g.start({ files = SAVED })
  open_list(g)
  g.keys("enter")
  g.settle()
  local ids = floating_ids(g)
  t.eq(#ids, 2)
  t.eq(focused(g), ids[2])
end)

t.case("open the settings", function(g)
  g.start({ files = SAVED })
  open_list(g)
  g.keys("j enter")
  g.settle()
  local info = plugin_window(g)
  t.ok(info.lines[1]:find("settings", 1, true))
  t.eq(g.client("return #gband.win.list()"), 1)
end)

t.case("settings from a preview", function(g)
  recent(g)
  open_list(g)
  g.keys("j j j j")
  g.settle()
  t.eq(focused(g), 2)
  g.keys("s")
  g.settle()
  t.ok(plugin_window(g).lines[1]:find("settings", 1, true))
  t.eq(focused(g), 3)
  t.ok(over(g, 3, 2))
end)

local function side_by_side(g)
  g.start({ config = CONFIG })
  prompts(g, 1)
  open_window(g)
  named(g, { "notes", "logs" })
  place(g, 1, 0.45, 10, 0, 0)
  place(g, 2, 0.45, 10, 40, 0)
  prompts(g, 2)
end

local function printed(g, text, count)
  g.wait(function(screen)
    local _, found = screen.text():gsub("│" .. text, "")
    return found == count
  end)
end

t.case("multi mode from the list", function(g)
  side_by_side(g)
  g.keys("ctrl+space m")
  g.settle()
  t.ok(list_closed(g))
  t.eq(g.client("return gband.keymap.current_table()"), "root")
  g.run("echo hi")
  printed(g, "hi", 2)
  t.eq(entries(open_list(g)), { "n New window", "s Settings", "m Multi mode (on)", "-", "1 logs", "2 notes" })
end)

t.case("multi mode off from the list", function(g)
  side_by_side(g)
  g.client("gband.action.toggle_multi()")
  g.settle()
  open_list(g)
  g.keys("j j enter")
  g.settle()
  t.eq(g.client("return gband.view().multi"), nil)
  g.run("echo solo")
  printed(g, "solo", 1)
  g.settle()
  printed(g, "solo", 1)
end)

t.case("multi mode from a preview", function(g)
  recent(g)
  open_list(g)
  g.keys("j j j j")
  g.settle()
  t.eq(focused(g), 2)
  g.keys("m")
  g.settle()
  t.ok(list_closed(g))
  t.eq(g.client("return gband.view().multi"), true)
  t.eq(focused(g), 3)
  t.ok(over(g, 3, 2))
end)

t.case("settings from a preview in another band", function(g)
  two_bands(g)
  open_list(g)
  g.keys("end")
  g.settle()
  t.eq(viewed_band(g), 2)
  g.keys("s")
  g.settle()
  t.eq(viewed_band(g), 1)
  t.ok(plugin_window(g).lines[1]:find("settings", 1, true))
  t.eq(g.client("return #gband.win.list()"), 1)
end)

t.case("scroll a long list", function(g)
  g.start({ config = CONFIG })
  for _ = 2, 20 do
    g.client("gband.action.open_window({ floating = true })")
  end
  g.settle()
  g.client([[
    for index, id in ipairs((function()
      local ids = {}
      for _, box in ipairs(gband.layout().bands[1].floating) do ids[#ids + 1] = box.id end
      return ids
    end)()) do
      gband.window.rename(id, "window " .. index)
    end
  ]])
  g.settle()
  open_list(g)
  g.keys("end")
  g.settle()
  local list = plugin_window(g)
  t.eq(list.height, 15)
  t.eq(#entries(list), 13)
  t.eq(list.selected, "  window 1")
  t.eq(entries(list)[13], "  window 1")
  g.expect_screenshot("end", { styles = false })
end)

t.case("lua prompt from the list, then the leader over it", function(g)
  g.start({ config = CONFIG })
  open_list(g)
  g.keys(":")
  g.settle()
  t.eq(g.client("return #gband.win.list()"), 1)
  t.ok(plugin_window(g).lines[1]:find("lua", 1, true))
  g.keys("ctrl+space")
  g.settle()
  t.eq(plugin_window(g).lines[1]:match("^┌(%a+)"), "windows")
end)

t.case("key list from the list", function(g)
  g.start({ config = CONFIG })
  open_list(g)
  g.keys("?")
  g.settle()
  local info = plugin_window(g)
  t.ok(info.lines[1]:find("prefix keys", 1, true))
  local keys = {}
  for index = 2, #info.lines - 1 do
    keys[#keys + 1] = info.lines[index]:match("^│(%S+)")
  end
  t.eq(keys, { "n", "?", ":", "N", "s", "!", "D", "C-space" })
end)

t.case("same list from a right press", function(g)
  g.start({ config = CONFIG })
  open_window(g)
  g.client("gband.window.rename(1, 'notes') gband.window.rename(2, 'logs')")
  g.settle()
  click(g, 75, 1, "right")
  t.eq(entries(plugin_window(g)), { "n New window", "s Settings", "m Multi mode", "-", "1 logs", "2 notes" })
end)

t.case("window menu and window list at the pointer", function(g)
  scene(g, { parked = false })
  click(g, 30, 4, "right")
  local menu = plugin_window(g)
  t.eq({ menu.width, menu.height, menu.col, menu.row }, { 16, 7, 30, 4 })
  t.eq(focused(g), 1)
  t.eq(menu.lines, {
    "┌notes─────────┐",
    "│ Close        │",
    "│ Maximize     │",
    "│ Minimize     │",
    "│ Tile left    │",
    "│ Tile right   │",
    "└──────────────┘",
  })
  prompts(g, 1)
  g.expect_screenshot("menu")

  click(g, 70, 20, "right")
  t.eq(g.client("return #gband.win.list()"), 1)
  local list = plugin_window(g)
  t.eq(list.lines[1]:match("^┌(%a+)"), "windows")
  t.eq({ list.width, list.height, list.col, list.row }, { 24, 7, 56, 17 })

  g.keys("escape")
  g.settle()
  click(g, 53, 4)
  click(g, 30, 0, "right")
  t.eq(entries(plugin_window(g))[2], "Restore")
  g.expect_screenshot("restore")
end)

t.case("right press on the sidebar", function(g)
  g.start({ files = SAVED })
  click(g, 0, 5, "right")
  t.eq(g.client("return #gband.win.list()"), 0)
end)

t.case("minimize and close from the menu", function(g)
  scene(g)
  click(g, 30, 4, "right")
  g.keys("j j enter")
  g.settle()
  t.ok(minimized(g, 1))
  t.eq(g.screen().row(4), "")

  g.client("gband.window.focus(1)")
  g.settle()
  click(g, 30, 4, "right")
  g.keys("enter")
  g.settle()
  t.eq(g.client("return #gband.win.list()"), 0)
  t.ok(not in_layout(g, 1))
end)

t.case("window closed under the menu", function(g)
  scene(g)
  click(g, 30, 4, "right")
  t.eq(g.client("return #gband.win.list()"), 1)
  g.client([[gband.window.send_text(1, "exit\n")]])
  g.wait(function()
    return not in_layout(g, 1)
  end)
  g.settle()
  t.eq(g.client("return #gband.win.list()"), 0)
end)

t.case("leader opens the list", function(g)
  g.start({ files = SAVED })
  prompts(g, 1)
  local list = open_list(g)
  t.eq(list.lines[1]:match("^┌(%a+)"), "windows")
  t.eq(g.client("return gband.keymap.current_table()"), "root")
  g.keys("j")
  g.settle()
  t.eq(plugin_window(g).selected, "s Settings")
  t.eq(g.screen().row(3):find("j", 1, true), nil)
end)

t.case("table changes of the leader", function(g)
  g.start({
    config = CONFIG .. [[

      tables = {}
      gband.on("KeyTableChanged", function(event)
        tables[#tables + 1] = event.table .. "<" .. event.previous
      end)
    ]],
  })
  g.keys("ctrl+space")
  g.settle()
  t.eq(g.client("return tables"), { "prefix<root", "root<prefix" })
end)

t.case("plain left drag on a title bar", function(g)
  g.start({ files = SAVED })
  local before = box(g, 1)
  drag(g, 1 + before[3] + 5, before[4], 1 + before[3] + 11, before[4])
  local after = box(g, 1)
  t.eq(after[3], before[3] + 6)
  t.eq(g.client("return gband.keymap.current_table()"), "root")
end)
