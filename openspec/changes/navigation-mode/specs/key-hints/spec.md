## MODIFIED Requirements

### Requirement: Hints of the active table
The segment SHALL show one hint for each binding of the active key table, in the order `gband.keymap.list` returns them. While `root` is active, the hints SHALL start with one hint for the prefix key, labelled with `gband.keymap.label("prefix")`, shown only when the `prefix` table holds a binding, followed by the bindings of `root`. While `root` is active and the `root` option is false, the segment SHALL be hidden. A binding left out by "Hint labels" SHALL take no hint. When no hint is left, the segment SHALL be hidden.

Each hint SHALL be the key, as "Key form" shows it, in the group `KeyHintKey`, one space, and the label in the group `KeyHintLabel`. Hints SHALL be separated by two spaces in `KeyHintLabel`.

#### Scenario: Root with the defaults
- **WHEN** the default configuration is in use and `root` is active
- **THEN** the left region shows `band 1`, the separator, then `C-space navigation`

#### Scenario: Prefix table that is not a mode
- **WHEN** `user/init.lua` binds `prefix h` to `gband.action.focus_column_left`, declares no mode, sets up the hints segment, and `root` is active
- **THEN** the segment shows `C-space prefix`

#### Scenario: Prefix table with the defaults
- **WHEN** the default configuration is in use, the client's terminal is 400 columns wide, and the user presses Ctrl+Space
- **THEN** the segment starts with `h left  l right  j down  k up  u band down  i band up  n open a window  q close`
- **AND** ends with `D detach  esc interactive mode  enter interactive mode  left left  right right  down down  up up  C-space send the prefix key`

#### Scenario: Hints stay in navigation mode
- **WHEN** navigation mode's hints are shown and the user presses `h`
- **THEN** the segment still shows navigation mode's hints

#### Scenario: Back to root
- **WHEN** navigation mode's hints are shown and the user presses Escape
- **THEN** the segment shows `C-space navigation` again

#### Scenario: Named table
- **WHEN** `user/init.lua` binds `h` with description `west` and `l` with description `east` in the table `move`, and a binding enters `move`
- **THEN** the segment shows `h west  l east`

#### Scenario: No prefix bindings
- **WHEN** `user/init.lua` binds only `alt+h` in `root` to `gband.action.focus_column_left` and sets up the hints segment
- **THEN** the segment shows `A-h left`

#### Scenario: Root hints turned off
- **WHEN** the hints segment is set up with `{ root = false }` and `root` is active
- **THEN** the segment is hidden
- **AND** it shows navigation mode's hints after Ctrl+Space

#### Scenario: Groups
- **WHEN** the segment shows `C-space navigation`
- **THEN** `C-space` is drawn in `KeyHintKey` and ` navigation` in `KeyHintLabel`

### Requirement: Default setup
The default configuration SHALL set up `gband.statusline.hints` with `gband.plugin` and no options, after `gband.statusline.mode` and before `gband.statusline.position`.

#### Scenario: Hints in the default line
- **WHEN** no `user/init.lua` exists and a client attaches with an 80×24 terminal
- **THEN** row 23 shows `band 1`, the separator, then `C-space navigation`, and the position ending at column 79
