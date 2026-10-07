# 10 · The key list and the prompt

Both key styles set up two more bundled plugins before making their bindings: the key list, Ctrl+Space then `?`, and the Lua prompt, Ctrl+Space then `:`.
This chapter reads them, `defaults/lua/gband/keylist.lua` and `defaults/lua/gband/prompt.lua`, and the small module the key list shows keys with, `defaults/lua/gband/keyform.lua`, all in full.

## `gband.keyform`

The key list shows keys short, as `C-space` rather than `ctrl+space`.
`gband.keyform` turns a key name into that form, and plugins can require it to show keys the same way.

```lua defaults/lua/gband/keyform.lua
local MOUSE = {
  leftmouse = true,
  middlemouse = true,
  rightmouse = true,
  wheelup = true,
  wheeldown = true,
  wheelleft = true,
  wheelright = true,
}
```

The mouse names that a binding may use, which the key list leaves out.

```lua defaults/lua/gband/keyform.lua
local function split(written)
  local modifiers, key
  if written == "+" then
    modifiers, key = "", "+"
  elseif written:sub(-2) == "++" then
    modifiers, key = written:sub(1, -3), "+"
  else
    modifiers, key = written:match("^(.*)%+([^+]*)$")
    if not modifiers then
      modifiers, key = "", written
    end
  end
  return modifiers, key
end
```

`split` separates the modifiers from the key.
The `+` key needs care, since `+` also joins the modifiers to the key: `+` alone and a name ending in `++`, such as `alt++`, both mean the key `+`.
Otherwise the key is everything after the last `+`.

```lua defaults/lua/gband/keyform.lua
local function is_mouse(written)
  local _, key = split(written)
  return MOUSE[key:lower()] == true
end
```

```lua defaults/lua/gband/keyform.lua
local function key_form(written)
  local modifiers, key = split(written)
  local held = {}
  for modifier in modifiers:gmatch("[^+]+") do
    held[modifier:lower()] = true
  end
  if utf8.len(key) == 1 then
    if held.shift and key:match("^%l$") then
      key = key:upper()
      held.shift = nil
    end
  else
    key = key:lower()
    if key == "escape" then
      key = "esc"
    end
  end
  return (held.ctrl and "C-" or "") .. (held.alt and "A-" or "") .. (held.shift and "S-" or "") .. key
end
```

A one-character key keeps its case, so `shift+d` shows as `D` and `ctrl+a` as `C-a`.
A named key is lowercased, and `escape` shortened to `esc`.
The modifiers come out in a fixed order, Ctrl, Alt, Shift, whatever order they were written in.
`key_form` does not check the name: the key list only ever passes it names that `gband.keymap` accepted.

```lua defaults/lua/gband/keyform.lua
return setmetatable({ is_mouse = is_mouse }, {
  __call = function(_, written)
    return key_form(written)
  end,
})
```

The module's value is a table that can be called, through the `__call` metamethod.
`require("gband.keyform")("ctrl+space")` returns `C-space`, and the table also carries `is_mouse`.

## The key list

```lua defaults/lua/gband/keylist.lua
local key_form = require("gband.keyform")
```

```lua defaults/lua/gband/keylist.lua
gband.hl.default("KeyListKey", { bold = true })
gband.hl.default("KeyListMuted", { dim = true })
```

```lua defaults/lua/gband/keylist.lua
local OWN = "keylist.open"
local MAX_HEIGHT = 15
local OWN_KEYS = {
  up = true, down = true, pageup = true, pagedown = true,
  home = true, ["end"] = true, enter = true, esc = true,
}
```

`OWN` is the plugin's own action.
`OWN_KEYS` are the keys the list keeps for itself, written in key form: moving the cursor line, Enter and Escape.

```lua defaults/lua/gband/keylist.lua
local current = nil
```

### The entries

```lua defaults/lua/gband/keylist.lua
local function present(text)
  return text ~= nil and text ~= ""
end
```

```lua defaults/lua/gband/keylist.lua
local function describe(binding, descs)
  if present(binding.desc) then
    return binding.desc
  end
  local action = binding.action
  if action == nil then
    return "function"
  end
  return present(descs[action]) and descs[action] or action
end
```

A line's text is the binding's description, else its action's description, else the action's name, and `function` for a binding to a function with no description.

```lua defaults/lua/gband/keylist.lua
local function entries()
  local descs = {}
  for _, action in ipairs(gband.action.list()) do
    descs[action.name] = action.desc
  end
  local prefix = key_form(gband.opt.prefix)
  local list = {}
```

The list reads everything when it opens: the actions' descriptions, the prefix key and the bindings of `prefix`.

```lua defaults/lua/gband/keylist.lua
  for _, binding in ipairs(gband.keymap.list("prefix")) do
    if not key_form.is_mouse(binding.key) then
      local form = binding.key == "prefix" and prefix or key_form(binding.key)
      list[#list + 1] = {
        key = form,
        text = describe(binding, descs),
        binding = binding.action ~= OWN and binding.key or nil,
        direct = binding.key ~= "prefix" and not OWN_KEYS[form] and binding.key or nil,
      }
    end
  end
  return list
end
```

Each entry holds three things besides its text.
`key` is the key as shown; the binding of the key `prefix`, which stands for the prefix key itself, shows the key that `gband.opt.prefix` names.
`binding` is the key to run, nil on the list's own line, so the list cannot open itself.
`direct` is the key that runs the line from inside the list.
Neither the prefix key nor one of the list's own keys can, since the prefix key enters `prefix` before the list sees it and the own keys keep their meaning.

```lua defaults/lua/gband/keylist.lua
local function lines_of(list)
  local widest = 0
  for _, entry in ipairs(list) do
    widest = math.max(widest, gband.ui.width(entry.key))
  end
  local lines, longest = {}, 0
  for index, entry in ipairs(list) do
    local key = entry.key .. string.rep(" ", widest + 2 - gband.ui.width(entry.key))
    lines[index] = {
      { text = key, hl = "KeyListKey" },
      { text = entry.text, hl = entry.binding and "PluginWindow" or "KeyListMuted" },
    }
    longest = math.max(longest, gband.ui.width(key) + gband.ui.width(entry.text))
  end
  return lines, longest
end
```

Keys are padded to two cells more than the widest, so the descriptions line up.
A line that cannot run, the list's own, draws its text in `KeyListMuted`.
`longest` is the widest line, which sizes the window.

```lua defaults/lua/gband/keylist.lua
local function is_open(win)
  for _, id in ipairs(gband.win.list()) do
    if id == win then
      return true
    end
  end
  return false
end
```

```lua defaults/lua/gband/keylist.lua
local function other_float_focused(win)
  for _, id in ipairs(gband.win.list()) do
    if id ~= win then
      local info = gband.win.info(id)
      if info.focused and info.kind == "floating" then
        return true
      end
    end
  end
  return false
end
```

`other_float_focused` asks whether some other floating plugin window has focus.

### Opening

```lua defaults/lua/gband/keylist.lua
local function open()
  gband.keymap.enter("root")
  if current and is_open(current) then
    gband.win.focus(current)
    return
  end
  local list = entries()
  local lines, longest = lines_of(list)
```

```lua defaults/lua/gband/keylist.lua
  local function run(win, index)
    local entry = list[index]
    if not (entry and entry.binding) then
      return
    end
    gband.keymap.run("prefix", entry.binding)
    if is_open(win) and other_float_focused(win) then
      gband.win.close(win)
    end
  end
```

`run` runs a line with `gband.keymap.run("prefix", key)`, which does what pressing the key in `prefix` would.
An action acts on the focused window behind the list, so you can run several in a row and the list stays open.
But when the binding opened another floating plugin window, such as the settings window, that window now has focus, and the list closes itself rather than stay behind it.

```lua defaults/lua/gband/keylist.lua
  local keys = {
    enter = function(win)
      run(win, gband.win.info(win).cursor)
    end,
  }
  for index, entry in ipairs(list) do
    if entry.direct then
      keys[entry.direct] = function(win)
        gband.win.set_cursor(win, index)
        run(win, index)
      end
    end
  end
```

Enter runs the cursor line.
Each line's `direct` key runs it too, after moving the cursor there.
A binding in `keys` wins over `gband.win`'s defaults, so with the default bindings `j` and `k` run their lines, focusing the window below and above, and only the arrows move the cursor line.

```lua defaults/lua/gband/keylist.lua
  current = gband.win.open({
    title = gband.keymap.label("prefix") .. " keys",
    width = longest + 2,
    height = math.min(math.max(#lines + 2, 3), MAX_HEIGHT),
    cursorline = true,
    lines = lines,
    keys = keys,
    on_close = function(win)
      if current == win then
        current = nil
      end
    end,
  })
end
```

The title is the label of `prefix`: `navigation keys` with the modal style and `prefix keys` with the direct one, where `prefix` is not a mode.
The window is tall enough for its lines, at least one, and at most 15 rows, borders included.

```lua defaults/lua/gband/keylist.lua
return {
  name = "keylist",
  api = 1,
  setup = function(opts)
    if type(opts) ~= "table" then
      error("the options of `keylist` must be a table", 2)
    end
    local field = next(opts)
    if field ~= nil then
      error("`keylist` takes no option `" .. tostring(field) .. "`", 2)
    end
    gband.action.register(OWN, open, { desc = "list the keys" })
  end,
}
```

`next(opts)` returns the first key of a table, or nil for an empty one, so it checks that `opts` is empty without a loop.

## The Lua prompt

The prompt is one plugin with two uses: running a line of Lua, and renaming the focused window.

```lua defaults/lua/gband/prompt.lua
local core = gband.core
```

```lua defaults/lua/gband/prompt.lua
gband.hl.default("PromptCursor", { reverse = true })
```

```lua defaults/lua/gband/prompt.lua
local HEIGHT = 3
```

```lua defaults/lua/gband/prompt.lua
local function is_open(win)
  for _, id in ipairs(gband.win.list()) do
    if id == win then
      return true
    end
  end
  return false
end
```

### The window to rename

```lua defaults/lua/gband/prompt.lua
local function program_window(id)
  if not id then
    return nil
  end
  for _, band in ipairs(gband.layout().bands) do
    for _, column in ipairs(band.columns) do
      for _, window in ipairs(column.windows) do
        if window.id == id then
          return not window.plugin_window and window or nil
        end
      end
    end
    for _, window in ipairs(band.floating) do
      if window.id == id then
        return not window.plugin_window and window or nil
      end
    end
  end
  return nil
end
```

Renaming needs a window that runs a program.
`program_window` looks the id up in every band's columns and floating windows from `gband.layout`, and refuses a plugin window's drawn window, which has no program to name.

### Running Lua

```lua defaults/lua/gband/prompt.lua
local function run(text)
  local chunk, message = load(text, "@prompt", "t")
  if not chunk then
    error(message, 0)
  end
  chunk()
end
```

`load` compiles the line as text only, never a binary chunk, under the chunk name `@prompt`, so its errors read `prompt:1:`.
A syntax error is raised with level 0, since the message already names `prompt:1:`.

```lua defaults/lua/gband/prompt.lua
local kinds = {
  lua = {
    title = "lua",
    marker = ":",
    submit = function(text)
      if text ~= "" then
        core.call(nil, nil, run, text)
      end
    end,
  },
  rename = {
    title = "rename",
    marker = "",
    submit = function(text, target)
      if program_window(target) then
        gband.window.rename(target, text)
      end
    end,
  },
}
```

Each kind of prompt holds its title, the marker shown before the text, and what Enter does with the text.
`lua` runs the line with `gband.core.call` as code of no plugin, as `user/init.lua` is, with an instruction limit of its own.
The prompt is the plugin `prompt`, but a line that loops forever or fails is reported as a configuration error and does not mark the plugin failed.
`rename` renames the window it was opened for, if it is still in the layout.

```lua defaults/lua/gband/prompt.lua
for _, kind in pairs(kinds) do
  kind.line = ""
  kind.width = 1
end
```

Each kind keeps its line, and the width it last fitted the line to.

### The line

```lua defaults/lua/gband/prompt.lua
local function fitted(kind)
  local shown = kind.marker .. kind.line
  local used = gband.ui.width(shown)
  local start = 1
  while used + 1 > kind.width and start <= #shown do
    local next_start = utf8.offset(shown, 2, start)
    used = used - gband.ui.width(shown:sub(start, next_start - 1))
    start = next_start
  end
  return shown:sub(start)
end
```

The prompt is one row high.
When the marker, the text and the cursor do not fit, `fitted` drops characters from the start until they do, so the end of the line, where you type, stays in view.
`utf8.offset(shown, 2, start)` is the byte where the character after the one at `start` begins.

```lua defaults/lua/gband/prompt.lua
local function show(kind)
  gband.win.set_lines(kind.win, { { fitted(kind), { text = " ", hl = "PromptCursor" } } })
end
```

The line is two spans: the text, and one space in `PromptCursor` as the cursor.

```lua defaults/lua/gband/prompt.lua
local function flatten(text)
  return (text:gsub("\r\n", " "):gsub("[%z\1-\31\127]", " "):gsub("\194[\128-\159]", " "))
end
```

A paste may hold line breaks and other control characters; each becomes a space, so the paste stays on one line.

### Opening

```lua defaults/lua/gband/prompt.lua
local function open(kind, line, target)
  gband.keymap.enter("root")
  if kind.win and is_open(kind.win) then
    gband.win.focus(kind.win)
    return
  end
  for _, other in pairs(kinds) do
    if other ~= kind and other.win and is_open(other.win) then
      gband.win.close(other.win)
    end
  end
  kind.line = line
  kind.target = target
```

Only one prompt is open at a time: opening one closes the other, dropping its line.

```lua defaults/lua/gband/prompt.lua
  local view = gband.view()
  kind.win = gband.win.open({
    title = kind.title,
    width = math.max(1, view.cols),
    height = HEIGHT,
    row = view.rows,
    col = 0,
```

The prompt is as wide as the view and sits on its last three rows, where `row` pulls a box that would leave the screen back onto it, as [05](05-plugin-windows.md) shows.

```lua defaults/lua/gband/prompt.lua
    keys = {
      enter = function(win)
        local text, target = kind.line, kind.target
        gband.win.close(win)
        kind.submit(text, target)
      end,
      backspace = function(win)
        if kind.line == "" then
          gband.win.close(win)
          return
        end
        kind.line = kind.line:sub(1, utf8.offset(kind.line, -1) - 1)
        show(kind)
      end,
      ["ctrl+u"] = function()
        kind.line = ""
        show(kind)
      end,
    },
```

Enter reads the text and the target, closes the prompt, and only then submits.
A line that fails therefore leaves no prompt behind, and a floating plugin window that the line opens keeps focus.
Backspace on an empty line closes the prompt.

```lua defaults/lua/gband/prompt.lua
    on_input = function(_, text)
      kind.line = kind.line .. flatten(text)
      show(kind)
    end,
    on_resize = function(_, cols)
      kind.width = math.max(1, cols)
      show(kind)
    end,
    on_close = function(win)
      if kind.win == win then
        kind.win = nil
        kind.line = ""
        kind.target = nil
      end
    end,
  })
```

Typed characters and pastes both arrive through `on_input`, since the prompt binds no character keys: every key with text that `keys` does not bind goes there, `j`, `k` and `q` included.
`on_resize` fits the line to a new width.
`on_close` forgets the line, whatever closed the window.

```lua defaults/lua/gband/prompt.lua
  kind.width = math.max(1, gband.win.info(kind.win).cols)
  show(kind)
end
```

A floating window knows its size as soon as it opens, so the line is shown at once.

```lua defaults/lua/gband/prompt.lua
local function open_lua()
  open(kinds.lua, "")
end
```

```lua defaults/lua/gband/prompt.lua
local function open_rename()
  local kind = kinds.rename
  if kind.win and is_open(kind.win) then
    open(kind)
    return
  end
  local window = program_window(gband.view().window)
  if window then
    open(kind, window.manual_name or "", window.id)
  end
end
```

`prompt.rename` focuses a rename prompt that is already open.
Otherwise it opens one only when the focused window, from `gband.view`, runs a program, with the window's manual name as the starting text.

```lua defaults/lua/gband/prompt.lua
return {
  name = "prompt",
  api = 1,
  setup = function(opts)
    if type(opts) ~= "table" then
      error("the options of `prompt` must be a table", 2)
    end
    for field in pairs(opts) do
      error("`prompt` takes no option `" .. tostring(field) .. "`", 2)
    end
    gband.action.register("prompt.open", open_lua, { desc = "run Lua" })
    gband.action.register("prompt.rename", open_rename, { desc = "rename the window" })
  end,
}
```

The loop over `opts` raises an error at its first field, if it has one.
