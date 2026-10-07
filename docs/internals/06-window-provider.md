# 06 · The windows provider

[05](05-plugin-windows.md) read how a plugin opens and changes a plugin window.
This chapter reads the other direction: the windows provider, the table of hooks through which the client tells `gband.win` that a key, a click, a paste or a resize reached one of its windows.
It is the rest of `defaults/lua/gband/win.lua`, from `LINE_STEPS` to the end.

## Moving through lines

```lua defaults/lua/gband/win.lua
local LINE_STEPS = { up = -1, k = -1, down = 1, j = 1 }
```

```lua defaults/lua/gband/win.lua
local function move(win, name)
  local height = content_height(win)
  local count = #win.lines
  local step = LINE_STEPS[name]
  if step then
    if win.cursorline then
      win.cursor = win.cursor + step
    else
      win.top = win.top + step
    end
  elseif name == "pageup" or name == "pagedown" then
    local page = name == "pageup" and -height or height
    win.top = win.top + page
    if win.cursorline then
      win.cursor = win.cursor + page
    end
  elseif name == "home" then
    win.top = 1
    if win.cursorline then
      win.cursor = 1
    end
  elseif name == "end" then
    win.top = count
    if win.cursorline then
      win.cursor = count
    end
  else
    return false
  end
  clamp(win)
  touch(win)
  return true
end
```

`move` is what a plugin window does with a navigation key that the plugin did not bind.
`j` and `k` move as the arrows do, so a plugin window reads like a pager.
With `cursorline` the cursor moves and `clamp` scrolls to follow it; without it the window scrolls.
Moving past either end is not checked here, because `clamp` pulls both `top` and `cursor` back.
`move` returns whether the key was one of its keys, so the caller knows whether anything handled it.

## Keys

```lua defaults/lua/gband/win.lua
local hooks = {}
```

`hooks` is the provider: every function in it is one the client calls.
[The plugin guide](../plugins.md#settings-and-providers) lists them with the moment each is called.

```lua defaults/lua/gband/win.lua
local function hold(id)
  if focused_float == id then
    held = id
  end
end
```

`hold` records that the focused floating window's own code is acting.
The last section shows what that protects.

```lua defaults/lua/gband/win.lua
function hooks.key(id, name, text)
  local win = wins[id]
  if not win then
    return
  end
  local fn = win.keys[name]
  if fn then
    hold(id)
    core.call(win.owner, nil, fn, id)
    return
  end
```

The client calls `key` for every key that reaches the focused plugin window, with its canonical name, such as `ctrl+a`, and the text it types, if any.
A binding in `keys` wins over everything else.

```lua defaults/lua/gband/win.lua
  if text and win.on_input then
    hold(id)
    callback(win, "on_input", text)
    return
  end
  if move(win, name) then
    return
  end
  if (name == "escape" or name == "q") and win.kind == "floating" then
    close(win, true, true)
  end
end
```

Without a binding, a key that types text goes to `on_input`, which is how the Lua prompt receives what you type.
Then come the navigation keys, and last `escape` and `q`, which close a floating window that bound neither.
A tiled window is a window of the layout, and closes like one.

## The mouse

```lua defaults/lua/gband/win.lua
local WHEEL_STEPS = { up = "up", down = "down" }
```

```lua defaults/lua/gband/win.lua
local function focus_from_mouse(win)
  if win.kind == "floating" then
    raise(win)
    return
  end
  unfocus()
  if win.window then
    core.focus_window(win.window)
  end
end
```

A press focuses the window under it: a floating window is raised, and a tiled one gets focus through its drawn window, exactly as `gband.win.focus` does.

```lua defaults/lua/gband/win.lua
function hooks.mouse(id, event)
  local win = wins[id]
  if not win then
    return
  end
  if event.kind == "press" then
    focus_from_mouse(win)
  end
  if event.content_row ~= nil and win.top + event.content_row <= #win.lines then
    event.line = win.top + event.content_row
  end
```

The client reports where a mouse event landed inside the content, as `content_row`, counted from 0, or nil on the border.
`mouse` adds `line`, the line of the window's text under the pointer, when there is one, so that a plugin need not know how far the window has scrolled.

```lua defaults/lua/gband/win.lua
  if win.on_mouse then
    if win.kind == "floating" and event.kind ~= "scroll" then
      hold(id)
    end
    core.call(win.owner, nil, win.on_mouse, id, event)
    return
  end
```

A window with `on_mouse` handles every event itself.
A press or drag in a floating window holds it, so that the focus changes its handler causes do not take focus from it.

```lua defaults/lua/gband/win.lua
  if event.kind == "scroll" then
    local name = WHEEL_STEPS[event.direction]
    if name then
      move(win, name)
    end
  elseif event.kind == "press" and event.button == "left" and win.cursorline and event.line then
    win.cursor = event.line
    clamp(win)
    touch(win)
  end
end
```

Without `on_mouse`, the wheel moves through the lines as the arrow keys do, and a left click puts the cursor on the line under it.

```lua defaults/lua/gband/win.lua
function hooks.set_box(id, col, row, width, height)
  local win = wins[id]
  if not win or win.kind ~= "floating" then
    return
  end
  local cols, rows = content_size(win)
  win.col, win.row, win.width, win.height = col, row, width, height
  place(win)
  resized(win, cols, rows)
  touch(win)
end
```

The client moves and resizes floating windows itself when you drag their border, and reports the new box through `set_box`.
It becomes the window's requested box, so it survives a resize of the terminal.

```lua defaults/lua/gband/win.lua
function hooks.raise(id)
  local win = wins[id]
  if win then
    focus_from_mouse(win)
  end
end
```

```lua defaults/lua/gband/win.lua
function hooks.unfocus()
  unfocus()
end
```

The client calls `raise` when a press focuses a plugin window, and `unfocus` when focus moves to a window of the layout, so no floating window has it.

## Pasting, closing and releasing

```lua defaults/lua/gband/win.lua
function hooks.paste(id, text)
  local win = wins[id]
  if win and win.on_input then
    hold(id)
    callback(win, "on_input", text)
  end
end
```

A paste reaches `on_input` as one piece of text, never the key bindings.

```lua defaults/lua/gband/win.lua
function hooks.close_focused()
  local win = focused_float and wins[focused_float]
  if not win then
    return false
  end
  close(win, true, true)
  return true
end
```

The window close action calls `close_focused` first.
When a floating plugin window has focus, it closes that and the action closes nothing else; `false` lets the action close the focused terminal window as usual.

```lua defaults/lua/gband/win.lua
function hooks.release()
  held = nil
end
```

## The drawn window of a tiled window

A tiled plugin window lives in a window of the layout, which the server opens.
Three hooks follow that window's life.

```lua defaults/lua/gband/win.lua
function hooks.opened(id, window)
  local win = wins[id]
  if not win then
    return
  end
  if window == nil then
    close(win, true, false)
    return
  end
  win.window = window
end
```

`opened` answers the request that `gband.win.open` sent.
When the server could not open the window, the plugin window closes, running `on_close` but asking the server for nothing.

```lua defaults/lua/gband/win.lua
function hooks.window_resized(id, cols, rows)
  local win = wins[id]
  if not win then
    return
  end
  local before_cols, before_rows = win.cols, win.rows
  win.cols, win.rows = cols, rows
  resized(win, before_cols, before_rows)
  touch(win)
end
```

`window_resized` gives the window its size, and with it something to render: [05](05-plugin-windows.md)'s `render` presents nothing until `cols` is set.
The first call always counts as a resize, since the size was nil before, so `on_resize` tells the plugin its size as soon as it has one.

```lua defaults/lua/gband/win.lua
function hooks.window_closed(id)
  local win = wins[id]
  if win then
    close(win, true, false)
  end
end
```

When you close a tiled plugin window with the window close action, the server closes its drawn window and the client reports it here.

## The ribbon area

```lua defaults/lua/gband/win.lua
function hooks.ribbon_resized()
  for _, win in pairs(wins) do
    if win.kind == "floating" then
      local cols, rows = content_size(win)
      place(win)
      resized(win, cols, rows)
      touch(win)
    end
  end
end
```

When the terminal or the bars change size, every floating window is placed again from its requested box.
A window that had to shrink to fit returns to its size once there is room.

## Questions from the client

```lua defaults/lua/gband/win.lua
function hooks.focused()
  if focused_float then
    return focused_float
  end
  local window = core.state().window
  if window == nil then
    return nil
  end
  return hooks.plugin_window_of(window)
end
```

```lua defaults/lua/gband/win.lua
function hooks.plugin_window_of(window)
  for id, win in pairs(wins) do
    if win.window == window then
      return id
    end
  end
  return nil
end
```

The client asks these to route keys and the mouse.
A focused floating window has focus over everything; otherwise the focused window of the layout may be a tiled plugin window's drawn window.

## Flushing

```lua defaults/lua/gband/win.lua
function hooks.flush()
  for id, win in pairs(wins) do
    if win.owner and core.failed(win.owner) then
      close(win, false, true)
    end
  end
  local pending = dirty
  dirty = {}
  for id in pairs(pending) do
    local win = wins[id]
    if win then
      render(win)
    end
  end
end
```

The client calls `flush` after each callback and before it draws.
A failed plugin's windows close first, without running their `on_close`, which is the failed plugin's own code.
Then every touched window is rendered once.
`dirty` is swapped for a new table before rendering, so a window touched while rendering is presented at the next flush rather than lost.

## Registering

```lua defaults/lua/gband/win.lua
core.provide("windows", hooks)
```

```lua defaults/lua/gband/win.lua
core.after_event(function(name)
  if name == "FocusChanged" or name == "BandChanged" then
    if held == nil or held ~= focused_float then
      unfocus()
    end
  elseif name == nil or name == "HighlightChanged" or name == "ColorschemeChanged" then
    for _, win in pairs(wins) do
      touch(win)
    end
  end
end)
```

After every built-in event, the module checks two things.
A `FocusChanged` or `BandChanged` means focus went to a window of the layout, so the floating window loses focus, unless it is held: its own key, input or mouse code caused the change, which is not a reason for it to lose focus.
A changed highlight or colorscheme, or the end of a load, redraws every window.

```lua defaults/lua/gband/win.lua
gband.win = api
```

`gband.win` is assigned last, as `gband.bar` is.
