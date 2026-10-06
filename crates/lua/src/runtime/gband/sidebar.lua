local host = ...

gband.hl.default("SidebarMode", { bold = true })
gband.hl.default("SidebarBand", { dim = true })
gband.hl.default("SidebarBandActive", { bold = true })
gband.hl.default("SidebarError", { fg = 1, bold = true })

local BAR = "sidebar"
local OPTIONS = { side = true, order = true }
local LETTERS = "abcdefghijklmnopqrstuvwxyz"
local MARKER_ROWS = 1
local FIRST_BAND_ROW = 2
local WHEEL = { down = "focus_band_down", up = "focus_band_up" }

local placed = false

local function mode_letter(active)
  if active == "root" then
    return "I"
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

local function position_at(row, height)
  local position = row - FIRST_BAND_ROW + 1
  if position < 1 or row > height - 1 - MARKER_ROWS or band_label(position) == nil then
    return nil
  end
  return position
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
      lines[1] = { { text = mode_letter(state.table), hl = "SidebarMode" } }
    end
    return lines, marker
  end
  lines[1] = { { text = mode_letter(state.table), hl = "SidebarMode" } }
  if marker then
    lines[height] = { { text = "!", hl = "SidebarError" } }
  end
  for position = 1, state.band.count do
    local row = FIRST_BAND_ROW + position - 1
    if position_at(row, height) == nil then
      break
    end
    local hl = position == state.band.index and "SidebarBandActive" or "SidebarBand"
    lines[row + 1] = { { text = band_label(position), hl = hl } }
  end
  return lines, marker
end

local function draw()
  if not placed then
    return
  end
  local state = host.state()
  local info = gband.bar.info(BAR)
  local height = info.height or state.height
  if height == 0 then
    host.error_item(false)
    return
  end
  local lines, marker = lines_for(state, height)
  gband.bar.set_lines(BAR, lines)
  host.error_item(marker and info.shown)
end

host.on_state(draw)

local function clicked(ev)
  if ev.button ~= "left" or ev.target ~= "outside" then
    return
  end
  local info = gband.bar.info(BAR)
  if not info.shown or ev.col ~= info.col then
    return
  end
  local position = position_at(ev.row, info.height)
  local band = position and gband.layout().bands[position]
  if band then
    gband.band.view(band.id)
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
    placed = true
    if not host.loading() then
      draw()
    end
  end,
}
