gband.keystyle.use()

gband.plugin("gband.errors")
gband.plugin("gband.sidebar")

gband.keymap.set("prefix", "e", gband.action["errors.open"], { desc = "list the errors" })

gband.keymap.set("root", "alt+h", gband.action.focus_column_left, { desc = "focus the column to the left" })
gband.keymap.set("root", "alt+l", gband.action.focus_column_right, { desc = "focus the column to the right" })
gband.bind("alt+n", gband.action.open_window)
gband.unbind("prefix D")

gband.keymap.mode("resize", { label = "resize" })
gband.keymap.set("resize", "h", gband.action.shrink_column_width, { desc = "narrower" })
gband.keymap.set("resize", "l", gband.action.grow_column_width, { desc = "wider" })
gband.keymap.set("resize", "escape", function()
  gband.keymap.enter("root")
end, { desc = "done" })
gband.keymap.set("root", "alt+r", function()
  gband.keymap.enter("resize")
end, { desc = "resize" })

local marks = {}
local letters = "abc"

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
    print("unmarked window " .. window)
    return
  end
  for letter in letters:gmatch(".") do
    if not taken(letter) then
      marks[window] = letter
      print("marked window " .. window .. " as " .. letter)
      return
    end
  end
end

gband.keymap.set("prefix", "m", function()
  local window = gband.view().window
  if window then
    toggle(window)
  end
end, { desc = "mark the window" })

gband.keymap.set("root", "ctrl+rightmouse", function(event)
  if event.window then
    toggle(event.window)
  end
end, { desc = "mark the window under the pointer" })

gband.keymap.set("root", "ctrl+leftmouse", function(event)
  if event.target == "window" and event.box_row == 0 then
    toggle(event.window)
    return
  end
  return false
end, { desc = "mark the window by its top border" })
