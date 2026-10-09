## MODIFIED Requirements

### Requirement: Floating bindings
The floating preset, `gband.keystyle.floating`, SHALL first set up, with `gband.plugin` and no options, the key list plugin `gband.keylist`, then the Lua prompt plugin `gband.prompt`, then the desktop plugin `gband.desktop`. It SHALL declare no mode and set no option. It SHALL then make these bindings in `prefix`, in this order. A row with a function binds a Lua function with the row's description. Every other row binds the action it names, with that action's description:

| key after the prefix | Lua binding | binds | description |
|---|---|---|---|
| `n` | `prefix n` | a function that opens a window running the user's shell, floating, in the viewed band, as `gband.action.open_window({ floating = true })` does | `open a floating window` |
| `?` | `prefix ?` | `keylist.open` | `list the keys` |
| `:` | `prefix :` | `prompt.open` | `run Lua` |
| `N` | `prefix N` | `prompt.rename` | `rename the window` |
| `s` | `prefix s` | the function `gband.settings.open` | `settings` |
| `!` | `prefix !` | `reload` | `reload the configuration` |
| `D` | `prefix D` | `detach` | `detach` |
| Ctrl+Space | `prefix prefix` | `send_prefix` | `send the prefix key to the focused window` |

It SHALL then make these bindings in `root`, in this order, each to the action it names, with that action's description:

| mouse name | action |
|---|---|
| `mod+leftmouse` | `drag_window` |
| `mod+rightmouse` | `drag_resize_window` |
| `mod+middlemouse` | `drag_band` |
| `mod+wheeldown` | `focus_band_down` |
| `mod+wheelup` | `focus_band_up` |
| `leftmouse` | `desktop.press` |
| `rightmouse` | `desktop.menu` |

It SHALL bind no key in `root` and no mouse name in `prefix`. Last, it SHALL register the handler of `KeyTableChanged` that "Leader" defines.

#### Scenario: Prefix bindings in order
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")` and reads `gband.keymap.list("prefix")`
- **THEN** the keys are `n`, `?`, `:`, `N`, `s`, `!`, `D` and `prefix`, in that order
- **AND** the entry for `n` has no action and the description `open a floating window`

#### Scenario: Root bindings in order
- **WHEN** the floating style is in use and `gband.keymap.list("root")` is read
- **THEN** it holds `mod+leftmouse`, `mod+rightmouse`, `mod+middlemouse`, `mod+wheeldown`, `mod+wheelup`, `leftmouse` and `rightmouse`, in that order, and nothing else
- **AND** the entry for `leftmouse` has the action `desktop.press` and the entry for `rightmouse` the action `desktop.menu`

#### Scenario: Plugins set up by the preset
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("floating")` and reads `gband.action.list()`
- **THEN** it holds `keylist.open`, `prompt.open` and `desktop.list`, and no `errors.open`
- **AND** no sidebar is drawn

#### Scenario: Alt drag still moves a window
- **WHEN** the floating style is in use, the screen area is 80×24, a floating window's box is 40×12 at column 20 and row 4, and the user drags from its content cell at column 30 and row 8 to column 36 and row 8 with the left button and Alt held
- **THEN** the box is at column 26 and row 4

