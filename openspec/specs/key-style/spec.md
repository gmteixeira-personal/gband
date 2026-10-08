# key-style Specification

## Purpose

Defines gband's two key styles, modal and direct: the bundled preset files that make each style's bindings, how a configuration uses one, where the user's choice is saved, the chooser that saves it, and the offer of that chooser on the first start.

## Requirements

### Requirement: Key style presets
gband SHALL bundle three key style presets, the modules `gband.keystyle.modal`, `gband.keystyle.direct` and `gband.keystyle.floating`, which `require` finds as the plugins capability defines for bundled modules. Each preset SHALL be a file of top-level calls that uses only the `gband` API a configuration file can use. Requiring a preset while the configuration loads SHALL make its bindings in that configuration. A preset SHALL set no option. In `root`, the modal and direct presets SHALL bind only the mouse names that the client-attach capability's "Default mouse bindings" gives for `root`, and no key. In `root`, the floating preset SHALL bind those mouse names, then `leftmouse` and `rightmouse`, as the floating-key-style capability's "Floating bindings" defines, and no key.

Before its bindings, each preset SHALL set up, with `gband.plugin` and no options, each bundled plugin that registers an action it binds: the key list plugin `gband.keylist`, then the Lua prompt plugin `gband.prompt`, and, for the floating preset only, then the desktop plugin `gband.desktop`. It SHALL set up no other plugin, so the error list plugin `gband.errors`, whose action no preset binds, and the sidebar plugin `gband.sidebar` stay with the default configuration, as the configuration capability defines. A binding to an action SHALL take the description of the action it binds, as the actions capability lists them or as `gband.action.list()` gives it for a registered action. A binding to a Lua function SHALL take the description that the client-attach capability's default table, or for the floating preset the floating-key-style capability's "Floating bindings", gives its key.

Each action that a preset binds to `h`, `j`, `k` or `l`, alone or with Ctrl, SHALL also be bound to Left, Down, Up or Right, with the same modifier.

#### Scenario: Preset required by name
- **WHEN** `user/init.lua` holds only `require("gband.keystyle.direct")`, and the user presses Ctrl+Space then `h` with the second of two columns focused
- **THEN** loading succeeds and the first column is focused

#### Scenario: Preset shadowed by a user module
- **WHEN** `user/lua/gband/keystyle/direct.lua` binds only `prefix x`, and `user/init.lua` calls `gband.keystyle.use("direct")`
- **THEN** `gband.keymap.list("prefix")` holds only the entry for `x`

#### Scenario: Arrow keys beside hjkl
- **WHEN** the modal or the direct preset is in use and `gband.keymap.list("prefix")` is read
- **THEN** `left` binds the action `h` binds, and `ctrl+up` binds the action `ctrl+k` binds

#### Scenario: Bundled plugins set up by the preset
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("modal")`, and the user presses Ctrl+Space then `?`
- **THEN** loading succeeds and the key list opens
- **AND** `gband.action.list()` holds `keylist.open` and `prompt.open`, and no `errors.open` and no `desktop.list`
- **AND** no sidebar is drawn

#### Scenario: Only mouse names in root
- **WHEN** the modal or the direct preset is in use and `gband.keymap.list("root")` is read
- **THEN** it holds `mod+leftmouse`, `mod+rightmouse`, `mod+middlemouse`, `mod+wheeldown` and `mod+wheelup`, in that order, and nothing else

#### Scenario: Floating preset's mouse names in root
- **WHEN** the floating preset is in use and `gband.keymap.list("root")` is read
- **THEN** it holds `mod+leftmouse`, `mod+rightmouse`, `mod+middlemouse`, `mod+wheeldown`, `mod+wheelup`, `leftmouse` and `rightmouse`, in that order, and nothing else

#### Scenario: Remove a preset mouse binding
- **WHEN** `user/init.lua` calls `gband.keystyle.use()` and then `gband.keymap.del("root", "mod+leftmouse")`, and the user drags across a program that reports the mouse with the left button and Alt held
- **THEN** no window moves and the program receives the press

### Requirement: Modal preset
`gband.keystyle.modal` SHALL declare `prefix` a mode with the label `navigation`, with `gband.keymap.mode`. It SHALL make the bindings of the modal key style that the client-attach capability's default table gives, in that order.

#### Scenario: Navigation mode
- **WHEN** `user/init.lua` calls `gband.keystyle.use("modal")`, the first of three columns is focused, and the user presses Ctrl+Space, `l`, then `l`
- **THEN** the third column is focused and no window receives a key
- **AND** `gband.keymap.label("prefix")` returns `navigation`

#### Scenario: Escape returns to typing
- **WHEN** the modal preset is in use and the user presses Ctrl+Space, `h`, Escape, then types `echo left` and Enter, with the second of two columns focused
- **THEN** `left` appears in the first tile only

### Requirement: Direct preset
`gband.keystyle.direct` SHALL declare no mode. It SHALL make the bindings of the direct key style that the client-attach capability defines, in that order. `prefix` is then not a mode, so each key after the prefix key SHALL end the key sequence, and the keys that follow SHALL go to the focused plugin window or window.

#### Scenario: One key per action
- **WHEN** the direct preset is in use, the second of two columns is focused, and the user presses Ctrl+Space, `h`, then types `echo left` and Enter
- **THEN** the first column is focused and `left` appears in the first tile only

#### Scenario: A second key needs the prefix again
- **WHEN** the direct preset is in use, the first of three columns is focused, and the user presses Ctrl+Space, `l`, then `l`
- **THEN** the second column is focused
- **AND** the focused window receives the second `l`

#### Scenario: Open a window
- **WHEN** the direct preset is in use, one window is focused, and the user presses Ctrl+Space then `n`
- **THEN** a second window opens right of the first and is focused
- **AND** `root` is active, so `echo $GBAND_WINDOW` runs in the new window

#### Scenario: Escape after the prefix
- **WHEN** the direct preset is in use and the user presses Ctrl+Space then Escape
- **THEN** the focused window receives nothing and `root` is active

#### Scenario: Hints and labels of the direct style
- **WHEN** the default configuration is in use with the direct key style saved, `root` is active, and the user presses Ctrl+Space
- **THEN** the sidebar's mode row shows `P`
- **AND** after `?`, the key list's title is `prefix keys`

### Requirement: Use a key style
`gband.keystyle.use(style)` SHALL make the bindings of one preset by requiring the module `gband.keystyle.<style>`, and SHALL return the style's name. `style` SHALL be `"modal"`, `"direct"`, `"floating"` or nil. With nil, it SHALL use the saved key style, as "Saved key style" defines, or `"modal"` when none is saved. A configuration MAY bind and unbind keys after the call, as with any other binding.

`gband.keystyle.use` SHALL be callable only while the configuration loads, and only once in one load. Any other `style`, a second call in the same load, and a call after loading SHALL be an error at the line of the call.

`gband.keystyle.current()` SHALL return the name of the style that `gband.keystyle.use` picked in this load, or nil when it was not called in this load. Requiring a preset by its module name SHALL NOT change what it returns. It SHALL be callable while the configuration loads and in any callback, and SHALL raise no error.

#### Scenario: Explicit style
- **WHEN** `user/init.lua` calls `gband.keystyle.use("direct")`
- **THEN** the call returns `direct`
- **AND** `gband.keymap.label("prefix")` returns `prefix`

#### Scenario: Floating style
- **WHEN** `user/init.lua` calls `gband.keystyle.use("floating")`
- **THEN** the call returns `floating`
- **AND** `gband.keymap.label("prefix")` returns `prefix`, and `gband.keystyle.current()` returns `floating`

#### Scenario: Saved style
- **WHEN** `user/keystyle.lua` holds `return "direct"` and `user/init.lua` calls `gband.keystyle.use()`
- **THEN** the call returns `direct` and the bindings are those of the direct preset

#### Scenario: Nothing saved
- **WHEN** no `user/keystyle.lua` exists and `user/init.lua` calls `gband.keystyle.use()`
- **THEN** the call returns `modal` and `prefix` is the mode `navigation`

#### Scenario: Current style without use
- **WHEN** `user/init.lua` holds only `require("gband.keystyle.floating")` and a callback calls `gband.keystyle.current()`
- **THEN** the call returns nil

#### Scenario: Current style after the saved style
- **WHEN** no `user/init.lua` exists, `user/keystyle.lua` holds `return "direct"`, and a callback calls `gband.keystyle.current()`
- **THEN** the call returns `direct`

#### Scenario: Unknown style
- **WHEN** line 3 of `user/init.lua` calls `gband.keystyle.use("vi")`
- **THEN** loading fails with an error at `user/init.lua` line 3 naming `vi`

#### Scenario: Used twice
- **WHEN** line 2 of `user/init.lua` calls `gband.keystyle.use()` and line 4 calls `gband.keystyle.use("direct")`
- **THEN** loading fails with an error at `user/init.lua` line 4

#### Scenario: Used after the load
- **WHEN** a binding function calls `gband.keystyle.use("modal")` on line 6 of `user/init.lua`
- **THEN** the client shows an error at `user/init.lua` line 6

#### Scenario: Change a preset binding
- **WHEN** `user/init.lua` calls `gband.keystyle.use()` and then binds `prefix q` to `gband.action.detach`
- **THEN** Ctrl+Space then `q` detaches, and no binding closes the window

### Requirement: Saved key style
The saved key style SHALL live in the file `user/keystyle.lua` of the configuration directory. Saving a style SHALL write the text `return "<style>"` followed by a newline to that file. It SHALL create the file or replace it in one step, so no process reads it half written. Saving SHALL change no other file in `user`, and SHALL never write `user/init.lua`. The settings window's `keys` line saves it, as the settings capability defines.

`gband.keystyle.saved()` SHALL evaluate `user/keystyle.lua` as a Lua chunk in text mode with an empty environment, and SHALL return the chunk's value when it is `"modal"`, `"direct"` or `"floating"`. It SHALL return nil when `gband.config_dir` is nil, and when the file is missing, cannot be read, does not compile, raises an error or returns any other value. It SHALL raise no error and report none. It SHALL be callable while the configuration loads and in any callback.

Loading SHALL evaluate `user/keystyle.lua` only through `gband.keystyle.saved()`, never as a configuration file, a plugin file or a module. Saving it SHALL reload the configuration, as the configuration capability's "Reload on change" defines for a file whose name ends in `.lua`. The reloaded configuration SHALL apply the saved style where it calls `gband.keystyle.use()` with no argument.

#### Scenario: Saved file
- **WHEN** the settings window saves the direct style
- **THEN** `user/keystyle.lua` holds `return "direct"` and a newline

#### Scenario: Written by hand
- **WHEN** `user/keystyle.lua` holds `return 'modal'` and a callback calls `gband.keystyle.saved()`
- **THEN** the call returns `modal`

#### Scenario: Floating saved
- **WHEN** `user/keystyle.lua` holds `return "floating"` and a callback calls `gband.keystyle.saved()`
- **THEN** the call returns `floating`

#### Scenario: Unknown saved value
- **WHEN** `user/keystyle.lua` holds `return "vi"`, no `user/init.lua` exists, and a client attaches
- **THEN** no error is reported and the modal style is in use

#### Scenario: Broken file is not a configuration error
- **WHEN** `user/keystyle.lua` holds `error("boom")` and `user/init.lua` calls `gband.keystyle.use()`
- **THEN** loading succeeds with no error and the modal style is in use

#### Scenario: Saving reloads the configuration
- **WHEN** a client is attached with no `user/init.lua`, the modal style is in use, and the settings window saves the direct style
- **THEN** within a second Ctrl+Space then `h` focuses the column to the left and the keys that follow reach the focused window

#### Scenario: User file untouched
- **WHEN** `user/init.lua` calls `gband.keystyle.use()` and binds `alt+h`, and the user picks the direct style in the settings window
- **THEN** `user/init.lua` keeps its content
- **AND** after the reload Alt+H still focuses the column to the left and the direct style's bindings apply

### Requirement: Preset parity
The modal and direct presets SHALL offer the same features. In `root` and in `prefix`, they SHALL bind the same key and mouse names, in the same order, each to the same action, or, for a binding to a Lua function, to a function that dispatches the same action. This requirement SHALL apply to the modal and direct presets only. The floating preset SHALL follow "Floating preset" instead.

The only difference allowed SHALL be in entering and leaving navigation mode, which only the modal style has:

- The modal preset's `escape` and `enter` in `prefix`, which return to interactive mode, SHALL have no binding in the direct preset, because Escape and Enter after the prefix key already end the sequence there.
- A modal binding that dispatches an action and then returns to interactive mode, such as `prefix`, or `n` while the settings capability's `I on new` setting is on, SHALL match a direct binding to that action alone, because every direct binding ends the sequence.
- A setting that only decides whether a modal binding returns to interactive mode SHALL have no effect on the direct preset.

A change that adds, removes or rebinds a name in the modal or the direct preset SHALL make the same change in the other of the two, or SHALL show it to be one of these differences.

#### Scenario: Same names in both presets
- **WHEN** `gband.keymap.list("root")` and `gband.keymap.list("prefix")` are read under the modal and the direct preset in turn
- **THEN** both `root` lists hold the same keys in the same order
- **AND** the `prefix` lists hold the same keys in the same order once the modal list's `escape` and `enter` are left out
- **AND** each pair of entries that both give an `action` names the same action

#### Scenario: Mouse after the prefix in both styles
- **WHEN** the modal or the direct preset is in use and the user presses Ctrl+Space and then drags a floating window 4 cells right with the left button
- **THEN** the floating window's box moves 4 cells right

### Requirement: Floating preset
`gband.keystyle.floating` SHALL declare no mode. It SHALL set up its plugins and make its bindings as the floating-key-style capability's "Floating bindings" defines, in that order, and register the handler that its "Leader" defines. `prefix` is then not a mode, and the prefix key SHALL open the window list instead of waiting for a second key, as that capability's "Leader" defines.

#### Scenario: Leader with the floating style
- **WHEN** no `user/init.lua` exists, `user/keystyle.lua` holds `return "floating"`, and the user presses Ctrl+Space
- **THEN** the window list is open and focused, and `root` is the active table

#### Scenario: Floating preset required by name
- **WHEN** `user/init.lua` holds only `require("gband.keystyle.floating")`
- **THEN** loading succeeds and `gband.keymap.list("root")` ends with `leftmouse` and `rightmouse`
