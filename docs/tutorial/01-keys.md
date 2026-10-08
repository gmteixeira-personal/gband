# 01 · Keys

This chapter binds keys of your own, adds a mode, and binds a mouse button.
It ends with the first piece of the feature the rest of the tutorial builds: marking a window with a letter.

The finished files are in [examples/tutorial/01-keys/](../../examples/tutorial/01-keys/).

## Key tables

gband looks up every key you press in a key table:

- `root` holds the keys pressed outside a key sequence, while you type into a window.
- `prefix` holds the keys pressed after the prefix key, Ctrl+Space.
- Any other name is a table of your own, which a binding enters.

`gband.keymap.set(table, key, action, opts)` binds a key in a table.
The action is an action value from `gband.action`, or a function.
Bind two keys in `root`, so Alt+H and Alt+L move focus without the prefix key:

```lua
gband.keymap.set("root", "alt+h", gband.action.focus_column_left, { desc = "focus the column to the left" })
gband.keymap.set("root", "alt+l", gband.action.focus_column_right, { desc = "focus the column to the right" })
```

The `desc` is what the key list, Ctrl+Space then `?`, shows for the binding.
[Actions](../plugins.md#actions-gbandaction) lists every built-in action.

`gband.bind` and `gband.unbind` are shorter forms.
A key alone binds in `root`, and `prefix` and a space before the key bind in `prefix`.
Bind Alt+N to open a window, and remove the key style's Ctrl+Space then `D`, which detaches:

```lua
gband.bind("alt+n", gband.action.open_window)
gband.unbind("prefix D")
```

Bindings made after `gband.keystyle.use()` replace the style's binding for the same key, so the style is a base to build on.
[Key styles](../plugins.md#key-styles-gbandkeystyle) says what each style binds.

The floating style binds far less after the prefix key, because its windows float and the mouse drives them.
Its prefix key opens the window list, and a key that its `prefix` table binds, such as `D` or `?`, runs from that list.
A key you bind in `prefix` becomes a shortcut of the list too, except `n`, `s` and the digits: in the list they pick a line, `New window`, `Settings` or a window, and do not run a `prefix` binding of the same key.
Its `leftmouse` and `rightmouse` bindings in `root` act only on a floating window's border and empty ribbon, and return `false` for every other click, so a click in a window still focuses it and reaches the program, as the next section explains.

## Modes

In a table that is not a mode, the next key ends the sequence.
A mode stays active after each key, until a binding enters another table.
`gband.keymap.mode` makes a table a mode, and `gband.keymap.enter` makes a table active from a binding:

```lua
gband.keymap.mode("resize", { label = "resize" })
gband.keymap.set("resize", "h", gband.action.shrink_column_width, { desc = "narrower" })
gband.keymap.set("resize", "l", gband.action.grow_column_width, { desc = "wider" })
gband.keymap.set("resize", "escape", function()
  gband.keymap.enter("root")
end, { desc = "done" })
gband.keymap.set("root", "alt+r", function()
  gband.keymap.enter("resize")
end, { desc = "resize" })
```

Alt+R enters the mode, and the sidebar's first row shows `R`, the first letter of the label.
`h` and `l` resize the focused column as often as you press them, and Escape returns to `root`.
The modal key style works the same way: its `prefix` table is a mode labelled `navigation`.
[Modes](../plugins.md#modes) has the rules.

## Marking a window

The rest of the tutorial builds one feature: you mark windows with a letter, then jump back to them, list them, and show them in a bar.
Start with the marks themselves, kept in a table from window numbers to letters:

```lua
local marks = {}
local letters = "abc"

local function taken(letter)
  for _, marked in pairs(marks) do
    if marked == letter then
      return true
    end
  end
  return false
end

local function toggle(window)
  if marks[window] then
    marks[window] = nil
    print("unmarked window " .. window)
    return
  end
  for letter in letters:gmatch(".") do
    if not taken(letter) then
      marks[window] = letter
      print("marked window " .. window .. " as " .. letter)
      return
    end
  end
end
```

`toggle` gives a window the first free letter, or takes its letter away.
Bind it to Ctrl+Space then `m`.
The function asks `gband.view()` for the focused window, which chapter 05 explains:

```lua
gband.keymap.set("prefix", "m", function()
  local window = gband.view().window
  if window then
    toggle(window)
  end
end, { desc = "mark the window" })
```

Press it and look in the client log for `marked window 1 as a`.

## Mouse names

Mouse buttons bind like keys: `leftmouse`, `middlemouse` and `rightmouse`, with `ctrl`, `alt` and `shift`, and `wheelup` and `wheeldown` for the wheel.
A function bound to a button gets the mouse event, whose `window` is the window under the pointer.
Mark a window by clicking it with Ctrl and the right button:

```lua
gband.keymap.set("root", "ctrl+rightmouse", function(event)
  if event.window then
    toggle(event.window)
  end
end, { desc = "mark the window under the pointer" })
```

A mouse binding in `root` replaces what the click would otherwise do, unless its function returns `false`, so pick a combination you do not use in programs.
[Mouse names](../plugins.md#mouse-names) lists the names and the event's fields.

### Passing a click on

A function that returns `false` passes the click on, and the click does what it would do with no binding: in `root`, it focuses the window and selects text or reaches the program.
The event's `box_row` is the row inside the window's box, 0 on its top border.
Mark a window by clicking its top border with Ctrl and the left button, and leave every other Ctrl click as it was:

```lua
gband.keymap.set("root", "ctrl+leftmouse", function(event)
  if event.target == "window" and event.box_row == 0 then
    toggle(event.window)
    return
  end
  return false
end, { desc = "mark the window by its top border" })
```

## The whole file

The complete `user/init.lua` is [examples/tutorial/01-keys/user/init.lua](../../examples/tutorial/01-keys/user/init.lua).
[Key bindings](../plugins.md#key-bindings-gbandkeymap) is the full reference for this chapter.

Next: [02 · Actions and commands](02-actions.md).
