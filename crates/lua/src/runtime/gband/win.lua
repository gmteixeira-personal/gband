local host = ...

gband.hl.default("Window", {})
gband.hl.default("WindowBorder", { fg = 8 })
gband.hl.default("WindowTitle", { bold = true })
gband.hl.default("WindowCursorLine", { reverse = true })

local COMMON = {
  kind = true, lines = true, focus = true, cursorline = true,
  keys = true, on_close = true, on_resize = true,
}
local FLOAT = { row = true, col = true, width = true, height = true, border = true, title = true }
local PANE = { band = true, after = true, column_width = true }

local windows = {}
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

local function touch(window)
  dirty[window.id] = true
end

local function require_dispatch(name)
  if not host.dispatching() then
    error(name .. " can only be called inside a binding function or another callback", 3)
  end
end

local function require_loaded(name)
  if host.loading() then
    error(name .. " cannot be called while the configuration loads", 3)
  end
end

local function lookup(id, name)
  local window = math.type(id) and windows[id] or nil
  if not window then
    error(name .. ": no window " .. tostring(id) .. " is open", 3)
  end
  return window
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
      spans[1] = { text = clean(line), hl = "Window" }
    elseif type(line) == "table" then
      for _, span in ipairs(line) do
        if type(span) == "string" then
          spans[#spans + 1] = { text = clean(span), hl = "Window" }
        elseif type(span) == "table" and type(span.text) == "string"
          and (span.hl == nil or valid_group(span.hl)) then
          spans[#spans + 1] = { text = clean(span.text), hl = span.hl or "Window" }
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
  local state = host.state()
  return state.ribbon.cols, state.ribbon.rows
end

local function place(window)
  local cols, rows = ribbon()
  local width = math.min(window.width or math.max(1, cols // 2), cols)
  local height = math.min(window.height or math.max(1, rows // 2), rows)
  local col = window.col == "center" and (cols - width) // 2 or math.min(window.col, cols - width)
  local row = window.row == "center" and (rows - height) // 2 or math.min(window.row, rows - height)
  window.placed = { row = row, col = col, width = width, height = height }
end

local function content_size(window)
  if window.kind == "float" then
    local placed = window.placed
    if window.border then
      return math.max(0, placed.width - 2), math.max(0, placed.height - 2)
    end
    return placed.width, placed.height
  end
  return window.cols, window.rows
end

local function content_height(window)
  local _, rows = content_size(window)
  return math.max(rows or 1, 1)
end

local function clamp(window)
  local count = #window.lines
  local height = content_height(window)
  window.cursor = math.max(1, math.min(window.cursor, math.max(1, count)))
  if window.cursorline then
    if window.cursor < window.top then
      window.top = window.cursor
    elseif window.cursor > window.top + height - 1 then
      window.top = window.cursor - height + 1
    end
  end
  window.top = math.max(1, math.min(window.top, math.max(1, count - height + 1)))
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

local function render(window)
  local cols, rows = content_size(window)
  if cols == nil then
    return
  end
  local base = host.hl.drawn("Window")
  local cursor = window.cursorline and host.hl.drawn("WindowCursorLine") or nil
  local plain, highlighted = {}, {}
  local lines = {}
  for row = 0, rows - 1 do
    local index = window.top + row
    local on_cursor = cursor ~= nil and index == window.cursor and index <= #window.lines
    local cache = on_cursor and highlighted or plain
    local row_base = on_cursor and over(base, cursor) or base
    local function style(group)
      if not cache[group] then
        local merged = over(base, host.hl.drawn(group))
        cache[group] = on_cursor and over(merged, cursor) or merged
      end
      return cache[group]
    end
    lines[#lines + 1] = row_runs(window.lines[index] or {}, cols, row_base, style)
  end
  if window.kind == "pane" then
    host.present_window(window.id, { kind = "pane", cols = cols, rows = rows, base = base, lines = lines })
    return
  end
  local placed = window.placed
  local border_style = over(base, host.hl.drawn("WindowBorder"))
  local title = nil
  if window.border and window.title then
    title = cut(clean(window.title), math.max(0, placed.width - 2))
  end
  host.present_window(window.id, {
    kind = "float",
    row = placed.row,
    col = placed.col,
    width = placed.width,
    height = placed.height,
    border = window.border,
    title = title,
    base = base,
    border_style = border_style,
    title_style = over(border_style, host.hl.drawn("WindowTitle")),
    lines = lines,
    z = window.z,
    focused = focused_float == window.id,
  })
end

local function callback(window, field, ...)
  local fn = window[field]
  if fn then
    host.call(window.owner, nil, fn, window.id, ...)
  end
end

local function resized(window, before_cols, before_rows)
  local cols, rows = content_size(window)
  if cols ~= before_cols or rows ~= before_rows then
    clamp(window)
    callback(window, "on_resize", cols, rows)
  end
end

local function unfocus()
  local window = focused_float and windows[focused_float]
  focused_float = nil
  if window then
    touch(window)
  end
end

local function raise(window)
  if focused_float ~= window.id then
    unfocus()
  end
  focused_float = window.id
  stack = stack + 1
  window.z = stack
  touch(window)
end

local function close(window, run_callback, request)
  windows[window.id] = nil
  dirty[window.id] = nil
  if focused_float == window.id then
    focused_float = nil
  end
  host.forget_window(window.id)
  if request and window.kind == "pane" then
    host.request({ op = "close", window = window.id })
  end
  if run_callback then
    callback(window, "on_close")
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
    local allowed = COMMON[field] or (kind == "float" and FLOAT or PANE)[field]
    if not allowed then
      if FLOAT[field] or PANE[field] then
        error(name .. ": `" .. field .. "` does not apply to a " .. kind .. " window", 3)
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
    local canonical = type(name) == "string" and host.parse_key(name) or nil
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
  local kind = opts.kind or "float"
  if kind ~= "float" and kind ~= "pane" then
    error("`kind` must be \"float\" or \"pane\"", 2)
  end
  check_fields(opts, kind, "gband.win.open")
  local lines, reason = normalise_lines(opts.lines or {})
  if not lines then
    error(reason, 2)
  end
  local window = {
    kind = kind,
    owner = host.owner(),
    lines = lines,
    top = 1,
    cursor = 1,
    cursorline = check_boolean(opts, "cursorline", false),
    keys = check_keys(opts),
    on_close = check_function(opts, "on_close"),
    on_resize = check_function(opts, "on_resize"),
  }
  local focus = check_boolean(opts, "focus", true)
  local request = nil
  if kind == "float" then
    window.row = check_position(opts, "row")
    window.col = check_position(opts, "col")
    window.width = check_size(opts, "width")
    window.height = check_size(opts, "height")
    window.border = check_boolean(opts, "border", true)
    window.title = check_title(opts)
  else
    local ok, band, after = host.open_target(opts.band, opts.after)
    if not ok then
      error("gband.win.open: " .. band, 2)
    end
    request = { op = "open", band = band, after = after, focus = focus }
    if opts.column_width ~= nil then
      local num, den = host.width(opts.column_width)
      if not num then
        error("`column_width`: " .. den, 2)
      end
      request.num, request.den = num, den
    end
  end
  window.id = host.next_window()
  windows[window.id] = window
  if kind == "float" then
    place(window)
    clamp(window)
    stack = stack + 1
    window.z = stack
    if focus then
      raise(window)
    end
    touch(window)
  else
    request.window = window.id
    host.request(request)
    clamp(window)
  end
  return window.id
end

function api.close(id)
  require_dispatch("gband.win.close")
  local window = math.type(id) and windows[id] or nil
  if window then
    close(window, true, true)
  end
end

function api.set_lines(id, lines)
  require_dispatch("gband.win.set_lines")
  local window = lookup(id, "gband.win.set_lines")
  local normalised, reason = normalise_lines(lines)
  if not normalised then
    error(reason, 2)
  end
  window.lines = normalised
  clamp(window)
  touch(window)
end

function api.scroll(id, count)
  require_dispatch("gband.win.scroll")
  local window = lookup(id, "gband.win.scroll")
  if type(count) ~= "number" or not is_integer(count) then
    error("gband.win.scroll expects a count as an integer", 2)
  end
  window.top = window.top + math.tointeger(count)
  clamp(window)
  touch(window)
end

function api.set_cursor(id, line)
  require_dispatch("gband.win.set_cursor")
  local window = lookup(id, "gband.win.set_cursor")
  if type(line) ~= "number" or not is_integer(line) then
    error("gband.win.set_cursor expects a line as an integer", 2)
  end
  window.cursor = math.tointeger(line)
  clamp(window)
  touch(window)
end

function api.focus(id)
  require_dispatch("gband.win.focus")
  local window = lookup(id, "gband.win.focus")
  if window.kind == "float" then
    raise(window)
    return
  end
  unfocus()
  if window.pane then
    host.focus_pane(window.pane)
  end
end

function api.set_config(id, config)
  require_dispatch("gband.win.set_config")
  local window = lookup(id, "gband.win.set_config")
  if window.kind ~= "float" then
    error("gband.win.set_config applies only to floats", 2)
  end
  if type(config) ~= "table" then
    error("gband.win.set_config expects a table", 2)
  end
  for field in pairs(config) do
    if not FLOAT[field] then
      error("gband.win.set_config: unknown field `" .. tostring(field) .. "`", 2)
    end
  end
  local changes = {
    row = config.row ~= nil and check_position(config, "row") or nil,
    col = config.col ~= nil and check_position(config, "col") or nil,
    width = check_size(config, "width"),
    height = check_size(config, "height"),
    border = check_boolean(config, "border", nil),
    title = check_title(config),
  }
  for field, value in pairs(changes) do
    window[field] = value
  end
  local cols, rows = content_size(window)
  place(window)
  resized(window, cols, rows)
  touch(window)
end

local function is_focused(window)
  if window.kind == "float" then
    return focused_float == window.id
  end
  return focused_float == nil and window.pane ~= nil and host.state().pane == window.pane
end

function api.info(id)
  require_loaded("gband.win.info")
  local window = lookup(id, "gband.win.info")
  local cols, rows = content_size(window)
  local info = {
    id = window.id,
    kind = window.kind,
    focused = is_focused(window),
    pane = window.pane,
    top = window.top,
    cursor = window.cursor,
    line_count = #window.lines,
    cols = cols,
    rows = rows,
  }
  if window.kind == "float" then
    local placed = window.placed
    info.row, info.col, info.width, info.height = placed.row, placed.col, placed.width, placed.height
  end
  return info
end

function api.list()
  require_loaded("gband.win.list")
  local ids = {}
  for id in pairs(windows) do
    ids[#ids + 1] = id
  end
  table.sort(ids)
  return ids
end

local LINE_STEPS = { up = -1, k = -1, down = 1, j = 1 }

local function move(window, name)
  local height = content_height(window)
  local count = #window.lines
  local step = LINE_STEPS[name]
  if step then
    if window.cursorline then
      window.cursor = window.cursor + step
    else
      window.top = window.top + step
    end
  elseif name == "pageup" or name == "pagedown" then
    local page = name == "pageup" and -height or height
    window.top = window.top + page
    if window.cursorline then
      window.cursor = window.cursor + page
    end
  elseif name == "home" then
    window.top = 1
    if window.cursorline then
      window.cursor = 1
    end
  elseif name == "end" then
    window.top = count
    if window.cursorline then
      window.cursor = count
    end
  else
    return false
  end
  clamp(window)
  touch(window)
  return true
end

local hooks = {}

function hooks.key(id, name)
  local window = windows[id]
  if not window then
    return
  end
  local fn = window.keys[name]
  if fn then
    if focused_float == id then
      held = id
    end
    host.call(window.owner, nil, fn, id)
    return
  end
  if move(window, name) then
    return
  end
  if name == "escape" and window.kind == "float" then
    close(window, true, true)
  end
end

function hooks.release()
  held = nil
end

function hooks.opened(id, pane)
  local window = windows[id]
  if not window then
    return
  end
  if pane == nil then
    close(window, true, false)
    return
  end
  window.pane = pane
end

function hooks.pane_resized(id, cols, rows)
  local window = windows[id]
  if not window then
    return
  end
  local before_cols, before_rows = window.cols, window.rows
  window.cols, window.rows = cols, rows
  resized(window, before_cols, before_rows)
  touch(window)
end

function hooks.pane_closed(id)
  local window = windows[id]
  if window then
    close(window, true, false)
  end
end

function hooks.ribbon_resized()
  for _, window in pairs(windows) do
    if window.kind == "float" then
      local cols, rows = content_size(window)
      place(window)
      resized(window, cols, rows)
      touch(window)
    end
  end
end

function hooks.focused()
  if focused_float then
    return focused_float
  end
  local pane = host.state().pane
  if pane == nil then
    return nil
  end
  return hooks.pane_window(pane)
end

function hooks.pane_window(pane)
  for id, window in pairs(windows) do
    if window.pane == pane then
      return id
    end
  end
  return nil
end

function hooks.flush()
  for id, window in pairs(windows) do
    if window.owner and host.failed(window.owner) then
      close(window, false, true)
    end
  end
  local pending = dirty
  dirty = {}
  for id in pairs(pending) do
    local window = windows[id]
    if window then
      render(window)
    end
  end
end

host.window_hooks(hooks)

host.after_event(function(name)
  if name == "FocusChanged" or name == "BandChanged" then
    if held == nil or held ~= focused_float then
      unfocus()
    end
  elseif name == nil or name == "HighlightChanged" or name == "ColorschemeChanged" then
    for _, window in pairs(windows) do
      touch(window)
    end
  end
end)

gband.win = api
