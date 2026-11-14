## MODIFIED Requirements

### Requirement: Modal preset
`gband.keystyle.modal` SHALL declare `prefix` a mode with the label `navigation`, with `gband.keymap.mode`. It SHALL make the bindings of the modal key style that the client-attach capability's default table gives, in that order. It SHALL make the `prefix` bindings by calling the bundled module `gband.navigation`'s `bind("prefix")`, as "Navigation bindings" defines.

#### Scenario: Navigation mode
- **WHEN** `user/init.lua` calls `gband.keystyle.use("modal")`, the first of three columns is focused, and the user presses Ctrl+Space, `l`, then `l`
- **THEN** the third column is focused and no window receives a key
- **AND** `gband.keymap.label("prefix")` returns `navigation`

#### Scenario: Escape returns to typing
- **WHEN** the modal preset is in use and the user presses Ctrl+Space, `h`, Escape, then types `echo left` and Enter, with the second of two columns focused
- **THEN** `left` appears in the first tile only

### Requirement: Direct preset
`gband.keystyle.direct` SHALL declare the table `navigation` a mode with the label `navigation`, with `gband.keymap.mode`, and SHALL declare no other mode. It SHALL make the bindings of the direct key style that the client-attach capability defines, in that order. It SHALL make the `navigation` bindings by calling `gband.navigation`'s `bind("navigation")`, as "Navigation bindings" defines. `prefix` is not a mode, so each key after the prefix key SHALL end the key sequence, and the keys that follow SHALL go to the focused plugin window or window. Space after the prefix key SHALL make `navigation` active, and it SHALL then stay active as any mode does.

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

#### Scenario: Navigation mode with the direct style
- **WHEN** the direct preset is in use, the first of three columns is focused, and the user presses Ctrl+Space, Space, `l`, then `l`
- **THEN** the third column is focused and no window receives a key
- **AND** `navigation` is active and `gband.keymap.label("navigation")` returns `navigation`

#### Scenario: Escape leaves direct navigation mode
- **WHEN** the direct preset is in use, the second of two columns is focused, and the user presses Ctrl+Space, Space, `h`, Escape, then types `echo left` and Enter
- **THEN** `root` is active and `left` appears in the first tile only

#### Scenario: Sidebar and key list in direct navigation mode
- **WHEN** the default configuration is in use with the direct key style saved, and the user presses Ctrl+Space then Space
- **THEN** the sidebar's mode row shows `N`
- **AND** after `?`, the key list's title is `navigation keys` and it lists the `navigation` table's keys

#### Scenario: Prefix key leaves direct navigation mode
- **WHEN** the direct preset is in use, navigation mode is active, and the user presses Ctrl+Space
- **THEN** the focused window receives Ctrl+Space and `root` is active

#### Scenario: Agents mode from direct navigation mode
- **WHEN** the direct preset is in use, `gband.agents.list()` holds entries for windows 3 and 2, navigation mode is active, and the user presses `a` then `2`
- **THEN** window 2 is focused and `root` is active

### Requirement: Preset parity
The modal and direct presets SHALL offer the same features. In `root` and in `prefix`, they SHALL bind the same key and mouse names, in the same order, each to the same action, or, for a binding to a Lua function, to a function that dispatches the same action. This requirement SHALL apply to the modal and direct presets only. The floating preset SHALL follow "Floating preset" instead.

The only differences allowed SHALL be in entering and leaving navigation mode, which the modal style enters with the prefix key and the direct style with the prefix key then Space:

- The modal preset's `escape` and `enter` in `prefix`, which return to interactive mode, SHALL have no binding in the direct preset's `prefix`, because Escape and Enter after the prefix key already end the sequence there.
- The direct preset's `space` in `prefix`, which enters navigation mode, SHALL have no binding in the modal preset's `prefix`, because the prefix key already enters navigation mode there. It SHALL take the place of the modal preset's `escape` and `enter` in the order.
- A modal binding that dispatches an action and then returns to interactive mode, such as `prefix`, or `n` while the settings capability's `I on new` setting is on, SHALL match a direct `prefix` binding to that action alone, because every direct `prefix` binding ends the sequence.
- A setting that only decides whether a navigation mode binding returns to interactive mode SHALL have no effect on the direct preset's `prefix` table.

The direct preset's `navigation` table SHALL equal the modal preset's `prefix` table: the same key and mouse names, in the same order, each with the same action and description, and each Lua function behaving the same, including whether it returns to interactive mode.

A change that adds, removes or rebinds a name in the modal or the direct preset SHALL make the same change in the other of the two, or SHALL show it to be one of these differences.

#### Scenario: Same names in both presets
- **WHEN** `gband.keymap.list("root")` and `gband.keymap.list("prefix")` are read under the modal and the direct preset in turn
- **THEN** both `root` lists hold the same keys in the same order
- **AND** the `prefix` lists hold the same keys in the same order once the modal list's `escape` and `enter` and the direct list's `space` are left out
- **AND** each pair of entries that both give an `action` names the same action

#### Scenario: Mouse after the prefix in both styles
- **WHEN** the modal or the direct preset is in use and the user presses Ctrl+Space and then drags a floating window 4 cells right with the left button
- **THEN** the floating window's box moves 4 cells right

#### Scenario: Direct navigation equals modal navigation
- **WHEN** `gband.keymap.list("prefix")` is read under the modal preset and `gband.keymap.list("navigation")` under the direct preset
- **THEN** both lists hold the same keys in the same order, with the same `action` and `desc` in each pair of entries

#### Scenario: I on new in direct navigation mode
- **WHEN** the direct preset is in use, `user/interactive_on_new.lua` holds `return false`, and the user presses Ctrl+Space, Space, then `n`
- **THEN** a second window opens and is focused
- **AND** navigation mode stays active

## ADDED Requirements

### Requirement: Navigation bindings
gband SHALL bundle the client module `gband.navigation`. Its `bind(table)` SHALL make, in the key table named `table`, the bindings that the client-attach capability's default table gives for navigation mode, in that order, and nothing else. It SHALL declare no mode. It SHALL be callable only while the configuration loads. A `table` that is not a non-empty string, or `root`, SHALL be an error at the line of the call.

#### Scenario: Bind into a named table
- **WHEN** `user/init.lua` calls `gband.keymap.mode("nav", { label = "nav" })`, `gband.navigation.bind("nav")`, binds `prefix space` to a function that enters `nav`, and the user presses Ctrl+Space, Space, `l`, then `l` with the first of three columns focused
- **THEN** the third column is focused and `nav` is active

#### Scenario: Root refused
- **WHEN** `user/init.lua` calls `gband.navigation.bind("root")`
- **THEN** loading reports an error at the line of the call
