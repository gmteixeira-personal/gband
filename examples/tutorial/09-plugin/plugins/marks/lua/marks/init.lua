local M = { name = "marks", api = 2 }

local marks = {}
local letters
local list
local bar

local function changed(window)
  gband.emit("marks.changed", { window = window, letter = marks[window] })
end

local function taken(letter)
  for _, marked in pairs(marks) do
    if marked == letter then
      return true
    end
  end
  return false
end

local function band_of(window)
  for _, band in ipairs(gband.layout().bands) do
    for _, column in ipairs(band.columns) do
      for _, tile in ipairs(column.windows) do
        if tile.id == window then
          return band.id
        end
      end
    end
    for _, box in ipairs(band.floating) do
      if box.id == window then
        return band.id
      end
    end
  end
end

local function toggle(window)
  if band_of(window) == nil then
    return
  end
  if marks[window] then
    marks[window] = nil
    gband.window.rename(window, nil)
    gband.notify("unmarked window " .. window)
    changed(window)
    return
  end
  for letter in gband.opt[letters]:gmatch(".") do
    if not taken(letter) then
      marks[window] = letter
      gband.window.rename(window, "mark " .. letter)
      gband.notify("marked window " .. window .. " as " .. letter)
      changed(window)
      return
    end
  end
  gband.bell()
end

local function jump(letter)
  for window, marked in pairs(marks) do
    if marked == letter then
      gband.window.focus(window)
      return
    end
  end
  gband.bell()
end

local function sorted()
  local entries = {}
  for window, letter in pairs(marks) do
    entries[#entries + 1] = { window = window, letter = letter }
  end
  table.sort(entries, function(a, b)
    return a.letter < b.letter
  end)
  return entries
end

local function describe(window)
  return "window " .. window .. " in band " .. band_of(window)
end

local function lines(width)
  local result = {}
  for _, entry in ipairs(sorted()) do
    result[#result + 1] = {
      { text = entry.letter, hl = "Mark" },
      " " .. gband.ui.truncate(describe(entry.window), width - 2),
    }
  end
  if #result == 0 then
    result[1] = "no marks"
  end
  return result
end

local function box()
  local view = gband.view()
  local entries = sorted()
  local width = gband.ui.width("no marks")
  for _, entry in ipairs(entries) do
    width = math.max(width, 2 + gband.ui.width(describe(entry.window)))
  end
  width = math.min(width, view.cols - 2)
  local height = math.min(math.max(#entries, 1), view.rows - 2)
  return { width = width + 2, height = height + 2, lines = lines(width) }
end

local function open_list()
  gband.keymap.enter("root")
  if list then
    gband.win.focus(list)
    return
  end
  local shown = box()
  list = gband.win.open({
    title = "marks",
    width = shown.width,
    height = shown.height,
    cursorline = true,
    lines = shown.lines,
    keys = {
      enter = function(win)
        local entry = sorted()[gband.win.info(win).cursor]
        gband.win.close(win)
        if entry then
          gband.window.focus(entry.window)
        end
      end,
    },
    on_close = function()
      list = nil
    end,
  })
end

local function draw(window)
  if bar == nil then
    return
  end
  local letter = window and marks[window]
  gband.bar.set_lines(bar, { letter and { { text = letter, hl = "Mark" } } or "" })
end

function M.setup(opts)
  assert(gband.side == "client", "set marks up in user/init.lua")
  if opts.bar ~= nil and opts.bar ~= false and opts.bar ~= "left" and opts.bar ~= "right" then
    error("`bar` is \"left\", \"right\" or false, found " .. tostring(opts.bar))
  end

  letters = gband.opt.declare("letters", {
    type = "string",
    default = "abc",
    desc = "the letters that marks take, in order",
  })

  gband.action.register("mark", function()
    local window = gband.view().window
    if window then
      toggle(window)
    end
  end, { desc = "mark the window" })

  gband.cmd.register("mark", function(args)
    local window = args.window or gband.view().window
    if window then
      toggle(window)
    end
  end, { desc = "mark a window", args = { "window" } })

  gband.action.register("copy", function()
    local copied = {}
    for _, entry in ipairs(sorted()) do
      copied[#copied + 1] = entry.letter .. " " .. entry.window
    end
    gband.clipboard(table.concat(copied, "\n"))
  end, { desc = "copy the marks" })

  gband.action.register("close", function()
    for _, entry in ipairs(sorted()) do
      gband.action.close_window({ window = entry.window })
    end
  end, { desc = "close the marked windows" })

  gband.action.register("list", open_list, { desc = "list the marks" })

  gband.action.register("jump", function()
    gband.keymap.enter("marks")
  end, { desc = "jump to a mark" })
  for letter in gband.opt[letters]:gmatch(".") do
    gband.keymap.set("marks", letter, function()
      jump(letter)
    end, { desc = "jump to mark " .. letter })
  end

  if opts.bar ~= false then
    bar = gband.bar.add({ side = opts.bar or "left", size = 1, order = 1 })
  end

  local group = gband.augroup("marks")
  gband.on("WindowClosed", function(event)
    if marks[event.window] then
      marks[event.window] = nil
      changed(event.window)
    end
  end, { group = group })
  gband.on("Attached", function()
    draw(gband.view().window)
  end, { group = group })
  gband.on("FocusChanged", function(event)
    draw(event.window)
  end, { group = group })
  gband.on("User", function()
    draw(gband.view().window)
    if list then
      local shown = box()
      gband.win.set_config(list, { width = shown.width, height = shown.height })
      gband.win.set_lines(list, shown.lines)
    end
  end, { group = group, pattern = "marks.changed" })
end

return M
