local core = gband.core
local hl = require("gband.hl")

gband.hl.default("Bar", {})

local SIDES = { left = true, right = true }
local FIELDS = {
  id = true, side = true, size = true, order = true, lines = true, hl = true, on_resize = true,
}
local CONFIG = { side = true, size = true, order = true, hl = true }

local bars = {}
local sequence = 0
local dirty = true
local presented_height = nil

local function clean(text)
  return (text:gsub("[%z\1-\31\127]", ""):gsub("\194[\128-\159]", ""))
end

local function valid_group(name)
  return type(name) == "string" and name:match("^[A-Za-z][A-Za-z0-9_.]*$") ~= nil
end

local function is_integer(value)
  return math.type(value) == "integer" or (math.type(value) == "float" and value % 1 == 0)
end

local function full_id(owner, id)
  if owner == nil then
    return id
  end
  if id == nil then
    return owner
  end
  if id:sub(1, #owner + 1) == owner .. "." then
    return id
  end
  if id:find(".", 1, true) then
    return nil, "`" .. id .. "` is outside the namespace of the plugin `" .. owner .. "`"
  end
  return owner .. "." .. id
end

local function prune()
  for id, bar in pairs(bars) do
    if bar.plugin and core.failed(bar.plugin) then
      bars[id] = nil
      dirty = true
    end
  end
end

local function lookup(id, name)
  prune()
  local bar = type(id) == "string" and bars[id] or nil
  if not bar then
    error(name .. ": no bar `" .. tostring(id) .. "` exists", 3)
  end
  return bar
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
      spans[1] = { text = clean(line) }
    elseif type(line) == "table" then
      for _, span in ipairs(line) do
        if type(span) == "string" then
          spans[#spans + 1] = { text = clean(span) }
        elseif type(span) == "table" and type(span.text) == "string"
          and (span.hl == nil or valid_group(span.hl)) then
          spans[#spans + 1] = { text = clean(span.text), hl = span.hl }
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

local function check_config(config, fields, name)
  if type(config) ~= "table" then
    error(name .. " expects a table", 3)
  end
  for field in pairs(config) do
    if not fields[field] then
      error(name .. ": unknown field `" .. tostring(field) .. "`", 3)
    end
  end
  if config.side ~= nil and not SIDES[config.side] then
    error(name .. ": `side` must be \"left\" or \"right\", found " .. tostring(config.side), 3)
  end
  if config.size ~= nil and (type(config.size) ~= "number" or not is_integer(config.size)
    or config.size < 1 or config.size > 65535) then
    error(name .. ": `size` must be an integer of at least 1", 3)
  end
  if config.order ~= nil and type(config.order) ~= "number" then
    error(name .. ": `order` must be a number", 3)
  end
  if config.hl ~= nil and not valid_group(config.hl) then
    error(name .. ": `hl` must be a highlight group name", 3)
  end
end

local function ordered()
  local list = {}
  for _, bar in pairs(bars) do
    list[#list + 1] = bar
  end
  table.sort(list, function(a, b)
    if a.order ~= b.order then
      return a.order < b.order
    end
    return a.seq < b.seq
  end)
  return list
end

local function slot(bar)
  return { side = bar.side, size = bar.size, order = bar.order, seq = bar.seq }
end

local function placements()
  local state = core.state()
  local list = ordered()
  local slots = {}
  for index, bar in ipairs(list) do
    slots[index] = slot(bar)
  end
  local shown = core.place_bars(slots, state.width)
  local result = {}
  for index, bar in ipairs(list) do
    local columns = shown[index]
    if columns then
      result[bar.id] = { col = columns.col, width = columns.width, height = state.height }
    end
  end
  return result
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

local function row_runs(spans, width, style)
  local runs = {}
  local used = 0
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
    local text = table.concat(pieces)
    if text ~= "" then
      runs[#runs + 1] = { text = text, style = style(span.hl) }
    end
    if full then
      break
    end
  end
  return runs
end

local function present()
  local height = core.state().height
  local list = {}
  for _, bar in ipairs(ordered()) do
    local base = hl.drawn(bar.hl)
    local styles = {}
    local function style(group)
      group = group or bar.hl
      styles[group] = styles[group] or over(base, hl.drawn(group))
      return styles[group]
    end
    local lines = {}
    for row = 1, math.min(#bar.lines, height) do
      lines[row] = row_runs(bar.lines[row], bar.size, style)
    end
    list[#list + 1] = {
      id = bar.id,
      side = bar.side,
      size = bar.size,
      order = bar.order,
      seq = bar.seq,
      base = base,
      lines = lines,
    }
  end
  presented_height = height
  core.present_bars(list)
end

local function notify()
  local placed = placements()
  for _, bar in ipairs(ordered()) do
    local shown = placed[bar.id]
    local width = shown and shown.width or 0
    local height = shown and shown.height or 0
    local last = bar.notified
    if bars[bar.id] == bar and (last == nil or last.width ~= width or last.height ~= height) then
      bar.notified = { width = width, height = height }
      if bar.on_resize then
        core.call(bar.plugin, nil, bar.on_resize, bar.id, width, height)
      end
    end
  end
end

local api = {}

function api.add(spec)
  prune()
  check_config(spec, FIELDS, "gband.bar.add")
  if spec.side == nil then
    error("gband.bar.add: `side` must be \"left\" or \"right\"", 2)
  end
  if spec.id ~= nil and (type(spec.id) ~= "string" or spec.id == "") then
    error("gband.bar.add: `id` must be a non-empty string", 2)
  end
  if spec.on_resize ~= nil and type(spec.on_resize) ~= "function" then
    error("gband.bar.add: `on_resize` must be a function", 2)
  end
  local lines = {}
  if spec.lines ~= nil then
    local reason
    lines, reason = normalise_lines(spec.lines)
    if not lines then
      error("gband.bar.add: " .. reason, 2)
    end
  end
  local owner = core.owner()
  if owner == nil and spec.id == nil then
    error("gband.bar.add: a bar added outside a plugin must have an `id`", 2)
  end
  local id, reason = full_id(owner, spec.id)
  if not id then
    error("gband.bar.add: " .. reason, 2)
  end
  if bars[id] then
    error("gband.bar.add: a bar `" .. id .. "` already exists", 2)
  end
  sequence = sequence + 1
  bars[id] = {
    id = id,
    plugin = owner,
    side = spec.side,
    size = spec.size and math.tointeger(spec.size) or 1,
    order = spec.order or 0,
    lines = lines,
    hl = spec.hl or "Bar",
    on_resize = spec.on_resize,
    seq = sequence,
  }
  dirty = true
  return id
end

function api.set_lines(id, lines)
  local bar = lookup(id, "gband.bar.set_lines")
  local normalised, reason = normalise_lines(lines)
  if not normalised then
    error("gband.bar.set_lines: " .. reason, 2)
  end
  bar.lines = normalised
  dirty = true
end

function api.set_config(id, config)
  local bar = lookup(id, "gband.bar.set_config")
  check_config(config, CONFIG, "gband.bar.set_config")
  for field in pairs(CONFIG) do
    if config[field] ~= nil then
      bar[field] = field == "size" and math.tointeger(config.size) or config[field]
    end
  end
  dirty = true
end

function api.remove(id)
  prune()
  if type(id) ~= "string" or not bars[id] then
    return false
  end
  bars[id] = nil
  dirty = true
  return true
end

local function info(bar, placed)
  local shown = placed[bar.id]
  return {
    id = bar.id,
    side = bar.side,
    size = bar.size,
    order = bar.order,
    hl = bar.hl,
    plugin = bar.plugin,
    shown = shown ~= nil,
    col = shown and shown.col,
    width = shown and shown.width,
    height = shown and shown.height,
  }
end

function api.info(id)
  local bar = lookup(id, "gband.bar.info")
  return info(bar, placements())
end

function api.list()
  prune()
  local placed = placements()
  local list = {}
  for _, bar in pairs(bars) do
    list[#list + 1] = info(bar, placed)
  end
  table.sort(list, function(a, b)
    return a.id < b.id
  end)
  return list
end

local hooks = {}

function hooks.flush()
  prune()
  if core.loading() then
    return
  end
  notify()
  prune()
  if core.state().height ~= presented_height then
    dirty = true
  end
  if dirty then
    dirty = false
    present()
  end
end

core.provide("bars", hooks)

core.after_event(function(name)
  if name == nil or name == "HighlightChanged" or name == "ColorschemeChanged" then
    dirty = true
  end
end)

gband.bar = api
