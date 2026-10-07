# 05 · Layout

This chapter reads the layout to find a marked window, jumps back to it, and writes the mark on the window's border.

The finished files are in [examples/tutorial/05-layout/](../../examples/tutorial/05-layout/).

## Reading the layout

`gband.view()` describes what this client shows: `band`, the viewed band, `window`, the focused window, and the active key table among others.
Chapter 01 already used it to find the focused window.

`gband.layout()` describes every band, column and window of the session.
Each band has `id`, `columns` and `floating`; each column has `windows`; each window has `id`.
A floating window this client minimized also has `minimized = true`.
Its `cols` and `rows` give the size of the screen area, and `LayoutChanged` runs when they change, whichever client changed them.
Walking it answers which band a window is in, or nil when the window has left the layout:

```lua
local function band_of(window)
  for _, band in ipairs(gband.layout().bands) do
    for _, column in ipairs(band.columns) do
      for _, tile in ipairs(column.windows) do
        if tile.id == window then
          return band.id
        end
      end
    end
    for _, box in ipairs(band.floating) do
      if box.id == window then
        return band.id
      end
    end
  end
end
```

Both functions return new tables each call, describing the state at that moment.
An action that the running callback dispatched takes effect after the callback returns, so the layout does not show it yet.
[Layout and view](../plugins.md#layout-and-view-gbandlayout-gbandview) lists every field.

## Acting on windows and bands

`gband.window` acts on a window by its number, and `gband.band` on a band:

- `gband.window.focus(window)` views the window's band and focuses the window, and restores it when this client minimized it.
- `gband.window.minimize(window)` hides a floating window from this client until it is focused again.
- `gband.window.rename(window, name)` sets the name its border shows.
- `gband.band.view(band)` views a band.

`toggle` now ignores a window that is not in the layout, and names a marked window after its mark:

```lua
local function toggle(window)
  if band_of(window) == nil then
    return
  end
  if marks[window] then
    marks[window] = nil
    gband.window.rename(window, nil)
    gband.notify("unmarked window " .. window)
    changed(window)
    return
  end
  for letter in gband.opt.mark_letters:gmatch(".") do
    if not taken(letter) then
      marks[window] = letter
      gband.window.rename(window, "mark " .. letter)
      gband.notify("marked window " .. window .. " as " .. letter)
      changed(window)
      return
    end
  end
  gband.bell()
end
```

The name replaces the program's title on the top border:

```screen tests/screenshots/layout_spec/a-mark-names-the-window.txt
I╭mark a───────────╮
 │                 │
1│                 │
2│                 │
 │                 │
 │                 │
 │                 │
 ╰─────────────────╯
```

A border can show more than the window's name: a Lua function can add buttons at its right end, as [window decorations](../plugins.md#window-decorations) describe.

A view of the first band shows how `gband.band` takes a band's number from the layout:

```lua
gband.keymap.set("prefix", "g", function()
  gband.band.view(gband.layout().bands[1].id)
end, { desc = "view the first band" })
```

[Windows and bands](../plugins.md#windows-and-bands-gbandwindow-gbandband) lists the other functions, which resize, move and type into windows.

## Jumping back

A jump finds the window holding a letter and focuses it, wherever it is:

```lua
local function jump(letter)
  for window, marked in pairs(marks) do
    if marked == letter then
      gband.window.focus(window)
      return
    end
  end
  gband.bell()
end
```

Ctrl+Space then `'` enters a key table named `jump`, and the next key picks the letter.
`jump` is not a mode, so the sequence ends with that key and gband returns to interactive mode.
Its bindings are made from the option, so they follow the letters you chose:

```lua
gband.keymap.set("prefix", "'", function()
  gband.keymap.enter("jump")
end, { desc = "jump to a mark" })
for letter in gband.opt.mark_letters:gmatch(".") do
  gband.keymap.set("jump", letter, function()
    jump(letter)
  end, { desc = "jump to mark " .. letter })
end
```

Mark a window with Ctrl+Space then `m`, wander off to another band with `u`, and press `'` then `a` to come back.

## The whole file

The complete `user/init.lua` is [examples/tutorial/05-layout/user/init.lua](../../examples/tutorial/05-layout/user/init.lua).

Next: [06 · Colors](06-colors.md).
