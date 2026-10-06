local host = ...

local active = nil

local function find(name)
  for _, entry in ipairs(gband.runtimepath or {}) do
    local path = entry .. "/colors/" .. name .. ".lua"
    local file = io.open(path, "r")
    if file then
      file:close()
      return function()
        return host.load(path)()
      end
    end
  end
  return host.bundled(name)
end

local function load(name, level)
  local label = "colors/" .. tostring(name)
  if type(name) ~= "string" or not name:match("^[A-Za-z0-9][A-Za-z0-9_-]*$") then
    host.report(label, "invalid colorscheme name `" .. tostring(name) .. "`", level)
    return false
  end
  local loader = find(name)
  if not loader then
    host.report(label, "no colorscheme `" .. name .. "` is on the runtimepath or bundled with gband", level)
    return false
  end
  local saved = host.hl.snapshot()
  local palette = host.palette.get()
  local quiet = host.hl.quiet
  host.hl.clear()
  host.palette.set({})
  host.hl.quiet = true
  local ok = host.call(nil, label, loader)
  host.hl.quiet = quiet
  if not ok then
    host.hl.restore(saved)
    host.palette.set(palette)
    return false
  end
  local previous = active
  active = name
  if not host.loading() then
    host.emit("ColorschemeChanged", { name = name, previous = previous })
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

function host.start_theme()
  local saved = gband.settings.theme()
  if saved and load(saved, nil) then
    return
  end
  load("default", nil)
end
