# 02 · The palette and highlight groups

A theme hands its colors to two modules: `gband.palette`, for the terminal's colors, and `gband.hl`, for the named styles gband draws with.
This chapter reads both, `defaults/lua/gband/palette.lua` and `defaults/lua/gband/hl.lua`, in full.

## `gband.palette`

The palette module is the smallest API module, and it shows the shape most of them share: check the caller's arguments in Lua, then hand clean values to a primitive.

```lua defaults/lua/gband/palette.lua
local core = gband.core
```

Every module that needs primitives keeps `gband.core` in a local.
That is also why a quote always says `core.palette.set` where the reference says `gband.core.palette.set`.

```lua defaults/lua/gband/palette.lua
local FIELDS = {
  fg = true, bg = true,
  black = true, red = true, green = true, yellow = true,
  blue = true, magenta = true, cyan = true, white = true,
  bright_black = true, bright_red = true, bright_green = true, bright_yellow = true,
  bright_blue = true, bright_magenta = true, bright_cyan = true, bright_white = true,
}
```

`FIELDS` is the set of names a palette may hold: `fg`, `bg` and the sixteen ANSI colors.
Writing it as a set, with `true` for each name, makes the membership test in `set` a single table lookup.

```lua defaults/lua/gband/palette.lua
gband.palette = {}
```

```lua defaults/lua/gband/palette.lua
function gband.palette.set(spec)
  if type(spec) ~= "table" then
    error("gband.palette.set expects a table of colors", 2)
  end
  local parsed = {}
  for field, value in pairs(spec) do
    if not FIELDS[field] then
      error("unknown palette field `" .. tostring(field) .. "`", 2)
    end
    if type(value) ~= "string" or not value:match("^#%x%x%x%x%x%x$") then
      error("invalid color for the palette field `" .. field .. "`: " .. tostring(value), 2)
    end
    parsed[field] = value:lower()
  end
  core.palette.set(parsed)
end
```

`set` checks every field before it changes anything.
Each `error` call passes level 2, so the message names the caller's line, such as `user/colors/dusk.lua:3:`, and not a line of `palette.lua`.
Colors are lowercased, so `#EBDBB2` and `#ebdbb2` are the same palette.

`gband.core.palette.set` replaces the whole palette.
A field the table leaves out is not kept from the previous palette; that color goes back to the terminal's own.
So `gband.palette.set({})` hands every color back to the terminal, which is what `gband.colorscheme` does before it loads a theme, in [03](03-colorschemes.md).

```lua defaults/lua/gband/palette.lua
function gband.palette.get()
  return core.palette.get()
end
```

`get` returns what `gband.core.palette.get` returns: a new table, so a caller that changes it changes nothing.

## `gband.hl`: the data

A highlight group is a named style, such as `SidebarMode`.
Each group has two layers: a default, which the module or plugin that draws with the group declares, and an explicit style, which a theme or your configuration sets.
The explicit style wins whenever it exists.
Keeping both lets a theme change a group without caring whether the plugin that declares it loaded first, and lets switching themes drop every explicit style while the defaults stay.

```lua defaults/lua/gband/hl.lua
local core = gband.core
```

```lua defaults/lua/gband/hl.lua
local NAMED = {
  black = 0, red = 1, green = 2, yellow = 3, blue = 4, magenta = 5, cyan = 6, white = 7,
  bright_black = 8, bright_red = 9, bright_green = 10, bright_yellow = 11,
  bright_blue = 12, bright_magenta = 13, bright_cyan = 14, bright_white = 15,
}
local FLAGS = { bold = true, italic = true, underline = true, reverse = true, dim = true }
local STYLE_FIELDS = { "fg", "bg", "bold", "italic", "underline", "reverse", "dim" }
```

A color in a style is one of three things: `#rrggbb`, a palette index from 0 to 255, or one of the sixteen names in `NAMED`.
A named color stays a name while it is stored, so `gband.hl.get` gives back what was set, and becomes its index only when the style is drawn.
`FLAGS` lists the boolean attributes, and `STYLE_FIELDS` every field a drawn style can hold, in a fixed order.

```lua defaults/lua/gband/hl.lua
local groups = {}
local warned = {}
local internal = { quiet = false }
```

`groups` maps each name to a table with up to two fields, `default` and `explicit`.
`warned` remembers the link cycles already logged, as `resolve` below shows.
`internal` is the table the module returns: its exports, which `gband.colorscheme` and the other modules use, and the flag `quiet`.

## Checking a style

```lua defaults/lua/gband/hl.lua
local function valid_name(name)
  return type(name) == "string" and name:match("^[A-Za-z][A-Za-z0-9_.]*$") ~= nil
end
```

A group name starts with a letter and holds letters, digits, underscores and full stops, so a plugin can name its groups `marks.Current`.

```lua defaults/lua/gband/hl.lua
local function color(value)
  if type(value) == "string" then
    if value:match("^#%x%x%x%x%x%x$") then
      return value:lower()
    end
    if NAMED[value] then
      return value
    end
  elseif math.type(value) == "integer" and value >= 0 and value <= 255 then
    return value
  elseif math.type(value) == "float" and value >= 0 and value <= 255 and value % 1 == 0 then
    return math.tointeger(value)
  end
  return nil
end
```

`color` returns the stored form of a color, or nil when the value is not one.
Lua 5.4 tells integers from floats, and `5.0` reads as a float, so a float that is a whole number is turned into an integer rather than refused.

```lua defaults/lua/gband/hl.lua
local function normalise(name, spec)
  if type(spec) ~= "table" then
    return nil, "the style of `" .. name .. "` must be a table"
  end
  local out = {}
  for field, value in pairs(spec) do
    if field == "fg" or field == "bg" then
      local normalised = color(value)
      if normalised == nil then
        return nil, "invalid color for `" .. field .. "` of `" .. name .. "`: " .. tostring(value)
      end
      out[field] = normalised
    elseif FLAGS[field] then
      if type(value) ~= "boolean" then
        return nil, "`" .. field .. "` of `" .. name .. "` must be a boolean"
      end
      out[field] = value
    elseif field == "link" then
      if not valid_name(value) then
        return nil, "`link` of `" .. name .. "` must be a group name"
      end
      if value == name then
        return nil, "the group `" .. name .. "` cannot link to itself"
      end
      out.link = value
    else
      return nil, "unknown field `" .. tostring(field) .. "` in the style of `" .. name .. "`"
    end
  end
  return out
end
```

`normalise` builds a new table rather than keeping the caller's, so a caller that later changes its table changes no group.
Every failure returns nil and a reason instead of raising an error.
The caller, `store`, decides how to report it.
A group may also `link` to another group, and draws as that group with its own fields on top; a link to itself is refused here, and a longer loop below.

```lua defaults/lua/gband/hl.lua
local function copy(spec)
  if spec == nil then
    return nil
  end
  local out = {}
  for field, value in pairs(spec) do
    out[field] = value
  end
  return out
end
```

```lua defaults/lua/gband/hl.lua
local function same(a, b)
  if a == nil or b == nil then
    return a == b
  end
  for field, value in pairs(a) do
    if b[field] ~= value then
      return false
    end
  end
  for field in pairs(b) do
    if a[field] == nil then
      return false
    end
  end
  return true
end
```

`copy` and `same` are shallow, because a style holds only strings, numbers and booleans.
`same` lets `store` skip the change event when a style is set to what it already is.

## Storing a style

```lua defaults/lua/gband/hl.lua
local function definition(name, override_name, override_layer, override_spec)
  local group = groups[name] or {}
  local explicit, default = group.explicit, group.default
  if name == override_name then
    if override_layer == "explicit" then
      explicit = override_spec
    else
      default = override_spec
    end
  end
  return explicit or default
end
```

`definition` returns what a group draws as: its explicit style, or its default.
The last three parameters ask "what would the group be if this layer held this style?", which the cycle check needs before it stores anything.
With them left out, as `resolve` calls it, the override never applies.

```lua defaults/lua/gband/hl.lua
local function cycle_from(name, layer, spec)
  local chain = { name }
  local seen = { [name] = true }
  local current = definition(name, name, layer, spec)
  while current and current.link do
    local target = current.link
    if target == name then
      return chain
    end
    if seen[target] then
      return nil
    end
    seen[target] = true
    chain[#chain + 1] = target
    current = definition(target, name, layer, spec)
  end
  return nil
end
```

`cycle_from` follows the links that `name` would have after the store.
It returns the chain only when the links come back to `name`, because only then is this store the one that closes a loop.
A loop among other groups that does not pass through `name` is reported elsewhere, by `resolve`.

```lua defaults/lua/gband/hl.lua
local function changed(name)
  if core.loading() or internal.quiet then
    return
  end
  core.emit("HighlightChanged", { group = name })
end
```

A changed group emits `HighlightChanged`, so a plugin can redraw.
It stays quiet while the configuration loads, and while `internal.quiet` is set, which `gband.colorscheme` does around a theme that sets dozens of groups at once.

```lua defaults/lua/gband/hl.lua
local function store(layer, name, spec)
  if not valid_name(name) then
    return "invalid highlight group name `" .. tostring(name) .. "`"
  end
  local normalised = nil
  if spec ~= nil then
    local reason
    normalised, reason = normalise(name, spec)
    if not normalised then
      return reason
    end
  end
  local cycle = cycle_from(name, layer, normalised)
  if cycle then
    return "highlight link cycle through " .. table.concat(cycle, ", ")
  end
  local group = groups[name] or {}
  groups[name] = group
  if same(group[layer], normalised) then
    group[layer] = normalised
    return nil
  end
  group[layer] = normalised
  changed(name)
  return nil
end
```

`store` returns an error message, or nil when it stored the style.
A nil `spec` clears the layer.
When the new style equals the old one, it is stored all the same, but no event is emitted.

## Resolving a style

A group's links have to be followed before it can be drawn.

```lua defaults/lua/gband/hl.lua
local function resolve(name)
  local chain = {}
  local seen = {}
  local visited = {}
  local current = name
```

```lua defaults/lua/gband/hl.lua
  while current do
    if seen[current] then
      local members = {}
      local inside = false
      for _, member in ipairs(visited) do
        inside = inside or member == current
        if inside then
          members[#members + 1] = member
        end
      end
      table.sort(members)
      local key = table.concat(members, ",")
      if not warned[key] then
        warned[key] = true
        core.warn("highlight link cycle through " .. table.concat(members, ", "))
      end
      break
    end
    local spec = definition(current)
    if not spec then
      break
    end
    seen[current] = true
    visited[#visited + 1] = current
    chain[#chain + 1] = spec
    current = spec.link
  end
```

The loop walks from the group along its links, collecting each definition in `chain`.
`cycle_from` stops a store from closing a loop, but a loop can still appear: a default can link `B` to `A` while an explicit style of `B` hides it, and `clear` and `restore` below change explicit styles without checking.
`resolve` therefore stops at the first group it has already seen and warns once per loop, naming the members in sorted order so the same loop always has the same key.

```lua defaults/lua/gband/hl.lua
  local style = {}
  for index = #chain, 1, -1 do
    for _, field in ipairs(STYLE_FIELDS) do
      local value = chain[index][field]
      if value ~= nil then
        style[field] = value
      end
    end
  end
  return style
end
```

The styles are merged from the end of the chain back to the group itself, so a group's own fields win over the fields of the group it links to.

```lua defaults/lua/gband/hl.lua
function internal.drawn(name)
  local style = resolve(name)
  for _, field in ipairs({ "fg", "bg" }) do
    if type(style[field]) == "string" and NAMED[style[field]] then
      style[field] = NAMED[style[field]]
    end
  end
  return style
end
```

`drawn` is the export that bars, frames and the styles provider use.
The client draws with palette indexes, not names, so this is where `"blue"` becomes `4`.

## The exports for themes

```lua defaults/lua/gband/hl.lua
function internal.snapshot()
  local saved = {}
  for name, group in pairs(groups) do
    saved[name] = group.explicit
  end
  return saved
end

function internal.clear()
  for _, group in pairs(groups) do
    group.explicit = nil
  end
end

function internal.restore(saved)
  for name, group in pairs(groups) do
    group.explicit = saved[name]
  end
end
```

These three exist for `gband.colorscheme`.
Before it loads a theme it takes a `snapshot` of every explicit style and calls `clear`.
If the theme fails, `restore` puts the snapshot back, so a broken theme leaves the previous one in place.
They touch the `explicit` layer only; defaults belong to the code that declared them.

## The public API

```lua defaults/lua/gband/hl.lua
gband.hl = {}

function gband.hl.set(name, spec)
  local reason = store("explicit", name, spec)
  if reason then
    error(reason, 2)
  end
end

function gband.hl.default(name, spec)
  if spec == nil then
    error("gband.hl.default expects a style table", 2)
  end
  local reason = store("default", name, spec)
  if reason then
    error(reason, 2)
  end
end

function gband.hl.get(name, opts)
  if not valid_name(name) then
    error("invalid highlight group name `" .. tostring(name) .. "`", 2)
  end
  if opts ~= nil and type(opts) ~= "table" then
    error("gband.hl.get expects its options as a table", 2)
  end
  if opts and opts.resolve then
    return resolve(name)
  end
  return copy(definition(name))
end
```

`gband.hl.set` writes the explicit layer, and `gband.hl.default` the default one, which must be a table.
Both raise their errors at level 2, the caller's line.
`gband.hl.get` returns a copy of the definition, or with `{ resolve = true }` the merged style, with named colors kept as names.

## The client's own groups

```lua defaults/lua/gband/hl.lua
gband.hl.default("WindowBorder", { dim = true })
gband.hl.default("WindowBorderFocused", { fg = "#b1b9f9", bold = true })
gband.hl.default("ErrorBanner", { fg = "red", reverse = true })
```

The client draws three things itself: the borders of windows and the error banner.
Their groups are declared here, with defaults that work on any palette.
Every other group is declared by the module that draws with it, as the next chapters show.

## The styles provider

```lua defaults/lua/gband/hl.lua
core.provide("styles", function()
  return {
    border = internal.drawn("WindowBorder"),
    border_focused = internal.drawn("WindowBorderFocused"),
    banner = internal.drawn("ErrorBanner"),
  }
end)
```

The client knows nothing of highlight groups.
Before each draw it calls the `styles` provider and gets plain styles back.
This is the first provider in the prelude; without it, the client would draw borders and the banner in its built-in styles, whatever the theme.

```lua defaults/lua/gband/hl.lua
return internal
```

The module returns `internal`, so `require("gband.hl")` gives other modules `drawn`, `snapshot`, `restore`, `clear` and `quiet`, while `gband.hl` is what configuration sees.
