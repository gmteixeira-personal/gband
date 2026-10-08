# 05 · Plugin windows

A plugin window is a window whose content a plugin writes: the key list, the error list, the Lua prompt and the settings window are all plugin windows.
It is floating, drawn over the ribbon area in a box of its own, or tiled, drawn in a window of the layout like a terminal window.
`gband.win`, in `defaults/lua/gband/win.lua`, is the longest bundled file.
This chapter reads its first half: the state, line handling, placing, rendering, option checks and the API.
[06](06-window-provider.md) reads the rest, the provider through which the client sends a plugin window its keys, clicks and sizes.

## State

```lua defaults/lua/gband/win.lua
local core = gband.core
local hl = require("gband.hl")
```

```lua defaults/lua/gband/win.lua
gband.hl.default("PluginWindow", {})
gband.hl.default("PluginWindowBorder", { fg = 8 })
gband.hl.default("PluginWindowTitle", { bold = true })
gband.hl.default("PluginWindowCursorLine", { reverse = true })
```

Four groups style every plugin window: the content, the border, the title and the cursor line.
Their defaults use only attributes and palette index 8, so they work with any palette before a theme sets them.

```lua defaults/lua/gband/win.lua
local COMMON = {
  kind = true, lines = true, focus = true, cursorline = true, hover = true,
  keys = true, on_input = true, on_close = true, on_resize = true, on_mouse = true,
}
local FLOATING = { row = true, col = true, width = true, height = true, border = true, title = true }
local TILED = { band = true, after = true, column_width = true }
```

`gband.win.open` takes the fields of `COMMON` for either kind, and those of `FLOATING` or `TILED` for its own kind.

```lua defaults/lua/gband/win.lua
local wins = {}
local dirty = {}
local focused_float = nil
local held = nil
local stack = 0
```

`wins` maps each window's id to its record.
`dirty` is the set of ids whose frame must be presented again, so one change to one window redraws only that window.
`focused_float` is the id of the floating window that has focus, or nil when a terminal window or a tiled plugin window has it.
`held` is the floating window whose own key, input or mouse code is running, which keeps its focus when that code moves the focus elsewhere, as [06](06-window-provider.md) describes.
`stack` counts upwards to give each raised floating window a higher `z` than every other.

## Small helpers

```lua defaults/lua/gband/win.lua
local function clean(text)
  return (text:gsub("[%z\1-\31\127]", ""):gsub("\194[\128-\159]", ""))
end
```

```lua defaults/lua/gband/win.lua
local function valid_group(name)
  return type(name) == "string" and name:match("^[A-Za-z][A-Za-z0-9_.]*$") ~= nil
end
```

```lua defaults/lua/gband/win.lua
local function is_integer(value)
  return math.type(value) == "integer" or (math.type(value) == "float" and value % 1 == 0)
end
```

These three are the same as in `gband.bar`, which [04](04-bars.md) explains.
Each module keeps its own copy, so that replacing one file under `user/lua/gband/` never changes the other.

```lua defaults/lua/gband/win.lua
local function touch(win)
  dirty[win.id] = true
end
```

`touch` marks a window for presenting at the next flush.

```lua defaults/lua/gband/win.lua
local function require_dispatch(name)
  if not core.dispatching() then
    error(name .. " can only be called inside a binding function or another callback", 3)
  end
end
```

Opening, closing and changing a plugin window asks the client to draw, and opening a tiled one asks the server for a window.
That can only happen in a callback, so these functions raise an error at top level.
`gband.core.dispatching` is true while a binding, a handler or another callback runs.

```lua defaults/lua/gband/win.lua
local function require_loaded(name)
  if core.loading() then
    error(name .. " cannot be called while the configuration loads", 3)
  end
end
```

`gband.win.info` and `gband.win.list` can be called at any time except while the configuration loads.

```lua defaults/lua/gband/win.lua
local function lookup(id, name)
  local win = math.type(id) and wins[id] or nil
  if not win then
    error(name .. ": no plugin window " .. tostring(id) .. " is open", 3)
  end
  return win
end
```

`math.type(id)` is nil for anything but a number, so a string id fails the lookup rather than indexing `wins` with it.

## Lines

```lua defaults/lua/gband/win.lua
local function normalise_lines(lines)
  if type(lines) ~= "table" then
    return nil, "`lines` must be a list of lines"
  end
  local result = {}
  for index = 1, #lines do
    local line = lines[index]
    local spans = {}
    if type(line) == "string" then
      spans[1] = { text = clean(line), hl = "PluginWindow" }
    elseif type(line) == "table" then
      for _, span in ipairs(line) do
        if type(span) == "string" then
          spans[#spans + 1] = { text = clean(span), hl = "PluginWindow" }
        elseif type(span) == "table" and type(span.text) == "string"
          and (span.hl == nil or valid_group(span.hl)) then
          spans[#spans + 1] = { text = clean(span.text), hl = span.hl or "PluginWindow" }
        else
          return nil, "line " .. index .. " holds an invalid span"
        end
      end
    else
      return nil, "line " .. index .. " must be a string or a list of spans"
    end
    result[index] = spans
  end
  return result
end
```

This is `gband.bar`'s `normalise_lines` with one difference: every span gets a group, `PluginWindow` when it names none.

## Placing a floating window

```lua defaults/lua/gband/win.lua
local function ribbon()
  local state = core.state()
  return state.ribbon.cols, state.ribbon.rows
end
```

`gband.core.state` returns the ribbon area's size, the screen minus the bars, which is the space floating windows are placed in.

```lua defaults/lua/gband/win.lua
local function place(win)
  local cols, rows = ribbon()
  local width = math.min(win.width or math.max(1, cols // 2), cols)
  local height = math.min(win.height or math.max(1, rows // 2), rows)
  local col = win.col == "center" and (cols - width) // 2 or math.min(win.col, cols - width)
  local row = win.row == "center" and (rows - height) // 2 or math.min(win.row, rows - height)
  win.placed = { row = row, col = col, width = width, height = height }
end
```

A floating window keeps the position and size it was asked for in `row`, `col`, `width` and `height`, and its actual box in `placed`.
A missing width or height is half the ribbon area.
`"center"` centers the box, and a number is pulled back so that the box stays on screen.
Because the request is kept apart from the result, a window that was pulled in when the terminal shrank goes back to its place when it grows.

```lua defaults/lua/gband/win.lua
local function content_size(win)
  if win.kind == "floating" then
    local placed = win.placed
    if win.border then
      return math.max(0, placed.width - 2), math.max(0, placed.height - 2)
    end
    return placed.width, placed.height
  end
  return win.cols, win.rows
end
```

The content is the box minus the border.
A tiled window's size is the size of its drawn window, which `cols` and `rows` hold once the client reports it, and nil before.

```lua defaults/lua/gband/win.lua
local function content_height(win)
  local _, rows = content_size(win)
  return math.max(rows or 1, 1)
end
```

```lua defaults/lua/gband/win.lua
local function clamp(win)
  local count = #win.lines
  local height = content_height(win)
  win.cursor = math.max(1, math.min(win.cursor, math.max(1, count)))
  if win.cursorline then
    if win.cursor < win.top then
      win.top = win.cursor
    elseif win.cursor > win.top + height - 1 then
      win.top = win.cursor - height + 1
    end
  end
  win.top = math.max(1, math.min(win.top, math.max(1, count - height + 1)))
end
```

`clamp` keeps the cursor on a line and `top`, the first line shown, inside the content.
With `cursorline`, the window scrolls to keep the cursor visible.
Without it the cursor is not shown, and only `top` matters.

## Rendering

```lua defaults/lua/gband/win.lua
local function over(style, extra)
  local result = {}
  for field, value in pairs(style) do
    result[field] = value
  end
  for field, value in pairs(extra) do
    result[field] = value
  end
  return result
end
```

```lua defaults/lua/gband/win.lua
local function row_runs(spans, width, base, style)
  local runs = {}
  local used = 0
  local function push(text, run_style)
    if text ~= "" then
      runs[#runs + 1] = { text = text, style = run_style }
    end
  end
  for _, span in ipairs(spans) do
    local pieces = {}
    local full = false
    for char in span.text:gmatch(utf8.charpattern) do
      local cells = gband.ui.width(char)
      if used + cells > width then
        full = true
        break
      end
      pieces[#pieces + 1] = char
      used = used + cells
    end
    push(table.concat(pieces), style(span.hl))
    if full then
      break
    end
  end
  if used < width then
    push(string.rep(" ", width - used), base)
  end
  return runs
end
```

`over` is `gband.bar`'s.
`row_runs` also cuts a line at the width, and then pads the rest of the row with spaces in the row's base style, so a window's background, or its cursor line, spans the whole width.

```lua defaults/lua/gband/win.lua
local function cut(text, width)
  local pieces = {}
  local used = 0
  for char in text:gmatch(utf8.charpattern) do
    local cells = gband.ui.width(char)
    if used + cells > width then
      break
    end
    pieces[#pieces + 1] = char
    used = used + cells
  end
  return table.concat(pieces)
end
```

`cut` cuts a title to a width by the same rule.

```lua defaults/lua/gband/win.lua
local function render(win)
  local cols, rows = content_size(win)
  if cols == nil then
    return
  end
  local base = hl.drawn("PluginWindow")
  local cursor = win.cursorline and hl.drawn("PluginWindowCursorLine") or nil
  local plain, highlighted = {}, {}
  local lines = {}
  for row = 0, rows - 1 do
    local index = win.top + row
    local on_cursor = cursor ~= nil and index == win.cursor and index <= #win.lines
    local cache = on_cursor and highlighted or plain
    local row_base = on_cursor and over(base, cursor) or base
    local function style(group)
      if not cache[group] then
        local merged = over(base, hl.drawn(group))
        cache[group] = on_cursor and over(merged, cursor) or merged
      end
      return cache[group]
    end
    lines[#lines + 1] = row_runs(win.lines[index] or {}, cols, row_base, style)
  end
```

A tiled window not yet opened by the server has no size, and nothing to present.
Each row draws with `PluginWindow`'s style under the span's group, and on the cursor row with `PluginWindowCursorLine` over both.
The two caches keep one merged style per group for plain rows and one for the cursor row.

```lua defaults/lua/gband/win.lua
  if win.kind == "tiled" then
    core.present_window(win.id, {
      kind = "tiled", cols = cols, rows = rows, base = base, lines = lines, hover = win.hover,
    })
    return
  end
```

A tiled frame holds its lines and size; the client draws it in the window the server opened for it.
`hover` tells the client to report pointer moves over the window, which it does only for a frame that sets it.

```lua defaults/lua/gband/win.lua
  local placed = win.placed
  local border_style = over(base, hl.drawn("PluginWindowBorder"))
  local title = nil
  if win.border and win.title then
    title = cut(clean(win.title), math.max(0, placed.width - 2))
  end
  core.present_window(win.id, {
    kind = "floating",
    row = placed.row,
    col = placed.col,
    width = placed.width,
    height = placed.height,
    border = win.border,
    title = title,
    base = base,
    border_style = border_style,
    title_style = over(border_style, hl.drawn("PluginWindowTitle")),
    lines = lines,
    z = win.z,
    focused = focused_float == win.id,
    hover = win.hover,
  })
end
```

A floating frame also carries its box, border, title and their styles, its `z`, whether it has focus, which the client shows with the border, and `hover` as a tiled frame does.
The border style lays `PluginWindowBorder` over the content style, and the title lays `PluginWindowTitle` over the border style, so a title by default is the border's color in bold.

## Callbacks, focus and closing

```lua defaults/lua/gband/win.lua
local function callback(win, field, ...)
  local fn = win[field]
  if fn then
    core.call(win.owner, nil, fn, win.id, ...)
  end
end
```

Every callback a plugin passes, `on_close`, `on_resize`, `on_mouse`, `on_input` and the functions of `keys`, runs through `callback`, with the window's id first.
`gband.core.call` runs it as code of the plugin that opened the window, as `gband.core.owner` recorded it in `open`.

```lua defaults/lua/gband/win.lua
local function resized(win, before_cols, before_rows)
  local cols, rows = content_size(win)
  if cols ~= before_cols or rows ~= before_rows then
    clamp(win)
    callback(win, "on_resize", cols, rows)
  end
end
```

```lua defaults/lua/gband/win.lua
local function unfocus()
  local win = focused_float and wins[focused_float]
  focused_float = nil
  if win then
    touch(win)
  end
end
```

```lua defaults/lua/gband/win.lua
local function raise(win)
  if focused_float ~= win.id then
    unfocus()
  end
  focused_float = win.id
  stack = stack + 1
  win.z = stack
  touch(win)
end
```

Raising a floating window gives it focus and puts it on top of the stack.
Both the window that lost focus and the one that gained it are touched, since each draws its border differently.

```lua defaults/lua/gband/win.lua
local function close(win, run_callback, request)
  wins[win.id] = nil
  dirty[win.id] = nil
  if focused_float == win.id then
    focused_float = nil
  end
  core.forget_window(win.id)
  if request and win.kind == "tiled" then
    core.request({ op = "close", id = win.id })
  end
  if run_callback then
    callback(win, "on_close")
  end
end
```

`close` forgets a window and stops the client drawing it.
`request` says whether to ask the server to close a tiled window's drawn window: when the plugin closes the window it must, but when the server reports that the drawn window closed, there is nothing left to close.

## Checking options

```lua defaults/lua/gband/win.lua
local function check_boolean(opts, field, default)
  local value = opts[field]
  if value == nil then
    return default
  end
  if type(value) ~= "boolean" then
    error("`" .. field .. "` must be a boolean", 3)
  end
  return value
end
```

```lua defaults/lua/gband/win.lua
local function check_border(opts, default)
  local value = opts.border
  if value == nil or type(value) == "boolean" then
    return check_boolean(opts, "border", default)
  end
  if type(value) ~= "table" then
    error("`border` must be a boolean or a border table", 3)
  end
  local border, reason = core.border(value)
  if not border then
    error("`border`: " .. reason, 3)
  end
  return border
end
```

```lua defaults/lua/gband/win.lua
local function check_function(opts, field)
  local value = opts[field]
  if value ~= nil and type(value) ~= "function" then
    error("`" .. field .. "` must be a function", 3)
  end
  return value
end
```

```lua defaults/lua/gband/win.lua
local function check_position(opts, field)
  local value = opts[field]
  if value == nil or value == "center" then
    return "center"
  end
  if type(value) ~= "number" or not is_integer(value) or value < 0 then
    error("`" .. field .. "` must be an integer of at least 0 or \"center\"", 3)
  end
  return math.tointeger(value)
end
```

```lua defaults/lua/gband/win.lua
local function check_size(opts, field)
  local value = opts[field]
  if value == nil then
    return nil
  end
  if type(value) ~= "number" or not is_integer(value) or value < 1 then
    error("`" .. field .. "` must be an integer of at least 1", 3)
  end
  return math.tointeger(value)
end
```

```lua defaults/lua/gband/win.lua
local function check_title(opts)
  local value = opts.title
  if value ~= nil and type(value) ~= "string" then
    error("`title` must be a string", 3)
  end
  return value
end
```

Each check returns the field's value in the form the record keeps, or its default, and raises its error at level 3: level 1 is the check, level 2 the API function, and level 3 the caller's line.
A border table goes through `gband.core.border`, which checks it by the same rules as the border options of terminal windows and returns it in canonical form.

```lua defaults/lua/gband/win.lua
local function check_fields(opts, kind, name)
  for field in pairs(opts) do
    local allowed = COMMON[field] or (kind == "floating" and FLOATING or TILED)[field]
    if not allowed then
      if FLOATING[field] or TILED[field] then
        error(name .. ": `" .. field .. "` does not apply to a " .. kind .. " plugin window", 3)
      end
      error(name .. ": unknown field `" .. tostring(field) .. "`", 3)
    end
  end
end
```

A field of the other kind of window gets a message that says so, rather than "unknown field", since it is likely a slip between the two.

```lua defaults/lua/gband/win.lua
local function check_keys(opts)
  local keys = {}
  if opts.keys == nil then
    return keys
  end
  if type(opts.keys) ~= "table" then
    error("`keys` must be a table from key names to functions", 3)
  end
  for name, fn in pairs(opts.keys) do
    local canonical = type(name) == "string" and core.parse_key(name) or nil
    if not canonical then
      error("invalid key name `" .. tostring(name) .. "` in `keys`", 3)
    end
    if type(fn) ~= "function" then
      error("the key `" .. name .. "` in `keys` must map to a function", 3)
    end
    keys[canonical] = fn
  end
  return keys
end
```

`gband.core.parse_key` turns each key name into the canonical form the client sends, so `keys = { ["Ctrl+x"] = ... }` matches the `ctrl+x` that the provider receives.

## The API

```lua defaults/lua/gband/win.lua
local api = {}
```

```lua defaults/lua/gband/win.lua
function api.open(opts)
  require_dispatch("gband.win.open")
  if opts == nil then
    opts = {}
  end
  if type(opts) ~= "table" then
    error("gband.win.open expects a table of options", 2)
  end
  local kind = opts.kind or "floating"
  local removed = type(kind) == "string" and core.removed('kind = "' .. kind .. '"')
  if removed then
    error(removed, 2)
  end
  if kind ~= "floating" and kind ~= "tiled" then
    error("`kind` must be \"floating\" or \"tiled\"", 2)
  end
```

`gband.core.removed` knows the API names a release removed.
An old plugin that passes `kind = "pane"` is told to write `kind = "tiled"`, rather than being told that `kind` is wrong.

```lua defaults/lua/gband/win.lua
  check_fields(opts, kind, "gband.win.open")
  local lines, reason = normalise_lines(opts.lines or {})
  if not lines then
    error(reason, 2)
  end
```

```lua defaults/lua/gband/win.lua
  local win = {
    kind = kind,
    owner = core.owner(),
    lines = lines,
    top = 1,
    cursor = 1,
    cursorline = check_boolean(opts, "cursorline", false),
    hover = check_boolean(opts, "hover", false),
    keys = check_keys(opts),
    on_input = check_function(opts, "on_input"),
    on_close = check_function(opts, "on_close"),
    on_resize = check_function(opts, "on_resize"),
    on_mouse = check_function(opts, "on_mouse"),
  }
```

The record starts at the first line with the cursor on it.
`hover` is read once here and never changed, since `gband.win.set_config` takes only the box, border and title.
`owner` is the plugin whose code opened the window, which every callback then runs as.

```lua defaults/lua/gband/win.lua
  local focus = check_boolean(opts, "focus", true)
  local request = nil
```

```lua defaults/lua/gband/win.lua
  if kind == "floating" then
    win.row = check_position(opts, "row")
    win.col = check_position(opts, "col")
    win.width = check_size(opts, "width")
    win.height = check_size(opts, "height")
    win.border = check_border(opts, true)
    win.title = check_title(opts)
  else
    local ok, band, after = core.open_target(opts.band, opts.after)
    if not ok then
      error("gband.win.open: " .. band, 2)
    end
    request = { op = "open", band = band, after = after, focus = focus }
    if opts.column_width ~= nil then
      local num, den = core.width(opts.column_width)
      if not num then
        error("`column_width`: " .. den, 2)
      end
      request.num, request.den = num, den
    end
  end
```

A floating window's options are all checked here.
A tiled window's target goes through `gband.core.open_target`, which checks the band and the window to open after against the layout, and `column_width` through `gband.core.width`, which turns a fraction such as `1/3` into a numerator and a denominator.
Nothing has changed yet, so a refused window leaves no trace.

```lua defaults/lua/gband/win.lua
  win.id = core.next_window()
  wins[win.id] = win
  if kind == "floating" then
    place(win)
    clamp(win)
    stack = stack + 1
    win.z = stack
    if focus then
      raise(win)
    end
    touch(win)
  else
    request.id = win.id
    core.request(request)
    clamp(win)
  end
  return win.id
end
```

`gband.core.next_window` hands out an id that this client never used before, not even in an earlier load, so a stale id held by a plugin never reaches a new window.
A floating window is placed, stacked and touched, and appears at the next flush.
A tiled window instead asks the server, through `gband.core.request`, to open a window in the layout.
It draws nothing until the client reports that window and its size, in [06](06-window-provider.md).

```lua defaults/lua/gband/win.lua
function api.close(id)
  require_dispatch("gband.win.close")
  local win = math.type(id) and wins[id] or nil
  if win then
    close(win, true, true)
  end
end
```

Closing a window that does not exist does nothing, so `on_close` handlers can close each other without care.

```lua defaults/lua/gband/win.lua
function api.set_lines(id, lines)
  require_dispatch("gband.win.set_lines")
  local win = lookup(id, "gband.win.set_lines")
  local normalised, reason = normalise_lines(lines)
  if not normalised then
    error(reason, 2)
  end
  win.lines = normalised
  clamp(win)
  touch(win)
end
```

```lua defaults/lua/gband/win.lua
function api.scroll(id, count)
  require_dispatch("gband.win.scroll")
  local win = lookup(id, "gband.win.scroll")
  if type(count) ~= "number" or not is_integer(count) then
    error("gband.win.scroll expects a count as an integer", 2)
  end
  win.top = win.top + math.tointeger(count)
  clamp(win)
  touch(win)
end
```

```lua defaults/lua/gband/win.lua
function api.set_cursor(id, line)
  require_dispatch("gband.win.set_cursor")
  local win = lookup(id, "gband.win.set_cursor")
  if type(line) ~= "number" or not is_integer(line) then
    error("gband.win.set_cursor expects a line as an integer", 2)
  end
  win.cursor = math.tointeger(line)
  clamp(win)
  touch(win)
end
```

Each of these changes the record, clamps it and touches it.
None of them draws: the frame is presented once at the next flush, however many calls a callback makes.

```lua defaults/lua/gband/win.lua
function api.focus(id)
  require_dispatch("gband.win.focus")
  local win = lookup(id, "gband.win.focus")
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

A tiled window is focused through its drawn window, with `gband.core.focus_window`, once the server has opened it.

```lua defaults/lua/gband/win.lua
function api.set_config(id, config)
  require_dispatch("gband.win.set_config")
  local win = lookup(id, "gband.win.set_config")
  if win.kind ~= "floating" then
    error("gband.win.set_config applies only to floating plugin windows", 2)
  end
  if type(config) ~= "table" then
    error("gband.win.set_config expects a table", 2)
  end
  for field in pairs(config) do
    if not FLOATING[field] then
      error("gband.win.set_config: unknown field `" .. tostring(field) .. "`", 2)
    end
  end
```

```lua defaults/lua/gband/win.lua
  local changes = {
    row = config.row ~= nil and check_position(config, "row") or nil,
    col = config.col ~= nil and check_position(config, "col") or nil,
    width = check_size(config, "width"),
    height = check_size(config, "height"),
    border = check_border(config, nil),
    title = check_title(config),
  }
  for field, value in pairs(changes) do
    win[field] = value
  end
  local cols, rows = content_size(win)
  place(win)
  resized(win, cols, rows)
  touch(win)
end
```

`set_config` changes only the fields it is given.
`check_position` returns `"center"` for a missing field, which would move the window, so a position is only checked when present; `check_size`, `check_border` with a nil default and `check_title` already return nil for a missing field, and `pairs` skips nil values.
The content size is measured before and after placing, so `on_resize` runs only when it changed.

```lua defaults/lua/gband/win.lua
local function is_focused(win)
  if win.kind == "floating" then
    return focused_float == win.id
  end
  return focused_float == nil and win.window ~= nil and core.state().window == win.window
end
```

A tiled window has focus when no floating window does and the client's focused window, from `gband.core.state`, is its drawn window.

```lua defaults/lua/gband/win.lua
function api.info(id)
  require_loaded("gband.win.info")
  local win = lookup(id, "gband.win.info")
  local cols, rows = content_size(win)
  local info = {
    id = win.id,
    kind = win.kind,
    focused = is_focused(win),
    window = win.window,
    top = win.top,
    cursor = win.cursor,
    line_count = #win.lines,
    cols = cols,
    rows = rows,
    hover = win.hover,
  }
  if win.kind == "floating" then
    local placed = win.placed
    info.row, info.col, info.width, info.height = placed.row, placed.col, placed.width, placed.height
  end
  return info
end
```

```lua defaults/lua/gband/win.lua
function api.list()
  require_loaded("gband.win.list")
  local ids = {}
  for id in pairs(wins) do
    ids[#ids + 1] = id
  end
  table.sort(ids)
  return ids
end
```

`info` returns a new table each time, and `list` the open ids in ascending order, which is the order they were opened in.
