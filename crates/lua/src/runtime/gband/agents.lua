local function entry(window, band)
  local state = gband.window_state(window)
  local agent = state and state.agent_state
  if type(agent) ~= "table" then
    return nil
  end
  return { window = window, band = band, agent = agent.agent, state = agent.state }
end

local function list()
  local entries = {}
  local function add(window, band)
    local found = entry(window, band)
    if found then
      entries[#entries + 1] = found
    end
  end
  for _, band in ipairs(gband.layout().bands) do
    for _, column in ipairs(band.columns) do
      for _, window in ipairs(column.windows) do
        add(window.id, band.id)
      end
    end
    for _, window in ipairs(band.floating) do
      add(window.id, band.id)
    end
  end
  return entries
end

local function focus(n)
  if type(n) ~= "number" or n < 1 or n ~= math.floor(n) then
    error("gband.agents.focus expects a positive integer, got `" .. tostring(n) .. "`", 2)
  end
  local found = list()[n]
  if not found then
    return false
  end
  gband.window.focus(found.window)
  return true
end

local function step(delta)
  if delta ~= 1 and delta ~= -1 then
    error("gband.agents.step expects 1 or -1, got `" .. tostring(delta) .. "`", 2)
  end
  local entries = list()
  if #entries == 0 then
    return nil
  end
  local focused = gband.view().window
  local index
  for i, found in ipairs(entries) do
    if found.window == focused then
      index = i
    end
  end
  if index then
    index = (index - 1 + delta) % #entries + 1
  elseif delta == 1 then
    index = 1
  else
    index = #entries
  end
  local window = entries[index].window
  gband.window.focus(window)
  return window
end

gband.agents = {
  list = list,
  focus = focus,
  step = step,
}
return gband.agents
