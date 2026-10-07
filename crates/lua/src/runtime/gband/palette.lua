local core = gband.core

local FIELDS = {
  fg = true, bg = true,
  black = true, red = true, green = true, yellow = true,
  blue = true, magenta = true, cyan = true, white = true,
  bright_black = true, bright_red = true, bright_green = true, bright_yellow = true,
  bright_blue = true, bright_magenta = true, bright_cyan = true, bright_white = true,
}

gband.palette = {}

function gband.palette.set(spec)
  if type(spec) ~= "table" then
    error("gband.palette.set expects a table of colors", 2)
  end
  local parsed = {}
  for field, value in pairs(spec) do
    if not FIELDS[field] then
      error("unknown palette field `" .. tostring(field) .. "`", 2)
    end
    if type(value) ~= "string" or not value:match("^#%x%x%x%x%x%x$") then
      error("invalid color for the palette field `" .. field .. "`: " .. tostring(value), 2)
    end
    parsed[field] = value:lower()
  end
  core.palette.set(parsed)
end

function gband.palette.get()
  return core.palette.get()
end
