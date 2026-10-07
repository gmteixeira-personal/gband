local M = { name = "marks", api = 1 }

local KEY = "marks.letter"
local letters
local list
local bar

local function letter_of(window)
  local state = gband.window_state(window)
  return state and state[KEY]
end

local function changed(window)
  gband.emit("marks.changed", { window = window, letter = letter_of(window) })
end

local function windows()
  local found = {}
  for _, band in ipairs(gband.layout().bands) do
    for _, column in ipairs(band.columns) do
      for _, tile in ipairs(column.windows) do
        found[#found + 1] = { window = tile.id, band = band.id }
      end
    end
    for _, box in ipairs(band.floating) do
      found[#found + 1] = { window = box.id, band = band.id }
    end
  end
  return found
end

local function band_of(window)
  for _, entry in ipairs(windows()) do
    if entry.window == window then
      return entry.band
    end
  end
end

local function toggle(window)
  if band_of(window) == nil then
    return
  end
  gband.rpc("marks.toggle", { window = window, letters = gband.opt[letters] }, function(ok, letter)
    if not ok then
      print(letter)
      return
    end
    if letter == false then
      gband.bell()
      return
    end
    if band_of(window) then
      gband.window.rename(window, letter and ("mark " .. letter))
    end
    if letter then
      gband.notify("marked window " .. window .. " as " .. letter)
    else
      gband.notify("unmarked window " .. window)
    end
  end)
end

local function sorted()
  local entries = {}
  for _, entry in ipairs(windows()) do
    local letter = letter_of(entry.window)
    if letter then
      entries[#entries + 1] = { window = entry.window, letter = letter }
    end
  end
  table.sort(entries, function(a, b)
    return a.letter < b.letter
  end)
  return entries
end

local function jump(letter)
  for _, entry in ipairs(sorted()) do
    if entry.letter == letter then
      gband.window.focus(entry.window)
      return
    end
  end
  gband.bell()
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
  local letter = window and letter_of(window)
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
  gband.on("WindowStateChanged", function(event)
    if event.key == KEY then
      changed(event.window)
    end
  end, { group = group })
  gband.on("WindowClosed", function(event)
    changed(event.window)
  end, { group = group })
  gband.on("ServerEvent", function(event)
    gband.notify("window " .. event.data.window .. ", mark " .. event.data.letter .. ", exited")
  end, { group = group, pattern = "marks.exited" })
  gband.on("Attached", function()
    draw(gband.view().window)
  end, { group = group })
  gband.on("ConfigReloaded", function()
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
