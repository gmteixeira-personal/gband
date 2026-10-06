local host = ...

local ALIGN = { top = true, center = true, bottom = true }
local FIELDS = {
  id = true, render = true, align = true, priority = true, order = true,
  hl = true, redraw_on = true, redraw_interval = true, fill = true,
}
local OPTIONS = { side = true, min_width = true, max_width = true, order = true }
local MIN_INTERVAL = 100
local REGIONS = { "top", "center", "bottom" }
local BAR = "statusline"

local EVENTS = { User = true }
for _, name in ipairs(host.events) do
  EVENTS[name] = true
end

gband.hl.default("StatusLine", { reverse = true })
gband.hl.default("StatusLineSegment", {})
gband.hl.default("StatusLineSeparator", { dim = true })
gband.hl.default("StatusLineMuted", { dim = true })
gband.hl.default("StatusLineAccent", { bold = true })
gband.hl.default("StatusLineError", { fg = "red", bold = true })

local components = {}
local sequence = 0
local placement = nil
local rendered_height = nil

local function clean(text)
  return (text:gsub("[%z\1-\31\127]", ""):gsub("\194[\128-\159]", ""))
end

local function valid_group(name)
  return type(name) == "string" and name:match("^[A-Za-z][A-Za-z0-9_.]*$") ~= nil
end

local function find(id)
  for index, component in ipairs(components) do
    if component.id == id then
      return index, component
    end
  end
  return nil
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

local function is_integer(value)
  return math.type(value) == "integer" or (math.type(value) == "float" and value % 1 == 0)
end

local function validate(spec)
  if type(spec) ~= "table" then
    return nil, "gband.ui.statusline.add expects a table"
  end
  for field in pairs(spec) do
    if not FIELDS[field] then
      return nil, "unknown field `" .. tostring(field) .. "` in a status line component"
    end
  end
  if spec.id ~= nil and (type(spec.id) ~= "string" or spec.id == "") then
    return nil, "`id` must be a non-empty string"
  end
  if type(spec.render) ~= "function" then
    return nil, "`render` must be a function"
  end
  local align = spec.align or "top"
  if not ALIGN[align] then
    return nil, "`align` must be \"top\", \"center\" or \"bottom\", found " .. tostring(spec.align)
  end
  if spec.priority ~= nil and type(spec.priority) ~= "number" then
    return nil, "`priority` must be a number"
  end
  if spec.order ~= nil and type(spec.order) ~= "number" then
    return nil, "`order` must be a number"
  end
  local hl = spec.hl or "StatusLineSegment"
  if not valid_group(hl) then
    return nil, "`hl` must be a highlight group name"
  end
  local redraw_on = {}
  if spec.redraw_on ~= nil then
    if type(spec.redraw_on) ~= "table" then
      return nil, "`redraw_on` must be a list of event names"
    end
    for _, name in ipairs(spec.redraw_on) do
      if type(name) ~= "string" then
        return nil, "`redraw_on` must be a list of event names"
      end
      if not EVENTS[name] then
        return nil, host.removed(name) or "unknown event `" .. name .. "` in `redraw_on`"
      end
      redraw_on[name] = true
    end
  end
  local interval = spec.redraw_interval
  if interval ~= nil and (type(interval) ~= "number" or not is_integer(interval) or interval < MIN_INTERVAL) then
    return nil, "`redraw_interval` must be an integer number of milliseconds, at least " .. MIN_INTERVAL
  end
  if spec.fill ~= nil and type(spec.fill) ~= "boolean" then
    return nil, "`fill` must be a boolean"
  end
  return {
    render = spec.render,
    align = align,
    priority = spec.priority or 0,
    order = spec.order or 0,
    hl = hl,
    redraw_on = redraw_on,
    interval = interval and math.tointeger(interval),
    fill = spec.fill or false,
    enabled = true,
  }
end

local function shown(component)
  return component.enabled and component.output ~= nil and not host.failed(component.plugin)
end

local function error_item(state)
  if not state.error then
    return nil
  end
  return {
    lines = { { { text = "error", hl = "StatusLineError" } } },
    width = gband.ui.width("error"),
    priority = math.huge,
    seq = -1,
    error = true,
  }
end

local function arrange(state, counted)
  local regions = { top = {}, center = {}, bottom = {} }
  local item = error_item(state)
  if item then
    regions.top[1] = item
  end
  local ordered = {}
  for _, component in ipairs(components) do
    if component ~= counted and shown(component) then
      ordered[#ordered + 1] = component
    end
  end
  table.sort(ordered, function(a, b)
    if a.order ~= b.order then
      return a.order < b.order
    end
    return a.seq < b.seq
  end)
  for _, component in ipairs(ordered) do
    local list = regions[component.align]
    list[#list + 1] = {
      component = component,
      lines = component.output,
      width = component.width,
      priority = component.priority,
      seq = component.seq,
      fill = component.fill,
    }
  end
  return regions
end

local function region_height(list)
  local total = 0
  for _, item in ipairs(list) do
    total = total + #item.lines
  end
  return total
end

local function total_height(regions)
  local total, filled = 0, 0
  for _, name in ipairs(REGIONS) do
    if #regions[name] > 0 then
      total = total + region_height(regions[name])
      filled = filled + 1
    end
  end
  if filled > 1 then
    total = total + filled - 1
  end
  return total
end

local function count(regions)
  return #regions.top + #regions.center + #regions.bottom
end

local function drop_lowest(regions)
  local lowest_region, lowest_index, lowest = nil, nil, nil
  for _, name in ipairs(REGIONS) do
    for index, item in ipairs(regions[name]) do
      if lowest == nil or item.priority < lowest.priority
        or (item.priority == lowest.priority and item.seq > lowest.seq) then
        lowest_region, lowest_index, lowest = name, index, item
      end
    end
  end
  table.remove(regions[lowest_region], lowest_index)
end

local function fit(regions, height)
  while total_height(regions) > height and count(regions) > 1 do
    drop_lowest(regions)
  end
  for _, name in ipairs(REGIONS) do
    local item = regions[name][1]
    if item and #item.lines > height then
      local kept = {}
      for row = 1, height do
        kept[row] = item.lines[row]
      end
      regions[name][1] = {
        lines = kept, width = item.width, priority = item.priority, seq = item.seq,
        fill = item.fill, error = item.error,
      }
    end
  end
end

local function widest(regions)
  local cells = 0
  for _, name in ipairs(REGIONS) do
    for _, item in ipairs(regions[name]) do
      if not item.fill then
        cells = math.max(cells, item.width)
      end
    end
  end
  return cells
end

local function clamp(cells)
  return math.max(placement.min_width, math.min(placement.max_width, cells))
end

local function cut(spans, limit)
  local result = {}
  local left = limit - 1
  for _, span in ipairs(spans) do
    local cells = gband.ui.width(span.text)
    if cells <= left then
      result[#result + 1] = span
      left = left - cells
    else
      result[#result + 1] = { text = gband.ui.truncate(span.text, left + 1), hl = span.hl }
      break
    end
  end
  return result
end

local function line_width(spans)
  local cells = 0
  for _, span in ipairs(spans) do
    cells = cells + gband.ui.width(span.text)
  end
  return cells
end

local function layout(state)
  if not placement or state.height == 0 then
    return
  end
  local height = state.height
  local regions = arrange(state)
  fit(regions, height)
  local width = clamp(widest(regions))
  if width ~= placement.size then
    placement.size = width
    gband.bar.set_config(BAR, { size = width })
  end
  local heights = {
    top = region_height(regions.top),
    center = region_height(regions.center),
    bottom = region_height(regions.bottom),
  }
  local start = (height - heights.center) // 2
  if heights.top > 0 then
    start = math.max(start, heights.top + 1)
  end
  if heights.bottom > 0 then
    start = math.min(start, height - heights.bottom - 1 - heights.center)
  end
  local starts = { top = 0, center = start, bottom = height - heights.bottom }
  local lines = {}
  local item_shown = false
  for _, name in ipairs(REGIONS) do
    local row = starts[name]
    for _, item in ipairs(regions[name]) do
      if item.error then
        item_shown = true
      end
      for _, spans in ipairs(item.lines) do
        if row >= 0 and row < height then
          lines[row + 1] = line_width(spans) > width and cut(spans, width) or spans
        end
        row = row + 1
      end
    end
  end
  for row = 1, height do
    lines[row] = lines[row] or {}
  end
  gband.bar.set_lines(BAR, lines)
  host.error_item(item_shown and gband.bar.info(BAR).shown)
end

local function available(component, state)
  local regions = arrange(state, component)
  return clamp(widest(regions)), math.max(0, state.height - total_height(regions))
end

local function context(component, state)
  local width, height = placement.max_width, state.height
  if component.fill then
    width, height = available(component, state)
  end
  return {
    id = component.id,
    side = "client",
    total_width = placement.max_width,
    total_height = state.height,
    width = width,
    height = height,
    table = state.table,
    band = { number = state.band.number, index = state.band.index, count = state.band.count },
    column = state.column and { index = state.column.index, count = state.column.count },
    window = state.window,
    windows = host.windows(),
  }
end

local function normalise_line(line, group)
  local spans = {}
  local function add(text, hl)
    text = clean(text)
    if text ~= "" then
      spans[#spans + 1] = { text = text, hl = hl }
    end
  end
  if type(line) == "string" then
    add(line, group)
  elseif type(line) == "table" then
    for _, item in ipairs(line) do
      if type(item) == "string" then
        add(item, group)
      elseif type(item) == "table" and type(item.text) == "string"
        and (item.hl == nil or valid_group(item.hl)) then
        add(item.text, item.hl or group)
      else
        return nil, "returned a line holding an invalid span"
      end
    end
  else
    return nil, "returned a " .. type(line) .. ", not a string or a list of spans"
  end
  return spans
end

local function normalise(result, group)
  if result == nil then
    return nil
  end
  local list = { result }
  if type(result) == "table" and result.lines ~= nil then
    if type(result.lines) ~= "table" then
      return nil, "returned `lines` that is not a list of lines"
    end
    list = result.lines
  end
  local lines = {}
  local filled = false
  for _, line in ipairs(list) do
    local spans, reason = normalise_line(line, group)
    if not spans then
      return nil, reason
    end
    lines[#lines + 1] = spans
    filled = filled or #spans > 0
  end
  if not filled then
    return nil
  end
  return lines
end

local function cancel(component)
  if component.timer then
    host.cancel(component.timer)
    component.timer = nil
  end
end

local function disable(component)
  component.enabled = false
  component.output = nil
  cancel(component)
end

local function render(component, state)
  if not component.enabled then
    return
  end
  if host.failed(component.plugin) then
    component.output = nil
    cancel(component)
    return
  end
  local ctx = context(component, state)
  component.context_width, component.context_height = ctx.width, ctx.height
  local ok, result = host.call(component.plugin, nil, component.render, ctx)
  if not ok then
    disable(component)
    return
  end
  local lines, reason = normalise(result, component.hl)
  if reason then
    host.report(component.plugin, "the status line component `" .. component.id .. "` " .. reason)
    disable(component)
    return
  end
  component.output = lines
  local cells = 0
  for _, spans in ipairs(lines or {}) do
    cells = math.max(cells, line_width(spans))
  end
  component.width = cells
end

local function render_order(a, b)
  if a.fill ~= b.fill then
    return b.fill
  end
  if a.fill and a.priority ~= b.priority then
    return a.priority > b.priority
  end
  return a.seq < b.seq
end

local function refill(state)
  local fills = {}
  for _, component in ipairs(components) do
    if component.fill and component.enabled then
      fills[#fills + 1] = component
    end
  end
  table.sort(fills, render_order)
  for _, component in ipairs(fills) do
    local width, height = available(component, state)
    if width ~= component.context_width or height ~= component.context_height then
      render(component, state)
    end
  end
end

local function stop_timers()
  for _, component in ipairs(components) do
    cancel(component)
  end
end

local function start_timer(component)
  if component.timer or not component.interval or not component.enabled then
    return
  end
  component.timer = host.timer(component.interval, function()
    if not placement then
      stop_timers()
      return
    end
    local state = host.state()
    render(component, state)
    refill(state)
    layout(state)
  end)
end

local function snapshot()
  local list = {}
  for index, component in ipairs(components) do
    list[index] = component
  end
  return list
end

local function redraw(every, name)
  if not placement then
    return
  end
  local state = host.state()
  if state.height == 0 then
    return
  end
  every = every or (rendered_height ~= nil and state.height ~= rendered_height)
  local due = {}
  for _, component in ipairs(snapshot()) do
    start_timer(component)
    if every or (name and component.redraw_on[name]) then
      due[#due + 1] = component
    end
  end
  table.sort(due, render_order)
  for _, component in ipairs(due) do
    render(component, state)
  end
  if every then
    rendered_height = state.height
  end
  if #due > 0 then
    refill(state)
  end
  layout(state)
end

host.after_event(function(name)
  redraw(name == nil or name == "HighlightChanged" or name == "ColorschemeChanged", name)
end)

host.on_state(function()
  if placement then
    layout(host.state())
  end
end)

local statusline = {}

function statusline.add(spec)
  local component, reason = validate(spec)
  if not component then
    error(reason, 2)
  end
  local owner = host.owner()
  if owner == nil and spec.id == nil then
    error("a status line component added outside a plugin must have an `id`", 2)
  end
  local id
  id, reason = full_id(owner, spec.id)
  if not id then
    error(reason, 2)
  end
  if find(id) then
    error("a status line component `" .. id .. "` already exists", 2)
  end
  sequence = sequence + 1
  component.id = id
  component.plugin = owner
  component.seq = sequence
  components[#components + 1] = component
  local state = host.state()
  if placement and not host.loading() and state.height > 0 then
    start_timer(component)
    render(component, state)
    refill(state)
    layout(state)
  end
  return id
end

function statusline.remove(id)
  local index, component = find(id)
  if not index then
    return false
  end
  cancel(component)
  table.remove(components, index)
  if placement and not host.loading() then
    layout(host.state())
  end
  return true
end

function statusline.list()
  local list = {}
  for _, component in ipairs(components) do
    list[#list + 1] = {
      id = component.id,
      align = component.align,
      priority = component.priority,
      order = component.order,
      hl = component.hl,
      fill = component.fill,
      plugin = component.plugin,
      enabled = component.enabled,
    }
  end
  table.sort(list, function(a, b)
    return a.id < b.id
  end)
  return list
end

gband.ui.statusline = statusline

local function check_options(opts)
  if type(opts) ~= "table" then
    error("the options of `statusline` must be a table", 3)
  end
  for field in pairs(opts) do
    if not OPTIONS[field] then
      error("unknown option `" .. tostring(field) .. "`", 3)
    end
  end
  local side = opts.side or "left"
  if side ~= "left" and side ~= "right" then
    error("`side` must be \"left\" or \"right\", found " .. tostring(opts.side), 3)
  end
  local function width(name, default, least)
    local value = opts[name]
    if value == nil then
      return default
    end
    if type(value) ~= "number" or not is_integer(value) or value < least or value > 65535 then
      error("`" .. name .. "` must be an integer of at least " .. least, 3)
    end
    return math.tointeger(value)
  end
  local min_width = width("min_width", 20, 1)
  local max_width = width("max_width", math.max(40, min_width), min_width)
  if opts.order ~= nil and type(opts.order) ~= "number" then
    error("`order` must be a number", 3)
  end
  return {
    side = side,
    min_width = min_width,
    max_width = max_width,
    order = opts.order or 0,
  }
end

return {
  name = "statusline",
  setup = function(opts)
    local options = check_options(opts)
    gband.bar.add({
      side = options.side,
      size = options.min_width,
      order = options.order,
      hl = "StatusLine",
      on_resize = function()
        redraw(false, nil)
      end,
    })
    options.size = options.min_width
    placement = options
  end,
}
