## MODIFIED Requirements

### Requirement: Key style presets
gband SHALL bundle two key style presets, the modules `gband.keystyle.modal` and `gband.keystyle.direct`, which `require` finds as the plugins capability defines for bundled modules. Each preset SHALL be a file of top-level calls that uses only the `gband` API a configuration file can use. Requiring a preset while the configuration loads SHALL make its bindings in that configuration. A preset SHALL set no option. In `root`, a preset SHALL bind only the mouse names that the client-attach capability's "Default mouse bindings" gives for `root`, and no key.

Before its bindings, each preset SHALL set up, with `gband.plugin` and no options, each bundled plugin that registers an action it binds: the key list plugin `gband.keylist`, then the Lua prompt plugin `gband.prompt`. It SHALL set up no other plugin, so the error list plugin `gband.errors`, whose action no preset binds, and the sidebar plugin `gband.sidebar` stay with the default configuration, as the configuration capability defines. A binding to an action SHALL take the description of the action it binds, as the actions capability lists them or as `gband.action.list()` gives it for a registered action. A binding to a Lua function SHALL take the description that the client-attach capability's default table gives its key.

Each action that a preset binds to `h`, `j`, `k` or `l`, alone or with Ctrl, SHALL also be bound to Left, Down, Up or Right, with the same modifier.

#### Scenario: Preset required by name
- **WHEN** `user/init.lua` holds only `require("gband.keystyle.direct")`, and the user presses Ctrl+Space then `h` with the second of two columns focused
- **THEN** loading succeeds and the first column is focused

#### Scenario: Preset shadowed by a user module
- **WHEN** `user/lua/gband/keystyle/direct.lua` binds only `prefix x`, and `user/init.lua` calls `gband.keystyle.use("direct")`
- **THEN** `gband.keymap.list("prefix")` holds only the entry for `x`

#### Scenario: Arrow keys beside hjkl
- **WHEN** either preset is in use and `gband.keymap.list("prefix")` is read
- **THEN** `left` binds the action `h` binds, and `ctrl+up` binds the action `ctrl+k` binds

#### Scenario: Bundled plugins set up by the preset
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("modal")`, and the user presses Ctrl+Space then `?`
- **THEN** loading succeeds and the key list opens
- **AND** `gband.action.list()` holds `keylist.open` and `prompt.open`, and no `errors.open`
- **AND** no sidebar is drawn

#### Scenario: Only mouse names in root
- **WHEN** either preset is in use and `gband.keymap.list("root")` is read
- **THEN** it holds `mod+leftmouse`, `mod+rightmouse`, `mod+middlemouse`, `mod+wheeldown` and `mod+wheelup`, in that order, and nothing else

#### Scenario: Remove a preset mouse binding
- **WHEN** `user/init.lua` calls `gband.keystyle.use()` and then `gband.keymap.del("root", "mod+leftmouse")`, and the user drags across a program that reports the mouse with the left button and Alt held
- **THEN** no window moves and the program receives the press

## ADDED Requirements

### Requirement: Preset parity
The modal and direct presets SHALL offer the same features. In `root` and in `prefix`, they SHALL bind the same key and mouse names, in the same order, each to the same action, or, for a binding to a Lua function, to a function that dispatches the same action.

The only difference allowed SHALL be in entering and leaving navigation mode, which only the modal style has:

- The modal preset's `escape` and `enter` in `prefix`, which return to interactive mode, SHALL have no binding in the direct preset, because Escape and Enter after the prefix key already end the sequence there.
- A modal binding that dispatches an action and then returns to interactive mode, such as `n` and `prefix`, SHALL match a direct binding to that action alone, because every direct binding ends the sequence.
- A setting that only decides whether a modal binding returns to interactive mode SHALL have no effect on the direct preset.

A change that adds, removes or rebinds a name in one preset SHALL make the same change in the other, or SHALL show it to be one of these differences.

#### Scenario: Same names in both presets
- **WHEN** `gband.keymap.list("root")` and `gband.keymap.list("prefix")` are read under each preset in turn
- **THEN** both `root` lists hold the same keys in the same order
- **AND** the `prefix` lists hold the same keys in the same order once the modal list's `escape` and `enter` are left out
- **AND** each pair of entries that both give an `action` names the same action

#### Scenario: Mouse after the prefix in both styles
- **WHEN** either preset is in use and the user presses Ctrl+Space and then drags a floating window 4 cells right with the left button
- **THEN** the floating window's box moves 4 cells right
