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
- **WHEN** the default configuration is in use, the client's terminal is 520 columns wide, and the user presses Ctrl+Space
- **THEN** the segment starts with `h left  l right  j down  k up  u band down  i band up  c center  n open a window  q close`
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

### Requirement: Hint labels
A hint's label SHALL be the first of these that applies:
1. For a binding to an action named in the `labels` option, that option's string. A binding to an action that `labels` maps to `false` SHALL be left out.
2. The binding's `desc`, when it is a non-empty string that differs from the description `gband.action.list()` gives its action.
3. For a built-in action, its short label in the table below.
4. The description `gband.action.list()` gives a registered action, when it is non-empty.
5. The action's full name.

A binding to a function SHALL be labelled by its `desc`, and SHALL be left out when its `desc` is nil or empty.

| action | short label |
|---|---|
| `focus_column_left` | `left` |
| `focus_column_right` | `right` |
| `focus_window_down` | `down` |
| `focus_window_up` | `up` |
| `focus_band_down` | `band down` |
| `focus_band_up` | `band up` |
| `center_column` | `center` |
| `open_window` | `new` |
| `close_window` | `close` |
| `consume_or_expel_left` | `stack left` |
| `consume_or_expel_right` | `stack right` |
| `move_column_left` | `move left` |
| `move_column_right` | `move right` |
| `move_window_down` | `move down` |
| `move_window_up` | `move up` |
| `toggle_window_floating` | `float` |
| `switch_focus_floating_tiled` | `layer` |
| `cycle_column_width` | `width` |
| `toggle_full_width` | `full` |
| `grow_column_width` | `wider` |
| `shrink_column_width` | `narrower` |
| `grow_window_height` | `taller` |
| `shrink_window_height` | `shorter` |
| `reset_window_height` | `reset height` |
| `detach` | `detach` |
| `send_prefix` | `send prefix` |

#### Scenario: Default description gives the short label
- **WHEN** the default configuration binds `prefix r` to `cycle_column_width` with the action's description
- **THEN** its hint shows `r width`

#### Scenario: Center label
- **WHEN** the default configuration binds `prefix c` to `center_column` with the action's description
- **THEN** its hint shows `c center`

#### Scenario: Own description
- **WHEN** `user/init.lua` binds `prefix g` to `gband.action.focus_column_left` with description `go west`
- **THEN** its hint shows `g go west`

#### Scenario: Label option
- **WHEN** the hints segment is set up with `{ labels = { close_window = "kill" } }` and the default bindings are in use
- **THEN** the hint for `q` shows `q kill`

#### Scenario: Label option hides an action
- **WHEN** the hints segment is set up with `{ labels = { detach = false } }`, the default bindings are in use, the segment is wide enough for every hint, and the user presses Ctrl+Space
- **THEN** no hint shows `detach`

#### Scenario: Registered action
- **WHEN** the plugin `hello` registers `greet` with description `say hi` and binds `alt+g` in `root` to it
- **THEN** the root hints show `C-space prefix  A-g say hi`

#### Scenario: Registered action without a description
- **WHEN** the plugin `hello` registers `greet` with no description and binds `alt+g` in `root` to it
- **THEN** its hint shows `A-g hello.greet`

#### Scenario: Function without a description
- **WHEN** `user/init.lua` binds `prefix x` to a Lua function with no description
- **THEN** the prefix table's hints hold no hint for `x`

#### Scenario: Floating keys
- **WHEN** the default bindings are in use, the segment is wide enough for every hint, and the user presses Ctrl+Space
- **THEN** the hints include `v float`, `V layer`, `C-h move left` and `C-left move left`

### Requirement: Default setup
The default configuration SHALL set up `gband.statusline.hints` with `gband.plugin` and no options, after `gband.statusline.mode` and before `gband.statusline.position`.

#### Scenario: Hints in the default line
- **WHEN** no `user/init.lua` exists and a client attaches with an 80×24 terminal
- **THEN** row 23 shows `band 1`, the separator, then `C-space navigation`, and the position ending at column 79
