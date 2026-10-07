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

local marks = {}

local function taken(letter)
  for _, marked in pairs(marks) do
    if marked == letter then
      return true
    end
  end
  return false
end

local function toggle(window)
  if marks[window] then
    marks[window] = nil
    gband.notify("unmarked window " .. window)
    return
  end
  for letter in gband.opt.mark_letters:gmatch(".") do
    if not taken(letter) then
      marks[window] = letter
      gband.notify("marked window " .. window .. " as " .. letter)
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

gband.keymap.set("prefix", "m", gband.action.mark)
gband.keymap.set("prefix", "y", gband.action.copy_marks)
gband.keymap.set("prefix", "X", gband.action.close_marked)

gband.keymap.set("root", "ctrl+rightmouse", function(event)
  if event.window then
    gband.cmd.run("mark", { window = event.window })
  end
end, { desc = "mark the window under the pointer" })
