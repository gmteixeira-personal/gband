local key_form = require("gband.keyform")

gband.hl.default("KeyListKey", { bold = true })
gband.hl.default("KeyListMuted", { dim = true })

local OWN = "keylist.open"
local MAX_HEIGHT = 15
local OWN_KEYS = {
  up = true, down = true, pageup = true, pagedown = true,
  home = true, ["end"] = true, enter = true, esc = true,
}

local current = nil

local function present(text)
  return text ~= nil and text ~= ""
end

local function describe(binding, descs)
  if present(binding.desc) then
    return binding.desc
  end
  local action = binding.action
  if action == nil then
    return "function"
  end
  return present(descs[action]) and descs[action] or action
end

local function entries()
  local descs = {}
  for _, action in ipairs(gband.action.list()) do
    descs[action.name] = action.desc
  end
  local prefix = key_form(gband.opt.prefix)
  local list = {}
  for _, binding in ipairs(gband.keymap.list("prefix")) do
    if not key_form.is_mouse(binding.key) then
      local form = binding.key == "prefix" and prefix or key_form(binding.key)
      list[#list + 1] = {
        key = form,
        text = describe(binding, descs),
        binding = binding.action ~= OWN and binding.key or nil,
        direct = binding.key ~= "prefix" and not OWN_KEYS[form] and binding.key or nil,
      }
    end
  end
  return list
end

local function lines_of(list)
  local widest = 0
  for _, entry in ipairs(list) do
    widest = math.max(widest, gband.ui.width(entry.key))
  end
  local lines, longest = {}, 0
  for index, entry in ipairs(list) do
    local key = entry.key .. string.rep(" ", widest + 2 - gband.ui.width(entry.key))
    lines[index] = {
      { text = key, hl = "KeyListKey" },
      { text = entry.text, hl = entry.binding and "PluginWindow" or "KeyListMuted" },
    }
    longest = math.max(longest, gband.ui.width(key) + gband.ui.width(entry.text))
  end
  return lines, longest
end

local function is_open(win)
  for _, id in ipairs(gband.win.list()) do
    if id == win then
      return true
    end
  end
  return false
end

local function other_float_focused(win)
  for _, id in ipairs(gband.win.list()) do
    if id ~= win then
      local info = gband.win.info(id)
      if info.focused and info.kind == "floating" then
        return true
      end
    end
  end
  return false
end

local function open()
  gband.keymap.enter("root")
  if current and is_open(current) then
    gband.win.focus(current)
    return
  end
  local list = entries()
  local lines, longest = lines_of(list)
  local function run(win, index)
    local entry = list[index]
    if not (entry and entry.binding) then
      return
    end
    gband.keymap.run("prefix", entry.binding)
    if is_open(win) and other_float_focused(win) then
      gband.win.close(win)
    end
  end
  local keys = {
    enter = function(win)
      run(win, gband.win.info(win).cursor)
    end,
  }
  for index, entry in ipairs(list) do
    if entry.direct then
      keys[entry.direct] = function(win)
        gband.win.set_cursor(win, index)
        run(win, index)
      end
    end
  end
  current = gband.win.open({
    title = gband.keymap.label("prefix") .. " keys",
    width = longest + 2,
    height = math.min(math.max(#lines + 2, 3), MAX_HEIGHT),
    cursorline = true,
    lines = lines,
    keys = keys,
    on_close = function(win)
      if current == win then
        current = nil
      end
    end,
  })
end

return {
  name = "keylist",
  api = 1,
  setup = function(opts)
    if type(opts) ~= "table" then
      error("the options of `keylist` must be a table", 2)
    end
    local field = next(opts)
    if field ~= nil then
      error("`keylist` takes no option `" .. tostring(field) .. "`", 2)
    end
    gband.action.register(OWN, open, { desc = "list the keys" })
  end,
}
