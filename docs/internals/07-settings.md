# 07 · Settings

The settings window, Ctrl+Space then `s`, picks the theme, turns the sidebar on or off, and chooses the key style.
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
```

`NAME` is the colorscheme name rule of [03](03-colorschemes.md).
The window has four lines, numbered by the four constants, and each line starts with its label padded to `LABEL_WIDTH` columns.

```lua defaults/lua/gband/settings.lua
local THEME_FILE = "user/theme.lua"
local SIDEBAR_FILE = "user/sidebar.lua"
local KEYSTYLE_FILE = "user/keystyle.lua"
local INTERACTIVE_ON_NEW_FILE = "user/interactive_on_new.lua"
```

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
    local label = LABELS[index]
    out[index] = {
      { text = label .. string.rep(" ", LABEL_WIDTH - #label), hl = "SettingsLabel" },
      value,
    }
  end
  return out
end
```

Each line is two spans: the label in `SettingsLabel`, padded, and the value in the window's own style.
The `I on new` line exists only for the modal style, since only the modal preset reads it.
It shows `on` only for a saved `true`, so a setting never saved shows `off`, as the preset reads it.

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
local function toggle_keys()
  local next_style = key_style() == "modal" and "direct" or "modal"
  save(KEYS, KEYSTYLE_FILE, 'return "' .. next_style .. '"\n')
end
```

```lua defaults/lua/gband/settings.lua
local function toggle_interactive_on_new()
  local next_value = interactive_on_new() == true and "false" or "true"
  save(INTERACTIVE_ON_NEW, INTERACTIVE_ON_NEW_FILE, "return " .. next_value .. "\n")
end
```

The other three lines only save.
The `I on new` toggle compares with `true` as the line does, so the first toggle of a setting never saved saves `true`.
None of them changes anything directly: the reload that the save causes runs the default configuration again, which reads the new value.

```lua defaults/lua/gband/settings.lua
local function change(direction)
  return function(win)
    local line = gband.win.info(win).cursor
    if line == THEME then
      step_theme(direction)
    elseif line == SIDEBAR then
      toggle_sidebar()
    elseif line == KEYS then
      toggle_keys()
    elseif line == INTERACTIVE_ON_NEW then
      toggle_interactive_on_new()
    end
  end
end
```

```lua defaults/lua/gband/settings.lua
local open_list
```

```lua defaults/lua/gband/settings.lua
local function enter(win)
  local line = gband.win.info(win).cursor
  if line == THEME then
    open_list()
  elseif line == SIDEBAR then
    toggle_sidebar()
  elseif line == KEYS then
    toggle_keys()
  elseif line == INTERACTIVE_ON_NEW then
    toggle_interactive_on_new()
  end
end
```

Enter on the theme line opens the theme list.
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
A theme list left open is cancelled, and the settings window is focused when it is already open, or opened otherwise.
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
