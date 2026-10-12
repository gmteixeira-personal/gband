local core = gband.core

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
        core.call(nil, nil, run, text)
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
  kind.cursor = 0
  kind.first = 0
  kind.width = 1
end

local function characters(text)
  local list = {}
  for character in text:gmatch(utf8.charpattern) do
    list[#list + 1] = character
  end
  return list
end

local function fewest_dropped(shown, last, used, width)
  for i = 1, last do
    used = used + gband.ui.width(shown[i])
  end
  local dropped = 0
  while used > width and dropped < last do
    dropped = dropped + 1
    used = used - gband.ui.width(shown[dropped])
  end
  return dropped
end

local function fitted(kind)
  local shown = characters(kind.marker .. kind.line)
  local at = utf8.len(kind.marker) + kind.cursor + 1
  local cell = shown[at] or " "
  local low = fewest_dropped(shown, at - 1, gband.ui.width(cell), kind.width)
  local fill = fewest_dropped(shown, #shown, 1, kind.width)
  kind.first = math.max(low, math.min(kind.first, kind.cursor, fill))
  local before = table.concat(shown, "", kind.first + 1, at - 1)
  local used = gband.ui.width(before) + gband.ui.width(cell)
  local after = {}
  for i = at + 1, #shown do
    used = used + gband.ui.width(shown[i])
    if used > kind.width then
      break
    end
    after[#after + 1] = shown[i]
  end
  return before, cell, table.concat(after)
end

local function show(kind)
  local before, cell, after = fitted(kind)
  gband.win.set_lines(kind.win, { { before, { text = cell, hl = "PromptCursor" }, after } })
end

local function clicked(kind, col)
  local shown = characters(kind.marker .. kind.line)
  local edge = 0
  for i = kind.first + 1, #shown do
    edge = edge + gband.ui.width(shown[i])
    if edge > kind.width then
      break
    end
    if col < edge then
      return math.max(0, i - 1 - utf8.len(kind.marker))
    end
  end
  return utf8.len(kind.line)
end

local function flatten(text)
  return (text:gsub("\r\n", " "):gsub("[%z\1-\31\127]", " "):gsub("\194[\128-\159]", " "))
end

local function insert(kind, text)
  local at = utf8.offset(kind.line, kind.cursor + 1)
  kind.line = kind.line:sub(1, at - 1) .. text .. kind.line:sub(at)
  kind.cursor = kind.cursor + utf8.len(text)
end

local function cut(kind, at)
  local from = utf8.offset(kind.line, at + 1)
  local to = utf8.offset(kind.line, at + 2)
  kind.line = kind.line:sub(1, from - 1) .. kind.line:sub(to)
end

local function move(kind, cursor)
  kind.cursor = math.max(0, math.min(cursor, utf8.len(kind.line)))
  show(kind)
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
  kind.cursor = utf8.len(line)
  kind.first = 0
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
        if kind.cursor == 0 then
          return
        end
        kind.cursor = kind.cursor - 1
        cut(kind, kind.cursor)
        show(kind)
      end,
      delete = function()
        if kind.cursor < utf8.len(kind.line) then
          cut(kind, kind.cursor)
          show(kind)
        end
      end,
      ["ctrl+u"] = function()
        kind.line = ""
        kind.cursor = 0
        show(kind)
      end,
      left = function()
        move(kind, kind.cursor - 1)
      end,
      right = function()
        move(kind, kind.cursor + 1)
      end,
      home = function()
        move(kind, 0)
      end,
      ["end"] = function()
        move(kind, utf8.len(kind.line))
      end,
    },
    on_input = function(_, text)
      insert(kind, flatten(text))
      show(kind)
    end,
    on_mouse = function(_, event)
      if event.kind == "press" and event.button == "left" and event.content_row == 0 then
        move(kind, clicked(kind, event.content_col))
      end
    end,
    on_resize = function(_, cols)
      kind.width = math.max(1, cols)
      show(kind)
    end,
    on_close = function(win)
      if kind.win == win then
        kind.win = nil
        kind.line = ""
        kind.cursor = 0
        kind.first = 0
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
  api = 2,
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
