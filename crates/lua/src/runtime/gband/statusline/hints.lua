gband.hl.default("KeyHintKey", { link = "StatusLineAccent" })
gband.hl.default("KeyHintLabel", { link = "StatusLineSegment" })

local ALIGN = { left = true, center = true, right = true }

local SHORT = {
  focus_column_left = "left",
  focus_column_right = "right",
  focus_window_down = "down",
  focus_window_up = "up",
  focus_band_down = "band down",
  focus_band_up = "band up",
  open_window = "new",
  close_window = "close",
  consume_or_expel_left = "stack left",
  consume_or_expel_right = "stack right",
  move_column_left = "move left",
  move_column_right = "move right",
  move_window_down = "move down",
  move_window_up = "move up",
  toggle_window_floating = "float",
  switch_focus_floating_tiled = "layer",
  cycle_column_width = "width",
  toggle_full_width = "full",
  grow_column_width = "wider",
  shrink_column_width = "narrower",
  grow_window_height = "taller",
  shrink_window_height = "shorter",
  reset_window_height = "reset height",
  detach = "detach",
  send_prefix = "send prefix",
}

local function key_form(written)
  local modifiers, key
  if written == "+" then
    modifiers, key = "", "+"
  elseif written:sub(-2) == "++" then
    modifiers, key = written:sub(1, -3), "+"
  else
    modifiers, key = written:match("^(.*)%+([^+]*)$")
    if not modifiers then
      modifiers, key = "", written
    end
  end
  local held = {}
  for modifier in modifiers:gmatch("[^+]+") do
    held[modifier:lower()] = true
  end
  if utf8.len(key) == 1 then
    if held.shift and key:match("^%l$") then
      key = key:upper()
      held.shift = nil
    end
  else
    key = key:lower()
    if key == "escape" then
      key = "esc"
    end
  end
  return (held.ctrl and "C-" or "") .. (held.alt and "A-" or "") .. (held.shift and "S-" or "") .. key
end

local function present(text)
  return text ~= nil and text ~= ""
end

local function label(binding, descs, labels)
  local action, desc = binding.action, binding.desc
  if action == nil then
    return present(desc) and desc or nil
  end
  local own = labels[action]
  if own ~= nil then
    return own or nil
  end
  if present(desc) and desc ~= descs[action] then
    return desc
  end
  if SHORT[action] then
    return SHORT[action]
  end
  if present(descs[action]) then
    return descs[action]
  end
  return action
end

local function hints(active, labels)
  local descs = {}
  for _, action in ipairs(gband.action.list()) do
    descs[action.name] = action.desc
  end
  local prefix = key_form(gband.opt.prefix)
  local list = {}
  if active == "root" and #gband.keymap.list("prefix") > 0 then
    list[1] = { key = prefix, label = "prefix" }
  end
  for _, binding in ipairs(gband.keymap.list(active)) do
    local text = label(binding, descs, labels)
    if text then
      local key = binding.key == "prefix" and prefix or key_form(binding.key)
      list[#list + 1] = { key = key, label = text }
    end
  end
  return list
end

local function fitted(list, width)
  local used, shown = 0, 0
  for index, hint in ipairs(list) do
    local cells = gband.ui.width(hint.key) + 1 + gband.ui.width(hint.label)
    if index > 1 then
      cells = cells + 2
    end
    local ellipsis = index < #list and 2 or 0
    if used + cells + ellipsis > width then
      break
    end
    used, shown = used + cells, index
  end
  if shown == 0 then
    return nil
  end
  local spans = {}
  for index = 1, shown do
    if index > 1 then
      spans[#spans + 1] = { text = "  ", hl = "KeyHintLabel" }
    end
    spans[#spans + 1] = { text = list[index].key, hl = "KeyHintKey" }
    spans[#spans + 1] = { text = " " .. list[index].label, hl = "KeyHintLabel" }
  end
  if shown < #list then
    spans[#spans + 1] = { text = " …", hl = "KeyHintLabel" }
  end
  return spans
end

local function invalid(opts)
  if opts.align ~= nil and not ALIGN[opts.align] then
    return "`align` must be \"left\", \"center\" or \"right\""
  end
  if opts.priority ~= nil and type(opts.priority) ~= "number" then
    return "`priority` must be a number"
  end
  if opts.order ~= nil and type(opts.order) ~= "number" then
    return "`order` must be a number"
  end
  if opts.root ~= nil and type(opts.root) ~= "boolean" then
    return "`root` must be a boolean"
  end
  if opts.labels ~= nil then
    if type(opts.labels) ~= "table" then
      return "`labels` must be a table from action names to strings or false"
    end
    for name, text in pairs(opts.labels) do
      if type(name) ~= "string" or (type(text) ~= "string" and text ~= false) then
        return "`labels` must be a table from action names to strings or false"
      end
    end
  end
end

return {
  name = "hints",
  api = 1,
  setup = function(opts)
    if type(opts) ~= "table" then
      error("the options of `hints` must be a table", 2)
    end
    local reason = invalid(opts)
    if reason then
      error(reason, 2)
    end
    local labels = opts.labels or {}
    local root = opts.root ~= false
    gband.ui.statusline.add({
      align = opts.align or "left",
      priority = opts.priority or 0,
      order = opts.order or 30,
      hl = "KeyHintLabel",
      redraw_on = { "KeyTableChanged" },
      fill = true,
      render = function(ctx)
        if ctx.table == "root" and not root then
          return nil
        end
        return fitted(hints(ctx.table, labels), ctx.width)
      end,
    })
  end,
}
