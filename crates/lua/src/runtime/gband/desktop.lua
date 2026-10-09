local hl = require("gband.hl")
local key_form = require("gband.keyform")

gband.hl.default("DesktopButton", { bold = true })
gband.hl.default("DesktopClose", { bold = true })
gband.hl.default("DesktopMinimized", { dim = true })
gband.hl.default("DesktopShortcut", { link = "KeyListKey" })

local MINIMIZE, MAXIMIZE, RESTORE, CLOSE = "[_]", "[□]", "[❐]", "[X]"
local LIST_TITLE = "windows"
local LIST_MIN_WIDTH = 24
local LIST_MAX_HEIGHT = 15
local MENU_MIN_WIDTH = 16
local MENU_HEIGHT = 7
local HINT = "? keys"
local LETTERS = "abcdefghijklmnopqrstuvwxyz"
local DIGITS = "1234567890"
local MENU_KEYS = {
  j = true, down = true, k = true, up = true, pagedown = true, pageup = true,
  home = true, ["end"] = true, enter = true, esc = true, q = true,
}
local LIST_KEYS = { n = true, s = true, m = true }
for index = 1, #DIGITS do
  LIST_KEYS[DIGITS:sub(index, index)] = true
end
local CORNERS = {
  { "left", "top" },
  { "right", "top" },
  { "left", "bottom" },
  { "right", "bottom" },
}

local remembered = {}
local pending = {}
local states = {}
local size = nil
local pressed = nil
local pressing = false
local menu = nil

local function find_box(layout, id)
  for _, band in ipairs(layout.bands) do
    for _, box in ipairs(band.floating) do
      if box.id == id then
        return box
      end
    end
  end
  return nil
end

local function box_width(box, cols)
  if box.full_width then
    return cols
  end
  return math.max(3, math.min(cols, math.floor(cols * box.width + 1e-9)))
end

local function state_of(box, cols, rows)
  if box.row ~= 0 or box.rows < rows then
    return nil
  end
  if box.full_width or box.width == 1 then
    return box.col == 0 and "maximized" or nil
  end
  if box.width == 1 / 2 then
    if box.col == 0 then
      return "left"
    end
    if box.col == cols - cols // 2 then
      return "right"
    end
  end
  return nil
end

local function states_of(layout)
  local found = {}
  for _, band in ipairs(layout.bands) do
    for _, box in ipairs(band.floating) do
      found[box.id] = state_of(box, layout.cols, layout.rows)
    end
  end
  return found
end

local function remember_layout(layout)
  size = { cols = layout.cols, rows = layout.rows }
  states = states_of(layout)
end

local function set_box(id, width, rows, col, row)
  gband.window.set_width(id, width)
  gband.window.set_height(id, { rows = rows })
  gband.window.set_position(id, { col = col, row = row })
end

local function apply_state(id, state, cols, rows)
  if state == "maximized" then
    set_box(id, 1, rows, 0, 0)
  elseif state == "left" then
    set_box(id, 1 / 2, rows, 0, 0)
  else
    set_box(id, 1 / 2, rows, cols - cols // 2, 0)
  end
end

local function enter_state(id, state)
  local layout = gband.layout()
  local box = find_box(layout, id)
  if not box then
    return
  end
  if state_of(box, layout.cols, layout.rows) == nil then
    remembered[id] = {
      col = box.col,
      row = box.row,
      width = box.width,
      full_width = box.full_width,
      rows = box.rows,
    }
  end
  apply_state(id, state, layout.cols, layout.rows)
  gband.window.focus(id)
end

local function restore(id)
  local layout = gband.layout()
  local saved = remembered[id]
  remembered[id] = nil
  if saved then
    gband.window.set_width(id, saved.width)
    if saved.full_width then
      gband.action.toggle_full_width({ window = id })
    end
    gband.window.set_height(id, { rows = saved.rows })
    gband.window.set_position(id, { col = saved.col, row = saved.row })
  else
    local cols, rows = layout.cols, layout.rows
    local height = math.max(3, rows // 2)
    set_box(id, 1 / 2, height, (cols - cols // 2) // 2, (rows - height) // 2)
  end
  gband.window.focus(id)
end

local function is_maximized(id)
  local layout = gband.layout()
  local box = find_box(layout, id)
  return box ~= nil and state_of(box, layout.cols, layout.rows) == "maximized"
end

local function toggle_maximized(id)
  if is_maximized(id) then
    restore(id)
  else
    enter_state(id, "maximized")
  end
end

local function window_buttons(box, cols, rows)
  local button = hl.drawn("DesktopButton")
  local maximized = state_of(box, cols, rows) == "maximized"
  return {
    { text = MINIMIZE, style = button },
    { text = maximized and RESTORE or MAXIMIZE, style = button },
    { text = CLOSE, style = hl.drawn("DesktopClose") },
  }
end

local function span_at(spans, width, col)
  local widths, total = {}, 0
  for index, span in ipairs(spans) do
    widths[index] = gband.ui.width(span.text)
    total = total + widths[index]
  end
  if total == 0 or total > width - 4 then
    return nil
  end
  local first = width - 2 - total
  for index, span_width in ipairs(widths) do
    if col >= first and col < first + span_width then
      return index
    end
    first = first + span_width
  end
  return nil
end

local function decorations(info)
  if not info.floating then
    return nil
  end
  local layout = gband.layout()
  local box = find_box(layout, info.window)
  if not (box and box.name) then
    return nil
  end
  return window_buttons(box, layout.cols, layout.rows)
end

local function is_open(win)
  for _, id in ipairs(gband.win.list()) do
    if id == win then
      return true
    end
  end
  return false
end

local function menu_open()
  return menu ~= nil and is_open(menu.win)
end

local function list_open()
  return menu_open() and menu.kind == "list"
end

local function close_menu()
  local current = menu
  menu = nil
  if current and is_open(current.win) then
    gband.win.close(current.win)
  end
end

local function pad(text, width)
  return text .. string.rep(" ", width - gband.ui.width(text))
end

local function border_line(left, right, width, title, spans)
  local total = 0
  for _, span in ipairs(spans) do
    total = total + gband.ui.width(span.text)
  end
  if total > width - 4 then
    spans, total = {}, 0
  end
  local cut = gband.ui.truncate(title or "", math.max(0, total > 0 and width - 4 - total or width - 2))
  local start = total > 0 and width - 2 - total or width - 1
  local line = {
    { text = left, hl = "PluginWindowBorder" },
    { text = cut, hl = "PluginWindowTitle" },
    { text = string.rep("─", math.max(0, start - 1 - gband.ui.width(cut))), hl = "PluginWindowBorder" },
  }
  for _, span in ipairs(spans) do
    line[#line + 1] = span
  end
  if total > 0 then
    line[#line + 1] = { text = "─", hl = "PluginWindowBorder" }
  end
  line[#line + 1] = { text = right, hl = "PluginWindowBorder" }
  return line
end

local function entry_cells(row, inner, style)
  if inner < 2 then
    return { { text = pad(gband.ui.truncate(row.label, inner), inner), hl = style } }
  end
  local band = row.band or ""
  local room = inner - 2 - (row.band and gband.ui.width(band) + 1 or 0)
  local label = gband.ui.truncate(row.label, math.max(0, room))
  local rest = gband.ui.truncate(pad(label, math.max(0, inner - 2 - gband.ui.width(band))) .. band, inner - 2)
  return {
    { text = row.shortcut or " ", hl = row.shortcut and style ~= "PluginWindowCursorLine" and "DesktopShortcut" or style },
    { text = " " .. pad(rest, inner - 2), hl = style },
  }
end

local function row_line(row, width, selected)
  if row.separator then
    return { { text = "├" .. string.rep("─", math.max(0, width - 2)) .. "┤", hl = "PluginWindowBorder" } }
  end
  local inner = math.max(0, width - 4)
  local style = selected and "PluginWindowCursorLine" or row.hl or "PluginWindow"
  local line = { { text = "│", hl = "PluginWindowBorder" }, { text = " ", hl = style } }
  if row.label then
    for _, span in ipairs(entry_cells(row, inner, style)) do
      line[#line + 1] = span
    end
  else
    line[#line + 1] = { text = pad(gband.ui.truncate(row.text, inner), inner), hl = style }
  end
  line[#line + 1] = { text = " ", hl = style }
  line[#line + 1] = { text = "│", hl = "PluginWindowBorder" }
  return line
end

local function shown_rows()
  return math.max(1, gband.win.info(menu.win).height - 2)
end

local function reveal()
  local shown = shown_rows()
  if menu.selected < menu.top then
    menu.top = menu.selected
  elseif menu.selected > menu.top + shown - 1 then
    menu.top = menu.selected - shown + 1
  end
end

local function draw()
  if not menu_open() then
    return
  end
  local info = gband.win.info(menu.win)
  local shown = math.max(0, info.height - 2)
  menu.top = math.max(1, math.min(menu.top, #menu.rows - shown + 1))
  local lines = { border_line("┌", "┐", info.width, menu.title, menu.buttons()) }
  for offset = 0, shown - 1 do
    local index = menu.top + offset
    lines[#lines + 1] = row_line(menu.rows[index] or { text = "" }, info.width, index == menu.selected)
  end
  lines[#lines + 1] = border_line("└", "┘", info.width, nil, menu.hint())
  gband.win.set_lines(menu.win, lines)
end

local function entry_from(index, step)
  while menu.rows[index] and menu.rows[index].separator do
    index = index + step
  end
  return menu.rows[index] and index or nil
end

local function opening_band(opening)
  local bands = gband.layout().bands
  for _, band in ipairs(bands) do
    if band.id == opening.band then
      return band.id
    end
  end
  local band = bands[opening.position] or bands[#bands]
  return band and band.id
end

local function view_opening(opening)
  local band = opening_band(opening)
  if band then
    gband.band.view(band)
  end
  return band
end

local function preview(row)
  local view = gband.view()
  if row.window then
    if view.window ~= row.window then
      gband.window.focus(row.window, { peek = true })
    end
    menu.shown = { band = row.band_id, window = row.window }
    return
  end
  local band = opening_band(menu.opening)
  if band and (view.peek or view.band ~= band) then
    gband.band.view(band)
  end
  menu.shown = { band = band, window = menu.opening.window }
end

local function select(index)
  if index then
    menu.selected = index
    reveal()
    if menu.kind == "list" then
      preview(menu.rows[index])
    end
    draw()
  end
end

local function step(by)
  select(entry_from(menu.selected + by, by))
end

local function page(by)
  local target = menu.selected + by * shown_rows()
  if target > #menu.rows then
    select(entry_from(#menu.rows, -1))
  elseif target < 1 then
    select(entry_from(1, 1))
  else
    select(entry_from(target, by) or entry_from(target, -by))
  end
end

local function pick(index)
  local row = menu.rows[index]
  if not row or row.separator then
    return
  end
  local closed = menu
  close_menu()
  row.act(closed)
end

local function keep()
  close_menu()
  local view = gband.view()
  if view.peek then
    gband.window.focus(view.window)
  end
end

local function cancel()
  local closed = menu
  close_menu()
  view_opening(closed.opening)
end

local function own(fn)
  return function(win)
    if menu and menu.win == win then
      fn()
    end
  end
end

local function target_at(event)
  if event.box_row == nil then
    return nil
  end
  if event.box_row == 0 then
    local button = span_at(menu.buttons(), event.box_width, event.box_col)
    return button and { button = button } or nil
  end
  if event.box_row >= event.box_height - 1 then
    return nil
  end
  local index = menu.top + event.box_row - 1
  local row = menu.rows[index]
  if row and not row.separator then
    return { entry = index }
  end
  return nil
end

local function on_mouse(win, event)
  if not (menu and menu.win == win) then
    return
  end
  if event.kind == "move" then
    local target = target_at(event)
    if target and target.entry and target.entry ~= menu.selected then
      select(target.entry)
    end
    return
  end
  if event.kind == "scroll" then
    local by = event.direction == "down" and 1 or event.direction == "up" and -1 or 0
    if menu.kind == "list" then
      menu.top = math.max(1, math.min(menu.top + by, #menu.rows - shown_rows() + 1))
      draw()
    elseif by ~= 0 then
      step(by)
    end
    return
  end
  if event.button ~= "left" then
    return
  end
  local target = target_at(event)
  if event.kind == "press" then
    menu.pressed = target
    if target and target.entry then
      select(target.entry)
    end
  elseif event.kind == "release" then
    local before = menu.pressed
    menu.pressed = nil
    if not (before and target) then
      return
    end
    if before.entry and before.entry == target.entry then
      pick(target.entry)
    elseif before.button and before.button == target.button then
      menu.press(target.button)
    end
  end
end

local function first_entry(rows)
  for index, row in ipairs(rows) do
    if not row.separator then
      return index
    end
  end
  return 1
end

local function open_menu(spec, at)
  if list_open() then
    keep()
  else
    close_menu()
  end
  gband.keymap.enter("root")
  spec.selected = first_entry(spec.rows)
  spec.top = 1
  spec.skip = pressing
  local keys = {
    j = own(function() step(1) end),
    down = own(function() step(1) end),
    k = own(function() step(-1) end),
    up = own(function() step(-1) end),
    pagedown = own(function() page(1) end),
    pageup = own(function() page(-1) end),
    home = own(function() select(entry_from(1, 1)) end),
    ["end"] = own(function() select(entry_from(#menu.rows, -1)) end),
    enter = own(function() pick(menu.selected) end),
    escape = own(spec.close),
    q = own(spec.close),
  }
  for key, fn in pairs(spec.keys or {}) do
    keys[key] = own(fn)
  end
  menu = spec
  spec.win = gband.win.open({
    border = false,
    width = spec.width,
    height = spec.height,
    col = at and math.max(0, at.col) or "center",
    row = at and math.max(0, at.row) or "center",
    keys = keys,
    on_mouse = on_mouse,
    hover = spec.hover,
    on_resize = function(win)
      if menu and menu.win == win then
        draw()
      end
    end,
    on_close = function(win)
      if menu and menu.win == win then
        local closed = menu
        menu = nil
        if closed.opening then
          view_opening(closed.opening)
        end
      end
    end,
  })
  draw()
end

local function ribbon()
  local view = gband.view()
  return view.cols, view.rows
end

local function widest(rows)
  local found = 0
  for _, row in ipairs(rows) do
    if row.width then
      found = math.max(found, row.width)
    elseif row.text then
      found = math.max(found, gband.ui.width(row.text))
    end
  end
  return found
end

local function band_label(position)
  if position <= 9 then
    return tostring(position)
  end
  if position <= 9 + #LETTERS then
    return LETTERS:sub(position - 9, position - 9)
  end
  return tostring(position)
end

local function program_windows(layout)
  local found, bands = {}, 0
  for position, band in ipairs(layout.bands) do
    local before = #found
    for _, column in ipairs(band.columns) do
      for _, window in ipairs(column.windows) do
        if window.name then
          found[#found + 1] = { position = position, band = band.id, window = window }
        end
      end
    end
    for _, box in ipairs(band.floating) do
      if box.name then
        found[#found + 1] = { position = position, band = band.id, window = box }
      end
    end
    if #found > before then
      bands = bands + 1
    end
  end
  return found, bands
end

local function recent_order(found)
  local focused, order = {}, {}
  for _, item in ipairs(found) do
    if item.window.last_focus then
      focused[#focused + 1] = item.window
    end
  end
  table.sort(focused, function(a, b)
    return a.last_focus > b.last_focus
  end)
  for _, window in ipairs(focused) do
    order[#order + 1] = window.id
  end
  for _, item in ipairs(found) do
    if not item.window.last_focus then
      order[#order + 1] = item.window.id
    end
  end
  return order
end

local function list_entry(index, item, minimized, bands)
  local id = item.window.id
  local label = item.window.name .. (minimized and " (minimized)" or "")
  local band = bands >= 2 and "band " .. band_label(item.position) or nil
  return {
    key = id,
    window = id,
    band_id = item.band,
    shortcut = index <= #DIGITS and DIGITS:sub(index, index) or nil,
    label = label,
    band = band,
    width = 2 + gband.ui.width(label) + (band and 1 + gband.ui.width(band) or 0),
    hl = minimized and "DesktopMinimized" or nil,
    act = function() gband.window.focus(id) end,
  }
end

local function list_rows(order, minimizing)
  local found, bands = program_windows(gband.layout())
  order = order or recent_order(found)
  local by_id, kept = {}, {}
  for _, item in ipairs(found) do
    by_id[item.window.id] = item
  end
  for _, id in ipairs(order) do
    if by_id[id] then
      kept[#kept + 1] = id
      by_id[id].kept = true
    end
  end
  for _, item in ipairs(found) do
    if not item.kept then
      kept[#kept + 1] = item.window.id
    end
  end
  local multi_label = gband.view().multi and "Multi mode (on)" or "Multi mode"
  local rows = {
    {
      key = "new",
      shortcut = "n",
      label = "New window",
      width = 2 + gband.ui.width("New window"),
      act = function(closed)
        local band = view_opening(closed.opening)
        gband.action.open_window({ floating = true, band = band })
      end,
    },
    {
      key = "settings",
      shortcut = "s",
      label = "Settings",
      width = 2 + gband.ui.width("Settings"),
      act = function(closed)
        view_opening(closed.opening)
        gband.settings.open()
      end,
    },
    {
      key = "multi",
      shortcut = "m",
      label = multi_label,
      width = 2 + gband.ui.width(multi_label),
      act = function(closed)
        view_opening(closed.opening)
        gband.action.toggle_multi()
      end,
    },
  }
  if #kept > 0 then
    rows[#rows + 1] = { separator = true }
  end
  for index, id in ipairs(kept) do
    local item = by_id[id]
    rows[#rows + 1] = list_entry(index, item, item.window.minimized or id == minimizing, bands)
  end
  return rows, kept
end

local function list_size(rows)
  local cols, height = ribbon()
  return math.min(cols, math.max(LIST_MIN_WIDTH, widest(rows) + 4)),
    math.min(height, LIST_MAX_HEIGHT, #rows + 2)
end

local function binds_hint_key()
  for _, binding in ipairs(gband.keymap.list("prefix")) do
    if binding.key == "?" then
      return true
    end
  end
  return false
end

local function list_shortcuts()
  local keys = {}
  for _, binding in ipairs(gband.keymap.list("prefix")) do
    local key = binding.key
    if key ~= "prefix" and not key_form.is_mouse(key) and not MENU_KEYS[key_form(key)] and not LIST_KEYS[key_form(key)] then
      keys[key] = function()
        keep()
        gband.keymap.run("prefix", key)
      end
    end
  end
  for key in pairs(LIST_KEYS) do
    keys[key] = function()
      for index, row in ipairs(menu.rows) do
        if row.shortcut == key then
          select(index)
          pick(index)
          return
        end
      end
    end
  end
  return keys
end

local function toggle_list_height()
  local info = gband.win.info(menu.win)
  if menu.restore then
    gband.win.set_config(menu.win, { row = menu.restore.row, height = menu.restore.height })
    menu.restore = nil
  else
    local _, rows = ribbon()
    menu.restore = { row = info.row, height = info.height }
    gband.win.set_config(menu.win, { row = 0, height = rows })
  end
  draw()
end

local function show_list(at)
  if list_open() and not at then
    gband.keymap.enter("root")
    gband.win.focus(menu.win)
    return
  end
  local rows, order = list_rows()
  local width, height = list_size(rows)
  local view = gband.view()
  local position = 1
  for index, band in ipairs(gband.layout().bands) do
    if band.id == view.band then
      position = index
    end
  end
  local spec = {
    kind = "list",
    title = LIST_TITLE,
    rows = rows,
    order = order,
    width = width,
    height = height,
    hover = true,
    opening = { band = view.band, position = position, window = view.window },
    shown = { band = view.band, window = view.window },
    close = cancel,
  }
  function spec.buttons()
    return {
      { text = spec.restore and RESTORE or MAXIMIZE, hl = "DesktopButton" },
      { text = CLOSE, hl = "DesktopClose" },
    }
  end
  function spec.hint()
    return binds_hint_key() and { { text = HINT, hl = "PluginWindowTitle" } } or {}
  end
  function spec.press(button)
    if button == 1 then
      toggle_list_height()
    else
      cancel()
    end
  end
  spec.keys = list_shortcuts()
  open_menu(spec, at)
end

local function rebuild(minimizing)
  if not list_open() then
    return
  end
  local before = menu.selected
  local key = menu.rows[before].key
  menu.rows, menu.order = list_rows(menu.order, minimizing)
  local selected = nil
  for index, row in ipairs(menu.rows) do
    if row.key == key then
      selected = index
    end
  end
  if not selected then
    local row = menu.rows[before]
    selected = row and not row.separator and before or entry_from(#menu.rows, -1)
  end
  menu.selected = selected
  if not menu.restore then
    local width, height = list_size(menu.rows)
    local info = gband.win.info(menu.win)
    if width ~= info.width or height ~= info.height then
      gband.win.set_config(menu.win, { width = width, height = height })
    end
  end
  if selected ~= before or menu.rows[selected].key ~= key then
    reveal()
  end
  draw()
end

local function minimize(id)
  gband.window.minimize(id)
  rebuild(id)
end

local function window_menu(id, at)
  local layout = gband.layout()
  local box = find_box(layout, id)
  local maximized = state_of(box, layout.cols, layout.rows) == "maximized"
  local rows = {
    { text = "Close", act = function() gband.action.close_window({ window = id }) end },
    {
      text = maximized and "Restore" or "Maximize",
      act = function()
        if maximized then
          restore(id)
        else
          enter_state(id, "maximized")
        end
      end,
    },
    { text = "Minimize", act = function() minimize(id) end },
    { text = "Tile left", act = function() enter_state(id, "left") end },
    { text = "Tile right", act = function() enter_state(id, "right") end },
  }
  local cols, height = ribbon()
  local spec = {
    kind = "window",
    window = id,
    title = box.name,
    rows = rows,
    width = math.min(cols, math.max(MENU_MIN_WIDTH, widest(rows) + 4, gband.ui.width(box.name) + 2)),
    height = math.min(MENU_HEIGHT, height),
    close = close_menu,
  }
  function spec.buttons()
    return {}
  end
  function spec.hint()
    return {}
  end
  function spec.press() end
  open_menu(spec, at)
end

local function pointer(event)
  local col = event.col
  for _, bar in ipairs(gband.bar.list()) do
    if bar.shown and bar.side == "left" then
      col = col - bar.width
    end
  end
  return { col = col, row = event.row }
end

local function program_box(event)
  if event.target ~= "window" or event.content_col ~= nil then
    return nil
  end
  local box = find_box(gband.layout(), event.window)
  if box and box.name then
    return box
  end
  return nil
end

local function corner(x, y, w, h)
  for _, edges in ipairs(CORNERS) do
    local cx = edges[1] == "left" and 0 or w - 1
    local cy = edges[2] == "top" and 0 or h - 1
    local dx = edges[1] == "left" and 1 or -1
    local dy = edges[2] == "top" and 1 or -1
    if (x == cx or x == cx + dx) and y == cy or x == cx and y == cy + dy then
      return { edges[1], edges[2] }
    end
  end
  return nil
end

local function press(event)
  if type(event) ~= "table" then
    return
  end
  pressing = true
  local box = program_box(event)
  if not box then
    return false
  end
  local x, y, w, h = event.box_col, event.box_row, event.box_width, event.box_height
  if y == 0 then
    local layout = gband.layout()
    local button = span_at(window_buttons(box, layout.cols, layout.rows), w, x)
    if button then
      pressed = { window = event.window, button = button }
      return
    end
  end
  local edges = corner(x, y, w, h)
  if edges then
    gband.action.drag_resize_window({ edges = edges })
  elseif y == 0 then
    gband.action.drag_window()
  elseif x == 0 then
    gband.action.drag_resize_window({ edges = { "left" } })
  elseif x == w - 1 then
    gband.action.drag_resize_window({ edges = { "right" } })
  elseif y == h - 1 then
    gband.action.drag_resize_window({ edges = { "bottom" } })
  else
    return false
  end
end

local function released(event)
  local before = pressed
  if event.button ~= "left" or not before then
    return
  end
  pressed = nil
  if event.target ~= "window" or event.window ~= before.window or event.box_row ~= 0 then
    return
  end
  local layout = gband.layout()
  local box = find_box(layout, before.window)
  if not box or span_at(window_buttons(box, layout.cols, layout.rows), event.box_width, event.box_col) ~= before.button then
    return
  end
  if before.button == 1 then
    minimize(before.window)
  elseif before.button == 2 then
    toggle_maximized(before.window)
  else
    gband.action.close_window({ window = before.window })
  end
end

local function open_menu_at(event)
  if type(event) ~= "table" then
    return
  end
  pressing = true
  local box = program_box(event)
  if box then
    window_menu(box.id, pointer(event))
  elseif event.target == "ribbon" then
    show_list(pointer(event))
  else
    return false
  end
end

local function dismiss(event)
  pressing = false
  if not menu_open() then
    return
  end
  if menu.skip then
    menu.skip = false
    return
  end
  if event.target == "plugin_window" and event.plugin_window == menu.win then
    return
  end
  local closed = menu
  close_menu()
  if closed.kind ~= "list" then
    return
  end
  if event.window then
    gband.window.focus(event.window)
    return
  end
  local view = gband.view()
  if view.band == closed.shown.band and view.window == closed.shown.window then
    view_opening(closed.opening)
  end
end

local function leader()
  if list_open() and gband.view().plugin_window == menu.win then
    keep()
    gband.keymap.run("prefix", "prefix")
    return
  end
  show_list()
end

local function float_tiled(layout)
  for _, band in ipairs(layout.bands) do
    for _, column in ipairs(band.columns) do
      for _, window in ipairs(column.windows) do
        if window.name then
          gband.action.toggle_window_floating({ window = window.id, floating = true })
          pending[window.id] = true
        end
      end
    end
  end
end

local function sweep()
  remembered = {}
  local layout = gband.layout()
  float_tiled(layout)
  remember_layout(layout)
end

local function tiled_unowned(layout, id)
  for _, band in ipairs(layout.bands) do
    for _, column in ipairs(band.columns) do
      for _, window in ipairs(column.windows) do
        if window.id == id then
          return window.plugin_window == nil
        end
      end
    end
  end
  return false
end

local function tile_awaited()
  for _, id in ipairs(gband.win.list()) do
    local info = gband.win.info(id)
    if info.kind == "tiled" and info.window == nil then
      return true
    end
  end
  return false
end

local function opened(event)
  if tiled_unowned(gband.layout(), event.window) and not tile_awaited() then
    gband.action.toggle_window_floating({ window = event.window, floating = true })
  end
  pending[event.window] = true
  rebuild()
end

local function closed(event)
  remembered[event.window] = nil
  pending[event.window] = nil
  if menu_open() and menu.kind == "window" and menu.window == event.window then
    close_menu()
  end
  rebuild()
end

local function cascade(layout)
  for _, band in ipairs(layout.bands) do
    local taken = {}
    for _, box in ipairs(band.floating) do
      local cell = box.col .. ":" .. box.row
      taken[cell] = (taken[cell] or 0) + 1
    end
    for _, box in ipairs(band.floating) do
      if pending[box.id] then
        pending[box.id] = nil
        local from = box.col .. ":" .. box.row
        local width = box_width(box, layout.cols)
        local k = 1
        while taken[from] > 1 do
          local col, row = box.col + 2 * k, box.row + k
          if col + width > layout.cols or row + box.rows > layout.rows then
            break
          end
          local cell = col .. ":" .. row
          if (taken[cell] or 0) == 0 then
            gband.window.set_position(box.id, { col = col, row = row })
            taken[from] = taken[from] - 1
            taken[cell] = 1
            break
          end
          k = k + 1
        end
      end
    end
  end
end

local function refit(layout)
  local fresh = states_of(layout)
  if size and (size.cols ~= layout.cols or size.rows ~= layout.rows) then
    for _, band in ipairs(layout.bands) do
      for _, box in ipairs(band.floating) do
        local state = states[box.id]
        if state then
          apply_state(box.id, state, layout.cols, layout.rows)
          fresh[box.id] = state
        end
      end
    end
  end
  size = { cols = layout.cols, rows = layout.rows }
  states = fresh
end

local function layout_changed()
  local layout = gband.layout()
  refit(layout)
  cascade(layout)
  rebuild()
end

return {
  name = "desktop",
  api = 2,
  setup = function(opts)
    if type(opts) ~= "table" then
      error("the options of `desktop` must be a table", 2)
    end
    local field = next(opts)
    if field ~= nil then
      error("`desktop` takes no option `" .. tostring(field) .. "`", 2)
    end
    gband.action.register("desktop.list", function()
      show_list()
    end, { desc = "list the windows" })
    gband.action.register("desktop.leader", leader, {
      desc = "open the window list, or send the prefix key from it",
    })
    gband.action.register("desktop.press", press, {
      desc = "move, resize or press a button of a floating window",
    })
    gband.action.register("desktop.menu", open_menu_at, {
      desc = "open the menu for the cell under the pointer",
    })
    gband.core.provide("decorations", decorations)
    gband.on("Attached", sweep)
    gband.on("ConfigReloaded", sweep)
    gband.on("WindowOpened", opened)
    gband.on("WindowClosed", closed)
    gband.on("LayoutChanged", layout_changed)
    gband.on("FocusChanged", function()
      rebuild()
    end)
    gband.on("MousePressed", dismiss)
    gband.on("MouseReleased", released)
  end,
}
