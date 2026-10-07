local core = gband.core
local hl = require("gband.hl")

local active = nil

local function find(name)
  for _, entry in ipairs(gband.runtimepath or {}) do
    local path = entry .. "/colors/" .. name .. ".lua"
    local file = io.open(path, "r")
    if file then
      file:close()
      return function()
        return core.load(path)()
      end
    end
  end
  return core.bundled(name)
end

local function load(name, level)
  local label = "colors/" .. tostring(name)
  if type(name) ~= "string" or not name:match("^[A-Za-z0-9][A-Za-z0-9_-]*$") then
    core.report(label, "invalid colorscheme name `" .. tostring(name) .. "`", level)
    return false
  end
  local loader = find(name)
  if not loader then
    core.report(label, "no colorscheme `" .. name .. "` is on the runtimepath or bundled with gband", level)
    return false
  end
  local saved = hl.snapshot()
  local palette = core.palette.get()
  local quiet = hl.quiet
  hl.clear()
  core.palette.set({})
  hl.quiet = true
  local ok = core.call(nil, label, loader)
  hl.quiet = quiet
  if not ok then
    hl.restore(saved)
    core.palette.set(palette)
    return false
  end
  local previous = active
  active = name
  if not core.loading() then
    core.emit("ColorschemeChanged", { name = name, previous = previous })
  end
  return true
end

function gband.colorscheme(name)
  if name == nil then
    return active
  end
  local loaded = load(name, 3)
  return loaded
end

local function start()
  local saved = gband.settings.theme()
  if saved and load(saved, nil) then
    return
  end
  load("default", nil)
end

return { start = start }
