# key-hints Specification

## Purpose

Defines the key hints segment bundled with gband: a status line component that shows the keys of the active key table and what each one does, as the hint bar of zellij does, fitted to the width the status line leaves it.

## Requirements

### Requirement: Hints segment plugin
gband SHALL bundle the plugin module `gband.statusline.hints`, whose plugin is named `hints`. Its `setup` SHALL add one component under the plugin's name with `gband.ui.statusline.add`, with these settings:

| field | value |
|---|---|
| `align` | `"left"` |
| `priority` | `0` |
| `order` | `30` |
| `hl` | `"KeyHintLabel"` |
| `redraw_on` | `KeyTableChanged` |
| `fill` | true |

`setup` SHALL take the options `align`, `priority` and `order`, which replace the defaults in the table, `labels`, a table from action names to strings or `false`, and `root`, a boolean that is true by default. An option of the wrong type or value SHALL make `setup` raise an error.

#### Scenario: Component entry
- **WHEN** `user/init.lua` calls `gband.plugin("gband.statusline.hints")` and reads `gband.ui.statusline.list()`
- **THEN** it holds an entry with `id` `hints`, `align` `left`, `priority` 0, `order` 30, `hl` `KeyHintLabel` and `plugin` `hints`

#### Scenario: Invalid option
- **WHEN** `user/init.lua` calls `gband.plugin("gband.statusline.hints", { labels = 3 })`
- **THEN** `gband.plugin` returns `false` and a plugin error names `hints`

### Requirement: Hints of the active table
The segment SHALL show one hint for each binding of the active key table, in the order `gband.keymap.list` returns them. While `root` is active, the hints SHALL start with one hint for the prefix key labelled `prefix`, shown only when the `prefix` table holds a binding, followed by the bindings of `root`. While `root` is active and the `root` option is false, the segment SHALL be hidden. A binding left out by "Hint labels" SHALL take no hint. When no hint is left, the segment SHALL be hidden.

Each hint SHALL be the key, as "Key form" shows it, in the group `KeyHintKey`, one space, and the label in the group `KeyHintLabel`. Hints SHALL be separated by two spaces in `KeyHintLabel`.

#### Scenario: Root with the defaults
- **WHEN** the default configuration is in use and `root` is active
- **THEN** the left region shows `band 1`, the separator, then `C-space prefix`

#### Scenario: Prefix table with the defaults
- **WHEN** the default configuration is in use, the client's terminal is 240 columns wide, and the user presses Ctrl+Space
- **THEN** the segment starts with `h left  l right  j down  k up  u band down  i band up`
- **AND** ends with `D detach  C-space send prefix`

#### Scenario: Back to root
- **WHEN** the prefix table's hints are shown and the user presses `h`
- **THEN** the segment shows `C-space prefix` again

#### Scenario: Named table
- **WHEN** `user/init.lua` binds `h` with description `west` and `l` with description `east` in the table `move`, and a binding enters `move`
- **THEN** the segment shows `h west  l east`

#### Scenario: No prefix bindings
- **WHEN** `user/init.lua` binds only `alt+h` in `root` to `gband.action.focus_column_left` and sets up the hints segment
- **THEN** the segment shows `A-h left`

#### Scenario: Root hints turned off
- **WHEN** the hints segment is set up with `{ root = false }` and `root` is active
- **THEN** the segment is hidden
- **AND** it shows the prefix table's hints after Ctrl+Space

#### Scenario: Groups
- **WHEN** the segment shows `C-space prefix`
- **THEN** `C-space` is drawn in `KeyHintKey` and ` prefix` in `KeyHintLabel`

### Requirement: Key form
A hint SHALL show its key in a short form. The modifiers SHALL be shown as `C-` for Ctrl, `A-` for Alt and `S-` for Shift, in that order, before the key. `shift` with a lowercase letter SHALL be shown as the uppercase letter, with no `S-`. A named key SHALL be shown in lowercase, with `escape` shown as `esc`. A one-character key SHALL be shown as written. The key `prefix` in a table other than `root`, and the prefix key's hint in `root`, SHALL be shown as the key the `prefix` option names, in the same form.

#### Scenario: Control and a named key
- **WHEN** a binding's key is `ctrl+space`
- **THEN** its hint shows `C-space`

#### Scenario: Shifted letter
- **WHEN** a binding's key is `shift+d`
- **THEN** its hint shows `D`

#### Scenario: Case of a named key
- **WHEN** a binding's key is `Alt+PageUp`
- **THEN** its hint shows `A-pageup`

#### Scenario: Plus as the key
- **WHEN** a binding's key is `alt++`
- **THEN** its hint shows `A-+`

#### Scenario: Prefix in a named table
- **WHEN** the `prefix` option is `"ctrl+b"`, the table `move` binds `prefix` to a function with description `back`, and `move` is active
- **THEN** the segment shows `C-b back`

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
- **WHEN** the hints segment is set up with `{ labels = { send_prefix = false } }` and the user presses Ctrl+Space
- **THEN** no hint shows `send prefix`

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

### Requirement: Fit to the available width
The segment SHALL show the longest leading run of its hints that fits in its context's `width`. When hints are left out, the run SHALL be followed by a space and `…` in `KeyHintLabel`, and the run with them SHALL fit in `width`. When no hint fits, the segment SHALL be hidden. Widths SHALL be display widths, as `gband.ui.width` measures them.

#### Scenario: Some hints left out
- **WHEN** the prefix table's default hints are shown and the segment's context has `width` 20
- **THEN** the segment shows `h left  l right …`

#### Scenario: All hints fit
- **WHEN** the active table `move` has the hints `h west  l east` and the segment's context has `width` 14
- **THEN** the segment shows `h west  l east`

#### Scenario: Nothing fits
- **WHEN** the segment's context has `width` 3 and the first hint is `C-space prefix`
- **THEN** the segment is hidden

### Requirement: Hint groups
The segment SHALL give the group `KeyHintKey` the default `{ link = "StatusLineAccent" }` and the group `KeyHintLabel` the default `{ link = "StatusLineSegment" }`, as the highlights capability defines defaults, when its module is first required.

#### Scenario: Linked defaults
- **WHEN** the hints segment is set up and no colorscheme or user setting names `KeyHintKey`
- **THEN** `KeyHintKey` resolves to the style of `StatusLineAccent`

#### Scenario: Theme override
- **WHEN** the active colorscheme sets `KeyHintKey` to `{ fg = "yellow" }`
- **THEN** the keys of the hints are drawn in yellow

### Requirement: Default setup
The default configuration SHALL set up `gband.statusline.hints` with `gband.plugin` and no options, after `gband.statusline.mode` and before `gband.statusline.position`.

#### Scenario: Hints in the default line
- **WHEN** no `user/init.lua` exists and a client attaches with an 80×24 terminal
- **THEN** row 23 shows `band 1`, the separator, then `C-space prefix`, and the position ending at column 79
