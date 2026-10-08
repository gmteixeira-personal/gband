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

t.case("buttons on a floating window", function(g)
  scene(g)
  t.eq(cells(g.screen().row(4), 49, 11), "[_][□][X]─╮")
  prompts(g, 2)
  g.expect_screenshot("buttons")
end)

t.case("restore glyph while maximized", function(g)
  scene(g)
  click(g, 53, 4)
  t.eq(cells(g.screen().row(0), 69, 9), "[_][❐][X]")
  prompts(g, 1)
  g.expect_screenshot("maximized")
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
  local cell = g.screen().cell(4, 56)
  t.eq(cell.char, "X")
  t.eq(cell.fg, 1)
  t.ok(cell.bold)
end)

t.case("restyle the close button", function(g)
  scene(g)
  g.client('gband.hl.set("DesktopClose", { fg = 2 })')
  g.settle()
  t.eq(g.screen().cell(4, 56).fg, 2)
end)

t.case("close by the button", function(g)
  scene(g)
  click(g, 56, 4)
  t.ok(not in_layout(g, 1))
end)

t.case("release off the button", function(g)
  scene(g)
  g.mouse("press", "left", 56, 4)
  g.mouse("drag", "left", 40, 10)
  g.mouse("release", "left", 40, 10)
  g.settle()
  t.eq(box(g, 1), { 40, 12, 20, 4 })
end)

t.case("minimize by the button", function(g)
  scene(g)
  click(g, 50, 4)
  t.ok(minimized(g, 1))
  t.eq(box(g, 1), { 40, 12, 20, 4 })
  t.eq(g.screen().row(4), "")
end)

t.case("maximize by the button", function(g)
  scene(g)
  g.client("gband.window.focus(2)")
  g.settle()
  click(g, 53, 4)
  t.eq(box(g, 1), { 80, 24, 0, 0 })
  t.eq(focused(g), 1)
end)

t.case("resize the top-left corner", function(g)
  scene(g)
  drag(g, 21, 4, 16, 2)
  t.eq(box(g, 1), { 45, 14, 15, 2 })
end)

t.case("cell beside the bottom-right corner", function(g)
  scene(g)
  drag(g, 58, 15, 62, 18)
  t.eq(box(g, 1), { 44, 15, 20, 4 })
end)

t.case("bottom edge near a corner", function(g)
  scene(g)
  drag(g, 22, 15, 18, 18)
  t.eq(box(g, 1), { 40, 15, 20, 4 })
end)

t.case("left edge", function(g)
  scene(g)
  drag(g, 20, 10, 15, 10)
  t.eq(box(g, 1), { 45, 12, 15, 4 })
end)

t.case("move by the title bar", function(g)
  scene(g)
  g.client("gband.window.focus(2)")
  g.settle()
  drag(g, 30, 4, 35, 6)
  t.eq(box(g, 1), { 40, 12, 25, 6 })
  t.eq(focused(g), 1)
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

t.case("content press selects", function(g)
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
end)

t.case("alt drag still moves a window", function(g)
  scene(g)
  drag(g, 30, 8, 36, 8, "alt")
  t.eq(box(g, 1), { 40, 12, 26, 4 })
end)

t.case("maximize and restore", function(g)
  scene(g)
  click(g, 53, 4)
  t.eq(box(g, 1), { 80, 24, 0, 0 })
  click(g, 73, 0)
  t.eq(box(g, 1), { 40, 12, 20, 4 })
end)

t.case("tile left", function(g)
  scene(g)
  click(g, 30, 4, "right")
  g.keys("j j j enter")
  g.settle()
  t.eq(box(g, 1), { 40, 24, 0, 0 })
  t.eq(focused(g), 1)
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

t.case("restore goes back past a tile", function(g)
  scene(g)
  click(g, 30, 4, "right")
  g.keys("j j j enter")
  g.settle()
  click(g, 33, 0)
  t.eq(box(g, 1), { 80, 24, 0, 0 })
  click(g, 73, 0)
  t.eq(box(g, 1), { 40, 12, 20, 4 })
end)

t.case("restore after a reload", function(g)
  scene(g)
  click(g, 53, 4)
  t.eq(g.reload(), nil)
  g.settle()
  t.eq(cells(g.screen().row(0), 69, 9), "[_][❐][X]")
  click(g, 73, 0)
  t.eq(box(g, 1), { 40, 12, 20, 6 })
end)

t.case("terminal grows while maximized", function(g)
  scene(g)
  click(g, 53, 4)
  g.resize("100x30")
  g.settle()
  t.eq(box(g, 1), { 100, 30, 0, 0 })
end)

t.case("smaller screen area keeps the state", function(g)
  scene(g, { size = "100x30" })
  click(g, 50 + 13, 4)
  t.eq(box(g, 1), { 100, 30, 0, 0 })
  g.resize("80x24")
  g.settle()
  t.eq(box(g, 1), { 80, 24, 0, 0 })
  t.eq(cells(g.screen().row(0), 69, 9), "[_][❐][X]")
end)

t.case("move past the separator", function(g)
  g.start({ config = CONFIG })
  g.client("gband.window.rename(1, 'notes')")
  g.settle()
  local list = open_list(g)
  t.eq(entries(list), { "New window", "Settings", "-", "notes" })
  t.eq(list.selected, "New window")
  g.keys("j j")
  g.settle()
  t.eq(plugin_window(g).selected, "notes")
end)

t.case("escape closes", function(g)
  g.start({ config = CONFIG })
  prompts(g, 1)
  open_list(g)
  g.keys("escape")
  g.settle()
  t.eq(plugin_window(g), nil)
  g.run("echo hi")
  g.wait(function(screen)
    return screen.text():find("│hi")
  end)
end)

t.case("press outside closes and takes its effect", function(g)
  scene(g)
  g.client("gband.window.focus(2)")
  g.client([[gband.window.send_text(1, "printf 'hello world\\n'\n")]])
  g.wait(function(screen)
    return screen.row(6):find("│hello world")
  end)
  open_list(g)
  t.ok(plugin_window(g))
  g.mouse("press", "left", 22, 6)
  g.mouse("drag", "left", 27, 6)
  g.mouse("release", "left", 27, 6)
  g.settle()
  t.eq(g.client("return #gband.win.list()"), 0)
  t.eq(focused(g), 1)
  t.eq(g.clipboard(), { "ello w" })
end)

t.case("wheel moves the selection", function(g)
  g.start({ config = CONFIG })
  local list = open_list(g)
  g.mouse("scroll", "down", list.col + 5, list.row + 2)
  g.settle()
  t.eq(plugin_window(g).selected, "Settings")
end)

t.case("one menu at a time", function(g)
  scene(g, { parked = false })
  click(g, 30, 4, "right")
  t.eq(plugin_window(g).lines[1]:match("^┌(%a+)"), "notes")
  click(g, 70, 20, "right")
  t.eq(g.client("return #gband.win.list()"), 1)
  t.eq(plugin_window(g).lines[1]:match("^┌(%a+)"), "windows")
end)

t.case("list of two windows", function(g)
  g.start({ files = SAVED })
  prompts(g, 1)
  open_window(g)
  g.client("gband.window.rename(1, 'notes') gband.window.rename(2, 'logs')")
  g.settle()
  g.client('gband.action["desktop.list"]()')
  g.settle()
  local list = plugin_window(g)
  t.eq({ list.width, list.height, list.col, list.row }, { 24, 7, 27, 8 })
  t.eq(list.lines, {
    "┌windows────────[□][X]─┐",
    "│ New window           │",
    "│ Settings             │",
    "├──────────────────────┤",
    "│ notes                │",
    "│ logs                 │",
    "└───────────────? keys─┘",
  })
  t.eq(g.client("return gband.keymap.current_table()"), "root")
  prompts(g, 2)
  g.expect_screenshot("list")
end)

t.case("pick a window", function(g)
  g.start({ files = SAVED })
  open_window(g)
  t.eq(focused(g), 2)
  open_list(g)
  g.keys("j j enter")
  g.settle()
  t.eq(g.client("return #gband.win.list()"), 0)
  t.eq(focused(g), 1)
end)

t.case("restore a minimized window", function(g)
  g.start({ files = SAVED })
  prompts(g, 1)
  open_window(g)
  g.client("gband.window.rename(1, 'notes') gband.window.rename(2, 'logs')")
  g.client("gband.window.minimize(2)")
  g.settle()
  local list = open_list(g)
  t.eq(entries(list)[5], "logs (minimized)")
  t.ok(g.screen().cell(list.row + 5, list.left + list.col + 2).dim)
  g.keys("j j j")
  g.settle()
  list = plugin_window(g)
  t.eq(list.selected, "logs (minimized)")
  g.expect_screenshot("minimized entry")
  g.keys("enter")
  g.settle()
  t.ok(not minimized(g, 2))
  t.eq(focused(g), 2)
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
  t.eq(entries(list), { "New window", "Settings", "-", "1 notes", "2 logs" })
  g.expect_screenshot("two bands", { styles = false })
  g.keys("j j j enter")
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
  t.eq(list.selected, "window 20")
  t.eq(entries(list)[13], "window 20")
  g.expect_screenshot("end", { styles = false })
end)

t.case("maximize the list's height", function(g)
  g.start({ files = SAVED })
  prompts(g, 1)
  open_window(g)
  g.client("gband.window.rename(1, 'notes') gband.window.rename(2, 'logs')")
  g.settle()
  g.client('gband.action["desktop.list"]()')
  g.settle()
  click(g, 45, 8)
  local list = plugin_window(g)
  t.eq({ list.row, list.height }, { 0, 24 })
  t.eq(cells(list.lines[1], 16, 6), "[❐][X]")
  prompts(g, 2)
  g.expect_screenshot("tall")
  click(g, 45, 0)
  list = plugin_window(g)
  t.eq({ list.row, list.height }, { 8, 7 })
end)

t.case("close the list by the button", function(g)
  g.start({ files = SAVED })
  open_window(g)
  g.client('gband.action["desktop.list"]()')
  g.settle()
  click(g, 48, 8)
  t.eq(g.client("return #gband.win.list()"), 0)
end)

t.case("lua prompt from the list", function(g)
  g.start({ config = CONFIG })
  open_list(g)
  g.keys(":")
  g.settle()
  t.eq(g.client("return #gband.win.list()"), 1)
  t.ok(plugin_window(g).lines[1]:find("lua", 1, true))
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
  t.eq(keys, { "n", "?", ":", "N", "s", "D", "C-space" })
end)

t.case("same list from a right press", function(g)
  g.start({ config = CONFIG })
  open_window(g)
  g.client("gband.window.rename(1, 'notes') gband.window.rename(2, 'logs')")
  g.settle()
  click(g, 75, 1, "right")
  t.eq(entries(plugin_window(g)), { "New window", "Settings", "-", "notes", "logs" })
end)

t.case("window menu at the pointer", function(g)
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
end)

t.case("window list at the pointer, cut to the ribbon area", function(g)
  scene(g, { parked = false })
  click(g, 70, 20, "right")
  local list = plugin_window(g)
  t.eq({ list.width, list.height, list.col, list.row }, { 24, 6, 56, 18 })
end)

t.case("right press in content pastes", function(g)
  scene(g)
  g.client([[gband.window.send_text(1, "printf 'xyz\\n'\n")]])
  g.wait(function(screen)
    return screen.row(6):find("│xyz")
  end)
  drag(g, 21, 6, 23, 6)
  t.eq(g.clipboard(), { "xyz" })
  g.client("gband.window.focus(2)")
  g.settle()
  click(g, 30, 10, "right")
  g.wait(function(screen)
    return screen.row(7):find("│%$ xyz")
  end)
  t.eq(focused(g), 1)
  t.eq(g.client("return #gband.win.list()"), 0)
end)

t.case("right press on the sidebar", function(g)
  g.start({ files = SAVED })
  click(g, 0, 5, "right")
  t.eq(g.client("return #gband.win.list()"), 0)
end)

t.case("restore entry while maximized", function(g)
  scene(g, { parked = false })
  click(g, 53, 4)
  click(g, 30, 0, "right")
  t.eq(entries(plugin_window(g))[2], "Restore")
  prompts(g, 1)
  g.expect_screenshot("restore")
end)

t.case("close from the menu", function(g)
  scene(g)
  click(g, 30, 4, "right")
  g.keys("enter")
  g.settle()
  t.eq(g.client("return #gband.win.list()"), 0)
  t.ok(not in_layout(g, 1))
end)

t.case("minimize from the menu", function(g)
  scene(g)
  click(g, 30, 4, "right")
  g.keys("j j enter")
  g.settle()
  t.ok(minimized(g, 1))
  t.eq(g.screen().row(4), "")
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
  t.eq(plugin_window(g).selected, "Settings")
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

t.case("leader over the lua prompt", function(g)
  g.start({ config = CONFIG })
  open_list(g)
  g.keys(":")
  g.settle()
  t.ok(plugin_window(g).lines[1]:find("lua", 1, true))
  g.keys("ctrl+space")
  g.settle()
  t.eq(plugin_window(g).lines[1]:match("^┌(%a+)"), "windows")
end)

t.case("plain left drag on a title bar", function(g)
  g.start({ files = SAVED })
  local before = box(g, 1)
  drag(g, 1 + before[3] + 5, before[4], 1 + before[3] + 11, before[4])
  local after = box(g, 1)
  t.eq(after[3], before[3] + 6)
  t.eq(g.client("return gband.keymap.current_table()"), "root")
end)
