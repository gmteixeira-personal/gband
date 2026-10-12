local core = gband.core
local agents = require("gband.agents")

gband.hl.default("SidebarMode", { bold = true })
gband.hl.default("SidebarBand", { dim = true })
gband.hl.default("SidebarBandActive", { bold = true })
gband.hl.default("SidebarError", { fg = 1, bold = true })
gband.hl.default("SidebarAgentBlocked", { fg = "red" })
gband.hl.default("SidebarAgentBlockedActive", { fg = "bright_red", bold = true })
gband.hl.default("SidebarAgentDone", { fg = "green" })
gband.hl.default("SidebarAgentDoneActive", { fg = "bright_green", bold = true })
gband.hl.default("SidebarAgentWorking", { fg = "yellow" })
gband.hl.default("SidebarAgentWorkingActive", { fg = "bright_yellow", bold = true })
gband.hl.default("SidebarAgentIdle", { fg = "blue" })
gband.hl.default("SidebarAgentIdleActive", { fg = "bright_blue", bold = true })
gband.hl.default("SidebarAgentUnknown", { link = "SidebarBand" })
gband.hl.default("SidebarAgentUnknownActive", { link = "SidebarBandActive" })
gband.hl.default("SidebarDivider", { link = "SidebarBand" })

local BAR = "sidebar"
local OPTIONS = { side = true, order = true }
local LETTERS = "abcdefghijklmnopqrstuvwxyz"
local MARKER_ROWS = 1
local FIRST_BAND_ROW = 2
local DIVIDER = "─"
local WHEEL = { down = "focus_band_down", up = "focus_band_up" }
local URGENCY = { "blocked", "done", "working", "idle", "unknown" }
local AGENT_GROUPS = {
  blocked = "SidebarAgentBlocked",
  done = "SidebarAgentDone",
  working = "SidebarAgentWorking",
  idle = "SidebarAgentIdle",
  unknown = "SidebarAgentUnknown",
}

local APPS = "∷"

local placed = false

local function floating_style()
  return gband.keystyle.current() == "floating"
end

local function mode_letter(active, multi)
  if floating_style() then
    return APPS
  end
  if active == "root" then
    return multi and "M" or "I"
  end
  local label = gband.keymap.label(active)
  local letter = label:match("^" .. utf8.charpattern) or ""
  if letter:match("^%l$") then
    letter = letter:upper()
  end
  return letter
end

local function band_label(position)
  if position <= 9 then
    return tostring(position)
  end
  if position <= 9 + #LETTERS then
    return LETTERS:sub(position - 9, position - 9)
  end
  return nil
end

local function divider_row(height)
  local row = height // 2
  if row <= FIRST_BAND_ROW or row >= height - 1 - MARKER_ROWS then
    return nil
  end
  return row
end

local function last_band_row(height)
  local divider = divider_row(height)
  return divider and divider - 1 or height - 1 - MARKER_ROWS
end

local function position_at(row, height)
  local position = row - FIRST_BAND_ROW + 1
  if position < 1 or row > last_band_row(height) or band_label(position) == nil then
    return nil
  end
  return position
end

local function agent_group(state, active)
  local group = AGENT_GROUPS[state] or AGENT_GROUPS.unknown
  return active and group .. "Active" or group
end

local function band_states(listed)
  local states = {}
  for _, agent in ipairs(listed) do
    local found = states[agent.band] or {}
    found[agent.state] = true
    states[agent.band] = found
  end
  return states
end

local function most_urgent(found)
  if not found then
    return nil
  end
  for _, state in ipairs(URGENCY) do
    if found[state] then
      return state
    end
  end
  return "unknown"
end

local function agent_rows(height)
  local divider = divider_row(height)
  if not divider then
    return nil
  end
  local first = divider + 1
  return first, math.min(height - 1 - MARKER_ROWS, first + 9 + #LETTERS - 1)
end

local function lines_for(state, height)
  local lines = {}
  for row = 1, height do
    lines[row] = {}
  end
  local marker = state.error ~= nil
  if height == 1 then
    if marker then
      lines[1] = { { text = "!", hl = "SidebarError" } }
    else
      lines[1] = { { text = mode_letter(state.table, state.multi), hl = "SidebarMode" } }
    end
    return lines, marker
  end
  lines[1] = { { text = mode_letter(state.table, state.multi), hl = "SidebarMode" } }
  if marker then
    lines[height] = { { text = "!", hl = "SidebarError" } }
  end
  local divider = divider_row(height)
  if divider then
    lines[divider + 1] = { { text = DIVIDER, hl = "SidebarDivider" } }
  end
  local listed = agents.list()
  local states = band_states(listed)
  local bands = gband.layout().bands
  for position = 1, state.band.count do
    local row = FIRST_BAND_ROW + position - 1
    if position_at(row, height) == nil then
      break
    end
    local active = position == state.band.index
    local band = bands[position]
    local urgent = most_urgent(band and states[band.id])
    local hl
    if urgent then
      hl = agent_group(urgent, active)
    else
      hl = active and "SidebarBandActive" or "SidebarBand"
    end
    lines[row + 1] = { { text = band_label(position), hl = hl } }
  end
  local first, last = agent_rows(height)
  if not first then
    return lines, marker
  end
  local focused = gband.view().window
  for position, agent in ipairs(listed) do
    local row = first + position - 1
    if row > last then
      break
    end
    lines[row + 1] = { { text = band_label(position), hl = agent_group(agent.state, agent.window == focused) } }
  end
  return lines, marker
end

local function draw()
  if not placed then
    return
  end
  local state = core.state()
  local info = gband.bar.info(BAR)
  local height = info.height or state.height
  if height == 0 then
    core.error_marker(false)
    return
  end
  local lines, marker = lines_for(state, height)
  gband.bar.set_lines(BAR, lines)
  core.error_marker(marker and info.shown)
end

core.on_state(draw)

local function redraw_on_events()
  local group = gband.augroup("gband.sidebar")
  gband.on("WindowStateChanged", function(ev)
    if ev.key == "agent_state" then
      draw()
    end
  end, { group = group })
  for _, event in ipairs({ "LayoutChanged", "FocusChanged", "Attached" }) do
    gband.on(event, draw, { group = group })
  end
end

local function agent_at(row, height)
  local first, last = agent_rows(height)
  if not first or row < first or row > last then
    return nil
  end
  return agents.list()[row - first + 1]
end

local MARKER_ACTIONS = { left = "errors.open", right = "errors.clear" }

local function clicked(ev)
  local marker_action = MARKER_ACTIONS[ev.button]
  if not marker_action or ev.target ~= "outside" then
    return
  end
  local info = gband.bar.info(BAR)
  if not info.shown or ev.col ~= info.col then
    return
  end
  if ev.row == info.height - 1 and core.state().error ~= nil then
    local run = gband.action[marker_action]
    if run then
      run()
    end
    return
  end
  if ev.button ~= "left" then
    return
  end
  if ev.row == 0 and floating_style() then
    local list = gband.action["desktop.list"]
    if list then
      list()
    end
    return
  end
  local position = position_at(ev.row, info.height)
  local band = position and gband.layout().bands[position]
  if band then
    gband.band.view(band.id)
    return
  end
  local agent = agent_at(ev.row, info.height)
  if agent then
    gband.window.focus(agent.window)
  end
end

local function scrolled(ev)
  local action = WHEEL[ev.direction]
  if not action or ev.target ~= "outside" or ev.ctrl or ev.alt or ev.shift then
    return
  end
  local info = gband.bar.info(BAR)
  if info.shown and ev.col == info.col then
    gband.action[action]()
  end
end

local function check_options(opts)
  if type(opts) ~= "table" then
    error("the options of `sidebar` must be a table", 2)
  end
  for field in pairs(opts) do
    if not OPTIONS[field] then
      error("unknown option `" .. tostring(field) .. "`", 2)
    end
  end
  local side = opts.side or "left"
  if side ~= "left" and side ~= "right" then
    error("`side` must be \"left\" or \"right\", found " .. tostring(opts.side), 2)
  end
  if opts.order ~= nil and type(opts.order) ~= "number" then
    error("`order` must be a number", 2)
  end
  return side, opts.order or 0
end

return {
  name = "sidebar",
  setup = function(opts)
    local side, order = check_options(opts)
    gband.bar.add({
      side = side,
      size = 1,
      order = order,
      on_resize = function()
        draw()
      end,
    })
    gband.on("MousePressed", clicked)
    gband.on("MouseScrolled", scrolled)
    redraw_on_events()
    placed = true
    if not core.loading() then
      draw()
    end
  end,
}
