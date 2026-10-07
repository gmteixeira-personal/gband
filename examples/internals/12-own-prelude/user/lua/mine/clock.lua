local core = gband.core

local function now()
  return { os.date("%H:%M") }
end

gband.bar.add({ id = "clock", side = "right", size = 5, lines = now() })

core.timer(1000, function()
  gband.bar.set_lines("clock", now())
end)
