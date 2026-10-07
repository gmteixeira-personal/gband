local host = ...

local NAMED = {
  black = 0, red = 1, green = 2, yellow = 3, blue = 4, magenta = 5, cyan = 6, white = 7,
  bright_black = 8, bright_red = 9, bright_green = 10, bright_yellow = 11,
  bright_blue = 12, bright_magenta = 13, bright_cyan = 14, bright_white = 15,
}
local FLAGS = { bold = true, italic = true, underline = true, reverse = true, dim = true }
local STYLE_FIELDS = { "fg", "bg", "bold", "italic", "underline", "reverse", "dim" }

local groups = {}
local warned = {}
local internal = { quiet = false }

local function valid_name(name)
  return type(name) == "string" and name:match("^[A-Za-z][A-Za-z0-9_.]*$") ~= nil
end

local function color(value)
  if type(value) == "string" then
    if value:match("^#%x%x%x%x%x%x$") then
      return value:lower()
    end
    if NAMED[value] then
      return value
    end
  elseif math.type(value) == "integer" and value >= 0 and value <= 255 then
    return value
  elseif math.type(value) == "float" and value >= 0 and value <= 255 and value % 1 == 0 then
    return math.tointeger(value)
  end
  return nil
end

local function normalise(name, spec)
  if type(spec) ~= "table" then
    return nil, "the style of `" .. name .. "` must be a table"
  end
  local out = {}
  for field, value in pairs(spec) do
    if field == "fg" or field == "bg" then
      local normalised = color(value)
      if normalised == nil then
        return nil, "invalid color for `" .. field .. "` of `" .. name .. "`: " .. tostring(value)
      end
      out[field] = normalised
    elseif FLAGS[field] then
      if type(value) ~= "boolean" then
        return nil, "`" .. field .. "` of `" .. name .. "` must be a boolean"
      end
      out[field] = value
    elseif field == "link" then
      if not valid_name(value) then
        return nil, "`link` of `" .. name .. "` must be a group name"
      end
      if value == name then
        return nil, "the group `" .. name .. "` cannot link to itself"
      end
      out.link = value
    else
      return nil, "unknown field `" .. tostring(field) .. "` in the style of `" .. name .. "`"
    end
  end
  return out
end

local function copy(spec)
  if spec == nil then
    return nil
  end
  local out = {}
  for field, value in pairs(spec) do
    out[field] = value
  end
  return out
end

local function same(a, b)
  if a == nil or b == nil then
    return a == b
  end
  for field, value in pairs(a) do
    if b[field] ~= value then
      return false
    end
  end
  for field in pairs(b) do
    if a[field] == nil then
      return false
    end
  end
  return true
end

local function definition(name, override_name, override_layer, override_spec)
  local group = groups[name] or {}
  local explicit, default = group.explicit, group.default
  if name == override_name then
    if override_layer == "explicit" then
      explicit = override_spec
    else
      default = override_spec
    end
  end
  return explicit or default
end

local function cycle_from(name, layer, spec)
  local chain = { name }
  local seen = { [name] = true }
  local current = definition(name, name, layer, spec)
  while current and current.link do
    local target = current.link
    if target == name then
      return chain
    end
    if seen[target] then
      return nil
    end
    seen[target] = true
    chain[#chain + 1] = target
    current = definition(target, name, layer, spec)
  end
  return nil
end

local function changed(name)
  if host.loading() or internal.quiet then
    return
  end
  host.emit("HighlightChanged", { group = name })
end

local function store(layer, name, spec)
  if not valid_name(name) then
    return "invalid highlight group name `" .. tostring(name) .. "`"
  end
  local normalised = nil
  if spec ~= nil then
    local reason
    normalised, reason = normalise(name, spec)
    if not normalised then
      return reason
    end
  end
  local cycle = cycle_from(name, layer, normalised)
  if cycle then
    return "highlight link cycle through " .. table.concat(cycle, ", ")
  end
  local group = groups[name] or {}
  groups[name] = group
  if same(group[layer], normalised) then
    group[layer] = normalised
    return nil
  end
  group[layer] = normalised
  changed(name)
  return nil
end

local function resolve(name)
  local chain = {}
  local seen = {}
  local visited = {}
  local current = name
  while current do
    if seen[current] then
      local members = {}
      local inside = false
      for _, member in ipairs(visited) do
        inside = inside or member == current
        if inside then
          members[#members + 1] = member
        end
      end
      table.sort(members)
      local key = table.concat(members, ",")
      if not warned[key] then
        warned[key] = true
        host.warn("highlight link cycle through " .. table.concat(members, ", "))
      end
      break
    end
    local spec = definition(current)
    if not spec then
      break
    end
    seen[current] = true
    visited[#visited + 1] = current
    chain[#chain + 1] = spec
    current = spec.link
  end
  local style = {}
  for index = #chain, 1, -1 do
    for _, field in ipairs(STYLE_FIELDS) do
      local value = chain[index][field]
      if value ~= nil then
        style[field] = value
      end
    end
  end
  return style
end

function internal.resolve(name)
  return resolve(name)
end

function internal.drawn(name)
  local style = resolve(name)
  for _, field in ipairs({ "fg", "bg" }) do
    if type(style[field]) == "string" and NAMED[style[field]] then
      style[field] = NAMED[style[field]]
    end
  end
  return style
end

function internal.snapshot()
  local saved = {}
  for name, group in pairs(groups) do
    saved[name] = group.explicit
  end
  return saved
end

function internal.clear()
  for _, group in pairs(groups) do
    group.explicit = nil
  end
end

function internal.restore(saved)
  for name, group in pairs(groups) do
    group.explicit = saved[name]
  end
end

host.hl = internal

gband.hl = {}

function gband.hl.set(name, spec)
  local reason = store("explicit", name, spec)
  if reason then
    error(reason, 2)
  end
end

function gband.hl.default(name, spec)
  if spec == nil then
    error("gband.hl.default expects a style table", 2)
  end
  local reason = store("default", name, spec)
  if reason then
    error(reason, 2)
  end
end

function gband.hl.get(name, opts)
  if not valid_name(name) then
    error("invalid highlight group name `" .. tostring(name) .. "`", 2)
  end
  if opts ~= nil and type(opts) ~= "table" then
    error("gband.hl.get expects its options as a table", 2)
  end
  if opts and opts.resolve then
    return resolve(name)
  end
  return copy(definition(name))
end

gband.hl.default("WindowBorder", { dim = true })
gband.hl.default("WindowBorderFocused", { fg = "#b1b9f9", bold = true })
gband.hl.default("ErrorBanner", { fg = "red", reverse = true })

host.client_styles(function()
  return {
    border = internal.drawn("WindowBorder"),
    border_focused = internal.drawn("WindowBorderFocused"),
    banner = internal.drawn("ErrorBanner"),
  }
end)
