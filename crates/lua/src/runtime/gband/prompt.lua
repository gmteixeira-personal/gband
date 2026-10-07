local host = ...

gband.hl.default("PromptCursor", { reverse = true })

local HEIGHT = 3

local function is_open(win)
  for _, id in ipairs(gband.win.list()) do
    if id == win then
      return true
    end
  end
  return false
end

local function program_window(id)
  if not id then
    return nil
  end
  for _, band in ipairs(gband.layout().bands) do
    for _, column in ipairs(band.columns) do
      for _, window in ipairs(column.windows) do
        if window.id == id then
          return not window.plugin_window and window or nil
        end
      end
    end
    for _, window in ipairs(band.floating) do
      if window.id == id then
        return not window.plugin_window and window or nil
      end
    end
  end
  return nil
end

local function run(text)
  local chunk, message = load(text, "@prompt", "t")
  if not chunk then
    error(message, 0)
  end
  chunk()
end

local kinds = {
  lua = {
    title = "lua",
    marker = ":",
    submit = function(text)
      if text ~= "" then
        host.call(nil, nil, run, text)
      end
    end,
  },
  rename = {
    title = "rename",
    marker = "",
    submit = function(text, target)
      if program_window(target) then
        gband.window.rename(target, text)
      end
    end,
  },
}

for _, kind in pairs(kinds) do
  kind.line = ""
  kind.width = 1
end

local function fitted(kind)
  local shown = kind.marker .. kind.line
  local used = gband.ui.width(shown)
  local start = 1
  while used + 1 > kind.width and start <= #shown do
    local next_start = utf8.offset(shown, 2, start)
    used = used - gband.ui.width(shown:sub(start, next_start - 1))
    start = next_start
  end
  return shown:sub(start)
end

local function show(kind)
  gband.win.set_lines(kind.win, { { fitted(kind), { text = " ", hl = "PromptCursor" } } })
end

local function flatten(text)
  return (text:gsub("\r\n", " "):gsub("[%z\1-\31\127]", " "):gsub("\194[\128-\159]", " "))
end

local function open(kind, line, target)
  gband.keymap.enter("root")
  if kind.win and is_open(kind.win) then
    gband.win.focus(kind.win)
    return
  end
  for _, other in pairs(kinds) do
    if other ~= kind and other.win and is_open(other.win) then
      gband.win.close(other.win)
    end
  end
  kind.line = line
  kind.target = target
  local view = gband.view()
  kind.win = gband.win.open({
    title = kind.title,
    width = math.max(1, view.cols),
    height = HEIGHT,
    row = view.rows,
    col = 0,
    keys = {
      enter = function(win)
        local text, target = kind.line, kind.target
        gband.win.close(win)
        kind.submit(text, target)
      end,
      backspace = function(win)
        if kind.line == "" then
          gband.win.close(win)
          return
        end
        kind.line = kind.line:sub(1, utf8.offset(kind.line, -1) - 1)
        show(kind)
      end,
      ["ctrl+u"] = function()
        kind.line = ""
        show(kind)
      end,
    },
    on_input = function(_, text)
      kind.line = kind.line .. flatten(text)
      show(kind)
    end,
    on_resize = function(_, cols)
      kind.width = math.max(1, cols)
      show(kind)
    end,
    on_close = function(win)
      if kind.win == win then
        kind.win = nil
        kind.line = ""
        kind.target = nil
      end
    end,
  })
  kind.width = math.max(1, gband.win.info(kind.win).cols)
  show(kind)
end

local function open_lua()
  open(kinds.lua, "")
end

local function open_rename()
  local kind = kinds.rename
  if kind.win and is_open(kind.win) then
    open(kind)
    return
  end
  local window = program_window(gband.view().window)
  if window then
    open(kind, window.manual_name or "", window.id)
  end
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
    gband.action.register("prompt.open", open_lua, { desc = "run Lua" })
    gband.action.register("prompt.rename", open_rename, { desc = "rename the window" })
  end,
}
