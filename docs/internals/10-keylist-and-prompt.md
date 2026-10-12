# 10 · The key list and the prompt

Every key style sets up two more bundled plugins before making its bindings: the key list, Ctrl+Space then `?`, and the Lua prompt, Ctrl+Space then `:`.
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

The title is the label of `prefix`: `navigation keys` with the modal style and `prefix keys` with the direct and floating ones, where `prefix` is not a mode.
The window is tall enough for its lines, at least one, and at most 15 rows, borders included.

```lua defaults/lua/gband/keylist.lua
return {
  name = "keylist",
  api = 2,
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
  kind.cursor = 0
  kind.first = 0
  kind.width = 1
end
```

Each kind keeps its line, its cursor, its scroll position and the width it last fitted the line to.
`cursor` counts the characters before the cursor, from 0 to `utf8.len(line)`, so a move never has to find a character boundary; `utf8.offset` turns it into a byte at the moment of an edit.
`first` counts the leading characters of the marker and the line that the last drawing left out.

### The line

```lua defaults/lua/gband/prompt.lua
local function characters(text)
  local list = {}
  for character in text:gmatch(utf8.charpattern) do
    list[#list + 1] = character
  end
  return list
end
```

Fitting and clicking walk the shown text one character at a time, so `characters` splits it into a list.

```lua defaults/lua/gband/prompt.lua
local function fewest_dropped(shown, last, used, width)
  for i = 1, last do
    used = used + gband.ui.width(shown[i])
  end
  local dropped = 0
  while used > width and dropped < last do
    dropped = dropped + 1
    used = used - gband.ui.width(shown[dropped])
  end
  return dropped
end
```

`fewest_dropped` answers one question: how many leading characters of `shown[1..last]` must go so the rest, plus `used` more cells, fits `width`.

```lua defaults/lua/gband/prompt.lua
local function fitted(kind)
  local shown = characters(kind.marker .. kind.line)
  local at = utf8.len(kind.marker) + kind.cursor + 1
  local cell = shown[at] or " "
  local low = fewest_dropped(shown, at - 1, gband.ui.width(cell), kind.width)
  local fill = fewest_dropped(shown, #shown, 1, kind.width)
  kind.first = math.max(low, math.min(kind.first, kind.cursor, fill))
```

The prompt is one row high.
`at` is the index of the cursor cell: the character after the cursor, or a space past the end.
`low` is the fewest characters to drop so the text through the cursor cell fits, and `fill` the fewest so the whole line and one more cell fit.

The scroll position starts from the one the last drawing used, so a long line does not jump as the cursor moves inside it.
Lowering it to `cursor` keeps one character left of the cursor cell as the Lua prompt scrolls left, and shows the `:` again once the cursor reaches the start.
Lowering it to `fill` brings dropped characters back when Backspace or a wider box leaves free cells at the end.
Raising it to `low` comes last, so the cursor cell stays shown even in a box too narrow for anything else.
Typing at the end makes `low` equal `fill`, so the end of the line, where you type, stays in view.

```lua defaults/lua/gband/prompt.lua
  local before = table.concat(shown, "", kind.first + 1, at - 1)
  local used = gband.ui.width(before) + gband.ui.width(cell)
  local after = {}
  for i = at + 1, #shown do
    used = used + gband.ui.width(shown[i])
    if used > kind.width then
      break
    end
    after[#after + 1] = shown[i]
  end
  return before, cell, table.concat(after)
end
```

`fitted` returns three pieces: the text before the cursor cell, the cell, and as much of the text after it as fits.

```lua defaults/lua/gband/prompt.lua
local function show(kind)
  local before, cell, after = fitted(kind)
  gband.win.set_lines(kind.win, { { before, { text = cell, hl = "PromptCursor" }, after } })
end
```

The line is three spans, and the middle one is in `PromptCursor`.
A double-width character under the cursor is one span, so both of its cells are reversed.

```lua defaults/lua/gband/prompt.lua
local function clicked(kind, col)
  local shown = characters(kind.marker .. kind.line)
  local edge = 0
  for i = kind.first + 1, #shown do
    edge = edge + gband.ui.width(shown[i])
    if edge > kind.width then
      break
    end
    if col < edge then
      return math.max(0, i - 1 - utf8.len(kind.marker))
    end
  end
  return utf8.len(kind.line)
end
```

`clicked` maps a content column to a cursor.
It walks the drawn characters from `first`, the scroll position of the drawing the user saw, and stops at the one whose cells hold the column, so either cell of a double-width character maps to it.
The marker maps to 0, and a column past the drawn text maps to the end of the line.
The cursor it returns lies inside the drawn run, so the next `fitted` keeps `first` and the text stays where it was clicked.

```lua defaults/lua/gband/prompt.lua
local function flatten(text)
  return (text:gsub("\r\n", " "):gsub("[%z\1-\31\127]", " "):gsub("\194[\128-\159]", " "))
end
```

A paste may hold line breaks and other control characters; each becomes a space, so the paste stays on one line.

```lua defaults/lua/gband/prompt.lua
local function insert(kind, text)
  local at = utf8.offset(kind.line, kind.cursor + 1)
  kind.line = kind.line:sub(1, at - 1) .. text .. kind.line:sub(at)
  kind.cursor = kind.cursor + utf8.len(text)
end
```

`insert` splits the line at the cursor's byte and moves the cursor past the inserted text.

```lua defaults/lua/gband/prompt.lua
local function cut(kind, at)
  local from = utf8.offset(kind.line, at + 1)
  local to = utf8.offset(kind.line, at + 2)
  kind.line = kind.line:sub(1, from - 1) .. kind.line:sub(to)
end
```

`cut` removes the character after the first `at` characters, between the bytes `from` and `to`.
Backspace and Delete both use it.

```lua defaults/lua/gband/prompt.lua
local function move(kind, cursor)
  kind.cursor = math.max(0, math.min(cursor, utf8.len(kind.line)))
  show(kind)
end
```

`move` clamps the cursor to the line, so Left at the start and Right at the end do nothing.

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
  kind.cursor = utf8.len(line)
  kind.first = 0
  kind.target = target
```

Only one prompt is open at a time: opening one closes the other, dropping its line.
A prompt opens with its cursor after the last character, so the rename prompt edits the manual name from its end.
A prompt that is already open only takes focus, so it keeps its line and its cursor.

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
        if kind.cursor == 0 then
          return
        end
        kind.cursor = kind.cursor - 1
        cut(kind, kind.cursor)
        show(kind)
      end,
      delete = function()
        if kind.cursor < utf8.len(kind.line) then
          cut(kind, kind.cursor)
          show(kind)
        end
      end,
      ["ctrl+u"] = function()
        kind.line = ""
        kind.cursor = 0
        show(kind)
      end,
```

Enter reads the text and the target, closes the prompt, and only then submits, wherever the cursor is.
A line that fails therefore leaves no prompt behind, and a floating plugin window that the line opens keeps focus.
Backspace on an empty line closes the prompt; with the cursor at the start of a line that holds text, it does nothing.
Otherwise it moves the cursor back one character and cuts the character there, the one that was before the cursor.
Delete cuts the character after the cursor and leaves the cursor where it is; after the last character it does nothing, and it never closes the prompt.

```lua defaults/lua/gband/prompt.lua
      left = function()
        move(kind, kind.cursor - 1)
      end,
      right = function()
        move(kind, kind.cursor + 1)
      end,
      home = function()
        move(kind, 0)
      end,
      ["end"] = function()
        move(kind, utf8.len(kind.line))
      end,
    },
```

The plugin window's defaults scroll on Home and End and discard Left and Right, as [05](05-plugin-windows.md) shows; a `keys` entry is tried first, so these four take the keys before a default can.

```lua defaults/lua/gband/prompt.lua
    on_input = function(_, text)
      insert(kind, flatten(text))
      show(kind)
    end,
    on_mouse = function(_, event)
      if event.kind == "press" and event.button == "left" and event.content_row == 0 then
        move(kind, clicked(kind, event.content_col))
      end
    end,
    on_resize = function(_, cols)
      kind.width = math.max(1, cols)
      show(kind)
    end,
    on_close = function(win)
      if kind.win == win then
        kind.win = nil
        kind.line = ""
        kind.cursor = 0
        kind.first = 0
        kind.target = nil
      end
    end,
  })
```

Typed characters and pastes both arrive through `on_input`, since the prompt binds no character keys: every key with text that `keys` does not bind goes there, `j`, `k` and `q` included.
`on_mouse` takes only a left press on the content row; a press focuses the plugin window before `on_mouse` runs, so a click on an unfocused prompt focuses it too.
Setting `on_mouse` drops the wheel default, which scrolled nothing in a one-line window.
`on_resize` fits the line to a new width.
`on_close` forgets the line, the cursor and the scroll position, whatever closed the window.

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
  api = 2,
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
