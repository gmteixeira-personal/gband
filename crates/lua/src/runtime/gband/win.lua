local core = gband.core
local hl = require("gband.hl")

gband.hl.default("PluginWindow", {})
gband.hl.default("PluginWindowBorder", { fg = 8 })
gband.hl.default("PluginWindowTitle", { bold = true })
gband.hl.default("PluginWindowCursorLine", { reverse = true })

local COMMON = {
  kind = true, lines = true, focus = true, cursorline = true, hover = true,
  keys = true, on_input = true, on_close = true, on_resize = true, on_mouse = true,
}
local FLOATING = { row = true, col = true, width = true, height = true, border = true, title = true }
local TILED = { band = true, after = true, column_width = true }

local wins = {}
local dirty = {}
local focused_float = nil
local held = nil
local stack = 0

local function clean(text)
  return (text:gsub("[%z\1-\31\127]", ""):gsub("\194[\128-\159]", ""))
end

local function valid_group(name)
  return type(name) == "string" and name:match("^[A-Za-z][A-Za-z0-9_.]*$") ~= nil
end

local function is_integer(value)
  return math.type(value) == "integer" or (math.type(value) == "float" and value % 1 == 0)
end

local function touch(win)
  dirty[win.id] = true
end

local function require_dispatch(name)
  if not core.dispatching() then
    error(name .. " can only be called inside a binding function or another callback", 3)
  end
end

local function require_loaded(name)
  if core.loading() then
    error(name .. " cannot be called while the configuration loads", 3)
  end
end

local function lookup(id, name)
  local win = math.type(id) and wins[id] or nil
  if not win then
    error(name .. ": no plugin window " .. tostring(id) .. " is open", 3)
  end
  return win
end

local function normalise_lines(lines)
  if type(lines) ~= "table" then
    return nil, "`lines` must be a list of lines"
  end
  local result = {}
  for index = 1, #lines do
    local line = lines[index]
    local spans = {}
    if type(line) == "string" then
      spans[1] = { text = clean(line), hl = "PluginWindow" }
    elseif type(line) == "table" then
      for _, span in ipairs(line) do
        if type(span) == "string" then
          spans[#spans + 1] = { text = clean(span), hl = "PluginWindow" }
        elseif type(span) == "table" and type(span.text) == "string"
          and (span.hl == nil or valid_group(span.hl)) then
          spans[#spans + 1] = { text = clean(span.text), hl = span.hl or "PluginWindow" }
        else
          return nil, "line " .. index .. " holds an invalid span"
        end
      end
    else
      return nil, "line " .. index .. " must be a string or a list of spans"
    end
    result[index] = spans
  end
  return result
end

local function ribbon()
  local state = core.state()
  return state.ribbon.cols, state.ribbon.rows
end

local function place(win)
  local cols, rows = ribbon()
  local width = math.min(win.width or math.max(1, cols // 2), cols)
  local height = math.min(win.height or math.max(1, rows // 2), rows)
  local col = win.col == "center" and (cols - width) // 2 or math.min(win.col, cols - width)
  local row = win.row == "center" and (rows - height) // 2 or math.min(win.row, rows - height)
  win.placed = { row = row, col = col, width = width, height = height }
end

local function content_size(win)
  if win.kind == "floating" then
    local placed = win.placed
    if win.border then
      return math.max(0, placed.width - 2), math.max(0, placed.height - 2)
    end
    return placed.width, placed.height
  end
  return win.cols, win.rows
end

local function content_height(win)
  local _, rows = content_size(win)
  return math.max(rows or 1, 1)
end

local function clamp(win)
  local count = #win.lines
  local height = content_height(win)
  win.cursor = math.max(1, math.min(win.cursor, math.max(1, count)))
  if win.cursorline then
    if win.cursor < win.top then
      win.top = win.cursor
    elseif win.cursor > win.top + height - 1 then
      win.top = win.cursor - height + 1
    end
  end
  win.top = math.max(1, math.min(win.top, math.max(1, count - height + 1)))
end

local function over(style, extra)
  local result = {}
  for field, value in pairs(style) do
    result[field] = value
  end
  for field, value in pairs(extra) do
    result[field] = value
  end
  return result
end

local function row_runs(spans, width, base, style)
  local runs = {}
  local used = 0
  local function push(text, run_style)
    if text ~= "" then
      runs[#runs + 1] = { text = text, style = run_style }
    end
  end
  for _, span in ipairs(spans) do
    local pieces = {}
    local full = false
    for char in span.text:gmatch(utf8.charpattern) do
      local cells = gband.ui.width(char)
      if used + cells > width then
        full = true
        break
      end
      pieces[#pieces + 1] = char
      used = used + cells
    end
    push(table.concat(pieces), style(span.hl))
    if full then
      break
    end
  end
  if used < width then
    push(string.rep(" ", width - used), base)
  end
  return runs
end

local function cut(text, width)
  local pieces = {}
  local used = 0
  for char in text:gmatch(utf8.charpattern) do
    local cells = gband.ui.width(char)
    if used + cells > width then
      break
    end
    pieces[#pieces + 1] = char
    used = used + cells
  end
  return table.concat(pieces)
end

local function render(win)
  local cols, rows = content_size(win)
  if cols == nil then
    return
  end
  local base = hl.drawn("PluginWindow")
  local cursor = win.cursorline and hl.drawn("PluginWindowCursorLine") or nil
  local plain, highlighted = {}, {}
  local lines = {}
  for row = 0, rows - 1 do
    local index = win.top + row
    local on_cursor = cursor ~= nil and index == win.cursor and index <= #win.lines
    local cache = on_cursor and highlighted or plain
    local row_base = on_cursor and over(base, cursor) or base
    local function style(group)
      if not cache[group] then
        local merged = over(base, hl.drawn(group))
        cache[group] = on_cursor and over(merged, cursor) or merged
      end
      return cache[group]
    end
    lines[#lines + 1] = row_runs(win.lines[index] or {}, cols, row_base, style)
  end
  if win.kind == "tiled" then
    core.present_window(win.id, {
      kind = "tiled", cols = cols, rows = rows, base = base, lines = lines, hover = win.hover,
    })
    return
  end
  local placed = win.placed
  local border_style = over(base, hl.drawn("PluginWindowBorder"))
  local title = nil
  if win.border and win.title then
    title = cut(clean(win.title), math.max(0, placed.width - 2))
  end
  core.present_window(win.id, {
    kind = "floating",
    row = placed.row,
    col = placed.col,
    width = placed.width,
    height = placed.height,
    border = win.border,
    title = title,
    base = base,
    border_style = border_style,
    title_style = over(border_style, hl.drawn("PluginWindowTitle")),
    lines = lines,
    z = win.z,
    focused = focused_float == win.id,
    hover = win.hover,
  })
end

local function callback(win, field, ...)
  local fn = win[field]
  if fn then
    core.call(win.owner, nil, fn, win.id, ...)
  end
end

local function resized(win, before_cols, before_rows)
  local cols, rows = content_size(win)
  if cols ~= before_cols or rows ~= before_rows then
    clamp(win)
    callback(win, "on_resize", cols, rows)
  end
end

local function unfocus()
  local win = focused_float and wins[focused_float]
  focused_float = nil
  if win then
    touch(win)
  end
end

local function raise(win)
  if focused_float ~= win.id then
    unfocus()
  end
  focused_float = win.id
  stack = stack + 1
  win.z = stack
  touch(win)
end

local function close(win, run_callback, request)
  wins[win.id] = nil
  dirty[win.id] = nil
  if focused_float == win.id then
    focused_float = nil
  end
  core.forget_window(win.id)
  if request and win.kind == "tiled" then
    core.request({ op = "close", id = win.id })
  end
  if run_callback then
    callback(win, "on_close")
  end
end

local function check_boolean(opts, field, default)
  local value = opts[field]
  if value == nil then
    return default
  end
  if type(value) ~= "boolean" then
    error("`" .. field .. "` must be a boolean", 3)
  end
  return value
end

local function check_border(opts, default)
  local value = opts.border
  if value == nil or type(value) == "boolean" then
    return check_boolean(opts, "border", default)
  end
  if type(value) ~= "table" then
    error("`border` must be a boolean or a border table", 3)
  end
  local border, reason = core.border(value)
  if not border then
    error("`border`: " .. reason, 3)
  end
  return border
end

local function check_function(opts, field)
  local value = opts[field]
  if value ~= nil and type(value) ~= "function" then
    error("`" .. field .. "` must be a function", 3)
  end
  return value
end

local function check_position(opts, field)
  local value = opts[field]
  if value == nil or value == "center" then
    return "center"
  end
  if type(value) ~= "number" or not is_integer(value) or value < 0 then
    error("`" .. field .. "` must be an integer of at least 0 or \"center\"", 3)
  end
  return math.tointeger(value)
end

local function check_size(opts, field)
  local value = opts[field]
  if value == nil then
    return nil
  end
  if type(value) ~= "number" or not is_integer(value) or value < 1 then
    error("`" .. field .. "` must be an integer of at least 1", 3)
  end
  return math.tointeger(value)
end

local function check_title(opts)
  local value = opts.title
  if value ~= nil and type(value) ~= "string" then
    error("`title` must be a string", 3)
  end
  return value
end

local function check_fields(opts, kind, name)
  for field in pairs(opts) do
    local allowed = COMMON[field] or (kind == "floating" and FLOATING or TILED)[field]
    if not allowed then
      if FLOATING[field] or TILED[field] then
        error(name .. ": `" .. field .. "` does not apply to a " .. kind .. " plugin window", 3)
      end
      error(name .. ": unknown field `" .. tostring(field) .. "`", 3)
    end
  end
end

local function check_keys(opts)
  local keys = {}
  if opts.keys == nil then
    return keys
  end
  if type(opts.keys) ~= "table" then
    error("`keys` must be a table from key names to functions", 3)
  end
  for name, fn in pairs(opts.keys) do
    local canonical = type(name) == "string" and core.parse_key(name) or nil
    if not canonical then
      error("invalid key name `" .. tostring(name) .. "` in `keys`", 3)
    end
    if type(fn) ~= "function" then
      error("the key `" .. name .. "` in `keys` must map to a function", 3)
    end
    keys[canonical] = fn
  end
  return keys
end

local api = {}

function api.open(opts)
  require_dispatch("gband.win.open")
  if opts == nil then
    opts = {}
  end
  if type(opts) ~= "table" then
    error("gband.win.open expects a table of options", 2)
  end
  local kind = opts.kind or "floating"
  local removed = type(kind) == "string" and core.removed('kind = "' .. kind .. '"')
  if removed then
    error(removed, 2)
  end
  if kind ~= "floating" and kind ~= "tiled" then
    error("`kind` must be \"floating\" or \"tiled\"", 2)
  end
  check_fields(opts, kind, "gband.win.open")
  local lines, reason = normalise_lines(opts.lines or {})
  if not lines then
    error(reason, 2)
  end
  local win = {
    kind = kind,
    owner = core.owner(),
    lines = lines,
    top = 1,
    cursor = 1,
    cursorline = check_boolean(opts, "cursorline", false),
    hover = check_boolean(opts, "hover", false),
    keys = check_keys(opts),
    on_input = check_function(opts, "on_input"),
    on_close = check_function(opts, "on_close"),
    on_resize = check_function(opts, "on_resize"),
    on_mouse = check_function(opts, "on_mouse"),
  }
  local focus = check_boolean(opts, "focus", true)
  local request = nil
  if kind == "floating" then
    win.row = check_position(opts, "row")
    win.col = check_position(opts, "col")
    win.width = check_size(opts, "width")
    win.height = check_size(opts, "height")
    win.border = check_border(opts, true)
    win.title = check_title(opts)
  else
    local ok, band, after = core.open_target(opts.band, opts.after)
    if not ok then
      error("gband.win.open: " .. band, 2)
    end
    request = { op = "open", band = band, after = after, focus = focus }
    if opts.column_width ~= nil then
      local num, den = core.width(opts.column_width)
      if not num then
        error("`column_width`: " .. den, 2)
      end
      request.num, request.den = num, den
    end
  end
  win.id = core.next_window()
  wins[win.id] = win
  if kind == "floating" then
    place(win)
    clamp(win)
    stack = stack + 1
    win.z = stack
    if focus then
      raise(win)
    end
    touch(win)
  else
    request.id = win.id
    core.request(request)
    clamp(win)
  end
  return win.id
end

function api.close(id)
  require_dispatch("gband.win.close")
  local win = math.type(id) and wins[id] or nil
  if win then
    close(win, true, true)
  end
end

function api.set_lines(id, lines)
  require_dispatch("gband.win.set_lines")
  local win = lookup(id, "gband.win.set_lines")
  local normalised, reason = normalise_lines(lines)
  if not normalised then
    error(reason, 2)
  end
  win.lines = normalised
  clamp(win)
  touch(win)
end

function api.scroll(id, count)
  require_dispatch("gband.win.scroll")
  local win = lookup(id, "gband.win.scroll")
  if type(count) ~= "number" or not is_integer(count) then
    error("gband.win.scroll expects a count as an integer", 2)
  end
  win.top = win.top + math.tointeger(count)
  clamp(win)
  touch(win)
end

function api.set_cursor(id, line)
  require_dispatch("gband.win.set_cursor")
  local win = lookup(id, "gband.win.set_cursor")
  if type(line) ~= "number" or not is_integer(line) then
    error("gband.win.set_cursor expects a line as an integer", 2)
  end
  win.cursor = math.tointeger(line)
  clamp(win)
  touch(win)
end

function api.focus(id)
  require_dispatch("gband.win.focus")
  local win = lookup(id, "gband.win.focus")
  if win.kind == "floating" then
    raise(win)
    return
  end
  unfocus()
  if win.window then
    core.focus_window(win.window)
  end
end

function api.set_config(id, config)
  require_dispatch("gband.win.set_config")
  local win = lookup(id, "gband.win.set_config")
  if win.kind ~= "floating" then
    error("gband.win.set_config applies only to floating plugin windows", 2)
  end
  if type(config) ~= "table" then
    error("gband.win.set_config expects a table", 2)
  end
  for field in pairs(config) do
    if not FLOATING[field] then
      error("gband.win.set_config: unknown field `" .. tostring(field) .. "`", 2)
    end
  end
  local changes = {
    row = config.row ~= nil and check_position(config, "row") or nil,
    col = config.col ~= nil and check_position(config, "col") or nil,
    width = check_size(config, "width"),
    height = check_size(config, "height"),
    border = check_border(config, nil),
    title = check_title(config),
  }
  for field, value in pairs(changes) do
    win[field] = value
  end
  local cols, rows = content_size(win)
  place(win)
  resized(win, cols, rows)
  touch(win)
end

local function is_focused(win)
  if win.kind == "floating" then
    return focused_float == win.id
  end
  return focused_float == nil and win.window ~= nil and core.state().window == win.window
end

function api.info(id)
  require_loaded("gband.win.info")
  local win = lookup(id, "gband.win.info")
  local cols, rows = content_size(win)
  local info = {
    id = win.id,
    kind = win.kind,
    focused = is_focused(win),
    window = win.window,
    top = win.top,
    cursor = win.cursor,
    line_count = #win.lines,
    cols = cols,
    rows = rows,
    hover = win.hover,
  }
  if win.kind == "floating" then
    local placed = win.placed
    info.row, info.col, info.width, info.height = placed.row, placed.col, placed.width, placed.height
  end
  return info
end

function api.list()
  require_loaded("gband.win.list")
  local ids = {}
  for id in pairs(wins) do
    ids[#ids + 1] = id
  end
  table.sort(ids)
  return ids
end

local LINE_STEPS = { up = -1, k = -1, down = 1, j = 1 }

local function move(win, name)
  local height = content_height(win)
  local count = #win.lines
  local step = LINE_STEPS[name]
  if step then
    if win.cursorline then
      win.cursor = win.cursor + step
    else
      win.top = win.top + step
    end
  elseif name == "pageup" or name == "pagedown" then
    local page = name == "pageup" and -height or height
    win.top = win.top + page
    if win.cursorline then
      win.cursor = win.cursor + page
    end
  elseif name == "home" then
    win.top = 1
    if win.cursorline then
      win.cursor = 1
    end
  elseif name == "end" then
    win.top = count
    if win.cursorline then
      win.cursor = count
    end
  else
    return false
  end
  clamp(win)
  touch(win)
  return true
end

local hooks = {}

local function hold(id)
  if focused_float == id then
    held = id
  end
end

function hooks.key(id, name, text)
  local win = wins[id]
  if not win then
    return
  end
  local fn = win.keys[name]
  if fn then
    hold(id)
    core.call(win.owner, nil, fn, id)
    return
  end
  if text and win.on_input then
    hold(id)
    callback(win, "on_input", text)
    return
  end
  if move(win, name) then
    return
  end
  if (name == "escape" or name == "q") and win.kind == "floating" then
    close(win, true, true)
  end
end

local WHEEL_STEPS = { up = "up", down = "down" }

local function focus_from_mouse(win)
  if win.kind == "floating" then
    raise(win)
    return
  end
  unfocus()
  if win.window then
    core.focus_window(win.window)
  end
end

function hooks.mouse(id, event)
  local win = wins[id]
  if not win then
    return
  end
  if event.kind == "press" then
    focus_from_mouse(win)
  end
  if event.content_row ~= nil and win.top + event.content_row <= #win.lines then
    event.line = win.top + event.content_row
  end
  if win.on_mouse then
    if win.kind == "floating" and (win.hover or event.kind ~= "scroll") then
      hold(id)
    end
    core.call(win.owner, nil, win.on_mouse, id, event)
    return
  end
  if event.kind == "scroll" then
    local name = WHEEL_STEPS[event.direction]
    if name then
      move(win, name)
    end
  elseif event.kind == "press" and event.button == "left" and win.cursorline and event.line then
    win.cursor = event.line
    clamp(win)
    touch(win)
  end
end

function hooks.set_box(id, col, row, width, height)
  local win = wins[id]
  if not win or win.kind ~= "floating" then
    return
  end
  local cols, rows = content_size(win)
  win.col, win.row, win.width, win.height = col, row, width, height
  place(win)
  resized(win, cols, rows)
  touch(win)
end

function hooks.raise(id)
  local win = wins[id]
  if win then
    focus_from_mouse(win)
  end
end

function hooks.unfocus()
  unfocus()
end

function hooks.paste(id, text)
  local win = wins[id]
  if win and win.on_input then
    hold(id)
    callback(win, "on_input", text)
  end
end

function hooks.close_focused()
  local win = focused_float and wins[focused_float]
  if not win then
    return false
  end
  close(win, true, true)
  return true
end

function hooks.release()
  held = nil
end

function hooks.opened(id, window)
  local win = wins[id]
  if not win then
    return
  end
  if window == nil then
    close(win, true, false)
    return
  end
  win.window = window
end

function hooks.window_resized(id, cols, rows)
  local win = wins[id]
  if not win then
    return
  end
  local before_cols, before_rows = win.cols, win.rows
  win.cols, win.rows = cols, rows
  resized(win, before_cols, before_rows)
  touch(win)
end

function hooks.window_closed(id)
  local win = wins[id]
  if win then
    close(win, true, false)
  end
end

function hooks.ribbon_resized()
  for _, win in pairs(wins) do
    if win.kind == "floating" then
      local cols, rows = content_size(win)
      place(win)
      resized(win, cols, rows)
      touch(win)
    end
  end
end

function hooks.focused()
  if focused_float then
    return focused_float
  end
  local window = core.state().window
  if window == nil then
    return nil
  end
  return hooks.plugin_window_of(window)
end

function hooks.plugin_window_of(window)
  for id, win in pairs(wins) do
    if win.window == window then
      return id
    end
  end
  return nil
end

function hooks.flush()
  for id, win in pairs(wins) do
    if win.owner and core.failed(win.owner) then
      close(win, false, true)
    end
  end
  local pending = dirty
  dirty = {}
  for id in pairs(pending) do
    local win = wins[id]
    if win then
      render(win)
    end
  end
end

core.provide("windows", hooks)

core.after_event(function(name)
  if name == "FocusChanged" or name == "BandChanged" then
    if held == nil or held ~= focused_float then
      unfocus()
    end
  elseif name == nil or name == "HighlightChanged" or name == "ColorschemeChanged" then
    for _, win in pairs(wins) do
      touch(win)
    end
  end
end)

gband.win = api
