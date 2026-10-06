local host = ...

gband.hl.default("PromptCursor", { reverse = true })

local OWN = "prompt.open"
local TITLE = "lua"
local HEIGHT = 3

local current = nil
local line = ""
local width = 1

local function is_open(win)
  for _, id in ipairs(gband.win.list()) do
    if id == win then
      return true
    end
  end
  return false
end

local function fitted()
  local shown = ":" .. line
  local used = gband.ui.width(shown)
  local start = 1
  while used + 1 > width and start <= #shown do
    local next_start = utf8.offset(shown, 2, start)
    used = used - gband.ui.width(shown:sub(start, next_start - 1))
    start = next_start
  end
  return shown:sub(start)
end

local function show(win)
  gband.win.set_lines(win, { { fitted(), { text = " ", hl = "PromptCursor" } } })
end

local function flatten(text)
  return (text:gsub("\r\n", " "):gsub("[%z\1-\31\127]", " "):gsub("\194[\128-\159]", " "))
end

local function run(text)
  local chunk, message = load(text, "@prompt", "t")
  if not chunk then
    error(message, 0)
  end
  chunk()
end

local function enter(win)
  local text = line
  gband.win.close(win)
  if text ~= "" then
    host.call(nil, nil, run, text)
  end
end

local function backspace(win)
  if line == "" then
    gband.win.close(win)
    return
  end
  line = line:sub(1, utf8.offset(line, -1) - 1)
  show(win)
end

local function open()
  gband.keymap.enter("root")
  if current and is_open(current) then
    gband.win.focus(current)
    return
  end
  line = ""
  local view = gband.view()
  current = gband.win.open({
    title = TITLE,
    width = math.max(1, view.cols),
    height = HEIGHT,
    row = view.rows,
    col = 0,
    keys = {
      enter = enter,
      backspace = backspace,
      ["ctrl+u"] = function(win)
        line = ""
        show(win)
      end,
    },
    on_input = function(win, text)
      line = line .. flatten(text)
      show(win)
    end,
    on_resize = function(win, cols)
      width = math.max(1, cols)
      show(win)
    end,
    on_close = function(win)
      if current == win then
        current = nil
        line = ""
      end
    end,
  })
  width = math.max(1, gband.win.info(current).cols)
  show(current)
end

return {
  name = "prompt",
  api = 1,
  setup = function(opts)
    if type(opts) ~= "table" then
      error("the options of `prompt` must be a table", 2)
    end
    for field in pairs(opts) do
      error("`prompt` takes no option `" .. tostring(field) .. "`", 2)
    end
    gband.action.register(OWN, open, { desc = "run Lua" })
  end,
}
