# 09 · The sidebar and the error list

The API modules are done.
What remains of gband's Lua is built on them exactly as your own plugins are: the bundled plugins use the public API, `gband.bar`, `gband.win`, `gband.on` and the rest, and reach `gband.core` only where the public API has nothing to offer.
This chapter reads two of them in full: the sidebar, `defaults/lua/gband/sidebar.lua`, and the error list, `defaults/lua/gband/errors.lua`.

## The sidebar

The sidebar is a bar one column wide: the mode letter at the top, a label for each band below it, and `!` at the bottom when there is an error.

```lua defaults/lua/gband/sidebar.lua
local core = gband.core
```

```lua defaults/lua/gband/sidebar.lua
gband.hl.default("SidebarMode", { bold = true })
gband.hl.default("SidebarBand", { dim = true })
gband.hl.default("SidebarBandActive", { bold = true })
gband.hl.default("SidebarError", { fg = 1, bold = true })
```

Its groups get defaults that work without a theme: the active band bold, the others dim, the error marker red.

```lua defaults/lua/gband/sidebar.lua
local BAR = "sidebar"
local OPTIONS = { side = true, order = true }
local LETTERS = "abcdefghijklmnopqrstuvwxyz"
local MARKER_ROWS = 1
local FIRST_BAND_ROW = 2
local WHEEL = { down = "focus_band_down", up = "focus_band_up" }
```

`BAR` is the bar's id, which is the plugin's name, since the bar is added without an id from inside the plugin's `setup`, as [04](04-bars.md) explains.
Bands are labelled `1` to `9`, then `a` to `z`.
Rows are counted from 0, as the mouse events count them: row 0 holds the mode letter, row 1 is empty, the bands start at row 2, and one row at the bottom is kept for the marker.

```lua defaults/lua/gband/sidebar.lua
local APPS = "∷"
```

`APPS` is the apps character that row 0 shows with the floating key style of [08](08-key-styles.md), in place of the mode letter.
`∷`, U+2237, is East Asian ambiguous width: `gband.ui.width` counts it as one cell, as it counts every ambiguous character, and a terminal that draws such characters two cells wide already draws the borders wide too.
`⸬`, U+2E2C, looks alike and has neutral width, for a copy of the sidebar whose fonts hold it.

```lua defaults/lua/gband/sidebar.lua
local placed = false
```

`placed` turns true once `setup` has added the bar; until then there is nothing to draw in.

### What it shows

```lua defaults/lua/gband/sidebar.lua
local function floating_style()
  return gband.keystyle.current() == "floating"
end
```

The sidebar knows nothing of the presets: it asks `gband.keystyle.current()` which style `use` picked in this load.

```lua defaults/lua/gband/sidebar.lua
local function mode_letter(active)
  if floating_style() then
    return APPS
  end
  if active == "root" then
    return "I"
  end
  local label = gband.keymap.label(active)
  local letter = label:match("^" .. utf8.charpattern) or ""
  if letter:match("^%l$") then
    letter = letter:upper()
  end
  return letter
end
```

With the floating style, row 0 always shows `∷`: its `prefix` table is active only until the leader handler returns to `root`, so a letter would always read `I`.
Otherwise the mode letter is `I` in `root`, which the key styles call interactive mode.
Otherwise it is the first character of the table's label, from `gband.keymap.label`, as a capital: `N` for the modal style's `navigation`.
A table that is not a mode has no label, and `label` returns its name.

```lua defaults/lua/gband/sidebar.lua
local function band_label(position)
  if position <= 9 then
    return tostring(position)
  end
  if position <= 9 + #LETTERS then
    return LETTERS:sub(position - 9, position - 9)
  end
  return nil
end
```

```lua defaults/lua/gband/sidebar.lua
local function position_at(row, height)
  local position = row - FIRST_BAND_ROW + 1
  if position < 1 or row > height - 1 - MARKER_ROWS or band_label(position) == nil then
    return nil
  end
  return position
end
```

`position_at` turns a row into a band's position, or nil for a row that holds no band: above the bands, on the marker's row, or past the last label.
Both drawing and clicking use it, so a band can be clicked exactly where it is drawn.

```lua defaults/lua/gband/sidebar.lua
local function lines_for(state, height)
  local lines = {}
  for row = 1, height do
    lines[row] = {}
  end
  local marker = state.error ~= nil
  if height == 1 then
    if marker then
      lines[1] = { { text = "!", hl = "SidebarError" } }
    else
      lines[1] = { { text = mode_letter(state.table), hl = "SidebarMode" } }
    end
    return lines, marker
  end
```

`lines_for` builds one line per row of the bar.
A bar one row high shows only the marker, or the mode letter when there is no error.

```lua defaults/lua/gband/sidebar.lua
  lines[1] = { { text = mode_letter(state.table), hl = "SidebarMode" } }
  if marker then
    lines[height] = { { text = "!", hl = "SidebarError" } }
  end
  for position = 1, state.band.count do
    local row = FIRST_BAND_ROW + position - 1
    if position_at(row, height) == nil then
      break
    end
    local hl = position == state.band.index and "SidebarBandActive" or "SidebarBand"
    lines[row + 1] = { { text = band_label(position), hl = hl } }
  end
  return lines, marker
end
```

Otherwise the mode letter goes on the first line, the marker on the last, and a label for each band that fits between them, the viewed band in `SidebarBandActive`.
`row` counts from 0 and `lines` from 1, hence `row + 1`.

### Drawing

```lua defaults/lua/gband/sidebar.lua
local function draw()
  if not placed then
    return
  end
  local state = core.state()
  local info = gband.bar.info(BAR)
  local height = info.height or state.height
  if height == 0 then
    core.error_marker(false)
    return
  end
  local lines, marker = lines_for(state, height)
  gband.bar.set_lines(BAR, lines)
  core.error_marker(marker and info.shown)
end
```

`draw` reads the client's state with `gband.core.state`: the active key table, the bands and the latest error.
The bar's height comes from `gband.bar.info`, which is 0 while the bar is not shown.
`gband.core.error_marker` tells the client whether the marker is on screen.
While it is, the client draws no error banner over the windows; a sidebar hidden by a narrow terminal leaves the banner to the client.

```lua defaults/lua/gband/sidebar.lua
core.on_state(draw)
```

`gband.core.on_state` calls `draw` whenever the key table, the viewed band, the number of bands or the error list changes.
One function then covers everything the sidebar shows.

### Clicks and the wheel

```lua defaults/lua/gband/sidebar.lua
local function clicked(ev)
  if ev.button ~= "left" or ev.target ~= "outside" then
    return
  end
  local info = gband.bar.info(BAR)
  if not info.shown or ev.col ~= info.col then
    return
  end
  if ev.row == 0 and floating_style() and (info.height > 1 or core.state().error == nil) then
    local list = gband.action["desktop.list"]
    if list then
      list()
    end
    return
  end
  local position = position_at(ev.row, info.height)
  local band = position and gband.layout().bands[position]
  if band then
    gband.band.view(band.id)
  end
end
```

A press that lands on no window has the target `outside`.
When it is in the sidebar's column, on a band's row, the sidebar views that band.
On row 0 with the floating style, while row 0 shows `∷` and not the error marker of a one-row bar, it dispatches `desktop.list` to open the window list, and only when that action exists, so a sidebar without the desktop plugin costs nothing.

```lua defaults/lua/gband/sidebar.lua
local function scrolled(ev)
  local action = WHEEL[ev.direction]
  if not action or ev.target ~= "outside" or ev.ctrl or ev.alt or ev.shift then
    return
  end
  local info = gband.bar.info(BAR)
  if info.shown and ev.col == info.col then
    gband.action[action]()
  end
end
```

The wheel over the sidebar, with no modifier held, views the band below or above.

### Setting up

```lua defaults/lua/gband/sidebar.lua
local function check_options(opts)
  if type(opts) ~= "table" then
    error("the options of `sidebar` must be a table", 2)
  end
  for field in pairs(opts) do
    if not OPTIONS[field] then
      error("unknown option `" .. tostring(field) .. "`", 2)
    end
  end
  local side = opts.side or "left"
  if side ~= "left" and side ~= "right" then
    error("`side` must be \"left\" or \"right\", found " .. tostring(opts.side), 2)
  end
  if opts.order ~= nil and type(opts.order) ~= "number" then
    error("`order` must be a number", 2)
  end
  return side, opts.order or 0
end
```

```lua defaults/lua/gband/sidebar.lua
return {
  name = "sidebar",
  setup = function(opts)
    local side, order = check_options(opts)
    gband.bar.add({
      side = side,
      size = 1,
      order = order,
      on_resize = function()
        draw()
      end,
    })
    gband.on("MousePressed", clicked)
    gband.on("MouseScrolled", scrolled)
    placed = true
    if not core.loading() then
      draw()
    end
  end,
}
```

The file returns a plugin module, so `gband.plugin("gband.sidebar")` sets it up, with `name` giving the plugin's name.
`setup` adds the bar and registers the mouse handlers, all of which then belong to the plugin `sidebar`.
`on_resize` draws whenever the bar's size changes, which also draws it the first time it is placed.
While the configuration loads, no bar is placed yet, so `setup` leaves the first draw to `on_resize`.

## The error list

The error list shows every error message in a plugin window. Neither key style binds it; you bind its action, `errors.open`, or run its command.

```lua defaults/lua/gband/errors.lua
local OWN = "errors.open"
local CLEAR = "errors.clear"
local TITLE = "errors"
local HINTED = "errors  c clear"
local KINDS = { floating = true, tiled = true }
```

The names `errors.open` and `errors.clear` already carry the plugin's namespace, so registering them keeps them as they are.
`HINTED` is the title shown while there is something to clear.

```lua defaults/lua/gband/errors.lua
local current = nil
local texts = {}
```

`current` is the open list's id, and `texts` the messages it shows.

```lua defaults/lua/gband/errors.lua
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
```

`wrap` breaks a message into lines of at most `width` cells.
A line always takes at least one character, so a character wider than the window gets a line of its own rather than an empty line before it.

```lua defaults/lua/gband/errors.lua
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
```

Messages are separated by an empty line.

```lua defaults/lua/gband/errors.lua
local function is_open(win)
  for _, id in ipairs(gband.win.list()) do
    if id == win then
      return true
    end
  end
  return false
end
```

### Clearing

```lua defaults/lua/gband/errors.lua
local function clear()
  gband.clear_errors()
  if not (current and is_open(current)) then
    return
  end
  texts = {}
  local info = gband.win.info(current)
  if info.kind == "floating" then
    gband.win.set_config(current, { title = TITLE })
  end
  gband.win.set_lines(current, lines_of(info.cols or 1))
end
```

`gband.clear_errors` empties the client's error list, which also clears the sidebar's marker through `gband.core.on_state`.
An open list is emptied too, and drops the `c clear` hint from its title.

### Opening

```lua defaults/lua/gband/errors.lua
local function open(kind)
  gband.keymap.enter("root")
  if current and is_open(current) then
    gband.win.focus(current)
    return
  end
```

`open` leaves the key table you opened it from, and focuses the list if it is already open.

```lua defaults/lua/gband/errors.lua
  texts = gband.errors()
  local spec = {
    kind = kind,
    lines = { "" },
    keys = {
      q = function(win)
        gband.win.close(win)
      end,
      c = clear,
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
```

The list takes a copy of the messages as they are now, from `gband.errors`.
It is opened with a single empty line: the text is wrapped to the window's width, which a tiled window does not know until the server has opened it.
`on_resize` wraps the text again at each new width, the first time included.

```lua defaults/lua/gband/errors.lua
  if kind == "floating" then
    local state = gband.view()
    spec.title = #texts > 0 and HINTED or TITLE
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
```

A floating list is three quarters of the view's width and half its height, in the middle.
Its width is known at once, from `gband.win.info`, so its lines are set straight away.

```lua defaults/lua/gband/errors.lua
local function check_kind(kind, field)
  if kind ~= nil and not KINDS[kind] then
    error("`" .. field .. "` must be \"floating\" or \"tiled\", found " .. tostring(kind), 3)
  end
end
```

```lua defaults/lua/gband/errors.lua
return {
  name = "errors",
  api = 2,
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
    gband.action.register(CLEAR, clear, { desc = "clear the errors" })
    gband.cmd.register(CLEAR, clear, { desc = "clear the errors" })
  end,
}
```

The plugin declares `api = 2`, the API version it was written for.
Its one option, `kind`, chooses floating or tiled, and the command `errors.open` can override it per call.
Each function is registered twice, as an action for key bindings and as a command for the Lua prompt and `gband.cmd`.
