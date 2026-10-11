## MODIFIED Requirements

### Requirement: Key style presets
gband SHALL bundle three key style presets, the modules `gband.keystyle.modal`, `gband.keystyle.direct` and `gband.keystyle.floating`, which `require` finds as the plugins capability defines for bundled modules. Each preset SHALL be a file of top-level calls that uses only the `gband` API a configuration file can use. Requiring a preset while the configuration loads SHALL make its bindings in that configuration. A preset SHALL set no option. In `root`, the modal and direct presets SHALL bind only the mouse names that the client-attach capability's "Default mouse bindings" gives for `root`, and no key. In `root`, the floating preset SHALL bind those mouse names, then `leftmouse` and `rightmouse`, as the floating-key-style capability's "Floating bindings" defines, and no key.

Before its bindings, each preset SHALL set up, with `gband.plugin` and no options, each bundled plugin that registers an action it binds: the key list plugin `gband.keylist`, then the Lua prompt plugin `gband.prompt`, and, for the floating preset only, then the desktop plugin `gband.desktop`. It SHALL set up no other plugin, so the error list plugin `gband.errors` and the sidebar plugin `gband.sidebar` stay with the default configuration, as the configuration capability defines. The modal and direct presets SHALL bind `e` and `E` in `prefix` to Lua functions that look up `errors.open` and `errors.clear` in `gband.action` when the key is pressed, as the client-attach capability's default table defines, so the binding works whether `gband.errors` is set up after the preset, before it, or not at all. A binding to an action SHALL take the description of the action it binds, as the actions capability lists them or as `gband.action.list()` gives it for a registered action. A binding to a Lua function SHALL take the description that the client-attach capability's default table, or for the floating preset the floating-key-style capability's "Floating bindings", gives its key.

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

#### Scenario: Error list set up after the preset
- **WHEN** `user/init.lua` calls `gband.keystyle.use("modal")`, then `gband.plugin("gband.errors", { kind = "tiled" })`, and the user presses Ctrl+Space then `e`
- **THEN** loading succeeds and the error list opens as a tiled plugin window

