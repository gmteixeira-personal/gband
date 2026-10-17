# 07 · Settings

The settings window, Ctrl+Space then `s`, picks the theme, turns the sidebar on or off, chooses the key style, and reloads the configuration.
Each choice is saved to a small file under `user/`, and the default configuration reads those files as it loads.
`gband.settings`, in `defaults/lua/gband/settings.lua`, is both: the window, built with `gband.win`, and the functions that read and write the saved choices.
This chapter reads it in full.

## Constants

```lua defaults/lua/gband/settings.lua
local core = gband.core
```

```lua defaults/lua/gband/settings.lua
gband.hl.default("SettingsLabel", { dim = true })
```

```lua defaults/lua/gband/settings.lua
local NAME = "^[A-Za-z0-9][A-Za-z0-9_-]*$"
local LABEL_WIDTH = 9
local WIDTH = 31
local THEME, SIDEBAR, KEYS, INTERACTIVE_ON_NEW = 1, 2, 3, 4
local LABELS = { "theme", "sidebar", "keys", "I on new" }
local RELOAD_LABEL = "reload"
local KEY_STYLES = { "modal", "direct", "floating" }
```

`NAME` is the colorscheme name rule of [03](03-colorschemes.md).
The window has at most five lines, and each line starts with its label padded to `LABEL_WIDTH` columns.
The first four are numbered by the four constants; the last, `reload`, moves up a line when `I on new` is absent, so it has a label and no number of its own.
`KEY_STYLES` is the order in which the `keys` line cycles through the three key styles of [08](08-key-styles.md).

```lua defaults/lua/gband/settings.lua
local THEME_FILE = "user/theme.lua"
local SIDEBAR_FILE = "user/sidebar.lua"
local KEYSTYLE_FILE = "user/keystyle.lua"
local INTERACTIVE_ON_NEW_FILE = "user/interactive_on_new.lua"
local RESTORE_FILE = "user/floating_restore.lua"
```

The first four files hold one saved choice each.
`RESTORE_FILE` holds the layout recorded when the `keys` line enters the floating style, as "The floating record" below describes.

```lua defaults/lua/gband/settings.lua
local MOVES = {
  j = 1, down = 1, k = -1, up = -1,
  pagedown = "page", pageup = "page", home = "first", ["end"] = "last",
}
```

`MOVES` is what the theme list does with each navigation key: a step of one line, a page, or a jump to either end.
`end` is a Lua keyword, so it has to be written `["end"]`.

## Reading and writing a saved choice

```lua defaults/lua/gband/settings.lua
local function read(file, valid)
  local dir = gband.config_dir
  if type(dir) ~= "string" then
    return nil
  end
  local ok, value = pcall(function()
    local chunk = loadfile(dir .. "/" .. file, "t", {})
    if chunk then
      return chunk()
    end
  end)
  if ok and valid(value) then
    return value
  end
  return nil
end
```

A saved choice is a file such as `user/theme.lua` that holds `return "gruvbox"`.
`read` runs it with `loadfile` in text mode, with an empty table as its environment, so the file can return a value but cannot call `gband`, `require` or anything else.
`pcall` catches a file that raises an error, and `valid` checks the value's type.
A missing, broken or invalid file reads as nil, the same as no choice, so a bad hand edit costs only that choice.
`read` is one of the module's two exports; `gband.keystyle` reads `user/keystyle.lua` with it in [08](08-key-styles.md).

```lua defaults/lua/gband/settings.lua
local function write(file, text)
  local dir = gband.config_dir
  if type(dir) ~= "string" then
    error("cannot save " .. file .. ": there is no configuration directory", 0)
  end
  local base = file:match("([^/]+)%.lua$")
  local temporary = string.format("%s/user/.%s.%d.%d", dir, base, os.time(), math.random(1, 1 << 30))
```

```lua defaults/lua/gband/settings.lua
  local handle, reason = io.open(temporary, "w")
  local ok = handle ~= nil
  if handle then
    ok, reason = handle:write(text)
    local closed, close_reason = handle:close()
    if ok and not closed then
      ok, reason = false, close_reason
    end
    if ok then
      ok, reason = os.rename(temporary, dir .. "/" .. file)
    end
    if not ok then
      os.remove(temporary)
    end
  end
```

`write` writes the text to a temporary file in `user/`, then renames it over the real one.
A rename within one directory replaces the file in one step, so a reload that reads the file never sees half of it.
The temporary file's name starts with a full stop and does not end in `.lua`, so only the rename looks to the watcher like a saved Lua file.
If anything fails, the temporary file is removed.

```lua defaults/lua/gband/settings.lua
  if not ok then
    error("cannot save " .. file .. ": " .. tostring(reason), 0)
  end
end
```

The errors use level 0, so the message carries no file and line: it is about the configuration directory, not about any line of Lua.

```lua defaults/lua/gband/settings.lua
local function forget(file)
  local dir = gband.config_dir
  if type(dir) == "string" then
    os.remove(dir .. "/" .. file)
  end
end
```

`forget` removes a saved file, and ignores a file that is already gone.

```lua defaults/lua/gband/settings.lua
local function serialize(value)
  if type(value) == "table" then
    local parts = {}
    for index, item in ipairs(value) do
      parts[index] = serialize(item)
    end
    local keys = {}
    for key in pairs(value) do
      if type(key) == "string" then
        keys[#keys + 1] = key
      end
    end
    table.sort(keys)
    for _, key in ipairs(keys) do
      parts[#parts + 1] = key .. " = " .. serialize(value[key])
    end
    return "{ " .. table.concat(parts, ", ") .. " }"
  elseif math.type(value) == "float" then
    return string.format("%.17g", value)
  end
  return tostring(value)
end
```

`serialize` turns a table of numbers, booleans and nested tables into a Lua constructor that `read` can load back.
The list part comes first, then the named fields in sorted order, so one layout always gives the same text.
A float is written with 17 significant digits, the most a double needs, so a width of 1/3 reads back as the same number.
Integers and booleans are written by `tostring`, and field names are plain identifiers, so no string escaping is needed.

## The saved choices

```lua defaults/lua/gband/settings.lua
local function is_name(value)
  return type(value) == "string" and value:match(NAME) ~= nil
end
```

```lua defaults/lua/gband/settings.lua
local function theme()
  return read(THEME_FILE, is_name)
end
```

```lua defaults/lua/gband/settings.lua
local function is_boolean(value)
  return type(value) == "boolean"
end
```

```lua defaults/lua/gband/settings.lua
local function sidebar()
  return read(SIDEBAR_FILE, is_boolean)
end
```

```lua defaults/lua/gband/settings.lua
local function interactive_on_new()
  return read(INTERACTIVE_ON_NEW_FILE, is_boolean)
end
```

`theme` is what `start` in [03](03-colorschemes.md) reads.
`sidebar` is read by the default configuration, in [11](11-default-configs.md), and `interactive_on_new` by the modal key style, in [08](08-key-styles.md).
Each returns nil when nothing is saved, so a caller can tell "off" from "never chosen".

```lua defaults/lua/gband/settings.lua
local function themes()
  local names, seen = {}, {}
  for _, name in ipairs(core.bundled_themes) do
    names[#names + 1] = name
    seen[name] = true
  end
  for _, name in ipairs(core.colorschemes()) do
    if is_name(name) and not seen[name] then
      names[#names + 1] = name
      seen[name] = true
    end
  end
  return names
end
```

The theme list starts with the bundled themes in their own order, from `gband.core.bundled_themes`, then every other colorscheme on the runtimepath, from `gband.core.colorschemes`.
A file in `user/colors/` with a bundled theme's name replaces that theme but keeps its place, and a name that the colorscheme rules would refuse is never listed.

## The floating record

The floating style's desktop plugin floats every tiled window at each attach and reload, as [08](08-key-styles.md) describes.
The modal and direct styles load no desktop plugin, so nothing would tile those windows again when the style leaves `floating`.
So the `keys` line records the layout in `RESTORE_FILE` as it enters `floating`, and rebuilds it from that file as it leaves.
A reload starts a new Lua state, so the record lives in a file and not in a local.

```lua defaults/lua/gband/settings.lua
local function is_index(value)
  return math.type(value) == "integer" and value >= 1
end
```

```lua defaults/lua/gband/settings.lua
local function is_cell(value)
  return math.type(value) == "integer" and value >= 0
end
```

```lua defaults/lua/gband/settings.lua
local function is_size(value)
  return type(value) == "number" and value > 0 and value <= 10000
end
```

`is_index` accepts a window or band number and a height in rows, `is_cell` a screen column or row, and `is_size` a width or weight in the range `gband.window.set_width` and `gband.window.set_height` accept.

```lua defaults/lua/gband/settings.lua
local function is_list(value, valid)
  if type(value) ~= "table" then
    return false
  end
  for _, item in ipairs(value) do
    if not valid(item) then
      return false
    end
  end
  return true
end
```

```lua defaults/lua/gband/settings.lua
local function is_tile(value)
  if type(value) ~= "table" or not is_index(value.id) then
    return false
  end
  if value.rows ~= nil then
    return value.weight == nil and is_index(value.rows)
  end
  return is_size(value.weight)
end
```

A tiled window holds `id` and exactly one of `rows` and `weight`, as `gband.window.set_height` takes them.

```lua defaults/lua/gband/settings.lua
local function is_column(value)
  return type(value) == "table"
    and is_size(value.width)
    and is_boolean(value.full_width)
    and is_boolean(value.tabbed)
    and is_list(value.windows, is_tile)
end
```

```lua defaults/lua/gband/settings.lua
local function is_box(value)
  return type(value) == "table"
    and is_index(value.id)
    and is_cell(value.col)
    and is_cell(value.row)
    and is_size(value.width)
    and is_boolean(value.full_width)
    and is_index(value.rows)
end
```

```lua defaults/lua/gband/settings.lua
local function is_band(value)
  return type(value) == "table"
    and is_index(value.id)
    and is_list(value.columns, is_column)
    and is_list(value.floating, is_box)
end
```

```lua defaults/lua/gband/settings.lua
local function is_record(value)
  return type(value) == "table" and is_list(value.bands, is_band)
end
```

`is_record` is the `valid` that `read` takes for `RESTORE_FILE`.
It accepts only the shape `record_of` writes, so a missing or hand-broken file reads as no record, and every value the restore passes to an action is in range.

```lua defaults/lua/gband/settings.lua
local function record_of(layout)
  local bands = {}
  for _, band in ipairs(layout.bands) do
    local columns, floating = {}, {}
    for _, column in ipairs(band.columns) do
      local windows = {}
      for _, window in ipairs(column.windows) do
        if window.name then
          windows[#windows + 1] = { id = window.id, rows = window.rows, weight = window.weight }
        end
      end
      if #windows > 0 then
        columns[#columns + 1] = {
          width = column.width,
          full_width = column.full_width,
          tabbed = column.tabbed,
          windows = windows,
        }
      end
    end
```

```lua defaults/lua/gband/settings.lua
    for _, box in ipairs(band.floating) do
      if box.name then
        floating[#floating + 1] = {
          id = box.id,
          col = box.col,
          row = box.row,
          width = box.width,
          full_width = box.full_width,
          rows = box.rows,
        }
      end
    end
    bands[#bands + 1] = { id = band.id, columns = columns, floating = floating }
  end
  return { bands = bands }
end
```

`record_of` copies from `gband.layout()` only the fields the restore sets again.
It keeps only windows that run a program, those with a `name`, as the desktop plugin's sweep floats only those.
A plugin window, such as the settings window itself, gets a new number when it reopens after the reload, so a record of it would name a window that no longer exists.

```lua defaults/lua/gband/settings.lua
local function floating_programs(layout)
  local ids, floats = {}, {}
  for _, band in ipairs(layout.bands) do
    for _, box in ipairs(band.floating) do
      if box.name then
        ids[#ids + 1] = box.id
        floats[box.id] = true
      end
    end
  end
  return ids, floats
end
```

`floating_programs` gives the floating program windows twice: as a list in layout order and as a set by number.
They are separate tables because window numbers are integers and would collide with the list's indexes.

```lua defaults/lua/gband/settings.lua
local function tile_all(ids)
  for index = #ids, 1, -1 do
    gband.action.toggle_window_floating({ window = ids[index], floating = false })
  end
end
```

The client checks a `toggle_window_floating` target's `after` against the layout it holds when the call runs, and every window still floats then, so `after` cannot name a window tiled earlier in the same key press.
Without `after`, a tiled window lands as its band's first column, so `tile_all` tiles the list from its end, and each window lands left of the one tiled before it.
The columns then read left to right in list order.

```lua defaults/lua/gband/settings.lua
local function restore_column(column)
  local first = column.windows[1].id
  for index = 2, #column.windows do
    gband.action.consume_or_expel_left({ window = column.windows[index].id })
  end
  gband.window.set_width(first, column.width)
  if column.full_width then
    gband.action.toggle_full_width({ window = first })
  end
  for _, window in ipairs(column.windows) do
    gband.window.set_height(window.id, window.rows and { rows = window.rows } or { weight = window.weight })
  end
  if column.tabbed then
    gband.action.toggle_column_tabbed({ window = first })
  end
end
```

After `tile_all`, each window of a recorded column is a column of its own, in recorded order.
`consume_or_expel_left` moves each further window, top to bottom, to the bottom of the column on its left, which rebuilds the stack.
`set_width` also turns full width off, so `toggle_full_width` runs only when the record has it on.
The heights come before `toggle_column_tabbed`, because a height set on a tabbed column is not applied.

```lua defaults/lua/gband/settings.lua
local function restore_box(box)
  gband.window.set_width(box.id, box.width)
  if box.full_width then
    gband.action.toggle_full_width({ window = box.id })
  end
  gband.window.set_height(box.id, { rows = box.rows })
  gband.window.set_position(box.id, { col = box.col, row = box.row })
end
```

A window that floated before `floating` was entered still floats, so `restore_box` only puts its box back, in the order the desktop plugin's restore uses in [08](08-key-styles.md).

```lua defaults/lua/gband/settings.lua
local function restore(record)
  local ids, floats = floating_programs(gband.layout())
  local recorded, columns, boxes = {}, {}, {}
  for _, band in ipairs(record.bands) do
    for _, column in ipairs(band.columns) do
      local windows = {}
      for _, window in ipairs(column.windows) do
        recorded[window.id] = true
        if floats[window.id] then
          windows[#windows + 1] = window
        end
      end
      if #windows > 0 then
        column.windows = windows
        columns[#columns + 1] = column
      end
    end
    for _, box in ipairs(band.floating) do
      recorded[box.id] = true
      if floats[box.id] then
        boxes[#boxes + 1] = box
      end
    end
  end
```

`restore` keeps from the record only the windows that still float now.
A window closed while `floating` was in use is skipped, and a column whose windows all closed does not come back.

```lua defaults/lua/gband/settings.lua
  local unrecorded, tiles = {}, {}
  for _, id in ipairs(ids) do
    if not recorded[id] then
      unrecorded[#unrecorded + 1] = id
    end
  end
  for _, column in ipairs(columns) do
    for _, window in ipairs(column.windows) do
      tiles[#tiles + 1] = window.id
    end
  end
  tile_all(unrecorded)
  tile_all(tiles)
  for _, column in ipairs(columns) do
    restore_column(column)
  end
  for _, box in ipairs(boxes) do
    restore_box(box)
  end
end
```

A window opened while `floating` was in use is in no record, and `tile_all` tiles those first.
The recorded windows tiled after them land to their left, so the new windows follow the restored ones.
Every band's windows go in one list, since each band's first column is its own.

## Saving and reopening

```lua defaults/lua/gband/settings.lua
local function save(line, file, text)
  core.reopen_settings(line)
  local ok, reason = pcall(write, file, text)
  if not ok then
    core.reopen_settings(nil)
    error(reason, 0)
  end
end
```

Saving a file under `user/` reloads the configuration, and a reload starts a new Lua state, where the settings window no longer exists.
So before it writes, `save` calls `gband.core.reopen_settings` with the window's line.
After the reload, the client calls the new state's settings provider with that line, and the window reopens where it was.
If the write fails, there will be no reload, and the request is withdrawn.

```lua defaults/lua/gband/settings.lua
local function save_theme(name)
  save(THEME, THEME_FILE, 'return "' .. name .. '"\n')
end
```

## The window

```lua defaults/lua/gband/settings.lua
local window = nil
local list = nil
```

`window` is the settings window's id and `list` the theme list's, each nil when closed.

```lua defaults/lua/gband/settings.lua
local function is_open(win)
  if win == nil then
    return false
  end
  for _, id in ipairs(gband.win.list()) do
    if id == win then
      return true
    end
  end
  return false
end
```

```lua defaults/lua/gband/settings.lua
local function key_style()
  return gband.keystyle.saved() or "modal"
end
```

`is_open` asks `gband.win` whether the id is still open, rather than trusting that it is set.

```lua defaults/lua/gband/settings.lua
local function reload_line()
  return key_style() == "modal" and INTERACTIVE_ON_NEW + 1 or KEYS + 1
end
```

```lua defaults/lua/gband/settings.lua
local function meaning(line)
  if line == reload_line() then
    return RELOAD_LABEL
  end
  return line
end
```

`reload_line` is the line `reload` sits on: fifth with the modal style, fourth otherwise.
`meaning` turns a cursor line into what the keys compare with: the label for the `reload` line, and the line's own number for the others.
Without it, the fourth line would read as `INTERACTIVE_ON_NEW` with the direct style, and `l` on `reload` would save `I on new`.

```lua defaults/lua/gband/settings.lua
local function error_count()
  local count = #gband.errors()
  if count == 0 then
    return "ok"
  elseif count == 1 then
    return "1 error"
  end
  return count .. " errors"
end
```

```lua defaults/lua/gband/settings.lua
local function labelled(label, value)
  return {
    { text = label .. string.rep(" ", LABEL_WIDTH - #label), hl = "SettingsLabel" },
    value,
  }
end
```

```lua defaults/lua/gband/settings.lua
local function lines()
  local values = {
    tostring(gband.colorscheme()),
    sidebar() == false and "off" or "on",
    key_style(),
  }
  if values[KEYS] == "modal" then
    values[INTERACTIVE_ON_NEW] = interactive_on_new() == true and "on" or "off"
  end
  local out = {}
  for index, value in ipairs(values) do
    out[index] = labelled(LABELS[index], value)
  end
  out[#out + 1] = labelled(RELOAD_LABEL, error_count())
  return out
end
```

Each line is two spans: the label in `SettingsLabel`, padded, and the value in the window's own style.
The `I on new` line exists only for the modal style, since only the modal preset reads it.
It shows `on` only for a saved `true`, so a setting never saved shows `off`, as the preset reads it.
The `reload` line comes last and counts `gband.errors()` each time the lines are built, so the window shows the errors of the moment it draws.

```lua defaults/lua/gband/settings.lua
local function refresh()
  if is_open(window) then
    gband.win.set_lines(window, lines())
  end
end
```

## Changing a value

```lua defaults/lua/gband/settings.lua
local function index_of(names, name)
  for index, candidate in ipairs(names) do
    if candidate == name then
      return index
    end
  end
  return nil
end
```

```lua defaults/lua/gband/settings.lua
local function step_theme(direction)
  local names = themes()
  local previous = gband.colorscheme()
  local at = index_of(names, previous)
  local target
  if at == nil then
    target = direction > 0 and 1 or #names
  else
    target = (at - 1 + direction) % #names + 1
  end
  local name = names[target]
  if not gband.colorscheme(name) then
    return
  end
  local ok, reason = pcall(save_theme, name)
  if not ok then
    gband.colorscheme(previous)
    error(reason, 0)
  end
  refresh()
end
```

`h` and `l` on the theme line step through the theme list, wrapping at either end.
The theme is loaded first and saved only once it loaded, so a broken theme is never saved; and if saving fails, the previous theme comes back.
Saving reloads the configuration, after which `start` loads the saved theme, so the change survives the reload.

```lua defaults/lua/gband/settings.lua
local function toggle_sidebar()
  local next_value = sidebar() == false and "true" or "false"
  save(SIDEBAR, SIDEBAR_FILE, "return " .. next_value .. "\n")
end
```

```lua defaults/lua/gband/settings.lua
local function step_keys(direction)
  local style = key_style()
  local at = index_of(KEY_STYLES, style) or 1
  local next_style = KEY_STYLES[(at - 1 + direction) % #KEY_STYLES + 1]
```

The `keys` line steps through `KEY_STYLES` as the theme line steps through the themes, wrapping at both ends, with `index_of` from above.

```lua defaults/lua/gband/settings.lua
  if style ~= "floating" and next_style == "floating" then
    write(RESTORE_FILE, "return " .. serialize(record_of(gband.layout())) .. "\n")
  elseif style == "floating" and next_style ~= "floating" then
    local record = read(RESTORE_FILE, is_record)
    if record then
      restore(record)
    else
      tile_all((floating_programs(gband.layout())))
    end
    forget(RESTORE_FILE)
  end
  save(KEYS, KEYSTYLE_FILE, 'return "' .. next_style .. '"\n')
end
```

Entering `floating` writes the record before `save`, while the layout still holds the tiled arrangement: the reload that `save` causes runs the desktop plugin's sweep.
Entering `floating` again replaces the record.
Leaving `floating` restores the record, or, with none, tiles every floating program window in layout order.
No record exists when `floating` was already in use as the settings window opened, since only this line writes one.
The extra parentheses keep only the first of `floating_programs`'s two results.
The record is removed once restored, so a later trip through `floating` starts fresh.
The actions dispatch when the key's callback returns, before the reload that `save` causes re-reads the configuration.

```lua defaults/lua/gband/settings.lua
local function toggle_interactive_on_new()
  local next_value = interactive_on_new() == true and "false" or "true"
  save(INTERACTIVE_ON_NEW, INTERACTIVE_ON_NEW_FILE, "return " .. next_value .. "\n")
end
```

The lines after `theme` only save.
The `I on new` toggle compares with `true` as the line does, so the first toggle of a setting never saved saves `true`.
None of them changes anything directly: the reload that the save causes runs the default configuration again, which reads the new value.

```lua defaults/lua/gband/settings.lua
local function reload()
  core.reopen_settings(reload_line())
  gband.action.reload()
end
```

`reload` saves nothing, so no file change will reload anything: it asks for the reload itself, with the `reload` action of [08](08-key-styles.md)'s `prefix !`.
It still records its line, as `save` does.
The client holds that line until both its own load and the server's have ended, then reopens the window on it whether they worked or not, so the line counts the errors of both.

```lua defaults/lua/gband/settings.lua
local function change(direction)
  return function(win)
    local line = meaning(gband.win.info(win).cursor)
    if line == THEME then
      step_theme(direction)
    elseif line == SIDEBAR then
      toggle_sidebar()
    elseif line == KEYS then
      step_keys(direction)
    elseif line == INTERACTIVE_ON_NEW then
      toggle_interactive_on_new()
    end
  end
end
```

```lua defaults/lua/gband/settings.lua
local open_list
```

`change` has no branch for `reload`, so `h`, `l` and the arrows do nothing there.

```lua defaults/lua/gband/settings.lua
local function enter(win)
  local line = meaning(gband.win.info(win).cursor)
  if line == RELOAD_LABEL then
    reload()
  elseif line == THEME then
    open_list()
  elseif line == SIDEBAR then
    toggle_sidebar()
  elseif line == KEYS then
    step_keys(1)
  elseif line == INTERACTIVE_ON_NEW then
    toggle_interactive_on_new()
  end
end
```

Enter on the theme line opens the theme list, on the `keys` line steps forward as `l` does, and on the `reload` line reloads.
`open_list` is defined further down, after the functions it uses, so it is declared here first: `enter` captures the local, and by the time a key runs `enter` the local holds the function.

## The theme list

```lua defaults/lua/gband/settings.lua
local listed = nil
```

`listed` holds the names the list shows and `original`, the theme that was active when it opened.

```lua defaults/lua/gband/settings.lua
local function close_list()
  local win = list
  list = nil
  if is_open(win) then
    gband.win.close(win)
  end
  if is_open(window) then
    gband.win.focus(window)
  end
end
```

```lua defaults/lua/gband/settings.lua
local function cancel_list()
  if listed and listed.original ~= gband.colorscheme() then
    gband.colorscheme(listed.original)
  end
  close_list()
end
```

Escape and `q` cancel: the original theme comes back before the list closes.
Focus returns to the settings window.

```lua defaults/lua/gband/settings.lua
local function preview(win, line)
  local info = gband.win.info(win)
  local target = math.max(1, math.min(line, info.line_count))
  if target == info.cursor then
    return
  end
  gband.win.set_cursor(win, target)
  gband.colorscheme(listed.names[target])
end
```

```lua defaults/lua/gband/settings.lua
local function move_list(name)
  return function(win)
    local info = gband.win.info(win)
    local move = MOVES[name]
    local target
    if move == "page" then
      target = info.cursor + (name == "pageup" and -info.rows or info.rows)
    elseif move == "first" then
      target = 1
    elseif move == "last" then
      target = info.line_count
    else
      target = info.cursor + move
    end
    preview(win, target)
  end
end
```

The list previews: moving the cursor loads the theme under it, so you see each theme as you pass it.
`preview` does nothing when the cursor does not move, which keeps a click on the current line from loading the theme again.

```lua defaults/lua/gband/settings.lua
local function pick(win)
  local name = listed.names[gband.win.info(win).cursor]
  local chosen = name == gband.colorscheme()
  close_list()
  if not chosen then
    return
  end
  local ok, reason = pcall(save_theme, name)
  if not ok then
    gband.colorscheme(listed.original)
    refresh()
    error(reason, 0)
  end
  refresh()
end
```

Enter saves the theme under the cursor, but only if it loaded: a theme that failed to preview is still the original.

```lua defaults/lua/gband/settings.lua
function open_list()
  local names = themes()
  local longest = 0
  for _, name in ipairs(names) do
    longest = math.max(longest, gband.ui.width(name))
  end
  listed = { names = names, original = gband.colorscheme() }
  local keys = { enter = pick, escape = cancel_list, q = cancel_list }
  for name in pairs(MOVES) do
    keys[name] = move_list(name)
  end
  list = gband.win.open({
    title = "theme",
    width = longest + 2,
    height = #names + 2,
    lines = names,
    cursorline = true,
    keys = keys,
    on_mouse = function(win, event)
      if event.kind == "press" and event.button == "left" and event.line then
        preview(win, event.line)
      elseif event.kind == "scroll" and (event.direction == "up" or event.direction == "down") then
        move_list(event.direction)(win)
      end
    end,
    on_close = function(win)
      if list == win then
        list = nil
      end
    end,
  })
  gband.win.set_cursor(list, index_of(names, listed.original) or 1)
end
```

The list is a floating window just wide enough for the longest name, `gband.ui.width` counting cells, plus its border.
Its keys are built from `MOVES`, and its own `on_mouse` turns a click into a preview and the wheel into a step.
`on_close` clears `list` only when it is still this window, so closing an old list does not forget a new one.

## Opening the window

```lua defaults/lua/gband/settings.lua
local function show(line)
  gband.keymap.enter("root")
  if is_open(list) then
    cancel_list()
  end
  if is_open(window) then
    gband.win.focus(window)
    gband.win.set_lines(window, lines())
  else
    local content = lines()
    window = gband.win.open({
      title = "settings",
      width = WIDTH,
      height = #content + 2,
      lines = content,
      cursorline = true,
      keys = {
        enter = enter,
        h = change(-1),
        left = change(-1),
        l = change(1),
        right = change(1),
      },
      on_close = function(win)
        if window == win then
          window = nil
        end
      end,
    })
  end
  if line then
    gband.win.set_cursor(window, line)
  end
end
```

`show` leaves any key table, such as the prefix table you opened it from, so keys reach the window.
A theme list left open is cancelled, and the settings window is focused and its lines drawn again when it is already open, or opened otherwise.
A load that fails closes no plugin window, so after a failed reload from the `reload` line the window is still open, and drawing its lines again shows the new count.
A line, given when the window reopens after a reload, puts the cursor back where it was.

```lua defaults/lua/gband/settings.lua
core.provide("settings", { open = show })
```

The settings provider is the client's way back in after a reload: it calls `open(line)` with the line passed to `gband.core.reopen_settings`.

```lua defaults/lua/gband/settings.lua
gband.settings = {
  open = function()
    if core.loading() then
      error("gband.settings.open cannot be called while the configuration loads", 2)
    end
    show(nil)
  end,
  theme = theme,
  sidebar = sidebar,
  interactive_on_new = interactive_on_new,
  themes = themes,
}
```

`gband.settings.open` refuses to run while the configuration loads, since `gband.win.open` can only run in a callback.

```lua defaults/lua/gband/settings.lua
return { read = read, write = write }
```

The module returns its two exports, `read` and `write`, for the other modules.
