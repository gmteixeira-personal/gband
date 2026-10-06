local host = ...

local STYLES = { "modal", "direct" }
local DESCRIPTIONS = {
  modal = "enters a mode, keys repeat until Escape",
  direct = "then one key per action, back to typing",
}
local NAME_WIDTH = 8
local TITLE = "key style  enter choose  esc later"
local FILE = "user/keystyle.lua"

local used = false
local chooser = nil

local function is_style(value)
  return value == "modal" or value == "direct"
end

local function is_open(win)
  for _, id in ipairs(gband.win.list()) do
    if id == win then
      return true
    end
  end
  return false
end

local function saved()
  local dir = gband.config_dir
  if type(dir) ~= "string" then
    return nil
  end
  local ok, value = pcall(function()
    local chunk = loadfile(dir .. "/" .. FILE, "t", {})
    if chunk then
      return chunk()
    end
  end)
  if ok and is_style(value) then
    return value
  end
  return nil
end

local function save(style)
  local dir = gband.config_dir
  if type(dir) ~= "string" then
    error("cannot save " .. FILE .. ": there is no configuration directory", 0)
  end
  local temporary = string.format("%s/user/.keystyle.%d.%d", dir, os.time(), math.random(1, 1 << 30))
  local file, reason = io.open(temporary, "w")
  local ok = file ~= nil
  if file then
    ok, reason = file:write('return "' .. style .. '"\n')
    local closed, close_reason = file:close()
    if ok and not closed then
      ok, reason = false, close_reason
    end
    if ok then
      ok, reason = os.rename(temporary, dir .. "/" .. FILE)
    end
    if not ok then
      os.remove(temporary)
    end
  end
  if not ok then
    error("cannot save " .. FILE .. ": " .. tostring(reason), 0)
  end
end

local function pick(win)
  local style = STYLES[gband.win.info(win).cursor]
  gband.win.close(win)
  if style then
    save(style)
  end
end

local function use(style)
  if not host.loading() then
    error("gband.keystyle.use can only be called while the configuration loads", 2)
  end
  if used then
    error("gband.keystyle.use can only be called once", 2)
  end
  if style == nil then
    style = saved() or "modal"
  end
  if not is_style(style) then
    error('gband.keystyle.use expects "modal" or "direct", got `' .. tostring(style) .. "`", 2)
  end
  used = true
  require("gband.keystyle." .. style)
  return style
end

local function choose()
  if host.loading() then
    error("gband.keystyle.choose cannot be called while the configuration loads", 2)
  end
  gband.keymap.enter("root")
  if chooser and is_open(chooser) then
    gband.win.focus(chooser)
    return
  end
  local prefix = require("gband.keyform")(gband.opt.prefix)
  local lines, longest, start = {}, 0, 1
  local current = saved()
  for index, style in ipairs(STYLES) do
    local text = style .. string.rep(" ", NAME_WIDTH - #style) .. prefix .. " " .. DESCRIPTIONS[style]
    lines[index] = text
    longest = math.max(longest, gband.ui.width(text))
    if style == current then
      start = index
    end
  end
  chooser = gband.win.open({
    title = TITLE,
    width = longest + 2,
    height = #STYLES + 2,
    lines = lines,
    cursorline = true,
    keys = { enter = pick },
    on_close = function(win)
      if chooser == win then
        chooser = nil
      end
    end,
  })
  gband.win.set_cursor(chooser, start)
end

gband.keystyle = {
  use = use,
  saved = saved,
  choose = choose,
}
