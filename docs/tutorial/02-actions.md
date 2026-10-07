# 02 · Actions and commands

This chapter turns marking into an action and a command, acts on windows other than the focused one, and reaches outside gband: a notification, the bell, the clipboard, a URL and a new program.

The finished files are in [examples/tutorial/02-actions/](../../examples/tutorial/02-actions/).

## Actions

An action is a named thing gband can do, such as `gband.action.focus_column_left`.
Binding a key to an action value, rather than to a function, lets the key list show the action's description and lets other code dispatch it by name.
`gband.action.register` adds an action of your own and returns its value:

```lua
gband.action.register("mark", function()
  local window = gband.view().window
  if window then
    toggle(window)
  end
end, { desc = "mark the window" })
```

From now on `gband.action.mark` holds it, and the binding becomes one line:

```lua
gband.keymap.set("prefix", "m", gband.action.mark)
```

Calling an action value inside a callback dispatches it, as pressing a key bound to it does.
[Actions](../plugins.md#actions-gbandaction) lists the built-in actions.

## Commands

A command is a named function that takes a table of arguments.
Where an action always acts on the focused window, a command can say which window:

```lua
gband.cmd.register("mark", function(args)
  local window = args.window or gband.view().window
  if window then
    toggle(window)
  end
end, { desc = "mark a window", args = { "window" } })
```

`gband.cmd.run("mark", { window = 2 })` marks window 2, and returns `true` when the command ran.
The mouse binding of chapter 01 now runs the command:

```lua
gband.keymap.set("root", "ctrl+rightmouse", function(event)
  if event.window then
    gband.cmd.run("mark", { window = event.window })
  end
end, { desc = "mark the window under the pointer" })
```

[Commands](../plugins.md#commands-gbandcmd) describes `gband.cmd.list`, which a command palette can read.

## Action targets

The built-in actions that act on the focused window also take a target, a table naming another window.
`gband.action.close_window({ window = 3 })` closes window 3, whichever window has focus.
An action that closes every marked window walks the marks in letter order:

```lua
local function sorted()
  local entries = {}
  for window, letter in pairs(marks) do
    entries[#entries + 1] = { window = window, letter = letter }
  end
  table.sort(entries, function(a, b)
    return a.letter < b.letter
  end)
  return entries
end
```

```lua
gband.action.register("close_marked", function()
  for _, entry in ipairs(sorted()) do
    gband.action.close_window({ window = entry.window })
  end
end, { desc = "close the marked windows" })
```

The marks of the closed windows stay in the table for now; chapter 04 forgets them.
[Action targets](../plugins.md#action-targets) lists which actions take a target and what else they take.

## Starting a program

`gband.spawn` opens a window running a command line.
`after` places the new column after a window, here the focused one:

```lua
gband.keymap.set("prefix", "t", function()
  gband.spawn({ cmd = "top", after = gband.view().window })
end, { desc = "open top beside the window" })
```

## Outside gband

These functions act on the machine and the terminal the client runs on, and work only in a callback:

- `gband.notify(text)` asks the terminal for a desktop notification.
- `gband.bell()` rings the terminal's bell.
- `gband.clipboard(text)` sets the terminal's clipboard.
- `gband.open(target)` opens a URL or a path with `xdg-open`, or `open` on macOS.

`toggle` now notifies instead of printing, and rings the bell when every letter is taken:

```lua
local function toggle(window)
  if marks[window] then
    marks[window] = nil
    gband.notify("unmarked window " .. window)
    return
  end
  for letter in letters:gmatch(".") do
    if not taken(letter) then
      marks[window] = letter
      gband.notify("marked window " .. window .. " as " .. letter)
      return
    end
  end
  gband.bell()
end
```

Copy the marks to the clipboard, one `letter window` line each:

```lua
gband.action.register("copy_marks", function()
  local lines = {}
  for _, entry in ipairs(sorted()) do
    lines[#lines + 1] = entry.letter .. " " .. entry.window
  end
  gband.clipboard(table.concat(lines, "\n"))
end, { desc = "copy the marks" })
```

And keep this tutorial a key away:

```lua
gband.keymap.set("prefix", "H", function()
  gband.open("https://github.com/gmteixeira-personal/gband/blob/main/docs/tutorial/README.md")
end, { desc = "open the tutorial" })
```

Whether a notification or a clipboard write arrives depends on your terminal; [Local actions](../plugins.md#local-actions) says which sequences gband writes.
Bind the new actions:

```lua
gband.keymap.set("prefix", "y", gband.action.copy_marks)
gband.keymap.set("prefix", "X", gband.action.close_marked)
```

## The whole file

The complete `user/init.lua` is [examples/tutorial/02-actions/user/init.lua](../../examples/tutorial/02-actions/user/init.lua).

Next: [03 · Options](03-options.md).
