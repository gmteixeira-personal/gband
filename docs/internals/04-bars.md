# 04 · Bars

A bar is a column of text at the left or right edge of the screen, such as the sidebar.
The client knows how to draw a list of bars it is handed, and how to share the screen's width between them and the windows.
Everything else, from the API that plugins call to deciding when to redraw, is `gband.bar`, in `defaults/lua/gband/bar.lua`.
This chapter reads it in full.

## State

```lua defaults/lua/gband/bar.lua
local core = gband.core
local hl = require("gband.hl")
```

```lua defaults/lua/gband/bar.lua
gband.hl.default("Bar", {})
```

`Bar` is the group every bar draws with unless it names another.
Its default is empty, so a bar takes the terminal's colors until a theme styles it, as `gband.theme` does with `ui.surface`.

```lua defaults/lua/gband/bar.lua
local SIDES = { left = true, right = true }
local FIELDS = {
  id = true, side = true, size = true, order = true, lines = true, hl = true, on_resize = true,
}
local CONFIG = { side = true, size = true, order = true, hl = true }
```

`FIELDS` lists what `gband.bar.add` accepts, and `CONFIG` the subset that `gband.bar.set_config` may change later.
The `id` and the `on_resize` callback are fixed once the bar exists.

```lua defaults/lua/gband/bar.lua
local bars = {}
local sequence = 0
local dirty = true
local presented_height = nil
```

`bars` maps each bar's id to its record.
`sequence` numbers bars in the order they were added, which breaks ties between equal `order`s.
`dirty` says the client's copy of the bars is out of date; it starts `true`, so the first flush presents them.
`presented_height` is the terminal height the last presented bars were cut to.

## Small checks

```lua defaults/lua/gband/bar.lua
local function clean(text)
  return (text:gsub("[%z\1-\31\127]", ""):gsub("\194[\128-\159]", ""))
end
```

`clean` drops control characters from a bar's text: the C0 controls and DEL, and the C1 controls, which UTF-8 encodes as `\194` followed by a byte from 128 to 159.
A bar's width is counted in cells, and a control character would take none while moving the terminal's cursor.
The outer parentheses keep only the first value of `gsub`, the text, and drop its count.

```lua defaults/lua/gband/bar.lua
local function valid_group(name)
  return type(name) == "string" and name:match("^[A-Za-z][A-Za-z0-9_.]*$") ~= nil
end
```

```lua defaults/lua/gband/bar.lua
local function is_integer(value)
  return math.type(value) == "integer" or (math.type(value) == "float" and value % 1 == 0)
end
```

`valid_group` is `gband.hl`'s rule for group names, repeated because `valid_name` is local to `hl.lua`.
`is_integer` accepts `3.0` as well as `3`, as `gband.hl` does for colors.

## Ids and owners

```lua defaults/lua/gband/bar.lua
local function full_id(owner, id)
  if owner == nil then
    return id
  end
  if id == nil then
    return owner
  end
  if id:sub(1, #owner + 1) == owner .. "." then
    return id
  end
  if id:find(".", 1, true) then
    return nil, "`" .. id .. "` is outside the namespace of the plugin `" .. owner .. "`"
  end
  return owner .. "." .. id
end
```

A bar added by a plugin lives in the plugin's namespace.
The plugin `marks` adding the id `list` gets `marks.list`; adding no id gets `marks`; adding `marks.list` already in its namespace keeps it.
Any other id with a full stop would reach into another plugin's namespace, and is refused.
Code of no plugin, such as `user/init.lua`, has `owner` nil and keeps its id as given.

```lua defaults/lua/gband/bar.lua
local function prune()
  for id, bar in pairs(bars) do
    if bar.plugin and core.failed(bar.plugin) then
      bars[id] = nil
      dirty = true
    end
  end
end
```

A plugin that fails, by raising an error in a callback or running past its instruction limit, loses its bars.
`gband.core.failed` tells whether it has, and `prune` runs at the start of nearly every function below, so a failed plugin's bars disappear the next time anyone looks.

```lua defaults/lua/gband/bar.lua
local function lookup(id, name)
  prune()
  local bar = type(id) == "string" and bars[id] or nil
  if not bar then
    error(name .. ": no bar `" .. tostring(id) .. "` exists", 3)
  end
  return bar
end
```

`lookup` raises its error at level 3: level 1 is `lookup`, level 2 the API function that called it, and level 3 the caller's line.

## Lines

```lua defaults/lua/gband/bar.lua
local function normalise_lines(lines)
  if type(lines) ~= "table" then
    return nil, "`lines` must be a list of lines"
  end
  local result = {}
  for index = 1, #lines do
    local line = lines[index]
    local spans = {}
    if type(line) == "string" then
      spans[1] = { text = clean(line) }
    elseif type(line) == "table" then
      for _, span in ipairs(line) do
        if type(span) == "string" then
          spans[#spans + 1] = { text = clean(span) }
        elseif type(span) == "table" and type(span.text) == "string"
          and (span.hl == nil or valid_group(span.hl)) then
          spans[#spans + 1] = { text = clean(span.text), hl = span.hl }
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

A line is a string or a list of spans, and a span a string or `{ text = ..., hl = ... }`.
`normalise_lines` turns every form into the one the rest of the module uses: a list of spans with cleaned text and an optional group.
Like `normalise` in `gband.hl`, it returns nil and a reason, and the caller raises the error with its own name.

```lua defaults/lua/gband/bar.lua
local function check_config(config, fields, name)
  if type(config) ~= "table" then
    error(name .. " expects a table", 3)
  end
  for field in pairs(config) do
    if not fields[field] then
      error(name .. ": unknown field `" .. tostring(field) .. "`", 3)
    end
  end
  if config.side ~= nil and not SIDES[config.side] then
    error(name .. ": `side` must be \"left\" or \"right\", found " .. tostring(config.side), 3)
  end
  if config.size ~= nil and (type(config.size) ~= "number" or not is_integer(config.size)
    or config.size < 1 or config.size > 65535) then
    error(name .. ": `size` must be an integer of at least 1", 3)
  end
  if config.order ~= nil and type(config.order) ~= "number" then
    error(name .. ": `order` must be a number", 3)
  end
  if config.hl ~= nil and not valid_group(config.hl) then
    error(name .. ": `hl` must be a highlight group name", 3)
  end
end
```

`check_config` serves both `gband.bar.add` and `gband.bar.set_config`, each with its own set of fields and its own name for the messages.
Every check leaves nil fields alone, so `set_config` can change one field at a time; `add` checks for its required `side` itself.
`size` is capped at 65535 because the client counts columns in 16 bits.

## Placing bars

```lua defaults/lua/gband/bar.lua
local function ordered()
  local list = {}
  for _, bar in pairs(bars) do
    list[#list + 1] = bar
  end
  table.sort(list, function(a, b)
    if a.order ~= b.order then
      return a.order < b.order
    end
    return a.seq < b.seq
  end)
  return list
end
```

Bars are drawn and placed in order of `order`, then of `seq`, so two bars of the same order keep the order they were added in.

```lua defaults/lua/gband/bar.lua
local function slot(bar)
  return { side = bar.side, size = bar.size, order = bar.order, seq = bar.seq }
end
```

A slot is what `gband.core.place_bars` needs to know of a bar: its side, its width and its place in the order.

```lua defaults/lua/gband/bar.lua
local function placements()
  local state = core.state()
  local list = ordered()
  local slots = {}
  for index, bar in ipairs(list) do
    slots[index] = slot(bar)
  end
  local shown = core.place_bars(slots, state.width)
  local result = {}
  for index, bar in ipairs(list) do
    local columns = shown[index]
    if columns then
      result[bar.id] = { col = columns.col, width = columns.width, height = state.height }
    end
  end
  return result
end
```

`gband.core.place_bars` gives each slot its first column and width, or `false` when the terminal is too narrow to show it beside the ribbon area.
Placing is done in Rust because the client lays out the ribbon area by the same rule, and both must agree on which bars fit.
`placements` turns the result into a table by bar id, with the terminal's height, which every bar spans.

## Presenting bars

```lua defaults/lua/gband/bar.lua
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

`over` lays one style over another, fields of `extra` winning.

```lua defaults/lua/gband/bar.lua
local function row_runs(spans, width, style)
  local runs = {}
  local used = 0
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
    local text = table.concat(pieces)
    if text ~= "" then
      runs[#runs + 1] = { text = text, style = style(span.hl) }
    end
    if full then
      break
    end
  end
  return runs
end
```

`row_runs` cuts a line to the bar's width, counting cells with `gband.ui.width`, so a wide character such as `界` takes two.
A character that would cross the edge ends the line; nothing is wrapped.
Each span becomes a run of text and a style, the form the client draws.

```lua defaults/lua/gband/bar.lua
local function present()
  local height = core.state().height
  local list = {}
  for _, bar in ipairs(ordered()) do
    local base = hl.drawn(bar.hl)
    local styles = {}
    local function style(group)
      group = group or bar.hl
      styles[group] = styles[group] or over(base, hl.drawn(group))
      return styles[group]
    end
```

A bar's base style is its group, resolved with `hl.drawn`.
A span with a group of its own draws with that group's style laid over the base, so a span that sets only `fg` keeps the bar's background.
`styles` caches each group's style for the length of one bar, since a bar of many lines tends to repeat a few groups.

```lua defaults/lua/gband/bar.lua
    local lines = {}
    for row = 1, math.min(#bar.lines, height) do
      lines[row] = row_runs(bar.lines[row], bar.size, style)
    end
    list[#list + 1] = {
      id = bar.id,
      side = bar.side,
      size = bar.size,
      order = bar.order,
      seq = bar.seq,
      base = base,
      lines = lines,
    }
  end
  presented_height = height
  core.present_bars(list)
end
```

Only as many lines as the terminal has rows are drawn.
`gband.core.present_bars` replaces the client's whole list of bars, so every bar is presented again, not only the one that changed.

## Telling a bar its size

```lua defaults/lua/gband/bar.lua
local function notify()
  local placed = placements()
  for _, bar in ipairs(ordered()) do
    local shown = placed[bar.id]
    local width = shown and shown.width or 0
    local height = shown and shown.height or 0
    local last = bar.notified
    if bars[bar.id] == bar and (last == nil or last.width ~= width or last.height ~= height) then
      bar.notified = { width = width, height = height }
      if bar.on_resize then
        core.call(bar.plugin, nil, bar.on_resize, bar.id, width, height)
      end
    end
  end
end
```

A bar that wants to fit its content to its space passes `on_resize`, and the module calls it whenever the bar's width or height changes, including the first time, when `notified` is nil.
A bar that is not shown is told width 0 and height 0.
`gband.core.call` runs the callback as code of the plugin that owns the bar, so its errors are that plugin's errors.
A callback may remove bars, so before each call the loop checks that the bar is still the one stored under its id.

## The API

```lua defaults/lua/gband/bar.lua
local api = {}
```

`api` becomes `gband.bar` at the end of the file.
Building the table first and assigning it last means that `gband.bar` never exists half-built, if any line in between fails.

```lua defaults/lua/gband/bar.lua
function api.add(spec)
  prune()
  check_config(spec, FIELDS, "gband.bar.add")
  if spec.side == nil then
    error("gband.bar.add: `side` must be \"left\" or \"right\"", 2)
  end
  if spec.id ~= nil and (type(spec.id) ~= "string" or spec.id == "") then
    error("gband.bar.add: `id` must be a non-empty string", 2)
  end
  if spec.on_resize ~= nil and type(spec.on_resize) ~= "function" then
    error("gband.bar.add: `on_resize` must be a function", 2)
  end
  local lines = {}
  if spec.lines ~= nil then
    local reason
    lines, reason = normalise_lines(spec.lines)
    if not lines then
      error("gband.bar.add: " .. reason, 2)
    end
  end
```

`add` checks every field before it changes anything, so a refused bar leaves no trace.

```lua defaults/lua/gband/bar.lua
  local owner = core.owner()
  if owner == nil and spec.id == nil then
    error("gband.bar.add: a bar added outside a plugin must have an `id`", 2)
  end
  local id, reason = full_id(owner, spec.id)
  if not id then
    error("gband.bar.add: " .. reason, 2)
  end
  if bars[id] then
    error("gband.bar.add: a bar `" .. id .. "` already exists", 2)
  end
```

Code of no plugin must name its bar, since it has no plugin name to fall back on.

```lua defaults/lua/gband/bar.lua
  sequence = sequence + 1
  bars[id] = {
    id = id,
    plugin = owner,
    side = spec.side,
    size = spec.size and math.tointeger(spec.size) or 1,
    order = spec.order or 0,
    lines = lines,
    hl = spec.hl or "Bar",
    on_resize = spec.on_resize,
    seq = sequence,
  }
  dirty = true
  return id
end
```

The record stores the size as an integer, the order defaulting to 0 and the group to `Bar`.
Adding a bar only marks the bars dirty; nothing is drawn until the next flush.

```lua defaults/lua/gband/bar.lua
function api.set_lines(id, lines)
  local bar = lookup(id, "gband.bar.set_lines")
  local normalised, reason = normalise_lines(lines)
  if not normalised then
    error("gband.bar.set_lines: " .. reason, 2)
  end
  bar.lines = normalised
  dirty = true
end
```

```lua defaults/lua/gband/bar.lua
function api.set_config(id, config)
  local bar = lookup(id, "gband.bar.set_config")
  check_config(config, CONFIG, "gband.bar.set_config")
  for field in pairs(CONFIG) do
    if config[field] ~= nil then
      bar[field] = field == "size" and math.tointeger(config.size) or config[field]
    end
  end
  dirty = true
end
```

`set_lines` replaces a bar's lines, and `set_config` its side, size, order or group.
Neither draws, both mark the bars dirty.

```lua defaults/lua/gband/bar.lua
function api.remove(id)
  prune()
  if type(id) ~= "string" or not bars[id] then
    return false
  end
  bars[id] = nil
  dirty = true
  return true
end
```

`remove` returns whether there was such a bar, rather than raising an error, so removing twice is harmless.

```lua defaults/lua/gband/bar.lua
local function info(bar, placed)
  local shown = placed[bar.id]
  return {
    id = bar.id,
    side = bar.side,
    size = bar.size,
    order = bar.order,
    hl = bar.hl,
    plugin = bar.plugin,
    shown = shown ~= nil,
    col = shown and shown.col,
    width = shown and shown.width,
    height = shown and shown.height,
  }
end
```

```lua defaults/lua/gband/bar.lua
function api.info(id)
  local bar = lookup(id, "gband.bar.info")
  return info(bar, placements())
end
```

```lua defaults/lua/gband/bar.lua
function api.list()
  prune()
  local placed = placements()
  local list = {}
  for _, bar in pairs(bars) do
    list[#list + 1] = info(bar, placed)
  end
  table.sort(list, function(a, b)
    return a.id < b.id
  end)
  return list
end
```

`info` and `list` describe bars as tables, including where each one is shown right now.
They place the bars on every call rather than reading the last presented placement, so they are right even before the next draw.

## The bars provider

```lua defaults/lua/gband/bar.lua
local hooks = {}
```

```lua defaults/lua/gband/bar.lua
function hooks.flush()
  prune()
  if core.loading() then
    return
  end
  notify()
  prune()
  if core.state().height ~= presented_height then
    dirty = true
  end
  if dirty then
    dirty = false
    present()
  end
end
```

The client calls `flush` before it draws.
Nothing is presented while the configuration loads: the bars are not complete until it has finished.
`notify` runs first, because an `on_resize` callback usually sets new lines.
The second `prune` drops the bars of a plugin whose callback just failed.
Presented lines are cut to the terminal's height, so a change of height presents again even when no bar changed.

```lua defaults/lua/gband/bar.lua
core.provide("bars", hooks)
```

`gband.core.provide` registers the hooks.
Without this line, gband draws no bars at all.

```lua defaults/lua/gband/bar.lua
core.after_event(function(name)
  if name == nil or name == "HighlightChanged" or name == "ColorschemeChanged" then
    dirty = true
  end
end)
```

`gband.core.after_event` runs after the handlers of every built-in event.
A changed group or colorscheme changes how bars look without changing any bar, so it marks them dirty.
The name is nil when a load finishes or the plugins are refreshed, which also calls for presenting again.

```lua defaults/lua/gband/bar.lua
gband.bar = api
```

`gband.bar` has no error marker of its own.
The sidebar draws one in its bar and calls `gband.core.error_marker` to tell the client so, as [09](09-sidebar-and-errors.md) shows.
