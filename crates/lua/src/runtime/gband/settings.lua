local core = gband.core

gband.hl.default("SettingsLabel", { dim = true })

local NAME = "^[A-Za-z0-9][A-Za-z0-9_-]*$"
local LABEL_WIDTH = 9
local WIDTH = 31
local THEME, SIDEBAR, KEYS, INTERACTIVE_ON_NEW = 1, 2, 3, 4
local LABELS = { "theme", "sidebar", "keys", "I on new" }
local RELOAD_LABEL = "reload"
local KEY_STYLES = { "modal", "direct", "floating" }
local THEME_FILE = "user/theme.lua"
local SIDEBAR_FILE = "user/sidebar.lua"
local KEYSTYLE_FILE = "user/keystyle.lua"
local INTERACTIVE_ON_NEW_FILE = "user/interactive_on_new.lua"
local RESTORE_FILE = "user/floating_restore.lua"
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

local function forget(file)
  local dir = gband.config_dir
  if type(dir) == "string" then
    os.remove(dir .. "/" .. file)
  end
end

local function serialize(value)
  if type(value) == "table" then
    local parts = {}
    for index, item in ipairs(value) do
      parts[index] = serialize(item)
    end
    local keys = {}
    for key in pairs(value) do
      if type(key) == "string" then
        keys[#keys + 1] = key
      end
    end
    table.sort(keys)
    for _, key in ipairs(keys) do
      parts[#parts + 1] = key .. " = " .. serialize(value[key])
    end
    return "{ " .. table.concat(parts, ", ") .. " }"
  elseif math.type(value) == "float" then
    return string.format("%.17g", value)
  end
  return tostring(value)
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

local function is_index(value)
  return math.type(value) == "integer" and value >= 1
end

local function is_cell(value)
  return math.type(value) == "integer" and value >= 0
end

local function is_size(value)
  return type(value) == "number" and value > 0 and value <= 10000
end

local function is_list(value, valid)
  if type(value) ~= "table" then
    return false
  end
  for _, item in ipairs(value) do
    if not valid(item) then
      return false
    end
  end
  return true
end

local function is_tile(value)
  if type(value) ~= "table" or not is_index(value.id) then
    return false
  end
  if value.rows ~= nil then
    return value.weight == nil and is_index(value.rows)
  end
  return is_size(value.weight)
end

local function is_column(value)
  return type(value) == "table"
    and is_size(value.width)
    and is_boolean(value.full_width)
    and is_boolean(value.tabbed)
    and is_list(value.windows, is_tile)
end

local function is_box(value)
  return type(value) == "table"
    and is_index(value.id)
    and is_cell(value.col)
    and is_cell(value.row)
    and is_size(value.width)
    and is_boolean(value.full_width)
    and is_index(value.rows)
end

local function is_band(value)
  return type(value) == "table"
    and is_index(value.id)
    and is_list(value.columns, is_column)
    and is_list(value.floating, is_box)
end

local function is_record(value)
  return type(value) == "table" and is_list(value.bands, is_band)
end

local function record_of(layout)
  local bands = {}
  for _, band in ipairs(layout.bands) do
    local columns, floating = {}, {}
    for _, column in ipairs(band.columns) do
      local windows = {}
      for _, window in ipairs(column.windows) do
        if window.name then
          windows[#windows + 1] = { id = window.id, rows = window.rows, weight = window.weight }
        end
      end
      if #windows > 0 then
        columns[#columns + 1] = {
          width = column.width,
          full_width = column.full_width,
          tabbed = column.tabbed,
          windows = windows,
        }
      end
    end
    for _, box in ipairs(band.floating) do
      if box.name then
        floating[#floating + 1] = {
          id = box.id,
          col = box.col,
          row = box.row,
          width = box.width,
          full_width = box.full_width,
          rows = box.rows,
        }
      end
    end
    bands[#bands + 1] = { id = band.id, columns = columns, floating = floating }
  end
  return { bands = bands }
end

local function floating_programs(layout)
  local ids, floats = {}, {}
  for _, band in ipairs(layout.bands) do
    for _, box in ipairs(band.floating) do
      if box.name then
        ids[#ids + 1] = box.id
        floats[box.id] = true
      end
    end
  end
  return ids, floats
end

local function tile_all(ids)
  for index = #ids, 1, -1 do
    gband.action.toggle_window_floating({ window = ids[index], floating = false })
  end
end

local function restore_column(column)
  local first = column.windows[1].id
  for index = 2, #column.windows do
    gband.action.consume_or_expel_left({ window = column.windows[index].id })
  end
  gband.window.set_width(first, column.width)
  if column.full_width then
    gband.action.toggle_full_width({ window = first })
  end
  for _, window in ipairs(column.windows) do
    gband.window.set_height(window.id, window.rows and { rows = window.rows } or { weight = window.weight })
  end
  if column.tabbed then
    gband.action.toggle_column_tabbed({ window = first })
  end
end

local function restore_box(box)
  gband.window.set_width(box.id, box.width)
  if box.full_width then
    gband.action.toggle_full_width({ window = box.id })
  end
  gband.window.set_height(box.id, { rows = box.rows })
  gband.window.set_position(box.id, { col = box.col, row = box.row })
end

local function restore(record)
  local ids, floats = floating_programs(gband.layout())
  local recorded, columns, boxes = {}, {}, {}
  for _, band in ipairs(record.bands) do
    for _, column in ipairs(band.columns) do
      local windows = {}
      for _, window in ipairs(column.windows) do
        recorded[window.id] = true
        if floats[window.id] then
          windows[#windows + 1] = window
        end
      end
      if #windows > 0 then
        column.windows = windows
        columns[#columns + 1] = column
      end
    end
    for _, box in ipairs(band.floating) do
      recorded[box.id] = true
      if floats[box.id] then
        boxes[#boxes + 1] = box
      end
    end
  end
  local unrecorded, tiles = {}, {}
  for _, id in ipairs(ids) do
    if not recorded[id] then
      unrecorded[#unrecorded + 1] = id
    end
  end
  for _, column in ipairs(columns) do
    for _, window in ipairs(column.windows) do
      tiles[#tiles + 1] = window.id
    end
  end
  tile_all(unrecorded)
  tile_all(tiles)
  for _, column in ipairs(columns) do
    restore_column(column)
  end
  for _, box in ipairs(boxes) do
    restore_box(box)
  end
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

local function reload_line()
  return key_style() == "modal" and INTERACTIVE_ON_NEW + 1 or KEYS + 1
end

local function meaning(line)
  if line == reload_line() then
    return RELOAD_LABEL
  end
  return line
end

local function error_count()
  local count = #gband.errors()
  if count == 0 then
    return "ok"
  elseif count == 1 then
    return "1 error"
  end
  return count .. " errors"
end

local function labelled(label, value)
  return {
    { text = label .. string.rep(" ", LABEL_WIDTH - #label), hl = "SettingsLabel" },
    value,
  }
end

local function lines()
  local values = {
    tostring(gband.colorscheme()),
    sidebar() == false and "off" or "on",
    key_style(),
  }
  if values[KEYS] == "modal" then
    values[INTERACTIVE_ON_NEW] = interactive_on_new() == true and "on" or "off"
  end
  local out = {}
  for index, value in ipairs(values) do
    out[index] = labelled(LABELS[index], value)
  end
  out[#out + 1] = labelled(RELOAD_LABEL, error_count())
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

local function step_keys(direction)
  local style = key_style()
  local at = index_of(KEY_STYLES, style) or 1
  local next_style = KEY_STYLES[(at - 1 + direction) % #KEY_STYLES + 1]
  if style ~= "floating" and next_style == "floating" then
    write(RESTORE_FILE, "return " .. serialize(record_of(gband.layout())) .. "\n")
  elseif style == "floating" and next_style ~= "floating" then
    local record = read(RESTORE_FILE, is_record)
    if record then
      restore(record)
    else
      tile_all((floating_programs(gband.layout())))
    end
    forget(RESTORE_FILE)
  end
  save(KEYS, KEYSTYLE_FILE, 'return "' .. next_style .. '"\n')
end

local function toggle_interactive_on_new()
  local next_value = interactive_on_new() == true and "false" or "true"
  save(INTERACTIVE_ON_NEW, INTERACTIVE_ON_NEW_FILE, "return " .. next_value .. "\n")
end

local function reload()
  core.reopen_settings(reload_line())
  gband.action.reload()
end

local function change(direction)
  return function(win)
    local line = meaning(gband.win.info(win).cursor)
    if line == THEME then
      step_theme(direction)
    elseif line == SIDEBAR then
      toggle_sidebar()
    elseif line == KEYS then
      step_keys(direction)
    elseif line == INTERACTIVE_ON_NEW then
      toggle_interactive_on_new()
    end
  end
end

local open_list

local function enter(win)
  local line = meaning(gband.win.info(win).cursor)
  if line == RELOAD_LABEL then
    reload()
  elseif line == THEME then
    open_list()
  elseif line == SIDEBAR then
    toggle_sidebar()
  elseif line == KEYS then
    step_keys(1)
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
    gband.win.set_lines(window, lines())
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
