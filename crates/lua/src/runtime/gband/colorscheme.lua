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

function gband.colorscheme(name)
  if name == nil then
    return active
  end
  local label = "colors/" .. tostring(name)
  if type(name) ~= "string" or not name:match("^[A-Za-z0-9][A-Za-z0-9_-]*$") then
    host.report(label, "invalid colorscheme name `" .. tostring(name) .. "`", 2)
    return false
  end
  local loader = find(name)
  if not loader then
    host.report(label, "no colorscheme `" .. name .. "` is on the runtimepath or bundled with gband", 2)
    return false
  end
  local saved = host.hl.snapshot()
  local quiet = host.hl.quiet
  host.hl.clear()
  host.hl.quiet = true
  local ok = host.call(nil, label, loader)
  host.hl.quiet = quiet
  if not ok then
    host.hl.restore(saved)
    return false
  end
  local previous = active
  active = name
  if not host.loading() then
    host.emit("ColorschemeChanged", { name = name, previous = previous })
  end
  return true
end
