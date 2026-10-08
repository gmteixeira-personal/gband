local core = gband.core

local FILE = "user/keystyle.lua"

local used

local function is_style(value)
  return value == "modal" or value == "direct" or value == "floating"
end

local function saved()
  return require("gband.settings").read(FILE, is_style)
end

local function use(style)
  if not core.loading() then
    error("gband.keystyle.use can only be called while the configuration loads", 2)
  end
  if used then
    error("gband.keystyle.use can only be called once", 2)
  end
  if style == nil then
    style = saved() or "modal"
  end
  if not is_style(style) then
    error('gband.keystyle.use expects "modal", "direct" or "floating", got `' .. tostring(style) .. "`", 2)
  end
  used = style
  require("gband.keystyle." .. style)
  return style
end

local function current()
  return used
end

gband.keystyle = {
  use = use,
  saved = saved,
  current = current,
}
