local waiting = {}
local bar

local function draw()
  local count = 0
  for _ in pairs(waiting) do
    count = count + 1
  end
  gband.bar.set_lines(bar, count == 0 and {} or { tostring(count) })
end

local function rescan()
  waiting = {}
  for _, band in ipairs(gband.layout().bands) do
    local windows = {}
    for _, column in ipairs(band.columns) do
      for _, entry in ipairs(column.windows) do
        windows[#windows + 1] = entry
      end
    end
    for _, entry in ipairs(band.floating) do
      windows[#windows + 1] = entry
    end
    for _, entry in ipairs(windows) do
      local state = gband.window_state(entry.id)
      if state ~= nil and state.agent == "waiting" then
        waiting[entry.id] = true
      end
    end
  end
  draw()
end

bar = gband.bar.add({ side = "right", size = 3, on_resize = rescan })

gband.on("Attached", rescan)

gband.on("WindowStateChanged", function(ev)
  if ev.key ~= "agent" then
    return
  end
  waiting[ev.window] = ev.value == "waiting" or nil
  draw()
end)

gband.on("WindowClosed", function(ev)
  waiting[ev.window] = nil
  draw()
end)

gband.on("ServerEvent", function(ev)
  gband.notify("Agent waiting in window " .. tostring(ev.data.window), { title = "gband" })
end, { pattern = "agent.waiting" })

gband.keymap.set("prefix", "a", function()
  gband.rpc("agent-status.next_waiting", { after = gband.view().window })
end, { desc = "focus the next waiting agent" })
