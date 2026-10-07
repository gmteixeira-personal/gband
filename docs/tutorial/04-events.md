# 04 · Events

This chapter reacts to what happens in gband: it forgets the mark of a window that closes, and raises an event of its own whenever a mark changes, which the plugin window and the bar of later chapters redraw on.

The finished files are in [examples/tutorial/04-events/](../../examples/tutorial/04-events/).

## Handlers

`gband.on(event, fn, opts)` runs `fn` each time `event` happens, with a table describing it.
`WindowClosed` happens when a window leaves the layout, and its table holds the window's number.
A handler for it fixes the stale marks that chapter 02 left behind:

```lua
gband.on("WindowClosed", function(event)
  if marks[event.window] then
    marks[event.window] = nil
    changed(event.window)
  end
end, { group = group })
```

[Events](../plugins.md#events-gbandon-gbandaugroup-gbandemit) lists every event and the fields of its table, such as `FocusChanged`, which chapter 08 uses.

## Groups

A group collects handlers that belong together.
`gband.augroup(name)` returns the group's id and removes every handler already in it, so defining the group at the top of a file keeps the file from adding the same handlers twice:

```lua
local marks = {}
local group = gband.augroup("marks")
```

Passing `{ group = group }` to `gband.on` puts a handler in the group.
Calling `gband.augroup("marks")` again, from a binding or the prompt, removes them all at once: try it, and marking no longer prints.

## Events of your own

`gband.emit(name, data)` raises a `User` event and runs its handlers before it returns.
The marks raise one whenever a window gains or loses its mark:

```lua
local function changed(window)
  gband.emit("marks.changed", { window = window, letter = marks[window] })
end
```

`toggle` calls `changed(window)` after each change, as the `WindowClosed` handler does.
A handler of `User` events gets `name` and `data`, and `pattern` runs it for one name only:

```lua
gband.on("User", function(event)
  print("marks changed", event.data.window, event.data.letter)
end, { group = group, pattern = "marks.changed" })
```

The code that changes the marks no longer needs to know who cares.
Chapter 07 redraws a list of marks from such a handler, and chapter 08 a bar.

`gband.emit` works only inside a callback: a binding, a handler, a command or a line at the prompt.
At the top level of `user/init.lua` the configuration is still loading, and nothing is listening yet.

## The whole file

The complete `user/init.lua` is [examples/tutorial/04-events/user/init.lua](../../examples/tutorial/04-events/user/init.lua).

Next: [05 · Layout](05-layout.md).
