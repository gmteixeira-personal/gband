# 07 · Plugin windows

This chapter lists the marks in a plugin window: a box of text that your Lua draws over the windows, where `j` and `k` move a cursor line and Enter jumps to the mark under it.

The finished files are in [examples/tutorial/07-plugin-windows/](../../examples/tutorial/07-plugin-windows/).

## What to show

A plugin window shows lines.
A line is a string, or a list of spans, and a span is a string or `{ text = ..., hl = "Group" }`.
Each line of the list shows the letter in the `Mark` group of chapter 06, then where the window is:

```lua
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
```

A plugin window cuts a line that is too long, but `gband.ui.truncate(text, width)` cuts it more politely, ending it with `…`.
`gband.ui.width(text)` measures text the way gband draws it, two cells for a wide character, which `#text` does not.
Together they size the box: as wide as the widest line, but never wider than the ribbon, which `gband.view()` gives as `cols` and `rows`:

```lua
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
```

The `+ 2` makes room for the border.

## Opening it

`gband.win.open` opens a plugin window and returns its number.
`cursorline` draws a cursor line that `j`, `k` and the arrow keys move, and `keys` maps key names to functions of the plugin window's number:

```lua
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
```

`list` is a `local list` declared above the functions, holding the open list's number, so a second Ctrl+Space then `M` focuses the list instead of opening another.
`on_close` forgets it, whether Enter, `q` or Escape closed the list.

The action enters `root` first.
The modal key style's navigation mode is a mode, and while it is active every key goes to its bindings, Enter included; in `root`, the keys that no binding takes reach the focused plugin window.

Bind it, and the list opens over the windows:

```lua
gband.keymap.set("prefix", "M", gband.action.list_marks)
```

```screen tests/screenshots/plugin_windows_spec/the-list-jumps-to-the-line-under-the-cursor--two-marks.txt
I╭─────────────────╮╭─────────────────╮
 │                 ││                 │
1│       ┌marks───────────────┐       │
2│       │a window 2 in band 1│       │
 │       │b window 1 in band 1│       │
 │       └────────────────────┘       │
 │                 ││                 │
 ╰─────────────────╯╰─────────────────╯
```

## Keeping it current

The `marks.changed` event of chapter 04 pays off here.
A second handler redraws the list whenever a mark changes, from anywhere:

```lua
gband.on("User", function()
  if list then
    local shown = box()
    gband.win.set_config(list, { width = shown.width, height = shown.height })
    gband.win.set_lines(list, shown.lines)
  end
end, { group = group, pattern = "marks.changed" })
```

The list could also preview the marked window under its cursor, with `gband.window.focus(window, { peek = true })` from one of its `keys` functions, which shows the window without recording the focus.
Opened with `hover = true`, it could preview as the pointer hovers a line: its `on_mouse` then runs with the kind `"move"` for each cell the pointer crosses, and the list stays focused after the peek.
[Windows and bands](../plugins.md#windows-and-bands-gbandwindow-gbandband) describes the peek.

[Plugin windows](../plugins.md#plugin-windows-gbandwin) describes the tiled kind, which takes a column of the layout, and every option and function.

## The whole file

The complete `user/init.lua` is [examples/tutorial/07-plugin-windows/user/init.lua](../../examples/tutorial/07-plugin-windows/user/init.lua).

Next: [08 · Bars](08-bars.md).
