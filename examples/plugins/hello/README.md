# hello

A sample gband plugin. It shows the parts of the plugin API that most plugins use:

- the option `hello.greeting`, declared with `gband.opt.declare`
- the action `hello.greet`, which opens a pane that prints the greeting and then runs your shell
- the command `hello.say`, which logs the greeting to the client log
- a `FocusChanged` handler in the handler group `hello`, which logs the focused pane

Read [the plugin guide](../../../docs/plugins.md) for the API it uses.

## Install

gband loads plugins from `$XDG_DATA_HOME/gband/plugins/`, or `~/.local/share/gband/plugins/` when `XDG_DATA_HOME` is unset.
Link this directory there:

```sh
mkdir -p ~/.local/share/gband/plugins
ln -s "$PWD/examples/plugins/hello" ~/.local/share/gband/plugins/hello
```

Then set the plugin up and bind its action in `~/.config/gband/user/init.lua`:

```lua
gband.plugin("hello", { greeting = "hi" })
gband.keymap.set("prefix", "g", gband.action["hello.greet"], { desc = "Greet" })
```

A `user/init.lua` replaces the default configuration, so start from a copy of `defaults/init.lua` to keep the default bindings.
Ctrl+Space then `g` opens a pane that prints `hi`.
The client log, in `$XDG_STATE_HOME/gband/log/`, records each focus change.

## Tests

`tests/hello_spec.lua` presses the greeting key and compares the screen with its reference, runs the `say` command, and reads the client log.
Run it with `gband test` in this directory; [the testing guide](../../../docs/testing.md) describes the API.
