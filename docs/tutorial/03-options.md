# 03 · Options

This chapter sets gband's own options, makes the mark letters an option of your own, and lets the settings window turn the sidebar off.

The finished files are in [examples/tutorial/03-options/](../../examples/tutorial/03-options/).

## gband's options

`gband.opt.<name>` reads and sets an option: `gband.opt.prefix = "ctrl+b"` moves the prefix key.
`gband.set` sets several at once.
Make the resize mode of chapter 01 move in smaller steps, a twentieth of the screen instead of a tenth:

```lua
gband.set {
  width_step = 1/20,
  height_step = 1/20,
}
```

Scrolling, band switches and resizes animate.
Make them run twice as fast:

```lua
gband.opt.animation_speed = 2
```

The speed goes from 0.1 to 10.
`gband.opt.animations = false` turns the animations off instead, and `GBAND_ANIMATIONS=off` before you attach turns them off for that client whatever the option holds.

Options are set while the configuration loads, so set them at the top level of `user/init.lua`, never inside a binding.
An invalid value is reported at its line and the option keeps its default.
[Options](../plugins.md#options-gbandopt) lists every option of the client and the server.

## An option of your own

`gband.opt.declare` adds an option, with a type, a default and a description.
The letters that marks take become one:

```lua
gband.opt.declare("mark_letters", {
  type = "string",
  default = "abc",
  desc = "the letters that marks take, in order",
})
```

`toggle` reads it where it read the local `letters`, which goes:

```lua
  for letter in gband.opt.mark_letters:gmatch(".") do
```

Now a configuration chooses its letters with one assignment.
An assignment made before the declaration waits for it, so this line works at the very top of `user/init.lua`:

```lua
gband.opt.mark_letters = "xyz"
```

In chapter 09 the marks move into a plugin, and the same line becomes `gband.opt["marks.letters"] = "xyz"`, because a plugin's options are named after it.
`gband.opt.list()` returns every option with its type, default and value; try `print(#gband.opt.list())` at the prompt.

## The settings window

Ctrl+Space then `s` opens the settings window.
It saves each choice in a file of its own under `user/`, and `gband.settings` reads them back.
The `sidebar` line saves `user/sidebar.lua`, but a choice applies only where your configuration asks for it.
Ask for this one:

```lua
if gband.settings.sidebar() ~= false then
  gband.plugin("gband.sidebar")
end
```

Saving a setting reloads the configuration, so turning the sidebar off in the window removes it at once.
`gband.settings.theme()`, `gband.settings.interactive_on_new()` and `gband.keystyle.saved()` read the other lines; `gband.keystyle.use()` already follows the `keys` line.
`I on new` is off until the window saves it on, and `interactive_on_new()` returns nil until then, so an `n` binding of your own compares the value with `true`.
[Settings](../plugins.md#settings-gbandsettings) describes the window and its files.

## The whole file

The complete `user/init.lua` is [examples/tutorial/03-options/user/init.lua](../../examples/tutorial/03-options/user/init.lua).

Next: [04 · Events](04-events.md).
