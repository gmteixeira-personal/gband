local core = gband.core

gband.hl.default("SettingsLabel", { dim = true })

local NAME = "^[A-Za-z0-9][A-Za-z0-9_-]*$"
local LABEL_WIDTH = 9
local WIDTH = 31
local THEME, SIDEBAR, KEYS, INTERACTIVE_ON_NEW = 1, 2, 3, 4
local LABELS = { "theme", "sidebar", "keys", "I on new" }
local THEME_FILE = "user/theme.lua"
local SIDEBAR_FILE = "user/sidebar.lua"
local KEYSTYLE_FILE = "user/keystyle.lua"
local INTERACTIVE_ON_NEW_FILE = "user/interactive_on_new.lua"
local MOVES = {
  j = 1, down = 1, k = -1, up = -1,
  pagedown = "page", pageup = "page", home = "first", ["end"] = "last",
}

local function read(file, valid)
  local dir = gband.config_dir
  if type(dir) ~= "string" then
    return nil
  end
  local ok, value = pcall(function()
    local chunk = loadfile(dir .. "/" .. file, "t", {})
    if chunk then
      return chunk()
    end
  end)
  if ok and valid(value) then
    return value
  end
  return nil
end

local function write(file, text)
  local dir = gband.config_dir
  if type(dir) ~= "string" then
    error("cannot save " .. file .. ": there is no configuration directory", 0)
  end
  local base = file:match("([^/]+)%.lua$")
  local temporary = string.format("%s/user/.%s.%d.%d", dir, base, os.time(), math.random(1, 1 << 30))
  local handle, reason = io.open(temporary, "w")
  local ok = handle ~= nil
  if handle then
    ok, reason = handle:write(text)
    local closed, close_reason = handle:close()
    if ok and not closed then
      ok, reason = false, close_reason
    end
    if ok then
      ok, reason = os.rename(temporary, dir .. "/" .. file)
    end
    if not ok then
      os.remove(temporary)
    end
  end
  if not ok then
    error("cannot save " .. file .. ": " .. tostring(reason), 0)
  end
end

local function is_name(value)
  return type(value) == "string" and value:match(NAME) ~= nil
end

local function theme()
  return read(THEME_FILE, is_name)
end

local function is_boolean(value)
  return type(value) == "boolean"
end

local function sidebar()
  return read(SIDEBAR_FILE, is_boolean)
end

local function interactive_on_new()
  return read(INTERACTIVE_ON_NEW_FILE, is_boolean)
end

local function themes()
  local names, seen = {}, {}
  for _, name in ipairs(core.bundled_themes) do
    names[#names + 1] = name
    seen[name] = true
  end
  for _, name in ipairs(core.colorschemes()) do
    if is_name(name) and not seen[name] then
      names[#names + 1] = name
      seen[name] = true
    end
  end
  return names
end

local function save(line, file, text)
  core.reopen_settings(line)
  local ok, reason = pcall(write, file, text)
  if not ok then
    core.reopen_settings(nil)
    error(reason, 0)
  end
end

local function save_theme(name)
  save(THEME, THEME_FILE, 'return "' .. name .. '"\n')
end

local window = nil
local list = nil

local function is_open(win)
  if win == nil then
    return false
  end
  for _, id in ipairs(gband.win.list()) do
    if id == win then
      return true
    end
  end
  return false
end

local function key_style()
  return gband.keystyle.saved() or "modal"
end

local function lines()
  local values = {
    tostring(gband.colorscheme()),
    sidebar() == false and "off" or "on",
    key_style(),
  }
  if values[KEYS] == "modal" then
    values[INTERACTIVE_ON_NEW] = interactive_on_new() == false and "off" or "on"
  end
  local out = {}
  for index, value in ipairs(values) do
    local label = LABELS[index]
    out[index] = {
      { text = label .. string.rep(" ", LABEL_WIDTH - #label), hl = "SettingsLabel" },
      value,
    }
  end
  return out
end

local function refresh()
  if is_open(window) then
    gband.win.set_lines(window, lines())
  end
end

local function index_of(names, name)
  for index, candidate in ipairs(names) do
    if candidate == name then
      return index
    end
  end
  return nil
end

local function step_theme(direction)
  local names = themes()
  local previous = gband.colorscheme()
  local at = index_of(names, previous)
  local target
  if at == nil then
    target = direction > 0 and 1 or #names
  else
    target = (at - 1 + direction) % #names + 1
  end
  local name = names[target]
  if not gband.colorscheme(name) then
    return
  end
  local ok, reason = pcall(save_theme, name)
  if not ok then
    gband.colorscheme(previous)
    error(reason, 0)
  end
  refresh()
end

local function toggle_sidebar()
  local next_value = sidebar() == false and "true" or "false"
  save(SIDEBAR, SIDEBAR_FILE, "return " .. next_value .. "\n")
end

local function toggle_keys()
  local next_style = key_style() == "modal" and "direct" or "modal"
  save(KEYS, KEYSTYLE_FILE, 'return "' .. next_style .. '"\n')
end

local function toggle_interactive_on_new()
  local next_value = interactive_on_new() == false and "true" or "false"
  save(INTERACTIVE_ON_NEW, INTERACTIVE_ON_NEW_FILE, "return " .. next_value .. "\n")
end

local function change(direction)
  return function(win)
    local line = gband.win.info(win).cursor
    if line == THEME then
      step_theme(direction)
    elseif line == SIDEBAR then
      toggle_sidebar()
    elseif line == KEYS then
      toggle_keys()
    elseif line == INTERACTIVE_ON_NEW then
      toggle_interactive_on_new()
    end
  end
end

local open_list

local function enter(win)
  local line = gband.win.info(win).cursor
  if line == THEME then
    open_list()
  elseif line == SIDEBAR then
    toggle_sidebar()
  elseif line == KEYS then
    toggle_keys()
  elseif line == INTERACTIVE_ON_NEW then
    toggle_interactive_on_new()
  end
end

local listed = nil

local function close_list()
  local win = list
  list = nil
  if is_open(win) then
    gband.win.close(win)
  end
  if is_open(window) then
    gband.win.focus(window)
  end
end

local function cancel_list()
  if listed and listed.original ~= gband.colorscheme() then
    gband.colorscheme(listed.original)
  end
  close_list()
end

local function preview(win, line)
  local info = gband.win.info(win)
  local target = math.max(1, math.min(line, info.line_count))
  if target == info.cursor then
    return
  end
  gband.win.set_cursor(win, target)
  gband.colorscheme(listed.names[target])
end

local function move_list(name)
  return function(win)
    local info = gband.win.info(win)
    local move = MOVES[name]
    local target
    if move == "page" then
      target = info.cursor + (name == "pageup" and -info.rows or info.rows)
    elseif move == "first" then
      target = 1
    elseif move == "last" then
      target = info.line_count
    else
      target = info.cursor + move
    end
    preview(win, target)
  end
end

local function pick(win)
  local name = listed.names[gband.win.info(win).cursor]
  local chosen = name == gband.colorscheme()
  close_list()
  if not chosen then
    return
  end
  local ok, reason = pcall(save_theme, name)
  if not ok then
    gband.colorscheme(listed.original)
    refresh()
    error(reason, 0)
  end
  refresh()
end

function open_list()
  local names = themes()
  local longest = 0
  for _, name in ipairs(names) do
    longest = math.max(longest, gband.ui.width(name))
  end
  listed = { names = names, original = gband.colorscheme() }
  local keys = { enter = pick, escape = cancel_list, q = cancel_list }
  for name in pairs(MOVES) do
    keys[name] = move_list(name)
  end
  list = gband.win.open({
    title = "theme",
    width = longest + 2,
    height = #names + 2,
    lines = names,
    cursorline = true,
    keys = keys,
    on_mouse = function(win, event)
      if event.kind == "press" and event.button == "left" and event.line then
        preview(win, event.line)
      elseif event.kind == "scroll" and (event.direction == "up" or event.direction == "down") then
        move_list(event.direction)(win)
      end
    end,
    on_close = function(win)
      if list == win then
        list = nil
      end
    end,
  })
  gband.win.set_cursor(list, index_of(names, listed.original) or 1)
end

local function show(line)
  gband.keymap.enter("root")
  if is_open(list) then
    cancel_list()
  end
  if is_open(window) then
    gband.win.focus(window)
  else
    local content = lines()
    window = gband.win.open({
      title = "settings",
      width = WIDTH,
      height = #content + 2,
      lines = content,
      cursorline = true,
      keys = {
        enter = enter,
        h = change(-1),
        left = change(-1),
        l = change(1),
        right = change(1),
      },
      on_close = function(win)
        if window == win then
          window = nil
        end
      end,
    })
  end
  if line then
    gband.win.set_cursor(window, line)
  end
end

core.provide("settings", { open = show })

gband.settings = {
  open = function()
    if core.loading() then
      error("gband.settings.open cannot be called while the configuration loads", 2)
    end
    show(nil)
  end,
  theme = theme,
  sidebar = sidebar,
  interactive_on_new = interactive_on_new,
  themes = themes,
}

return { read = read, write = write }
