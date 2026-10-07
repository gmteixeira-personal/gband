gband.keystyle.use()

gband.plugin("gband.errors")
if gband.settings.sidebar() ~= false then
  gband.plugin("gband.sidebar")
end

gband.keymap.set("prefix", "e", gband.action["errors.open"], { desc = "list the errors" })

gband.keymap.set("root", "alt+h", gband.action.focus_column_left, { desc = "focus the column to the left" })
gband.keymap.set("root", "alt+l", gband.action.focus_column_right, { desc = "focus the column to the right" })
gband.bind("alt+n", gband.action.open_window)
gband.unbind("prefix D")

gband.set {
  width_step = 1/20,
  height_step = 1/20,
}

gband.opt.animation_speed = 2

gband.keymap.mode("resize", { label = "resize" })
gband.keymap.set("resize", "h", gband.action.shrink_column_width, { desc = "narrower" })
gband.keymap.set("resize", "l", gband.action.grow_column_width, { desc = "wider" })
gband.keymap.set("resize", "escape", function()
  gband.keymap.enter("root")
end, { desc = "done" })
gband.keymap.set("root", "alt+r", function()
  gband.keymap.enter("resize")
end, { desc = "resize" })

gband.keymap.set("prefix", "t", function()
  gband.spawn({ cmd = "top", after = gband.view().window })
end, { desc = "open top beside the window" })
gband.keymap.set("prefix", "H", function()
  gband.open("https://github.com/gmteixeira-personal/gband/blob/main/docs/tutorial/README.md")
end, { desc = "open the tutorial" })

gband.opt.declare("mark_letters", {
  type = "string",
  default = "abc",
  desc = "the letters that marks take, in order",
})

gband.hl.default("Mark", { fg = "yellow", bold = true })

local marks = {}
local group = gband.augroup("marks")

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
  for letter in gband.opt.mark_letters:gmatch(".") do
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

gband.on("WindowClosed", function(event)
  if marks[event.window] then
    marks[event.window] = nil
    changed(event.window)
  end
end, { group = group })

gband.on("User", function(event)
  print("marks changed", event.data.window, event.data.letter)
end, { group = group, pattern = "marks.changed" })

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

gband.action.register("copy_marks", function()
  local lines = {}
  for _, entry in ipairs(sorted()) do
    lines[#lines + 1] = entry.letter .. " " .. entry.window
  end
  gband.clipboard(table.concat(lines, "\n"))
end, { desc = "copy the marks" })

gband.action.register("close_marked", function()
  for _, entry in ipairs(sorted()) do
    gband.action.close_window({ window = entry.window })
  end
end, { desc = "close the marked windows" })

local list

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

gband.action.register("list_marks", function()
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
end, { desc = "list the marks" })

gband.on("User", function()
  if list then
    local shown = box()
    gband.win.set_config(list, { width = shown.width, height = shown.height })
    gband.win.set_lines(list, shown.lines)
  end
end, { group = group, pattern = "marks.changed" })

gband.keymap.set("prefix", "m", gband.action.mark)
gband.keymap.set("prefix", "y", gband.action.copy_marks)
gband.keymap.set("prefix", "X", gband.action.close_marked)
gband.keymap.set("prefix", "M", gband.action.list_marks)

gband.keymap.set("root", "ctrl+rightmouse", function(event)
  if event.window then
    gband.cmd.run("mark", { window = event.window })
  end
end, { desc = "mark the window under the pointer" })

gband.keymap.set("prefix", "'", function()
  gband.keymap.enter("jump")
end, { desc = "jump to a mark" })
for letter in gband.opt.mark_letters:gmatch(".") do
  gband.keymap.set("jump", letter, function()
    jump(letter)
  end, { desc = "jump to mark " .. letter })
end

gband.keymap.set("prefix", "g", function()
  gband.band.view(gband.layout().bands[1].id)
end, { desc = "view the first band" })
