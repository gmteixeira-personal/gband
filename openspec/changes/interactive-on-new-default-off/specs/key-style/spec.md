## MODIFIED Requirements

### Requirement: Preset parity
The modal and direct presets SHALL offer the same features. In `root` and in `prefix`, they SHALL bind the same key and mouse names, in the same order, each to the same action, or, for a binding to a Lua function, to a function that dispatches the same action.

The only difference allowed SHALL be in entering and leaving navigation mode, which only the modal style has:

- The modal preset's `escape` and `enter` in `prefix`, which return to interactive mode, SHALL have no binding in the direct preset, because Escape and Enter after the prefix key already end the sequence there.
- A modal binding that dispatches an action and then returns to interactive mode, such as `prefix`, or `n` while the settings capability's `I on new` setting is on, SHALL match a direct binding to that action alone, because every direct binding ends the sequence.
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
