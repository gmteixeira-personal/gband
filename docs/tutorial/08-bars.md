# 08 · Bars

This chapter adds a bar, one column wide, beside the sidebar, showing the focused window's mark.

The finished files are in [examples/tutorial/08-bars/](../../examples/tutorial/08-bars/).

## Adding a bar

A bar reserves columns at the left or right edge of your terminal and shows lines there.
The sidebar is one: the bundled `gband.sidebar` is plain Lua that adds a bar and draws into it.
The windows get the columns the bars leave, as if the terminal were narrower.

`gband.bar.add` adds a bar and returns its id:

```lua
local bar = gband.bar.add({ id = "mark", side = "left", size = 1, order = 1 })
```

Bars on one side stack from the edge inward in ascending `order`.
The sidebar's order is 0, so order 1 puts this bar just inside it.

## Drawing into it

`gband.bar.set_lines` replaces a bar's lines, which take the same spans as a plugin window's.
Row 0 shows the focused window's letter in the `Mark` group, or nothing:

```lua
local function draw(window)
  local letter = window and marks[window]
  gband.bar.set_lines(bar, { letter and { { text = letter, hl = "Mark" } } or "" })
end
```

A bar does not draw itself: your handlers decide when it changes.
Three events cover every case: the client attaching, focus moving, and a mark changing:

```lua
gband.on("Attached", function()
  draw(gband.view().window)
end, { group = group })
gband.on("FocusChanged", function(event)
  draw(event.window)
end, { group = group })
gband.on("User", function()
  draw(gband.view().window)
end, { group = group, pattern = "marks.changed" })
```

With two windows, the right one marked and focused, the bar shows `a` between the sidebar's `I` and the window:

```screen tests/screenshots/bars_spec/shows-the-bar--two-windows.txt
Ia╭─────────────────╮╭─────────────────╮
  │                 ││                 │
1 │                 ││                 │
2 │                 ││                 │
  │                 ││                 │
  │                 ││                 │
  │                 ││                 │
  ╰─────────────────╯╰─────────────────╯
```

`gband.bar.info(bar)` tells where the bar is drawn, and `on_resize` in the spec runs when its size changes, for a bar that lays out its text to fit.
[Side bars](../plugins.md#side-bars-gbandbar) has every field and function, and [The sidebar](../plugins.md#the-sidebar-gbandsidebar) shows what a bar of many rows can hold.

## The whole file

`user/init.lua` now holds a complete small feature: marks with an option, an action, a command, events, a jump table, a highlight group, a plugin window and a bar.
It is [examples/tutorial/08-bars/user/init.lua](../../examples/tutorial/08-bars/user/init.lua).
The next chapter moves it out of your configuration and into a plugin.

Next: [09 · A plugin](09-plugin.md).
