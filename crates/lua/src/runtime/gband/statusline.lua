local host = ...

local ALIGN = { left = true, center = true, right = true }
local FIELDS = {
  id = true, render = true, align = true, priority = true, order = true,
  hl = true, redraw_on = true, redraw_interval = true, fill = true,
}
local MIN_INTERVAL = 100
local REGIONS = { "left", "center", "right" }

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
local rendered_width = nil

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
  local align = spec.align or "left"
  if not ALIGN[align] then
    return nil, "`align` must be \"left\", \"center\" or \"right\", found " .. tostring(spec.align)
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
        return nil, "unknown event `" .. name .. "` in `redraw_on`"
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

local function separator()
  return gband.opt.statusline_separator
end

local function error_item(state)
  if not state.error then
    return nil
  end
  local text = clean(state.error:match("^[^\n]*"))
  if text == "" then
    return nil
  end
  return {
    spans = { { text = text, hl = "StatusLineError" } },
    width = gband.ui.width(text),
    priority = math.huge,
    seq = -1,
  }
end

local function arrange(state, counted)
  local regions = { left = {}, center = {}, right = {} }
  local item = error_item(state)
  if item then
    regions.left[1] = item
  end
  local ordered = {}
  for _, component in ipairs(components) do
    if component == counted or shown(component) then
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
      spans = component == counted and {} or component.output,
      width = component == counted and 0 or component.width,
      priority = component.priority,
      seq = component.seq,
    }
  end
  return regions
end

local function region_width(list, gap)
  if #list == 0 then
    return 0
  end
  local total = gap * (#list - 1)
  for _, item in ipairs(list) do
    total = total + item.width
  end
  return total
end

local function line_width(regions, gap)
  local total, filled = 0, 0
  for _, name in ipairs(REGIONS) do
    if #regions[name] > 0 then
      total = total + region_width(regions[name], gap)
      filled = filled + 1
    end
  end
  if filled > 1 then
    total = total + filled - 1
  end
  return total
end

local function count(regions)
  return #regions.left + #regions.center + #regions.right
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

local function cut(item, limit)
  local spans = {}
  local left = limit - 1
  for _, span in ipairs(item.spans) do
    local cells = gband.ui.width(span.text)
    if cells <= left then
      spans[#spans + 1] = span
      left = left - cells
    else
      spans[#spans + 1] = { text = gband.ui.truncate(span.text, left + 1), hl = span.hl }
      break
    end
  end
  item.spans = spans
  item.width = limit
end

local function fit(regions, total, gap)
  while line_width(regions, gap) > total and count(regions) > 1 do
    drop_lowest(regions)
  end
  if count(regions) == 1 and line_width(regions, gap) > total then
    for _, name in ipairs(REGIONS) do
      local item = regions[name][1]
      if item then
        if total == 0 then
          regions[name] = {}
        else
          cut(item, total)
        end
      end
    end
  end
end

local function merged(base, group)
  local style = {}
  for field, value in pairs(base) do
    style[field] = value
  end
  for field, value in pairs(host.hl.drawn(group)) do
    style[field] = value
  end
  return style
end

local function layout(state)
  if not state.drawn then
    return
  end
  local total = state.width
  local sep = separator()
  local gap = gband.ui.width(sep)
  local regions = arrange(state)
  fit(regions, total, gap)
  local base = host.hl.drawn("StatusLine")
  local styles = {}
  local function style(group)
    styles[group] = styles[group] or merged(base, group)
    return styles[group]
  end
  local left = region_width(regions.left, gap)
  local right = region_width(regions.right, gap)
  local center = region_width(regions.center, gap)
  local starts = { left = 0, right = total - right }
  local start = (total - center) // 2
  if #regions.left > 0 then
    start = math.max(start, left + 1)
  end
  if #regions.right > 0 then
    start = math.min(start, total - right - 1 - center)
  end
  starts.center = start
  local spans = {}
  for _, name in ipairs(REGIONS) do
    local col = starts[name]
    for index, item in ipairs(regions[name]) do
      if index > 1 then
        spans[#spans + 1] = { col = col, text = sep, style = style("StatusLineSeparator") }
        col = col + gap
      end
      for _, span in ipairs(item.spans) do
        spans[#spans + 1] = { col = col, text = span.text, style = style(span.hl) }
        col = col + gband.ui.width(span.text)
      end
    end
  end
  host.present({ height = gband.opt.statusline_height, base = base, spans = spans })
end

local function available(component, state)
  local regions = arrange(state, component)
  return math.max(0, state.width - line_width(regions, gband.ui.width(separator())))
end

local function context(component, state)
  return {
    id = component.id,
    side = "client",
    total_width = state.width,
    width = available(component, state),
    table = state.table,
    band = { number = state.band.number, index = state.band.index, count = state.band.count },
    column = state.column and { index = state.column.index, count = state.column.count },
    window = state.window,
    windows = host.windows(),
  }
end

local function normalise(result, group)
  local spans = {}
  local function add(text, hl)
    text = clean(text)
    if text ~= "" then
      spans[#spans + 1] = { text = text, hl = hl }
    end
  end
  if result == nil then
    return nil
  elseif type(result) == "string" then
    add(result, group)
  elseif type(result) == "table" then
    for _, item in ipairs(result) do
      if type(item) == "string" then
        add(item, group)
      elseif type(item) == "table" and type(item.text) == "string"
        and (item.hl == nil or valid_group(item.hl)) then
        add(item.text, item.hl or group)
      else
        return nil, "returned a list holding an invalid span"
      end
    end
  else
    return nil, "returned a " .. type(result) .. ", not a string or a list of spans"
  end
  if #spans == 0 then
    return nil
  end
  return spans
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
  component.context_width = ctx.width
  local ok, result = host.call(component.plugin, nil, component.render, ctx)
  if not ok then
    disable(component)
    return
  end
  local spans, reason = normalise(result, component.hl)
  if reason then
    host.report(component.plugin, "the status line component `" .. component.id .. "` " .. reason)
    disable(component)
    return
  end
  component.output = spans
  local cells = 0
  for _, span in ipairs(spans or {}) do
    cells = cells + gband.ui.width(span.text)
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
    if available(component, state) ~= component.context_width then
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
    local state = host.state()
    if not state.drawn then
      stop_timers()
      return
    end
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

host.after_event(function(name)
  local state = host.state()
  if not state.drawn then
    stop_timers()
    return
  end
  local every = name == nil or name == "HighlightChanged" or name == "ColorschemeChanged"
    or (name == "TerminalResized" and state.width ~= rendered_width)
  local due = {}
  for _, component in ipairs(snapshot()) do
    start_timer(component)
    if every or component.redraw_on[name] then
      due[#due + 1] = component
    end
  end
  table.sort(due, render_order)
  for _, component in ipairs(due) do
    render(component, state)
  end
  if every then
    rendered_width = state.width
  end
  if #due > 0 then
    refill(state)
  end
  layout(state)
end)

host.on_state(function()
  local state = host.state()
  if state.drawn then
    layout(state)
  else
    stop_timers()
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
  if not host.loading() then
    local state = host.state()
    if state.drawn then
      start_timer(component)
      render(component, state)
      refill(state)
      layout(state)
    end
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
  if not host.loading() then
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
