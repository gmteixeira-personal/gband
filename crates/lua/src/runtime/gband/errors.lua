local OWN = "errors.open"
local KINDS = { floating = true, tiled = true }

local current = nil
local texts = {}

local function wrap(text, width)
  local lines = {}
  local pieces, used = {}, 0
  for char in text:gmatch(utf8.charpattern) do
    local cells = gband.ui.width(char)
    if used + cells > width and used > 0 then
      lines[#lines + 1] = table.concat(pieces)
      pieces, used = {}, 0
    end
    pieces[#pieces + 1] = char
    used = used + cells
  end
  lines[#lines + 1] = table.concat(pieces)
  return lines
end

local function lines_of(width)
  if #texts == 0 then
    return { "no errors" }
  end
  local lines = {}
  for index, text in ipairs(texts) do
    if index > 1 then
      lines[#lines + 1] = ""
    end
    for _, line in ipairs(wrap(text, math.max(1, width))) do
      lines[#lines + 1] = line
    end
  end
  return lines
end

local function is_open(win)
  for _, id in ipairs(gband.win.list()) do
    if id == win then
      return true
    end
  end
  return false
end

local function open(kind)
  gband.keymap.enter("root")
  if current and is_open(current) then
    gband.win.focus(current)
    return
  end
  texts = gband.errors()
  local spec = {
    kind = kind,
    lines = { "" },
    keys = {
      q = function(win)
        gband.win.close(win)
      end,
    },
    on_resize = function(win, cols)
      gband.win.set_lines(win, lines_of(cols))
    end,
    on_close = function(win)
      if current == win then
        current = nil
      end
    end,
  }
  if kind == "floating" then
    local state = gband.view()
    spec.title = "errors"
    spec.width = math.max(1, state.cols * 3 // 4)
    spec.height = math.max(1, state.rows // 2)
    spec.row = "center"
    spec.col = "center"
  end
  current = gband.win.open(spec)
  local cols = gband.win.info(current).cols
  if cols then
    gband.win.set_lines(current, lines_of(cols))
  end
end

local function check_kind(kind, field)
  if kind ~= nil and not KINDS[kind] then
    error("`" .. field .. "` must be \"floating\" or \"tiled\", found " .. tostring(kind), 3)
  end
end

return {
  name = "errors",
  api = 1,
  setup = function(opts)
    if type(opts) ~= "table" then
      error("the options of `errors` must be a table", 2)
    end
    for field in pairs(opts) do
      if field ~= "kind" then
        error("`errors` takes no option `" .. tostring(field) .. "`", 2)
      end
    end
    check_kind(opts.kind, "kind")
    local kind = opts.kind or "floating"
    gband.action.register(OWN, function()
      open(kind)
    end, { desc = "list the errors" })
    gband.cmd.register(OWN, function(args)
      local requested = type(args) == "table" and args.kind or nil
      check_kind(requested, "kind")
      open(requested or kind)
    end, { desc = "list the errors", args = { "kind" } })
  end,
}
