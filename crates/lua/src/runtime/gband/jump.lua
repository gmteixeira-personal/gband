local agents = require("gband.agents")

local POSITIONS = 9
local PREVIOUS = { "k", "up" }
local NEXT = { "j", "down" }

local function interactive()
  gband.keymap.enter("root")
end

local function enter_agents_mode()
  gband.keymap.enter("agents")
end

local function enter_bands_mode()
  gband.keymap.enter("bands")
end

local function agents_mode()
  local set = gband.keymap.set
  gband.keymap.mode("agents", { label = "agents" })
  for position = 1, POSITIONS do
    set("agents", tostring(position), function()
      if agents.focus(position) then
        interactive()
      end
    end, { desc = "focus agent " .. position })
  end
  for _, key in ipairs(PREVIOUS) do
    set("agents", key, function()
      agents.step(-1)
    end, { desc = "focus the previous agent" })
  end
  for _, key in ipairs(NEXT) do
    set("agents", key, function()
      agents.step(1)
    end, { desc = "focus the next agent" })
  end
  set("agents", "b", enter_bands_mode, { desc = "bands mode" })
  set("agents", "escape", interactive, { desc = "interactive mode" })
  set("agents", "enter", interactive, { desc = "interactive mode" })
end

local function bands_mode()
  local set = gband.keymap.set
  gband.keymap.mode("bands", { label = "bands" })
  for position = 1, POSITIONS do
    set("bands", tostring(position), function()
      local band = gband.layout().bands[position]
      if band then
        gband.band.view(band.id)
      end
    end, { desc = "view band " .. position })
  end
  for _, key in ipairs(PREVIOUS) do
    set("bands", key, gband.action.focus_band_up, { desc = "view the band above" })
  end
  for _, key in ipairs(NEXT) do
    set("bands", key, gband.action.focus_band_down, { desc = "view the band below" })
  end
  set("bands", "a", enter_agents_mode, { desc = "agents mode" })
  set("bands", "escape", interactive, { desc = "interactive mode" })
  set("bands", "enter", interactive, { desc = "interactive mode" })
end

local function setup()
  agents_mode()
  bands_mode()
  gband.keymap.set("prefix", "a", enter_agents_mode, { desc = "agents mode" })
  gband.keymap.set("prefix", "b", enter_bands_mode, { desc = "bands mode" })
end

gband.jump = {
  setup = setup,
}
return gband.jump
